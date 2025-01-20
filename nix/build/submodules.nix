# Converts submodules in the format { src = PATH, dest = "RELATIVE PATH" }
{ submodules }:
let
  submodulesAsStrings = map ({ src, dest }: "${src}:${dest}") submodules;
  submodulesEnvVar = builtins.concatStringsSep "\n" submodulesAsStrings;
in { submodules = submodulesEnvVar; }
