-module(gleam@int).
-compile([no_auto_import, nowarn_unused_vars, nowarn_unused_function, nowarn_nomatch]).

-export([absolute_value/1, parse/1, base_parse/2, to_string/1, to_base_string/2, to_base2/1, to_base8/1, to_base16/1, to_base36/1, to_float/1, power/2, square_root/1, compare/2, min/2, max/2, clamp/3, is_even/1, is_odd/1, negate/1, sum/1, product/1, digits/2, undigits/2, random/1, divide/2, remainder/2, modulo/2, floor_divide/2, add/2, multiply/2, subtract/2, bitwise_and/2, bitwise_not/1, bitwise_or/2, bitwise_exclusive_or/2, bitwise_shift_left/2, bitwise_shift_right/2]).

-if(?OTP_RELEASE >= 27).
-define(MODULEDOC(Str), -moduledoc(Str)).
-define(DOC(Str), -doc(Str)).
-else.
-define(MODULEDOC(Str), -compile([])).
-define(DOC(Str), -compile([])).
-endif.

?MODULEDOC(
    " Functions for working with integers.\n"
    "\n"
    " ## Division by zero\n"
    "\n"
    " In Erlang division by zero results in a crash, however Gleam does not have\n"
    " partial functions and operators in core so instead division by zero returns\n"
    " zero, a behaviour taken from Pony, Coq, and Lean.\n"
    "\n"
    " This may seem unexpected at first, but it is no less mathematically valid\n"
    " than crashing or returning a special value. Division by zero is undefined\n"
    " in mathematics.\n"
).

-file("src/gleam/int.gleam", 30).
?DOC(
    " Returns the absolute value of the input.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " absolute_value(-12)\n"
    " // -> 12\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " absolute_value(10)\n"
    " // -> 10\n"
    " ```\n"
).
-spec absolute_value(integer()) -> integer().
absolute_value(X) ->
    case X >= 0 of
        true ->
            X;

        false ->
            X * -1
    end.

