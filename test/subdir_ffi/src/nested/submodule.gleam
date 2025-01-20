pub fn submodule_main() {
  parent_println(message())
  parent_println(elixir_message())
}

@external(erlang, "project_ffi", "log")
@external(javascript, "../project_ffi.mjs", "log")
@external(nix, "../project_ffi.nix", "log")
fn parent_println(a: String) -> Nil

@external(erlang, "submodule_ffi", "main")
@external(javascript, "./submodule_ffi.mjs", "main")
@external(nix, "./submodule_ffi.nix", "main")
fn message() -> String

@external(erlang, "Elixir.ElixirFile", "main")
@external(javascript, "./submodule_ffi.mjs", "main")
@external(nix, "./submodule_ffi.nix", "main")
fn elixir_message() -> String
