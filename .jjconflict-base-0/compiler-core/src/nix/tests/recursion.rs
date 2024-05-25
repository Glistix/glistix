use crate::assert_nix;

#[test]
fn tco() {
    assert_nix!(
        r#"
pub fn main(x) {
  case x {
    0 -> Nil
    _ -> main(x - 1)
  }
}
"#
    );
}

#[test]
fn tco_case_block() {
    assert_nix!(
        r#"
pub fn main(x) {
  case x {
    0 -> Nil
    _ -> {
      let y = x
      main(y - 1)
    }
  }
}
"#
    );
}

// https://github.com/gleam-lang/gleam/issues/1779
#[test]
fn not_tco_due_to_assignment() {
    assert_nix!(
        r#"
pub fn main(x) {
  let z = {
    let y = x
    main(y - 1)
  }
  z
}
"#
    );
}

// https://github.com/gleam-lang/gleam/issues/2400
#[test]
fn shadowing_so_not_recursive() {
    // This funtion is calling an argument with the same name as itself, so it is not recursive
    assert_nix!(
        r#"
pub fn map(map) {
  map()
}
"#
    );
}
