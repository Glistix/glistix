import { Ok, Error, toList, prepend as listPrepend, remainderInt, divideInt } from "../gleam.mjs";
import * as $float from "../gleam/float.mjs";
import * as $order from "../gleam/order.mjs";
import {
  parse_int as parse,
  int_from_base_string as do_base_parse,
  to_string,
  int_to_base_string as do_to_base_string,
  identity as to_float,
  bitwise_and,
  bitwise_not,
  bitwise_or,
  bitwise_exclusive_or,
  bitwise_shift_left,
  bitwise_shift_right,
} from "../gleam_stdlib.mjs";

export {
  bitwise_and,
  bitwise_exclusive_or,
  bitwise_not,
  bitwise_or,
  bitwise_shift_left,
  bitwise_shift_right,
  parse,
  to_float,
  to_string,
};

export function absolute_value(x) {
  let $ = x >= 0;
  if ($) {
    return x;
  } else {
    return x * -1;
  }
}

export function base_parse(string, base) {
  let $ = (base >= 2) && (base <= 36);
  if ($) {
    return do_base_parse(string, base);
  } else {
    return new Error(undefined);
  }
}

export function to_base_string(x, base) {
  let $ = (base >= 2) && (base <= 36);
  if ($) {
    return new Ok(do_to_base_string(x, base));
  } else {
    return new Error(undefined);
  }
}

export function to_base2(x) {
  return do_to_base_string(x, 2);
}

export function to_base8(x) {
  return do_to_base_string(x, 8);
}

export function to_base16(x) {
  return do_to_base_string(x, 16);
}

export function to_base36(x) {
  return do_to_base_string(x, 36);
}

export function power(base, exponent) {
  let _pipe = to_float(base);
  return $float.power(_pipe, exponent);
}

export function square_root(x) {
  let _pipe = to_float(x);
  return $float.square_root(_pipe);
}

export function compare(a, b) {
  let $ = a === b;
  if ($) {
    return new $order.Eq();
  } else {
    let $1 = a < b;
    if ($1) {
      return new $order.Lt();
    } else {
      return new $order.Gt();
    }
  }
}

export function min(a, b) {
  let $ = a < b;
  if ($) {
    return a;
  } else {
    return b;
  }
}

export function max(a, b) {
  let $ = a > b;
  if ($) {
    return a;
  } else {
    return b;
  }
}

export function clamp(x, min_bound, max_bound) {
  let _pipe = x;
  let _pipe$1 = min(_pipe, max_bound);
  return max(_pipe$1, min_bound);
}

export function is_even(x) {
  return (remainderInt(x, 2)) === 0;
}

export function is_odd(x) {
  return (remainderInt(x, 2)) !== 0;
}

export function negate(x) {
  return -1 * x;
}

function sum_loop(loop$numbers, loop$initial) {
  while (true) {
    let numbers = loop$numbers;
    let initial = loop$initial;
    if (numbers.atLeastLength(1)) {
      let first = numbers.head;
      let rest = numbers.tail;
      loop$numbers = rest;
      loop$initial = first + initial;
    } else {
      return initial;
    }
  }
}

export function sum(numbers) {
  return sum_loop(numbers, 0);
}

function product_loop(loop$numbers, loop$initial) {
  while (true) {
    let numbers = loop$numbers;
    let initial = loop$initial;
    if (numbers.atLeastLength(1)) {
      let first = numbers.head;
      let rest = numbers.tail;
      loop$numbers = rest;
      loop$initial = first * initial;
    } else {
      return initial;
    }
  }
}

export function product(numbers) {
  return product_loop(numbers, 1);
}

function digits_loop(loop$x, loop$base, loop$acc) {
  while (true) {
    let x = loop$x;
    let base = loop$base;
    let acc = loop$acc;
    let $ = absolute_value(x) < base;
    if ($) {
      return listPrepend(x, acc);
    } else {
      loop$x = divideInt(x, base);
      loop$base = base;
      loop$acc = listPrepend(remainderInt(x, base), acc);
    }
  }
}

export function digits(x, base) {
  let $ = base < 2;
  if ($) {
    return new Error(undefined);
  } else {
    return new Ok(digits_loop(x, base, toList([])));
  }
}

function undigits_loop(loop$numbers, loop$base, loop$acc) {
  while (true) {
    let numbers = loop$numbers;
    let base = loop$base;
    let acc = loop$acc;
    if (numbers.hasLength(0)) {
      return new Ok(acc);
    } else if (numbers.atLeastLength(1) && (numbers.head >= base)) {
      let digit = numbers.head;
      return new Error(undefined);
    } else {
      let digit = numbers.head;
      let rest = numbers.tail;
      loop$numbers = rest;
      loop$base = base;
      loop$acc = acc * base + digit;
    }
  }
}

export function undigits(numbers, base) {
  let $ = base < 2;
  if ($) {
    return new Error(undefined);
  } else {
    return undigits_loop(numbers, base, 0);
  }
}

export function random(max) {
  let _pipe = ($float.random() * to_float(max));
  let _pipe$1 = $float.floor(_pipe);
  return $float.round(_pipe$1);
}

export function divide(dividend, divisor) {
  if (divisor === 0) {
    return new Error(undefined);
  } else {
    let divisor$1 = divisor;
    return new Ok(divideInt(dividend, divisor$1));
  }
}

export function remainder(dividend, divisor) {
  if (divisor === 0) {
    return new Error(undefined);
  } else {
    let divisor$1 = divisor;
    return new Ok(remainderInt(dividend, divisor$1));
  }
}

export function modulo(dividend, divisor) {
  if (divisor === 0) {
    return new Error(undefined);
  } else {
    let remainder$1 = remainderInt(dividend, divisor);
    let $ = remainder$1 * divisor < 0;
    if ($) {
      return new Ok(remainder$1 + divisor);
    } else {
      return new Ok(remainder$1);
    }
  }
}

export function floor_divide(dividend, divisor) {
  if (divisor === 0) {
    return new Error(undefined);
  } else {
    let divisor$1 = divisor;
    let $ = (dividend * divisor$1 < 0) && ((remainderInt(dividend, divisor$1)) !== 0);
    if ($) {
      return new Ok((divideInt(dividend, divisor$1)) - 1);
    } else {
      return new Ok(divideInt(dividend, divisor$1));
    }
  }
}

export function add(a, b) {
  return a + b;
}

export function multiply(a, b) {
  return a * b;
}

export function subtract(a, b) {
  return a - b;
}
