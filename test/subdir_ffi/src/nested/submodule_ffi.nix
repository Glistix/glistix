let
  main = x: "Hello from the nested Nix native module!";
  main2 = x: "Hello again from the nested Nix native module!";
in { inherit main main2; }
