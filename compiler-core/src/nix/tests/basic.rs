use crate::assert_nix;

#[test]
fn variable_renaming() {
    assert_nix!(
        r#"

fn go(x, foo) {
  let a = 1
  foo(a)
  let a = 2
  foo(a)
  let b = a
  foo(b)
  let c = {
    let a = a
    [a]
  }
  foo(a)
  // make sure arguments are counted in initial state
  let x = c
  x
}
"#,
    )
}
