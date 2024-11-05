use crate::{assert_nix, assert_nix_error};

#[test]
fn empty() {
    assert_nix!(
        r#"
fn go() {
  <<>>
}
"#,
    );
}

#[test]
fn one() {
    assert_nix!(
        r#"
fn go() {
  <<256>>
}
"#,
    );
}

#[test]
fn two() {
    assert_nix!(
        r#"
fn go() {
  <<256, 4>>
}
"#,
    );
}

#[test]
fn integer() {
    assert_nix!(
        r#"
fn go() {
  <<256:int>>
}
"#,
    );
}

// #[test]
// fn float() {
//     assert_nix!(
//         r#"
// fn go() {
//   <<1.1:float>>
// }
// "#,
//     );
// }

#[test]
fn sized() {
    assert_nix!(
        r#"
fn go() {
  <<256:64>>
}
"#,
    );
}

#[test]
fn explicit_sized() {
    assert_nix!(
        r#"
fn go() {
  <<256:size(64)>>
}
"#,
    );
}

#[test]
fn variable_sized() {
    assert_nix!(
        r#"
fn go(x, y) {
  <<x:size(y)>>
}
"#,
    );
}

#[test]
fn variable() {
    assert_nix!(
        r#"
fn go(x) {
  <<256, 4, x>>
}
"#,
    );
}

#[test]
fn utf8() {
    assert_nix!(
        r#"
fn go(x) {
  <<256, 4, x, "Gleam":utf8>>
}
"#,
    );
}

#[test]
fn utf8_codepoint() {
    assert_nix!(
        r#"
fn go(x) {
  <<x:utf8_codepoint, "Gleam":utf8>>
}
"#,
    );
}

#[test]
fn bit_string() {
    assert_nix!(
        r#"
fn go(x) {
  <<x:bits>>
}
"#,
    );
}

#[test]
fn bits() {
    assert_nix!(
        r#"
fn go(x) {
  <<x:bits>>
}
"#,
    );
}

#[test]
fn empty_match() {
    assert_nix!(
        r#"
fn go(x) {
  let assert <<>> = x
}
"#,
    );
}

#[test]
fn match_bytes() {
    assert_nix!(
        r#"
fn go(x) {
  let assert <<1, y>> = x
  y
}
"#,
    );
}

#[test]
fn match_sized() {
    assert_nix!(
        r#"
fn go(x) {
  let assert <<a:16, b:8>> = x
  #(a, b)
}
"#,
    );
}

#[test]
fn discard_sized() {
    assert_nix!(
        r#"
fn go(x) {
  let assert <<_:16, _:8>> = x
}
"#,
    );
}

#[test]
fn match_sized_value() {
    assert_nix!(
        r#"
fn go(x) {
  let assert <<258:16>> = x
}
"#,
    );
}

#[test]
fn match_rest() {
    assert_nix!(
        r#"
fn go(x) {
  let assert <<_, b:bytes>> = <<1,2,3>>
  b
}
"#,
    );
}

#[test]
fn match_binary_size() {
    assert_nix!(
        r#"
fn go(x) {
  let assert <<_, a:2-bytes>> = x
  let assert <<_, b:bytes-size(2)>> = x
  #(a, b)
}
"#,
    );
}

#[test]
fn as_module_const() {
    assert_nix!(
        r#"
          pub const data = <<
            0x1,
            2,
            2:size(16),
            0x4:size(32),
            "Gleam":utf8,
            // 4.2:float,
            <<
              <<1, 2, 3>>:bits,
              "Gleam":utf8,
              1024
            >>:bits
          >>
        "#
    )
}

#[test]
fn negative_size() {
    assert_nix!(
        r#"
fn go() {
  <<1:size(-1)>>
}
"#,
    );
}

// https://github.com/gleam-lang/gleam/issues/1591
#[test]
fn not_byte_aligned() {
    assert_nix_error!(
        r#"
fn thing() {
  4
}
fn go() {
  <<256:4>>
}
"#,
    );
}

#[test]
fn not_byte_aligned_explicit_sized() {
    assert_nix_error!(
        r#"
fn go() {
  <<256:size(4)>>
}
"#,
    );
}

// This test would ideally also result in go() being deleted like the previous tests
// but we can not know for sure what the value of a variable is going to be
// so right now go() is not deleted.
#[test]
fn not_byte_aligned_variable() {
    assert_nix!(
        r#"
fn go() {
  let x = 4
  <<256:size(x)>>
}
"#,
    );
}
