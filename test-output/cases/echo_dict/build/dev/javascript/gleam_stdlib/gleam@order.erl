-module(gleam@order).
-compile([no_auto_import, nowarn_unused_vars, nowarn_unused_function, nowarn_nomatch]).

-export([negate/1, to_int/1, compare/2, reverse/1, break_tie/2, lazy_break_tie/2]).
-export_type([order/0]).

-if(?OTP_RELEASE >= 27).
-define(MODULEDOC(Str), -moduledoc(Str)).
-define(DOC(Str), -doc(Str)).
-else.
-define(MODULEDOC(Str), -compile([])).
-define(DOC(Str), -compile([])).
-endif.

-type order() :: lt | eq | gt.

-file("src/gleam/order.gleam", 35).
?DOC(
    " Inverts an order, so less-than becomes greater-than and greater-than\n"
    " becomes less-than.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " negate(Lt)\n"
    " // -> Gt\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " negate(Eq)\n"
    " // -> Eq\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " negate(Gt)\n"
    " // -> Lt\n"
    " ```\n"
).
-spec negate(order()) -> order().
negate(Order) ->
    case Order of
        lt ->
            gt;

        eq ->
            eq;

        gt ->
            lt
    end.

-file("src/gleam/order.gleam", 62).
?DOC(
    " Produces a numeric representation of the order.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " to_int(Lt)\n"
    " // -> -1\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " to_int(Eq)\n"
    " // -> 0\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " to_int(Gt)\n"
    " // -> 1\n"
    " ```\n"
).
-spec to_int(order()) -> integer().
to_int(Order) ->
    case Order of
        lt ->
            -1;

        eq ->
            0;

        gt ->
            1
    end.

-file("src/gleam/order.gleam", 79).
?DOC(
    " Compares two `Order` values to one another, producing a new `Order`.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " compare(Eq, with: Lt)\n"
    " // -> Gt\n"
    " ```\n"
).
-spec compare(order(), order()) -> order().
compare(A, B) ->
    case {A, B} of
        {X, Y} when X =:= Y ->
            eq;

        {lt, _} ->
            lt;

        {eq, gt} ->
            lt;

        {_, _} ->
            gt
    end.

-file("src/gleam/order.gleam", 100).
?DOC(
    " Inverts an ordering function, so less-than becomes greater-than and greater-than\n"
    " becomes less-than.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " import gleam/int\n"
    " import gleam/list\n"
    "\n"
    " list.sort([1, 5, 4], by: reverse(int.compare))\n"
    " // -> [5, 4, 1]\n"
    " ```\n"
).
-spec reverse(fun((I, I) -> order())) -> fun((I, I) -> order()).
reverse(Orderer) ->
    fun(A, B) -> Orderer(B, A) end.

-file("src/gleam/order.gleam", 122).
?DOC(
    " Return a fallback `Order` in case the first argument is `Eq`.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " import gleam/int\n"
    "\n"
    " break_tie(in: int.compare(1, 1), with: Lt)\n"
    " // -> Lt\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " import gleam/int\n"
    "\n"
    " break_tie(in: int.compare(1, 0), with: Eq)\n"
    " // -> Gt\n"
    " ```\n"
).
-spec break_tie(order(), order()) -> order().
break_tie(Order, Other) ->
    case Order of
        lt ->
            Order;

        gt ->
            Order;

        eq ->
            Other
    end.

-file("src/gleam/order.gleam", 151).
?DOC(
    " Invokes a fallback function returning an `Order` in case the first argument\n"
    " is `Eq`.\n"
    "\n"
    " This can be useful when the fallback comparison might be expensive and it\n"
    " needs to be delayed until strictly necessary.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " import gleam/int\n"
    "\n"
    " lazy_break_tie(in: int.compare(1, 1), with: fn() { Lt })\n"
    " // -> Lt\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " import gleam/int\n"
    "\n"
    " lazy_break_tie(in: int.compare(1, 0), with: fn() { Eq })\n"
    " // -> Gt\n"
    " ```\n"
).
-spec lazy_break_tie(order(), fun(() -> order())) -> order().
lazy_break_tie(Order, Comparison) ->
    case Order of
        lt ->
            Order;

        gt ->
            Order;

        eq ->
            Comparison()
    end.
