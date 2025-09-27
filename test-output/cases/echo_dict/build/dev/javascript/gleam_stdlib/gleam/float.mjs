import { Ok, Error, divideFloat } from "../gleam.mjs";
import * as $order from "../gleam/order.mjs";
import {
  parse_float as parse,
  float_to_string as to_string,
  ceiling,
  floor,
  round as js_round,
  truncate,
  identity as do_to_float,
  power as do_power,
  random_uniform as random,
  log as do_log,
  exp as exponential,
} from "../gleam_stdlib.mjs";

export { ceiling, exponential, floor, parse, random, to_string, truncate };

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

export function absolute_value(x) {
  let $ = x >= 0.0;
  if ($) {
    return x;
  } else {
    return 0.0 - x;
  }
}

export function loosely_compare(a, b, tolerance) {
  let difference = absolute_value(a - b);
  let $ = difference <= tolerance;
  if ($) {
    return new $order.Eq();
  } else {
    return compare(a, b);
  }
}

export function loosely_equals(a, b, tolerance) {
  let difference = absolute_value(a - b);
  return difference <= tolerance;
}

export function power(base, exponent) {
  let fractional = (ceiling(exponent) - exponent) > 0.0;
  let $ = ((base < 0.0) && fractional) || ((base === 0.0) && (exponent < 0.0));
  if ($) {
    return new Error(undefined);
  } else {
    return new Ok(do_power(base, exponent));
  }
}

export function square_root(x) {
  return power(x, 0.5);
}

export function negate(x) {
  return -1.0 * x;
}

export function round(x) {
  let $ = x >= 0.0;
  if ($) {
    return js_round(x);
  } else {
    return 0 - js_round(negate(x));
  }
}

export function to_precision(x, precision) {
  let $ = precision <= 0;
  if ($) {
    let factor = do_power(10.0, do_to_float(- precision));
    return do_to_float(round(divideFloat(x, factor))) * factor;
  } else {
    let factor = do_power(10.0, do_to_float(precision));
    return divideFloat(do_to_float(round(x * factor)), factor);
  }
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
  return sum_loop(numbers, 0.0);
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
  return product_loop(numbers, 1.0);
}

export function modulo(dividend, divisor) {
  if (divisor === 0.0) {
    return new Error(undefined);
  } else {
    return new Ok(dividend - (floor(divideFloat(dividend, divisor)) * divisor));
  }
}

export function divide(a, b) {
  if (b === 0.0) {
    return new Error(undefined);
  } else {
    let b$1 = b;
    return new Ok(divideFloat(a, b$1));
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

export function logarithm(x) {
  let $ = x <= 0.0;
  if ($) {
    return new Error(undefined);
  } else {
    return new Ok(do_log(x));
  }
}
