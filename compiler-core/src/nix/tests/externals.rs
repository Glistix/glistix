use crate::{assert_module_error, assert_nix};

#[test]
fn type_() {
    assert_nix!(r#"pub type Thing"#,);
}

#[test]
fn module_fn() {
    assert_nix!(
        r#"
@external(nix, "./utils", "inspect")
fn show(x: anything) -> Nil"#,
    );
}

#[test]
fn pub_module_fn() {
    assert_nix!(
        r#"
@external(nix, "./utils", "inspect")
pub fn show(x: anything) -> Nil"#,
    );
}

#[test]
fn valid_patterns() {
    assert_nix!(
        r#"
@external(nix, "/abs/path", "inspect")
fn abs_path(x: anything) -> Nil

@external(nix, "/", "inspect")
fn root(x: anything) -> Nil

@external(nix, "./a", "inspect")
fn current_dir_path(x: anything) -> Nil

@external(nix, ".", "inspect")
fn current_dir(x: anything) -> Nil

@external(nix, "./", "inspect")
fn current_dir_slash(x: anything) -> Nil

@external(nix, "../a", "inspect")
fn top_dir_path(x: anything) -> Nil

@external(nix, "..", "inspect")
fn top_dir(x: anything) -> Nil

@external(nix, "../", "inspect")
fn top_dir_slash(x: anything) -> Nil

@external(nix, "./..", "inspect")
fn curr_dir_top_dir(x: anything) -> Nil

"#,
    );
}

#[test]
fn same_name_external() {
    assert_nix!(
        r#"
@external(nix, "./thingy", "fetch")
pub fn fetch(request: Nil) -> Nil"#,
    );
}

#[test]
fn same_module_multiple_imports() {
    assert_nix!(
        r#"
@external(nix, "./the/module.nix", "one")
pub fn one() -> Nil

@external(nix, "./the/module.nix", "two")
pub fn two() -> Nil
"#,
    );
}

#[test]
fn duplicate_import() {
    assert_nix!(
        r#"
@external(nix, "./the/module.nix", "dup")
pub fn one() -> Nil

@external(nix, "./the/module.nix", "dup")
pub fn two() -> Nil
"#,
    );
}

#[test]
fn name_to_escape() {
    assert_nix!(
        r#"
@external(nix, "./the/module.nix", "one")
pub fn inherit() -> Nil

@external(nix, "./the/module.nix", "one")
pub fn builtins() -> Nil
"#,
    );
}

// https://github.com/gleam-lang/gleam/issues/1636
#[test]
fn external_fn_escaping() {
    assert_nix!(
        r#"
@external(nix, "./ffi.nix", "then")
pub fn then(a: a) -> b

@external(nix, "./ffi.nix", "inherit")
pub fn escaped_inherit(a: a) -> b

@external(nix, "./ffi.nix", "inherit'")
pub fn inherit(a: a) -> b

@external(nix, "./ffi.nix", "builtins")
pub fn not_escaped(a: a) -> b

@external(nix, "./ffi.nix", "a'b")
pub fn also_not_escaped(a: a) -> b
"#,
    );
}

// https://github.com/gleam-lang/gleam/issues/1954
#[test]
fn pipe_variable_shadow() {
    assert_nix!(
        r#"
@external(nix, "./module", "string")
fn name() -> String

pub fn main() {
  let name = name()
  name
}
"#
    );
}

#[test]
fn attribute_erlang() {
    assert_nix!(
        r#"
@external(erlang, "one", "one_erl")
pub fn one(x: Int) -> Int {
  todo
}

pub fn main() {
  one(1)
}
"#
    );
}

#[test]
fn attribute_javascript() {
    assert_nix!(
        r#"
@external(javascript, "./one.mjs", "oneJs")
pub fn one(x: Int) -> Int {
  todo
}

pub fn main() {
  one(1)
}
"#
    );
}

#[test]
fn attribute_nix() {
    assert_nix!(
        r#"
@external(nix, "./one.nix", "oneNix")
pub fn one(x: Int) -> Int {
  todo
}

pub fn main() {
  one(1)
}
"#
    );
}

#[test]
fn erlang_javascript_and_nix() {
    assert_nix!(
        r#"
@external(erlang, "one", "one")
@external(javascript, "./one.mjs", "oneJs")
@external(nix, "./one.nix", "oneNix")
pub fn one(x: Int) -> Int {
  todo
}

pub fn main() {
  one(1)
}
"#
    );
}

#[test]
fn private_attribute_erlang() {
    assert_nix!(
        r#"
@external(erlang, "one", "one_erl")
fn one(x: Int) -> Int {
  todo
}

pub fn main() {
  one(1)
}
"#
    );
}

#[test]
fn private_attribute_javascript() {
    assert_nix!(
        r#"
@external(javascript, "./one.mjs", "oneJs")
fn one(x: Int) -> Int {
  todo
}

pub fn main() {
  one(1)
}
"#
    );
}

#[test]
fn private_attribute_nix() {
    assert_nix!(
        r#"
@external(nix, "./one.nix", "oneNix")
fn one(x: Int) -> Int {
  todo
}

pub fn main() {
  one(1)
}
"#
    );
}

#[test]
fn private_erlang_javascript_and_nix() {
    assert_nix!(
        r#"
@external(erlang, "one", "one")
@external(javascript, "./one.mjs", "oneJs")
@external(nix, "./one.nix", "oneNix")
fn one(x: Int) -> Int {
  todo
}

pub fn main() {
  one(1)
}
"#
    );
}

#[test]
fn no_body() {
    assert_nix!(
        r#"
@external(nix, "./one", "one")
pub fn one(x: Int) -> Int
"#
    );
}

#[test]
fn not_relative() {
    assert_module_error!(
        r#"
@external(nix, "name", "one")
pub fn one(x: Int) -> Int {
  1
}
"#
    );
}

#[test]
fn home_path() {
    assert_module_error!(
        r#"
@external(nix, "~/a", "inspect")
fn home_path(x: anything) -> Nil
"#
    );

    assert_module_error!(
        r#"
@external(nix, "~", "inspect")
fn home(x: anything) -> Nil
"#
    );

    assert_module_error!(
        r#"
@external(nix, "~/", "inspect")
fn home_slash(x: anything) -> Nil
"#
    );
}

#[test]
fn no_module() {
    assert_module_error!(
        r#"
@external(nix, "", "one")
pub fn one(x: Int) -> Int {
  1
}
"#
    );
}

#[test]
fn inline_function() {
    assert_module_error!(
        r#"
@external(nix, "./blah", "(x: x)")
pub fn one(x: Int) -> Int {
  1
}
"#
    );
}

#[test]
fn erlang_only() {
    assert_nix!(
        r#"
pub fn should_be_generated(x: Int) -> Int {
  x
}

@external(erlang, "one", "one")
pub fn should_not_be_generated(x: Int) -> Int
"#
    );
}

#[test]
fn erlang_bit_patterns() {
    assert_nix!(
        r#"
pub fn should_not_be_generated(x) {
  case x {
    <<_, rest:bits>> -> rest
    _ -> x
  }
}
"#
    );
}
