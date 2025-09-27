-module(gleam@string_tree).
-compile([no_auto_import, nowarn_unused_vars, nowarn_unused_function, nowarn_nomatch]).

-export([append_tree/2, prepend_tree/2, from_strings/1, new/0, concat/1, from_string/1, prepend/2, append/2, to_string/1, byte_size/1, join/2, lowercase/1, uppercase/1, reverse/1, split/2, replace/3, is_equal/2, is_empty/1]).
-export_type([string_tree/0, direction/0]).

-if(?OTP_RELEASE >= 27).
-define(MODULEDOC(Str), -moduledoc(Str)).
-define(DOC(Str), -doc(Str)).
-else.
-define(MODULEDOC(Str), -compile([])).
-define(DOC(Str), -compile([])).
-endif.

-type string_tree() :: any().

-type direction() :: all.

-file("src/gleam/string_tree.gleam", 61).
?DOC(
    " Appends some `StringTree` onto the end of another.\n"
    "\n"
    " Runs in constant time.\n"
).
-spec append_tree(string_tree(), string_tree()) -> string_tree().
append_tree(Tree, Suffix) ->
    gleam_stdlib:iodata_append(Tree, Suffix).

-file("src/gleam/string_tree.gleam", 48).
?DOC(
    " Prepends some `StringTree` onto the start of another.\n"
    "\n"
    " Runs in constant time.\n"
).
-spec prepend_tree(string_tree(), string_tree()) -> string_tree().
prepend_tree(Tree, Prefix) ->
    gleam_stdlib:iodata_append(Prefix, Tree).

-file("src/gleam/string_tree.gleam", 69).
?DOC(
    " Converts a list of strings into a `StringTree`.\n"
    "\n"
    " Runs in constant time.\n"
).
-spec from_strings(list(binary())) -> string_tree().
from_strings(Strings) ->
    gleam_stdlib:identity(Strings).

-file("src/gleam/string_tree.gleam", 24).
?DOC(
    " Create an empty `StringTree`. Useful as the start of a pipe chaining many\n"
    " trees together.\n"
).
-spec new() -> string_tree().
new() ->
    gleam_stdlib:identity([]).

-file("src/gleam/string_tree.gleam", 77).
?DOC(
    " Joins a list of trees into a single tree.\n"
    "\n"
    " Runs in constant time.\n"
).
-spec concat(list(string_tree())) -> string_tree().
concat(Trees) ->
    gleam_stdlib:identity(Trees).

-file("src/gleam/string_tree.gleam", 85).
?DOC(
    " Converts a string into a `StringTree`.\n"
    "\n"
    " Runs in constant time.\n"
).
-spec from_string(binary()) -> string_tree().
from_string(String) ->
    gleam_stdlib:identity(String).

-file("src/gleam/string_tree.gleam", 32).
?DOC(
    " Prepends a `String` onto the start of some `StringTree`.\n"
    "\n"
    " Runs in constant time.\n"
).
-spec prepend(string_tree(), binary()) -> string_tree().
prepend(Tree, Prefix) ->
    gleam_stdlib:iodata_append(gleam_stdlib:identity(Prefix), Tree).

-file("src/gleam/string_tree.gleam", 40).
?DOC(
    " Appends a `String` onto the end of some `StringTree`.\n"
    "\n"
    " Runs in constant time.\n"
).
-spec append(string_tree(), binary()) -> string_tree().
append(Tree, Second) ->
    gleam_stdlib:iodata_append(Tree, gleam_stdlib:identity(Second)).

-file("src/gleam/string_tree.gleam", 94).
?DOC(
    " Turns a `StringTree` into a `String`\n"
    "\n"
    " This function is implemented natively by the virtual machine and is highly\n"
    " optimised.\n"
).
-spec to_string(string_tree()) -> binary().
to_string(Tree) ->
    unicode:characters_to_binary(Tree).

-file("src/gleam/string_tree.gleam", 100).
?DOC(" Returns the size of the `StringTree` in bytes.\n").
-spec byte_size(string_tree()) -> integer().
byte_size(Tree) ->
    erlang:iolist_size(Tree).

-file("src/gleam/string_tree.gleam", 104).
?DOC(" Joins the given trees into a new tree separated with the given string.\n").
-spec join(list(string_tree()), binary()) -> string_tree().
join(Trees, Sep) ->
    _pipe = Trees,
    _pipe@1 = gleam@list:intersperse(_pipe, gleam_stdlib:identity(Sep)),
    gleam_stdlib:identity(_pipe@1).

-file("src/gleam/string_tree.gleam", 115).
?DOC(
    " Converts a `StringTree` to a new one where the contents have been\n"
    " lowercased.\n"
).
-spec lowercase(string_tree()) -> string_tree().
lowercase(Tree) ->
    string:lowercase(Tree).

-file("src/gleam/string_tree.gleam", 122).
?DOC(
    " Converts a `StringTree` to a new one where the contents have been\n"
    " uppercased.\n"
).
-spec uppercase(string_tree()) -> string_tree().
uppercase(Tree) ->
    string:uppercase(Tree).

-file("src/gleam/string_tree.gleam", 127).
?DOC(" Converts a `StringTree` to a new one with the contents reversed.\n").
-spec reverse(string_tree()) -> string_tree().
reverse(Tree) ->
    string:reverse(Tree).

-file("src/gleam/string_tree.gleam", 145).
?DOC(" Splits a `StringTree` on a given pattern into a list of trees.\n").
-spec split(string_tree(), binary()) -> list(string_tree()).
split(Tree, Pattern) ->
    string:split(Tree, Pattern, all).

-file("src/gleam/string_tree.gleam", 156).
?DOC(" Replaces all instances of a pattern with a given string substitute.\n").
-spec replace(string_tree(), binary(), binary()) -> string_tree().
replace(Tree, Pattern, Substitute) ->
    gleam_stdlib:string_replace(Tree, Pattern, Substitute).

-file("src/gleam/string_tree.gleam", 182).
?DOC(
    " Compares two string trees to determine if they have the same textual\n"
    " content.\n"
    "\n"
    " Comparing two string trees using the `==` operator may return `False` even\n"
    " if they have the same content as they may have been build in different ways,\n"
    " so using this function is often preferred.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " from_strings([\"a\", \"b\"]) == from_string(\"ab\")\n"
    " // -> False\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " is_equal(from_strings([\"a\", \"b\"]), from_string(\"ab\"))\n"
    " // -> True\n"
    " ```\n"
).
-spec is_equal(string_tree(), string_tree()) -> boolean().
is_equal(A, B) ->
    string:equal(A, B).

-file("src/gleam/string_tree.gleam", 206).
?DOC(
    " Inspects a `StringTree` to determine if it is equivalent to an empty string.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " from_string(\"ok\") |> is_empty\n"
    " // -> False\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " from_string(\"\") |> is_empty\n"
    " // -> True\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " from_strings([]) |> is_empty\n"
    " // -> True\n"
    " ```\n"
).
-spec is_empty(string_tree()) -> boolean().
is_empty(Tree) ->
    string:is_empty(Tree).
