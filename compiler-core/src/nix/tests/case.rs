use crate::assert_nix;

// https://github.com/gleam-lang/gleam/issues/1187
#[test]
fn pointless() {
    assert_nix!(
        r#"
fn go(x) {
  case x {
    _ -> x
  }
}
"#,
    )
}

// https://github.com/gleam-lang/gleam/issues/1188
#[test]
fn following_todo() {
    assert_nix!(
        r#"
fn go(x) {
  case x {
    True -> todo
    _ -> 1
  }
}
"#,
    )
}

#[test]
fn multi_subject_catch_all() {
    assert_nix!(
        r#"
fn go(x, y) {
  case x, y {
    True, True -> 1
    _, _ -> 0
  }
}
"#,
    )
}

#[test]
fn multi_subject_or() {
    assert_nix!(
        r#"
fn go(x, y) {
  case x, y {
    True, _ | _, True -> 1
    _, _ -> 0
  }
}
"#,
    )
}

#[test]
fn multi_subject_no_catch_all() {
    assert_nix!(
        r#"
fn go(x, y) {
  case x, y {
    True, _ -> 1
    _, True -> 2
    False, False -> 0
  }
}
"#,
    )
}

#[test]
fn multi_subject_subject_assignments() {
    assert_nix!(
        r#"
fn go() {
  case True, False {
    True, True -> 1
    _, _ -> 0
  }
}
"#,
    )
}

#[test]
fn assignment() {
    assert_nix!(
        r#"
fn go(x) {
  let y = case x {
    True -> 1
    _ -> 0
  }
  y
}
"#,
    )
}

#[test]
fn preassign_assignment() {
    assert_nix!(
        r#"
fn go(x) {
  let y = case x() {
    True -> 1
    _ -> 0
  }
  y
}
"#,
    )
}

// https://github.com/gleam-lang/gleam/issues/1237
#[test]
fn pipe() {
    assert_nix!(
        r#"
fn go(x, f) {
  case x |> f {
    0 -> 1
    _ -> 2
  }
}
"#,
    )
}

#[test]
fn result() {
    assert_nix!(
        r#"
fn go(x) {
  case x {
    Ok(_) -> 1
    Error(_) -> 0
  }
}
"#,
    )
}

// https://github.com/gleam-lang/gleam/issues/1506
#[test]
fn called_case() {
    assert_nix!(
        r#"
fn go(x, y) {
  case x {
    0 -> y
    _ -> y
  }()
}
"#,
    )
}

// https://github.com/gleam-lang/gleam/issues/1978
#[test]
fn case_local_var_in_tuple() {
    assert_nix!(
        r#"
fn go(x, y) {
  let z = False
  case True {
    x if #(x, z) == #(True, False) -> x
    _ -> False
  }
}
"#,
    )
}

// https://github.com/gleam-lang/gleam/issues/2665
#[test]
fn case_branches_guards_are_wrapped_in_parentheses() {
    assert_nix!(
        r#"
fn anything() -> a {
  case [] {
    [a] if False || True -> a
    _ -> anything()
  }
}
"#,
    )
}

// https://github.com/gleam-lang/gleam/issues/2759
#[test]
fn nested_string_prefix_match() {
    assert_nix!(
        r#"
fn main() {
  case Ok(["a", "b c", "d"]) {
    Ok(["a", "b " <> _, "d"]) -> 1
    _ -> 1
  }
}
"#
    );
}

// https://github.com/gleam-lang/gleam/issues/2759
#[test]
fn nested_string_prefix_match_that_would_crash_on_js() {
    assert_nix!(
        r#"
fn main() {
  case Ok(["b c", "d"]) {
    Ok(["b " <> _, "d"]) -> 1
    _ -> 1
  }
}
"#
    );
}

#[test]
fn slicing_is_handled_properly_with_multiple_branches() {
    assert_nix!(
        r#"
pub fn main() {
  case "12345" {
    "0" <> rest -> rest
    "123" <> rest -> rest
    _ -> ""
  }
}
"#
    )
}

#[test]
fn matching_on_record_with_keyword_field() {
    assert_nix!(
        r#"
type Bad {
  Bad(inherit: Int)
}

pub fn main() {
  case Bad(inherit: 5) {
    Bad(inherit: 10) -> True
    Bad(inherit: inherit) -> False
  }
}
"#
    )
}

// https://github.com/gleam-lang/gleam/issues/3379
#[test]
fn single_clause_variables() {
    assert_nix!(
        r#"
pub fn main() {
  let text = "first defined"
  case "defined again" {
    text -> Nil
  }
  let text = "a third time"
  Nil
}
"#
    )
}

// https://github.com/gleam-lang/gleam/issues/3379
#[test]
fn single_clause_variables_assigned() {
    assert_nix!(
        r#"
pub fn main() {
  let text = "first defined"
  let other = case "defined again" {
    text -> Nil
  }
  let text = "a third time"
  Nil
}
"#
    )
}

// https://github.com/gleam-lang/gleam/issues/3894
#[test]
fn nested_string_prefix_assignment() {
    assert_nix!(
        r#"
type Wibble {
  Wibble(wobble: String)
}

pub fn main() {
  let tmp = Wibble(wobble: "wibble")
  case tmp {
    Wibble(wobble: "w" as wibble <> rest) -> wibble <> rest
    _ -> panic
  }
}
"#
    )
}

#[test]
fn deeply_nested_string_prefix_assignment() {
    assert_nix!(
        r#"
type Wibble {
  Wibble(Wobble)
}
type Wobble {
  Wobble(wabble: Wabble)
}
type Wabble {
  Wabble(tuple: #(Int, String))
}

pub fn main() {
  let tmp = Wibble(Wobble(Wabble(#(42, "wibble"))))
  case tmp {
    Wibble(Wobble(Wabble(#(_int, "w" as wibble <> rest)))) -> wibble <> rest
    _ -> panic
  }
}
"#
    )
}
