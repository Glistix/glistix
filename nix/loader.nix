# Allows quickly loading Glistix packages.
{ src
, output ? src + "/output"
, derivation ? null
, glistixPackage ? derivation.glistixPackage
, glistixMain ? derivation.glistixMain or "nix/${glistixPackage}/${glistixPackage}.nix"
, fileToLoad ? glistixMain
}:

let
  hasOutput = output != null && builtins.pathExists (output + "/nix");
  importPath =
    if hasOutput
    then output + "/${fileToLoad}"
    else derivation.outPath + "/${fileToLoad}";
in

# Either have your package's build output be somewhere in specific,
# or build it from scratch.
assert derivation == null -> hasOutput;

import importPath
