# Values marked with @internal are not part of the public API and may change
# without notice.

let
  Ok = x0: { __gleam_tag' = "Ok"; _0 = x0; };

  Error = x0: { __gleam_tag' = "Error"; _0 = x0; };

  # @internal
  remainderInt = a: b: if b == 0 then 0 else a - (b * (a / b));

  # @internal
  divideInt = a: b: if b == 0 then 0 else a / b;

  # @internal
  divideFloat = a: b: if b == 0 then 0 else a / b;

  toList = foldr prepend { __gleam_tag' = "Empty"; };

  prepend = head: tail: { __gleam_tag' = "NotEmpty"; inherit head tail; };

  # @internal
  listIsEmpty = lst: lst.__gleam_tag' == "Empty";

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
  foldr = fun: init: lst:
    let
      len = builtins.length lst;
      fold' = index:
        if index == len
        then init
        else fun (builtins.elemAt lst index) (fold' (index + 1));
    in fold' 0;

  # @internal
  strHasPrefix =
    prefix: haystack:
      prefix == (builtins.substring 0 (builtins.stringLength prefix) haystack);

  # @internal
  parseTOML = value: (builtins.fromTOML "x = ${value}").x;

  # @internal
  parseNumber =
    format:
      let
        hasMinus = strHasPrefix "-" format;
        numberToParse =
          if hasMinus
          then builtins.substring 1 (-1) format
          else format;
        parsedNumber = parseTOML numberToParse;
      in if hasMinus then -parsedNumber else parsedNumber;

  # @internal
  parseEscape = content: parseTOML "\"${content}\"";

  # --- bit array ---
  BitArray =
    buffer:
      if !(builtins.isList buffer)
      then builtins.throw "Bit arrays can only be constructed from Nix lists"
      else
        { __gleam_tag' = "BitArray"; inherit buffer; };

  # @internal
  # Repeats an element 'n' times in a Nix list.
  repeated = x: n: builtins.genList (_: x) n;

  # @internal
  sizedInt =
    int: size:
      if size <= 0
      then []
      else if remainderInt size 8 != 0
      then builtins.throw "Bit arrays must be byte aligned on Nix, got size of ${builtins.toString(size)} bits"
      else
        let
          byteArray = repeated 0 (size / 8);
          foldFun =
            acc: elem:
              let
                value = acc.value;
                arr = acc.arr;
                byte = builtins.bitAnd value 255;
              in { value = (value - byte) / 256; arr = [ byte ] ++ arr; };
        in (builtins.foldl' foldFun { value = int; arr = []; } byteArray).arr;

  # @internal
  toBitArray =
    segments:
      let
        intoBuffer = elem:
          if builtins.isList elem
          then elem
          else if builtins.isInt elem
          then [ (remainderInt elem 256) ]
          else [ elem ];
        buffer = builtins.concatMap intoBuffer segments;
      in BitArray buffer;

  # Get the amount of bytes in the bitarray.
  bitArrayByteSize = array: builtins.length array.buffer;

in {
  inherit
    Ok
    Error
    BitArray
    remainderInt
    divideInt
    divideFloat
    toList
    prepend
    listHasAtLeastLength
    listHasLength
    strHasPrefix
    parseNumber
    parseEscape
    sizedInt
    toBitArray
    bitArrayByteSize;
}
