import * as $string from "../gleam/string.mjs";
import {
  print,
  print_error,
  console_log as println,
  console_error as println_error,
  print_debug as do_debug_println,
} from "../gleam_stdlib.mjs";

export { print, print_error, println, println_error };

export function debug(term) {
  let _pipe = term;
  let _pipe$1 = $string.inspect(_pipe);
  do_debug_println(_pipe$1)
  return term;
}
