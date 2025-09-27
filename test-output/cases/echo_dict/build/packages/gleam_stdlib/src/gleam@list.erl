-module(gleam@list).
-compile([no_auto_import, nowarn_unused_vars, nowarn_unused_function, nowarn_nomatch]).

-export([length/1, reverse/1, is_empty/1, contains/2, first/1, rest/1, filter/2, filter_map/2, map/2, map2/3, index_map/2, try_map/2, drop/2, take/2, new/0, wrap/1, append/2, prepend/2, flatten/1, flat_map/2, fold/3, count/2, group/2, map_fold/3, fold_right/3, index_fold/3, try_fold/3, fold_until/3, find/2, find_map/2, all/2, any/2, zip/2, strict_zip/2, unzip/1, intersperse/2, unique/1, sort/2, range/2, repeat/2, split/2, split_while/2, key_find/2, key_filter/2, pop/2, pop_map/2, key_pop/2, key_set/3, each/2, try_each/2, partition/2, permutations/1, window/2, window_by_2/1, drop_while/2, take_while/2, chunk/2, sized_chunk/2, reduce/2, scan/3, last/1, combinations/2, combination_pairs/1, transpose/1, interleave/1, shuffle/1, max/2, sample/2]).
-export_type([continue_or_stop/1, sorting/0]).

-if(?OTP_RELEASE >= 27).
-define(MODULEDOC(Str), -moduledoc(Str)).
-define(DOC(Str), -doc(Str)).
-else.
-define(MODULEDOC(Str), -compile([])).
-define(DOC(Str), -compile([])).
-endif.

?MODULEDOC(
    " Lists are an ordered sequence of elements and are one of the most common\n"
    " data types in Gleam.\n"
    "\n"
    " New elements can be added and removed from the front of a list in\n"
    " constant time, while adding and removing from the end requires traversing\n"
    " and copying the whole list, so keep this in mind when designing your\n"
    " programs.\n"
    "\n"
    " There is a dedicated syntax for prefixing to a list:\n"
    "\n"
    " ```gleam\n"
    " let new_list = [1, 2, ..existing_list]\n"
    " ```\n"
    "\n"
    " And a matching syntax for getting the first elements of a list:\n"
    "\n"
    " ```gleam\n"
    " case list {\n"
    "   [first_element, ..rest] -> first_element\n"
    "   _ -> \"this pattern matches when the list is empty\"\n"
    " }\n"
    " ```\n"
    "\n"
).

-type continue_or_stop(YP) :: {continue, YP} | {stop, YP}.

-type sorting() :: ascending | descending.

-file("src/gleam/list.gleam", 61).
-spec length_loop(list(any()), integer()) -> integer().
length_loop(List, Count) ->
    case List of
        [_ | List@1] ->
            length_loop(List@1, Count + 1);

        [] ->
            Count
    end.

-file("src/gleam/list.gleam", 57).
?DOC(
    " Counts the number of elements in a given list.\n"
    "\n"
    " This function has to traverse the list to determine the number of elements,\n"
    " so it runs in linear time.\n"
    "\n"
    " This function is natively implemented by the virtual machine and is highly\n"
    " optimised.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " length([])\n"
    " // -> 0\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " length([1])\n"
    " // -> 1\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " length([1, 2])\n"
    " // -> 2\n"
    " ```\n"
).
-spec length(list(any())) -> integer().
length(List) ->
    erlang:length(List).

-file("src/gleam/list.gleam", 131).
-spec reverse_and_prepend(list(YZ), list(YZ)) -> list(YZ).
reverse_and_prepend(Prefix, Suffix) ->
    case Prefix of
        [] ->
            Suffix;

        [First | Rest] ->
            reverse_and_prepend(Rest, [First | Suffix])
    end.

-file("src/gleam/list.gleam", 126).
?DOC(
    " Creates a new list from a given list containing the same elements but in the\n"
    " opposite order.\n"
    "\n"
    " This function has to traverse the list to create the new reversed list, so\n"
    " it runs in linear time.\n"
    "\n"
    " This function is natively implemented by the virtual machine and is highly\n"
    " optimised.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " reverse([])\n"
    " // -> []\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " reverse([1])\n"
    " // -> [1]\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " reverse([1, 2])\n"
    " // -> [2, 1]\n"
    " ```\n"
).
-spec reverse(list(YW)) -> list(YW).
reverse(List) ->
    lists:reverse(List).

-file("src/gleam/list.gleam", 159).
?DOC(
    " Determines whether or not the list is empty.\n"
    "\n"
    " This function runs in constant time.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " is_empty([])\n"
    " // -> True\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " is_empty([1])\n"
    " // -> False\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " is_empty([1, 1])\n"
    " // -> False\n"
    " ```\n"
).
-spec is_empty(list(any())) -> boolean().
is_empty(List) ->
    List =:= [].

-file("src/gleam/list.gleam", 195).
?DOC(
    " Determines whether or not a given element exists within a given list.\n"
    "\n"
    " This function traverses the list to find the element, so it runs in linear\n"
    " time.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " [] |> contains(any: 0)\n"
    " // -> False\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " [0] |> contains(any: 0)\n"
    " // -> True\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " [1] |> contains(any: 0)\n"
    " // -> False\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " [1, 1] |> contains(any: 0)\n"
    " // -> False\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " [1, 0] |> contains(any: 0)\n"
    " // -> True\n"
    " ```\n"
).
-spec contains(list(AAF), AAF) -> boolean().
contains(List, Elem) ->
    case List of
        [] ->
            false;

        [First | _] when First =:= Elem ->
            true;

        [_ | Rest] ->
            contains(Rest, Elem)
    end.

-file("src/gleam/list.gleam", 222).
?DOC(
    " Gets the first element from the start of the list, if there is one.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " first([])\n"
    " // -> Error(Nil)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " first([0])\n"
    " // -> Ok(0)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " first([1, 2])\n"
    " // -> Ok(1)\n"
    " ```\n"
).
-spec first(list(AAH)) -> {ok, AAH} | {error, nil}.
first(List) ->
    case List of
        [] ->
            {error, nil};

        [First | _] ->
            {ok, First}
    end.

-file("src/gleam/list.gleam", 251).
?DOC(
    " Returns the list minus the first element. If the list is empty, `Error(Nil)` is\n"
    " returned.\n"
    "\n"
    " This function runs in constant time and does not make a copy of the list.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " rest([])\n"
    " // -> Error(Nil)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " rest([0])\n"
    " // -> Ok([])\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " rest([1, 2])\n"
    " // -> Ok([2])\n"
    " ```\n"
).
-spec rest(list(AAL)) -> {ok, list(AAL)} | {error, nil}.
rest(List) ->
    case List of
        [] ->
            {error, nil};

        [_ | Rest] ->
            {ok, Rest}
    end.

-file("src/gleam/list.gleam", 293).
-spec update_group(fun((AAW) -> AAX)) -> fun((gleam@dict:dict(AAX, list(AAW)), AAW) -> gleam@dict:dict(AAX, list(AAW))).
update_group(F) ->
    fun(Groups, Elem) -> case gleam_stdlib:map_get(Groups, F(Elem)) of
            {ok, Existing} ->
                gleam@dict:insert(Groups, F(Elem), [Elem | Existing]);

            {error, _} ->
                gleam@dict:insert(Groups, F(Elem), [Elem])
        end end.

-file("src/gleam/list.gleam", 321).
-spec filter_loop(list(ABH), fun((ABH) -> boolean()), list(ABH)) -> list(ABH).
filter_loop(List, Fun, Acc) ->
    case List of
        [] ->
            lists:reverse(Acc);

        [First | Rest] ->
            New_acc = case Fun(First) of
                true ->
                    [First | Acc];

                false ->
                    Acc
            end,
            filter_loop(Rest, Fun, New_acc)
    end.

-file("src/gleam/list.gleam", 317).
?DOC(
    " Returns a new list containing only the elements from the first list for\n"
    " which the given functions returns `True`.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " filter([2, 4, 6, 1], fn(x) { x > 2 })\n"
    " // -> [4, 6]\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " filter([2, 4, 6, 1], fn(x) { x > 6 })\n"
    " // -> []\n"
    " ```\n"
).
-spec filter(list(ABE), fun((ABE) -> boolean())) -> list(ABE).
filter(List, Predicate) ->
    filter_loop(List, Predicate, []).

-file("src/gleam/list.gleam", 353).
-spec filter_map_loop(
    list(ABS),
    fun((ABS) -> {ok, ABU} | {error, any()}),
    list(ABU)
) -> list(ABU).
filter_map_loop(List, Fun, Acc) ->
    case List of
        [] ->
            lists:reverse(Acc);

        [First | Rest] ->
            New_acc = case Fun(First) of
                {ok, First@1} ->
                    [First@1 | Acc];

                {error, _} ->
                    Acc
            end,
            filter_map_loop(Rest, Fun, New_acc)
    end.

-file("src/gleam/list.gleam", 349).
?DOC(
    " Returns a new list containing only the elements from the first list for\n"
    " which the given functions returns `Ok(_)`.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " filter_map([2, 4, 6, 1], Error)\n"
    " // -> []\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " filter_map([2, 4, 6, 1], fn(x) { Ok(x + 1) })\n"
    " // -> [3, 5, 7, 2]\n"
    " ```\n"
).
-spec filter_map(list(ABL), fun((ABL) -> {ok, ABN} | {error, any()})) -> list(ABN).
filter_map(List, Fun) ->
    filter_map_loop(List, Fun, []).

-file("src/gleam/list.gleam", 384).
-spec map_loop(list(ACE), fun((ACE) -> ACG), list(ACG)) -> list(ACG).
map_loop(List, Fun, Acc) ->
    case List of
        [] ->
            lists:reverse(Acc);

        [First | Rest] ->
            map_loop(Rest, Fun, [Fun(First) | Acc])
    end.

