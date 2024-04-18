# Generates a function which allows quickly loading a Glistix package
# by importing its main Nix file (at least by default).
# The arguments to the generator are defaults which can be overridden
# by passing the same arguments to the generated function.
#
# Its project folder is all that is needed if we're looking for
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
#     glistix.lib.makeGlistixPackageLoader {
#       src = ./.;
#       # assuming output is at ./output
#       package = "pkgname";
#       module = "pkgname";
#     };
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
{ src ? null
, output ? null
, derivation ? null
, nixRoot ? null
, package ? null
, module ? null
}@defaults:

# Allow overriding
{ output ? defaults.output or (src + "/output")
, derivation ? defaults.derivation or null
, nixRoot ? defaults.nixRoot or derivation.nixRoot or "dev/nix"
, package ? defaults.package or derivation.glistixPackage or (builtins.fromTOML (builtins.readFile (src + "/gleam.toml"))).name
, module ? defaults.module or package
, fileToLoad ? "${nixRoot}/${package}/${module}.nix"
}:

assert builtins.match "[a-zA-Z0-9_\\-]+" (builtins.toString package) != null;
assert builtins.match "[a-zA-Z0-9_\\-\\/]+" module != null;

let
  hasOutput = output != null && builtins.pathExists (output + "/dev/nix");
  importPath =
    if hasOutput
    then output + "/${fileToLoad}"
    else derivation.outPath + "/${fileToLoad}";
in

# Either have your package's build output be somewhere in specific,
# or build it from scratch.
assert derivation == null -> hasOutput;

import importPath
