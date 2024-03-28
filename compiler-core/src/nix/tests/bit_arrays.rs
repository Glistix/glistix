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

// #[test]
// fn utf8() {
//     assert_nix!(
//         r#"
// fn go(x) {
//   <<256, 4, x, "Gleam":utf8>>
// }
// "#,
//     );
// }

// #[test]
// fn utf8_codepoint() {
//     assert_nix!(
//         r#"
// fn go(x) {
//   <<x:utf8_codepoint, "Gleam":utf8>>
// }
// "#,
//     );
// }

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
