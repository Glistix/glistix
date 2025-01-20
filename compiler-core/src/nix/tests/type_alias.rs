use crate::assert_nix;

#[test]
fn type_alias() {
    assert_nix!(
        r#"
pub type Headers = List(#(String, String))
"#,
    );
}
