import gleam/io
import local_dep

pub fn main() {
  io.println("Hello, from project_nix + stdlib!")
  local_dep.println("Hello, from project_nix + other_local_dep!")
}
