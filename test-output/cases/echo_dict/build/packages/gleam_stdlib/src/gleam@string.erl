-module(gleam@string).
-compile([no_auto_import, nowarn_unused_vars, nowarn_unused_function, nowarn_nomatch]).

-export([is_empty/1, length/1, reverse/1, replace/3, lowercase/1, uppercase/1, compare/2, slice/3, crop/2, drop_end/2, contains/2, starts_with/2, ends_with/2, split_once/2, append/2, concat/1, repeat/2, join/2, pad_start/3, pad_end/3, trim_start/1, trim_end/1, trim/1, pop_grapheme/1, drop_start/2, to_graphemes/1, split/2, to_utf_codepoints/1, from_utf_codepoints/1, utf_codepoint/1, utf_codepoint_to_int/1, to_option/1, first/1, last/1, capitalise/1, inspect/1, byte_size/1]).
-export_type([direction/0]).

-if(?OTP_RELEASE >= 27).
-define(MODULEDOC(Str), -moduledoc(Str)).
-define(DOC(Str), -doc(Str)).
-else.
-define(MODULEDOC(Str), -compile([])).
-define(DOC(Str), -compile([])).
-endif.

?MODULEDOC(
    " Strings in Gleam are UTF-8 binaries. They can be written in your code as\n"
    " text surrounded by `\"double quotes\"`.\n"
).

-type direction() :: leading | trailing.

