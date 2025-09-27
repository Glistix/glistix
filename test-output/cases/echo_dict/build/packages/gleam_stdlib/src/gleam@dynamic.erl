-module(gleam@dynamic).
-compile([no_auto_import, nowarn_unused_vars, nowarn_unused_function, nowarn_nomatch]).

-export([classify/1, from/1, dynamic/1, bit_array/1, string/1, int/1, float/1, bool/1, shallow_list/1, optional/1, result/2, list/1, field/2, optional_field/2, element/2, tuple2/2, tuple3/3, tuple4/4, tuple5/5, tuple6/6, dict/2, any/1, decode1/2, decode2/3, decode3/4, decode4/5, decode5/6, decode6/7, decode7/8, decode8/9, decode9/10]).
-export_type([dynamic_/0, decode_error/0, unknown_tuple/0]).

-if(?OTP_RELEASE >= 27).
-define(MODULEDOC(Str), -moduledoc(Str)).
-define(DOC(Str), -doc(Str)).
-else.
-define(MODULEDOC(Str), -compile([])).
-define(DOC(Str), -compile([])).
-endif.

-type dynamic_() :: any().

-type decode_error() :: {decode_error, binary(), binary(), list(binary())}.

-type unknown_tuple() :: any().

-file("src/gleam/dynamic.gleam", 28).
?DOC(
    " Return a string indicating the type of the dynamic value.\n"
    "\n"
    " This function may be useful for constructing error messages or logs. If you\n"
    " want to turn dynamic data into well typed data then you want the\n"
    " `gleam/dynamic/decode` module.\n"
    "\n"
    " ```gleam\n"
    " classify(from(\"Hello\"))\n"
    " // -> \"String\"\n"
    " ```\n"
).
-spec classify(dynamic_()) -> binary().
classify(Data) ->
    gleam_stdlib:classify_dynamic(Data).

-file("src/gleam/dynamic.gleam", 44).
?DOC(" Converts any Gleam data into `Dynamic` data.\n").
-spec from(any()) -> dynamic_().
from(A) ->
    gleam_stdlib:identity(A).

-file("src/gleam/dynamic.gleam", 47).
-spec dynamic(dynamic_()) -> {ok, dynamic_()} | {error, list(decode_error())}.
dynamic(Value) ->
    {ok, Value}.

-file("src/gleam/dynamic.gleam", 52).
-spec bit_array(dynamic_()) -> {ok, bitstring()} | {error, list(decode_error())}.
bit_array(Data) ->
    gleam_stdlib:decode_bit_array(Data).

-file("src/gleam/dynamic.gleam", 78).
-spec map_errors(
    {ok, CIV} | {error, list(decode_error())},
    fun((decode_error()) -> decode_error())
) -> {ok, CIV} | {error, list(decode_error())}.
map_errors(Result, F) ->
    gleam@result:map_error(
        Result,
        fun(_capture) -> gleam@list:map(_capture, F) end
    ).

-file("src/gleam/dynamic.gleam", 85).
-spec put_expected(decode_error(), binary()) -> decode_error().
put_expected(Error, Expected) ->
    _record = Error,
    {decode_error,
        Expected,
        erlang:element(3, _record),
        erlang:element(4, _record)}.

-file("src/gleam/dynamic.gleam", 66).
-spec decode_string(dynamic_()) -> {ok, binary()} |
    {error, list(decode_error())}.
decode_string(Data) ->
    _pipe = gleam_stdlib:decode_bit_array(Data),
    _pipe@1 = map_errors(
        _pipe,
        fun(_capture) -> put_expected(_capture, <<"String"/utf8>>) end
    ),
    gleam@result:'try'(
        _pipe@1,
        fun(Raw) -> case gleam@bit_array:to_string(Raw) of
                {ok, String} ->
                    {ok, String};

                {error, nil} ->
                    {error,
                        [{decode_error,
                                <<"String"/utf8>>,
                                <<"BitArray"/utf8>>,
                                []}]}
            end end
    ).

-file("src/gleam/dynamic.gleam", 61).
-spec string(dynamic_()) -> {ok, binary()} | {error, list(decode_error())}.
string(Data) ->
    decode_string(Data).

-file("src/gleam/dynamic.gleam", 90).
-spec int(dynamic_()) -> {ok, integer()} | {error, list(decode_error())}.
int(Data) ->
    gleam_stdlib:decode_int(Data).

-file("src/gleam/dynamic.gleam", 99).
-spec float(dynamic_()) -> {ok, float()} | {error, list(decode_error())}.
float(Data) ->
    gleam_stdlib:decode_float(Data).