-file("src/gleam/list.gleam", 380).
?DOC(
    " Returns a new list containing only the elements of the first list after the\n"
    " function has been applied to each one.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " map([2, 4, 6], fn(x) { x * 2 })\n"
    " // -> [4, 8, 12]\n"
    " ```\n"
).
-spec map(list(ACA), fun((ACA) -> ACC)) -> list(ACC).
map(List, Fun) ->
    map_loop(List, Fun, []).

-file("src/gleam/list.gleam", 411).
-spec map2_loop(list(ACP), list(ACR), fun((ACP, ACR) -> ACT), list(ACT)) -> list(ACT).
map2_loop(List1, List2, Fun, Acc) ->
    case {List1, List2} of
        {[], _} ->
            lists:reverse(Acc);

        {_, []} ->
            lists:reverse(Acc);

        {[A | As_], [B | Bs]} ->
            map2_loop(As_, Bs, Fun, [Fun(A, B) | Acc])
    end.

-file("src/gleam/list.gleam", 407).
?DOC(
    " Combines two lists into a single list using the given function.\n"
    "\n"
    " If a list is longer than the other the extra elements are dropped.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " map2([1, 2, 3], [4, 5, 6], fn(x, y) { x + y })\n"
    " // -> [5, 7, 9]\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " map2([1, 2], [\"a\", \"b\", \"c\"], fn(i, x) { #(i, x) })\n"
    " // -> [#(1, \"a\"), #(2, \"b\")]\n"
    " ```\n"
).
-spec map2(list(ACJ), list(ACL), fun((ACJ, ACL) -> ACN)) -> list(ACN).
map2(List1, List2, Fun) ->
    map2_loop(List1, List2, Fun, []).

-file("src/gleam/list.gleam", 466).
-spec index_map_loop(
    list(ADF),
    fun((ADF, integer()) -> ADH),
    integer(),
    list(ADH)
) -> list(ADH).
index_map_loop(List, Fun, Index, Acc) ->
    case List of
        [] ->
            lists:reverse(Acc);

        [First | Rest] ->
            Acc@1 = [Fun(First, Index) | Acc],
            index_map_loop(Rest, Fun, Index + 1, Acc@1)
    end.

-file("src/gleam/list.gleam", 462).
?DOC(
    " Returns a new list containing only the elements of the first list after the\n"
    " function has been applied to each one and their index.\n"
    "\n"
    " The index starts at 0, so the first element is 0, the second is 1, and so\n"
    " on.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " index_map([\"a\", \"b\"], fn(x, i) { #(i, x) })\n"
    " // -> [#(0, \"a\"), #(1, \"b\")]\n"
    " ```\n"
).
-spec index_map(list(ADB), fun((ADB, integer()) -> ADD)) -> list(ADD).
index_map(List, Fun) ->
    index_map_loop(List, Fun, 0, []).

-file("src/gleam/list.gleam", 520).
-spec try_map_loop(list(ADT), fun((ADT) -> {ok, ADV} | {error, ADW}), list(ADV)) -> {ok,
        list(ADV)} |
    {error, ADW}.
try_map_loop(List, Fun, Acc) ->
    case List of
        [] ->
            {ok, lists:reverse(Acc)};

        [First | Rest] ->
            case Fun(First) of
                {ok, First@1} ->
                    try_map_loop(Rest, Fun, [First@1 | Acc]);

                {error, Error} ->
                    {error, Error}
            end
    end.

-file("src/gleam/list.gleam", 513).
?DOC(
    " Takes a function that returns a `Result` and applies it to each element in a\n"
    " given list in turn.\n"
    "\n"
    " If the function returns `Ok(new_value)` for all elements in the list then a\n"
    " list of the new values is returned.\n"
    "\n"
    " If the function returns `Error(reason)` for any of the elements then it is\n"
    " returned immediately. None of the elements in the list are processed after\n"
    " one returns an `Error`.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " try_map([1, 2, 3], fn(x) { Ok(x + 2) })\n"
    " // -> Ok([3, 4, 5])\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " try_map([1, 2, 3], fn(_) { Error(0) })\n"
    " // -> Error(0)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " try_map([[1], [2, 3]], first)\n"
    " // -> Ok([1, 2])\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " try_map([[1], [], [2]], first)\n"
    " // -> Error(Nil)\n"
    " ```\n"
).
-spec try_map(list(ADK), fun((ADK) -> {ok, ADM} | {error, ADN})) -> {ok,
        list(ADM)} |
    {error, ADN}.
try_map(List, Fun) ->
    try_map_loop(List, Fun, []).

-file("src/gleam/list.gleam", 555).
?DOC(
    " Returns a list that is the given list with up to the given number of\n"
    " elements removed from the front of the list.\n"
    "\n"
    " If the element has less than the number of elements an empty list is\n"
    " returned.\n"
    "\n"
    " This function runs in linear time but does not copy the list.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " drop([1, 2, 3, 4], 2)\n"
    " // -> [3, 4]\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " drop([1, 2, 3, 4], 9)\n"
    " // -> []\n"
    " ```\n"
).
-spec drop(list(AED), integer()) -> list(AED).
drop(List, N) ->
    case N =< 0 of
        true ->
            List;

        false ->
            case List of
                [] ->
                    [];

                [_ | Rest] ->
                    drop(Rest, N - 1)
            end
    end.

-file("src/gleam/list.gleam", 590).
-spec take_loop(list(AEJ), integer(), list(AEJ)) -> list(AEJ).
take_loop(List, N, Acc) ->
    case N =< 0 of
        true ->
            lists:reverse(Acc);

        false ->
            case List of
                [] ->
                    lists:reverse(Acc);

                [First | Rest] ->
                    take_loop(Rest, N - 1, [First | Acc])
            end
    end.

-file("src/gleam/list.gleam", 586).
?DOC(
    " Returns a list containing the first given number of elements from the given\n"
    " list.\n"
    "\n"
    " If the element has less than the number of elements then the full list is\n"
    " returned.\n"
    "\n"
    " This function runs in linear time.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " take([1, 2, 3, 4], 2)\n"
    " // -> [1, 2]\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " take([1, 2, 3, 4], 9)\n"
    " // -> [1, 2, 3, 4]\n"
    " ```\n"
).
-spec take(list(AEG), integer()) -> list(AEG).
take(List, N) ->
    take_loop(List, N, []).

-file("src/gleam/list.gleam", 610).
?DOC(
    " Returns a new empty list.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " new()\n"
    " // -> []\n"
    " ```\n"
).
-spec new() -> list(any()).
new() ->
    [].

-file("src/gleam/list.gleam", 630).
?DOC(
    " Returns the given item wrapped in a list.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " wrap(1)\n"
    " // -> [1]\n"
    "\n"
    " wrap([\"a\", \"b\", \"c\"])\n"
    " // -> [[\"a\", \"b\", \"c\"]]\n"
    "\n"
    " wrap([[]])\n"
    " // -> [[[]]]\n"
    " ```\n"
).
-spec wrap(AEP) -> list(AEP).
wrap(Item) ->
    [Item].

-file("src/gleam/list.gleam", 651).
-spec append_loop(list(AEV), list(AEV)) -> list(AEV).
append_loop(First, Second) ->
    case First of
        [] ->
            Second;

        [First@1 | Rest] ->
            append_loop(Rest, [First@1 | Second])
    end.

-file("src/gleam/list.gleam", 647).
?DOC(
    " Joins one list onto the end of another.\n"
    "\n"
    " This function runs in linear time, and it traverses and copies the first\n"
    " list.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " append([1, 2], [3])\n"
    " // -> [1, 2, 3]\n"
    " ```\n"
).
-spec append(list(AER), list(AER)) -> list(AER).
append(First, Second) ->
    lists:append(First, Second).

-file("src/gleam/list.gleam", 671).
?DOC(
    " Prefixes an item to a list. This can also be done using the dedicated\n"
    " syntax instead\n"
    "\n"
    " ```gleam\n"
    " let existing_list = [2, 3, 4]\n"
    "\n"
    " [1, ..existing_list]\n"
    " // -> [1, 2, 3, 4]\n"
    "\n"
    " prepend(to: existing_list, this: 1)\n"
    " // -> [1, 2, 3, 4]\n"
    " ```\n"
).
-spec prepend(list(AEZ), AEZ) -> list(AEZ).
prepend(List, Item) ->
    [Item | List].

-file("src/gleam/list.gleam", 675).
-spec flatten_loop(list(list(AFC)), list(AFC)) -> list(AFC).
flatten_loop(Lists, Acc) ->
    case Lists of
        [] ->
            lists:reverse(Acc);

        [List | Further_lists] ->
            flatten_loop(Further_lists, reverse_and_prepend(List, Acc))
    end.

-file("src/gleam/list.gleam", 695).
?DOC(
    " This is the same as `concat`: it joins a list of lists into a single\n"
    " list.\n"
    "\n"
    " This function traverses all elements twice.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " flatten([[1], [2, 3], []])\n"
    " // -> [1, 2, 3]\n"
    " ```\n"
).
-spec flatten(list(list(AFH))) -> list(AFH).
flatten(Lists) ->
    flatten_loop(Lists, []).

-file("src/gleam/list.gleam", 708).
?DOC(
    " Maps the list with the given function into a list of lists, and then flattens it.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " flat_map([2, 4, 6], fn(x) { [x, x + 1] })\n"
    " // -> [2, 3, 4, 5, 6, 7]\n"
    " ```\n"
).
-spec flat_map(list(AFL), fun((AFL) -> list(AFN))) -> list(AFN).
flat_map(List, Fun) ->
    _pipe = map(List, Fun),
    flatten(_pipe).

-file("src/gleam/list.gleam", 721).
?DOC(
    " Reduces a list of elements into a single value by calling a given function\n"
    " on each element, going from left to right.\n"
    "\n"
    " `fold([1, 2, 3], 0, add)` is the equivalent of\n"
    " `add(add(add(0, 1), 2), 3)`.\n"
    "\n"
    " This function runs in linear time.\n"
).
-spec fold(list(AFQ), AFS, fun((AFS, AFQ) -> AFS)) -> AFS.
fold(List, Initial, Fun) ->
    case List of
        [] ->
            Initial;

        [First | Rest] ->
            fold(Rest, Fun(Initial, First), Fun)
    end.

-file("src/gleam/list.gleam", 90).
?DOC(
    " Counts the number of elements in a given list satisfying a given predicate.\n"
    "\n"
    " This function has to traverse the list to determine the number of elements,\n"
    " so it runs in linear time.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " count([], fn(a) { a > 0 })\n"
    " // -> 0\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " count([1], fn(a) { a > 0 })\n"
    " // -> 1\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " count([1, 2, 3], int.is_odd)\n"
    " // -> 2\n"
    " ```\n"
).
-spec count(list(YU), fun((YU) -> boolean())) -> integer().
count(List, Predicate) ->
    fold(List, 0, fun(Acc, Value) -> case Predicate(Value) of
                true ->
                    Acc + 1;

                false ->
                    Acc
            end end).

-file("src/gleam/list.gleam", 289).
?DOC(
    " Groups the elements from the given list by the given key function.\n"
    "\n"
    " Does not preserve the initial value order.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " import gleam/dict\n"
    "\n"
    " [Ok(3), Error(\"Wrong\"), Ok(200), Ok(73)]\n"
    " |> group(by: fn(i) {\n"
    "   case i {\n"
    "     Ok(_) -> \"Successful\"\n"
    "     Error(_) -> \"Failed\"\n"
    "   }\n"
    " })\n"
    " |> dict.to_list\n"
    " // -> [\n"
    " //   #(\"Failed\", [Error(\"Wrong\")]),\n"
    " //   #(\"Successful\", [Ok(73), Ok(200), Ok(3)])\n"
    " // ]\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " import gleam/dict\n"
    "\n"
    " group([1,2,3,4,5], by: fn(i) { i - i / 3 * 3 })\n"
    " |> dict.to_list\n"
    " // -> [#(0, [3]), #(1, [4, 1]), #(2, [5, 2])]\n"
    " ```\n"
).
-spec group(list(AAQ), fun((AAQ) -> AAS)) -> gleam@dict:dict(AAS, list(AAQ)).
group(List, Key) ->
    fold(List, maps:new(), update_group(Key)).

-file("src/gleam/list.gleam", 436).
?DOC(
    " Similar to `map` but also lets you pass around an accumulated value.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " map_fold(\n"
    "   over: [1, 2, 3],\n"
    "   from: 100,\n"
    "   with: fn(memo, i) { #(memo + i, i * 2) }\n"
    " )\n"
    " // -> #(106, [2, 4, 6])\n"
    " ```\n"
).
-spec map_fold(list(ACW), ACY, fun((ACY, ACW) -> {ACY, ACZ})) -> {ACY,
    list(ACZ)}.
map_fold(List, Initial, Fun) ->
    _pipe = fold(
        List,
        {Initial, []},
        fun(Acc, Item) ->
            {Current_acc, Items} = Acc,
            {Next_acc, Next_item} = Fun(Current_acc, Item),
            {Next_acc, [Next_item | Items]}
        end
    ),
    gleam@pair:map_second(_pipe, fun lists:reverse/1).

-file("src/gleam/list.gleam", 743).
?DOC(
    " Reduces a list of elements into a single value by calling a given function\n"
    " on each element, going from right to left.\n"
    "\n"
    " `fold_right([1, 2, 3], 0, add)` is the equivalent of\n"
    " `add(add(add(0, 3), 2), 1)`.\n"
    "\n"
    " This function runs in linear time.\n"
    "\n"
    " Unlike `fold` this function is not tail recursive. Where possible use\n"
    " `fold` instead as it will use less memory.\n"
).
-spec fold_right(list(AFT), AFV, fun((AFV, AFT) -> AFV)) -> AFV.
fold_right(List, Initial, Fun) ->
    case List of
        [] ->
            Initial;

        [First | Rest] ->
            Fun(fold_right(Rest, Initial, Fun), First)
    end.

-file("src/gleam/list.gleam", 771).
-spec index_fold_loop(
    list(AFZ),
    AGB,
    fun((AGB, AFZ, integer()) -> AGB),
    integer()
) -> AGB.
index_fold_loop(Over, Acc, With, Index) ->
    case Over of
        [] ->
            Acc;

        [First | Rest] ->
            index_fold_loop(Rest, With(Acc, First, Index), With, Index + 1)
    end.

-file("src/gleam/list.gleam", 763).
?DOC(
    " Like fold but the folding function also receives the index of the current element.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " [\"a\", \"b\", \"c\"]\n"
    " |> index_fold([], fn(acc, item, index) { ... })\n"
    " ```\n"
).
-spec index_fold(list(AFW), AFY, fun((AFY, AFW, integer()) -> AFY)) -> AFY.
index_fold(List, Initial, Fun) ->
    index_fold_loop(List, Initial, Fun, 0).

-file("src/gleam/list.gleam", 803).
?DOC(
    " A variant of fold that might fail.\n"
    "\n"
    " The folding function should return `Result(accumulator, error)`.\n"
    " If the returned value is `Ok(accumulator)` try_fold will try the next value in the list.\n"
    " If the returned value is `Error(error)` try_fold will stop and return that error.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " [1, 2, 3, 4]\n"
    " |> try_fold(0, fn(acc, i) {\n"
    "   case i < 3 {\n"
    "     True -> Ok(acc + i)\n"
    "     False -> Error(Nil)\n"
    "   }\n"
    " })\n"
    " // -> Error(Nil)\n"
    " ```\n"
).
-spec try_fold(list(AGC), AGE, fun((AGE, AGC) -> {ok, AGE} | {error, AGF})) -> {ok,
        AGE} |
    {error, AGF}.
try_fold(List, Initial, Fun) ->
    case List of
        [] ->
            {ok, Initial};

        [First | Rest] ->
            case Fun(Initial, First) of
                {ok, Result} ->
                    try_fold(Rest, Result, Fun);

                {error, _} = Error ->
                    Error
            end
    end.

-file("src/gleam/list.gleam", 842).
?DOC(
    " A variant of fold that allows to stop folding earlier.\n"
    "\n"
    " The folding function should return `ContinueOrStop(accumulator)`.\n"
    " If the returned value is `Continue(accumulator)` fold_until will try the next value in the list.\n"
    " If the returned value is `Stop(accumulator)` fold_until will stop and return that accumulator.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " [1, 2, 3, 4]\n"
    " |> fold_until(0, fn(acc, i) {\n"
    "   case i < 3 {\n"
    "     True -> Continue(acc + i)\n"
    "     False -> Stop(acc)\n"
    "   }\n"
    " })\n"
    " // -> 3\n"
    " ```\n"
).
-spec fold_until(list(AGK), AGM, fun((AGM, AGK) -> continue_or_stop(AGM))) -> AGM.
fold_until(List, Initial, Fun) ->
    case List of
        [] ->
            Initial;

        [First | Rest] ->
            case Fun(Initial, First) of
                {continue, Next_accumulator} ->
                    fold_until(Rest, Next_accumulator, Fun);

                {stop, B} ->
                    B
            end
    end.

-file("src/gleam/list.gleam", 879).
?DOC(
    " Finds the first element in a given list for which the given function returns\n"
    " `True`.\n"
    "\n"
    " Returns `Error(Nil)` if no such element is found.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " find([1, 2, 3], fn(x) { x > 2 })\n"
    " // -> Ok(3)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " find([1, 2, 3], fn(x) { x > 4 })\n"
    " // -> Error(Nil)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " find([], fn(_) { True })\n"
    " // -> Error(Nil)\n"
    " ```\n"
).
-spec find(list(AGO), fun((AGO) -> boolean())) -> {ok, AGO} | {error, nil}.
find(List, Is_desired) ->
    case List of
        [] ->
            {error, nil};

        [First | Rest] ->
            case Is_desired(First) of
                true ->
                    {ok, First};

                false ->
                    find(Rest, Is_desired)
            end
    end.

-file("src/gleam/list.gleam", 915).
?DOC(
    " Finds the first element in a given list for which the given function returns\n"
    " `Ok(new_value)`, then returns the wrapped `new_value`.\n"
    "\n"
    " Returns `Error(Nil)` if no such element is found.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " find_map([[], [2], [3]], first)\n"
    " // -> Ok(2)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " find_map([[], []], first)\n"
    " // -> Error(Nil)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " find_map([], first)\n"
    " // -> Error(Nil)\n"
    " ```\n"
).
-spec find_map(list(AGS), fun((AGS) -> {ok, AGU} | {error, any()})) -> {ok, AGU} |
    {error, nil}.
find_map(List, Fun) ->
    case List of
        [] ->
            {error, nil};

        [First | Rest] ->
            case Fun(First) of
                {ok, First@1} ->
                    {ok, First@1};

                {error, _} ->
                    find_map(Rest, Fun)
            end
    end.

-file("src/gleam/list.gleam", 950).
?DOC(
    " Returns `True` if the given function returns `True` for all the elements in\n"
    " the given list. If the function returns `False` for any of the elements it\n"
    " immediately returns `False` without checking the rest of the list.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " all([], fn(x) { x > 3 })\n"
    " // -> True\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " all([4, 5], fn(x) { x > 3 })\n"
    " // -> True\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " all([4, 3], fn(x) { x > 3 })\n"
    " // -> False\n"
    " ```\n"
).
-spec all(list(AHA), fun((AHA) -> boolean())) -> boolean().
all(List, Predicate) ->
    case List of
        [] ->
            true;

        [First | Rest] ->
            case Predicate(First) of
                true ->
                    all(Rest, Predicate);

                false ->
                    false
            end
    end.

-file("src/gleam/list.gleam", 987).
?DOC(
    " Returns `True` if the given function returns `True` for any the elements in\n"
    " the given list. If the function returns `True` for any of the elements it\n"
    " immediately returns `True` without checking the rest of the list.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " any([], fn(x) { x > 3 })\n"
    " // -> False\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " any([4, 5], fn(x) { x > 3 })\n"
    " // -> True\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " any([4, 3], fn(x) { x > 4 })\n"
    " // -> False\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " any([3, 4], fn(x) { x > 3 })\n"
    " // -> True\n"
    " ```\n"
).
-spec any(list(AHC), fun((AHC) -> boolean())) -> boolean().
any(List, Predicate) ->
    case List of
        [] ->
            false;

        [First | Rest] ->
            case Predicate(First) of
                true ->
                    true;

                false ->
                    any(Rest, Predicate)
            end
    end.

-file("src/gleam/list.gleam", 1029).
-spec zip_loop(list(AHJ), list(AHL), list({AHJ, AHL})) -> list({AHJ, AHL}).
zip_loop(One, Other, Acc) ->
    case {One, Other} of
        {[First_one | Rest_one], [First_other | Rest_other]} ->
            zip_loop(Rest_one, Rest_other, [{First_one, First_other} | Acc]);

        {_, _} ->
            lists:reverse(Acc)
    end.

-file("src/gleam/list.gleam", 1025).
?DOC(
    " Takes two lists and returns a single list of 2-element tuples.\n"
    "\n"
    " If one of the lists is longer than the other, the remaining elements from\n"
    " the longer list are not used.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " zip([], [])\n"
    " // -> []\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " zip([1, 2], [3])\n"
    " // -> [#(1, 3)]\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " zip([1], [3, 4])\n"
    " // -> [#(1, 3)]\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " zip([1, 2], [3, 4])\n"
    " // -> [#(1, 3), #(2, 4)]\n"
    " ```\n"
).
-spec zip(list(AHE), list(AHG)) -> list({AHE, AHG}).
zip(List, Other) ->
    zip_loop(List, Other, []).

-file("src/gleam/list.gleam", 1070).
-spec strict_zip_loop(list(AHW), list(AHY), list({AHW, AHY})) -> {ok,
        list({AHW, AHY})} |
    {error, nil}.
strict_zip_loop(One, Other, Acc) ->
    case {One, Other} of
        {[], []} ->
            {ok, lists:reverse(Acc)};

        {[], _} ->
            {error, nil};

        {_, []} ->
            {error, nil};

        {[First_one | Rest_one], [First_other | Rest_other]} ->
            strict_zip_loop(
                Rest_one,
                Rest_other,
                [{First_one, First_other} | Acc]
            )
    end.

-file("src/gleam/list.gleam", 1063).
?DOC(
    " Takes two lists and returns a single list of 2-element tuples.\n"
    "\n"
    " If one of the lists is longer than the other, an `Error` is returned.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " strict_zip([], [])\n"
    " // -> Ok([])\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " strict_zip([1, 2], [3])\n"
    " // -> Error(Nil)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " strict_zip([1], [3, 4])\n"
    " // -> Error(Nil)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " strict_zip([1, 2], [3, 4])\n"
    " // -> Ok([#(1, 3), #(2, 4)])\n"
    " ```\n"
).
-spec strict_zip(list(AHP), list(AHR)) -> {ok, list({AHP, AHR})} | {error, nil}.
strict_zip(List, Other) ->
    strict_zip_loop(List, Other, []).

-file("src/gleam/list.gleam", 1101).
-spec unzip_loop(list({AIJ, AIK}), list(AIJ), list(AIK)) -> {list(AIJ),
    list(AIK)}.
unzip_loop(Input, One, Other) ->
    case Input of
        [] ->
            {lists:reverse(One), lists:reverse(Other)};

        [{First_one, First_other} | Rest] ->
            unzip_loop(Rest, [First_one | One], [First_other | Other])
    end.

-file("src/gleam/list.gleam", 1097).
?DOC(
    " Takes a single list of 2-element tuples and returns two lists.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " unzip([#(1, 2), #(3, 4)])\n"
    " // -> #([1, 3], [2, 4])\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " unzip([])\n"
    " // -> #([], [])\n"
    " ```\n"
).
-spec unzip(list({AIE, AIF})) -> {list(AIE), list(AIF)}.
unzip(Input) ->
    unzip_loop(Input, [], []).

-file("src/gleam/list.gleam", 1136).
-spec intersperse_loop(list(AIT), AIT, list(AIT)) -> list(AIT).
intersperse_loop(List, Separator, Acc) ->
    case List of
        [] ->
            lists:reverse(Acc);

        [First | Rest] ->
            intersperse_loop(Rest, Separator, [First, Separator | Acc])
    end.

-file("src/gleam/list.gleam", 1129).
?DOC(
    " Inserts a given value between each existing element in a given list.\n"
    "\n"
    " This function runs in linear time and copies the list.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " intersperse([1, 1, 1], 2)\n"
    " // -> [1, 2, 1, 2, 1]\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " intersperse([], 2)\n"
    " // -> []\n"
    " ```\n"
).
-spec intersperse(list(AIQ), AIQ) -> list(AIQ).
intersperse(List, Elem) ->
    case List of
        [] ->
            List;

        [_] ->
            List;

        [First | Rest] ->
            intersperse_loop(Rest, Elem, [First])
    end.

-file("src/gleam/list.gleam", 1159).
-spec unique_loop(list(AJA), gleam@dict:dict(AJA, nil), list(AJA)) -> list(AJA).
unique_loop(List, Seen, Acc) ->
    case List of
        [] ->
            lists:reverse(Acc);

        [First | Rest] ->
            case gleam@dict:has_key(Seen, First) of
                true ->
                    unique_loop(Rest, Seen, Acc);

                false ->
                    unique_loop(
                        Rest,
                        gleam@dict:insert(Seen, First, nil),
                        [First | Acc]
                    )
            end
    end.

-file("src/gleam/list.gleam", 1155).
?DOC(
    " Removes any duplicate elements from a given list.\n"
    "\n"
    " This function returns in loglinear time.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " unique([1, 1, 1, 4, 7, 3, 3, 4])\n"
    " // -> [1, 4, 7, 3]\n"
    " ```\n"
).
-spec unique(list(AIX)) -> list(AIX).
unique(List) ->
    unique_loop(List, maps:new(), []).

-file("src/gleam/list.gleam", 1245).
?DOC(
    " Given a list it returns slices of it that are locally sorted in ascending\n"
    " order.\n"
    "\n"
    " Imagine you have this list:\n"
    "\n"
    " ```\n"
    "   [1, 2, 3, 2, 1, 0]\n"
    "    ^^^^^^^  ^^^^^^^ This is a slice in descending order\n"
    "    |\n"
    "    | This is a slice that is sorted in ascending order\n"
    " ```\n"
    "\n"
    " So the produced result will contain these two slices, each one sorted in\n"
    " ascending order: `[[1, 2, 3], [0, 1, 2]]`.\n"
    "\n"
    " - `growing` is an accumulator with the current slice being grown\n"
    " - `direction` is the growing direction of the slice being grown, it could\n"
    "   either be ascending or strictly descending\n"
    " - `prev` is the previous element that needs to be added to the growing slice\n"
    "   it is carried around to check whether we have to keep growing the current\n"
    "   slice or not\n"
    " - `acc` is the accumulator containing the slices sorted in ascending order\n"
).
-spec sequences(
    list(AJJ),
    fun((AJJ, AJJ) -> gleam@order:order()),
    list(AJJ),
    sorting(),
    AJJ,
    list(list(AJJ))
) -> list(list(AJJ)).
sequences(List, Compare, Growing, Direction, Prev, Acc) ->
    Growing@1 = [Prev | Growing],
    case List of
        [] ->
            case Direction of
                ascending ->
                    [lists:reverse(Growing@1) | Acc];

                descending ->
                    [Growing@1 | Acc]
            end;

        [New | Rest] ->
            case {Compare(Prev, New), Direction} of
                {gt, descending} ->
                    sequences(Rest, Compare, Growing@1, Direction, New, Acc);

                {lt, ascending} ->
                    sequences(Rest, Compare, Growing@1, Direction, New, Acc);

                {eq, ascending} ->
                    sequences(Rest, Compare, Growing@1, Direction, New, Acc);

                {gt, ascending} ->
                    Acc@1 = case Direction of
                        ascending ->
                            [lists:reverse(Growing@1) | Acc];

                        descending ->
                            [Growing@1 | Acc]
                    end,
                    case Rest of
                        [] ->
                            [[New] | Acc@1];

                        [Next | Rest@1] ->
                            Direction@1 = case Compare(New, Next) of
                                lt ->
                                    ascending;

                                eq ->
                                    ascending;

                                gt ->
                                    descending
                            end,
                            sequences(
                                Rest@1,
                                Compare,
                                [New],
                                Direction@1,
                                Next,
                                Acc@1
                            )
                    end;

                {lt, descending} ->
                    Acc@1 = case Direction of
                        ascending ->
                            [lists:reverse(Growing@1) | Acc];

                        descending ->
                            [Growing@1 | Acc]
                    end,
                    case Rest of
                        [] ->
                            [[New] | Acc@1];

                        [Next | Rest@1] ->
                            Direction@1 = case Compare(New, Next) of
                                lt ->
                                    ascending;

                                eq ->
                                    ascending;

                                gt ->
                                    descending
                            end,
                            sequences(
                                Rest@1,
                                Compare,
                                [New],
                                Direction@1,
                                Next,
                                Acc@1
                            )
                    end;

                {eq, descending} ->
                    Acc@1 = case Direction of
                        ascending ->
                            [lists:reverse(Growing@1) | Acc];

                        descending ->
                            [Growing@1 | Acc]
                    end,
                    case Rest of
                        [] ->
                            [[New] | Acc@1];

                        [Next | Rest@1] ->
                            Direction@1 = case Compare(New, Next) of
                                lt ->
                                    ascending;

                                eq ->
                                    ascending;

                                gt ->
                                    descending
                            end,
                            sequences(
                                Rest@1,
                                Compare,
                                [New],
                                Direction@1,
                                Next,
                                Acc@1
                            )
                    end
            end
    end.

-file("src/gleam/list.gleam", 1393).
?DOC(
    " Merges two lists sorted in ascending order into a single list sorted in\n"
    " descending order according to the given comparator function.\n"
    "\n"
    " This reversing of the sort order is not avoidable if we want to implement\n"
    " merge as a tail recursive function. We could reverse the accumulator before\n"
    " returning it but that would end up being less efficient; so the merging\n"
    " algorithm has to play around this.\n"
).
-spec merge_ascendings(
    list(AKG),
    list(AKG),
    fun((AKG, AKG) -> gleam@order:order()),
    list(AKG)
) -> list(AKG).
merge_ascendings(List1, List2, Compare, Acc) ->
    case {List1, List2} of
        {[], List} ->
            reverse_and_prepend(List, Acc);

        {List, []} ->
            reverse_and_prepend(List, Acc);

        {[First1 | Rest1], [First2 | Rest2]} ->
            case Compare(First1, First2) of
                lt ->
                    merge_ascendings(Rest1, List2, Compare, [First1 | Acc]);

                gt ->
                    merge_ascendings(List1, Rest2, Compare, [First2 | Acc]);

                eq ->
                    merge_ascendings(List1, Rest2, Compare, [First2 | Acc])
            end
    end.

-file("src/gleam/list.gleam", 1346).
?DOC(
    " Given a list of ascending lists, it merges adjacent pairs into a single\n"
    " descending list, halving their number.\n"
    " It returns a list of the remaining descending lists.\n"
).
-spec merge_ascending_pairs(
    list(list(AJU)),
    fun((AJU, AJU) -> gleam@order:order()),
    list(list(AJU))
) -> list(list(AJU)).
merge_ascending_pairs(Sequences, Compare, Acc) ->
    case Sequences of
        [] ->
            lists:reverse(Acc);

        [Sequence] ->
            lists:reverse([lists:reverse(Sequence) | Acc]);

        [Ascending1, Ascending2 | Rest] ->
            Descending = merge_ascendings(Ascending1, Ascending2, Compare, []),
            merge_ascending_pairs(Rest, Compare, [Descending | Acc])
    end.

-file("src/gleam/list.gleam", 1420).
?DOC(
    " This is exactly the same as merge_ascendings but mirrored: it merges two\n"
    " lists sorted in descending order into a single list sorted in ascending\n"
    " order according to the given comparator function.\n"
    "\n"
    " This reversing of the sort order is not avoidable if we want to implement\n"
    " merge as a tail recursive function. We could reverse the accumulator before\n"
    " returning it but that would end up being less efficient; so the merging\n"
    " algorithm has to play around this.\n"
).
-spec merge_descendings(
    list(AKL),
    list(AKL),
    fun((AKL, AKL) -> gleam@order:order()),
    list(AKL)
) -> list(AKL).
merge_descendings(List1, List2, Compare, Acc) ->
    case {List1, List2} of
        {[], List} ->
            reverse_and_prepend(List, Acc);

        {List, []} ->
            reverse_and_prepend(List, Acc);

        {[First1 | Rest1], [First2 | Rest2]} ->
            case Compare(First1, First2) of
                lt ->
                    merge_descendings(List1, Rest2, Compare, [First2 | Acc]);

                gt ->
                    merge_descendings(Rest1, List2, Compare, [First1 | Acc]);

                eq ->
                    merge_descendings(Rest1, List2, Compare, [First1 | Acc])
            end
    end.

-file("src/gleam/list.gleam", 1368).
?DOC(" This is the same as merge_ascending_pairs but flipped for descending lists.\n").
-spec merge_descending_pairs(
    list(list(AKA)),
    fun((AKA, AKA) -> gleam@order:order()),
    list(list(AKA))
) -> list(list(AKA)).
merge_descending_pairs(Sequences, Compare, Acc) ->
    case Sequences of
        [] ->
            lists:reverse(Acc);

        [Sequence] ->
            lists:reverse([lists:reverse(Sequence) | Acc]);

        [Descending1, Descending2 | Rest] ->
            Ascending = merge_descendings(Descending1, Descending2, Compare, []),
            merge_descending_pairs(Rest, Compare, [Ascending | Acc])
    end.

-file("src/gleam/list.gleam", 1312).
?DOC(
    " Given some some sorted sequences (assumed to be sorted in `direction`) it\n"
    " merges them all together until we're left with just a list sorted in\n"
    " ascending order.\n"
).
-spec merge_all(
    list(list(AJQ)),
    sorting(),
    fun((AJQ, AJQ) -> gleam@order:order())
) -> list(AJQ).
merge_all(Sequences, Direction, Compare) ->
    case {Sequences, Direction} of
        {[], _} ->
            [];

        {[Sequence], ascending} ->
            Sequence;

        {[Sequence@1], descending} ->
            lists:reverse(Sequence@1);

        {_, ascending} ->
            Sequences@1 = merge_ascending_pairs(Sequences, Compare, []),
            merge_all(Sequences@1, descending, Compare);

        {_, descending} ->
            Sequences@2 = merge_descending_pairs(Sequences, Compare, []),
            merge_all(Sequences@2, ascending, Compare)
    end.

-file("src/gleam/list.gleam", 1183).
?DOC(
    " Sorts from smallest to largest based upon the ordering specified by a given\n"
    " function.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " import gleam/int\n"
    "\n"
    " sort([4, 3, 6, 5, 4, 1, 2], by: int.compare)\n"
    " // -> [1, 2, 3, 4, 4, 5, 6]\n"
    " ```\n"
).
-spec sort(list(AJG), fun((AJG, AJG) -> gleam@order:order())) -> list(AJG).
sort(List, Compare) ->
    case List of
        [] ->
            [];

        [X] ->
            [X];

        [X@1, Y | Rest] ->
            Direction = case Compare(X@1, Y) of
                lt ->
                    ascending;

                eq ->
                    ascending;

                gt ->
                    descending
            end,
            Sequences = sequences(Rest, Compare, [X@1], Direction, Y, []),
            merge_all(Sequences, ascending, Compare)
    end.

-file("src/gleam/list.gleam", 1460).
-spec range_loop(integer(), integer(), list(integer())) -> list(integer()).
range_loop(Start, Stop, Acc) ->
    case gleam@int:compare(Start, Stop) of
        eq ->
            [Stop | Acc];

        gt ->
            range_loop(Start, Stop + 1, [Stop | Acc]);

        lt ->
            range_loop(Start, Stop - 1, [Stop | Acc])
    end.

-file("src/gleam/list.gleam", 1456).
?DOC(
    " Creates a list of ints ranging from a given start and finish.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " range(0, 0)\n"
    " // -> [0]\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " range(0, 5)\n"
    " // -> [0, 1, 2, 3, 4, 5]\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " range(1, -5)\n"
    " // -> [1, 0, -1, -2, -3, -4, -5]\n"
    " ```\n"
).
-spec range(integer(), integer()) -> list(integer()).
range(Start, Stop) ->
    range_loop(Start, Stop, []).

-file("src/gleam/list.gleam", 1486).
-spec repeat_loop(AKV, integer(), list(AKV)) -> list(AKV).
repeat_loop(Item, Times, Acc) ->
    case Times =< 0 of
        true ->
            Acc;

        false ->
            repeat_loop(Item, Times - 1, [Item | Acc])
    end.

-file("src/gleam/list.gleam", 1482).
?DOC(
    " Builds a list of a given value a given number of times.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " repeat(\"a\", times: 0)\n"
    " // -> []\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " repeat(\"a\", times: 5)\n"
    " // -> [\"a\", \"a\", \"a\", \"a\", \"a\"]\n"
    " ```\n"
).
-spec repeat(AKT, integer()) -> list(AKT).
repeat(A, Times) ->
    repeat_loop(A, Times, []).

-file("src/gleam/list.gleam", 1519).
-spec split_loop(list(ALC), integer(), list(ALC)) -> {list(ALC), list(ALC)}.
split_loop(List, N, Taken) ->
    case N =< 0 of
        true ->
            {lists:reverse(Taken), List};

        false ->
            case List of
                [] ->
                    {lists:reverse(Taken), []};

                [First | Rest] ->
                    split_loop(Rest, N - 1, [First | Taken])
            end
    end.

-file("src/gleam/list.gleam", 1515).
?DOC(
    " Splits a list in two before the given index.\n"
    "\n"
    " If the list is not long enough to have the given index the before list will\n"
    " be the input list, and the after list will be empty.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " split([6, 7, 8, 9], 0)\n"
    " // -> #([], [6, 7, 8, 9])\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " split([6, 7, 8, 9], 2)\n"
    " // -> #([6, 7], [8, 9])\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " split([6, 7, 8, 9], 4)\n"
    " // -> #([6, 7, 8, 9], [])\n"
    " ```\n"
).
-spec split(list(AKY), integer()) -> {list(AKY), list(AKY)}.
split(List, Index) ->
    split_loop(List, Index, []).

-file("src/gleam/list.gleam", 1555).
-spec split_while_loop(list(ALL), fun((ALL) -> boolean()), list(ALL)) -> {list(ALL),
    list(ALL)}.
split_while_loop(List, F, Acc) ->
    case List of
        [] ->
            {lists:reverse(Acc), []};

        [First | Rest] ->
            case F(First) of
                true ->
                    split_while_loop(Rest, F, [First | Acc]);

                false ->
                    {lists:reverse(Acc), List}
            end
    end.

-file("src/gleam/list.gleam", 1548).
?DOC(
    " Splits a list in two before the first element that a given function returns\n"
    " `False` for.\n"
    "\n"
    " If the function returns `True` for all elements the first list will be the\n"
    " input list, and the second list will be empty.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " split_while([1, 2, 3, 4, 5], fn(x) { x <= 3 })\n"
    " // -> #([1, 2, 3], [4, 5])\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " split_while([1, 2, 3, 4, 5], fn(x) { x <= 5 })\n"
    " // -> #([1, 2, 3, 4, 5], [])\n"
    " ```\n"
).
-spec split_while(list(ALH), fun((ALH) -> boolean())) -> {list(ALH), list(ALH)}.
split_while(List, Predicate) ->
    split_while_loop(List, Predicate, []).

-file("src/gleam/list.gleam", 1595).
?DOC(
    " Given a list of 2-element tuples, finds the first tuple that has a given\n"
    " key as the first element and returns the second element.\n"
    "\n"
    " If no tuple is found with the given key then `Error(Nil)` is returned.\n"
    "\n"
    " This function may be useful for interacting with Erlang code where lists of\n"
    " tuples are common.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " key_find([#(\"a\", 0), #(\"b\", 1)], \"a\")\n"
    " // -> Ok(0)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " key_find([#(\"a\", 0), #(\"b\", 1)], \"b\")\n"
    " // -> Ok(1)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " key_find([#(\"a\", 0), #(\"b\", 1)], \"c\")\n"
    " // -> Error(Nil)\n"
    " ```\n"
).
-spec key_find(list({ALQ, ALR}), ALQ) -> {ok, ALR} | {error, nil}.
key_find(Keyword_list, Desired_key) ->
    find_map(
        Keyword_list,
        fun(Keyword) ->
            {Key, Value} = Keyword,
            case Key =:= Desired_key of
                true ->
                    {ok, Value};

                false ->
                    {error, nil}
            end
        end
    ).

-file("src/gleam/list.gleam", 1626).
?DOC(
    " Given a list of 2-element tuples, finds all tuples that have a given\n"
    " key as the first element and returns the second element.\n"
    "\n"
    " This function may be useful for interacting with Erlang code where lists of\n"
    " tuples are common.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " key_filter([#(\"a\", 0), #(\"b\", 1), #(\"a\", 2)], \"a\")\n"
    " // -> [0, 2]\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " key_filter([#(\"a\", 0), #(\"b\", 1)], \"c\")\n"
    " // -> []\n"
    " ```\n"
).
-spec key_filter(list({ALV, ALW}), ALV) -> list(ALW).
key_filter(Keyword_list, Desired_key) ->
    filter_map(
        Keyword_list,
        fun(Keyword) ->
            {Key, Value} = Keyword,
            case Key =:= Desired_key of
                true ->
                    {ok, Value};

                false ->
                    {error, nil}
            end
        end
    ).

-file("src/gleam/list.gleam", 1668).
-spec pop_loop(list(BFB), fun((BFB) -> boolean()), list(BFB)) -> {ok,
        {BFB, list(BFB)}} |
    {error, nil}.
pop_loop(Haystack, Predicate, Checked) ->
    case Haystack of
        [] ->
            {error, nil};

        [First | Rest] ->
            case Predicate(First) of
                true ->
                    {ok, {First, lists:append(lists:reverse(Checked), Rest)}};

                false ->
                    pop_loop(Rest, Predicate, [First | Checked])
            end
    end.

-file("src/gleam/list.gleam", 1661).
?DOC(
    " Removes the first element in a given list for which the predicate function returns `True`.\n"
    "\n"
    " Returns `Error(Nil)` if no such element is found.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " pop([1, 2, 3], fn(x) { x > 2 })\n"
    " // -> Ok(#(3, [1, 2]))\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " pop([1, 2, 3], fn(x) { x > 4 })\n"
    " // -> Error(Nil)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " pop([], fn(_) { True })\n"
    " // -> Error(Nil)\n"
    " ```\n"
).
-spec pop(list(ALZ), fun((ALZ) -> boolean())) -> {ok, {ALZ, list(ALZ)}} |
    {error, nil}.
pop(List, Is_desired) ->
    pop_loop(List, Is_desired, []).

-file("src/gleam/list.gleam", 1709).
-spec pop_map_loop(
    list(AMR),
    fun((AMR) -> {ok, AMT} | {error, any()}),
    list(AMR)
) -> {ok, {AMT, list(AMR)}} | {error, nil}.
pop_map_loop(List, Mapper, Checked) ->
    case List of
        [] ->
            {error, nil};

        [First | Rest] ->
            case Mapper(First) of
                {ok, Mapped} ->
                    {ok, {Mapped, lists:append(lists:reverse(Checked), Rest)}};

                {error, _} ->
                    pop_map_loop(Rest, Mapper, [First | Checked])
            end
    end.

-file("src/gleam/list.gleam", 1702).
?DOC(
    " Removes the first element in a given list for which the given function returns\n"
    " `Ok(new_value)`, then returns the wrapped `new_value` as well as list with the value removed.\n"
    "\n"
    " Returns `Error(Nil)` if no such element is found.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " pop_map([[], [2], [3]], first)\n"
    " // -> Ok(#(2, [[], [3]]))\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " pop_map([[], []], first)\n"
    " // -> Error(Nil)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " pop_map([], first)\n"
    " // -> Error(Nil)\n"
    " ```\n"
).
-spec pop_map(list(AMI), fun((AMI) -> {ok, AMK} | {error, any()})) -> {ok,
        {AMK, list(AMI)}} |
    {error, nil}.
pop_map(Haystack, Is_desired) ->
    pop_map_loop(Haystack, Is_desired, []).

-file("src/gleam/list.gleam", 1751).
-spec key_pop_loop(list({ANH, ANI}), ANH, list({ANH, ANI})) -> {ok,
        {ANI, list({ANH, ANI})}} |
    {error, nil}.
key_pop_loop(List, Key, Checked) ->
    case List of
        [] ->
            {error, nil};

        [{K, V} | Rest] when K =:= Key ->
            {ok, {V, reverse_and_prepend(Checked, Rest)}};

        [First | Rest@1] ->
            key_pop_loop(Rest@1, Key, [First | Checked])
    end.

-file("src/gleam/list.gleam", 1747).
?DOC(
    " Given a list of 2-element tuples, finds the first tuple that has a given\n"
    " key as the first element. This function will return the second element\n"
    " of the found tuple and list with tuple removed.\n"
    "\n"
    " If no tuple is found with the given key then `Error(Nil)` is returned.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " key_pop([#(\"a\", 0), #(\"b\", 1)], \"a\")\n"
    " // -> Ok(#(0, [#(\"b\", 1)]))\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " key_pop([#(\"a\", 0), #(\"b\", 1)], \"b\")\n"
    " // -> Ok(#(1, [#(\"a\", 0)]))\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " key_pop([#(\"a\", 0), #(\"b\", 1)], \"c\")\n"
    " // -> Error(Nil)\n"
    " ```\n"
).
-spec key_pop(list({ANB, ANC}), ANB) -> {ok, {ANC, list({ANB, ANC})}} |
    {error, nil}.
key_pop(List, Key) ->
    key_pop_loop(List, Key, []).

-file("src/gleam/list.gleam", 1785).
-spec key_set_loop(list({ANS, ANT}), ANS, ANT, list({ANS, ANT})) -> list({ANS,
    ANT}).
key_set_loop(List, Key, Value, Inspected) ->
    case List of
        [{K, _} | Rest] when K =:= Key ->
            reverse_and_prepend(Inspected, [{K, Value} | Rest]);

        [First | Rest@1] ->
            key_set_loop(Rest@1, Key, Value, [First | Inspected]);

        [] ->
            lists:reverse([{Key, Value} | Inspected])
    end.

-file("src/gleam/list.gleam", 1781).
?DOC(
    " Given a list of 2-element tuples, inserts a key and value into the list.\n"
    "\n"
    " If there was already a tuple with the key then it is replaced, otherwise it\n"
    " is added to the end of the list.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " key_set([#(5, 0), #(4, 1)], 4, 100)\n"
    " // -> [#(5, 0), #(4, 100)]\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " key_set([#(5, 0), #(4, 1)], 1, 100)\n"
    " // -> [#(5, 0), #(4, 1), #(1, 100)]\n"
    " ```\n"
).
-spec key_set(list({ANO, ANP}), ANO, ANP) -> list({ANO, ANP}).
key_set(List, Key, Value) ->
    key_set_loop(List, Key, Value, []).

-file("src/gleam/list.gleam", 1813).
?DOC(
    " Calls a function for each element in a list, discarding the return value.\n"
    "\n"
    " Useful for calling a side effect for every item of a list.\n"
    "\n"
    " ```gleam\n"
    " import gleam/io\n"
    "\n"
    " each([\"1\", \"2\", \"3\"], io.println)\n"
    " // -> Nil\n"
    " // 1\n"
    " // 2\n"
    " // 3\n"
    " ```\n"
).
-spec each(list(ANX), fun((ANX) -> any())) -> nil.
each(List, F) ->
    case List of
        [] ->
            nil;

        [First | Rest] ->
            F(First),
            each(Rest, F)
    end.

-file("src/gleam/list.gleam", 1839).
?DOC(
    " Calls a `Result` returning function for each element in a list, discarding\n"
    " the return value. If the function returns `Error` then the iteration is\n"
    " stopped and the error is returned.\n"
    "\n"
    " Useful for calling a side effect for every item of a list.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " try_each(\n"
    "   over: [1, 2, 3],\n"
    "   with: function_that_might_fail,\n"
    " )\n"
    " // -> Ok(Nil)\n"
    " ```\n"
).
-spec try_each(list(AOA), fun((AOA) -> {ok, any()} | {error, AOD})) -> {ok, nil} |
    {error, AOD}.
try_each(List, Fun) ->
    case List of
        [] ->
            {ok, nil};

        [First | Rest] ->
            case Fun(First) of
                {ok, _} ->
                    try_each(Rest, Fun);

                {error, E} ->
                    {error, E}
            end
    end.

-file("src/gleam/list.gleam", 1872).
-spec partition_loop(list(BHM), fun((BHM) -> boolean()), list(BHM), list(BHM)) -> {list(BHM),
    list(BHM)}.
partition_loop(List, Categorise, Trues, Falses) ->
    case List of
        [] ->
            {lists:reverse(Trues), lists:reverse(Falses)};

        [First | Rest] ->
            case Categorise(First) of
                true ->
                    partition_loop(Rest, Categorise, [First | Trues], Falses);

                false ->
                    partition_loop(Rest, Categorise, Trues, [First | Falses])
            end
    end.

-file("src/gleam/list.gleam", 1865).
?DOC(
    " Partitions a list into a tuple/pair of lists\n"
    " by a given categorisation function.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " import gleam/int\n"
    "\n"
    " [1, 2, 3, 4, 5] |> partition(int.is_odd)\n"
    " // -> #([1, 3, 5], [2, 4])\n"
    " ```\n"
).
-spec partition(list(AOI), fun((AOI) -> boolean())) -> {list(AOI), list(AOI)}.
partition(List, Categorise) ->
    partition_loop(List, Categorise, [], []).

-file("src/gleam/list.gleam", 1892).
?DOC(
    " Returns all the permutations of a list.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " permutations([1, 2])\n"
    " // -> [[1, 2], [2, 1]]\n"
    " ```\n"
).
-spec permutations(list(AOR)) -> list(list(AOR)).
permutations(List) ->
    case List of
        [] ->
            [[]];

        [_ | _] ->
            _pipe@3 = index_map(
                List,
                fun(I, I_idx) ->
                    _pipe = index_fold(
                        List,
                        [],
                        fun(Acc, J, J_idx) -> case I_idx =:= J_idx of
                                true ->
                                    Acc;

                                false ->
                                    [J | Acc]
                            end end
                    ),
                    _pipe@1 = lists:reverse(_pipe),
                    _pipe@2 = permutations(_pipe@1),
                    map(_pipe@2, fun(Permutation) -> [I | Permutation] end)
                end
            ),
            flatten(_pipe@3)
    end.

-file("src/gleam/list.gleam", 1932).
-spec window_loop(list(list(AOZ)), list(AOZ), integer()) -> list(list(AOZ)).
window_loop(Acc, List, N) ->
    Window = take(List, N),
    case erlang:length(Window) =:= N of
        true ->
            window_loop([Window | Acc], drop(List, 1), N);

        false ->
            lists:reverse(Acc)
    end.

-file("src/gleam/list.gleam", 1925).
?DOC(
    " Returns a list of sliding windows.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " window([1,2,3,4,5], 3)\n"
    " // -> [[1, 2, 3], [2, 3, 4], [3, 4, 5]]\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " window([1, 2], 4)\n"
    " // -> []\n"
    " ```\n"
).
-spec window(list(AOV), integer()) -> list(list(AOV)).
window(List, N) ->
    case N =< 0 of
        true ->
            [];

        false ->
            window_loop([], List, N)
    end.

-file("src/gleam/list.gleam", 1955).
?DOC(
    " Returns a list of tuples containing two contiguous elements.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " window_by_2([1,2,3,4])\n"
    " // -> [#(1, 2), #(2, 3), #(3, 4)]\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " window_by_2([1])\n"
    " // -> []\n"
    " ```\n"
).
-spec window_by_2(list(APF)) -> list({APF, APF}).
window_by_2(List) ->
    zip(List, drop(List, 1)).

-file("src/gleam/list.gleam", 1968).
?DOC(
    " Drops the first elements in a given list for which the predicate function returns `True`.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " drop_while([1, 2, 3, 4], fn (x) { x < 3 })\n"
    " // -> [3, 4]\n"
    " ```\n"
).
-spec drop_while(list(API), fun((API) -> boolean())) -> list(API).
drop_while(List, Predicate) ->
    case List of
        [] ->
            [];

        [First | Rest] ->
            case Predicate(First) of
                true ->
                    drop_while(Rest, Predicate);

                false ->
                    [First | Rest]
            end
    end.

-file("src/gleam/list.gleam", 1998).
-spec take_while_loop(list(APO), fun((APO) -> boolean()), list(APO)) -> list(APO).
take_while_loop(List, Predicate, Acc) ->
    case List of
        [] ->
            lists:reverse(Acc);

        [First | Rest] ->
            case Predicate(First) of
                true ->
                    take_while_loop(Rest, Predicate, [First | Acc]);

                false ->
                    lists:reverse(Acc)
            end
    end.

-file("src/gleam/list.gleam", 1991).
?DOC(
    " Takes the first elements in a given list for which the predicate function returns `True`.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " take_while([1, 2, 3, 2, 4], fn (x) { x < 3 })\n"
    " // -> [1, 2]\n"
    " ```\n"
).
-spec take_while(list(APL), fun((APL) -> boolean())) -> list(APL).
take_while(List, Predicate) ->
    take_while_loop(List, Predicate, []).

-file("src/gleam/list.gleam", 2030).
-spec chunk_loop(list(APX), fun((APX) -> APZ), APZ, list(APX), list(list(APX))) -> list(list(APX)).
chunk_loop(List, F, Previous_key, Current_chunk, Acc) ->
    case List of
        [First | Rest] ->
            Key = F(First),
            case Key =:= Previous_key of
                true ->
                    chunk_loop(Rest, F, Key, [First | Current_chunk], Acc);

                false ->
                    New_acc = [lists:reverse(Current_chunk) | Acc],
                    chunk_loop(Rest, F, Key, [First], New_acc)
            end;

        [] ->
            lists:reverse([lists:reverse(Current_chunk) | Acc])
    end.

-file("src/gleam/list.gleam", 2023).
?DOC(
    " Returns a list of chunks in which\n"
    " the return value of calling `f` on each element is the same.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " [1, 2, 2, 3, 4, 4, 6, 7, 7] |> chunk(by: fn(n) { n % 2 })\n"
    " // -> [[1], [2, 2], [3], [4, 4, 6], [7, 7]]\n"
    " ```\n"
).
-spec chunk(list(APS), fun((APS) -> any())) -> list(list(APS)).
chunk(List, F) ->
    case List of
        [] ->
            [];

        [First | Rest] ->
            chunk_loop(Rest, F, F(First), [First], [])
    end.

-file("src/gleam/list.gleam", 2075).
-spec sized_chunk_loop(
    list(AQJ),
    integer(),
    integer(),
    list(AQJ),
    list(list(AQJ))
) -> list(list(AQJ)).
sized_chunk_loop(List, Count, Left, Current_chunk, Acc) ->
    case List of
        [] ->
            case Current_chunk of
                [] ->
                    lists:reverse(Acc);

                Remaining ->
                    lists:reverse([lists:reverse(Remaining) | Acc])
            end;

        [First | Rest] ->
            Chunk = [First | Current_chunk],
            case Left > 1 of
                true ->
                    sized_chunk_loop(Rest, Count, Left - 1, Chunk, Acc);

                false ->
                    sized_chunk_loop(
                        Rest,
                        Count,
                        Count,
                        [],
                        [lists:reverse(Chunk) | Acc]
                    )
            end
    end.

-file("src/gleam/list.gleam", 2071).
?DOC(
    " Returns a list of chunks containing `count` elements each.\n"
    "\n"
    " If the last chunk does not have `count` elements, it is instead\n"
    " a partial chunk, with less than `count` elements.\n"
    "\n"
    " For any `count` less than 1 this function behaves as if it was set to 1.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " [1, 2, 3, 4, 5, 6] |> sized_chunk(into: 2)\n"
    " // -> [[1, 2], [3, 4], [5, 6]]\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " [1, 2, 3, 4, 5, 6, 7, 8] |> sized_chunk(into: 3)\n"
    " // -> [[1, 2, 3], [4, 5, 6], [7, 8]]\n"
    " ```\n"
).
-spec sized_chunk(list(AQF), integer()) -> list(list(AQF)).
sized_chunk(List, Count) ->
    sized_chunk_loop(List, Count, Count, [], []).

-file("src/gleam/list.gleam", 2119).
?DOC(
    " This function acts similar to fold, but does not take an initial state.\n"
    " Instead, it starts from the first element in the list\n"
    " and combines it with each subsequent element in turn using the given\n"
    " function. The function is called as `fun(accumulator, current_element)`.\n"
    "\n"
    " Returns `Ok` to indicate a successful run, and `Error` if called on an\n"
    " empty list.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " [] |> reduce(fn(acc, x) { acc + x })\n"
    " // -> Error(Nil)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " [1, 2, 3, 4, 5] |> reduce(fn(acc, x) { acc + x })\n"
    " // -> Ok(15)\n"
    " ```\n"
).
-spec reduce(list(AQQ), fun((AQQ, AQQ) -> AQQ)) -> {ok, AQQ} | {error, nil}.
reduce(List, Fun) ->
    case List of
        [] ->
            {error, nil};

        [First | Rest] ->
            {ok, fold(Rest, First, Fun)}
    end.

-file("src/gleam/list.gleam", 2143).
-spec scan_loop(list(AQY), ARA, list(ARA), fun((ARA, AQY) -> ARA)) -> list(ARA).
scan_loop(List, Accumulator, Accumulated, Fun) ->
    case List of
        [] ->
            lists:reverse(Accumulated);

        [First | Rest] ->
            Next = Fun(Accumulator, First),
            scan_loop(Rest, Next, [Next | Accumulated], Fun)
    end.

-file("src/gleam/list.gleam", 2135).
?DOC(
    " Similar to `fold`, but yields the state of the accumulator at each stage.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " scan(over: [1, 2, 3], from: 100, with: fn(acc, i) { acc + i })\n"
    " // -> [101, 103, 106]\n"
    " ```\n"
).
-spec scan(list(AQU), AQW, fun((AQW, AQU) -> AQW)) -> list(AQW).
scan(List, Initial, Fun) ->
    scan_loop(List, Initial, [], Fun).

-file("src/gleam/list.gleam", 2176).
?DOC(
    " Returns the last element in the given list.\n"
    "\n"
    " Returns `Error(Nil)` if the list is empty.\n"
    "\n"
    " This function runs in linear time.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " last([])\n"
    " // -> Error(Nil)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " last([1, 2, 3, 4, 5])\n"
    " // -> Ok(5)\n"
    " ```\n"
).
-spec last(list(ARD)) -> {ok, ARD} | {error, nil}.
last(List) ->
    reduce(List, fun(_, Elem) -> Elem end).

-file("src/gleam/list.gleam", 2194).
?DOC(
    " Return unique combinations of elements in the list.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " combinations([1, 2, 3], 2)\n"
    " // -> [[1, 2], [1, 3], [2, 3]]\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " combinations([1, 2, 3, 4], 3)\n"
    " // -> [[1, 2, 3], [1, 2, 4], [1, 3, 4], [2, 3, 4]]\n"
    " ```\n"
).
-spec combinations(list(ARH), integer()) -> list(list(ARH)).
combinations(Items, N) ->
    case N of
        0 ->
            [[]];

        _ ->
            case Items of
                [] ->
                    [];

                [First | Rest] ->
                    First_combinations = begin
                        _pipe = map(
                            combinations(Rest, N - 1),
                            fun(Com) -> [First | Com] end
                        ),
                        lists:reverse(_pipe)
                    end,
                    fold(
                        First_combinations,
                        combinations(Rest, N),
                        fun(Acc, C) -> [C | Acc] end
                    )
            end
    end.

-file("src/gleam/list.gleam", 2226).
-spec combination_pairs_loop(list(ARO)) -> list(list({ARO, ARO})).
combination_pairs_loop(Items) ->
    case Items of
        [] ->
            [];

        [First | Rest] ->
            First_combinations = map(Rest, fun(Other) -> {First, Other} end),
            [First_combinations | combination_pairs_loop(Rest)]
    end.

-file("src/gleam/list.gleam", 2221).
?DOC(
    " Return unique pair combinations of elements in the list\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " combination_pairs([1, 2, 3])\n"
    " // -> [#(1, 2), #(1, 3), #(2, 3)]\n"
    " ```\n"
).
-spec combination_pairs(list(ARL)) -> list({ARL, ARL}).
combination_pairs(Items) ->
    _pipe = combination_pairs_loop(Items),
    flatten(_pipe).

-file("src/gleam/list.gleam", 2263).
?DOC(
    " Transpose rows and columns of the list of lists.\n"
    "\n"
    " Notice: This function is not tail recursive,\n"
    " and thus may exceed stack size if called,\n"
    " with large lists (on target JavaScript).\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " transpose([[1, 2, 3], [101, 102, 103]])\n"
    " // -> [[1, 101], [2, 102], [3, 103]]\n"
    " ```\n"
).
-spec transpose(list(list(ARW))) -> list(list(ARW)).
transpose(List_of_list) ->
    Take_first = fun(List) -> case List of
            [] ->
                [];

            [First] ->
                [First];

            [First@1 | _] ->
                [First@1]
        end end,
    case List_of_list of
        [] ->
            [];

        [[] | Rest] ->
            transpose(Rest);

        Rows ->
            Firsts = begin
                _pipe = Rows,
                _pipe@1 = map(_pipe, Take_first),
                flatten(_pipe@1)
            end,
            Rest@1 = transpose(
                map(Rows, fun(_capture) -> drop(_capture, 1) end)
            ),
            [Firsts | Rest@1]
    end.

-file("src/gleam/list.gleam", 2245).
?DOC(
    " Make a list alternating the elements from the given lists\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " interleave([[1, 2], [101, 102], [201, 202]])\n"
    " // -> [1, 101, 201, 2, 102, 202]\n"
    " ```\n"
).
-spec interleave(list(list(ARS))) -> list(ARS).
interleave(List) ->
    _pipe = transpose(List),
    flatten(_pipe).

-file("src/gleam/list.gleam", 2304).
-spec shuffle_pair_unwrap_loop(list({float(), ASE}), list(ASE)) -> list(ASE).
shuffle_pair_unwrap_loop(List, Acc) ->
    case List of
        [] ->
            Acc;

        [Elem_pair | Enumerable] ->
            shuffle_pair_unwrap_loop(
                Enumerable,
                [erlang:element(2, Elem_pair) | Acc]
            )
    end.

-file("src/gleam/list.gleam", 2312).
-spec do_shuffle_by_pair_indexes(list({float(), ASI})) -> list({float(), ASI}).
do_shuffle_by_pair_indexes(List_of_pairs) ->
    sort(
        List_of_pairs,
        fun(A_pair, B_pair) ->
            gleam@float:compare(
                erlang:element(1, A_pair),
                erlang:element(1, B_pair)
            )
        end
    ).

-file("src/gleam/list.gleam", 2297).
?DOC(
    " Takes a list, randomly sorts all items and returns the shuffled list.\n"
    "\n"
    " This function uses `float.random` to decide the order of the elements.\n"
    "\n"
    " ## Example\n"
    "\n"
    " ```gleam\n"
    " range(1, 10) |> shuffle()\n"
    " // -> [1, 6, 9, 10, 3, 8, 4, 2, 7, 5]\n"
    " ```\n"
).
-spec shuffle(list(ASB)) -> list(ASB).
shuffle(List) ->
    _pipe = List,
    _pipe@1 = fold(_pipe, [], fun(Acc, A) -> [{rand:uniform(), A} | Acc] end),
    _pipe@2 = do_shuffle_by_pair_indexes(_pipe@1),
    shuffle_pair_unwrap_loop(_pipe@2, []).

-file("src/gleam/list.gleam", 2334).
?DOC(
    " Takes a list and a comparator, and returns the maximum element in the list\n"
    "\n"
    "\n"
    " ## Example\n"
    "\n"
    " ```gleam\n"
    " range(1, 10) |> list.max(int.compare)\n"
    " // -> Ok(10)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " [\"a\", \"c\", \"b\"] |> list.max(string.compare)\n"
    " // -> Ok(\"c\")\n"
    " ```\n"
).
-spec max(list(ASL), fun((ASL, ASL) -> gleam@order:order())) -> {ok, ASL} |
    {error, nil}.
max(List, Compare) ->
    reduce(List, fun(Acc, Other) -> case Compare(Acc, Other) of
                gt ->
                    Acc;

                lt ->
                    Other;

                eq ->
                    Other
            end end).

-file("src/gleam/list.gleam", 2405).
-spec log_random() -> float().
log_random() ->
    Min_positive = 2.2250738585072014e-308,
    _assert_subject = gleam@float:logarithm(rand:uniform() + Min_positive),
    {ok, Random} = case _assert_subject of
        {ok, _} -> _assert_subject;
        _assert_fail ->
            erlang:error(#{gleam_error => let_assert,
                        message => <<"Pattern match failed, no pattern matched the value."/utf8>>,
                        value => _assert_fail,
                        module => <<"gleam/list"/utf8>>,
                        function => <<"log_random"/utf8>>,
                        line => 2407})
    end,
    Random.

-file("src/gleam/list.gleam", 2381).
-spec sample_loop(
    list(AST),
    gleam@dict:dict(integer(), AST),
    integer(),
    integer(),
    float()
) -> gleam@dict:dict(integer(), AST).
sample_loop(List, Reservoir, K, Index, W) ->
    Skip = begin
        _assert_subject = gleam@float:logarithm(1.0 - W),
        {ok, Log_result} = case _assert_subject of
            {ok, _} -> _assert_subject;
            _assert_fail ->
                erlang:error(#{gleam_error => let_assert,
                            message => <<"Pattern match failed, no pattern matched the value."/utf8>>,
                            value => _assert_fail,
                            module => <<"gleam/list"/utf8>>,
                            function => <<"sample_loop"/utf8>>,
                            line => 2389})
        end,
        _pipe = case Log_result of
            +0.0 -> +0.0;
            -0.0 -> -0.0;
            Gleam@denominator -> log_random() / Gleam@denominator
        end,
        _pipe@1 = math:floor(_pipe),
        erlang:round(_pipe@1)
    end,
    Index@1 = (Index + Skip) + 1,
    case drop(List, Skip) of
        [] ->
            Reservoir;

        [First | Rest] ->
            Reservoir@1 = gleam@dict:insert(
                Reservoir,
                gleam@int:random(K),
                First
            ),
            W@1 = W * math:exp(case erlang:float(K) of
                    +0.0 -> +0.0;
                    -0.0 -> -0.0;
                    Gleam@denominator@1 -> log_random() / Gleam@denominator@1
                end),
            sample_loop(Rest, Reservoir@1, K, Index@1, W@1)
    end.

-file("src/gleam/list.gleam", 2358).
?DOC(
    " Take a random sample of k elements from a list using reservoir sampling via\n"
    " Algo L. Returns an empty list if the sample size is less than or equal to 0.\n"
    "\n"
    " Order is not random, only selection is.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " reservoir_sample([1, 2, 3, 4, 5], 3)\n"
    " // -> [2, 4, 5]  // A random sample of 3 items\n"
    " ```\n"
).
-spec sample(list(ASP), integer()) -> list(ASP).
sample(List, K) ->
    case K =< 0 of
        true ->
            [];

        false ->
            {Reservoir, List@1} = split(List, K),
            case erlang:length(Reservoir) < K of
                true ->
                    Reservoir;

                false ->
                    Reservoir@1 = begin
                        _pipe = Reservoir,
                        _pipe@1 = map2(
                            range(0, K - 1),
                            _pipe,
                            fun(A, B) -> {A, B} end
                        ),
                        maps:from_list(_pipe@1)
                    end,
                    W = math:exp(case erlang:float(K) of
                            +0.0 -> +0.0;
                            -0.0 -> -0.0;
                            Gleam@denominator -> log_random() / Gleam@denominator
                        end),
                    _pipe@2 = sample_loop(List@1, Reservoir@1, K, K, W),
                    maps:values(_pipe@2)
            end
    end.
