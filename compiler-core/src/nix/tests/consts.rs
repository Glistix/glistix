use crate::assert_nix;

#[test]
fn custom_type_constructor_imported_and_aliased() {
    assert_nix!(
        ("package", "other_module", "pub type T { A }"),
        r#"import other_module.{A as B}
pub const local = B
"#,
    );
}

#[test]
fn imported_aliased_ok() {
    assert_nix!(
        r#"import gleam.{Ok as Y}
pub type X {
  Ok
}
pub const y = Y
"#,
    );
}

#[test]
fn imported_ok() {
    assert_nix!(
        r#"import gleam
pub type X {
  Ok
}
pub const y = gleam.Ok
"#,
    );
}
