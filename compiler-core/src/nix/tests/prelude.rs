use crate::assert_nix;

#[test]
fn qualified_ok() {
    assert_nix!(
        r#"import gleam
pub fn go() { gleam.Ok(1) }
"#,
    );
}

#[test]
fn qualified_error() {
    assert_nix!(
        r#"import gleam
pub fn go() { gleam.Error(1) }
"#,
    );
}

#[test]
fn qualified_nil() {
    assert_nix!(
        r#"import gleam
pub fn go() { gleam.Nil }
"#,
    );
}
