import {
  Ok,
  Error,
  prepend as listPrepend,
  CustomType as $CustomType,
  remainderInt,
  divideInt,
} from "../gleam.mjs";
import * as $list from "../gleam/list.mjs";
import * as $option from "../gleam/option.mjs";
import { None, Some } from "../gleam/option.mjs";
import * as $order from "../gleam/order.mjs";
import * as $string_tree from "../gleam/string_tree.mjs";
import {
  string_length as length,
  lowercase,
  uppercase,
  less_than,
  string_slice as do_slice,
  crop_string as crop,
  contains_string as contains,
  starts_with,
  ends_with,
  split_once,
  join,
  trim_start,
  trim_end,
  pop_grapheme,
  graphemes as to_graphemes,
  codepoint as unsafe_int_to_utf_codepoint,
  string_to_codepoint_integer_list,
  utf_codepoint_list_to_string as from_utf_codepoints,
  utf_codepoint_to_int,
  inspect as do_inspect,
  byte_size,
} from "../gleam_stdlib.mjs";

export {
  byte_size,
  contains,
  crop,
  ends_with,
  from_utf_codepoints,
  join,
  length,
  lowercase,
  pop_grapheme,
  split_once,
  starts_with,
  to_graphemes,
  trim_end,
  trim_start,
  uppercase,
  utf_codepoint_to_int,
};

class Leading extends $CustomType {}

class Trailing extends $CustomType {}

export function is_empty(str) {
  return str === "";
}

export function reverse(string) {
  let _pipe = string;
  let _pipe$1 = $string_tree.from_string(_pipe);
  let _pipe$2 = $string_tree.reverse(_pipe$1);
  return $string_tree.to_string(_pipe$2);
}

export function replace(string, pattern, substitute) {
  let _pipe = string;
  let _pipe$1 = $string_tree.from_string(_pipe);
  let _pipe$2 = $string_tree.replace(_pipe$1, pattern, substitute);
  return $string_tree.to_string(_pipe$2);
}

export function compare(a, b) {
  let $ = a === b;
  if ($) {
    return new $order.Eq();
  } else {
    let $1 = less_than(a, b);
    if ($1) {
      return new $order.Lt();
    } else {
      return new $order.Gt();
    }
  }
}

export function slice(string, idx, len) {
  let $ = len < 0;
  if ($) {
    return "";
  } else {
    let $1 = idx < 0;
    if ($1) {
      let translated_idx = length(string) + idx;
      let $2 = translated_idx < 0;
      if ($2) {
        return "";
      } else {
        return do_slice(string, translated_idx, len);
      }
    } else {
      return do_slice(string, idx, len);
    }
  }
}

export function drop_end(string, num_graphemes) {
  let $ = num_graphemes < 0;
  if ($) {
    return string;
  } else {
    return slice(string, 0, length(string) - num_graphemes);
  }
}

export function append(first, second) {
  let _pipe = first;
  let _pipe$1 = $string_tree.from_string(_pipe);
  let _pipe$2 = $string_tree.append(_pipe$1, second);
  return $string_tree.to_string(_pipe$2);
}

export function concat(strings) {
  let _pipe = strings;
  let _pipe$1 = $string_tree.from_strings(_pipe);
  return $string_tree.to_string(_pipe$1);
}

function repeat_loop(loop$string, loop$times, loop$acc) {
  while (true) {
    let string = loop$string;
    let times = loop$times;
    let acc = loop$acc;
    let $ = times <= 0;
    if ($) {
      return acc;
    } else {
      loop$string = string;
      loop$times = times - 1;
      loop$acc = acc + string;
    }
  }
}

export function repeat(string, times) {
  return repeat_loop(string, times, "");
}

function padding(size, pad_string) {
  let pad_string_length = length(pad_string);
  let num_pads = divideInt(size, pad_string_length);
  let extra = remainderInt(size, pad_string_length);
  return repeat(pad_string, num_pads) + slice(pad_string, 0, extra);
}

export function pad_start(string, desired_length, pad_string) {
  let current_length = length(string);
  let to_pad_length = desired_length - current_length;
  let $ = to_pad_length <= 0;
  if ($) {
    return string;
  } else {
    return padding(to_pad_length, pad_string) + string;
  }
}

export function pad_end(string, desired_length, pad_string) {
  let current_length = length(string);
  let to_pad_length = desired_length - current_length;
  let $ = to_pad_length <= 0;
  if ($) {
    return string;
  } else {
    return string + padding(to_pad_length, pad_string);
  }
}

export function trim(string) {
  let _pipe = string;
  let _pipe$1 = trim_start(_pipe);
  return trim_end(_pipe$1);
}

export function drop_start(loop$string, loop$num_graphemes) {
  while (true) {
    let string = loop$string;
    let num_graphemes = loop$num_graphemes;
    let $ = num_graphemes > 0;
    if (!$) {
      return string;
    } else {
      let $1 = pop_grapheme(string);
      if ($1.isOk()) {
        let string$1 = $1[0][1];
        loop$string = string$1;
        loop$num_graphemes = num_graphemes - 1;
      } else {
        return string;
      }
    }
  }
}

function to_graphemes_loop(loop$string, loop$acc) {
  while (true) {
    let string = loop$string;
    let acc = loop$acc;
    let $ = pop_grapheme(string);
    if ($.isOk()) {
      let grapheme = $[0][0];
      let rest = $[0][1];
      loop$string = rest;
      loop$acc = listPrepend(grapheme, acc);
    } else {
      return acc;
    }
  }
}

export function split(x, substring) {
  if (substring === "") {
    return to_graphemes(x);
  } else {
    let _pipe = x;
    let _pipe$1 = $string_tree.from_string(_pipe);
    let _pipe$2 = $string_tree.split(_pipe$1, substring);
    return $list.map(_pipe$2, $string_tree.to_string);
  }
}

function do_to_utf_codepoints(string) {
  let _pipe = string;
  let _pipe$1 = string_to_codepoint_integer_list(_pipe);
  return $list.map(_pipe$1, unsafe_int_to_utf_codepoint);
}

export function to_utf_codepoints(string) {
  return do_to_utf_codepoints(string);
}

export function utf_codepoint(value) {
  if (value > 1_114_111) {
    let i = value;
    return new Error(undefined);
  } else if ((value >= 55_296) && (value <= 57_343)) {
    let i = value;
    return new Error(undefined);
  } else if (value < 0) {
    let i = value;
    return new Error(undefined);
  } else {
    let i = value;
    return new Ok(unsafe_int_to_utf_codepoint(i));
  }
}

export function to_option(string) {
  if (string === "") {
    return new None();
  } else {
    return new Some(string);
  }
}

export function first(string) {
  let $ = pop_grapheme(string);
  if ($.isOk()) {
    let first$1 = $[0][0];
    return new Ok(first$1);
  } else {
    let e = $[0];
    return new Error(e);
  }
}

export function last(string) {
  let $ = pop_grapheme(string);
  if ($.isOk() && $[0][1] === "") {
    let first$1 = $[0][0];
    return new Ok(first$1);
  } else if ($.isOk()) {
    let rest = $[0][1];
    return new Ok(slice(rest, -1, 1));
  } else {
    let e = $[0];
    return new Error(e);
  }
}

export function capitalise(string) {
  let $ = pop_grapheme(string);
  if ($.isOk()) {
    let first$1 = $[0][0];
    let rest = $[0][1];
    return append(uppercase(first$1), lowercase(rest));
  } else {
    return "";
  }
}

export function inspect(term) {
  let _pipe = do_inspect(term);
  return $string_tree.to_string(_pipe);
}