-file("src/gleam/dynamic.gleam", 108).
-spec bool(dynamic_()) -> {ok, boolean()} | {error, list(decode_error())}.
bool(Data) ->
    gleam_stdlib:decode_bool(Data).

-file("src/gleam/dynamic.gleam", 117).
-spec shallow_list(dynamic_()) -> {ok, list(dynamic_())} |
    {error, list(decode_error())}.
shallow_list(Value) ->
    gleam_stdlib:decode_list(Value).

-file("src/gleam/dynamic.gleam", 169).
-spec optional(fun((dynamic_()) -> {ok, CKK} | {error, list(decode_error())})) -> fun((dynamic_()) -> {ok,
        gleam@option:option(CKK)} |
    {error, list(decode_error())}).
optional(Decode) ->
    fun(Value) -> gleam_stdlib:decode_option(Value, Decode) end.

-file("src/gleam/dynamic.gleam", 235).
-spec at_least_decode_tuple_error(integer(), dynamic_()) -> {ok, any()} |
    {error, list(decode_error())}.
at_least_decode_tuple_error(Size, Data) ->
    S = case Size of
        1 ->
            <<""/utf8>>;

        _ ->
            <<"s"/utf8>>
    end,
    Error = begin
        _pipe = [<<"Tuple of at least "/utf8>>,
            erlang:integer_to_binary(Size),
            <<" element"/utf8>>,
            S],
        _pipe@1 = gleam_stdlib:identity(_pipe),
        _pipe@2 = unicode:characters_to_binary(_pipe@1),
        {decode_error, _pipe@2, gleam_stdlib:classify_dynamic(Data), []}
    end,
    {error, [Error]}.

-file("src/gleam/dynamic.gleam", 472).
-spec do_any(
    list(fun((dynamic_()) -> {ok, COO} | {error, list(decode_error())}))
) -> fun((dynamic_()) -> {ok, COO} | {error, list(decode_error())}).
do_any(Decoders) ->
    fun(Data) -> case Decoders of
            [] ->
                {error,
                    [{decode_error,
                            <<"another type"/utf8>>,
                            gleam_stdlib:classify_dynamic(Data),
                            []}]};

            [Decoder | Decoders@1] ->
                case Decoder(Data) of
                    {ok, Decoded} ->
                        {ok, Decoded};

                    {error, _} ->
                        (do_any(Decoders@1))(Data)
                end
        end end.

-file("src/gleam/dynamic.gleam", 307).
-spec push_path(decode_error(), any()) -> decode_error().
push_path(Error, Name) ->
    Name@1 = gleam_stdlib:identity(Name),
    Decoder = do_any(
        [fun decode_string/1,
            fun(X) ->
                gleam@result:map(
                    gleam_stdlib:decode_int(X),
                    fun erlang:integer_to_binary/1
                )
            end]
    ),
    Name@3 = case Decoder(Name@1) of
        {ok, Name@2} ->
            Name@2;

        {error, _} ->
            _pipe = [<<"<"/utf8>>,
                gleam_stdlib:classify_dynamic(Name@1),
                <<">"/utf8>>],
            _pipe@1 = gleam_stdlib:identity(_pipe),
            unicode:characters_to_binary(_pipe@1)
    end,
    _record = Error,
    {decode_error,
        erlang:element(2, _record),
        erlang:element(3, _record),
        [Name@3 | erlang:element(4, Error)]}.

-file("src/gleam/dynamic.gleam", 126).
-spec result(
    fun((dynamic_()) -> {ok, CJS} | {error, list(decode_error())}),
    fun((dynamic_()) -> {ok, CJU} | {error, list(decode_error())})
) -> fun((dynamic_()) -> {ok, {ok, CJS} | {error, CJU}} |
    {error, list(decode_error())}).
result(Decode_ok, Decode_error) ->
    fun(Value) ->
        gleam@result:'try'(
            gleam_stdlib:decode_result(Value),
            fun(Inner_result) -> case Inner_result of
                    {ok, Raw} ->
                        gleam@result:'try'(
                            begin
                                _pipe = Decode_ok(Raw),
                                map_errors(
                                    _pipe,
                                    fun(_capture) ->
                                        push_path(_capture, <<"ok"/utf8>>)
                                    end
                                )
                            end,
                            fun(Value@1) -> {ok, {ok, Value@1}} end
                        );

                    {error, Raw@1} ->
                        gleam@result:'try'(
                            begin
                                _pipe@1 = Decode_error(Raw@1),
                                map_errors(
                                    _pipe@1,
                                    fun(_capture@1) ->
                                        push_path(_capture@1, <<"error"/utf8>>)
                                    end
                                )
                            end,
                            fun(Value@2) -> {ok, {error, Value@2}} end
                        )
                end end
        )
    end.

