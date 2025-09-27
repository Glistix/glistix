-module(gleam@float).
-compile([no_auto_import, nowarn_unused_vars, nowarn_unused_function, nowarn_nomatch]).

-export([parse/1, to_string/1, compare/2, min/2, max/2, clamp/3, ceiling/1, floor/1, truncate/1, absolute_value/1, loosely_compare/3, loosely_equals/3, power/2, square_root/1, negate/1, round/1, to_precision/2, sum/1, product/1, random/0, modulo/2, divide/2, add/2, multiply/2, subtract/2, logarithm/1, exponential/1]).

-if(?OTP_RELEASE >= 27).
-define(MODULEDOC(Str), -moduledoc(Str)).
-define(DOC(Str), -doc(Str)).
-else.
-define(MODULEDOC(Str), -compile([])).
-define(DOC(Str), -compile([])).
-endif.

?MODULEDOC(
    " Functions for working with floats.\n"
    "\n"
    " ## Float representation\n"
    "\n"
    " Floats are represented as 64 bit floating point numbers on both the Erlang\n"
    " and JavaScript runtimes. The floating point behaviour is native to their\n"
    " respective runtimes, so their exact behaviour will be slightly different on\n"
    " the two runtimes.\n"
    "\n"
    " ### Infinity and NaN\n"
    "\n"
    " Under the JavaScript runtime, exceeding the maximum (or minimum)\n"
    " representable value for a floating point value will result in Infinity (or\n"
    " -Infinity). Should you try to divide two infinities you will get NaN as a\n"
    " result.\n"
    "\n"
    " When running on BEAM, exceeding the maximum (or minimum) representable\n"
    " value for a floating point value will raise an error.\n"
    "\n"
    " ## Division by zero\n"
    "\n"
    " Gleam runs on the Erlang virtual machine, which does not follow the IEEE\n"
    " 754 standard for floating point arithmetic and does not have an `Infinity`\n"
    " value.  In Erlang division by zero results in a crash, however Gleam does\n"
    " not have partial functions and operators in core so instead division by zero\n"
    " returns zero, a behaviour taken from Pony, Coq, and Lean.\n"
    "\n"
    " This may seem unexpected at first, but it is no less mathematically valid\n"
    " than crashing or returning a special value. Division by zero is undefined\n"
    " in mathematics.\n"
).

