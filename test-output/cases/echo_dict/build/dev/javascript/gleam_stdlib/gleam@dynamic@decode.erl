-module(gleam@dynamic@decode).
-compile([no_auto_import, nowarn_unused_vars, nowarn_unused_function, nowarn_nomatch]).

-export([run/2, success/1, decode_dynamic/1, map/2, map_errors/2, then/2, one_of/2, recursive/1, optional/1, decode_error/2, decode_bool/1, decode_int/1, decode_float/1, decode_bit_array/1, collapse_errors/2, failure/2, new_primitive_decoder/2, decode_string/1, dict/2, list/1, subfield/3, at/2, field/3, optional_field/4, optionally_at/3]).
-export_type([decode_error/0, decoder/1]).

-if(?OTP_RELEASE >= 27).
-define(MODULEDOC(Str), -moduledoc(Str)).
-define(DOC(Str), -doc(Str)).
-else.
-define(MODULEDOC(Str), -compile([])).
-define(DOC(Str), -compile([])).
-endif.

?MODULEDOC(
    " The `Dynamic` type is used to represent dynamically typed data. That is, data\n"
    " that we don't know the precise type of yet, so we need to introspect the data to\n"
    " see if it is of the desired type before we can use it. Typically data like this\n"
    " would come from user input or from untyped languages such as Erlang or JavaScript.\n"
    "\n"
    " This module provides the `Decoder` type and associated functions, which provides\n"
    " a type-safe and composable way to convert dynamic data into some desired type,\n"
    " or into errors if the data doesn't have the desired structure.\n"
    "\n"
    " The `Decoder` type is generic and has 1 type parameter, which is the type that\n"
    " it attempts to decode. A `Decoder(String)` can be used to decode strings, and a\n"
    " `Decoder(Option(Int))` can be used to decode `Option(Int)`s\n"
    "\n"
    " Decoders work using _runtime reflection_ and the data structures of the target\n"
    " platform. Differences between Erlang and JavaScript data structures may impact\n"
    " your decoders, so it is important to test your decoders on all supported\n"
    " platforms.\n"
    "\n"
    " The decoding technique used by this module was inspired by Juraj Petráš'\n"
    " [Toy](https://github.com/Hackder/toy), Go's `encoding/json`, and Elm's\n"
    " `Json.Decode`. Thank you to them!\n"
    "\n"
    " # Examples\n"
    "\n"
    " Dynamic data may come from various sources and so many different syntaxes could\n"
    " be used to describe or construct them. In these examples a pseudocode\n"
    " syntax is used to describe the data.\n"
    "\n"
    " ## Simple types\n"
    "\n"
    " This module defines decoders for simple data types such as [`string`](#string),\n"
    " [`int`](#int), [`float`](#float), [`bit_array`](#bit_array), and [`bool`](#bool).\n"
    "\n"
    " ```gleam\n"
    " // Data:\n"
    " // \"Hello, Joe!\"\n"
    "\n"
    " let result = decode.run(data, decode.string)\n"
    " assert result == Ok(\"Hello, Joe!\")\n"
    " ```\n"
    "\n"
    " ## Lists\n"
    "\n"
    " The [`list`](#list) decoder decodes `List`s. To use it you must construct it by\n"
    " passing in another decoder into the `list` function, which is the decoder that\n"
    " is to be used for the elements of the list, type checking both the list and its\n"
    " elements.\n"
    "\n"
    " ```gleam\n"
    " // Data:\n"
    " // [1, 2, 3, 4]\n"
    "\n"
    " let result = decode.run(data, decode.list(decode.int))\n"
    " assert result == Ok([1, 2, 3, 4])\n"
    " ```\n"
    "\n"
    " On Erlang this decoder can decode from lists, and on JavaScript it can\n"
    " decode from lists as well as JavaScript arrays.\n"
    "\n"
    " ## Options\n"
    "\n"
    " The [`optional`](#optional) decoder is used to decode values that may or may not\n"
    " be present. In other environment these might be called \"nullable\" values.\n"
    "\n"
    " Like the `list` decoder, the `optional` decoder takes another decoder,\n"
    " which is used to decode the value if it is present.\n"
    "\n"
    " ```gleam\n"
    " // Data:\n"
    " // 12.45\n"
    "\n"
    " let result = decode.run(data, decode.optional(decode.float))\n"
    " assert result == Ok(option.Some(12.45))\n"
    " ```\n"
    " ```gleam\n"
    " // Data:\n"
    " // null\n"
    "\n"
    " let result = decode.run(data, decode.optional(decode.int))\n"
    " assert result == Ok(option.None)\n"
    " ```\n"
    "\n"
    " This decoder knows how to handle multiple different runtime representations of\n"
    " absent values, including `Nil`, `None`, `null`, and `undefined`.\n"
    "\n"
    " ## Dicts\n"
    "\n"
    " The [`dict`](#dict) decoder decodes `Dicts` and contains two other decoders, one\n"
    " for the keys, one for the values.\n"
    "\n"
    " ```gleam\n"
    " // Data:\n"
    " // { \"Lucy\" -> 10, \"Nubi\" -> 20 }\n"
    "\n"
    " let result = decode.run(data, decode.dict(decode.string, decode.int))\n"
    " assert result == Ok(dict.from_list([\n"
    "   #(\"Lucy\", 10),\n"
    "   #(\"Nubi\", 20),\n"
    " ]))\n"
    " ```\n"
    "\n"
    " ## Indexing objects\n"
    "\n"
    " The [`at`](#at) decoder can be used to decode a value that is nested within\n"
    " key-value containers such as Gleam dicts, Erlang maps, or JavaScript objects.\n"
    "\n"
    " ```gleam\n"
    " // Data:\n"
    " // { \"one\" -> { \"two\" -> 123 } }\n"
    "\n"
    " let result = decode.run(data, decode.at([\"one\", \"two\"], decode.int))\n"
    " assert result == Ok(123)\n"
    " ```\n"
    "\n"
    " ## Indexing arrays\n"
    "\n"
    " If you use ints as keys then the [`at`](#at) decoder can be used to index into\n"
    " array-like containers such as Gleam or Erlang tuples, or JavaScript arrays.\n"
    "\n"
    " ```gleam\n"
    " // Data:\n"
    " // [\"one\", \"two\", \"three\"]\n"
    "\n"
    " let result = decode.run(data, decode.at([1], decode.string))\n"
    " assert result == Ok(\"two\")\n"
    " ```\n"
    "\n"
    " ## Records\n"
    "\n"
    " Decoding records from dynamic data is more complex and requires combining a\n"
    " decoder for each field and a special constructor that builds your records with\n"
    " the decoded field values.\n"
    "\n"
    " ```gleam\n"
    " // Data:\n"
    " // {\n"
    " //   \"score\" -> 180,\n"
    " //   \"name\" -> \"Mel Smith\",\n"
    " //   \"is-admin\" -> false,\n"
    " //   \"enrolled\" -> true,\n"
    " //   \"colour\" -> \"Red\",\n"
    " // }\n"
    "\n"
    " let decoder = {\n"
    "   use name <- decode.field(\"name\", decode.string)\n"
    "   use score <- decode.field(\"score\", decode.int)\n"
    "   use colour <- decode.field(\"colour\", decode.string)\n"
    "   use enrolled <- decode.field(\"enrolled\", decode.bool)\n"
    "   decode.success(Player(name:, score:, colour:, enrolled:))\n"
    " }\n"
    "\n"
    " let result = decode.run(data, decoder)\n"
    " assert result == Ok(Player(\"Mel Smith\", 180, \"Red\", True))\n"
    " ```\n"
    "\n"
    " ## Enum variants\n"
    "\n"
    " Imagine you have a custom type where all the variants do not contain any values.\n"
    "\n"
    " ```gleam\n"
    " pub type PocketMonsterType {\n"
    "   Fire\n"
    "   Water\n"
    "   Grass\n"
    "   Electric\n"
    " }\n"
    " ```\n"
    "\n"
    " You might choose to encode these variants as strings, `\"fire\"` for `Fire`,\n"
    " `\"water\"` for `Water`, and so on. To decode them you'll need to decode the dynamic\n"
    " data as a string, but then you'll need to decode it further still as not all\n"
    " strings are valid values for the enum. This can be done with the `then`\n"
    " function, which enables running a second decoder after the first one\n"
    " succeeds.\n"
    "\n"
    " ```gleam\n"
    " let decoder = {\n"
    "   use decoded_string <- decode.then(decode.string)\n"
    "   case decoded_string {\n"
    "     // Return succeeding decoders for valid strings\n"
    "     \"fire\" -> decode.success(Fire)\n"
    "     \"water\" -> decode.success(Water)\n"
    "     \"grass\" -> decode.success(Grass)\n"
    "     \"electric\" -> decode.success(Electric)\n"
    "     // Return a failing decoder for any other strings\n"
    "     _ -> decode.failure(Fire, \"PocketMonsterType\")\n"
    "   }\n"
    " }\n"
    "\n"
    " let result = decode.run(dynamic.from(\"water\"), decoder)\n"
    " assert result == Ok(Water)\n"
    "\n"
    " let result = decode.run(dynamic.from(\"wobble\"), decoder)\n"
    " assert result == Error([DecodeError(\"PocketMonsterType\", \"String\", [])])\n"
    " ```\n"
    "\n"
    " ## Record variants\n"
    "\n"
    " Decoding type variants that contain other values is done by combining the\n"
    " techniques from the \"enum variants\" and \"records\" examples. Imagine you have\n"
    " this custom type that you want to decode:\n"
    "\n"
    " ```gleam\n"
    " pub type PocketMonsterPerson {\n"
    "   Trainer(name: String, badge_count: Int)\n"
    "   GymLeader(name: String, speciality: PocketMonsterType)\n"
    " }\n"
    " ```\n"
    " And you would like to be able to decode these from dynamic data like this:\n"
    " ```erlang\n"
    " {\n"
    "   \"type\" -> \"trainer\",\n"
    "   \"name\" -> \"Ash\",\n"
    "   \"badge-count\" -> 1,\n"
    " }\n"
    " ```\n"
    " ```erlang\n"
    " {\n"
    "   \"type\" -> \"gym-leader\",\n"
    "   \"name\" -> \"Misty\",\n"
    "   \"speciality\" -> \"water\",\n"
    " }\n"
    " ```\n"
    "\n"
    " Notice how both documents have a `\"type\"` field, which is used to indicate which\n"
    " variant the data is for.\n"
    "\n"
    " First, define decoders for each of the variants:\n"
    "\n"
    " ```gleam\n"
    " let trainer_decoder = {\n"
    "   use name <- decode.field(\"name\", decode.string)\n"
    "   use badge_count <- decode.field(\"badge-count\", decode.int)\n"
    "   decode.success(Trainer(name, badge_count))\n"
    " })\n"
    "\n"
    " let gym_leader_decoder = {\n"
    "   use name <- decode.field(\"name\", decode.string)\n"
    "   use speciality <- decode.field(\"speciality\", pocket_monster_type_decoder)\n"
    "   decode.success(GymLeader(name, speciality))\n"
    " }\n"
    " ```\n"
    "\n"
    " A third decoder can be used to extract and decode the `\"type\"` field, and the\n"
    " `then` function then returns whichever decoder is suitable for the document.\n"
    "\n"
    " ```gleam\n"
    " let decoder = {\n"
    "   use tag <- decode.field(\"type\", decode.string)\n"
    "   case tag {\n"
    "     \"gym-leader\" -> gym_leader_decoder\n"
    "     _ -> trainer_decoder\n"
    "   }\n"
    " }\n"
    "\n"
    " decode.run(data, decoder)\n"
    " ```\n"
).

