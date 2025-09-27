import { toList, CustomType as $CustomType, isEqual } from "../gleam.mjs";
import * as $list from "../gleam/list.mjs";
import {
  add as append_tree,
  concat as from_strings,
  concat,
  identity as from_string,
  identity as to_string,
  length as byte_size,
  lowercase,
  uppercase,
  graphemes as do_to_graphemes,
  split,
  string_replace as replace,
} from "../gleam_stdlib.mjs";

export {
  append_tree,
  byte_size,
  concat,
  from_string,
  from_strings,
  lowercase,
  replace,
  split,
  to_string,
  uppercase,
};

class All extends $CustomType {}

export function prepend_tree(tree, prefix) {
  return append_tree(prefix, tree);
}

export function new$() {
  return from_strings(toList([]));
}

export function prepend(tree, prefix) {
  return append_tree(from_string(prefix), tree);
}

export function append(tree, second) {
  return append_tree(tree, from_string(second));
}

export function join(trees, sep) {
  let _pipe = trees;
  let _pipe$1 = $list.intersperse(_pipe, from_string(sep));
  return concat(_pipe$1);
}

export function reverse(tree) {
  let _pipe = tree;
  let _pipe$1 = to_string(_pipe);
  let _pipe$2 = do_to_graphemes(_pipe$1);
  let _pipe$3 = $list.reverse(_pipe$2);
  return from_strings(_pipe$3);
}

export function is_equal(a, b) {
  return isEqual(a, b);
}

export function is_empty(tree) {
  return isEqual(from_string(""), tree);
}