-file("src/gleam/dynamic.gleam", 157).
-spec list(fun((dynamic_()) -> {ok, CKF} | {error, list(decode_error())})) -> fun((dynamic_()) -> {ok,
        list(CKF)} |
    {error, list(decode_error())}).
list(Decoder_type) ->
    fun(Dynamic) ->
        gleam@result:'try'(
            gleam_stdlib:decode_list(Dynamic),
            fun(List) -> _pipe = List,
                _pipe@1 = gleam@list:try_map(_pipe, Decoder_type),
                map_errors(
                    _pipe@1,
                    fun(_capture) -> push_path(_capture, <<"*"/utf8>>) end
                ) end
        )
    end.

-file("src/gleam/dynamic.gleam", 178).
-spec field(
    any(),
    fun((dynamic_()) -> {ok, CKU} | {error, list(decode_error())})
) -> fun((dynamic_()) -> {ok, CKU} | {error, list(decode_error())}).
field(Name, Inner_type) ->
    fun(Value) ->
        Missing_field_error = {decode_error,
            <<"field"/utf8>>,
            <<"nothing"/utf8>>,
            []},
        gleam@result:'try'(
            gleam_stdlib:decode_field(Value, Name),
            fun(Maybe_inner) -> _pipe = Maybe_inner,
                _pipe@1 = gleam@option:to_result(_pipe, [Missing_field_error]),
                _pipe@2 = gleam@result:'try'(_pipe@1, Inner_type),
                map_errors(
                    _pipe@2,
                    fun(_capture) -> push_path(_capture, Name) end
                ) end
        )
    end.

-file("src/gleam/dynamic.gleam", 192).
-spec optional_field(
    any(),
    fun((dynamic_()) -> {ok, CKY} | {error, list(decode_error())})
) -> fun((dynamic_()) -> {ok, gleam@option:option(CKY)} |
    {error, list(decode_error())}).
optional_field(Name, Inner_type) ->
    fun(Value) ->
        gleam@result:'try'(
            gleam_stdlib:decode_field(Value, Name),
            fun(Maybe_inner) -> case Maybe_inner of
                    none ->
                        {ok, none};

                    {some, Dynamic_inner} ->
                        _pipe = Inner_type(Dynamic_inner),
                        _pipe@1 = gleam@result:map(
                            _pipe,
                            fun(Field@0) -> {some, Field@0} end
                        ),
                        map_errors(
                            _pipe@1,
                            fun(_capture) -> push_path(_capture, Name) end
                        )
                end end
        )
    end.

-file("src/gleam/dynamic.gleam", 213).
-spec element(
    integer(),
    fun((dynamic_()) -> {ok, CLG} | {error, list(decode_error())})
) -> fun((dynamic_()) -> {ok, CLG} | {error, list(decode_error())}).
element(Index, Inner_type) ->
    fun(Data) ->
        gleam@result:'try'(
            gleam_stdlib:decode_tuple(Data),
            fun(Tuple) ->
                Size = gleam_stdlib:size_of_tuple(Tuple),
                gleam@result:'try'(case Index >= 0 of
                        true ->
                            case Index < Size of
                                true ->
                                    gleam_stdlib:tuple_get(Tuple, Index);

                                false ->
                                    at_least_decode_tuple_error(Index + 1, Data)
                            end;

                        false ->
                            case gleam@int:absolute_value(Index) =< Size of
                                true ->
                                    gleam_stdlib:tuple_get(Tuple, Size + Index);

                                false ->
                                    at_least_decode_tuple_error(
                                        gleam@int:absolute_value(Index),
                                        Data
                                    )
                            end
                    end, fun(Data@1) -> _pipe = Inner_type(Data@1),
                        map_errors(
                            _pipe,
                            fun(_capture) -> push_path(_capture, Index) end
                        ) end)
            end
        )
    end.

-file("src/gleam/dynamic.gleam", 297).
-spec tuple_errors({ok, any()} | {error, list(decode_error())}, binary()) -> list(decode_error()).
tuple_errors(Result, Name) ->
    case Result of
        {ok, _} ->
            [];

        {error, Errors} ->
            gleam@list:map(
                Errors,
                fun(_capture) -> push_path(_capture, Name) end
            )
    end.

