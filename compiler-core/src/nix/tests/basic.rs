use crate::assert_nix;

#[test]
fn variable_renaming() {
    assert_nix!(
        r#"
pub type Amogus {
  AmogusA
  AmogusB(a: Int, b: Int)
  AmogusC(Int, Float)
}

pub const sumongus: Int = 5

pub fn sus(x: Int) -> Int {
    5 + 5
}

fn go(x, foo, boolthing, bmong) {
  let a = 1
  foo
  let a = 2
  let z = [fn(y: Int) { y }, foo(a, a, 50), fn(x: Int) { x }]
  let d = {
    "aaa"
  }
  let dd = {
    let amogus = 5.5e5
  }
  let intdiv = 5 / 5
  let floatdiv = 5.0 /. 5.0
  let remm = 5 % 5
  let g = 0.5
  let b = a
  foo(a, a, 50)
  let c = {
    let a = a
    [a]
  }
  foo
  let www = a == 5
  let wwww = {a + a} == {10 - 5 * 5}
  let first_list = [1 < 4, 3 >= 5]
  let list_thing = [www, ..first_list]
  let pipes =
    a
    |> foo(_, a, 50)
    |> fn(x) { x }
  let pipes2 = a |> fn(i: Int) { i }
  panic as "amongus"
  todo as "amongus"
  panic
  todo
  // let mongus: Amogus = bmong
  // let mongus = bmong.a
  // let mongus = Amogus(..bmong, a: 6)
  let y = -500 + 10 - {-a} - {-5}
  let yy = [ -a, -5, -y ]
  let f = -5.5 -. 5.2e-5
  let f = 5.5 -. 5.2e5
  // TODO: fix string escaping
  let ss = "a" <> "b" <> "c d \\ a \f \n \r \t \u{202b} \\x{202}"
  let ff = [ -5.2e-5, -5.5 ]
  let z = !boolthing
  let tupdatup = #(1, 2, "a", -5.5, #(1, 2, 3))
  let tata = tupdatup.1 + tupdatup.0 + 20
  let mabool = True
  let mnotbool = False
  let amagus_a = AmogusA
  let amagus_b = AmogusB(10, b: 15)
  let amagus_c = AmogusC(10, 5.5)
  let simplefunc = fn() { 5 }
  let gg = simplefunc() + 10
  let less_simple_func = fn(x) { fn() { x } }
  let gg = less_simple_func(5)() + 10
  let mnull = Nil
  let sus = "hey! I'm still sus :("
  // make sure arguments are counted in initial state
  let x = tata
  x
}

fn mongus(a, b, c) {
  let x = fn() { 5 + 5 }
  let y = x()
  #(1, 2, 3, fn(x: Int) { x + 1 }, { 5 6 10 {10 + 5} })
}

fn condman(a, b) {
  let p1 = case a > b {
    True -> 5
    False -> 6
  }

  let p2 = case a > b {
    True -> 5
    _ -> 6
  }

  let p3 = case a > b {
    False -> 6
    True -> 5
  }

  let p4 = case a > b {
    False -> 6
    _ -> 5
  }

  let v1 = True
  let v2 = False
  let vv1 = case v1 { _ -> 10 }
  let vv2 = case v2 { True -> 50 _ -> 10 }
  let vv3 = case True { True -> 50 _ -> 10 }

  #(p1, p2, p3, p4, vv1, vv2, vv3)
}

pub type SimpleEnum {
  SA
  SB
  SC
  SD
}

fn simple_test(x: SimpleEnum) {
  case x {
    SC -> Ok(Ok(100))
    _ -> Error(Error(100))
  }

  case x {
    SA -> Ok(5)
    SB -> Error(10)
    SC -> Ok(10)
    SD -> Ok(10)
  }
}

pub fn simple_test2(x: SimpleEnum, y: Int, z: Float, w: String, p: Nil) {
  let x1 = case x {
    SC -> "is SC"
    SA -> "is SA"
    _ -> "not SC or SA"
  }
  let y1 = case y {
    0 -> "is 0"
    -1 -> "is -1"
    100 -> "is 100"
    _ -> "unknown int"
  }
  let z1 = case z {
    0.0 -> "is 0.0"
    53.53 -> "is 53.53"
    1.0e2 -> "is 1.0e2"
    _ -> "unknown float"
  }
  let w1 = case w {
    "a" -> "is a"
    "b" -> "is b"
    "c" -> "is c"
    _ -> "not a, b, c"
  }
  let p1 = case p {
    Nil -> "is nil"
    _ -> "not nil (impossible)"
  }
  #(x1, y1, z1, w1, p1)
}

pub fn fact(n: Int) -> #(Nil, Int) {
  let res = case n < 0 {
    True -> panic as "Don't."
    False -> Nil
  }
  #(res, case n {
    0 -> 1
    _ -> n * fact(n - 1).1
  })
}
"#,
    )
}