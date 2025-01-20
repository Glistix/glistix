use crate::assert_nix;

#[test]
fn record_accessors() {
    // We can use record accessors for types with only one constructor
    assert_nix!(
        r#"
pub type Person { Person(name: String, age: Int) }
pub fn get_age(person: Person) { person.age }
pub fn get_name(person: Person) { person.name }
"#
    );
}

#[test]
fn record_accessors_with_escaping() {
    assert_nix!(
        r#"
pub type Person { Person(then: String, builtins: Int) }
pub fn get_age(person: Person) { person.then }
pub fn get_name(person: Person) { person.builtins }
"#
    );
}

#[test]
fn record_accessor_multiple_variants() {
    // We can access fields on custom types with multiple variants
    assert_nix!(
        "
pub type Person {
    Teacher(name: String, title: String)
    Student(name: String, age: Int)
}
pub fn get_name(person: Person) { person.name }"
    );
}

#[test]
fn record_accessor_multiple_variants_positions_other_than_first() {
    // We can access fields on custom types with multiple variants
    // In positions other than the 1st field
    assert_nix!(
        "
pub type Person {
    Teacher(name: String, age: Int, title: String)
    Student(name: String, age: Int)
}
pub fn get_name(person: Person) { person.name }
pub fn get_age(person: Person) { person.age }"
    );
}

#[test]
fn record_accessor_multiple_with_first_position_different_types() {
    // We can access fields on custom types with multiple variants
    // In positions other than the 1st field
    assert_nix!(
        "
pub type Person {
    Teacher(name: Nil, age: Int)
    Student(name: String, age: Int)
}
pub fn get_age(person: Person) { person.age }"
    );
}

#[test]
fn record_accessor_multiple_variants_parameterised_types() {
    // We can access fields on custom types with multiple variants
    // In positions other than the 1st field
    assert_nix!(
        "
pub type Person {
    Teacher(name: String, age: List(Int), title: String)
    Student(name: String, age: List(Int))
}
pub fn get_name(person: Person) { person.name }
pub fn get_age(person: Person) { person.age }"
    );
}

// https://github.com/gleam-lang/gleam/pull/3878
#[test]
fn nested_record_update() {
    assert_nix!(
        "pub type Wibble {
  Wibble(a: Int, b: Wobble, c: Int)
}

pub type Wobble {
  Wobble(a: Int, b: Int)
}

pub fn main() {
  let base = Wibble(1, Wobble(2, 3), 4)
  Wibble(..base, b: Wobble(..base.b, b: 5))
}"
    );
}