-file("src/gleam/dynamic.gleam", 325).
-spec tuple2(
    fun((dynamic_()) -> {ok, CMG} | {error, list(decode_error())}),
    fun((dynamic_()) -> {ok, CMI} | {error, list(decode_error())})
) -> fun((dynamic_()) -> {ok, {CMG, CMI}} | {error, list(decode_error())}).
tuple2(Decode1, Decode2) ->
    fun(Value) ->
        gleam@result:'try'(
            gleam_stdlib:decode_tuple2(Value),
            fun(_use0) ->
                {A, B} = _use0,
                case {Decode1(A), Decode2(B)} of
                    {{ok, A@1}, {ok, B@1}} ->
                        {ok, {A@1, B@1}};

                    {A@2, B@2} ->
                        _pipe = tuple_errors(A@2, <<"0"/utf8>>),
                        _pipe@1 = lists:append(
                            _pipe,
                            tuple_errors(B@2, <<"1"/utf8>>)
                        ),
                        {error, _pipe@1}
                end
            end
        )
    end.

-file("src/gleam/dynamic.gleam", 342).
-spec tuple3(
    fun((dynamic_()) -> {ok, CML} | {error, list(decode_error())}),
    fun((dynamic_()) -> {ok, CMN} | {error, list(decode_error())}),
    fun((dynamic_()) -> {ok, CMP} | {error, list(decode_error())})
) -> fun((dynamic_()) -> {ok, {CML, CMN, CMP}} | {error, list(decode_error())}).
tuple3(Decode1, Decode2, Decode3) ->
    fun(Value) ->
        gleam@result:'try'(
            gleam_stdlib:decode_tuple3(Value),
            fun(_use0) ->
                {A, B, C} = _use0,
                case {Decode1(A), Decode2(B), Decode3(C)} of
                    {{ok, A@1}, {ok, B@1}, {ok, C@1}} ->
                        {ok, {A@1, B@1, C@1}};

                    {A@2, B@2, C@2} ->
                        _pipe = tuple_errors(A@2, <<"0"/utf8>>),
                        _pipe@1 = lists:append(
                            _pipe,
                            tuple_errors(B@2, <<"1"/utf8>>)
                        ),
                        _pipe@2 = lists:append(
                            _pipe@1,
                            tuple_errors(C@2, <<"2"/utf8>>)
                        ),
                        {error, _pipe@2}
                end
            end
        )
    end.

-file("src/gleam/dynamic.gleam", 361).
-spec tuple4(
    fun((dynamic_()) -> {ok, CMS} | {error, list(decode_error())}),
    fun((dynamic_()) -> {ok, CMU} | {error, list(decode_error())}),
    fun((dynamic_()) -> {ok, CMW} | {error, list(decode_error())}),
    fun((dynamic_()) -> {ok, CMY} | {error, list(decode_error())})
) -> fun((dynamic_()) -> {ok, {CMS, CMU, CMW, CMY}} |
    {error, list(decode_error())}).
tuple4(Decode1, Decode2, Decode3, Decode4) ->
    fun(Value) ->
        gleam@result:'try'(
            gleam_stdlib:decode_tuple4(Value),
            fun(_use0) ->
                {A, B, C, D} = _use0,
                case {Decode1(A), Decode2(B), Decode3(C), Decode4(D)} of
                    {{ok, A@1}, {ok, B@1}, {ok, C@1}, {ok, D@1}} ->
                        {ok, {A@1, B@1, C@1, D@1}};

                    {A@2, B@2, C@2, D@2} ->
                        _pipe = tuple_errors(A@2, <<"0"/utf8>>),
                        _pipe@1 = lists:append(
                            _pipe,
                            tuple_errors(B@2, <<"1"/utf8>>)
                        ),
                        _pipe@2 = lists:append(
                            _pipe@1,
                            tuple_errors(C@2, <<"2"/utf8>>)
                        ),
                        _pipe@3 = lists:append(
                            _pipe@2,
                            tuple_errors(D@2, <<"3"/utf8>>)
                        ),
                        {error, _pipe@3}
                end
            end
        )
    end.

-file("src/gleam/dynamic.gleam", 382).
-spec tuple5(
    fun((dynamic_()) -> {ok, CNB} | {error, list(decode_error())}),
    fun((dynamic_()) -> {ok, CND} | {error, list(decode_error())}),
    fun((dynamic_()) -> {ok, CNF} | {error, list(decode_error())}),
    fun((dynamic_()) -> {ok, CNH} | {error, list(decode_error())}),
    fun((dynamic_()) -> {ok, CNJ} | {error, list(decode_error())})
) -> fun((dynamic_()) -> {ok, {CNB, CND, CNF, CNH, CNJ}} |
    {error, list(decode_error())}).
