import { Error, toList, prepend as listPrepend, isEqual } from "../gleam.mjs";
import * as $option from "../gleam/option.mjs";
import {
  map_size as size,
  map_to_list as to_list,
  new_map as new$,
  map_get as get,
  map_insert as do_insert,
  map_remove as do_delete,
} from "../gleam_stdlib.mjs";

export { get, new$, size, to_list };

export function is_empty(dict) {
  return size(dict) === 0;
}

function do_has_key(key, dict) {
  return !isEqual(get(dict, key), new Error(undefined));
}

export function has_key(dict, key) {
  return do_has_key(key, dict);
}

export function insert(dict, key, value) {
  return do_insert(key, value, dict);
}

function from_list_loop(loop$list, loop$initial) {
  while (true) {
    let list = loop$list;
    let initial = loop$initial;
    if (list.hasLength(0)) {
      return initial;
    } else {
      let key = list.head[0];
      let value = list.head[1];
      let rest = list.tail;
      loop$list = rest;
      loop$initial = insert(initial, key, value);
    }
  }
}

export function from_list(list) {
  return from_list_loop(list, new$());
}

function reverse_and_concat(loop$remaining, loop$accumulator) {
  while (true) {
    let remaining = loop$remaining;
    let accumulator = loop$accumulator;
    if (remaining.hasLength(0)) {
      return accumulator;
    } else {
      let first = remaining.head;
      let rest = remaining.tail;
      loop$remaining = rest;
      loop$accumulator = listPrepend(first, accumulator);
    }
  }
}

function do_keys_loop(loop$list, loop$acc) {
  while (true) {
    let list = loop$list;
    let acc = loop$acc;
    if (list.hasLength(0)) {
      return reverse_and_concat(acc, toList([]));
    } else {
      let key = list.head[0];
      let rest = list.tail;
      loop$list = rest;
      loop$acc = listPrepend(key, acc);
    }
  }
}

export function keys(dict) {
  return do_keys_loop(to_list(dict), toList([]));
}

function do_values_loop(loop$list, loop$acc) {
  while (true) {
    let list = loop$list;
    let acc = loop$acc;
    if (list.hasLength(0)) {
      return reverse_and_concat(acc, toList([]));
    } else {
      let value = list.head[1];
      let rest = list.tail;
      loop$list = rest;
      loop$acc = listPrepend(value, acc);
    }
  }
}

export function values(dict) {
  let list_of_pairs = to_list(dict);
  return do_values_loop(list_of_pairs, toList([]));
}

function do_take_loop(loop$dict, loop$desired_keys, loop$acc) {
  while (true) {
    let dict = loop$dict;
    let desired_keys = loop$desired_keys;
    let acc = loop$acc;
    let insert$1 = (taken, key) => {
      let $ = get(dict, key);
      if ($.isOk()) {
        let value = $[0];
        return insert(taken, key, value);
      } else {
        return taken;
      }
    };
    if (desired_keys.hasLength(0)) {
      return acc;
    } else {
      let first = desired_keys.head;
      let rest = desired_keys.tail;
      loop$dict = dict;
      loop$desired_keys = rest;
      loop$acc = insert$1(acc, first);
    }
  }
}

function do_take(desired_keys, dict) {
  return do_take_loop(dict, desired_keys, new$());
}

export function take(dict, desired_keys) {
  return do_take(desired_keys, dict);
}

function insert_pair(dict, pair) {
  return insert(dict, pair[0], pair[1]);
}

function fold_inserts(loop$new_entries, loop$dict) {
  while (true) {
    let new_entries = loop$new_entries;
    let dict = loop$dict;
    if (new_entries.hasLength(0)) {
      return dict;
    } else {
      let first = new_entries.head;
      let rest = new_entries.tail;
      loop$new_entries = rest;
      loop$dict = insert_pair(dict, first);
    }
  }
}

export function merge(dict, new_entries) {
  let _pipe = new_entries;
  let _pipe$1 = to_list(_pipe);
  return fold_inserts(_pipe$1, dict);
}

export function delete$(dict, key) {
  return do_delete(key, dict);
}

export function drop(loop$dict, loop$disallowed_keys) {
  while (true) {
    let dict = loop$dict;
    let disallowed_keys = loop$disallowed_keys;
    if (disallowed_keys.hasLength(0)) {
      return dict;
    } else {
      let first = disallowed_keys.head;
      let rest = disallowed_keys.tail;
      loop$dict = delete$(dict, first);
      loop$disallowed_keys = rest;
    }
  }
}

export function upsert(dict, key, fun) {
  let $ = get(dict, key);
  if ($.isOk()) {
    let value = $[0];
    return insert(dict, key, fun(new $option.Some(value)));
  } else {
    return insert(dict, key, fun(new $option.None()));
  }
}

function fold_loop(loop$list, loop$initial, loop$fun) {
  while (true) {
    let list = loop$list;
    let initial = loop$initial;
    let fun = loop$fun;
    if (list.hasLength(0)) {
      return initial;
    } else {
      let k = list.head[0];
      let v = list.head[1];
      let rest = list.tail;
      loop$list = rest;
      loop$initial = fun(initial, k, v);
      loop$fun = fun;
    }
  }
}

export function fold(dict, initial, fun) {
  return fold_loop(to_list(dict), initial, fun);
}

function do_map_values(f, dict) {
  let f$1 = (dict, k, v) => { return insert(dict, k, f(k, v)); };
  return fold(dict, new$(), f$1);
}

export function map_values(dict, fun) {
  return do_map_values(fun, dict);
}

function do_filter(f, dict) {
  let insert$1 = (dict, k, v) => {
    let $ = f(k, v);
    if ($) {
      return insert(dict, k, v);
    } else {
      return dict;
    }
  };
  return fold(dict, new$(), insert$1);
}

export function filter(dict, predicate) {
  return do_filter(predicate, dict);
}

export function each(dict, fun) {
  return fold(
    dict,
    undefined,
    (nil, k, v) => {
      fun(k, v);
      return nil;
    },
  );
}

export function combine(dict, other, fun) {
  return fold(
    dict,
    other,
    (acc, key, value) => {
      let $ = get(acc, key);
      if ($.isOk()) {
        let other_value = $[0];
        return insert(acc, key, fun(value, other_value));
      } else {
        return insert(acc, key, value);
      }
    },
  );
}
