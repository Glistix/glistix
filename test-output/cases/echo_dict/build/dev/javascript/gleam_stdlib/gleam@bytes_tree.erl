-module(gleam@bytes_tree).
-compile([no_auto_import, nowarn_unused_vars, nowarn_unused_function, nowarn_nomatch]).

-export([append_tree/2, prepend_tree/2, concat/1, new/0, from_string/1, prepend_string/2, append_string/2, from_string_tree/1, from_bit_array/1, prepend/2, append/2, concat_bit_arrays/1, to_bit_array/1, byte_size/1]).
-export_type([bytes_tree/0]).

-if(?OTP_RELEASE >= 27).
-define(MODULEDOC(Str), -moduledoc(Str)).
-define(DOC(Str), -doc(Str)).
-else.
-define(MODULEDOC(Str), -compile([])).
-define(DOC(Str), -compile([])).
-endif.

?MODULEDOC(
    " `BytesTree` is a type used for efficiently building binary content to be\n"
    " written to a file or a socket. Internally it is represented as tree so to\n"
    " append or prepend to a bytes tree is a constant time operation that\n"
    " allocates a new node in the tree without copying any of the content. When\n"
    " writing to an output stream the tree is traversed and the content is sent\n"
    " directly rather than copying it into a single buffer beforehand.\n"
    "\n"
    " If we append one bit array to another the bit arrays must be copied to a\n"
    " new location in memory so that they can sit together. This behaviour\n"
    " enables efficient reading of the data but copying can be expensive,\n"
    " especially if we want to join many bit arrays together.\n"
    "\n"
    " BytesTree is different in that it can be joined together in constant\n"
    " time using minimal memory, and then can be efficiently converted to a\n"
    " bit array using the `to_bit_array` function.\n"
    "\n"
    " Byte trees are always byte aligned, so that a number of bits that is not\n"
    " divisible by 8 will be padded with 0s.\n"
    "\n"
    " On Erlang this type is compatible with Erlang's iolists.\n"
).

-opaque bytes_tree() :: {bytes, bitstring()} |
    {text, gleam@string_tree:string_tree()} |
    {many, list(bytes_tree())}.

-file("src/gleam/bytes_tree.gleam", 68).
?DOC(
    " Appends a bytes tree onto the end of another.\n"
    "\n"
    " Runs in constant time.\n"
).
-spec append_tree(bytes_tree(), bytes_tree()) -> bytes_tree().
append_tree(First, Second) ->
    gleam_stdlib:iodata_append(First, Second).

-file("src/gleam/bytes_tree.gleam", 59).
?DOC(
    " Prepends a bytes tree onto the start of another.\n"
    "\n"
    " Runs in constant time.\n"
).
-spec prepend_tree(bytes_tree(), bytes_tree()) -> bytes_tree().
prepend_tree(Second, First) ->
    gleam_stdlib:iodata_append(First, Second).

-file("src/gleam/bytes_tree.gleam", 98).
?DOC(
    " Joins a list of bytes trees into a single one.\n"
    "\n"
    " Runs in constant time.\n"
).
-spec concat(list(bytes_tree())) -> bytes_tree().
concat(Trees) ->
    gleam_stdlib:identity(Trees).

-file("src/gleam/bytes_tree.gleam", 35).
?DOC(
    " Create an empty `BytesTree`. Useful as the start of a pipe chaining many\n"
    " trees together.\n"
).
-spec new() -> bytes_tree().
new() ->
    gleam_stdlib:identity([]).

-file("src/gleam/bytes_tree.gleam", 118).
?DOC(
    " Creates a new bytes tree from a string.\n"
    "\n"
    " Runs in constant time when running on Erlang.\n"
    " Runs in linear time otherwise.\n"
).
-spec from_string(binary()) -> bytes_tree().
from_string(String) ->
    gleam_stdlib:wrap_list(String).

-file("src/gleam/bytes_tree.gleam", 80).
?DOC(
    " Prepends a string onto the start of a bytes tree.\n"
    "\n"
    " Runs in constant time when running on Erlang.\n"
    " Runs in linear time with the length of the string otherwise.\n"
).
-spec prepend_string(bytes_tree(), binary()) -> bytes_tree().
prepend_string(Second, First) ->
    gleam_stdlib:iodata_append(gleam_stdlib:wrap_list(First), Second).

