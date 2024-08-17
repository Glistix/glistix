//! This file is generated by build.rs
//! Do not edit it directly, instead add new test cases to ./cases

#[rustfmt::skip]
#[test]
fn alias_unqualified_import() {
    let output = crate::prepare("./cases/alias_unqualified_import");
    insta::assert_snapshot!(
        "alias_unqualified_import",
        output,
        "./cases/alias_unqualified_import",
    );
}

#[rustfmt::skip]
#[test]
fn duplicate_module() {
    let output = crate::prepare("./cases/duplicate_module");
    insta::assert_snapshot!(
        "duplicate_module",
        output,
        "./cases/duplicate_module",
    );
}

#[rustfmt::skip]
#[test]
fn erlang_app_generation() {
    let output = crate::prepare("./cases/erlang_app_generation");
    insta::assert_snapshot!(
        "erlang_app_generation",
        output,
        "./cases/erlang_app_generation",
    );
}

#[rustfmt::skip]
#[test]
fn erlang_bug_752() {
    let output = crate::prepare("./cases/erlang_bug_752");
    insta::assert_snapshot!(
        "erlang_bug_752",
        output,
        "./cases/erlang_bug_752",
    );
}

#[rustfmt::skip]
#[test]
fn erlang_empty() {
    let output = crate::prepare("./cases/erlang_empty");
    insta::assert_snapshot!(
        "erlang_empty",
        output,
        "./cases/erlang_empty",
    );
}

#[rustfmt::skip]
#[test]
fn erlang_escape_names() {
    let output = crate::prepare("./cases/erlang_escape_names");
    insta::assert_snapshot!(
        "erlang_escape_names",
        output,
        "./cases/erlang_escape_names",
    );
}

#[rustfmt::skip]
#[test]
fn erlang_import() {
    let output = crate::prepare("./cases/erlang_import");
    insta::assert_snapshot!(
        "erlang_import",
        output,
        "./cases/erlang_import",
    );
}

#[rustfmt::skip]
#[test]
fn erlang_import_shadowing_prelude() {
    let output = crate::prepare("./cases/erlang_import_shadowing_prelude");
    insta::assert_snapshot!(
        "erlang_import_shadowing_prelude",
        output,
        "./cases/erlang_import_shadowing_prelude",
    );
}

#[rustfmt::skip]
#[test]
fn erlang_nested() {
    let output = crate::prepare("./cases/erlang_nested");
    insta::assert_snapshot!(
        "erlang_nested",
        output,
        "./cases/erlang_nested",
    );
}

#[rustfmt::skip]
#[test]
fn erlang_nested_qualified_constant() {
    let output = crate::prepare("./cases/erlang_nested_qualified_constant");
    insta::assert_snapshot!(
        "erlang_nested_qualified_constant",
        output,
        "./cases/erlang_nested_qualified_constant",
    );
}

#[rustfmt::skip]
#[test]
fn hello_joe() {
    let output = crate::prepare("./cases/hello_joe");
    insta::assert_snapshot!(
        "hello_joe",
        output,
        "./cases/hello_joe",
    );
}

#[rustfmt::skip]
#[test]
fn import_cycle() {
    let output = crate::prepare("./cases/import_cycle");
    insta::assert_snapshot!(
        "import_cycle",
        output,
        "./cases/import_cycle",
    );
}

#[rustfmt::skip]
#[test]
fn import_cycle_multi() {
    let output = crate::prepare("./cases/import_cycle_multi");
    insta::assert_snapshot!(
        "import_cycle_multi",
        output,
        "./cases/import_cycle_multi",
    );
}

#[rustfmt::skip]
#[test]
fn import_shadowed_name_warning() {
    let output = crate::prepare("./cases/import_shadowed_name_warning");
    insta::assert_snapshot!(
        "import_shadowed_name_warning",
        output,
        "./cases/import_shadowed_name_warning",
    );
}