-file("src/gleam/string.gleam", 23).
?DOC(
    " Determines if a `String` is empty.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " is_empty(\"\")\n"
    " // -> True\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " is_empty(\"the world\")\n"
    " // -> False\n"
    " ```\n"
).
-spec is_empty(binary()) -> boolean().
is_empty(Str) ->
    Str =:= <<""/utf8>>.

-file("src/gleam/string.gleam", 51).
?DOC(
    " Gets the number of grapheme clusters in a given `String`.\n"
    "\n"
    " This function has to iterate across the whole string to count the number of\n"
    " graphemes, so it runs in linear time.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " length(\"Gleam\")\n"
    " // -> 5\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " length(\"ß↑e̊\")\n"
    " // -> 3\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " length(\"\")\n"
    " // -> 0\n"
    " ```\n"
).
-spec length(binary()) -> integer().
length(String) ->
    string:length(String).

-file("src/gleam/string.gleam", 65).
?DOC(
    " Reverses a `String`.\n"
    "\n"
    " This function has to iterate across the whole `String` so it runs in linear\n"
    " time.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " reverse(\"stressed\")\n"
    " // -> \"desserts\"\n"
    " ```\n"
).
-spec reverse(binary()) -> binary().
reverse(String) ->
    _pipe = String,
    _pipe@1 = gleam_stdlib:identity(_pipe),
    _pipe@2 = string:reverse(_pipe@1),
    unicode:characters_to_binary(_pipe@2).

-file("src/gleam/string.gleam", 86).
?DOC(
    " Creates a new `String` by replacing all occurrences of a given substring.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " replace(\"www.example.com\", each: \".\", with: \"-\")\n"
    " // -> \"www-example-com\"\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " replace(\"a,b,c,d,e\", each: \",\", with: \"/\")\n"
    " // -> \"a/b/c/d/e\"\n"
    " ```\n"
).
-spec replace(binary(), binary(), binary()) -> binary().
replace(String, Pattern, Substitute) ->
    _pipe = String,
    _pipe@1 = gleam_stdlib:identity(_pipe),
    _pipe@2 = gleam_stdlib:string_replace(_pipe@1, Pattern, Substitute),
    unicode:characters_to_binary(_pipe@2).

-file("src/gleam/string.gleam", 111).
?DOC(
    " Creates a new `String` with all the graphemes in the input `String` converted to\n"
    " lowercase.\n"
    "\n"
    " Useful for case-insensitive comparisons.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " lowercase(\"X-FILES\")\n"
    " // -> \"x-files\"\n"
    " ```\n"
).
-spec lowercase(binary()) -> binary().
lowercase(String) ->
    string:lowercase(String).

-file("src/gleam/string.gleam", 127).
?DOC(
    " Creates a new `String` with all the graphemes in the input `String` converted to\n"
    " uppercase.\n"
    "\n"
    " Useful for case-insensitive comparisons and VIRTUAL YELLING.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " uppercase(\"skinner\")\n"
    " // -> \"SKINNER\"\n"
    " ```\n"
).
-spec uppercase(binary()) -> binary().
uppercase(String) ->
    string:uppercase(String).

-file("src/gleam/string.gleam", 145).
?DOC(
    " Compares two `String`s to see which is \"larger\" by comparing their graphemes.\n"
    "\n"
    " This does not compare the size or length of the given `String`s.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " compare(\"Anthony\", \"Anthony\")\n"
    " // -> order.Eq\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " compare(\"A\", \"B\")\n"
    " // -> order.Lt\n"
    " ```\n"
).
-spec compare(binary(), binary()) -> gleam@order:order().
compare(A, B) ->
    case A =:= B of
        true ->
            eq;

        _ ->
            case gleam_stdlib:less_than(A, B) of
                true ->
                    lt;

                false ->
                    gt
            end
    end.

-file("src/gleam/string.gleam", 190).
?DOC(
    " Takes a substring given a start grapheme index and a length. Negative indexes\n"
    " are taken starting from the *end* of the list.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " slice(from: \"gleam\", at_index: 1, length: 2)\n"
    " // -> \"le\"\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " slice(from: \"gleam\", at_index: 1, length: 10)\n"
    " // -> \"leam\"\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " slice(from: \"gleam\", at_index: 10, length: 3)\n"
    " // -> \"\"\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " slice(from: \"gleam\", at_index: -2, length: 2)\n"
    " // -> \"am\"\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " slice(from: \"gleam\", at_index: -12, length: 2)\n"
    " // -> \"\"\n"
    " ```\n"
).
-spec slice(binary(), integer(), integer()) -> binary().
slice(String, Idx, Len) ->
    case Len < 0 of
        true ->
            <<""/utf8>>;

        false ->
            case Idx < 0 of
                true ->
                    Translated_idx = string:length(String) + Idx,
                    case Translated_idx < 0 of
                        true ->
                            <<""/utf8>>;

                        false ->
                            gleam_stdlib:slice(String, Translated_idx, Len)
                    end;

                false ->
                    gleam_stdlib:slice(String, Idx, Len)
            end
    end.

-file("src/gleam/string.gleam", 223).
?DOC(
    " Drops contents of the first `String` that occur before the second `String`.\n"
    " If the `from` string does not contain the `before` string, `from` is returned unchanged.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " crop(from: \"The Lone Gunmen\", before: \"Lone\")\n"
    " // -> \"Lone Gunmen\"\n"
    " ```\n"
).
-spec crop(binary(), binary()) -> binary().
crop(String, Substring) ->
    gleam_stdlib:crop_string(String, Substring).

-file("src/gleam/string.gleam", 254).
?DOC(
    " Drops *n* graphemes from the end of a `String`.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " drop_end(from: \"Cigarette Smoking Man\", up_to: 2)\n"
    " // -> \"Cigarette Smoking M\"\n"
    " ```\n"
).
-spec drop_end(binary(), integer()) -> binary().
drop_end(String, Num_graphemes) ->
    case Num_graphemes < 0 of
        true ->
            String;

        false ->
            slice(String, 0, string:length(String) - Num_graphemes)
    end.

-file("src/gleam/string.gleam", 282).
?DOC(
    " Checks if the first `String` contains the second.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " contains(does: \"theory\", contain: \"ory\")\n"
    " // -> True\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " contains(does: \"theory\", contain: \"the\")\n"
    " // -> True\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " contains(does: \"theory\", contain: \"THE\")\n"
    " // -> False\n"
    " ```\n"
).
-spec contains(binary(), binary()) -> boolean().
contains(Haystack, Needle) ->
    gleam_stdlib:contains_string(Haystack, Needle).

-file("src/gleam/string.gleam", 295).
?DOC(
    " Checks whether the first `String` starts with the second one.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " starts_with(\"theory\", \"ory\")\n"
    " // -> False\n"
    " ```\n"
).
-spec starts_with(binary(), binary()) -> boolean().
starts_with(String, Prefix) ->
    gleam_stdlib:string_starts_with(String, Prefix).

-file("src/gleam/string.gleam", 308).
?DOC(
    " Checks whether the first `String` ends with the second one.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " ends_with(\"theory\", \"ory\")\n"
    " // -> True\n"
    " ```\n"
).
-spec ends_with(binary(), binary()) -> boolean().
ends_with(String, Suffix) ->
    gleam_stdlib:string_ends_with(String, Suffix).

-file("src/gleam/string.gleam", 347).
?DOC(
    " Splits a `String` a single time on the given substring.\n"
    "\n"
    " Returns an `Error` if substring not present.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " split_once(\"home/gleam/desktop/\", on: \"/\")\n"
    " // -> Ok(#(\"home\", \"gleam/desktop/\"))\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " split_once(\"home/gleam/desktop/\", on: \"?\")\n"
    " // -> Error(Nil)\n"
    " ```\n"
).
-spec split_once(binary(), binary()) -> {ok, {binary(), binary()}} |
    {error, nil}.
split_once(String, Substring) ->
    case string:split(String, Substring) of
        [First, Rest] ->
            {ok, {First, Rest}};

        _ ->
            {error, nil}
    end.

-file("src/gleam/string.gleam", 373).
?DOC(
    " Creates a new `String` by joining two `String`s together.\n"
    "\n"
    " This function copies both `String`s and runs in linear time. If you find\n"
    " yourself joining `String`s frequently consider using the [`string_tree`](../gleam/string_tree.html)\n"
    " module as it can append `String`s much faster!\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " append(to: \"butter\", suffix: \"fly\")\n"
    " // -> \"butterfly\"\n"
    " ```\n"
).
-spec append(binary(), binary()) -> binary().
append(First, Second) ->
    _pipe = First,
    _pipe@1 = gleam_stdlib:identity(_pipe),
    _pipe@2 = gleam@string_tree:append(_pipe@1, Second),
    unicode:characters_to_binary(_pipe@2).

-file("src/gleam/string.gleam", 393).
?DOC(
    " Creates a new `String` by joining many `String`s together.\n"
    "\n"
    " This function copies both `String`s and runs in linear time. If you find\n"
    " yourself joining `String`s frequently consider using the [`string_tree`](../gleam/string_tree.html)\n"
    " module as it can append `String`s much faster!\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " concat([\"never\", \"the\", \"less\"])\n"
    " // -> \"nevertheless\"\n"
    " ```\n"
).
-spec concat(list(binary())) -> binary().
concat(Strings) ->
    _pipe = Strings,
    _pipe@1 = gleam_stdlib:identity(_pipe),
    unicode:characters_to_binary(_pipe@1).

-file("src/gleam/string.gleam", 414).
-spec repeat_loop(binary(), integer(), binary()) -> binary().
repeat_loop(String, Times, Acc) ->
    case Times =< 0 of
        true ->
            Acc;

        false ->
            repeat_loop(String, Times - 1, <<Acc/binary, String/binary>>)
    end.

-file("src/gleam/string.gleam", 410).
?DOC(
    " Creates a new `String` by repeating a `String` a given number of times.\n"
    "\n"
    " This function runs in linear time.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " repeat(\"ha\", times: 3)\n"
    " // -> \"hahaha\"\n"
    " ```\n"
).
-spec repeat(binary(), integer()) -> binary().
repeat(String, Times) ->
    repeat_loop(String, Times, <<""/utf8>>).

-file("src/gleam/string.gleam", 433).
?DOC(
    " Joins many `String`s together with a given separator.\n"
    "\n"
    " This function runs in linear time.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " join([\"home\",\"evan\",\"Desktop\"], with: \"/\")\n"
    " // -> \"home/evan/Desktop\"\n"
    " ```\n"
).
-spec join(list(binary()), binary()) -> binary().
join(Strings, Separator) ->
    _pipe = Strings,
    _pipe@1 = gleam@list:intersperse(_pipe, Separator),
    concat(_pipe@1).

-file("src/gleam/string.gleam", 505).
-spec padding(integer(), binary()) -> binary().
padding(Size, Pad_string) ->
    Pad_string_length = string:length(Pad_string),
    Num_pads = case Pad_string_length of
        0 -> 0;
        Gleam@denominator -> Size div Gleam@denominator
    end,
    Extra = case Pad_string_length of
        0 -> 0;
        Gleam@denominator@1 -> Size rem Gleam@denominator@1
    end,
    <<(repeat(Pad_string, Num_pads))/binary,
        (slice(Pad_string, 0, Extra))/binary>>.

-file("src/gleam/string.gleam", 458).
?DOC(
    " Pads the start of a `String` until it has a given length.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " pad_start(\"121\", to: 5, with: \".\")\n"
    " // -> \"..121\"\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " pad_start(\"121\", to: 3, with: \".\")\n"
    " // -> \"121\"\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " pad_start(\"121\", to: 2, with: \".\")\n"
    " // -> \"121\"\n"
    " ```\n"
).
-spec pad_start(binary(), integer(), binary()) -> binary().
pad_start(String, Desired_length, Pad_string) ->
    Current_length = string:length(String),
    To_pad_length = Desired_length - Current_length,
    case To_pad_length =< 0 of
        true ->
            String;

        false ->
            <<(padding(To_pad_length, Pad_string))/binary, String/binary>>
    end.

-file("src/gleam/string.gleam", 491).
?DOC(
    " Pads the end of a `String` until it has a given length.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " pad_end(\"123\", to: 5, with: \".\")\n"
    " // -> \"123..\"\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " pad_end(\"123\", to: 3, with: \".\")\n"
    " // -> \"123\"\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " pad_end(\"123\", to: 2, with: \".\")\n"
    " // -> \"123\"\n"
    " ```\n"
).
-spec pad_end(binary(), integer(), binary()) -> binary().
pad_end(String, Desired_length, Pad_string) ->
    Current_length = string:length(String),
    To_pad_length = Desired_length - Current_length,
    case To_pad_length =< 0 of
        true ->
            String;

        false ->
            <<String/binary, (padding(To_pad_length, Pad_string))/binary>>
    end.

-file("src/gleam/string.gleam", 549).
?DOC(
    " Removes whitespace at the start of a `String`.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " trim_start(\"  hats  \\n\")\n"
    " // -> \"hats  \\n\"\n"
    " ```\n"
).
-spec trim_start(binary()) -> binary().
trim_start(String) ->
    string:trim(String, leading).

-file("src/gleam/string.gleam", 563).
?DOC(
    " Removes whitespace at the end of a `String`.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " trim_end(\"  hats  \\n\")\n"
    " // -> \"  hats\"\n"
    " ```\n"
).
-spec trim_end(binary()) -> binary().
trim_end(String) ->
    string:trim(String, trailing).

-file("src/gleam/string.gleam", 527).
?DOC(
    " Removes whitespace on both sides of a `String`.\n"
    "\n"
    " Whitespace in this function is the set of nonbreakable whitespace\n"
    " codepoints, defined as Pattern_White_Space in [Unicode Standard Annex #31][1].\n"
    "\n"
    " [1]: https://unicode.org/reports/tr31/\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " trim(\"  hats  \\n\")\n"
    " // -> \"hats\"\n"
    " ```\n"
).
-spec trim(binary()) -> binary().
trim(String) ->
    _pipe = String,
    _pipe@1 = trim_start(_pipe),
    trim_end(_pipe@1).

-file("src/gleam/string.gleam", 588).
?DOC(
    " Splits a non-empty `String` into its first element (head) and rest (tail).\n"
    " This lets you pattern match on `String`s exactly as you would with lists.\n"
    "\n"
    " Note on JavaScript using the function to iterate over a string will likely\n"
    " be slower than using `to_graphemes` due to string slicing being more\n"
    " expensive on JavaScript than Erlang.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " pop_grapheme(\"gleam\")\n"
    " // -> Ok(#(\"g\", \"leam\"))\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " pop_grapheme(\"\")\n"
    " // -> Error(Nil)\n"
    " ```\n"
).
-spec pop_grapheme(binary()) -> {ok, {binary(), binary()}} | {error, nil}.
pop_grapheme(String) ->
    gleam_stdlib:string_pop_grapheme(String).

-file("src/gleam/string.gleam", 234).
?DOC(
    " Drops *n* graphemes from the start of a `String`.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " drop_start(from: \"The Lone Gunmen\", up_to: 2)\n"
    " // -> \"e Lone Gunmen\"\n"
    " ```\n"
).
-spec drop_start(binary(), integer()) -> binary().
drop_start(String, Num_graphemes) ->
    case Num_graphemes > 0 of
        false ->
            String;

        true ->
            case gleam_stdlib:string_pop_grapheme(String) of
                {ok, {_, String@1}} ->
                    drop_start(String@1, Num_graphemes - 1);

                {error, nil} ->
                    String
            end
    end.

-file("src/gleam/string.gleam", 604).
-spec to_graphemes_loop(binary(), list(binary())) -> list(binary()).
to_graphemes_loop(String, Acc) ->
    case gleam_stdlib:string_pop_grapheme(String) of
        {ok, {Grapheme, Rest}} ->
            to_graphemes_loop(Rest, [Grapheme | Acc]);

        {error, _} ->
            Acc
    end.

-file("src/gleam/string.gleam", 599).
?DOC(
    " Converts a `String` to a list of\n"
    " [graphemes](https://en.wikipedia.org/wiki/Grapheme).\n"
    "\n"
    " ```gleam\n"
    " to_graphemes(\"abc\")\n"
    " // -> [\"a\", \"b\", \"c\"]\n"
    " ```\n"
).
-spec to_graphemes(binary()) -> list(binary()).
to_graphemes(String) ->
    _pipe = to_graphemes_loop(String, []),
    lists:reverse(_pipe).

-file("src/gleam/string.gleam", 319).
?DOC(
    " Creates a list of `String`s by splitting a given string on a given substring.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " split(\"home/gleam/desktop/\", on: \"/\")\n"
    " // -> [\"home\", \"gleam\", \"desktop\", \"\"]\n"
    " ```\n"
).
-spec split(binary(), binary()) -> list(binary()).
split(X, Substring) ->
    case Substring of
        <<""/utf8>> ->
            to_graphemes(X);

        _ ->
            _pipe = X,
            _pipe@1 = gleam_stdlib:identity(_pipe),
            _pipe@2 = gleam@string_tree:split(_pipe@1, Substring),
            gleam@list:map(_pipe@2, fun unicode:characters_to_binary/1)
    end.

-file("src/gleam/string.gleam", 651).
-spec to_utf_codepoints_loop(bitstring(), list(integer())) -> list(integer()).
to_utf_codepoints_loop(Bit_array, Acc) ->
    case Bit_array of
        <<First/utf8, Rest/binary>> ->
            to_utf_codepoints_loop(Rest, [First | Acc]);

        _ ->
            lists:reverse(Acc)
    end.

-file("src/gleam/string.gleam", 646).
-spec do_to_utf_codepoints(binary()) -> list(integer()).
do_to_utf_codepoints(String) ->
    to_utf_codepoints_loop(<<String/binary>>, []).

-file("src/gleam/string.gleam", 641).
?DOC(
    " Converts a `String` to a `List` of `UtfCodepoint`.\n"
    "\n"
    " See <https://en.wikipedia.org/wiki/Code_point> and\n"
    " <https://en.wikipedia.org/wiki/Unicode#Codespace_and_Code_Points> for an\n"
    " explanation on code points.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " \"a\" |> to_utf_codepoints\n"
    " // -> [UtfCodepoint(97)]\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " // Semantically the same as:\n"
    " // [\"🏳\", \"️\", \"‍\", \"🌈\"] or:\n"
    " // [waving_white_flag, variant_selector_16, zero_width_joiner, rainbow]\n"
    " \"🏳️‍🌈\" |> to_utf_codepoints\n"
    " // -> [\n"
    " //   UtfCodepoint(127987),\n"
    " //   UtfCodepoint(65039),\n"
    " //   UtfCodepoint(8205),\n"
    " //   UtfCodepoint(127752),\n"
    " // ]\n"
    " ```\n"
).
-spec to_utf_codepoints(binary()) -> list(integer()).
to_utf_codepoints(String) ->
    do_to_utf_codepoints(String).

-file("src/gleam/string.gleam", 691).
?DOC(
    " Converts a `List` of `UtfCodepoint`s to a `String`.\n"
    "\n"
    " See <https://en.wikipedia.org/wiki/Code_point> and\n"
    " <https://en.wikipedia.org/wiki/Unicode#Codespace_and_Code_Points> for an\n"
    " explanation on code points.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " let assert Ok(a) = utf_codepoint(97)\n"
    " let assert Ok(b) = utf_codepoint(98)\n"
    " let assert Ok(c) = utf_codepoint(99)\n"
    " from_utf_codepoints([a, b, c])\n"
    " // -> \"abc\"\n"
    " ```\n"
).
-spec from_utf_codepoints(list(integer())) -> binary().
from_utf_codepoints(Utf_codepoints) ->
    gleam_stdlib:utf_codepoint_list_to_string(Utf_codepoints).

-file("src/gleam/string.gleam", 697).
?DOC(
    " Converts an integer to a `UtfCodepoint`.\n"
    "\n"
    " Returns an `Error` if the integer does not represent a valid UTF codepoint.\n"
).
-spec utf_codepoint(integer()) -> {ok, integer()} | {error, nil}.
utf_codepoint(Value) ->
    case Value of
        I when I > 1114111 ->
            {error, nil};

        I@1 when (I@1 >= 55296) andalso (I@1 =< 57343) ->
            {error, nil};

        I@2 when I@2 < 0 ->
            {error, nil};

        I@3 ->
            {ok, gleam_stdlib:identity(I@3)}
    end.

-file("src/gleam/string.gleam", 718).
?DOC(
    " Converts an UtfCodepoint to its ordinal code point value.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " let assert [utf_codepoint, ..] = to_utf_codepoints(\"💜\")\n"
    " utf_codepoint_to_int(utf_codepoint)\n"
    " // -> 128156\n"
    " ```\n"
).
-spec utf_codepoint_to_int(integer()) -> integer().
utf_codepoint_to_int(Cp) ->
    gleam_stdlib:identity(Cp).

-file("src/gleam/string.gleam", 735).
?DOC(
    " Converts a `String` into `Option(String)` where an empty `String` becomes\n"
    " `None`.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " to_option(\"\")\n"
    " // -> None\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " to_option(\"hats\")\n"
    " // -> Some(\"hats\")\n"
    " ```\n"
).
-spec to_option(binary()) -> gleam@option:option(binary()).
to_option(String) ->
    case String of
        <<""/utf8>> ->
            none;

        _ ->
            {some, String}
    end.

-file("src/gleam/string.gleam", 758).
?DOC(
    " Returns the first grapheme cluster in a given `String` and wraps it in a\n"
    " `Result(String, Nil)`. If the `String` is empty, it returns `Error(Nil)`.\n"
    " Otherwise, it returns `Ok(String)`.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " first(\"\")\n"
    " // -> Error(Nil)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " first(\"icecream\")\n"
    " // -> Ok(\"i\")\n"
    " ```\n"
).
-spec first(binary()) -> {ok, binary()} | {error, nil}.
first(String) ->
    case gleam_stdlib:string_pop_grapheme(String) of
        {ok, {First, _}} ->
            {ok, First};

        {error, E} ->
            {error, E}
    end.

-file("src/gleam/string.gleam", 781).
?DOC(
    " Returns the last grapheme cluster in a given `String` and wraps it in a\n"
    " `Result(String, Nil)`. If the `String` is empty, it returns `Error(Nil)`.\n"
    " Otherwise, it returns `Ok(String)`.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " last(\"\")\n"
    " // -> Error(Nil)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " last(\"icecream\")\n"
    " // -> Ok(\"m\")\n"
    " ```\n"
).
-spec last(binary()) -> {ok, binary()} | {error, nil}.
last(String) ->
    case gleam_stdlib:string_pop_grapheme(String) of
        {ok, {First, <<""/utf8>>}} ->
            {ok, First};

        {ok, {_, Rest}} ->
            {ok, slice(Rest, -1, 1)};

        {error, E} ->
            {error, E}
    end.

-file("src/gleam/string.gleam", 799).
?DOC(
    " Creates a new `String` with the first grapheme in the input `String`\n"
    " converted to uppercase and the remaining graphemes to lowercase.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " capitalise(\"mamouna\")\n"
    " // -> \"Mamouna\"\n"
    " ```\n"
).
-spec capitalise(binary()) -> binary().
capitalise(String) ->
    case gleam_stdlib:string_pop_grapheme(String) of
        {ok, {First, Rest}} ->
            append(string:uppercase(First), string:lowercase(Rest));

        {error, _} ->
            <<""/utf8>>
    end.

-file("src/gleam/string.gleam", 808).
?DOC(" Returns a `String` representation of a term in Gleam syntax.\n").
-spec inspect(any()) -> binary().
inspect(Term) ->
    _pipe = gleam_stdlib:inspect(Term),
    unicode:characters_to_binary(_pipe).

-file("src/gleam/string.gleam", 831).
?DOC(
    " Returns the number of bytes in a `String`.\n"
    "\n"
    " This function runs in constant time on Erlang and in linear time on\n"
    " JavaScript.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " byte_size(\"🏳️‍⚧️🏳️‍🌈👩🏾‍❤️‍👨🏻\")\n"
    " // -> 58\n"
    " ```\n"
).
-spec byte_size(binary()) -> integer().
byte_size(String) ->
    erlang:byte_size(String).
