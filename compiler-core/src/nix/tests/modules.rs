use crate::nix::tests::CURRENT_PACKAGE;
use crate::{assert_nix, assert_nix_with_multiple_imports};

#[test]
fn empty_module() {
    assert_nix!("", "{}\n")
}

#[test]
fn unqualified_fn_call() {
    assert_nix!(
        (CURRENT_PACKAGE, "rocket_ship", r#"pub fn launch() { 1 }"#),
        r#"import rocket_ship.{launch}
pub fn go() { launch() }
"#,
    );
}

#[test]
fn aliased_unqualified_fn_call() {
    assert_nix!(
        (CURRENT_PACKAGE, "rocket_ship", r#"pub fn launch() { 1 }"#),
        r#"import rocket_ship.{launch as boom_time}
pub fn go() { boom_time() }
"#,
    );
}

#[test]
fn multiple_unqualified_fn_call() {
    assert_nix!(
        (
            CURRENT_PACKAGE,
            "rocket_ship",
            r#"
pub fn a() { 1 }
pub fn b() { 2 }"#
        ),
        r#"import rocket_ship.{a,b as bb}
pub fn go() { a() + bb() }
"#,
    );
}

#[test]
fn constant() {
    assert_nix!(
        (CURRENT_PACKAGE, "rocket_ship", r#"pub const x = 1"#),
        r#"
import rocket_ship
pub fn go() { rocket_ship.x }
"#,
    );
}

#[test]
fn alias_aliased_constant() {
    assert_nix!(
        (CURRENT_PACKAGE, "rocket_ship", r#"pub const x = 1"#),
        r#"
import rocket_ship.{ x as y }
const z = y
"#,
    );
}

#[test]
fn renamed_module() {
    assert_nix!(
        (CURRENT_PACKAGE, "x", r#"pub const v = 1"#),
        r#"
import x as y
const z = y.v
"#,
    );
}

#[test]
fn nested_module_constant() {
    assert_nix!(
        (
            CURRENT_PACKAGE,
            "rocket_ship/launcher",
            r#"pub const x = 1"#
        ),
        r#"
import rocket_ship/launcher
pub fn go() { launcher.x }
"#,
    );
}

#[test]
fn alias_constant() {
    assert_nix!(
        (CURRENT_PACKAGE, "rocket_ship", r#"pub const x = 1"#),
        r#"
import rocket_ship as boop
pub fn go() { boop.x }
"#,
    );
}

#[test]
fn alias_fn_call() {
    assert_nix!(
        (CURRENT_PACKAGE, "rocket_ship", r#"pub fn go() { 1 }"#),
        r#"
import rocket_ship as boop
pub fn go() { boop.go() }
"#,
    );
}

#[test]
fn nested_fn_call() {
    assert_nix!(
        (CURRENT_PACKAGE, "one/two", r#"pub fn go() { 1 }"#),
        r#"import one/two
pub fn go() { two.go() }"#,
    );
}

#[test]
fn nested_nested_fn_call() {
    assert_nix!(
        (CURRENT_PACKAGE, "one/two/three", r#"pub fn go() { 1 }"#),
        r#"import one/two/three
pub fn go() { three.go() }"#,
    );
}

#[test]
fn different_package_import() {
    assert_nix!(
        ("other_package", "one", r#"pub fn go() { 1 }"#),
        r#"import one
pub fn go() { one.go() }
"#,
    );
}

#[test]
fn nested_same_package() {
    assert_nix!(
        (CURRENT_PACKAGE, "one/two/three", r#"pub fn go() { 1 }"#),
        r#"import one/two/three
pub fn go() { three.go() }
"#,
    );
}

#[test]
fn discarded_duplicate_import() {
    assert_nix_with_multiple_imports!(
        ("esa/rocket_ship", r#"pub fn go() { 1 }"#),
        ("nasa/rocket_ship", r#"pub fn go() { 1 }"#);
        r#"
import esa/rocket_ship
import nasa/rocket_ship as _nasa_rocket
pub fn go() { rocket_ship.go() }
"#
    );
}

#[test]
fn discarded_duplicate_import_with_unqualified() {
    assert_nix_with_multiple_imports!(
        ("esa/rocket_ship", r#"pub fn go() { 1 }"#),
        ("nasa/rocket_ship", r#"pub fn go() { 1 }"#);
        r#"
import esa/rocket_ship
import nasa/rocket_ship.{go} as _nasa_rocket
pub fn esa_go() { rocket_ship.go() }
pub fn nasa_go() { go() }
"#
    );
}

#[test]
fn import_with_keyword() {
    assert_nix!(
        (
            CURRENT_PACKAGE,
            "rocket_ship",
            r#"
pub const inherit = 1
pub const true = 2
"#
        ),
        r#"
import rocket_ship.{inherit, true as false}
pub fn main() {
  #(inherit, false)
}
"#
    );
}

#[test]
fn constant_module_access_with_keyword() {
    assert_nix!(
        (CURRENT_PACKAGE, "rocket_ship", r#"pub const inherit = 1"#),
        r#"
import rocket_ship
pub const variable = rocket_ship.inherit
"#,
    );
}