-type decode_error() :: {decode_error, binary(), binary(), list(binary())}.

-opaque decoder(DOU) :: {decoder,
        fun((gleam@dynamic:dynamic_()) -> {DOU, list(decode_error())})}.

-file("src/gleam/dynamic/decode.gleam", 348).
?DOC(
    " Run a decoder on a `Dynamic` value, decoding the value if it is of the\n"
    " desired type, or returning errors.\n"
    "\n"
    " # Examples\n"
    "\n"
    " ```gleam\n"
    " let decoder = {\n"
    "   use name <- decode.field(\"email\", decode.string)\n"
    "   use email <- decode.field(\"password\", decode.string)\n"
    "   decode.success(SignUp(name: name, email: email))\n"
    " }\n"
    "\n"
    " decode.run(data, decoder)\n"
    " ```\n"
).
-spec run(gleam@dynamic:dynamic_(), decoder(DPC)) -> {ok, DPC} |
    {error, list(decode_error())}.
run(Data, Decoder) ->
    {Maybe_invalid_data, Errors} = (erlang:element(2, Decoder))(Data),
    case Errors of
        [] ->
            {ok, Maybe_invalid_data};

        [_ | _] ->
            {error, Errors}
    end.

-file("src/gleam/dynamic/decode.gleam", 470).
?DOC(
    " Finalise a decoder having successfully extracted a value.\n"
    "\n"
    " # Examples\n"
    "\n"
    " ```gleam\n"
    " let data = dynamic.from(dict.from_list([\n"
    "   #(\"email\", \"lucy@example.com\"),\n"
    "   #(\"name\", \"Lucy\"),\n"
    " ]))\n"
    "\n"
    " let decoder = {\n"
    "   use name <- decode.field(\"name\", string)\n"
    "   use email <- decode.field(\"email\", string)\n"
    "   decode.success(SignUp(name: name, email: email))\n"
    " }\n"
    "\n"
    " let result = decode.run(data, decoder)\n"
    " assert result == Ok(SignUp(name: \"Lucy\", email: \"lucy@example.com\"))\n"
    " ```\n"
).
-spec success(DQD) -> decoder(DQD).
success(Data) ->
    {decoder, fun(_) -> {Data, []} end}.