tuple5(Decode1, Decode2, Decode3, Decode4, Decode5) ->
    fun(Value) ->
        gleam@result:'try'(
            gleam_stdlib:decode_tuple5(Value),
            fun(_use0) ->
                {A, B, C, D, E} = _use0,
                case {Decode1(A),
                    Decode2(B),
                    Decode3(C),
                    Decode4(D),
                    Decode5(E)} of
                    {{ok, A@1}, {ok, B@1}, {ok, C@1}, {ok, D@1}, {ok, E@1}} ->
                        {ok, {A@1, B@1, C@1, D@1, E@1}};

                    {A@2, B@2, C@2, D@2, E@2} ->
                        _pipe = tuple_errors(A@2, <<"0"/utf8>>),
                        _pipe@1 = lists:append(
                            _pipe,
                            tuple_errors(B@2, <<"1"/utf8>>)
                        ),
                        _pipe@2 = lists:append(
                            _pipe@1,
                            tuple_errors(C@2, <<"2"/utf8>>)
                        ),
                        _pipe@3 = lists:append(
                            _pipe@2,
                            tuple_errors(D@2, <<"3"/utf8>>)
                        ),
                        _pipe@4 = lists:append(
                            _pipe@3,
                            tuple_errors(E@2, <<"4"/utf8>>)
                        ),
                        {error, _pipe@4}
                end
            end
        )
    end.

-file("src/gleam/dynamic.gleam", 405).
-spec tuple6(
    fun((dynamic_()) -> {ok, CNM} | {error, list(decode_error())}),
    fun((dynamic_()) -> {ok, CNO} | {error, list(decode_error())}),
    fun((dynamic_()) -> {ok, CNQ} | {error, list(decode_error())}),
    fun((dynamic_()) -> {ok, CNS} | {error, list(decode_error())}),
    fun((dynamic_()) -> {ok, CNU} | {error, list(decode_error())}),
    fun((dynamic_()) -> {ok, CNW} | {error, list(decode_error())})
) -> fun((dynamic_()) -> {ok, {CNM, CNO, CNQ, CNS, CNU, CNW}} |
    {error, list(decode_error())}).
tuple6(Decode1, Decode2, Decode3, Decode4, Decode5, Decode6) ->
    fun(Value) ->
        gleam@result:'try'(
            gleam_stdlib:decode_tuple6(Value),
            fun(_use0) ->
                {A, B, C, D, E, F} = _use0,
                case {Decode1(A),
                    Decode2(B),
                    Decode3(C),
                    Decode4(D),
                    Decode5(E),
                    Decode6(F)} of
                    {{ok, A@1},
                        {ok, B@1},
                        {ok, C@1},
                        {ok, D@1},
                        {ok, E@1},
                        {ok, F@1}} ->
                        {ok, {A@1, B@1, C@1, D@1, E@1, F@1}};

                    {A@2, B@2, C@2, D@2, E@2, F@2} ->
                        _pipe = tuple_errors(A@2, <<"0"/utf8>>),
                        _pipe@1 = lists:append(
                            _pipe,
                            tuple_errors(B@2, <<"1"/utf8>>)
                        ),
                        _pipe@2 = lists:append(
                            _pipe@1,
                            tuple_errors(C@2, <<"2"/utf8>>)
                        ),
                        _pipe@3 = lists:append(
                            _pipe@2,
                            tuple_errors(D@2, <<"3"/utf8>>)
                        ),
                        _pipe@4 = lists:append(
                            _pipe@3,
                            tuple_errors(E@2, <<"4"/utf8>>)
                        ),
                        _pipe@5 = lists:append(
                            _pipe@4,
                            tuple_errors(F@2, <<"5"/utf8>>)
                        ),
                        {error, _pipe@5}
                end
            end
        )
    end.

-file("src/gleam/dynamic.gleam", 437).
-spec dict(
    fun((dynamic_()) -> {ok, CNZ} | {error, list(decode_error())}),
    fun((dynamic_()) -> {ok, COB} | {error, list(decode_error())})
) -> fun((dynamic_()) -> {ok, gleam@dict:dict(CNZ, COB)} |
    {error, list(decode_error())}).
