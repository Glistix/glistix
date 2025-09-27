import {
  Ok,
  Error,
  toList,
  prepend as listPrepend,
  CustomType as $CustomType,
  isEqual,
} from "../../gleam.mjs";
import * as $bit_array from "../../gleam/bit_array.mjs";
import * as $dict from "../../gleam/dict.mjs";
import * as $dynamic from "../../gleam/dynamic.mjs";
import * as $int from "../../gleam/int.mjs";
import * as $list from "../../gleam/list.mjs";
import * as $option from "../../gleam/option.mjs";
import { None, Some } from "../../gleam/option.mjs";
import { identity as cast } from "../../gleam_stdlib.mjs";
import {
  index as bare_index,
  int as dynamic_int,
  float as dynamic_float,
  bit_array as dynamic_bit_array,
  list as decode_list,
  dict as decode_dict,
  is_null,
  string as dynamic_string,
} from "../../gleam_stdlib_decode_ffi.mjs";

export class DecodeError extends $CustomType {
  constructor(expected, found, path) {
    super();
    this.expected = expected;
    this.found = found;
    this.path = path;
  }
}

class Decoder extends $CustomType {
  constructor(function$) {
    super();
    this.function = function$;
  }
}

export function run(data, decoder) {
  let $ = decoder.function(data);
  let maybe_invalid_data = $[0];
  let errors = $[1];
  if (errors.hasLength(0)) {
    return new Ok(maybe_invalid_data);
  } else {
    return new Error(errors);
  }
}

export function success(data) {
  return new Decoder((_) => { return [data, toList([])]; });
}

function decode_dynamic(data) {
  return [data, toList([])];
}

export function map(decoder, transformer) {
  return new Decoder(
    (d) => {
      let $ = decoder.function(d);
      let data = $[0];
      let errors = $[1];
      return [transformer(data), errors];
    },
  );
}

export function map_errors(decoder, transformer) {
  return new Decoder(
    (d) => {
      let $ = decoder.function(d);
      let data = $[0];
      let errors = $[1];
      return [data, transformer(errors)];
    },
  );
}

export function then$(decoder, next) {
  return new Decoder(
    (dynamic_data) => {
      let $ = decoder.function(dynamic_data);
      let data = $[0];
      let errors = $[1];
      let decoder$1 = next(data);
      let $1 = decoder$1.function(dynamic_data);
      let layer = $1;
      let data$1 = $1[0];
      if (errors.hasLength(0)) {
        return layer;
      } else {
        return [data$1, errors];
      }
    },
  );
}

function run_decoders(loop$data, loop$failure, loop$decoders) {
  while (true) {
    let data = loop$data;
    let failure = loop$failure;
    let decoders = loop$decoders;
    if (decoders.hasLength(0)) {
      return failure;
    } else {
      let decoder = decoders.head;
      let decoders$1 = decoders.tail;
      let $ = decoder.function(data);
      let layer = $;
      let errors = $[1];
      if (errors.hasLength(0)) {
        return layer;
      } else {
        loop$data = data;
        loop$failure = failure;
        loop$decoders = decoders$1;
      }
    }
  }
}

export function one_of(first, alternatives) {
  return new Decoder(
    (dynamic_data) => {
      let $ = first.function(dynamic_data);
      let layer = $;
      let errors = $[1];
      if (errors.hasLength(0)) {
        return layer;
      } else {
        return run_decoders(dynamic_data, layer, alternatives);
      }
    },
  );
}

export function recursive(inner) {
  return new Decoder(
    (data) => {
      let decoder = inner();
      return decoder.function(data);
    },
  );
}

export function optional(inner) {
  return new Decoder(
    (data) => {
      let $ = is_null(data);
      if ($) {
        return [new $option.None(), toList([])];
      } else {
        let $1 = inner.function(data);
        let data$1 = $1[0];
        let errors = $1[1];
        return [new $option.Some(data$1), errors];
      }
    },
  );
}

export const dynamic = /* @__PURE__ */ new Decoder(decode_dynamic);

