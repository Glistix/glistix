use crate::assert_nix;

#[test]
fn tuple_matching() {
    assert_nix!(
        r#"
fn go(x) {
  let assert #(1, 2) = x
}
"#,
    )
}

#[test]
fn assert() {
    assert_nix!(r#"fn go(x) { let assert 1 = x }"#,);
}

#[test]
fn assert1() {
    assert_nix!(r#"fn go(x) { let assert #(1, 2) = x }"#,);
}

#[test]
fn nested_binding() {
    assert_nix!(
        r#"
fn go(x) {
  let assert #(a, #(b, c, 2) as t, _, 1) = x
}
"#,
    )
}

#[test]
fn variable_renaming() {
    assert_nix!(
        r#"

fn go(x, wibble) {
  let a = 1
  wibble(a)
  let a = 2
  wibble(a)
  let assert #(a, 3) = x
  let b = a
  wibble(b)
  let c = {
    let a = a
    #(a, b)
  }
  wibble(a)
  // make sure arguments are counted in initial state
  let x = c
  x
}
"#,
    )
}

#[test]
fn constant_assignments() {
    assert_nix!(
        r#"
const a = True

fn go() {
  a
  let a = 10
  a + 20
}

fn second() {
  let a = 10
  a + 20
}
"#,
    );
}

#[test]
fn returning_literal_subject() {
    assert_nix!(r#"fn go(x) { let assert 1 = x + 1 }"#,);
}

#[test]
fn rebound_argument() {
    assert_nix!(
        r#"pub fn main(x) {
  let x = False
  x
}
"#,
    );
}

#[test]
fn rebound_function() {
    assert_nix!(
        r#"pub fn x() {
  Nil
}

pub fn main() {
  let x = False
  x
}
"#,
    );
}

#[test]
fn rebound_function_and_arg() {
    assert_nix!(
        r#"pub fn x() {
  Nil
}

pub fn main(x) {
  let x = False
  x
}
"#,
    );
}

#[test]
fn variable_used_in_pattern_and_assignment() {
    assert_nix!(
        r#"pub fn main(x) {
  let #(x) = #(x)
  x
}
"#,
    );
}

// https://github.com/gleam-lang/gleam/issues/1253
#[test]
fn correct_variable_renaming_in_assigned_functions() {
    assert_nix!(
        r#"
pub fn debug(x) {
  let x = x
  fn(x) {
    x + 1
    let x = x
    let x = x
    x
  }
}
"#,
    );
}

#[test]
fn module_const_var() {
    assert_nix!(
        r#"
pub const int = 42
pub const int_alias = int
pub fn use_int_alias() { int_alias }

pub const compound: #(Int, Int) = #(int, int_alias)
pub fn use_compound() { compound.0 + compound.1 }
"#
    );
}

// https://github.com/gleam-lang/gleam/issues/2443
#[test]
fn let_assert_string_prefix() {
    assert_nix!(
        r#"
pub fn main(x) {
  let assert "Game " <> id = x
  id
}
"#
    );
}

// https://github.com/gleam-lang/gleam/issues/3894
#[test]
fn let_assert_nested_string_prefix() {
    assert_nix!(
        r#"
type Wibble {
  Wibble(wibble: String)
}

pub fn main() {
  let assert Wibble(wibble: "w" as prefix <> rest) = Wibble("wibble")
  prefix <> rest
}
"#
    );
}

// Inspired by https://github.com/gleam-lang/gleam/issues/2931
#[test]
fn keyword_assignment() {
    assert_nix!(
        r#"
pub fn main() {
  let with = 10
  let in = 50
  in
}
"#
    );
}

// Inspired by https://github.com/gleam-lang/gleam/issues/3004
#[test]
fn escaped_variables_in_constants() {
    assert_nix!(
        r#"
pub const with = 5
pub const in = with
"#
    );
}

#[test]
fn message() {
    assert_nix!(
        r#"
pub fn unwrap_or_panic(value) {
  let assert Ok(inner) = value as "Oops, there was an error"
  inner
}
"#
    );
}

#[test]
fn variable_message() {
    assert_nix!(
        r#"
pub fn expect(value, message) {
  let assert Ok(inner) = value as message
  inner
}
"#
    );
}
