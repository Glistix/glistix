pub fn main() {
  println("Hello, from other_local_dep!")
}

@external(nix, "./other_local_dep_ffi.nix", "log")
pub fn println(a: String) -> Nil
