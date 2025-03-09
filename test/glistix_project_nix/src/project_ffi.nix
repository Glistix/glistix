let
  log = x: builtins.trace x null;
in
{ inherit log; }