export function decode_error(expected, found) {
  return toList([
    new DecodeError(expected, $dynamic.classify(found), toList([])),
  ]);
}

function run_dynamic_function(data, name, f) {
  let $ = f(data);
  if ($.isOk()) {
    let data$1 = $[0];
    return [data$1, toList([])];
  } else {
    let zero = $[0];
    return [
      zero,
      toList([new DecodeError(name, $dynamic.classify(data), toList([]))]),
    ];
  }
}

function decode_bool(data) {
  let $ = isEqual(cast(true), data);
  if ($) {
    return [true, toList([])];
  } else {
    let $1 = isEqual(cast(false), data);
    if ($1) {
      return [false, toList([])];
    } else {
      return [false, decode_error("Bool", data)];
    }
  }
}

function decode_int(data) {
  return run_dynamic_function(data, "Int", dynamic_int);
}

function decode_float(data) {
  return run_dynamic_function(data, "Float", dynamic_float);
}

function decode_bit_array(data) {
  return run_dynamic_function(data, "BitArray", dynamic_bit_array);
}

export function collapse_errors(decoder, name) {
  return new Decoder(
    (dynamic_data) => {
      let $ = decoder.function(dynamic_data);
      let layer = $;
      let data = $[0];
      let errors = $[1];
      if (errors.hasLength(0)) {
        return layer;
      } else {
        return [data, decode_error(name, dynamic_data)];
      }
    },
  );
}

export function failure(zero, expected) {
  return new Decoder((d) => { return [zero, decode_error(expected, d)]; });
}

export function new_primitive_decoder(name, decoding_function) {
  return new Decoder(
    (d) => {
      let $ = decoding_function(d);
      if ($.isOk()) {
        let t = $[0];
        return [t, toList([])];
      } else {
        let zero = $[0];
        return [
          zero,
          toList([new DecodeError(name, $dynamic.classify(d), toList([]))]),
        ];
      }
    },
  );
}

export const bool = /* @__PURE__ */ new Decoder(decode_bool);

export const int = /* @__PURE__ */ new Decoder(decode_int);

export const float = /* @__PURE__ */ new Decoder(decode_float);

export const bit_array = /* @__PURE__ */ new Decoder(decode_bit_array);

function decode_string(data) {
  return run_dynamic_function(data, "String", dynamic_string);
}

export const string = /* @__PURE__ */ new Decoder(decode_string);

function fold_dict(acc, key, value, key_decoder, value_decoder) {
  let $ = key_decoder(key);
  if ($[1].hasLength(0)) {
    let key$1 = $[0];
    let $1 = value_decoder(value);
    if ($1[1].hasLength(0)) {
      let value$1 = $1[0];
      let dict$1 = $dict.insert(acc[0], key$1, value$1);
      return [dict$1, acc[1]];
    } else {
      let errors = $1[1];
      return push_path([$dict.new$(), errors], toList(["values"]));
    }
  } else {
    let errors = $[1];
    return push_path([$dict.new$(), errors], toList(["keys"]));
  }
}

export function dict(key, value) {
  return new Decoder(
    (data) => {
      let $ = decode_dict(data);
      if (!$.isOk()) {
        return [$dict.new$(), decode_error("Dict", data)];
      } else {
        let dict$1 = $[0];
        return $dict.fold(
          dict$1,
          [$dict.new$(), toList([])],
          (a, k, v) => {
            let $1 = a[1];
            if ($1.hasLength(0)) {
              return fold_dict(a, k, v, key.function, value.function);
            } else {
              return a;
            }
          },
        );
      }
    },
  );
}

export function list(inner) {
  return new Decoder(
    (data) => {
      return decode_list(
        data,
        inner.function,
        (p, k) => { return push_path(p, toList([k])); },
        0,
        toList([]),
      );
    },
  );
}