dict(Key_type, Value_type) ->
    fun(Value) ->
        gleam@result:'try'(
            gleam_stdlib:decode_map(Value),
            fun(Dict) ->
                gleam@result:'try'(
                    begin
                        _pipe = Dict,
                        _pipe@1 = maps:to_list(_pipe),
                        gleam@list:try_map(
                            _pipe@1,
                            fun(Pair) ->
                                {K, V} = Pair,
                                gleam@result:'try'(
                                    begin
                                        _pipe@2 = Key_type(K),
                                        map_errors(
                                            _pipe@2,
                                            fun(_capture) ->
                                                push_path(
                                                    _capture,
                                                    <<"keys"/utf8>>
                                                )
                                            end
                                        )
                                    end,
                                    fun(K@1) ->
                                        gleam@result:'try'(
                                            begin
                                                _pipe@3 = Value_type(V),
                                                map_errors(
                                                    _pipe@3,
                                                    fun(_capture@1) ->
                                                        push_path(
                                                            _capture@1,
                                                            <<"values"/utf8>>
                                                        )
                                                    end
                                                )
                                            end,
                                            fun(V@1) -> {ok, {K@1, V@1}} end
                                        )
                                    end
                                )
                            end
                        )
                    end,
                    fun(Pairs) -> {ok, maps:from_list(Pairs)} end
                )
            end
        )
    end.

-file("src/gleam/dynamic.gleam", 468).
-spec any(list(fun((dynamic_()) -> {ok, COK} | {error, list(decode_error())}))) -> fun((dynamic_()) -> {ok,
        COK} |
    {error, list(decode_error())}).
any(Decoders) ->
    do_any(Decoders).

-file("src/gleam/dynamic.gleam", 707).
-spec all_errors({ok, any()} | {error, list(decode_error())}) -> list(decode_error()).
all_errors(Result) ->
    case Result of
        {ok, _} ->
            [];

        {error, Errors} ->
            Errors
    end.

-file("src/gleam/dynamic.gleam", 490).
-spec decode1(
    fun((COS) -> COT),
    fun((dynamic_()) -> {ok, COS} | {error, list(decode_error())})
) -> fun((dynamic_()) -> {ok, COT} | {error, list(decode_error())}).
decode1(Constructor, T1) ->
    fun(Value) -> case T1(Value) of
            {ok, A} ->
                {ok, Constructor(A)};

            A@1 ->
                {error, all_errors(A@1)}
        end end.

-file("src/gleam/dynamic.gleam", 500).
-spec decode2(
    fun((COW, COX) -> COY),
    fun((dynamic_()) -> {ok, COW} | {error, list(decode_error())}),
    fun((dynamic_()) -> {ok, COX} | {error, list(decode_error())})
) -> fun((dynamic_()) -> {ok, COY} | {error, list(decode_error())}).
decode2(Constructor, T1, T2) ->
    fun(Value) -> case {T1(Value), T2(Value)} of
            {{ok, A}, {ok, B}} ->
                {ok, Constructor(A, B)};

            {A@1, B@1} ->
                {error, gleam@list:flatten([all_errors(A@1), all_errors(B@1)])}
        end end.

-file("src/gleam/dynamic.gleam", 514).
-spec decode3(
    fun((CPC, CPD, CPE) -> CPF),
    fun((dynamic_()) -> {ok, CPC} | {error, list(decode_error())}),
    fun((dynamic_()) -> {ok, CPD} | {error, list(decode_error())}),
    fun((dynamic_()) -> {ok, CPE} | {error, list(decode_error())})
) -> fun((dynamic_()) -> {ok, CPF} | {error, list(decode_error())}).
decode3(Constructor, T1, T2, T3) ->
    fun(Value) -> case {T1(Value), T2(Value), T3(Value)} of
            {{ok, A}, {ok, B}, {ok, C}} ->
                {ok, Constructor(A, B, C)};

            {A@1, B@1, C@1} ->
                {error,
                    gleam@list:flatten(
                        [all_errors(A@1), all_errors(B@1), all_errors(C@1)]
                    )}
        end end.

