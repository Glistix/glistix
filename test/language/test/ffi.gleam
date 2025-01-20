pub type Dynamic

@external(erlang, "ffi_erlang", "print")
@external(javascript, "./ffi_javascript.mjs", "print")
@external(nix, "./ffi_nix.nix", "print")
pub fn print(a: String) -> Nil

@external(erlang, "ffi_erlang", "append")
@external(javascript, "./ffi_javascript.mjs", "append")
@external(nix, "./ffi_nix.nix", "append")
pub fn append(a: String, b: String) -> String

@external(erlang, "ffi_erlang", "to_string")
@external(javascript, "./ffi_javascript.mjs", "toString")
@external(nix, "./ffi_nix.nix", "toString")
pub fn to_string(a: anything) -> String

@external(erlang, "ffi_erlang", "file_exists")
@external(javascript, "./ffi_javascript.mjs", "fileExists")
@external(nix, "./ffi_nix.nix", "fileExists")
pub fn file_exists(a: String) -> Bool

@external(erlang, "ffi_erlang", "halt")
@external(javascript, "./ffi_javascript.mjs", "halt")
@external(nix, "./ffi_nix.nix", "halt")
pub fn halt(a: Int) -> Nil

@external(erlang, "ffi_erlang", "to_dynamic")
@external(javascript, "./ffi_javascript.mjs", "toDynamic")
@external(nix, "./ffi_nix.nix", "toDynamic")
pub fn to_dynamic(a: x) -> Dynamic