function push_path(layer, path) {
  let decoder = one_of(
    string,
    toList([
      (() => {
        let _pipe = int;
        return map(_pipe, $int.to_string);
      })(),
    ]),
  );
  let path$1 = $list.map(
    path,
    (key) => {
      let key$1 = cast(key);
      let $ = run(key$1, decoder);
      if ($.isOk()) {
        let key$2 = $[0];
        return key$2;
      } else {
        return ("<" + $dynamic.classify(key$1)) + ">";
      }
    },
  );
  let errors = $list.map(
    layer[1],
    (error) => {
      let _record = error;
      return new DecodeError(
        _record.expected,
        _record.found,
        $list.append(path$1, error.path),
      );
    },
  );
  return [layer[0], errors];
}

function index(
  loop$path,
  loop$position,
  loop$inner,
  loop$data,
  loop$handle_miss
) {
  while (true) {
    let path = loop$path;
    let position = loop$position;
    let inner = loop$inner;
    let data = loop$data;
    let handle_miss = loop$handle_miss;
    if (path.hasLength(0)) {
      let _pipe = inner(data);
      return push_path(_pipe, $list.reverse(position));
    } else {
      let key = path.head;
      let path$1 = path.tail;
      let $ = bare_index(data, key);
      if ($.isOk() && $[0] instanceof Some) {
        let data$1 = $[0][0];
        loop$path = path$1;
        loop$position = listPrepend(key, position);
        loop$inner = inner;
        loop$data = data$1;
        loop$handle_miss = handle_miss;
      } else if ($.isOk() && $[0] instanceof None) {
        return handle_miss(data, listPrepend(key, position));
      } else {
        let kind = $[0];
        let $1 = inner(data);
        let default$ = $1[0];
        let _pipe = [
          default$,
          toList([new DecodeError(kind, $dynamic.classify(data), toList([]))]),
        ];
        return push_path(_pipe, $list.reverse(position));
      }
    }
  }
}

export function subfield(field_path, field_decoder, next) {
  return new Decoder(
    (data) => {
      let $ = index(
        field_path,
        toList([]),
        field_decoder.function,
        data,
        (data, position) => {
          let $1 = field_decoder.function(data);
          let default$ = $1[0];
          let _pipe = [
            default$,
            toList([new DecodeError("Field", "Nothing", toList([]))]),
          ];
          return push_path(_pipe, $list.reverse(position));
        },
      );
      let out = $[0];
      let errors1 = $[1];
      let $1 = next(out).function(data);
      let out$1 = $1[0];
      let errors2 = $1[1];
      return [out$1, $list.append(errors1, errors2)];
    },
  );
}

export function at(path, inner) {
  return new Decoder(
    (data) => {
      return index(
        path,
        toList([]),
        inner.function,
        data,
        (data, position) => {
          let $ = inner.function(data);
          let default$ = $[0];
          let _pipe = [
            default$,
            toList([new DecodeError("Field", "Nothing", toList([]))]),
          ];
          return push_path(_pipe, $list.reverse(position));
        },
      );
    },
  );
}

export function field(field_name, field_decoder, next) {
  return subfield(toList([field_name]), field_decoder, next);
}

export function optional_field(key, default$, field_decoder, next) {
  return new Decoder(
    (data) => {
      let $ = (() => {
        let $1 = bare_index(data, key);
        if ($1.isOk() && $1[0] instanceof Some) {
          let data$1 = $1[0][0];
          return field_decoder.function(data$1);
        } else if ($1.isOk() && $1[0] instanceof None) {
          return [default$, toList([])];
        } else {
          let kind = $1[0];
          let _pipe = [
            default$,
            toList([new DecodeError(kind, $dynamic.classify(data), toList([]))]),
          ];
          return push_path(_pipe, toList([key]));
        }
      })();
      let out = $[0];
      let errors1 = $[1];
      let $1 = next(out).function(data);
      let out$1 = $1[0];
      let errors2 = $1[1];
      return [out$1, $list.append(errors1, errors2)];
    },
  );
}

export function optionally_at(path, default$, inner) {
  return new Decoder(
    (data) => {
      return index(
        path,
        toList([]),
        inner.function,
        data,
        (_, _1) => { return [default$, toList([])]; },
      );
    },
  );
}
