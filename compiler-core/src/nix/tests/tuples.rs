use crate::assert_nix;

#[test]
fn tuple() {
    assert_nix!(
        r#"
fn go() {
  #("1", "2", "3")
}
"#,
    );
}

#[test]
fn tuple1() {
    assert_nix!(
        r#"
fn go() {
  #(
    "1111111111111111111111111111111",
    #("1111111111111111111111111111111", "2", "3"),
    "3",
  )
}
"#,
    );
}

#[test]
fn tuple_access() {
    assert_nix!(
        r#"
fn go() {
  #(1, 2).0
}
"#,
    )
}

#[test]
fn tuple_with_block_element() {
    assert_nix!(
        r#"
fn go() {
  #(
    "1", 
    {
      "2"
      "3"
    },
  )
}
"#,
    );
}

#[test]
fn tuple_with_block_element1() {
    assert_nix!(
        r#"
fn go() {
  #(
    "1111111111111111111111111111111",
    #("1111111111111111111111111111111", "2", "3"),
    "3",
  )
}
"#,
    );
}

#[test]
fn constant_tuples() {
    assert_nix!(
        r#"
const a = "Hello"
const b = 1
const c = 2.0
const e = #("bob", "dug")
        "#,
    );
}

#[test]
fn constant_tuples1() {
    assert_nix!(
        r#"
const e = #(
  "loooooooooooooong", "loooooooooooong", "loooooooooooooong",
  "loooooooooooooong", "loooooooooooong", "loooooooooooooong",
)
"#
    );
}

#[test]
fn case() {
    assert_nix!(
        r#"
fn go(a) {
  case a {
    #(2, a) -> a
    #(1, 1) -> 1
    #(a, b) -> a + b
  }
}
"#
    );
}

#[test]
fn nested_pattern() {
    assert_nix!(
        r#"
fn go(x) {
  case x {
    #(2, #(a, b)) -> a + b
    _ -> 1
  }
}
"#
    );
}

#[test]
fn tuple_wrapping() {
    assert_nix!(
        r#"
const tup = #(1, -1435, 0b00110, -14.342, [1, 2, 3], <<1, 2>>, "x\u{202f}")

fn f(x) {
  x
}

fn go(x) {
  let tup = #(1, f(5), -1435, 0b00110, -14.342, [1, 2, 3], <<1, 2>>, "x\u{202f}")
  tup
}
"#
    );
}
