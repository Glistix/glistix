-module(gleam@set).
-compile([no_auto_import, nowarn_unused_vars, nowarn_unused_function, nowarn_nomatch]).

-export([new/0, size/1, is_empty/1, contains/2, delete/2, to_list/1, fold/3, filter/2, drop/2, take/2, intersection/2, difference/2, is_subset/2, is_disjoint/2, each/2, insert/2, from_list/1, map/2, union/2, symmetric_difference/2]).
-export_type([set/1]).

-if(?OTP_RELEASE >= 27).
-define(MODULEDOC(Str), -moduledoc(Str)).
-define(DOC(Str), -doc(Str)).
-else.
-define(MODULEDOC(Str), -compile([])).
-define(DOC(Str), -compile([])).
-endif.

-opaque set(EGC) :: {set, gleam@dict:dict(EGC, list(nil))}.

-file("src/gleam/set.gleam", 32).
?DOC(" Creates a new empty set.\n").
-spec new() -> set(any()).
new() ->
    {set, maps:new()}.

-file("src/gleam/set.gleam", 50).
?DOC(
    " Gets the number of members in a set.\n"
    "\n"
    " This function runs in constant time.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " new()\n"
    " |> insert(1)\n"
    " |> insert(2)\n"
    " |> size\n"
    " // -> 2\n"
    " ```\n"
).
-spec size(set(any())) -> integer().
size(Set) ->
    maps:size(erlang:element(2, Set)).

-file("src/gleam/set.gleam", 68).
?DOC(
    " Determines whether or not the set is empty.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " new() |> is_empty\n"
    " // -> True\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " new() |> insert(1) |> is_empty\n"
    " // -> False\n"
    " ```\n"
).
-spec is_empty(set(any())) -> boolean().
is_empty(Set) ->
    Set =:= new().

-file("src/gleam/set.gleam", 110).
?DOC(
    " Checks whether a set contains a given member.\n"
    "\n"
    " This function runs in logarithmic time.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " new()\n"
    " |> insert(2)\n"
    " |> contains(2)\n"
    " // -> True\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " new()\n"
    " |> insert(2)\n"
    " |> contains(1)\n"
    " // -> False\n"
    " ```\n"
).
-spec contains(set(EGN), EGN) -> boolean().
contains(Set, Member) ->
    _pipe = erlang:element(2, Set),
    _pipe@1 = gleam_stdlib:map_get(_pipe, Member),
    gleam@result:is_ok(_pipe@1).

-file("src/gleam/set.gleam", 131).
?DOC(
    " Removes a member from a set. If the set does not contain the member then\n"
    " the set is returned unchanged.\n"
    "\n"
    " This function runs in logarithmic time.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " new()\n"
    " |> insert(2)\n"
    " |> delete(2)\n"
    " |> contains(1)\n"
    " // -> False\n"
    " ```\n"
).
-spec delete(set(EGP), EGP) -> set(EGP).
delete(Set, Member) ->
    {set, gleam@dict:delete(erlang:element(2, Set), Member)}.

-file("src/gleam/set.gleam", 149).
?DOC(
    " Converts the set into a list of the contained members.\n"
    "\n"
    " The list has no specific ordering, any unintentional ordering may change in\n"
    " future versions of Gleam or Erlang.\n"
    "\n"
    " This function runs in linear time.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " new() |> insert(2) |> to_list\n"
    " // -> [2]\n"
    " ```\n"
).
-spec to_list(set(EGS)) -> list(EGS).
to_list(Set) ->
    maps:keys(erlang:element(2, Set)).

-file("src/gleam/set.gleam", 190).
?DOC(
    " Combines all entries into a single value by calling a given function on each\n"
    " one.\n"
    "\n"
    " Sets are not ordered so the values are not returned in any specific order.\n"
    " Do not write code that relies on the order entries are used by this\n"
    " function as it may change in later versions of Gleam or Erlang.\n"
    "\n"
    " # Examples\n"
    "\n"
    " ```gleam\n"
    " from_list([1, 3, 9])\n"
    " |> fold(0, fn(accumulator, member) { accumulator + member })\n"
    " // -> 13\n"
    " ```\n"
).
-spec fold(set(EGY), EHA, fun((EHA, EGY) -> EHA)) -> EHA.
fold(Set, Initial, Reducer) ->
    gleam@dict:fold(
        erlang:element(2, Set),
        Initial,
        fun(A, K, _) -> Reducer(A, K) end
    ).