-file("src/gleam/dynamic/decode.gleam", 708).
-spec decode_dynamic(gleam@dynamic:dynamic_()) -> {gleam@dynamic:dynamic_(),
    list(decode_error())}.
decode_dynamic(Data) ->
    {Data, []}.

-file("src/gleam/dynamic/decode.gleam", 862).
?DOC(
    " Apply a transformation function to any value decoded by the decoder.\n"
    "\n"
    " # Examples\n"
    "\n"
    " ```gleam\n"
    " let decoder = decode.int |> decode.map(int.to_string)\n"
    " let result = decode.run(dynamic.from(1000), decoder)\n"
    " assert result == Ok(\"1000\")\n"
    " ```\n"
).
-spec map(decoder(DTA), fun((DTA) -> DTC)) -> decoder(DTC).
map(Decoder, Transformer) ->
    {decoder,
        fun(D) ->
            {Data, Errors} = (erlang:element(2, Decoder))(D),
            {Transformer(Data), Errors}
        end}.

-file("src/gleam/dynamic/decode.gleam", 871).
?DOC(" Apply a transformation function to any errors returned by the decoder.\n").
-spec map_errors(
    decoder(DTE),
    fun((list(decode_error())) -> list(decode_error()))
) -> decoder(DTE).
map_errors(Decoder, Transformer) ->
    {decoder,
        fun(D) ->
            {Data, Errors} = (erlang:element(2, Decoder))(D),
            {Data, Transformer(Errors)}
        end}.

