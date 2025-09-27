-module(gleam@bool).
-compile([no_auto_import, nowarn_unused_vars, nowarn_unused_function, nowarn_nomatch]).

-export(['and'/2, 'or'/2, negate/1, nor/2, nand/2, exclusive_or/2, exclusive_nor/2, to_string/1, guard/3, lazy_guard/3]).

-if(?OTP_RELEASE >= 27).
-define(MODULEDOC(Str), -moduledoc(Str)).
-define(DOC(Str), -doc(Str)).
-else.
-define(MODULEDOC(Str), -compile([])).
-define(DOC(Str), -compile([])).
-endif.

?MODULEDOC(
    " A type with two possible values, `True` and `False`. Used to indicate whether\n"
    " things are... true or false!\n"
    "\n"
    " Often is it clearer and offers more type safety to define a custom type\n"
    " than to use `Bool`. For example, rather than having a `is_teacher: Bool`\n"
    " field consider having a `role: SchoolRole` field where `SchoolRole` is a custom\n"
    " type that can be either `Student` or `Teacher`.\n"
).

-file("src/gleam/bool.gleam", 31).
?DOC(
    " Returns the and of two bools, but it evaluates both arguments.\n"
    "\n"
    " It's the function equivalent of the `&&` operator.\n"
    " This function is useful in higher order functions or pipes.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " and(True, True)\n"
    " // -> True\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " and(False, True)\n"
    " // -> False\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " False |> and(True)\n"
    " // -> False\n"
    " ```\n"
).
-spec 'and'(boolean(), boolean()) -> boolean().
'and'(A, B) ->
    A andalso B.

-file("src/gleam/bool.gleam", 57).
?DOC(
    " Returns the or of two bools, but it evaluates both arguments.\n"
    "\n"
    " It's the function equivalent of the `||` operator.\n"
    " This function is useful in higher order functions or pipes.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " or(True, True)\n"
    " // -> True\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " or(False, True)\n"
    " // -> True\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " False |> or(True)\n"
    " // -> True\n"
    " ```\n"
).
-spec 'or'(boolean(), boolean()) -> boolean().
'or'(A, B) ->
    A orelse B.

-file("src/gleam/bool.gleam", 77).
?DOC(
    " Returns the opposite bool value.\n"
    "\n"
    " This is the same as the `!` or `not` operators in some other languages.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " negate(True)\n"
    " // -> False\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " negate(False)\n"
    " // -> True\n"
    " ```\n"
).
-spec negate(boolean()) -> boolean().
negate(Bool) ->
    not Bool.

-file("src/gleam/bool.gleam", 105).
?DOC(
    " Returns the nor of two bools.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " nor(False, False)\n"
    " // -> True\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " nor(False, True)\n"
    " // -> False\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " nor(True, False)\n"
    " // -> False\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " nor(True, True)\n"
    " // -> False\n"
    " ```\n"
).
-spec nor(boolean(), boolean()) -> boolean().
nor(A, B) ->
    not (A orelse B).

-file("src/gleam/bool.gleam", 133).
?DOC(
    " Returns the nand of two bools.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " nand(False, False)\n"
    " // -> True\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " nand(False, True)\n"
    " // -> True\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " nand(True, False)\n"
    " // -> True\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " nand(True, True)\n"
    " // -> False\n"
    " ```\n"
).
-spec nand(boolean(), boolean()) -> boolean().
nand(A, B) ->
    not (A andalso B).

-file("src/gleam/bool.gleam", 161).
?DOC(
    " Returns the exclusive or of two bools.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " exclusive_or(False, False)\n"
    " // -> False\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " exclusive_or(False, True)\n"
    " // -> True\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " exclusive_or(True, False)\n"
    " // -> True\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " exclusive_or(True, True)\n"
    " // -> False\n"
    " ```\n"
).
-spec exclusive_or(boolean(), boolean()) -> boolean().
exclusive_or(A, B) ->
    A /= B.

