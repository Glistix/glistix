use crate::assert_nix;

#[test]
fn function_with_documentation() {
    assert_nix!(
        r#"
/// Function doc!
pub fn documented() { 1 }"#
    );
}

#[test]
fn record_with_documentation() {
    assert_nix!(
        r#"
/// My record
type Data {
  /// Creates a single datum.
  Datum(field: Int)

  /// Creates empty data.
  Empty
}"#
    );
}

#[test]
fn function_with_multiline_documentation() {
    assert_nix!(
        r#"
/// Function doc!
/// Hello!!
///
pub fn documented() { 1 }"#
    );
}

#[test]
fn block_comments_in_documentation_are_escaped() {
    assert_nix!(
        r#"
/// /* hello */
pub fn documented() { 1 }"#
    );
}

#[test]
fn single_line_module_comment() {
    assert_nix!(
        r#"
//// Hello! This is a single line module comment.
pub fn main() { 1 }"#
    );
}

#[test]
fn multi_line_module_comment() {
    assert_nix!(
        r#"
//// Hello! This is a multi-
//// line module comment.
////
pub fn main() { 1 }"#
    );
}
