# Values marked with @internal are not part of the public API and may change
# without notice.

let
  Ok = x0: { __gleamTag = "Ok"; _0 = x0; };

  Error = x0: { __gleamTag = "Error"; _0 = x0; };

  isOk = res: res.__gleamTag == "Ok";

  # @internal
  remainderInt = a: b: if b == 0 then 0 else a - (b * (a / b));

  # @internal
  divideInt = a: b: if b == 0 then 0 else a / b;

  # @internal
  divideFloat = a: b: if b == 0 then 0 else a / b;

  toList = foldr prepend { __gleamTag = "Empty"; __gleamBuiltIn = "List"; };

  prepend = head: tail: { __gleamTag = "NotEmpty"; __gleamBuiltIn = "List"; inherit head tail; };

  listIsEmpty = list: list.__gleamTag == "Empty";

  listToArray = list: if list.__gleamTag == "Empty" then [] else [ list.head ] ++ listToArray list.tail;

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

  # @internal
  # Strictly evaluates each expression and returns the second parameter.
  seqAll =
    exprs: returning:
      builtins.seq
        (builtins.foldl' (acc: elem: builtins.seq elem acc) null exprs)
        returning;

  # --- UTF-8 ---

  UtfCodepoint =
    value:
      {
        __gleamTag = "UtfCodepoint";
        __gleamBuiltIn = "UtfCodepoint";
        inherit value;
      };

  decToHex =
    let
      digitMap = [ "0" "1" "2" "3" "4" "5" "6" "7" "8" "9" "A" "B" "C" "D" "E" "F" ];
      remHex = n: n - (16 * (n / 16));
    in
      n:
        let
          lastDigitValue = remHex n;
          lastDigit = builtins.elemAt digitMap lastDigitValue;
          otherDigits = n / 16;
        in
          if n < 16
          then lastDigit
          else decToHex otherDigits + lastDigit;

  # @internal
  # Converts a codepoint's integer value to its UTF-8 string representation
  # by invoking a \U(hex) escape sequence within TOML and reading it.
  # Using TOML over JSON is necessary because JSON restricts the \u escape
  # sequence to up to 4 hex digits instead of 6, requiring workarounds.
  intCodepointToStringInternal =
    n:
      let
        hex = decToHex n;
        zeroes = builtins.substring 0 (8 - (builtins.stringLength hex)) "00000000";
      in (builtins.fromTOML "x = \"\\U${zeroes}${hex}\"").x;


  # @internal
  asciiChars = builtins.genList (i: intCodepointToStringInternal i) 128;

  # @internal
  # See comment at 'intCodepointToStringInternal'.
  # Also applies a fast ASCII table lookup if possible.
  intCodepointToString =
    n:
      if n < 128
      then builtins.elemAt asciiChars n
      else intCodepointToStringInternal n;

  # @internal
  # Prepares a table mapping each possible UTF-8 byte as a string to the corresponding integers.
  # Valid UTF-8 byte integers include 0-243, excluding 192 and 193.
  # What this function does is generate one UTF-8 codepoint for each valid UTF-8 byte and extract
  # a single byte from that codepoint's representation as a string.
  # We then map each pair (char, value) through the provided function. For example, a pair can be
  # mapped to '{ name = char; value = value; }' in order to be able to use 'builtins.listToAttrs'
  # to generate an attribute set mapping each single UTF-8 byte string to its integer value.
  # Then, we return arrays corresponding to each kind of UTF-8 byte
  # (ASCII, 10..., 110..., 1110... or 11110...). This is because it might be necessary to insert some
  # padding between 10... and 110... characters before joining them into a single array, given that bytes
  # 192 and 193, a.k.a. 0b1100_0000 and 0b1100_0001, do not appear in any valid UTF-8 codepoint's representation.
  utf8ByteTableGen = charValueMapper:
    let
      charAt = i: builtins.substring i 1;
      first10Codepoint = 8 * 16; # 0x0080
      first110Codepoint = 8 * 16; # 0x0080
      first1110Codepoint = 8 * 16 * 16; # 0x0800
      first11110Codepoint = 16 * 16 * 16 * 16; # 0x10000
      minInvalidChar = 55296; # 0xd800 - 0xdfff (57343) are invalid UTF-8
      asciiBytes = builtins.genList (i: charValueMapper { char = builtins.elemAt asciiChars i; value = i; }) 128;

      # Generator for arrays of UTF-8 bytes following a certain pattern. Generates the codepoints
      # necessary to extract their bytes. 'byteIndex' is the index of the byte to extract from each
      # generated codepoint; 'k_i' is a function which converts the current iteration index to the
      # corresponding codepoint (usually in the form 'firstCodepointInRange + offsetToIncreaseByte * i');
      # 'v_i' is a function which returns the actual numeric value of the byte we expect to extract
      # (usually in the form 'firstPossibleByte + i'); and 'max_i' is the maximum iteration number
      # (amount of bytes to generate - 1).
      codepointGen =
        { byteIndex, k_i, v_i, max_i }:
          let
            gen =
              i:
                let
                  code = k_i i;

                  # It appears that 0xd800 is actually reached in our algorithm for
                  # bytes starting with 1110... after 13 iterations.
                  # This results in a missing 237 byte (with leading 'd' in hexadecimal).
                  # Therefore, go back to 0xd799 so we still get 'd'.
                  actualCode =
                    if code == minInvalidChar
                    then minInvalidChar - 1
                    else code;

                  # Obtain the byte by converting the codepoint to a string and obtaining
                  # the byte at the relevant index.
                  character = charAt byteIndex (intCodepointToStringInternal actualCode);
                in charValueMapper { char = character; value = v_i i; };
          in builtins.genList gen (max_i + 1);

      # 0x0080 + 0..63, converted to string, will generate all possible 0x10... bytes at the second byte.
      startsWith10 =
        codepointGen { byteIndex = 1; k_i = i: first10Codepoint + i; v_i = i: first10Codepoint + i; max_i = 63; };

      # 0x0080 + 64 * (0..29), converted to string, will generate all possible 0x110... bytes at the first byte.
      startsWith110 =
        codepointGen { byteIndex = 0; k_i = i: first110Codepoint + 64 * i; v_i = i: 194 + i; max_i = 29; };

      # 0x0800 + 4096 * (0..15), converted to string, will generate all possible 0x1110... bytes at the first byte.
      startsWith1110 =
        codepointGen { byteIndex = 0; k_i = i: first1110Codepoint + 4096 * i; v_i = i: 224 + i; max_i = 15; };

      # 0x10000 + 262144 * (0..3), converted to string, will generate all possible 0x11110... bytes at the first byte.
      startsWith11110 =
        codepointGen { byteIndex = 0; k_i = i: first11110Codepoint + 262144 * i; v_i = i: 240 + i; max_i = 3; };
    in { inherit asciiBytes startsWith10 startsWith110 startsWith1110 startsWith11110; };

  # @internal
  # Attribute set mapping each possible UTF-8 byte as a string to its integer value.
  # Used to quickly map bytes in a string to integers.
  utf8ByteTable =
    let
      # Convert each (char, value) to a format understood by 'builtins.listToAttrs'.
      gen = { char, value }: { name = char; inherit value; };

      # Join all UTF-8 byte kinds into one large array of name/value pairs.
      listsToAttrsList =
        { asciiBytes, startsWith10, startsWith110, startsWith1110, startsWith11110 }:
          asciiBytes ++ startsWith10 ++ startsWith110 ++ startsWith1110 ++ startsWith11110;
    in builtins.listToAttrs (listsToAttrsList (utf8ByteTableGen gen));

  # @internal
  # The inverse of 'utf8ByteTable'.
  # Contains a list with all possible UTF-8 bytes. Their indices correspond to their integer values.
  # As such, one can index into this list to convert an integer to the corresponding UTF-8 byte.
  # Note that only indices up to 243 are valid, excluding 192 and 193.
  utf8ByteInvTable =
    let
      # In the resulting arrays, just keep the byte strings, as we'll use list indexing instead of
      # attribute sets.
      gen = { char, value }: char;

      # Bytes 192 and 193 don't exist, so we insert two empty strings at their positions
      # to "pad" indices. All other bytes (up to 243) are present and sorted in ascending order.
      supplement = [ "" "" ];

      # Join all the lists of UTF-8 byte kinds, with the padding above where 192 and 193 would be.
      genList =
        { asciiBytes, startsWith10, startsWith110, startsWith1110, startsWith11110 }:
          asciiBytes ++ startsWith10 ++ supplement ++ startsWith110 ++ startsWith1110 ++ startsWith11110;
    in genList (utf8ByteTableGen gen);

  # @internal
  # Convert a string to an array of UTF-8 bytes as unsigned 8-bit integers.
  stringBits =
    s:
      let
        charAt = n: builtins.substring n 1 s;

        # Invalid UTF-8 bytes are represented as 0 instead of throwing.
        byteStringToInt = char: utf8ByteTable."${char}" or 0;
      in
        builtins.genList (i: byteStringToInt (charAt i)) (builtins.stringLength s);

  # @internal
  # Convert a codepoint to an array of UTF-8 bytes as unsigned 8-bit integers.
  codepointBits =
    let
      last1Byte = 127;  # 0x007f
      last2Bytes = 2047;  # 0x07ff
      last3Bytes = 65535;  # 0xffff
      last4Bytes = 1114111;  # 0x10ffff
      oneOneHeader = 128;  # 0b1000_0000
      twoOnesHeader = 128 + 64;  # 0b1100_0000
      threeOnesHeader = 128 + 64 + 32;  # 0b1110_0000
      fourOnesHeader = 128 + 64 + 32 + 16;  # 0b1111_0000
      withMask = builtins.bitAnd;
      maskHalfByte0 = withMask 15;                                 # 0x00000f
      maskHalfByte1h1 = n: (withMask 48 n) / 16;                   # 0x000030 (lower half) >> 4
      maskHalfByte1h2 = n: (withMask 192 n) / (16*4);              # 0x0000c0 (higher half) >> 6
      maskHalfByte2 = n: (withMask 3840 n) / (16*16);              # 0x000f00 >> 16
      maskHalfByte3 = n: (withMask 61440 n) / (16*16*16);          # 0x00f000 >> 16
      maskHalfByte4h1 = n: (withMask 196608 n) / (16*16*16*16);    # 0x030000 (lower half) >> 64
      maskHalfByte4h2 = n: (withMask 786432 n) / (16*16*16*16*4);  # 0x0c0000 (higher half) >> 96
      maskHalfByte5 = n: (withMask 15728640 n) / (16*16*16*16*16); # 0xf00000 >> 256
    in
      c:
        if c <= last1Byte
        then [ c ]
        else if c > last4Bytes
        then [ 0 ]
        else if c <= last2Bytes
        then
          [
            # (110)22211 (10)110000
            (twoOnesHeader + (maskHalfByte2 c) * 4 + (maskHalfByte1h2 c))
            (oneOneHeader + (maskHalfByte1h1 c) * 16 + (maskHalfByte0 c))
          ]
        else if c <= last3Bytes
        then
          [
            # (1110)3333 (10)222211 (10)110000
            (threeOnesHeader + (maskHalfByte3 c))
            (oneOneHeader + (maskHalfByte2 c) * 4 + (maskHalfByte1h2 c))
            (oneOneHeader + (maskHalfByte1h1 c) * 16 + (maskHalfByte0 c))
          ]
        else
          [
            # (11110)544 (10)443333 (10)222211 (10)110000
            (fourOnesHeader + (maskHalfByte5 c) * 4 + (maskHalfByte4h2 c))
            (oneOneHeader + (maskHalfByte4h1 c) * 16 + (maskHalfByte3 c))
            (oneOneHeader + (maskHalfByte2 c) * 4 + (maskHalfByte1h2 c))
            (oneOneHeader + (maskHalfByte1h1 c) * 16 + (maskHalfByte0 c))
          ];

  # --- bit array ---

  BitArray =
    buffer:
      if !(builtins.isList buffer)
      then builtins.throw "Bit arrays can only be constructed from Nix lists"
      else
        { __gleamTag = "BitArray"; __gleamBuiltIn = "BitArray"; inherit buffer; };

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
    UtfCodepoint
    BitArray
    isOk
    remainderInt
    divideInt
    divideFloat
    toList
    prepend
    listIsEmpty
    listToArray
    listHasAtLeastLength
    listHasLength
    strHasPrefix
    parseNumber
    parseEscape
    seqAll
    stringBits
    codepointBits
    sizedInt
    toBitArray
    bitArrayByteSize;
}
