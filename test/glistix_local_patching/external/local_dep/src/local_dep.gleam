import other_local_dep

pub fn main() {
  println("Hello, from local_dep!")
}

pub fn println(a: String) -> Nil {
  other_local_dep.println("Hello from local_dep.println!")
  other_local_dep.println(a)
}
