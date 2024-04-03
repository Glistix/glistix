use crate::assert_nix;

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
  <<256:4>>
}
"#,
    );
}

#[test]
fn explicit_sized() {
    assert_nix!(
        r#"
fn go() {
  <<256:size(4)>>
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