-file("src/gleam/dynamic/decode.gleam", 909).
?DOC(
    " Create a new decoder based upon the value of a previous decoder.\n"
    "\n"
    " This may be useful to run one previous decoder to use in further decoding.\n"
).
-spec then(decoder(DTM), fun((DTM) -> decoder(DTO))) -> decoder(DTO).
then(Decoder, Next) ->
    {decoder,
        fun(Dynamic_data) ->
            {Data, Errors} = (erlang:element(2, Decoder))(Dynamic_data),
            Decoder@1 = Next(Data),
            {Data@1, _} = Layer = (erlang:element(2, Decoder@1))(Dynamic_data),
            case Errors of
                [] ->
                    Layer;

                [_ | _] ->
                    {Data@1, Errors}
            end
        end}.

-file("src/gleam/dynamic/decode.gleam", 952).
-spec run_decoders(
    gleam@dynamic:dynamic_(),
    {DTW, list(decode_error())},
    list(decoder(DTW))
) -> {DTW, list(decode_error())}.
run_decoders(Data, Failure, Decoders) ->
    case Decoders of
        [] ->
            Failure;

        [Decoder | Decoders@1] ->
            {_, Errors} = Layer = (erlang:element(2, Decoder))(Data),
            case Errors of
                [] ->
                    Layer;

                [_ | _] ->
                    run_decoders(Data, Failure, Decoders@1)
            end
    end.

-file("src/gleam/dynamic/decode.gleam", 939).
?DOC(
    " Create a new decoder from several other decoders. Each of the inner\n"
    " decoders is run in turn, and the value from the first to succeed is used.\n"
    "\n"
    " If no decoder succeeds then the errors from the first decoder is used.\n"
    " If you wish for different errors then you may wish to use the\n"
    " `collapse_errors` or `map_errors` functions.\n"
    "\n"
    " # Examples\n"
    "\n"
    " ```gleam\n"
    " let decoder = decode.one_of(decode.string, or: [\n"
    "   decode.int |> decode.map(int.to_string),\n"
    "   decode.float |> decode.map(float.to_string),\n"
    " ])\n"
    " decode.run(dynamic.from(1000), decoder)\n"
    " // -> Ok(\"1000\")\n"
    " ```\n"
).
-spec one_of(decoder(DTR), list(decoder(DTR))) -> decoder(DTR).
one_of(First, Alternatives) ->
    {decoder,
        fun(Dynamic_data) ->
            {_, Errors} = Layer = (erlang:element(2, First))(Dynamic_data),
            case Errors of
                [] ->
                    Layer;

                [_ | _] ->
                    run_decoders(Dynamic_data, Layer, Alternatives)
            end
        end}.

-file("src/gleam/dynamic/decode.gleam", 1035).
?DOC(
    " Create a decoder that can refer to itself, useful for decoding deeply\n"
    " nested data.\n"
    "\n"
    " Attempting to create a recursive decoder without this function could result\n"
    " in an infinite loop. If you are using `field` or other `use`able functions\n"
    " then you may not need to use this function.\n"
    "\n"
    " ```gleam\n"
    " type Nested {\n"
    "   Nested(List(Nested))\n"
    "   Value(String)\n"
    " }\n"
    "\n"
    " fn nested_decoder() -> decode.Decoder(Nested) {\n"
    "   use <- decode.recursive\n"
    "   decode.one_of(decode.string |> decode.map(Value), [\n"
    "     decode.list(nested_decoder()) |> decode.map(Nested),\n"
    "   ])\n"
    " }\n"
    " ```\n"
).
-spec recursive(fun(() -> decoder(DUH))) -> decoder(DUH).
recursive(Inner) ->
    {decoder,
        fun(Data) ->
            Decoder = Inner(),
            (erlang:element(2, Decoder))(Data)
        end}.

-file("src/gleam/dynamic/decode.gleam", 840).
?DOC(
    " A decoder that decodes nullable values of a type decoded by with a given\n"
    " decoder.\n"
    "\n"
    " This function can handle common representations of null on all runtimes, such as\n"
    " `nil`, `null`, and `undefined` on Erlang, and `undefined` and `null` on\n"
    " JavaScript.\n"
    "\n"
    " # Examples\n"
    "\n"
    " ```gleam\n"
    " let result = decode.run(dynamic.from(100), decode.optional(decode.int))\n"
    " assert result == Ok(option.Some(100))\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " let result = decode.run(dynamic.from(Nil), decode.optional(decode.int))\n"
    " assert result == Ok(option.None)\n"
    " ```\n"
).
-spec optional(decoder(DSW)) -> decoder(gleam@option:option(DSW)).
optional(Inner) ->
    {decoder, fun(Data) -> case gleam_stdlib_decode_ffi:is_null(Data) of
                true ->
                    {none, []};

                false ->
                    {Data@1, Errors} = (erlang:element(2, Inner))(Data),
                    {{some, Data@1}, Errors}
            end end}.

-file("src/gleam/dynamic/decode.gleam", 476).
?DOC(" Construct a decode error for some unexpected dynamic data.\n").
-spec decode_error(binary(), gleam@dynamic:dynamic_()) -> list(decode_error()).
decode_error(Expected, Found) ->
    [{decode_error, Expected, gleam_stdlib:classify_dynamic(Found), []}].

