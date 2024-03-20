# Values marked with @internal are not part of the public API and may change
# without notice.

let
  Ok = x0: { __gleam_tag' = "Ok"; _0 = x0; };

  Err = x0: { __gleam_tag' = "Err"; _0 = x0; };

  # @internal
  remainderInt = a: b: if b == 0 then 0 else a - (b * (a / b));

  # @internal
  divideInt = a: b: if b == 0 then 0 else a / b;

  # @internal
  divideFloat = a: b: if b == 0 then 0 else a / b;
in { inherit Ok Err remainderInt divideInt divideFloat; }
