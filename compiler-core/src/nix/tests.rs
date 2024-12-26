use crate::ast::TypedModule;
use crate::config::PackageConfig;
use crate::type_::PRELUDE_MODULE_NAME;
use crate::{
    analyse::TargetSupport,
    build::{Origin, Target},
    nix::*,
    uid::UniqueIdGenerator,
    warning::{TypeWarningEmitter, WarningEmitter},
};
use camino::{Utf8Path, Utf8PathBuf};

mod assignments;
mod basic;
mod bit_arrays;
mod blocks;
mod bools;
mod case;
mod case_clause_guards;
mod consts;
mod custom_types;
mod externals;
mod functions;
mod lists;
mod modules;
mod numbers;
mod panic;
mod prelude;
mod records;
mod recursion;
mod results;
mod strings;
mod todo;
mod tuples;
mod type_alias;
mod use_;

pub static CURRENT_PACKAGE: &str = "thepackage";

#[macro_export]
macro_rules! assert_nix_with_multiple_imports {
    ($(($name:literal, $module_src:literal)),+; $src:literal) => {
        let compiled =
            $crate::nix::tests::compile_nix($src, vec![$((CURRENT_PACKAGE, $name, $module_src)),*]).expect("compilation failed");
        let mut output = String::from("----- SOURCE CODE\n");
        for (name, src) in [$(($name, $module_src)),*] {
            output.push_str(&format!("-- {name}.gleam\n{src}\n\n"));
        }
        output.push_str(&format!("-- main.gleam\n{}\n\n----- COMPILED NIX\n{compiled}", $src));
        insta::assert_snapshot!(insta::internals::AutoName, output, $src);
    };
}

#[macro_export]
macro_rules! assert_nix {
    (($dep_package:expr, $dep_name:expr, $dep_src:expr), $src:expr $(,)?) => {{
        let compiled =
            $crate::nix::tests::compile_nix($src, vec![($dep_package, $dep_name, $dep_src)])
                .expect("compilation failed");
        let output = format!(
            "----- SOURCE CODE\n{}\n\n----- COMPILED NIX\n{}",
            $src, compiled
        );
        insta::assert_snapshot!(insta::internals::AutoName, output, $src);
    }};

    (($dep_package:expr, $dep_name:expr, $dep_src:expr), $src:expr, $nix:expr $(,)?) => {{
        let compiled =
            $crate::nix::tests::compile_nix($src, Some(($dep_package, $dep_name, $dep_src)))
                .expect("compilation failed");
        let output = format!(
            "----- SOURCE CODE\n{}\n\n----- COMPILED NIX\n{}",
            $src, compiled
        );
        assert_eq!(($src, output), ($src, $nix.to_string()));
    }};

    ($src:expr $(,)?) => {{
        let output = $crate::nix::tests::compile_nix($src, vec![]).expect("compilation failed");
        insta::assert_snapshot!(insta::internals::AutoName, output, $src);
    }};

    ($src:expr, $js:expr $(,)?) => {{
        let output = $crate::nix::tests::compile_nix($src, vec![]).expect("compilation failed");
        assert_eq!(($src, output), ($src, $js.to_string()));
    }};
}

#[macro_export]
macro_rules! assert_nix_error {
    ($src:expr $(,)?) => {{
        let error = $crate::nix::tests::expect_nix_error($src, vec![]);
        let output = format!("----- SOURCE CODE\n{}\n\n----- ERROR\n{}", $src, error);
        insta::assert_snapshot!(insta::internals::AutoName, output, $src);
    }};
}

pub fn compile(src: &str, deps: Vec<(&str, &str, &str)>) -> TypedModule {
    let mut modules = im::HashMap::new();
    let ids = UniqueIdGenerator::new();
    // DUPE: preludeinsertion
    // TODO: Currently we do this here and also in the tests. It would be better
    // to have one place where we create all this required state for use in each
    // place.
    let _ = modules.insert(
        PRELUDE_MODULE_NAME.into(),
        crate::type_::build_prelude(&ids),
    );
    let mut direct_dependencies = std::collections::HashMap::from_iter(vec![]);

    deps.iter().for_each(|(dep_package, dep_name, dep_src)| {
        let mut dep_config = PackageConfig::default();
        dep_config.name = (*dep_package).into();
        let parsed = crate::parse::parse_module(
            Utf8PathBuf::from("test/path"),
            dep_src,
            &WarningEmitter::null(),
        )
        .expect("dep syntax error");
        let mut ast = parsed.module;
        ast.name = (*dep_name).into();
        let line_numbers = LineNumbers::new(dep_src);

        let dep = crate::analyse::ModuleAnalyzerConstructor::<()> {
            target: Target::Nix,
            ids: &ids,
            origin: Origin::Src,
            importable_modules: &modules,
            warnings: &TypeWarningEmitter::null(),
            direct_dependencies: &std::collections::HashMap::new(),
            target_support: TargetSupport::Enforced,
            package_config: &dep_config,
        }
        .infer_module(ast, line_numbers, "".into())
        .expect("should successfully infer");
        let _ = modules.insert((*dep_name).into(), dep.type_info);
        let _ = direct_dependencies.insert((*dep_package).into(), ());
    });

    let parsed =
        crate::parse::parse_module(Utf8PathBuf::from("test/path"), src, &WarningEmitter::null())
            .expect("syntax error");
    let mut ast = parsed.module;
    ast.name = "my/mod".into();
    let line_numbers = LineNumbers::new(src);
    let mut config = PackageConfig::default();
    config.name = "thepackage".into();

    crate::analyse::ModuleAnalyzerConstructor::<()> {
        target: Target::Nix,
        ids: &ids,
        origin: Origin::Src,
        importable_modules: &modules,
        warnings: &TypeWarningEmitter::null(),
        direct_dependencies: &direct_dependencies,
        target_support: TargetSupport::NotEnforced,
        package_config: &config,
    }
    .infer_module(ast, line_numbers, "".into())
    .expect("should successfully infer")
}

pub fn compile_nix(src: &str, deps: Vec<(&str, &str, &str)>) -> Result<String, crate::Error> {
    let ast = compile(src, deps);
    let line_numbers = LineNumbers::new(src);
    module(
        &ast,
        &line_numbers,
        Utf8Path::new(""),
        &"".into(),
        TargetSupport::Enforced,
    )
}

pub fn expect_nix_error(src: &str, deps: Vec<(&str, &str, &str)>) -> String {
    let error = compile_nix(src, deps).expect_err("should not compile");
    println!("er: {error:#?}");
    let better_error = match error {
        crate::Error::Nix {
            error: inner_error, ..
        } => crate::Error::Nix {
            src: src.into(),
            path: Utf8PathBuf::from("/src/nix/error.gleam"),
            error: inner_error,
        },
        _ => panic!("expected nix error, got {error:#?}"),
    };
    better_error.pretty_string()
}