-file("src/gleam/dynamic/decode.gleam", 599).
-spec run_dynamic_function(
    gleam@dynamic:dynamic_(),
    binary(),
    fun((gleam@dynamic:dynamic_()) -> {ok, DQX} | {error, DQX})
) -> {DQX, list(decode_error())}.
run_dynamic_function(Data, Name, F) ->
    case F(Data) of
        {ok, Data@1} ->
            {Data@1, []};

        {error, Zero} ->
            {Zero,
                [{decode_error, Name, gleam_stdlib:classify_dynamic(Data), []}]}
    end.

-file("src/gleam/dynamic/decode.gleam", 648).
-spec decode_bool(gleam@dynamic:dynamic_()) -> {boolean(), list(decode_error())}.
decode_bool(Data) ->
    case gleam_stdlib:identity(true) =:= Data of
        true ->
            {true, []};

        false ->
            case gleam_stdlib:identity(false) =:= Data of
                true ->
                    {false, []};

                false ->
                    {false, decode_error(<<"Bool"/utf8>>, Data)}
            end
    end.

-file("src/gleam/dynamic/decode.gleam", 670).
-spec decode_int(gleam@dynamic:dynamic_()) -> {integer(), list(decode_error())}.
decode_int(Data) ->
    run_dynamic_function(
        Data,
        <<"Int"/utf8>>,
        fun gleam_stdlib_decode_ffi:int/1
    ).

-file("src/gleam/dynamic/decode.gleam", 689).
-spec decode_float(gleam@dynamic:dynamic_()) -> {float(), list(decode_error())}.
decode_float(Data) ->
    run_dynamic_function(
        Data,
        <<"Float"/utf8>>,
        fun gleam_stdlib_decode_ffi:float/1
    ).

-file("src/gleam/dynamic/decode.gleam", 723).
-spec decode_bit_array(gleam@dynamic:dynamic_()) -> {bitstring(),
    list(decode_error())}.
decode_bit_array(Data) ->
    run_dynamic_function(
        Data,
        <<"BitArray"/utf8>>,
        fun gleam_stdlib_decode_ffi:bit_array/1
    ).

-file("src/gleam/dynamic/decode.gleam", 895).
?DOC(
    " Replace all errors produced by a decoder with one single error for a named\n"
    " expected type.\n"
    "\n"
    " This function may be useful if you wish to simplify errors before\n"
    " presenting them to a user, particularly when using the `one_of` function.\n"
    "\n"
    " # Examples\n"
    "\n"
    " ```gleam\n"
    " let decoder = decode.string |> decode.collapse_errors(\"MyThing\")\n"
    " let result = decode.run(dynamic.from(1000), decoder)\n"
    " assert result == Error([DecodeError(\"MyThing\", \"Int\", [])])\n"
    " ```\n"
).
-spec collapse_errors(decoder(DTJ), binary()) -> decoder(DTJ).
collapse_errors(Decoder, Name) ->
    {decoder,
        fun(Dynamic_data) ->
            {Data, Errors} = Layer = (erlang:element(2, Decoder))(Dynamic_data),
            case Errors of
                [] ->
                    Layer;

                [_ | _] ->
                    {Data, decode_error(Name, Dynamic_data)}
            end
        end}.

-file("src/gleam/dynamic/decode.gleam", 973).
?DOC(
    " Define a decoder that always fails. The parameter for this function is the\n"
    " name of the type that has failed to decode.\n"
).
-spec failure(DUB, binary()) -> decoder(DUB).
failure(Zero, Expected) ->
    {decoder, fun(D) -> {Zero, decode_error(Expected, D)} end}.

-file("src/gleam/dynamic/decode.gleam", 1002).
?DOC(
    " Create a decoder for a new data type from a decoding function.\n"
    "\n"
    " This function is used for new primitive types. For example, you might\n"
    " define a decoder for Erlang's pid type.\n"
    "\n"
    " A default \"zero\" value is also required to make a decoder. When this\n"
    " decoder is used as part of a larger decoder this zero value used as\n"
    " a placeholder so that the rest of the decoder can continue to run and\n"
    " collect all decoding errors.\n"
    "\n"
    " If you were to make a decoder for the `String` type (rather than using the\n"
    " build-in `string` decoder) you would define it like so:\n"
    "\n"
    " ```gleam\n"
    " pub fn string_decoder() -> decode.Decoder(String) {\n"
    "   let default = \"\"\n"
    "   decode.new_primitive_decoder(\"String\", fn(data) {\n"
    "     case dynamic.string(data) {\n"
    "       Ok(x) -> Ok(x)\n"
    "       Error(_) -> Error(default)\n"
    "     }\n"
    "   })\n"
    " }\n"
    " ```\n"
).
-spec new_primitive_decoder(
    binary(),
    fun((gleam@dynamic:dynamic_()) -> {ok, DUD} | {error, DUD})
) -> decoder(DUD).
new_primitive_decoder(Name, Decoding_function) ->
    {decoder, fun(D) -> case Decoding_function(D) of
                {ok, T} ->
                    {T, []};

                {error, Zero} ->
                    {Zero,
                        [{decode_error,
                                Name,
                                gleam_stdlib:classify_dynamic(D),
                                []}]}
            end end}.

