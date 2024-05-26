use crate::assert_nix;

#[test]
fn bare() {
    assert_nix!(
        r#"
fn go() {
  panic
}
"#,
    );
}

#[test]
fn panic_as() {
    assert_nix!(
        r#"
fn go() {
  let x = "wibble"
  panic as x
}
"#,
    );
}

#[test]
fn as_expression() {
    assert_nix!(
        r#"
fn go(f) {
  let boop = panic
  f(panic)
}
"#,
    );
}

#[test]
fn pipe() {
    assert_nix!(
        r#"
fn go(f) {
  f |> panic
}
"#,
    );
}

#[test]
fn sequence() {
    assert_nix!(
        r#"
fn go(at_the_disco) {
  panic
  at_the_disco
}
"#,
    );
}

#[test]
fn case() {
    assert_nix!(
        r#"
fn go(x) {
  case x {
    _ -> panic
  }
}
"#,
    );
}

#[test]
fn panic_with_call() {
    assert_nix! {
        r#"
fn go(x) {
  panic as x(5, "abc")
}
"#
    }
}
