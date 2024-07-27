pub type NoFields {
  NoFields
}

/// This function has argument names that are not valid in Erlang or JavaScript
pub fn bad_argument_names(in, class, receive) {
  #(in, class, receive)
}

/// This custom type has label names that are not valid in Erlang or JavaScript
pub type BadLabelNames {
  BadLabelNames(in: String, class: String, receive: String)
}

pub const ints_in_bit_array = <<1, 2, 3>>

pub const string_in_bit_array = <<"Gleam":utf8>>

@target(erlang)
pub const data = <<
  0x1,
  2,
  2:size(16),
  0x4:size(32),
  "Gleam":utf8,
  4.2:float,
  <<<<1, 2, 3>>:bits, "Gleam":utf8, 1024>>:bits,
>>

@target(javascript)
pub const data = <<
  0x1,
  2,
  2:size(16),
  0x4:size(32),
  "Gleam":utf8,
  4.2:float,
  <<<<1, 2, 3>>:bits, "Gleam":utf8, 1024>>:bits,
>>

@target(nix)
pub const data = <<
  0x1,
  2,
  2:size(16),
  0x4:size(32),
  "Gleam":utf8,
  <<<<1, 2, 3>>:bits, "Gleam":utf8, 1024>>:bits,
>>

@target(erlang)
pub fn get_bit_array() {
  <<
    0x1,
    2,
    2:size(16),
    0x4:size(32),
    "Gleam":utf8,
    4.2:float,
    <<<<1, 2, 3>>:bits, "Gleam":utf8, 1024>>:bits,
  >>
}

@target(javascript)
pub fn get_bit_array() {
  <<
    0x1,
    2,
    2:size(16),
    0x4:size(32),
    "Gleam":utf8,
    4.2:float,
    <<<<1, 2, 3>>:bits, "Gleam":utf8, 1024>>:bits,
  >>
}

// TODO: impl :float on nix
@target(nix)
pub fn get_bit_array() {
  <<
    0x1,
    2,
    2:size(16),
    0x4:size(32),
    "Gleam":utf8,
    <<<<1, 2, 3>>:bits, "Gleam":utf8, 1024>>:bits,
  >>
}

pub const language = "gleam"

pub type Movie {
  Movie(title: String)
}

pub const war_games = Movie("WarGames")
