-module(gleam@dict).
-compile([no_auto_import, nowarn_unused_vars, nowarn_unused_function, nowarn_nomatch]).

-export([size/1, is_empty/1, to_list/1, new/0, get/2, has_key/2, insert/3, from_list/1, keys/1, values/1, take/2, merge/2, delete/2, drop/2, upsert/3, fold/3, map_values/2, filter/2, each/2, combine/3]).
-export_type([dict/2]).

-if(?OTP_RELEASE >= 27).
-define(MODULEDOC(Str), -moduledoc(Str)).
-define(DOC(Str), -doc(Str)).
-else.
-define(MODULEDOC(Str), -compile([])).
-define(DOC(Str), -compile([])).
-endif.

-type dict(KU, KV) :: any() | {gleam_phantom, KU, KV}.

-file("src/gleam/dict.gleam", 36).
?DOC(
    " Determines the number of key-value pairs in the dict.\n"
    " This function runs in constant time and does not need to iterate the dict.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " new() |> size\n"
    " // -> 0\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " new() |> insert(\"key\", \"value\") |> size\n"
    " // -> 1\n"
    " ```\n"
).
-spec size(dict(any(), any())) -> integer().
size(Dict) ->
    maps:size(Dict).

-file("src/gleam/dict.gleam", 52).
?DOC(
    " Determines whether or not the dict is empty.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " new() |> is_empty\n"
    " // -> True\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " new() |> insert(\"b\", 1) |> is_empty\n"
    " // -> False\n"
    " ```\n"
).
-spec is_empty(dict(any(), any())) -> boolean().
is_empty(Dict) ->
    maps:size(Dict) =:= 0.

-file("src/gleam/dict.gleam", 80).
?DOC(
    " Converts the dict to a list of 2-element tuples `#(key, value)`, one for\n"
    " each key-value pair in the dict.\n"
    "\n"
    " The tuples in the list have no specific order.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " Calling `to_list` on an empty `dict` returns an empty list.\n"
    "\n"
    " ```gleam\n"
    " new() |> to_list\n"
    " // -> []\n"
    " ```\n"
    "\n"
    " The ordering of elements in the resulting list is an implementation detail\n"
    " that should not be relied upon.\n"
    "\n"
    " ```gleam\n"
    " new() |> insert(\"b\", 1) |> insert(\"a\", 0) |> insert(\"c\", 2) |> to_list\n"
    " // -> [#(\"a\", 0), #(\"b\", 1), #(\"c\", 2)]\n"
    " ```\n"
).
-spec to_list(dict(LE, LF)) -> list({LE, LF}).
to_list(Dict) ->
    maps:to_list(Dict).

-file("src/gleam/dict.gleam", 129).
?DOC(" Creates a fresh dict that contains no values.\n").
-spec new() -> dict(any(), any()).
new() ->
    maps:new().

-file("src/gleam/dict.gleam", 150).
?DOC(
    " Fetches a value from a dict for a given key.\n"
    "\n"
    " The dict may not have a value for the key, so the value is wrapped in a\n"
    " `Result`.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " new() |> insert(\"a\", 0) |> get(\"a\")\n"
    " // -> Ok(0)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " new() |> insert(\"a\", 0) |> get(\"b\")\n"
    " // -> Error(Nil)\n"
    " ```\n"
).
-spec get(dict(MH, MI), MH) -> {ok, MI} | {error, nil}.
get(From, Get) ->
    gleam_stdlib:map_get(From, Get).

-file("src/gleam/dict.gleam", 116).
?DOC(
    " Determines whether or not a value present in the dict for a given key.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " new() |> insert(\"a\", 0) |> has_key(\"a\")\n"
    " // -> True\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " new() |> insert(\"a\", 0) |> has_key(\"b\")\n"
    " // -> False\n"
    " ```\n"
).
-spec has_key(dict(LV, any()), LV) -> boolean().
has_key(Dict, Key) ->
    maps:is_key(Key, Dict).

-file("src/gleam/dict.gleam", 169).
?DOC(
    " Inserts a value into the dict with the given key.\n"
    "\n"
    " If the dict already has a value for the given key then the value is\n"
    " replaced with the new value.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " new() |> insert(\"a\", 0)\n"
    " // -> from_list([#(\"a\", 0)])\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " new() |> insert(\"a\", 0) |> insert(\"a\", 5)\n"
    " // -> from_list([#(\"a\", 5)])\n"
    " ```\n"
).
-spec insert(dict(MN, MO), MN, MO) -> dict(MN, MO).
insert(Dict, Key, Value) ->
    maps:put(Key, Value, Dict).

-file("src/gleam/dict.gleam", 92).
-spec from_list_loop(list({LO, LP}), dict(LO, LP)) -> dict(LO, LP).
from_list_loop(List, Initial) ->
    case List of
        [] ->
            Initial;

        [{Key, Value} | Rest] ->
            from_list_loop(Rest, insert(Initial, Key, Value))
    end.

-file("src/gleam/dict.gleam", 88).
?DOC(
    " Converts a list of 2-element tuples `#(key, value)` to a dict.\n"
    "\n"
    " If two tuples have the same key the last one in the list will be the one\n"
    " that is present in the dict.\n"
).
-spec from_list(list({LJ, LK})) -> dict(LJ, LK).
from_list(List) ->
    maps:from_list(List).

-file("src/gleam/dict.gleam", 223).
-spec reverse_and_concat(list(NX), list(NX)) -> list(NX).
reverse_and_concat(Remaining, Accumulator) ->
    case Remaining of
        [] ->
            Accumulator;

        [First | Rest] ->
            reverse_and_concat(Rest, [First | Accumulator])
    end.

-file("src/gleam/dict.gleam", 216).
-spec do_keys_loop(list({NS, any()}), list(NS)) -> list(NS).
do_keys_loop(List, Acc) ->
    case List of
        [] ->
            reverse_and_concat(Acc, []);

        [{Key, _} | Rest] ->
            do_keys_loop(Rest, [Key | Acc])
    end.

-file("src/gleam/dict.gleam", 212).
?DOC(
    " Gets a list of all keys in a given dict.\n"
    "\n"
    " Dicts are not ordered so the keys are not returned in any specific order. Do\n"
    " not write code that relies on the order keys are returned by this function\n"
    " as it may change in later versions of Gleam or Erlang.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " from_list([#(\"a\", 0), #(\"b\", 1)]) |> keys\n"
    " // -> [\"a\", \"b\"]\n"
    " ```\n"
).
-spec keys(dict(NN, any())) -> list(NN).
keys(Dict) ->
    maps:keys(Dict).

-file("src/gleam/dict.gleam", 249).
-spec do_values_loop(list({any(), OH}), list(OH)) -> list(OH).
do_values_loop(List, Acc) ->
    case List of
        [] ->
            reverse_and_concat(Acc, []);

        [{_, Value} | Rest] ->
            do_values_loop(Rest, [Value | Acc])
    end.

-file("src/gleam/dict.gleam", 244).
?DOC(
    " Gets a list of all values in a given dict.\n"
    "\n"
    " Dicts are not ordered so the values are not returned in any specific order. Do\n"
    " not write code that relies on the order values are returned by this function\n"
    " as it may change in later versions of Gleam or Erlang.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " from_list([#(\"a\", 0), #(\"b\", 1)]) |> values\n"
    " // -> [0, 1]\n"
    " ```\n"
).
-spec values(dict(any(), OC)) -> list(OC).
values(Dict) ->
    maps:values(Dict).

-file("src/gleam/dict.gleam", 318).
-spec do_take_loop(dict(PL, PM), list(PL), dict(PL, PM)) -> dict(PL, PM).
do_take_loop(Dict, Desired_keys, Acc) ->
    Insert = fun(Taken, Key) -> case gleam_stdlib:map_get(Dict, Key) of
            {ok, Value} ->
                insert(Taken, Key, Value);

            {error, _} ->
                Taken
        end end,
    case Desired_keys of
        [] ->
            Acc;

        [First | Rest] ->
            do_take_loop(Dict, Rest, Insert(Acc, First))
    end.

-file("src/gleam/dict.gleam", 309).
?DOC(
    " Creates a new dict from a given dict, only including any entries for which the\n"
    " keys are in a given list.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " from_list([#(\"a\", 0), #(\"b\", 1)])\n"
    " |> take([\"b\"])\n"
    " // -> from_list([#(\"b\", 1)])\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " from_list([#(\"a\", 0), #(\"b\", 1)])\n"
    " |> take([\"a\", \"b\", \"c\"])\n"
    " // -> from_list([#(\"a\", 0), #(\"b\", 1)])\n"
    " ```\n"
).
-spec take(dict(OX, OY), list(OX)) -> dict(OX, OY).
take(Dict, Desired_keys) ->
    maps:with(Desired_keys, Dict).

-file("src/gleam/dict.gleam", 363).
-spec insert_pair(dict(QJ, QK), {QJ, QK}) -> dict(QJ, QK).
insert_pair(Dict, Pair) ->
    insert(Dict, erlang:element(1, Pair), erlang:element(2, Pair)).

-file("src/gleam/dict.gleam", 356).
-spec fold_inserts(list({QC, QD}), dict(QC, QD)) -> dict(QC, QD).
fold_inserts(New_entries, Dict) ->
    case New_entries of
        [] ->
            Dict;

        [First | Rest] ->
            fold_inserts(Rest, insert_pair(Dict, First))
    end.

-file("src/gleam/dict.gleam", 350).
?DOC(
    " Creates a new dict from a pair of given dicts by combining their entries.\n"
    "\n"
    " If there are entries with the same keys in both dicts the entry from the\n"
    " second dict takes precedence.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " let a = from_list([#(\"a\", 0), #(\"b\", 1)])\n"
    " let b = from_list([#(\"b\", 2), #(\"c\", 3)])\n"
    " merge(a, b)\n"
    " // -> from_list([#(\"a\", 0), #(\"b\", 2), #(\"c\", 3)])\n"
    " ```\n"
).
-spec merge(dict(PU, PV), dict(PU, PV)) -> dict(PU, PV).
merge(Dict, New_entries) ->
    maps:merge(Dict, New_entries).

-file("src/gleam/dict.gleam", 382).
?DOC(
    " Creates a new dict from a given dict with all the same entries except for the\n"
    " one with a given key, if it exists.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " from_list([#(\"a\", 0), #(\"b\", 1)]) |> delete(\"a\")\n"
    " // -> from_list([#(\"b\", 1)])\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " from_list([#(\"a\", 0), #(\"b\", 1)]) |> delete(\"c\")\n"
    " // -> from_list([#(\"a\", 0), #(\"b\", 1)])\n"
    " ```\n"
).
-spec delete(dict(QP, QQ), QP) -> dict(QP, QQ).
delete(Dict, Key) ->
    maps:remove(Key, Dict).

-file("src/gleam/dict.gleam", 410).
?DOC(
    " Creates a new dict from a given dict with all the same entries except any with\n"
    " keys found in a given list.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " from_list([#(\"a\", 0), #(\"b\", 1)]) |> drop([\"a\"])\n"
    " // -> from_list([#(\"b\", 1)])\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " from_list([#(\"a\", 0), #(\"b\", 1)]) |> drop([\"c\"])\n"
    " // -> from_list([#(\"a\", 0), #(\"b\", 1)])\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " from_list([#(\"a\", 0), #(\"b\", 1)]) |> drop([\"a\", \"b\", \"c\"])\n"
    " // -> from_list([])\n"
    " ```\n"
).
-spec drop(dict(RB, RC), list(RB)) -> dict(RB, RC).
drop(Dict, Disallowed_keys) ->
    case Disallowed_keys of
        [] ->
            Dict;

        [First | Rest] ->
            drop(delete(Dict, First), Rest)
    end.

-file("src/gleam/dict.gleam", 440).
?DOC(
    " Creates a new dict with one entry inserted or updated using a given function.\n"
    "\n"
    " If there was not an entry in the dict for the given key then the function\n"
    " gets `None` as its argument, otherwise it gets `Some(value)`.\n"
    "\n"
    " ## Example\n"
    "\n"
    " ```gleam\n"
    " let dict = from_list([#(\"a\", 0)])\n"
    " let increment = fn(x) {\n"
    "   case x {\n"
    "     Some(i) -> i + 1\n"
    "     None -> 0\n"
    "   }\n"
    " }\n"
    "\n"
    " upsert(dict, \"a\", increment)\n"
    " // -> from_list([#(\"a\", 1)])\n"
    "\n"
    " upsert(dict, \"b\", increment)\n"
    " // -> from_list([#(\"a\", 0), #(\"b\", 0)])\n"
    " ```\n"
).
-spec upsert(dict(RI, RJ), RI, fun((gleam@option:option(RJ)) -> RJ)) -> dict(RI, RJ).
upsert(Dict, Key, Fun) ->
    case gleam_stdlib:map_get(Dict, Key) of
        {ok, Value} ->
            insert(Dict, Key, Fun({some, Value}));

        {error, _} ->
            insert(Dict, Key, Fun(none))
    end.

-file("src/gleam/dict.gleam", 484).
-spec fold_loop(list({RU, RV}), RX, fun((RX, RU, RV) -> RX)) -> RX.
fold_loop(List, Initial, Fun) ->
    case List of
        [] ->
            Initial;

        [{K, V} | Rest] ->
            fold_loop(Rest, Fun(Initial, K, V), Fun)
    end.

-file("src/gleam/dict.gleam", 476).
?DOC(
    " Combines all entries into a single value by calling a given function on each\n"
    " one.\n"
    "\n"
    " Dicts are not ordered so the values are not returned in any specific order. Do\n"
    " not write code that relies on the order entries are used by this function\n"
    " as it may change in later versions of Gleam or Erlang.\n"
    "\n"
    " # Examples\n"
    "\n"
    " ```gleam\n"
    " let dict = from_list([#(\"a\", 1), #(\"b\", 3), #(\"c\", 9)])\n"
    " fold(dict, 0, fn(accumulator, key, value) { accumulator + value })\n"
    " // -> 13\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " import gleam/string\n"
    "\n"
    " let dict = from_list([#(\"a\", 1), #(\"b\", 3), #(\"c\", 9)])\n"
    " fold(dict, \"\", fn(accumulator, key, value) {\n"
    "   string.append(accumulator, key)\n"
    " })\n"
    " // -> \"abc\"\n"
    " ```\n"
).
-spec fold(dict(RP, RQ), RT, fun((RT, RP, RQ) -> RT)) -> RT.
fold(Dict, Initial, Fun) ->
    fold_loop(maps:to_list(Dict), Initial, Fun).

-file("src/gleam/dict.gleam", 188).
?DOC(
    " Updates all values in a given dict by calling a given function on each key\n"
    " and value.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " from_list([#(3, 3), #(2, 4)])\n"
    " |> map_values(fn(key, value) { key * value })\n"
    " // -> from_list([#(3, 9), #(2, 8)])\n"
    " ```\n"
).
-spec map_values(dict(MZ, NA), fun((MZ, NA) -> ND)) -> dict(MZ, ND).
map_values(Dict, Fun) ->
    maps:map(Fun, Dict).

-file("src/gleam/dict.gleam", 273).
?DOC(
    " Creates a new dict from a given dict, minus any entries that a given function\n"
    " returns `False` for.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " from_list([#(\"a\", 0), #(\"b\", 1)])\n"
    " |> filter(fn(key, value) { value != 0 })\n"
    " // -> from_list([#(\"b\", 1)])\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " from_list([#(\"a\", 0), #(\"b\", 1)])\n"
    " |> filter(fn(key, value) { True })\n"
    " // -> from_list([#(\"a\", 0), #(\"b\", 1)])\n"
    " ```\n"
).
-spec filter(dict(OL, OM), fun((OL, OM) -> boolean())) -> dict(OL, OM).
filter(Dict, Predicate) ->
    maps:filter(Predicate, Dict).

-file("src/gleam/dict.gleam", 517).
?DOC(
    " Calls a function for each key and value in a dict, discarding the return\n"
    " value.\n"
    "\n"
    " Useful for producing a side effect for every item of a dict.\n"
    "\n"
    " ```gleam\n"
    " import gleam/io\n"
    "\n"
    " let dict = from_list([#(\"a\", \"apple\"), #(\"b\", \"banana\"), #(\"c\", \"cherry\")])\n"
    "\n"
    " each(dict, fn(k, v) {\n"
    "   io.println(key <> \" => \" <> value)\n"
    " })\n"
    " // -> Nil\n"
    " // a => apple\n"
    " // b => banana\n"
    " // c => cherry\n"
    " ```\n"
    "\n"
    " The order of elements in the iteration is an implementation detail that\n"
    " should not be relied upon.\n"
).
-spec each(dict(RY, RZ), fun((RY, RZ) -> any())) -> nil.
each(Dict, Fun) ->
    fold(
        Dict,
        nil,
        fun(Nil, K, V) ->
            Fun(K, V),
            Nil
        end
    ).

-file("src/gleam/dict.gleam", 538).
?DOC(
    " Creates a new dict from a pair of given dicts by combining their entries.\n"
    "\n"
    " If there are entries with the same keys in both dicts the given function is\n"
    " used to determine the new value to use in the resulting dict.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " let a = from_list([#(\"a\", 0), #(\"b\", 1)])\n"
    " let b = from_list([#(\"a\", 2), #(\"c\", 3)])\n"
    " combine(a, b, fn(one, other) { one + other })\n"
    " // -> from_list([#(\"a\", 2), #(\"b\", 1), #(\"c\", 3)])\n"
    " ```\n"
).
-spec combine(dict(SD, SE), dict(SD, SE), fun((SE, SE) -> SE)) -> dict(SD, SE).
combine(Dict, Other, Fun) ->
    fold(
        Dict,
        Other,
        fun(Acc, Key, Value) -> case gleam_stdlib:map_get(Acc, Key) of
                {ok, Other_value} ->
                    insert(Acc, Key, Fun(Value, Other_value));

                {error, _} ->
                    insert(Acc, Key, Value)
            end end
    ).
