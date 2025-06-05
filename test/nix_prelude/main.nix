let
  inherit
    (import ./prelude.nix)
    Ok
    Error
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
    UtfCodepoint
    stringBits
    codepointBits
    BitArray
    toBitArray
    sizedInt
    byteSize
    byteAt
    binaryFromBitSlice
    intFromBitSlice
    byteArrayToInt
    byteArrayToUtf8String
    ;

  assertEq = a: b: assert ((a == b) || builtins.throw ["Assertion failed:" a "!=" b]); null;
in builtins.deepSeq {
    testRES1 = assert isOk (Ok 5); null;
    testRES2 = assert !(isOk (Error 5)); null;

    testAR1 = assertEq (remainderInt 10 0) 0;
    testAR2 = assertEq (remainderInt 11 3) 2;
    testAR3 = assertEq (remainderInt (-27) 5) (-2);
    testAR4 = assertEq (divideInt 11 0) 0;
    testAR5 = assertEq (divideInt 121 2) 60;
    testAR6 = assertEq (divideFloat 121.5 0.0) 0.0;
    testAR7 = assertEq (divideFloat 121.0 2.0) 60.5;

    testLS1 = assertEq (toList [ 1 2 3 4 ]) (prepend 1 (prepend 2 (prepend 3 (prepend 4 (toList [])))));
    testLS2 = assertEq (toList [ 1 2 3 4 ]).tail.tail.head 3;
    testLS3 = assertEq (listToArray (toList [ 1 2 3 "abc" { a = 5; } ])) [ 1 2 3 "abc" { a = 5; } ];
    testLS4 = assert listIsEmpty (toList [ 1 2 3 4 ]).tail.tail.tail.tail; null;
    testLS5 = assert listHasAtLeastLength (toList [ 1 2 3 4 ]) 4; null;
    testLS6 = assert listHasAtLeastLength (toList [ 1 2 3 4 ]) 0; null;
    testLS7 = assert listHasAtLeastLength (toList [ 1 2 3 4 ]) (-1); null;
    testLS8 = assert !(listHasAtLeastLength (toList [ 1 2 3 4 ]) 5); null;
    testLS9 = assert listHasAtLeastLength (toList []) 0; null;
    testLS10 = assert !(listHasAtLeastLength (toList []) 1); null;
    testLS11 = assert listHasLength (toList [ 1 2 3 4 ]) 4; null;
    testLS12 = assert !(listHasLength (toList [ 1 2 3 4 ]) 3); null;
    testLS13 = assert listHasLength (toList []) 0; null;
    testLS14 = assert !(listHasLength (toList []) 1); null;

    testSTR1 = assert strHasPrefix "ab" "abc"; null;
    testSTR2 = assert strHasPrefix "" "abc"; null;
    testSTR3 = assert !(strHasPrefix "bb" "abc"); null;

    testLT1 = assertEq (parseNumber "0xff") 255;
    testLT2 = assertEq (parseNumber "0b1000") 8;
    testLT3 = assertEq (parseNumber "0o11") 9;
    testLT4 = assertEq (parseNumber "1234") 1234;
    testLT5 = assertEq (parseEscape "\\n") "\n";

    testUTF1 = assertEq (UtfCodepoint 123).value 123;
    testUTF2 =
        assertEq
        (stringBits "héllo [ˈaʳʊ] ℕ ⊆ ℕ₀ ὦ ἄνδρ ⡌ コンニ ░▒▓█")
        [ 104 195 169 108 108 111 32 91 203 136 97 202 179 202 138 93 32 226 132 149 32 226 138 134 32 226 132 149 226 130 128 32 225 189 166 32 225 188 132 206 189 206 180 207 129 32 226 161 140 32 227 130 179 227 131 179 227 131 139 32 226 150 145 226 150 146 226 150 147 226 150 136 ]
        ;
    testUTF3 = assertEq (codepointBits (UtfCodepoint 9608)) [ 226 150 136 ];

    testBIT1 = assertEq (BitArray [ 1 2 3 ]).buffer [ 1 2 3 ];
    # testBIT2 = assertEq (toBitArray [ 1 (-2) 511 256 255 ]).buffer [ 1 254 255 0 255 ];
    testBIT3 = assertEq (toBitArray [ 1 [ 1 2 255 ] 256 [ ] 24 [ 3 ] [ 4 5 ] ]) (BitArray [ 1 1 2 255 0 24 3 4 5 ]);
    testBIT4 = assertEq (sizedInt 8 32) [ 0 0 0 8 ];
    testBIT5 = assertEq (sizedInt 32767 16) [ 127 255 ];
    testBIT6 = assertEq (byteSize (toBitArray [ 1 2 3 4 5 ])) 5;
    testBIT7 = assertEq (byteAt (toBitArray [ 1 2 3 4 5 ]) 3) 4;
    testBIT8 = assertEq (binaryFromBitSlice (toBitArray [ 1 2 3 4 5 ]) 1 3) (BitArray [ 2 3 ]);
    # testBIT9 = assertEq (binaryFromBitSlice (toBitArray [ 1 2 3 4 5 ]) (-99) 99) (BitArray [ 1 2 3 4 5 ]);
    # testBIT10 = assertEq (binaryFromBitSlice (toBitArray [ ]) 1 3) (BitArray [ ]);
    testBIT11 = assertEq (intFromBitSlice (toBitArray [ 1 2 3 ]) 1 3) 515;
    # testBIT12 = assertEq (intFromBitSlice (toBitArray [ 1 2 3 ]) (-99) 99) 66051;
    # testBIT13 = assertEq (intFromBitSlice (toBitArray [ ]) 1 3) 0;
    testBIT14 = assertEq (byteArrayToInt (toBitArray [ 1 2 3 ])) 66051;
    testBIT15 = assertEq (byteArrayToInt (toBitArray [ ])) 0;
    testBIT16 = assertEq (byteArrayToUtf8String (toBitArray [ 226 150 136 65 ])) "█A";
} null
