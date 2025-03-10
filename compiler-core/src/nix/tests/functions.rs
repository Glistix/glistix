use crate::assert_nix;

use super::CURRENT_PACKAGE;

#[test]
fn exported_functions() {
    assert_nix!(
        r#"
pub fn add(x, y) {
    x + y
}"#,
    );
}

#[test]
fn calling_functions() {
    assert_nix!(
        r#"
pub fn twice(f: fn(t) -> t, x: t) -> t {
  f(f(x))
}
pub fn add_one(x: Int) -> Int {
  x + 1
}
pub fn add_two(x: Int) -> Int {
  twice(add_one, x)
}

pub fn take_two(x: Int) -> Int {
  twice(fn(y) {y - 1}, x)
}
"#,
    );
}

#[test]
fn function_formatting() {
    assert_nix!(
        r#"
pub fn add(the_first_variable_that_should_be_added, the_second_variable_that_should_be_added) {
  the_first_variable_that_should_be_added + the_second_variable_that_should_be_added
}"#,
    );
}

#[test]
fn function_formatting1() {
    assert_nix!(
        r#"
pub fn this_function_really_does_have_a_ludicrously_unfeasibly_long_name_for_a_function(x, y) {
x + y
}"#,
    );
}

#[test]
fn function_formatting2() {
    assert_nix!(
        r#"
pub fn add(x, y) {
x + y
}

pub fn long() {
  add(1, add(1, add(1, add(1, add(1, add(1, add(1, add(1, add(1, add(1, add(1, add(1, add(1, add(1, add(1, 1)))))))))))))))
}"#,
    );
}

#[test]
fn function_formatting3() {
    assert_nix!(
        r#"
pub fn math(x, y) {
  fn() {
    x + y
    x - y
    2 * x
  }
}"#,
    );
}

#[test]
fn tail_call() {
    assert_nix!(
        r#"
pub fn count(xs, n) {
  case xs {
    [] -> n
    [_, ..xs] -> count(xs, n + 1)
  }
}
"#,
    );
}

#[test]
fn tail_call_doesnt_clobber_tail_position_tracking() {
    assert_nix!(
        r#"
pub fn loop(indentation) {
  case indentation > 0 {
    True -> loop(indentation - 1)
    False -> Nil
  }
}
"#,
    );
}

#[test]
fn pipe_last() {
    assert_nix!(
        r#"fn id(x) { x }
pub fn main() {
  1
  |> id
}
"#,
    );
}

#[test]
fn calling_fn_literal() {
    assert_nix!(
        r#"pub fn main() {
  fn(x) { x }(1)
}
"#,
    );
}

// Don't mistake calling a function with the same name as the current function
// as tail recursion
#[test]
fn shadowing_current() {
    assert_nix!(
        r#"pub fn main() {
  let main = fn() { 0 }
  main()
}
"#,
    );
}

#[test]
fn recursion_with_discards() {
    assert_nix!(
        r#"pub fn main(f, _) {
  f()
  main(f, 1)
}
"#,
    );
}

#[test]
fn no_recur_in_anon_fn() {
    assert_nix!(
        r#"pub fn main() {
  fn() { main() }
  1
}
"#,
    );
}

#[test]
fn case_in_call() {
    assert_nix!(
        r#"pub fn main(f, x) {
  f(case x {
    1 -> 2
    _ -> 0
  })
}
"#,
    );
}

#[test]
fn reserved_word_fn() {
    assert_nix!(
        r#"pub fn inherit() {
  Nil
}
"#,
    );
}

#[test]
fn reserved_word_imported() {
    assert_nix!(
        (CURRENT_PACKAGE, "inherit", "pub fn with() { 1 }"),
        r#"import inherit.{with}

pub fn in() {
  with()
}
"#,
    );
}

#[test]
fn reserved_word_imported_alias() {
    assert_nix!(
        (CURRENT_PACKAGE, "inherit", "pub fn with() { 1 }"),
        r#"import inherit.{with as null} as or

pub fn in() {
  let rec = or.with
  null()
}
"#,
    );
}

#[test]
fn reserved_word_const() {
    assert_nix!(
        r#"const in = 1

pub fn rec() {
  in
}
"#,
    );
}