-file("src/gleam/dynamic/decode.gleam", 626).
-spec dynamic_string(gleam@dynamic:dynamic_()) -> {ok, binary()} |
    {error, binary()}.
dynamic_string(Data) ->
    case gleam_stdlib_decode_ffi:bit_array(Data) of
        {ok, Data@1} ->
            case gleam@bit_array:to_string(Data@1) of
                {ok, String} ->
                    {ok, String};

                {error, _} ->
                    {error, <<""/utf8>>}
            end;

        {error, _} ->
            {error, <<""/utf8>>}
    end.

-file("src/gleam/dynamic/decode.gleam", 621).
-spec decode_string(gleam@dynamic:dynamic_()) -> {binary(),
    list(decode_error())}.
decode_string(Data) ->
    run_dynamic_function(Data, <<"String"/utf8>>, fun dynamic_string/1).

-file("src/gleam/dynamic/decode.gleam", 794).
-spec fold_dict(
    {gleam@dict:dict(DSI, DSJ), list(decode_error())},
    gleam@dynamic:dynamic_(),
    gleam@dynamic:dynamic_(),
    fun((gleam@dynamic:dynamic_()) -> {DSI, list(decode_error())}),
    fun((gleam@dynamic:dynamic_()) -> {DSJ, list(decode_error())})
) -> {gleam@dict:dict(DSI, DSJ), list(decode_error())}.
fold_dict(Acc, Key, Value, Key_decoder, Value_decoder) ->
    case Key_decoder(Key) of
        {Key@1, []} ->
            case Value_decoder(Value) of
                {Value@1, []} ->
                    Dict = gleam@dict:insert(
                        erlang:element(1, Acc),
                        Key@1,
                        Value@1
                    ),
                    {Dict, erlang:element(2, Acc)};

                {_, Errors} ->
                    push_path({maps:new(), Errors}, [<<"values"/utf8>>])
            end;

        {_, Errors@1} ->
            push_path({maps:new(), Errors@1}, [<<"keys"/utf8>>])
    end.

-file("src/gleam/dynamic/decode.gleam", 774).
?DOC(
    " A decoder that decodes dicts where all keys and vales are decoded with\n"
    " given decoders.\n"
    "\n"
    " # Examples\n"
    "\n"
    " ```gleam\n"
    " let values = dict.from_list([\n"
    "   #(\"one\", 1),\n"
    "   #(\"two\", 2),\n"
    " ])\n"
    "\n"
    " let result =\n"
    "   decode.run(dynamic.from(values), decode.dict(decode.string, decode.int))\n"
    " assert result == Ok(values)\n"
    " ```\n"
).
-spec dict(decoder(DSB), decoder(DSD)) -> decoder(gleam@dict:dict(DSB, DSD)).
dict(Key, Value) ->
    {decoder, fun(Data) -> case gleam_stdlib_decode_ffi:dict(Data) of
                {error, _} ->
                    {maps:new(), decode_error(<<"Dict"/utf8>>, Data)};

                {ok, Dict} ->
                    gleam@dict:fold(
                        Dict,
                        {maps:new(), []},
                        fun(A, K, V) -> case erlang:element(2, A) of
                                [] ->
                                    fold_dict(
                                        A,
                                        K,
                                        V,
                                        erlang:element(2, Key),
                                        erlang:element(2, Value)
                                    );

                                [_ | _] ->
                                    A
                            end end
                    )
            end end}.

-file("src/gleam/dynamic/decode.gleam", 742).
?DOC(
    " A decoder that decodes lists where all elements are decoded with a given\n"
    " decoder.\n"
    "\n"
    " # Examples\n"
    "\n"
    " ```gleam\n"
    " let result =\n"
    "   decode.run(dynamic.from([1, 2, 3]), decode.list(of: decode.int))\n"
    " assert result == Ok([1, 2, 3])\n"
    " ```\n"
).
-spec list(decoder(DRP)) -> decoder(list(DRP)).
list(Inner) ->
    {decoder,
        fun(Data) ->
            gleam_stdlib_decode_ffi:list(
                Data,
                erlang:element(2, Inner),
                fun(P, K) -> push_path(P, [K]) end,
                0,
                []
            )
        end}.

-file("src/gleam/dynamic/decode.gleam", 430).
-spec push_path({DPY, list(decode_error())}, list(any())) -> {DPY,
    list(decode_error())}.
push_path(Layer, Path) ->
    Decoder = one_of(
        {decoder, fun decode_string/1},
        [begin
                _pipe = {decoder, fun decode_int/1},
                map(_pipe, fun erlang:integer_to_binary/1)
            end]
    ),
    Path@1 = gleam@list:map(
        Path,
        fun(Key) ->
            Key@1 = gleam_stdlib:identity(Key),
            case run(Key@1, Decoder) of
                {ok, Key@2} ->
                    Key@2;

                {error, _} ->
                    <<<<"<"/utf8,
                            (gleam_stdlib:classify_dynamic(Key@1))/binary>>/binary,
                        ">"/utf8>>
            end
        end
    ),
    Errors = gleam@list:map(
        erlang:element(2, Layer),
        fun(Error) -> _record = Error,
            {decode_error,
                erlang:element(2, _record),
                erlang:element(3, _record),
                lists:append(Path@1, erlang:element(4, Error))} end
    ),
    {erlang:element(1, Layer), Errors}.

