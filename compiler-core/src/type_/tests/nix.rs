use crate::{
    glistix_assert_nix_error, glistix_assert_nix_module_error, glistix_assert_nix_module_infer,
};

#[test]
fn nix_unsafe_int_decimal() {
    glistix_assert_nix_error!(
        r#"
  [
    9_223_372_036_854_775_806,
    9_223_372_036_854_775_807,
    9_223_372_036_854_775_808,
    -9_223_372_036_854_775_806,
    -9_223_372_036_854_775_807,
    -9_223_372_036_854_775_808,
  ]
"#
    );
}

#[test]
fn nix_unsafe_int_binary() {
    glistix_assert_nix_error!(
        r#"
  [
    0b111111111111111111111111111111111111111111111111111111111111110,
    0b111111111111111111111111111111111111111111111111111111111111111,
    0b1000000000000000000000000000000000000000000000000000000000000000,
  ]
"#
    );
}

#[test]
fn nix_unsafe_int_octal() {
    glistix_assert_nix_error!(
        r#"
  [
    0o777777777777777777776,
    0o777777777777777777777,
    0o1000000000000000000000,
  ]
"#
    );
}

#[test]
fn nix_unsafe_int_hex() {
    glistix_assert_nix_error!(
        r#"
  [
    0x7FFFFFFFFFFFFFFE,
    0x7FFFFFFFFFFFFFFF,
    0x8000000000000000,
  ]
"#
    );
}

#[test]
fn nix_unsafe_int_in_tuple() {
    glistix_assert_nix_error!(
        r#"
  #(9_223_372_036_854_775_808)
"#
    );
}

#[test]
fn nix_unsafe_int_segment_in_bit_array() {
    glistix_assert_nix_error!(
        r#"
  <<9_223_372_036_854_775_808:64>>
"#
    );
}

#[test]
fn nix_unsafe_int_segment_size_in_bit_array() {
    glistix_assert_nix_error!(
        r#"
  [
    <<0:9_223_372_036_854_775_808>>,
    <<0:size(9_223_372_036_854_775_808)>>,
  ]
"#
    );
}

#[test]
fn nix_unsafe_int_in_const() {
    glistix_assert_nix_module_error!(r#"const i = 9_223_372_036_854_775_808"#);
}

#[test]
fn nix_unsafe_int_in_const_tuple() {
    glistix_assert_nix_module_error!(r#"const i = #(9_223_372_036_854_775_808)"#);
}

#[test]
fn nix_unsafe_int_segment_in_const_bit_array() {
    glistix_assert_nix_module_error!(r#"const i = <<9_223_372_036_854_775_808:64>>"#);
}

#[test]
fn nix_unsafe_int_segment_size_in_const_bit_array() {
    glistix_assert_nix_module_error!(
        r#"const ints = [
  <<0:9_223_372_036_854_775_808>>,
  <<0:size(9_223_372_036_854_775_808)>>,
]"#
    );
}

#[test]
fn nix_unsafe_int_in_pattern() {
    glistix_assert_nix_error!(r#"let assert <<9_223_372_036_854_775_808:64>> = <<>>"#);
}

#[test]
fn nix_unsafe_int_segment_size_in_pattern() {
    glistix_assert_nix_error!(r#"let assert <<0:9_223_372_036_854_775_808>> = <<>>"#);
}

#[test]
fn nix_unsafe_int_with_external_implementation() {
    glistix_assert_nix_module_infer!(
        r#"
@external(nix, "./test.mjs", "go")
pub fn go() -> Int {
  9_223_372_036_854_775_808
}
"#,
        vec![("go", "fn() -> Int")]
    );
}

#[test]
fn nix_unsafe_int_segment_in_pattern_with_external_implementation() {
    glistix_assert_nix_module_infer!(
        r#"
@external(nix, "./test.mjs", "go")
pub fn go(b: BitArray) -> BitArray {
  let assert <<0x800000000000000000:64>> = b
}
"#,
        vec![("go", "fn(BitArray) -> BitArray")]
    );
}

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
