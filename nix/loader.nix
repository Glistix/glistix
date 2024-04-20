# A function which allows quickly loading a Glistix package
# by importing its main Nix file (at least by default).
#
# Its project folder ("src") is all that is needed if we're looking for
# the package's main function and it has an output folder inside
# its project folder.
# The "src" can be omitted when a non-default output is given,
# or when a derivation which compiles the Glistix package from scratch
# with the compiler is given.
#
# You can pick which package and module you want to import.
# By default, this will be `(package name)/(package name.nix)`,
# so you can load your main function.
# The "nixRoot" parameter lets you customize the path relative
# to the output / derivation output root folder which contains
# the generated Nix files.
# You can also completely override "fileToLoad" to load some
# arbitrary file relative to the generated Nix directory.
#
# SAMPLE USAGE:
#
# 1. Generate your loader:
# ```nix
# let
#   loadGlistixPackage =
#     { ... }@overrides:
#       glistix.lib.loadGlistixPackage ({
#         src = ./.;
#         # assuming output is at ./output
#         package = "pkgname";
#         module = "pkgname";
#       } // overrides);
# in { inherit loadGlistixPackage; }
# ```
#
# 2. Import and run the loader:
# ```nix
# let
#   mypkg = import ./path/to/pkg;
#   result = (mypkg.loadGlistixPackage {}).main {};
# in result
# ```

let
  # :: path-like -> any
  tryReadGleamToml =
    src:
      if builtins.pathExists (src + "/gleam.toml")
      then builtins.fromTOML (builtins.readFile (src + "/gleam.toml"))
      else {};
in
{ src ? null
, output ? src + "/output"
, derivation ? null
, nixRoot ? null
, package ? (tryReadGleamToml src).name or derivation.glistixPackage or null
, module ? package
, fileToLoad ? null
}:

let
  effectiveNixRoot =
    if nixRoot != null
    then nixRoot
    else if output != null && builtins.pathExists (output + "/dev/nix")
    then "dev/nix" # ignore the derivation when the output is going to be used
    else derivation.glistixNixRoot or "dev/nix";

  effectiveFileToLoad =
    if fileToLoad != null
    then fileToLoad # allow caller to fully customize the file to load
    else "${effectiveNixRoot}/${package}/${module}.nix";

  hasOutput = output != null && builtins.pathExists (output + "/${effectiveNixRoot}");
  importPath =
    if hasOutput
    then output + "/${effectiveFileToLoad}"
    else derivation.outPath + "/${effectiveFileToLoad}";
in

# When the file to load isn't fully overridden, ensure we won't create invalid
# paths
assert fileToLoad == null -> builtins.match "[a-zA-Z0-9_\\-]+" (builtins.toString package) != null;
assert fileToLoad == null -> builtins.match "[a-zA-Z0-9_\\-\\/]+" module != null;

# Either have your package's build output be somewhere in specific,
# or build it from scratch.
assert derivation == null -> hasOutput;

import importPath
