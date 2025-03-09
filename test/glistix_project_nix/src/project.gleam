pub fn main() {
  println("Hello, from project_nix!")
}

@external(nix, "./project_ffi.nix", "log")
fn println(a: String) -> Nil