-file("src/gleam/bool.gleam", 189).
?DOC(
    " Returns the exclusive nor of two bools.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " exclusive_nor(False, False)\n"
    " // -> True\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " exclusive_nor(False, True)\n"
    " // -> False\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " exclusive_nor(True, False)\n"
    " // -> False\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " exclusive_nor(True, True)\n"
    " // -> True\n"
    " ```\n"
).
-spec exclusive_nor(boolean(), boolean()) -> boolean().
exclusive_nor(A, B) ->
    A =:= B.

-file("src/gleam/bool.gleam", 207).
?DOC(
    " Returns a string representation of the given bool.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " to_string(True)\n"
    " // -> \"True\"\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " to_string(False)\n"
    " // -> \"False\"\n"
    " ```\n"
).
-spec to_string(boolean()) -> binary().
to_string(Bool) ->
    case Bool of
        false ->
            <<"False"/utf8>>;

        true ->
            <<"True"/utf8>>
    end.

-file("src/gleam/bool.gleam", 266).
?DOC(
    " Run a callback function if the given bool is `False`, otherwise return a\n"
    " default value.\n"
    "\n"
    " With a `use` expression this function can simulate the early-return pattern\n"
    " found in some other programming languages.\n"
    "\n"
    " In a procedural language:\n"
    "\n"
    " ```js\n"
    " if (predicate) return value;\n"
    " // ...\n"
    " ```\n"
    "\n"
    " In Gleam with a `use` expression:\n"
    "\n"
    " ```gleam\n"
    " use <- guard(when: predicate, return: value)\n"
    " // ...\n"
    " ```\n"
    "\n"
    " Like everything in Gleam `use` is an expression, so it short circuits the\n"
    " current block, not the entire function. As a result you can assign the value\n"
    " to a variable:\n"
    "\n"
    " ```gleam\n"
    " let x = {\n"
    "   use <- guard(when: predicate, return: value)\n"
    "   // ...\n"
    " }\n"
    " ```\n"
    "\n"
    " Note that unlike in procedural languages the `return` value is evaluated\n"
    " even when the predicate is `False`, so it is advisable not to perform\n"
    " expensive computation nor side-effects there.\n"
    "\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " let name = \"\"\n"
    " use <- guard(when: name == \"\", return: \"Welcome!\")\n"
    " \"Hello, \" <> name\n"
    " // -> \"Welcome!\"\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " let name = \"Kamaka\"\n"
    " use <- guard(when: name == \"\", return: \"Welcome!\")\n"
    " \"Hello, \" <> name\n"
    " // -> \"Hello, Kamaka\"\n"
    " ```\n"
).
-spec guard(boolean(), BWY, fun(() -> BWY)) -> BWY.
guard(Requirement, Consequence, Alternative) ->
    case Requirement of
        true ->
            Consequence;

        false ->
            Alternative()
    end.

-file("src/gleam/bool.gleam", 307).
?DOC(
    " Runs a callback function if the given bool is `True`, otherwise runs an\n"
    " alternative callback function.\n"
    "\n"
    " Useful when further computation should be delayed regardless of the given\n"
    " bool's value.\n"
    "\n"
    " See [`guard`](#guard) for more info.\n"
    "\n"
    " ## Examples\n"
    "\n"
    " ```gleam\n"
    " let name = \"Kamaka\"\n"
    " let inquiry = fn() { \"How may we address you?\" }\n"
    " use <- lazy_guard(when: name == \"\", return: inquiry)\n"
    " \"Hello, \" <> name\n"
    " // -> \"Hello, Kamaka\"\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " import gleam/int\n"
    "\n"
    " let name = \"\"\n"
    " let greeting = fn() { \"Hello, \" <> name }\n"
    " use <- lazy_guard(when: name == \"\", otherwise: greeting)\n"
    " let number = int.random(99)\n"
    " let name = \"User \" <> int.to_string(number)\n"
    " \"Welcome, \" <> name\n"
    " // -> \"Welcome, User 54\"\n"
    " ```\n"
).
-spec lazy_guard(boolean(), fun(() -> BWZ), fun(() -> BWZ)) -> BWZ.
lazy_guard(Requirement, Consequence, Alternative) ->
    case Requirement of
        true ->
            Consequence();

        false ->
            Alternative()
    end.