-file("src/gleam/dynamic/decode.gleam", 395).
-spec index(
    list(DPM),
    list(DPM),
    fun((gleam@dynamic:dynamic_()) -> {DPP, list(decode_error())}),
    gleam@dynamic:dynamic_(),
    fun((gleam@dynamic:dynamic_(), list(DPM)) -> {DPP, list(decode_error())})
) -> {DPP, list(decode_error())}.
index(Path, Position, Inner, Data, Handle_miss) ->
    case Path of
        [] ->
            _pipe = Inner(Data),
            push_path(_pipe, lists:reverse(Position));

        [Key | Path@1] ->
            case gleam_stdlib_decode_ffi:index(Data, Key) of
                {ok, {some, Data@1}} ->
                    index(Path@1, [Key | Position], Inner, Data@1, Handle_miss);

                {ok, none} ->
                    Handle_miss(Data, [Key | Position]);

                {error, Kind} ->
                    {Default, _} = Inner(Data),
                    _pipe@1 = {Default,
                        [{decode_error,
                                Kind,
                                gleam_stdlib:classify_dynamic(Data),
                                []}]},
                    push_path(_pipe@1, lists:reverse(Position))
            end
    end.

-file("src/gleam/dynamic/decode.gleam", 316).
?DOC(
    " The same as [`field`](#field), except taking a path to the value rather\n"
    " than a field name.\n"
    "\n"
    " This function will index into dictionaries with any key type, and if the key is\n"
    " an int then it'll also index into Erlang tuples and JavaScript arrays, and\n"
    " the first eight elements of Gleam lists.\n"
    "\n"
    " # Examples\n"
    "\n"
    " ```gleam\n"
    " let data = dynamic.from(dict.from_list([\n"
    "   #(\"data\", dict.from_list([\n"
    "     #(\"email\", \"lucy@example.com\"),\n"
    "     #(\"name\", \"Lucy\"),\n"
    "   ]))\n"
    " ]))\n"
    "\n"
    " let decoder = {\n"
    "   use name <- decode.subfield([\"data\", \"name\"], decode.string)\n"
    "   use email <- decode.subfield([\"data\", \"email\"], decode.string)\n"
    "   decode.success(SignUp(name: name, email: email))\n"
    " }\n"
    " let result = decode.run(data, decoder)\n"
    " assert result == Ok(SignUp(name: \"Lucy\", email: \"lucy@example.com\"))\n"
    " ```\n"
).
-spec subfield(list(any()), decoder(DOX), fun((DOX) -> decoder(DOZ))) -> decoder(DOZ).
subfield(Field_path, Field_decoder, Next) ->
    {decoder,
        fun(Data) ->
            {Out, Errors1} = index(
                Field_path,
                [],
                erlang:element(2, Field_decoder),
                Data,
                fun(Data@1, Position) ->
                    {Default, _} = (erlang:element(2, Field_decoder))(Data@1),
                    _pipe = {Default,
                        [{decode_error,
                                <<"Field"/utf8>>,
                                <<"Nothing"/utf8>>,
                                []}]},
                    push_path(_pipe, lists:reverse(Position))
                end
            ),
            {Out@1, Errors2} = (erlang:element(2, Next(Out)))(Data),
            {Out@1, lists:append(Errors1, Errors2)}
        end}.

-file("src/gleam/dynamic/decode.gleam", 385).
?DOC(
    " A decoder that decodes a value that is nested within other values. For\n"
    " example, decoding a value that is within some deeply nested JSON objects.\n"
    "\n"
    " This function will index into dictionaries with any key type, and if the key is\n"
    " an int then it'll also index into Erlang tuples and JavaScript arrays, and\n"
    " the first eight elements of Gleam lists.\n"
    "\n"
    " # Examples\n"
    "\n"
    " ```gleam\n"
    " let decoder = decode.at([\"one\", \"two\"], decode.int)\n"
    "\n"
    " let data = dynamic.from(dict.from_list([\n"
    "   #(\"one\", dict.from_list([\n"
    "     #(\"two\", 1000),\n"
    "   ])),\n"
    " ]))\n"
    "\n"
    "\n"
    " decode.run(data, decoder)\n"
    " // -> Ok(1000)\n"
    " ```\n"
    "\n"
    " ```gleam\n"
    " dynamic.from(Nil)\n"
    " |> decode.run(decode.optional(decode.int))\n"
    " // -> Ok(option.None)\n"
    " ```\n"
).
-spec at(list(any()), decoder(DPJ)) -> decoder(DPJ).
at(Path, Inner) ->
    {decoder,
        fun(Data) ->
            index(
                Path,
                [],
                erlang:element(2, Inner),
                Data,
                fun(Data@1, Position) ->
                    {Default, _} = (erlang:element(2, Inner))(Data@1),
                    _pipe = {Default,
                        [{decode_error,
                                <<"Field"/utf8>>,
                                <<"Nothing"/utf8>>,
                                []}]},
                    push_path(_pipe, lists:reverse(Position))
                end
            )
        end}.

