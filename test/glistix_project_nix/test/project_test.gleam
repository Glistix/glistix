import gleam/io
import gleeunit
import project

pub fn main() {
  project.main()
  io.println("Hello from project_test!")
  gleeunit.main()
}

pub fn equality_test() {
  let assert True = 5 == { 3 + 2 }
}