-file("src/gleam/set.gleam", 214).
?DOC(
    " Creates a new set from an existing set, minus any members that a given\n"
    " function returns `False` for.\n"
    "\n"
    " This function runs in loglinear time.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " import gleam/int\n"
    "\n"
    " from_list([1, 4, 6, 3, 675, 44, 67])\n"
    " |> filter(keeping: int.is_even)\n"
    " |> to_list\n"
    " // -> [4, 6, 44]\n"
    " ```\n"
).
-spec filter(set(EHB), fun((EHB) -> boolean())) -> set(EHB).
filter(Set, Predicate) ->
    {set,
        gleam@dict:filter(erlang:element(2, Set), fun(M, _) -> Predicate(M) end)}.

-file("src/gleam/set.gleam", 249).
?DOC(
    " Creates a new set from a given set with all the same entries except any\n"
    " entry found on the given list.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " from_list([1, 2, 3, 4])\n"
    " |> drop([1, 3])\n"
    " |> to_list\n"
    " // -> [2, 4]\n"
    " ```\n"
).
-spec drop(set(EHI), list(EHI)) -> set(EHI).
drop(Set, Disallowed) ->
    gleam@list:fold(Disallowed, Set, fun delete/2).

-file("src/gleam/set.gleam", 267).
?DOC(
    " Creates a new set from a given set, only including any members which are in\n"
    " a given list.\n"
    "\n"
    " This function runs in loglinear time.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " from_list([1, 2, 3])\n"
    " |> take([1, 3, 5])\n"
    " |> to_list\n"
    " // -> [1, 3]\n"
    " ```\n"
).
-spec take(set(EHM), list(EHM)) -> set(EHM).
take(Set, Desired) ->
    {set, gleam@dict:take(erlang:element(2, Set), Desired)}.

-file("src/gleam/set.gleam", 287).
-spec order(set(EHU), set(EHU)) -> {set(EHU), set(EHU)}.
order(First, Second) ->
    case maps:size(erlang:element(2, First)) > maps:size(
        erlang:element(2, Second)
    ) of
        true ->
            {First, Second};

        false ->
            {Second, First}
    end.

-file("src/gleam/set.gleam", 305).
?DOC(
    " Creates a new set that contains members that are present in both given sets.\n"
    "\n"
    " This function runs in loglinear time.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " intersection(from_list([1, 2]), from_list([2, 3])) |> to_list\n"
    " // -> [2]\n"
    " ```\n"
).
-spec intersection(set(EHZ), set(EHZ)) -> set(EHZ).
intersection(First, Second) ->
    {Larger, Smaller} = order(First, Second),
    take(Larger, to_list(Smaller)).

-file("src/gleam/set.gleam", 323).
?DOC(
    " Creates a new set that contains members that are present in the first set\n"
    " but not the second.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " difference(from_list([1, 2]), from_list([2, 3, 4])) |> to_list\n"
    " // -> [1]\n"
    " ```\n"
).
-spec difference(set(EID), set(EID)) -> set(EID).
difference(First, Second) ->
    drop(First, to_list(Second)).

-file("src/gleam/set.gleam", 344).
?DOC(
    " Determines if a set is fully contained by another.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " is_subset(from_list([1]), from_list([1, 2]))\n"
    " // -> True\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " is_subset(from_list([1, 2, 3]), from_list([3, 4, 5]))\n"
    " // -> False\n"
    " ```\n"
).
-spec is_subset(set(EIH), set(EIH)) -> boolean().
is_subset(First, Second) ->
    intersection(First, Second) =:= First.