-file("src/gleam/dynamic/decode.gleam", 515).
?DOC(
    " Run a decoder on a field of a `Dynamic` value, decoding the value if it is\n"
    " of the desired type, or returning errors. An error is returned if there is\n"
    " no field for the specified key.\n"
    "\n"
    " This function will index into dictionaries with any key type, and if the key is\n"
    " an int then it'll also index into Erlang tuples and JavaScript arrays, and\n"
    " the first eight elements of Gleam lists.\n"
    "\n"
    " # Examples\n"
    "\n"
    " ```gleam\n"
    " let data = dynamic.from(dict.from_list([\n"
    "   #(\"email\", \"lucy@example.com\"),\n"
    "   #(\"name\", \"Lucy\"),\n"
    " ]))\n"
    "\n"
    " let decoder = {\n"
    "   use name <- decode.field(\"name\", string)\n"
    "   use email <- decode.field(\"email\", string)\n"
    "   decode.success(SignUp(name: name, email: email))\n"
    " }\n"
    "\n"
    " let result = decode.run(data, decoder)\n"
    " assert result == Ok(SignUp(name: \"Lucy\", email: \"lucy@example.com\"))\n"
    " ```\n"
    "\n"
    " If you wish to decode a value that is more deeply nested within the dynamic\n"
    " data, see [`subfield`](#subfield) and [`at`](#at).\n"
    "\n"
    " If you wish to return a default in the event that a field is not present,\n"
    " see [`optional_field`](#optional_field) and / [`optionally_at`](#optionally_at).\n"
).
-spec field(any(), decoder(DQH), fun((DQH) -> decoder(DQJ))) -> decoder(DQJ).
field(Field_name, Field_decoder, Next) ->
    subfield([Field_name], Field_decoder, Next).

-file("src/gleam/dynamic/decode.gleam", 548).
?DOC(
    " Run a decoder on a field of a `Dynamic` value, decoding the value if it is\n"
    " of the desired type, or returning errors. The given default value is\n"
    " returned if there is no field for the specified key.\n"
    "\n"
    " This function will index into dictionaries with any key type, and if the key is\n"
    " an int then it'll also index into Erlang tuples and JavaScript arrays, and\n"
    " the first eight elements of Gleam lists.\n"
    "\n"
    " # Examples\n"
    "\n"
    " ```gleam\n"
    " let data = dynamic.from(dict.from_list([\n"
    "   #(\"name\", \"Lucy\"),\n"
    " ]))\n"
    "\n"
    " let decoder = {\n"
    "   use name <- decode.field(\"name\", string)\n"
    "   use email <- decode.optional_field(\"email\", \"n/a\", string)\n"
    "   decode.success(SignUp(name: name, email: email))\n"
    " }\n"
    "\n"
    " let result = decode.run(data, decoder)\n"
    " assert result == Ok(SignUp(name: \"Lucy\", email: \"n/a\"))\n"
    " ```\n"
).
-spec optional_field(any(), DQN, decoder(DQN), fun((DQN) -> decoder(DQP))) -> decoder(DQP).
optional_field(Key, Default, Field_decoder, Next) ->
    {decoder,
        fun(Data) ->
            {Out, Errors1} = case gleam_stdlib_decode_ffi:index(Data, Key) of
                {ok, {some, Data@1}} ->
                    (erlang:element(2, Field_decoder))(Data@1);

                {ok, none} ->
                    {Default, []};

                {error, Kind} ->
                    _pipe = {Default,
                        [{decode_error,
                                Kind,
                                gleam_stdlib:classify_dynamic(Data),
                                []}]},
                    push_path(_pipe, [Key])
            end,
            {Out@1, Errors2} = (erlang:element(2, Next(Out)))(Data),
            {Out@1, lists:append(Errors1, Errors2)}
        end}.

-file("src/gleam/dynamic/decode.gleam", 589).
?DOC(
    " A decoder that decodes a value that is nested within other values. For\n"
    " example, decoding a value that is within some deeply nested JSON objects.\n"
    "\n"
    " This function will index into dictionaries with any key type, and if the key is\n"
    " an int then it'll also index into Erlang tuples and JavaScript arrays, and\n"
    " the first eight elements of Gleam lists.\n"
    "\n"
    " # Examples\n"
    "\n"
    " ```gleam\n"
    " let decoder = decode.optionally_at([\"one\", \"two\"], 100, decode.int)\n"
    "\n"
    " let data = dynamic.from(dict.from_list([\n"
    "   #(\"one\", dict.from_list([])),\n"
    " ]))\n"
    "\n"
    "\n"
    " decode.run(data, decoder)\n"
    " // -> Ok(100)\n"
    " ```\n"
).
-spec optionally_at(list(any()), DQU, decoder(DQU)) -> decoder(DQU).
optionally_at(Path, Default, Inner) ->
    {decoder,
        fun(Data) ->
            index(
                Path,
                [],
                erlang:element(2, Inner),
                Data,
                fun(_, _) -> {Default, []} end
            )
        end}.
