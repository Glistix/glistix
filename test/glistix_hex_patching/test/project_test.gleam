import gleam/io
import gleeunit
import gleeunit/should
import project

pub fn main() {
  io.println("Hello from project_test!")
  gleeunit.main()
}

pub fn equality_test() {
  project.main()
  |> should.equal(
    "---
apiVersion: v1
kind: Pod
metadata:
  name: example-pod
spec:
  containers:
  - name: example-container
    image: nginx
    ports:
    - containerPort: 80
",
  )
}