-file("src/gleam/dynamic.gleam", 530).
-spec decode4(
    fun((CPK, CPL, CPM, CPN) -> CPO),
    fun((dynamic_()) -> {ok, CPK} | {error, list(decode_error())}),
    fun((dynamic_()) -> {ok, CPL} | {error, list(decode_error())}),
    fun((dynamic_()) -> {ok, CPM} | {error, list(decode_error())}),
    fun((dynamic_()) -> {ok, CPN} | {error, list(decode_error())})
) -> fun((dynamic_()) -> {ok, CPO} | {error, list(decode_error())}).
decode4(Constructor, T1, T2, T3, T4) ->
    fun(X) -> case {T1(X), T2(X), T3(X), T4(X)} of
            {{ok, A}, {ok, B}, {ok, C}, {ok, D}} ->
                {ok, Constructor(A, B, C, D)};

            {A@1, B@1, C@1, D@1} ->
                {error,
                    gleam@list:flatten(
                        [all_errors(A@1),
                            all_errors(B@1),
                            all_errors(C@1),
                            all_errors(D@1)]
                    )}
        end end.

-file("src/gleam/dynamic.gleam", 554).
-spec decode5(
    fun((CPU, CPV, CPW, CPX, CPY) -> CPZ),
    fun((dynamic_()) -> {ok, CPU} | {error, list(decode_error())}),
    fun((dynamic_()) -> {ok, CPV} | {error, list(decode_error())}),
    fun((dynamic_()) -> {ok, CPW} | {error, list(decode_error())}),
    fun((dynamic_()) -> {ok, CPX} | {error, list(decode_error())}),
    fun((dynamic_()) -> {ok, CPY} | {error, list(decode_error())})
) -> fun((dynamic_()) -> {ok, CPZ} | {error, list(decode_error())}).
decode5(Constructor, T1, T2, T3, T4, T5) ->
    fun(X) -> case {T1(X), T2(X), T3(X), T4(X), T5(X)} of
            {{ok, A}, {ok, B}, {ok, C}, {ok, D}, {ok, E}} ->
                {ok, Constructor(A, B, C, D, E)};

            {A@1, B@1, C@1, D@1, E@1} ->
                {error,
                    gleam@list:flatten(
                        [all_errors(A@1),
                            all_errors(B@1),
                            all_errors(C@1),
                            all_errors(D@1),
                            all_errors(E@1)]
                    )}
        end end.

-file("src/gleam/dynamic.gleam", 580).
-spec decode6(
    fun((CQG, CQH, CQI, CQJ, CQK, CQL) -> CQM),
    fun((dynamic_()) -> {ok, CQG} | {error, list(decode_error())}),
    fun((dynamic_()) -> {ok, CQH} | {error, list(decode_error())}),
    fun((dynamic_()) -> {ok, CQI} | {error, list(decode_error())}),
    fun((dynamic_()) -> {ok, CQJ} | {error, list(decode_error())}),
    fun((dynamic_()) -> {ok, CQK} | {error, list(decode_error())}),
    fun((dynamic_()) -> {ok, CQL} | {error, list(decode_error())})
) -> fun((dynamic_()) -> {ok, CQM} | {error, list(decode_error())}).
decode6(Constructor, T1, T2, T3, T4, T5, T6) ->
    fun(X) -> case {T1(X), T2(X), T3(X), T4(X), T5(X), T6(X)} of
            {{ok, A}, {ok, B}, {ok, C}, {ok, D}, {ok, E}, {ok, F}} ->
                {ok, Constructor(A, B, C, D, E, F)};

            {A@1, B@1, C@1, D@1, E@1, F@1} ->
                {error,
                    gleam@list:flatten(
                        [all_errors(A@1),
                            all_errors(B@1),
                            all_errors(C@1),
                            all_errors(D@1),
                            all_errors(E@1),
                            all_errors(F@1)]
                    )}
        end end.

-file("src/gleam/dynamic.gleam", 609).
-spec decode7(
    fun((CQU, CQV, CQW, CQX, CQY, CQZ, CRA) -> CRB),
    fun((dynamic_()) -> {ok, CQU} | {error, list(decode_error())}),
    fun((dynamic_()) -> {ok, CQV} | {error, list(decode_error())}),
    fun((dynamic_()) -> {ok, CQW} | {error, list(decode_error())}),
    fun((dynamic_()) -> {ok, CQX} | {error, list(decode_error())}),
    fun((dynamic_()) -> {ok, CQY} | {error, list(decode_error())}),
    fun((dynamic_()) -> {ok, CQZ} | {error, list(decode_error())}),
    fun((dynamic_()) -> {ok, CRA} | {error, list(decode_error())})
) -> fun((dynamic_()) -> {ok, CRB} | {error, list(decode_error())}).
decode7(Constructor, T1, T2, T3, T4, T5, T6, T7) ->
    fun(X) -> case {T1(X), T2(X), T3(X), T4(X), T5(X), T6(X), T7(X)} of
            {{ok, A}, {ok, B}, {ok, C}, {ok, D}, {ok, E}, {ok, F}, {ok, G}} ->
                {ok, Constructor(A, B, C, D, E, F, G)};

            {A@1, B@1, C@1, D@1, E@1, F@1, G@1} ->
                {error,
                    gleam@list:flatten(
                        [all_errors(A@1),
                            all_errors(B@1),
                            all_errors(C@1),
                            all_errors(D@1),
                            all_errors(E@1),
                            all_errors(F@1),
                            all_errors(G@1)]
                    )}
        end end.

