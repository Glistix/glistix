use crate::assert_nix;

#[test]
fn list_literals() {
    assert_nix!(
        r#"
fn go(x) {
    []
    [1]
    [1, 2]
    [1, 2, ..x]
}
"#,
    );
}

#[test]
fn long_list_literals() {
    assert_nix!(
        r#"
fn go() {
  [111111111111111111111111111111111111111111111111111111111111111111111111]
  [11111111111111111111111111111111111111111111, 1111111111111111111111111111111111111111111]
}
"#,
    );
}

#[test]
fn multi_line_list_literals() {
    assert_nix!(
        r#"
fn go(x) {
    [{True 1}]
}
"#,
    );
}

#[test]
fn list_constants() {
    assert_nix!(
        r#"
const a = []
const b = [1, 2, 3]
"#,
    );
}

#[test]
fn list_destructuring() {
    assert_nix!(
        r#"
fn go(x, y) {
  let assert [] = x
  let assert [a] = x
  let assert [1, 2] = x
  let assert [_, #(3, b)] = y
  let assert [head, ..tail] = y
}
"#,
    );
}

#[test]
fn equality() {
    assert_nix!(
        r#"
fn go() {
  [] == [1]
  [] != [1]
}
"#,
    );
}

#[test]
fn case() {
    assert_nix!(
        r#"
fn go(xs) {
  case xs {
    [] -> 0
    [_] -> 1
    [_, _] -> 2
    _ -> 9999
  }
}
"#,
    );
}