#[rustfmt::skip]
#[test]
fn imported_constants() {
    let output = crate::prepare("./cases/imported_constants");
    insta::assert_snapshot!(
        "imported_constants",
        output,
        "./cases/imported_constants",
    );
}

#[rustfmt::skip]
#[test]
fn imported_external_fns() {
    let output = crate::prepare("./cases/imported_external_fns");
    insta::assert_snapshot!(
        "imported_external_fns",
        output,
        "./cases/imported_external_fns",
    );
}

#[rustfmt::skip]
#[test]
fn imported_record_constructors() {
    let output = crate::prepare("./cases/imported_record_constructors");
    insta::assert_snapshot!(
        "imported_record_constructors",
        output,
        "./cases/imported_record_constructors",
    );
}

#[rustfmt::skip]
#[test]
fn javascript_d_ts() {
    let output = crate::prepare("./cases/javascript_d_ts");
    insta::assert_snapshot!(
        "javascript_d_ts",
        output,
        "./cases/javascript_d_ts",
    );
}

#[rustfmt::skip]
#[test]
fn javascript_empty() {
    let output = crate::prepare("./cases/javascript_empty");
    insta::assert_snapshot!(
        "javascript_empty",
        output,
        "./cases/javascript_empty",
    );
}

#[rustfmt::skip]
#[test]
fn javascript_import() {
    let output = crate::prepare("./cases/javascript_import");
    insta::assert_snapshot!(
        "javascript_import",
        output,
        "./cases/javascript_import",
    );
}

#[rustfmt::skip]
#[test]
fn not_overwriting_erlang_module() {
    let output = crate::prepare("./cases/not_overwriting_erlang_module");
    insta::assert_snapshot!(
        "not_overwriting_erlang_module",
        output,
        "./cases/not_overwriting_erlang_module",
    );
}

#[rustfmt::skip]
#[test]
fn opaque_type_accessor() {
    let output = crate::prepare("./cases/opaque_type_accessor");
    insta::assert_snapshot!(
        "opaque_type_accessor",
        output,
        "./cases/opaque_type_accessor",
    );
}

#[rustfmt::skip]
#[test]
fn opaque_type_destructure() {
    let output = crate::prepare("./cases/opaque_type_destructure");
    insta::assert_snapshot!(
        "opaque_type_destructure",
        output,
        "./cases/opaque_type_destructure",
    );
}

#[rustfmt::skip]
#[test]
fn overwriting_erlang_module() {
    let output = crate::prepare("./cases/overwriting_erlang_module");
    insta::assert_snapshot!(
        "overwriting_erlang_module",
        output,
        "./cases/overwriting_erlang_module",
    );
}

#[rustfmt::skip]
#[test]
fn src_importing_test() {
    let output = crate::prepare("./cases/src_importing_test");
    insta::assert_snapshot!(
        "src_importing_test",
        output,
        "./cases/src_importing_test",
    );
}

#[rustfmt::skip]
#[test]
fn unknown_module_field_in_constant() {
    let output = crate::prepare("./cases/unknown_module_field_in_constant");
    insta::assert_snapshot!(
        "unknown_module_field_in_constant",
        output,
        "./cases/unknown_module_field_in_constant",
    );
}

#[rustfmt::skip]
#[test]
fn unknown_module_field_in_expression() {
    let output = crate::prepare("./cases/unknown_module_field_in_expression");
    insta::assert_snapshot!(
        "unknown_module_field_in_expression",
        output,
        "./cases/unknown_module_field_in_expression",
    );
}

#[rustfmt::skip]
#[test]
fn unknown_module_field_in_import() {
    let output = crate::prepare("./cases/unknown_module_field_in_import");
    insta::assert_snapshot!(
        "unknown_module_field_in_import",
        output,
        "./cases/unknown_module_field_in_import",
    );
}

#[rustfmt::skip]
#[test]
fn variable_or_module() {
    let output = crate::prepare("./cases/variable_or_module");
    insta::assert_snapshot!(
        "variable_or_module",
        output,
        "./cases/variable_or_module",
    );
}
