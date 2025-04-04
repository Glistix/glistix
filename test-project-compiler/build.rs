use std::path::PathBuf;

pub fn main() {
    println!("cargo:rerun-if-changed=cases");

    let mut module = "//! This file is generated by build.rs
//! Do not edit it directly, instead add new test cases to ./cases

use glistix_core::build::Mode;
"
    .to_string();

    let cases = PathBuf::from("./cases");

    let mut names: Vec<_> = std::fs::read_dir(&cases)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    names.sort();

    for name in names {
        let path = cases.join(&name);
        let path = path.to_str().unwrap().replace('\\', "/");
        module.push_str(&testcase(&name, &path, "Dev"));
        module.push_str(&testcase(&name, &path, "Prod"));
        module.push_str(&testcase(&name, &path, "Lsp"));
    }

    let out = PathBuf::from("./src/generated_tests.rs");
    std::fs::write(out, module).unwrap();
}

fn testcase(name: &str, path: &str, mode: &str) -> String {
    format!(
        r#"
#[rustfmt::skip]
#[test]
fn {name}() {{
    let output = crate::prepare("{path}", Mode::{mode});
    insta::assert_snapshot!(
        "{name}",
        output,
        "{path}",
    );
}}
"#,
        name = format!("{name}_{}", mode.to_lowercase())
    )
}