-file("src/gleam/set.gleam", 362).
?DOC(
    " Determines if two sets contain no common members\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " is_disjoint(from_list([1, 2, 3]), from_list([4, 5, 6]))\n"
    " // -> True\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " is_disjoint(from_list([1, 2, 3]), from_list([3, 4, 5]))\n"
    " // -> False\n"
    " ```\n"
).
-spec is_disjoint(set(EIK), set(EIK)) -> boolean().
is_disjoint(First, Second) ->
    intersection(First, Second) =:= new().

-file("src/gleam/set.gleam", 402).
?DOC(
    " Calls a function for each member in a set, discarding the return\n"
    " value.\n"
    "\n"
    " Useful for producing a side effect for every item of a set.\n"
    "\n"
    " ```gleam\n"
    " let set = from_list([\"apple\", \"banana\", \"cherry\"])\n"
    "\n"
    " each(set, io.println)\n"
    " // -> Nil\n"
    " // apple\n"
    " // banana\n"
    " // cherry\n"
    " ```\n"
    "\n"
    " The order of elements in the iteration is an implementation detail that\n"
    " should not be relied upon.\n"
).
-spec each(set(EIR), fun((EIR) -> any())) -> nil.
each(Set, Fun) ->
    fold(
        Set,
        nil,
        fun(Nil, Member) ->
            Fun(Member),
            Nil
        end
    ).

-file("src/gleam/set.gleam", 86).
?DOC(
    " Inserts an member into the set.\n"
    "\n"
    " This function runs in logarithmic time.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " new()\n"
    " |> insert(1)\n"
    " |> insert(2)\n"
    " |> size\n"
    " // -> 2\n"
    " ```\n"
).
-spec insert(set(EGK), EGK) -> set(EGK).
insert(Set, Member) ->
    {set, gleam@dict:insert(erlang:element(2, Set), Member, [])}.

-file("src/gleam/set.gleam", 167).
?DOC(
    " Creates a new set of the members in a given list.\n"
    "\n"
    " This function runs in loglinear time.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " import gleam/int\n"
    " import gleam/list\n"
    "\n"
    " [1, 1, 2, 4, 3, 2] |> from_list |> to_list |> list.sort(by: int.compare)\n"
    " // -> [1, 2, 3, 4]\n"
    " ```\n"
).
-spec from_list(list(EGV)) -> set(EGV).
from_list(Members) ->
    Dict = gleam@list:fold(
        Members,
        maps:new(),
        fun(M, K) -> gleam@dict:insert(M, K, []) end
    ),
    {set, Dict}.

-file("src/gleam/set.gleam", 232).
?DOC(
    " Creates a new set from a given set with the result of applying the given\n"
    " function to each member.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " from_list([1, 2, 3, 4])\n"
    " |> map(with: fn(x) { x * 2 })\n"
    " |> to_list\n"
    " // -> [2, 4, 6, 8]\n"
    " ```\n"
).
-spec map(set(EHE), fun((EHE) -> EHG)) -> set(EHG).
map(Set, Fun) ->
    fold(Set, new(), fun(Acc, Member) -> insert(Acc, Fun(Member)) end).

-file("src/gleam/set.gleam", 282).
?DOC(
    " Creates a new set that contains all members of both given sets.\n"
    "\n"
    " This function runs in loglinear time.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " union(from_list([1, 2]), from_list([2, 3])) |> to_list\n"
    " // -> [1, 2, 3]\n"
    " ```\n"
).
-spec union(set(EHQ), set(EHQ)) -> set(EHQ).
union(First, Second) ->
    {Larger, Smaller} = order(First, Second),
    fold(Smaller, Larger, fun insert/2).

-file("src/gleam/set.gleam", 374).
?DOC(
    " Creates a new set that contains members that are present in either set, but\n"
    " not both.\n"
    "\n"
    " ```gleam\n"
    " symmetric_difference(from_list([1, 2, 3]), from_list([3, 4])) |> to_list\n"
    " // -> [1, 2, 4]\n"
    " ```\n"
).
-spec symmetric_difference(set(EIN), set(EIN)) -> set(EIN).
symmetric_difference(First, Second) ->
    difference(union(First, Second), intersection(First, Second)).