-file("src/gleam/dynamic.gleam", 640).
-spec decode8(
    fun((CRK, CRL, CRM, CRN, CRO, CRP, CRQ, CRR) -> CRS),
    fun((dynamic_()) -> {ok, CRK} | {error, list(decode_error())}),
    fun((dynamic_()) -> {ok, CRL} | {error, list(decode_error())}),
    fun((dynamic_()) -> {ok, CRM} | {error, list(decode_error())}),
    fun((dynamic_()) -> {ok, CRN} | {error, list(decode_error())}),
    fun((dynamic_()) -> {ok, CRO} | {error, list(decode_error())}),
    fun((dynamic_()) -> {ok, CRP} | {error, list(decode_error())}),
    fun((dynamic_()) -> {ok, CRQ} | {error, list(decode_error())}),
    fun((dynamic_()) -> {ok, CRR} | {error, list(decode_error())})
) -> fun((dynamic_()) -> {ok, CRS} | {error, list(decode_error())}).
decode8(Constructor, T1, T2, T3, T4, T5, T6, T7, T8) ->
    fun(X) -> case {T1(X), T2(X), T3(X), T4(X), T5(X), T6(X), T7(X), T8(X)} of
            {{ok, A},
                {ok, B},
                {ok, C},
                {ok, D},
                {ok, E},
                {ok, F},
                {ok, G},
                {ok, H}} ->
                {ok, Constructor(A, B, C, D, E, F, G, H)};

            {A@1, B@1, C@1, D@1, E@1, F@1, G@1, H@1} ->
                {error,
                    gleam@list:flatten(
                        [all_errors(A@1),
                            all_errors(B@1),
                            all_errors(C@1),
                            all_errors(D@1),
                            all_errors(E@1),
                            all_errors(F@1),
                            all_errors(G@1),
                            all_errors(H@1)]
                    )}
        end end.

-file("src/gleam/dynamic.gleam", 673).
-spec decode9(
    fun((CSC, CSD, CSE, CSF, CSG, CSH, CSI, CSJ, CSK) -> CSL),
    fun((dynamic_()) -> {ok, CSC} | {error, list(decode_error())}),
    fun((dynamic_()) -> {ok, CSD} | {error, list(decode_error())}),
    fun((dynamic_()) -> {ok, CSE} | {error, list(decode_error())}),
    fun((dynamic_()) -> {ok, CSF} | {error, list(decode_error())}),
    fun((dynamic_()) -> {ok, CSG} | {error, list(decode_error())}),
    fun((dynamic_()) -> {ok, CSH} | {error, list(decode_error())}),
    fun((dynamic_()) -> {ok, CSI} | {error, list(decode_error())}),
    fun((dynamic_()) -> {ok, CSJ} | {error, list(decode_error())}),
    fun((dynamic_()) -> {ok, CSK} | {error, list(decode_error())})
) -> fun((dynamic_()) -> {ok, CSL} | {error, list(decode_error())}).
decode9(Constructor, T1, T2, T3, T4, T5, T6, T7, T8, T9) ->
    fun(X) ->
        case {T1(X), T2(X), T3(X), T4(X), T5(X), T6(X), T7(X), T8(X), T9(X)} of
            {{ok, A},
                {ok, B},
                {ok, C},
                {ok, D},
                {ok, E},
                {ok, F},
                {ok, G},
                {ok, H},
                {ok, I}} ->
                {ok, Constructor(A, B, C, D, E, F, G, H, I)};

            {A@1, B@1, C@1, D@1, E@1, F@1, G@1, H@1, I@1} ->
                {error,
                    gleam@list:flatten(
                        [all_errors(A@1),
                            all_errors(B@1),
                            all_errors(C@1),
                            all_errors(D@1),
                            all_errors(E@1),
                            all_errors(F@1),
                            all_errors(G@1),
                            all_errors(H@1),
                            all_errors(I@1)]
                    )}
        end
    end.
