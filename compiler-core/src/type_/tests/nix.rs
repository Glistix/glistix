use crate::{glistix_assert_nix_error, glistix_assert_nix_module_error};

#[test]
fn out_of_range_nix_float() {
    glistix_assert_nix_error!(r#"1.8e308"#);
}

#[test]
fn out_of_range_nix_float_in_pattern() {
    glistix_assert_nix_error!(r#"let assert [1.8e308, b] = [x, y]"#);
}

#[test]
fn out_of_range_nix_float_in_const() {
    glistix_assert_nix_module_error!(r#"const x = 1.8e308"#);
}

#[test]
fn negative_out_of_range_nix_float() {
    glistix_assert_nix_error!(r#"-1.8e308"#);
}

#[test]
fn negative_out_of_range_nix_float_in_pattern() {
    glistix_assert_nix_error!(r#"let assert [-1.8e308, b] = [x, y]"#);
}

#[test]
fn negative_out_of_range_nix_float_in_const() {
    glistix_assert_nix_module_error!(r#"const x = -1.8e308"#);
}
