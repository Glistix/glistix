# Allows quickly loading a Glistix package by importing
# its main Nix file (at least by default).
#
# Its project folder is all that is needed if we're looking for
# the package's main function and it has an output folder inside
# its project folder.
# The "src" can be omitted when a non-default output is given,
# or when a derivation which compiles the Glistix package from scratch
# with the compiler is given.
# You can pick which file to import (defaults to `glistixMain`,
# equivalent to "dev/nix/(package name)/(package name.nix)" within
# the output or derivation output) with `fileToLoad` (relative
# to the output!).
{ src ? null
, output ? src + "/output"
, derivation ? null
, glistixNixRoot ? derivation.glistixNixRoot or "dev/nix"
, glistixPackage ? derivation.glistixPackage
, glistixMain ? derivation.glistixMain or "${glistixNixRoot}/${glistixPackage}/${glistixPackage}.nix"
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