// https://github.com/gleam-lang/gleam/issues/1208
#[test]
fn reserved_word_argument() {
    assert_nix!(
        r#"pub fn main(with) {
  with
}
"#,
    );
}

// https://github.com/gleam-lang/gleam/issues/1186
#[test]
fn multiple_discard() {
    assert_nix!(
        r#"pub fn main(_, _, _) {
  1
}
"#,
    );
}

#[test]
fn keyword_in_recursive_function() {
    assert_nix!(
        r#"pub fn main(with: Int) -> Nil {
  main(with - 1)
}
"#,
    );
}

#[test]
fn reserved_word_in_function_arguments() {
    assert_nix!(
        r#"pub fn main(arguments, eval) {
  #(arguments, eval)
}
"#,
    );
}

#[test]
fn let_last() {
    assert_nix!(
        r#"pub fn main() {
  let x = 1
}
"#,
    );
}

#[test]
fn assert_last() {
    assert_nix!(
        r#"pub fn main() {
  let assert x = 1
}
"#,
    );
}

// https://github.com/gleam-lang/gleam/issues/1637
#[test]
fn variable_rewriting_in_anon_fn_with_matching_parameter() {
    assert_nix!(
        r#"pub fn bad() {
  fn(state) {
    let state = state
    state
  }
}
"#,
    );
}

// https://github.com/gleam-lang/gleam/issues/1637
#[test]
fn variable_rewriting_in_anon_fn_with_matching_parameter_in_case() {
    assert_nix!(
        r#"pub fn bad() {
  fn(state) {
    let state = case Nil {
      _ -> state
    }
    state
  }
}
"#,
    );
}

// https://github.com/gleam-lang/gleam/issues/1508
#[test]
fn pipe_variable_rebinding() {
    assert_nix!(
        "
pub fn main() {
  let version = 1 |> version()
  version
}

pub fn version(n) {
  Ok(1)
}"
    )
}

#[test]
fn pipe_shadow_import() {
    assert_nix!(
        (CURRENT_PACKAGE, "wibble", "pub fn println(x: String) {  }"),
        r#"
        import wibble.{println}
        pub fn main() {
          let println =
            "oh dear"
            |> println
          println
        }"#
    );
}

#[test]
fn module_const_fn() {
    assert_nix!(
        r#"
pub fn int_identity(i: Int) -> Int { i }
pub const int_identity_alias: fn(Int) -> Int = int_identity
pub fn use_int_identity_alias() { int_identity_alias(42) }

pub const compound: #(fn(Int) -> Int, fn(Int) -> Int) = #(int_identity, int_identity_alias)
pub fn use_compound() { compound.0(compound.1(42)) }"#
    );
}

// https://github.com/gleam-lang/gleam/issues/2399
#[test]
fn bad_comma() {
    assert_nix!(
        r#"
fn function_with_a_long_name_that_is_intended_to_sit_right_on_the_limit() {
  Nil
}

fn identity(x) {
  x
}

pub fn main() {
  function_with_a_long_name_that_is_intended_to_sit_right_on_the_limit()
  |> identity
}
"#
    )
}

// https://github.com/gleam-lang/gleam/issues/2518
#[test]
fn function_literals_get_properly_wrapped() {
    assert_nix!(
        r#"pub fn main() {
  fn(n) { n + 1 }(10)
}
"#
    );

    assert_nix!(
        r#"pub fn main() {
  { fn(n) { n + 1 } }(10)
}
"#
    );

    assert_nix!(
        r#"pub fn main() {
  { let a = fn(n) { n + 1 } }(10)
}
"#
    );
}

#[test]
fn labelled_argument_ordering() {
    // https://github.com/gleam-lang/gleam/issues/3671
    assert_nix!(
        "
type A { A }
type B { B }
type C { C }
type D { D }

fn wibble(a a: A, b b: B, c c: C, d d: D) {
  Nil
}

pub fn main() {
  wibble(A, C, D, b: B)
  wibble(A, C, D, b: B)
  wibble(B, C, D, a: A)
  wibble(B, C, a: A, d: D)
  wibble(B, C, d: D, a: A)
  wibble(B, D, a: A, c: C)
  wibble(B, D, c: C, a: A)
  wibble(C, D, b: B, a: A)
}
"
    );
}
