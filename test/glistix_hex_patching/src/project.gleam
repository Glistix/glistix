import cymbal.{array, block, int, string}
import gleam/io

pub fn main() -> String {
  io.println("Hello from main!")
  let document =
    block([
      #("apiVersion", string("v1")),
      #("kind", string("Pod")),
      #("metadata", block([#("name", string("example-pod"))])),
      #(
        "spec",
        block([
          #(
            "containers",
            array([
              block([
                #("name", string("example-container")),
                #("image", string("nginx")),
                #("ports", array([block([#("containerPort", int(80))])])),
              ]),
            ]),
          ),
        ]),
      ),
    ])

  cymbal.encode(document)
}