-file("src/gleam/int.gleam", 107).
?DOC(
    " Parses a given string as an int if possible.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " parse(\"2\")\n"
    " // -> Ok(2)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " parse(\"ABC\")\n"
    " // -> Error(Nil)\n"
    " ```\n"
).
-spec parse(binary()) -> {ok, integer()} | {error, nil}.
parse(String) ->
    gleam_stdlib:parse_int(String).

-file("src/gleam/int.gleam", 139).
?DOC(
    " Parses a given string as an int in a given base if possible.\n"
    " Supports only bases 2 to 36, for values outside of which this function returns an `Error(Nil)`.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " base_parse(\"10\", 2)\n"
    " // -> Ok(2)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " base_parse(\"30\", 16)\n"
    " // -> Ok(48)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " base_parse(\"1C\", 36)\n"
    " // -> Ok(48)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " base_parse(\"48\", 1)\n"
    " // -> Error(Nil)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " base_parse(\"48\", 37)\n"
    " // -> Error(Nil)\n"
    " ```\n"
).
-spec base_parse(binary(), integer()) -> {ok, integer()} | {error, nil}.
base_parse(String, Base) ->
    case (Base >= 2) andalso (Base =< 36) of
        true ->
            gleam_stdlib:int_from_base_string(String, Base);

        false ->
            {error, nil}
    end.

-file("src/gleam/int.gleam", 161).
?DOC(
    " Prints a given int to a string.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " to_string(2)\n"
    " // -> \"2\"\n"
    " ```\n"
).
-spec to_string(integer()) -> binary().
to_string(X) ->
    erlang:integer_to_binary(X).

-file("src/gleam/int.gleam", 194).
?DOC(
    " Prints a given int to a string using the base number provided.\n"
    " Supports only bases 2 to 36, for values outside of which this function returns an `Error(Nil)`.\n"
    " For common bases (2, 8, 16, 36), use the `to_baseN` functions.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " to_base_string(2, 2)\n"
    " // -> Ok(\"10\")\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " to_base_string(48, 16)\n"
    " // -> Ok(\"30\")\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " to_base_string(48, 36)\n"
    " // -> Ok(\"1C\")\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " to_base_string(48, 1)\n"
    " // -> Error(Nil)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " to_base_string(48, 37)\n"
    " // -> Error(Nil)\n"
    " ```\n"
).
-spec to_base_string(integer(), integer()) -> {ok, binary()} | {error, nil}.
to_base_string(X, Base) ->
    case (Base >= 2) andalso (Base =< 36) of
        true ->
            {ok, erlang:integer_to_binary(X, Base)};

        false ->
            {error, nil}
    end.

-file("src/gleam/int.gleam", 214).
?DOC(
    " Prints a given int to a string using base-2.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " to_base2(2)\n"
    " // -> \"10\"\n"
    " ```\n"
).
-spec to_base2(integer()) -> binary().
to_base2(X) ->
    erlang:integer_to_binary(X, 2).

-file("src/gleam/int.gleam", 227).
?DOC(
    " Prints a given int to a string using base-8.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " to_base8(15)\n"
    " // -> \"17\"\n"
    " ```\n"
).
-spec to_base8(integer()) -> binary().
to_base8(X) ->
    erlang:integer_to_binary(X, 8).

-file("src/gleam/int.gleam", 240).
?DOC(
    " Prints a given int to a string using base-16.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " to_base16(48)\n"
    " // -> \"30\"\n"
    " ```\n"
).
-spec to_base16(integer()) -> binary().
to_base16(X) ->
    erlang:integer_to_binary(X, 16).

-file("src/gleam/int.gleam", 253).
?DOC(
    " Prints a given int to a string using base-36.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " to_base36(48)\n"
    " // -> \"1C\"\n"
    " ```\n"
).
-spec to_base36(integer()) -> binary().
to_base36(X) ->
    erlang:integer_to_binary(X, 36).

-file("src/gleam/int.gleam", 278).
?DOC(
    " Takes an int and returns its value as a float.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " to_float(5)\n"
    " // -> 5.0\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " to_float(0)\n"
    " // -> 0.0\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " to_float(-3)\n"
    " // -> -3.0\n"
    " ```\n"
).
-spec to_float(integer()) -> float().
to_float(X) ->
    erlang:float(X).

-file("src/gleam/int.gleam", 67).
?DOC(
    " Returns the results of the base being raised to the power of the\n"
    " exponent, as a `Float`.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " power(2, -1.0)\n"
    " // -> Ok(0.5)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " power(2, 2.0)\n"
    " // -> Ok(4.0)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " power(8, 1.5)\n"
    " // -> Ok(22.627416997969522)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " 4 |> power(of: 2.0)\n"
    " // -> Ok(16.0)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " power(-1, 0.5)\n"
    " // -> Error(Nil)\n"
    " ```\n"
).
-spec power(integer(), float()) -> {ok, float()} | {error, nil}.
power(Base, Exponent) ->
    _pipe = erlang:float(Base),
    gleam@float:power(_pipe, Exponent).

-file("src/gleam/int.gleam", 86).
?DOC(
    " Returns the square root of the input as a `Float`.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " square_root(4)\n"
    " // -> Ok(2.0)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " square_root(-16)\n"
    " // -> Error(Nil)\n"
    " ```\n"
).
-spec square_root(integer()) -> {ok, float()} | {error, nil}.
square_root(X) ->
    _pipe = erlang:float(X),
    gleam@float:square_root(_pipe).

-file("src/gleam/int.gleam", 314).
?DOC(
    " Compares two ints, returning an order.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " compare(2, 3)\n"
    " // -> Lt\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " compare(4, 3)\n"
    " // -> Gt\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " compare(3, 3)\n"
    " // -> Eq\n"
    " ```\n"
).
-spec compare(integer(), integer()) -> gleam@order:order().
compare(A, B) ->
    case A =:= B of
        true ->
            eq;

        false ->
            case A < B of
                true ->
                    lt;

                false ->
                    gt
            end
    end.

-file("src/gleam/int.gleam", 334).
?DOC(
    " Compares two ints, returning the smaller of the two.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " min(2, 3)\n"
    " // -> 2\n"
    " ```\n"
).
-spec min(integer(), integer()) -> integer().
min(A, B) ->
    case A < B of
        true ->
            A;

        false ->
            B
    end.

-file("src/gleam/int.gleam", 350).
?DOC(
    " Compares two ints, returning the larger of the two.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " max(2, 3)\n"
    " // -> 3\n"
    " ```\n"
).
-spec max(integer(), integer()) -> integer().
max(A, B) ->
    case A > B of
        true ->
            A;

        false ->
            B
    end.

-file("src/gleam/int.gleam", 289).
?DOC(
    " Restricts an int between a lower and upper bound.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " clamp(40, min: 50, max: 60)\n"
    " // -> 50\n"
    " ```\n"
).
-spec clamp(integer(), integer(), integer()) -> integer().
clamp(X, Min_bound, Max_bound) ->
    _pipe = X,
    _pipe@1 = min(_pipe, Max_bound),
    max(_pipe@1, Min_bound).

-file("src/gleam/int.gleam", 371).
?DOC(
    " Returns whether the value provided is even.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " is_even(2)\n"
    " // -> True\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " is_even(3)\n"
    " // -> False\n"
    " ```\n"
).
-spec is_even(integer()) -> boolean().
is_even(X) ->
    (X rem 2) =:= 0.

-file("src/gleam/int.gleam", 389).
?DOC(
    " Returns whether the value provided is odd.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " is_odd(3)\n"
    " // -> True\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " is_odd(2)\n"
    " // -> False\n"
    " ```\n"
).
-spec is_odd(integer()) -> boolean().
is_odd(X) ->
    (X rem 2) /= 0.

-file("src/gleam/int.gleam", 402).
?DOC(
    " Returns the negative of the value provided.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " negate(1)\n"
    " // -> -1\n"
    " ```\n"
).
-spec negate(integer()) -> integer().
negate(X) ->
    -1 * X.

-file("src/gleam/int.gleam", 419).
-spec sum_loop(list(integer()), integer()) -> integer().
sum_loop(Numbers, Initial) ->
    case Numbers of
        [First | Rest] ->
            sum_loop(Rest, First + Initial);

        [] ->
            Initial
    end.

-file("src/gleam/int.gleam", 415).
?DOC(
    " Sums a list of ints.\n"
    "\n"
    " ## Example\n"
    "\n"
    " ```gleam\n"
    " sum([1, 2, 3])\n"
    " // -> 6\n"
    " ```\n"
).
-spec sum(list(integer())) -> integer().
sum(Numbers) ->
    sum_loop(Numbers, 0).

-file("src/gleam/int.gleam", 439).
-spec product_loop(list(integer()), integer()) -> integer().
product_loop(Numbers, Initial) ->
    case Numbers of
        [First | Rest] ->
            product_loop(Rest, First * Initial);

        [] ->
            Initial
    end.

-file("src/gleam/int.gleam", 435).
?DOC(
    " Multiplies a list of ints and returns the product.\n"
    "\n"
    " ## Example\n"
    "\n"
    " ```gleam\n"
    " product([2, 3, 4])\n"
    " // -> 24\n"
    " ```\n"
).
-spec product(list(integer())) -> integer().
product(Numbers) ->
    product_loop(Numbers, 1).

-file("src/gleam/int.gleam", 468).
-spec digits_loop(integer(), integer(), list(integer())) -> list(integer()).
digits_loop(X, Base, Acc) ->
    case absolute_value(X) < Base of
        true ->
            [X | Acc];

        false ->
            digits_loop(case Base of
                    0 -> 0;
                    Gleam@denominator -> X div Gleam@denominator
                end, Base, [case Base of
                        0 -> 0;
                        Gleam@denominator@1 -> X rem Gleam@denominator@1
                    end | Acc])
    end.

-file("src/gleam/int.gleam", 461).
?DOC(
    " Splits an integer into its digit representation in the specified base.\n"
    " Returns an error if the base is less than 2.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " digits(234, 10)\n"
    " // -> Ok([2,3,4])\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " digits(234, 1)\n"
    " // -> Error(Nil)\n"
    " ```\n"
).
-spec digits(integer(), integer()) -> {ok, list(integer())} | {error, nil}.
digits(X, Base) ->
    case Base < 2 of
        true ->
            {error, nil};

        false ->
            {ok, digits_loop(X, Base, [])}
    end.

-file("src/gleam/int.gleam", 502).
-spec undigits_loop(list(integer()), integer(), integer()) -> {ok, integer()} |
    {error, nil}.
undigits_loop(Numbers, Base, Acc) ->
    case Numbers of
        [] ->
            {ok, Acc};

        [Digit | _] when Digit >= Base ->
            {error, nil};

        [Digit@1 | Rest] ->
            undigits_loop(Rest, Base, (Acc * Base) + Digit@1)
    end.

-file("src/gleam/int.gleam", 495).
?DOC(
    " Joins a list of digits into a single value.\n"
    " Returns an error if the base is less than 2 or if the list contains a digit greater than or equal to the specified base.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " undigits([2,3,4], 10)\n"
    " // -> Ok(234)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " undigits([2,3,4], 1)\n"
    " // -> Error(Nil)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " undigits([2,3,4], 2)\n"
    " // -> Error(Nil)\n"
    " ```\n"
).
-spec undigits(list(integer()), integer()) -> {ok, integer()} | {error, nil}.
undigits(Numbers, Base) ->
    case Base < 2 of
        true ->
            {error, nil};

        false ->
            undigits_loop(Numbers, Base, 0)
    end.

-file("src/gleam/int.gleam", 531).
?DOC(
    " Generates a random int between zero and the given maximum.\n"
    "\n"
    " The lower number is inclusive, the upper number is exclusive.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " random(10)\n"
    " // -> 4\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " random(1)\n"
    " // -> 0\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " random(-1)\n"
    " // -> -1\n"
    " ```\n"
).
-spec random(integer()) -> integer().
random(Max) ->
    _pipe = (rand:uniform() * erlang:float(Max)),
    _pipe@1 = math:floor(_pipe),
    erlang:round(_pipe@1).

-file("src/gleam/int.gleam", 564).
?DOC(
    " Performs a truncated integer division.\n"
    "\n"
    " Returns division of the inputs as a `Result`: If the given divisor equals\n"
    " `0`, this function returns an `Error`.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " divide(0, 1)\n"
    " // -> Ok(0)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " divide(1, 0)\n"
    " // -> Error(Nil)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " divide(5, 2)\n"
    " // -> Ok(2)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " divide(-99, 2)\n"
    " // -> Ok(-49)\n"
    " ```\n"
).
-spec divide(integer(), integer()) -> {ok, integer()} | {error, nil}.
divide(Dividend, Divisor) ->
    case Divisor of
        0 ->
            {error, nil};

        Divisor@1 ->
            {ok, case Divisor@1 of
                    0 -> 0;
                    Gleam@denominator -> Dividend div Gleam@denominator
                end}
    end.

-file("src/gleam/int.gleam", 616).
?DOC(
    " Computes the remainder of an integer division of inputs as a `Result`.\n"
    "\n"
    " Returns division of the inputs as a `Result`: If the given divisor equals\n"
    " `0`, this function returns an `Error`.\n"
    "\n"
    " Most the time you will want to use the `%` operator instead of this\n"
    " function.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " remainder(3, 2)\n"
    " // -> Ok(1)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " remainder(1, 0)\n"
    " // -> Error(Nil)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " remainder(10, -1)\n"
    " // -> Ok(0)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " remainder(13, by: 3)\n"
    " // -> Ok(1)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " remainder(-13, by: 3)\n"
    " // -> Ok(-1)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " remainder(13, by: -3)\n"
    " // -> Ok(1)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " remainder(-13, by: -3)\n"
    " // -> Ok(-1)\n"
    " ```\n"
).
-spec remainder(integer(), integer()) -> {ok, integer()} | {error, nil}.
remainder(Dividend, Divisor) ->
    case Divisor of
        0 ->
            {error, nil};

        Divisor@1 ->
            {ok, case Divisor@1 of
                    0 -> 0;
                    Gleam@denominator -> Dividend rem Gleam@denominator
                end}
    end.

-file("src/gleam/int.gleam", 658).
?DOC(
    " Computes the modulo of an integer division of inputs as a `Result`.\n"
    "\n"
    " Returns division of the inputs as a `Result`: If the given divisor equals\n"
    " `0`, this function returns an `Error`.\n"
    "\n"
    " Most the time you will want to use the `%` operator instead of this\n"
    " function.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " modulo(3, 2)\n"
    " // -> Ok(1)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " modulo(1, 0)\n"
    " // -> Error(Nil)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " modulo(10, -1)\n"
    " // -> Ok(0)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " modulo(13, by: 3)\n"
    " // -> Ok(1)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " modulo(-13, by: 3)\n"
    " // -> Ok(2)\n"
    " ```\n"
).
-spec modulo(integer(), integer()) -> {ok, integer()} | {error, nil}.
modulo(Dividend, Divisor) ->
    case Divisor of
        0 ->
            {error, nil};

        _ ->
            Remainder = case Divisor of
                0 -> 0;
                Gleam@denominator -> Dividend rem Gleam@denominator
            end,
            case (Remainder * Divisor) < 0 of
                true ->
                    {ok, Remainder + Divisor};

                false ->
                    {ok, Remainder}
            end
    end.

-file("src/gleam/int.gleam", 702).
?DOC(
    " Performs a *floored* integer division, which means that the result will\n"
    " always be rounded towards negative infinity.\n"
    "\n"
    " If you want to perform truncated integer division (rounding towards zero),\n"
    " use `int.divide()` or the `/` operator instead.\n"
    "\n"
    " Returns division of the inputs as a `Result`: If the given divisor equals\n"
    " `0`, this function returns an `Error`.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " floor_divide(1, 0)\n"
    " // -> Error(Nil)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " floor_divide(5, 2)\n"
    " // -> Ok(2)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " floor_divide(6, -4)\n"
    " // -> Ok(-2)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " floor_divide(-99, 2)\n"
    " // -> Ok(-50)\n"
    " ```\n"
).
-spec floor_divide(integer(), integer()) -> {ok, integer()} | {error, nil}.
floor_divide(Dividend, Divisor) ->
    case Divisor of
        0 ->
            {error, nil};

        Divisor@1 ->
            case ((Dividend * Divisor@1) < 0) andalso ((case Divisor@1 of
                0 -> 0;
                Gleam@denominator -> Dividend rem Gleam@denominator
            end) /= 0) of
                true ->
                    {ok, (case Divisor@1 of
                            0 -> 0;
                            Gleam@denominator@1 -> Dividend div Gleam@denominator@1
                        end) - 1};

                false ->
                    {ok, case Divisor@1 of
                            0 -> 0;
                            Gleam@denominator@2 -> Dividend div Gleam@denominator@2
                        end}
            end
    end.

-file("src/gleam/int.gleam", 736).
?DOC(
    " Adds two integers together.\n"
    "\n"
    " It's the function equivalent of the `+` operator.\n"
    " This function is useful in higher order functions or pipes.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " add(1, 2)\n"
    " // -> 3\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " import gleam/list\n"
    " list.fold([1, 2, 3], 0, add)\n"
    " // -> 6\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " 3 |> add(2)\n"
    " // -> 5\n"
    " ```\n"
).
-spec add(integer(), integer()) -> integer().
add(A, B) ->
    A + B.

-file("src/gleam/int.gleam", 764).
?DOC(
    " Multiplies two integers together.\n"
    "\n"
    " It's the function equivalent of the `*` operator.\n"
    " This function is useful in higher order functions or pipes.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " multiply(2, 4)\n"
    " // -> 8\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " import gleam/list\n"
    "\n"
    " list.fold([2, 3, 4], 1, multiply)\n"
    " // -> 24\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " 3 |> multiply(2)\n"
    " // -> 6\n"
    " ```\n"
).
-spec multiply(integer(), integer()) -> integer().
multiply(A, B) ->
    A * B.

-file("src/gleam/int.gleam", 797).
?DOC(
    " Subtracts one int from another.\n"
    "\n"
    " It's the function equivalent of the `-` operator.\n"
    " This function is useful in higher order functions or pipes.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " subtract(3, 1)\n"
    " // -> 2\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " import gleam/list\n"
    "\n"
    " list.fold([1, 2, 3], 10, subtract)\n"
    " // -> 4\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " 3 |> subtract(2)\n"
    " // -> 1\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " 3 |> subtract(2, _)\n"
    " // -> -1\n"
    " ```\n"
).
-spec subtract(integer(), integer()) -> integer().
subtract(A, B) ->
    A - B.

-file("src/gleam/int.gleam", 809).
?DOC(
    " Calculates the bitwise AND of its arguments.\n"
    "\n"
    " The exact behaviour of this function depends on the target platform.\n"
    " On Erlang it is equivalent to bitwise operations on ints, on JavaScript it\n"
    " is equivalent to bitwise operations on big-ints.\n"
).
-spec bitwise_and(integer(), integer()) -> integer().
bitwise_and(X, Y) ->
    erlang:'band'(X, Y).

-file("src/gleam/int.gleam", 819).
?DOC(
    " Calculates the bitwise NOT of its argument.\n"
    "\n"
    " The exact behaviour of this function depends on the target platform.\n"
    " On Erlang it is equivalent to bitwise operations on ints, on JavaScript it\n"
    " is equivalent to bitwise operations on big-ints.\n"
).
-spec bitwise_not(integer()) -> integer().
bitwise_not(X) ->
    erlang:'bnot'(X).

-file("src/gleam/int.gleam", 829).
?DOC(
    " Calculates the bitwise OR of its arguments.\n"
    "\n"
    " The exact behaviour of this function depends on the target platform.\n"
    " On Erlang it is equivalent to bitwise operations on ints, on JavaScript it\n"
    " is equivalent to bitwise operations on big-ints.\n"
).
-spec bitwise_or(integer(), integer()) -> integer().
bitwise_or(X, Y) ->
    erlang:'bor'(X, Y).

-file("src/gleam/int.gleam", 839).
?DOC(
    " Calculates the bitwise XOR of its arguments.\n"
    "\n"
    " The exact behaviour of this function depends on the target platform.\n"
    " On Erlang it is equivalent to bitwise operations on ints, on JavaScript it\n"
    " is equivalent to bitwise operations on big-ints.\n"
).
-spec bitwise_exclusive_or(integer(), integer()) -> integer().
bitwise_exclusive_or(X, Y) ->
    erlang:'bxor'(X, Y).

-file("src/gleam/int.gleam", 849).
?DOC(
    " Calculates the result of an arithmetic left bitshift.\n"
    "\n"
    " The exact behaviour of this function depends on the target platform.\n"
    " On Erlang it is equivalent to bitwise operations on ints, on JavaScript it\n"
    " is equivalent to bitwise operations on big-ints.\n"
).
-spec bitwise_shift_left(integer(), integer()) -> integer().
bitwise_shift_left(X, Y) ->
    erlang:'bsl'(X, Y).

-file("src/gleam/int.gleam", 859).
?DOC(
    " Calculates the result of an arithmetic right bitshift.\n"
    "\n"
    " The exact behaviour of this function depends on the target platform.\n"
    " On Erlang it is equivalent to bitwise operations on ints, on JavaScript it\n"
    " is equivalent to bitwise operations on big-ints.\n"
).
-spec bitwise_shift_right(integer(), integer()) -> integer().
bitwise_shift_right(X, Y) ->
    erlang:'bsr'(X, Y).
