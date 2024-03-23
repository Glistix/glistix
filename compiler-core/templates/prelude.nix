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

  toList = foldr prepend { __gleam_tag' = "Empty"; };

  prepend = head: tail: { __gleam_tag' = "NotEmpty"; inherit head tail; };

  # @internal
  listHasAtLeastLength =
    lst: len:
      len <= 0 || !(listIsEmpty lst) && listHasAtLeastLength lst.tail (len - 1);

  # @internal
  listHasLength =
    lst: len:
      if listIsEmpty lst
      then len == 0
      else len > 0 && listHasLength lst.tail (len - 1);

  # @internal
  listIsEmpty = lst: lst.__gleam_tag' == "Empty";

  # @internal
  foldr = fun: init: lst:
    let
      len = builtins.length lst;
      fold' = index:
        if index == len
        then init
        else fun (builtins.elemAt lst index) (fold' (index + 1));
    in fold' 0;
in { inherit Ok Err remainderInt divideInt divideFloat toList prepend listHasAtLeastLength listHasLength; }