-file("src/gleam/float.gleam", 51).
?DOC(
    " Attempts to parse a string as a `Float`, returning `Error(Nil)` if it was\n"
    " not possible.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " parse(\"2.3\")\n"
    " // -> Ok(2.3)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " parse(\"ABC\")\n"
    " // -> Error(Nil)\n"
    " ```\n"
).
-spec parse(binary()) -> {ok, float()} | {error, nil}.
parse(String) ->
    gleam_stdlib:parse_float(String).

-file("src/gleam/float.gleam", 64).
?DOC(
    " Returns the string representation of the provided `Float`.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " to_string(2.3)\n"
    " // -> \"2.3\"\n"
    " ```\n"
).
-spec to_string(float()) -> binary().
to_string(X) ->
    gleam_stdlib:float_to_string(X).

-file("src/gleam/float.gleam", 95).
?DOC(
    " Compares two `Float`s, returning an `Order`:\n"
    " `Lt` for lower than, `Eq` for equals, or `Gt` for greater than.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " compare(2.0, 2.3)\n"
    " // -> Lt\n"
    " ```\n"
    "\n"
    " To handle\n"
    " [Floating Point Imprecision](https://en.wikipedia.org/wiki/Floating-point_arithmetic#Accuracy_problems)\n"
    " you may use [`loosely_compare`](#loosely_compare) instead.\n"
).
-spec compare(float(), float()) -> gleam@order:order().
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

-file("src/gleam/float.gleam", 176).
?DOC(
    " Compares two `Float`s, returning the smaller of the two.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " min(2.0, 2.3)\n"
    " // -> 2.0\n"
    " ```\n"
).
-spec min(float(), float()) -> float().
min(A, B) ->
    case A < B of
        true ->
            A;

        false ->
            B
    end.

-file("src/gleam/float.gleam", 192).
?DOC(
    " Compares two `Float`s, returning the larger of the two.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " max(2.0, 2.3)\n"
    " // -> 2.3\n"
    " ```\n"
).
-spec max(float(), float()) -> float().
max(A, B) ->
    case A > B of
        true ->
            A;

        false ->
            B
    end.

-file("src/gleam/float.gleam", 75).
?DOC(
    " Restricts a `Float` between a lower and upper bound.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " clamp(1.2, min: 1.4, max: 1.6)\n"
    " // -> 1.4\n"
    " ```\n"
).
-spec clamp(float(), float(), float()) -> float().
clamp(X, Min_bound, Max_bound) ->
    _pipe = X,
    _pipe@1 = min(_pipe, Max_bound),
    max(_pipe@1, Min_bound).

-file("src/gleam/float.gleam", 210).
?DOC(
    " Rounds the value to the next highest whole number as a `Float`.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " ceiling(2.3)\n"
    " // -> 3.0\n"
    " ```\n"
).
-spec ceiling(float()) -> float().
ceiling(X) ->
    math:ceil(X).

-file("src/gleam/float.gleam", 223).
?DOC(
    " Rounds the value to the next lowest whole number as a `Float`.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " floor(2.3)\n"
    " // -> 2.0\n"
    " ```\n"
).
-spec floor(float()) -> float().
floor(X) ->
    math:floor(X).

-file("src/gleam/float.gleam", 261).
?DOC(
    " Returns the value as an `Int`, truncating all decimal digits.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " truncate(2.4343434847383438)\n"
    " // -> 2\n"
    " ```\n"
).
-spec truncate(float()) -> integer().
truncate(X) ->
    erlang:trunc(X).

-file("src/gleam/float.gleam", 311).
?DOC(
    " Returns the absolute value of the input as a `Float`.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " absolute_value(-12.5)\n"
    " // -> 12.5\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " absolute_value(10.2)\n"
    " // -> 10.2\n"
    " ```\n"
).
-spec absolute_value(float()) -> float().
absolute_value(X) ->
    case X >= +0.0 of
        true ->
            X;

        false ->
            +0.0 - X
    end.

-file("src/gleam/float.gleam", 125).
?DOC(
    " Compares two `Float`s within a tolerance, returning an `Order`:\n"
    " `Lt` for lower than, `Eq` for equals, or `Gt` for greater than.\n"
    "\n"
    " This function allows Float comparison while handling\n"
    " [Floating Point Imprecision](https://en.wikipedia.org/wiki/Floating-point_arithmetic#Accuracy_problems).\n"
    "\n"
    " Notice: For `Float`s the tolerance won't be exact:\n"
    " `5.3 - 5.0` is not exactly `0.3`.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " loosely_compare(5.0, with: 5.3, tolerating: 0.5)\n"
    " // -> Eq\n"
    " ```\n"
    "\n"
    " If you want to check only for equality you may use\n"
    " [`loosely_equals`](#loosely_equals) instead.\n"
).
-spec loosely_compare(float(), float(), float()) -> gleam@order:order().
loosely_compare(A, B, Tolerance) ->
    Difference = absolute_value(A - B),
    case Difference =< Tolerance of
        true ->
            eq;

        false ->
            compare(A, B)
    end.

-file("src/gleam/float.gleam", 158).
?DOC(
    " Checks for equality of two `Float`s within a tolerance,\n"
    " returning an `Bool`.\n"
    "\n"
    " This function allows Float comparison while handling\n"
    " [Floating Point Imprecision](https://en.wikipedia.org/wiki/Floating-point_arithmetic#Accuracy_problems).\n"
    "\n"
    " Notice: For `Float`s the tolerance won't be exact:\n"
    " `5.3 - 5.0` is not exactly `0.3`.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " loosely_equals(5.0, with: 5.3, tolerating: 0.5)\n"
    " // -> True\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " loosely_equals(5.0, with: 5.1, tolerating: 0.1)\n"
    " // -> False\n"
    " ```\n"
).
-spec loosely_equals(float(), float(), float()) -> boolean().
loosely_equals(A, B, Tolerance) ->
    Difference = absolute_value(A - B),
    Difference =< Tolerance.

-file("src/gleam/float.gleam", 348).
?DOC(
    " Returns the results of the base being raised to the power of the\n"
    " exponent, as a `Float`.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " power(2.0, -1.0)\n"
    " // -> Ok(0.5)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " power(2.0, 2.0)\n"
    " // -> Ok(4.0)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " power(8.0, 1.5)\n"
    " // -> Ok(22.627416997969522)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " 4.0 |> power(of: 2.0)\n"
    " // -> Ok(16.0)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " power(-1.0, 0.5)\n"
    " // -> Error(Nil)\n"
    " ```\n"
).
-spec power(float(), float()) -> {ok, float()} | {error, nil}.
power(Base, Exponent) ->
    Fractional = (math:ceil(Exponent) - Exponent) > +0.0,
    case ((Base < +0.0) andalso Fractional) orelse ((Base =:= +0.0) andalso (Exponent
    < +0.0)) of
        true ->
            {error, nil};

        false ->
            {ok, math:pow(Base, Exponent)}
    end.

-file("src/gleam/float.gleam", 380).
?DOC(
    " Returns the square root of the input as a `Float`.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " square_root(4.0)\n"
    " // -> Ok(2.0)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " square_root(-16.0)\n"
    " // -> Error(Nil)\n"
    " ```\n"
).
-spec square_root(float()) -> {ok, float()} | {error, nil}.
square_root(X) ->
    power(X, 0.5).

-file("src/gleam/float.gleam", 393).
?DOC(
    " Returns the negative of the value provided.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " negate(1.0)\n"
    " // -> -1.0\n"
    " ```\n"
).
-spec negate(float()) -> float().
negate(X) ->
    -1.0 * X.

-file("src/gleam/float.gleam", 240).
?DOC(
    " Rounds the value to the nearest whole number as an `Int`.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " round(2.3)\n"
    " // -> 2\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " round(2.5)\n"
    " // -> 3\n"
    " ```\n"
).
-spec round(float()) -> integer().
round(X) ->
    erlang:round(X).

-file("src/gleam/float.gleam", 280).
?DOC(
    " Converts the value to a given precision as a `Float`.\n"
    " The precision is the number of allowed decimal places.\n"
    " Negative precisions are allowed and force rounding\n"
    " to the nearest tenth, hundredth, thousandth etc.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " to_precision(2.43434348473, precision: 2)\n"
    " // -> 2.43\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " to_precision(547890.453444, precision: -3)\n"
    " // -> 548000.0\n"
    " ```\n"
).
-spec to_precision(float(), integer()) -> float().
to_precision(X, Precision) ->
    case Precision =< 0 of
        true ->
            Factor = math:pow(10.0, erlang:float(- Precision)),
            erlang:float(erlang:round(case Factor of
                        +0.0 -> +0.0;
                        -0.0 -> -0.0;
                        Gleam@denominator -> X / Gleam@denominator
                    end)) * Factor;

        false ->
            Factor@1 = math:pow(10.0, erlang:float(Precision)),
            case Factor@1 of
                +0.0 -> +0.0;
                -0.0 -> -0.0;
                Gleam@denominator@1 -> erlang:float(erlang:round(X * Factor@1))
                / Gleam@denominator@1
            end
    end.

-file("src/gleam/float.gleam", 410).
-spec sum_loop(list(float()), float()) -> float().
sum_loop(Numbers, Initial) ->
    case Numbers of
        [First | Rest] ->
            sum_loop(Rest, First + Initial);

        [] ->
            Initial
    end.

-file("src/gleam/float.gleam", 406).
?DOC(
    " Sums a list of `Float`s.\n"
    "\n"
    " ## Example\n"
    "\n"
    " ```gleam\n"
    " sum([1.0, 2.2, 3.3])\n"
    " // -> 6.5\n"
    " ```\n"
).
-spec sum(list(float())) -> float().
sum(Numbers) ->
    sum_loop(Numbers, +0.0).

-file("src/gleam/float.gleam", 430).
-spec product_loop(list(float()), float()) -> float().
product_loop(Numbers, Initial) ->
    case Numbers of
        [First | Rest] ->
            product_loop(Rest, First * Initial);

        [] ->
            Initial
    end.

-file("src/gleam/float.gleam", 426).
?DOC(
    " Multiplies a list of `Float`s and returns the product.\n"
    "\n"
    " ## Example\n"
    "\n"
    " ```gleam\n"
    " product([2.5, 3.2, 4.2])\n"
    " // -> 33.6\n"
    " ```\n"
).
-spec product(list(float())) -> float().
product(Numbers) ->
    product_loop(Numbers, 1.0).

-file("src/gleam/float.gleam", 452).
?DOC(
    " Generates a random float between the given zero (inclusive) and one\n"
    " (exclusive).\n"
    "\n"
    " On Erlang this updates the random state in the process dictionary.\n"
    " See: <https://www.erlang.org/doc/man/rand.html#uniform-0>\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " random()\n"
    " // -> 0.646355926896028\n"
    " ```\n"
).
-spec random() -> float().
random() ->
    rand:uniform().

-file("src/gleam/float.gleam", 481).
?DOC(
    " Computes the modulo of an float division of inputs as a `Result`.\n"
    "\n"
    " Returns division of the inputs as a `Result`: If the given divisor equals\n"
    " `0`, this function returns an `Error`.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " modulo(13.3, by: 3.3)\n"
    " // -> Ok(0.1)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " modulo(-13.3, by: 3.3)\n"
    " // -> Ok(3.2)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " modulo(13.3, by: -3.3)\n"
    " // -> Ok(-3.2)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " modulo(-13.3, by: -3.3)\n"
    " // -> Ok(-0.1)\n"
    " ```\n"
).
-spec modulo(float(), float()) -> {ok, float()} | {error, nil}.
modulo(Dividend, Divisor) ->
    case Divisor of
        +0.0 ->
            {error, nil};

        _ ->
            {ok, Dividend - (math:floor(case Divisor of
                        +0.0 -> +0.0;
                        -0.0 -> -0.0;
                        Gleam@denominator -> Dividend / Gleam@denominator
                    end) * Divisor)}
    end.

-file("src/gleam/float.gleam", 502).
?DOC(
    " Returns division of the inputs as a `Result`.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " divide(0.0, 1.0)\n"
    " // -> Ok(0.0)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " divide(1.0, 0.0)\n"
    " // -> Error(Nil)\n"
    " ```\n"
).
-spec divide(float(), float()) -> {ok, float()} | {error, nil}.
divide(A, B) ->
    case B of
        +0.0 ->
            {error, nil};

        B@1 ->
            {ok, case B@1 of
                    +0.0 -> +0.0;
                    -0.0 -> -0.0;
                    Gleam@denominator -> A / Gleam@denominator
                end}
    end.

-file("src/gleam/float.gleam", 533).
?DOC(
    " Adds two floats together.\n"
    "\n"
    " It's the function equivalent of the `+.` operator.\n"
    " This function is useful in higher order functions or pipes.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " add(1.0, 2.0)\n"
    " // -> 3.0\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " import gleam/list\n"
    "\n"
    " list.fold([1.0, 2.0, 3.0], 0.0, add)\n"
    " // -> 6.0\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " 3.0 |> add(2.0)\n"
    " // -> 5.0\n"
    " ```\n"
).
-spec add(float(), float()) -> float().
add(A, B) ->
    A + B.

-file("src/gleam/float.gleam", 561).
?DOC(
    " Multiplies two floats together.\n"
    "\n"
    " It's the function equivalent of the `*.` operator.\n"
    " This function is useful in higher order functions or pipes.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " multiply(2.0, 4.0)\n"
    " // -> 8.0\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " import gleam/list\n"
    "\n"
    " list.fold([2.0, 3.0, 4.0], 1.0, multiply)\n"
    " // -> 24.0\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " 3.0 |> multiply(2.0)\n"
    " // -> 6.0\n"
    " ```\n"
).
-spec multiply(float(), float()) -> float().
multiply(A, B) ->
    A * B.

-file("src/gleam/float.gleam", 594).
?DOC(
    " Subtracts one float from another.\n"
    "\n"
    " It's the function equivalent of the `-.` operator.\n"
    " This function is useful in higher order functions or pipes.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " subtract(3.0, 1.0)\n"
    " // -> 2.0\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " import gleam/list\n"
    "\n"
    " list.fold([1.0, 2.0, 3.0], 10.0, subtract)\n"
    " // -> 4.0\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " 3.0 |> subtract(_, 2.0)\n"
    " // -> 1.0\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " 3.0 |> subtract(2.0, _)\n"
    " // -> -1.0\n"
    " ```\n"
).
-spec subtract(float(), float()) -> float().
subtract(A, B) ->
    A - B.

-file("src/gleam/float.gleam", 623).
?DOC(
    " Returns the natural logarithm (base e) of the given as a `Result`. If the\n"
    " input is less than or equal to 0, returns `Error(Nil)`.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " logarithm(1.0)\n"
    " // -> Ok(0.0)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " logarithm(2.718281828459045)  // e\n"
    " // -> Ok(1.0)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " logarithm(0.0)\n"
    " // -> Error(Nil)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " logarithm(-1.0)\n"
    " // -> Error(Nil)\n"
    " ```\n"
).
-spec logarithm(float()) -> {ok, float()} | {error, nil}.
logarithm(X) ->
    case X =< +0.0 of
        true ->
            {error, nil};

        false ->
            {ok, math:log(X)}
    end.

-file("src/gleam/float.gleam", 661).
?DOC(
    " Returns e (Euler's number) raised to the power of the given exponent, as\n"
    " a `Float`.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " exponential(0.0)\n"
    " // -> Ok(1.0)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " exponential(1.0)\n"
    " // -> Ok(2.718281828459045)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " exponential(-1.0)\n"
    " // -> Ok(0.36787944117144233)\n"
    " ```\n"
).
-spec exponential(float()) -> float().
exponential(X) ->
    math:exp(X).