-file("src/gleam/bytes_tree.gleam", 89).
?DOC(
    " Appends a string onto the end of a bytes tree.\n"
    "\n"
    " Runs in constant time when running on Erlang.\n"
    " Runs in linear time with the length of the string otherwise.\n"
).
-spec append_string(bytes_tree(), binary()) -> bytes_tree().
append_string(First, Second) ->
    gleam_stdlib:iodata_append(First, gleam_stdlib:wrap_list(Second)).

-file("src/gleam/bytes_tree.gleam", 128).
?DOC(
    " Creates a new bytes tree from a string tree.\n"
    "\n"
    " Runs in constant time when running on Erlang.\n"
    " Runs in linear time otherwise.\n"
).
-spec from_string_tree(gleam@string_tree:string_tree()) -> bytes_tree().
from_string_tree(Tree) ->
    gleam_stdlib:wrap_list(Tree).

-file("src/gleam/bytes_tree.gleam", 136).
?DOC(
    " Creates a new bytes tree from a bit array.\n"
    "\n"
    " Runs in constant time.\n"
).
-spec from_bit_array(bitstring()) -> bytes_tree().
from_bit_array(Bits) ->
    _pipe = Bits,
    _pipe@1 = gleam_stdlib:bit_array_pad_to_bytes(_pipe),
    gleam_stdlib:wrap_list(_pipe@1).

-file("src/gleam/bytes_tree.gleam", 43).
?DOC(
    " Prepends a bit array to the start of a bytes tree.\n"
    "\n"
    " Runs in constant time.\n"
).
-spec prepend(bytes_tree(), bitstring()) -> bytes_tree().
prepend(Second, First) ->
    gleam_stdlib:iodata_append(from_bit_array(First), Second).

-file("src/gleam/bytes_tree.gleam", 51).
?DOC(
    " Appends a bit array to the end of a bytes tree.\n"
    "\n"
    " Runs in constant time.\n"
).
-spec append(bytes_tree(), bitstring()) -> bytes_tree().
append(First, Second) ->
    gleam_stdlib:iodata_append(First, from_bit_array(Second)).

-file("src/gleam/bytes_tree.gleam", 106).
?DOC(
    " Joins a list of bit arrays into a single bytes tree.\n"
    "\n"
    " Runs in constant time.\n"
).
-spec concat_bit_arrays(list(bitstring())) -> bytes_tree().
concat_bit_arrays(Bits) ->
    _pipe = Bits,
    _pipe@1 = gleam@list:map(_pipe, fun(B) -> from_bit_array(B) end),
    gleam_stdlib:identity(_pipe@1).

-file("src/gleam/bytes_tree.gleam", 162).
-spec to_list(list(list(bytes_tree())), list(bitstring())) -> list(bitstring()).
to_list(Stack, Acc) ->
    case Stack of
        [] ->
            Acc;

        [[] | Remaining_stack] ->
            to_list(Remaining_stack, Acc);

        [[{bytes, Bits} | Rest] | Remaining_stack@1] ->
            to_list([Rest | Remaining_stack@1], [Bits | Acc]);

        [[{text, Tree} | Rest@1] | Remaining_stack@2] ->
            Bits@1 = gleam_stdlib:identity(unicode:characters_to_binary(Tree)),
            to_list([Rest@1 | Remaining_stack@2], [Bits@1 | Acc]);

        [[{many, Trees} | Rest@2] | Remaining_stack@3] ->
            to_list([Trees, Rest@2 | Remaining_stack@3], Acc)
    end.

-file("src/gleam/bytes_tree.gleam", 155).
?DOC(
    " Turns a bytes tree into a bit array.\n"
    "\n"
    " Runs in linear time.\n"
    "\n"
    " When running on Erlang this function is implemented natively by the\n"
    " virtual machine and is highly optimised.\n"
).
-spec to_bit_array(bytes_tree()) -> bitstring().
to_bit_array(Tree) ->
    erlang:list_to_bitstring(Tree).

-file("src/gleam/bytes_tree.gleam", 186).
?DOC(
    " Returns the size of the bytes tree's content in bytes.\n"
    "\n"
    " Runs in linear time.\n"
).
-spec byte_size(bytes_tree()) -> integer().
byte_size(Tree) ->
    erlang:iolist_size(Tree).
