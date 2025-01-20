# A function which allows quickly loading a Glistix package
# by importing its main Nix file (at least by default).
#
# Its project folder ("src") is all that is needed if we're looking for
# the package's main function and it has an "output/" folder inside
# its project folder with built Nix files at "output/dev/nix".
# The "src" can be omitted when a non-default output is given,
# or when a derivation which compiles the Glistix package from scratch
# with the compiler is given.
#
# You can pick which package and module you want to import.
# By default, this will be `(package name)/(package name).nix`,
# so you can load your main function with "(loadGlistixPackage {}).main {}".
# The "nixRoot" parameter lets you customize the path relative
# to the output / derivation output root folder which contains
# the generated Nix files.
# You can also completely override "fileToLoad" to load some
# arbitrary file relative to the build output directory.
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
  fixNullString = string: if string == null then "(null)" else builtins.toString string;

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
assert (derivation == null -> hasOutput) || builtins.throw ''
  Couldn't load the Glistix package named '${fixNullString package}',
  as its build output wasn't cached, and there was no suitable derivation to build it with.
  (Searched for build cache at: '${fixNullString output}')

  This can happen if you try to call a package's `lib.loadGlistixPackage { }` within a flake, where
  pure evaluation is the default, without specifying a `system` attribute (so that the correct derivation
  can be used for build when the build output isn't cached). This can also happen when the package forces
  the usage of build output cache (usually at a folder named 'output'), but the cache folder wasn't found.

  Make sure to provide either a path to the build cache (as the 'output' attribute to 'loadGlistixPackage'),
  a derivation to build the package with (as the 'derivation' attribute, usually using 'buildGlistixPackage'),
  or, if using a package's 'loadGlistixPackage' implementation, the 'system' attribute so it can correctly pick
  the derivation to use to build the package (if the build output isn't cached). For example, you can try to specify
  `thePackage.lib.loadGlistixPackage { system = "x86_64-linux"; }`.

  Read the Glistix docs for more information.
'';

assert builtins.pathExists importPath || builtins.throw ''
  Couldn't import module '${fixNullString module}' of Glistix package '${fixNullString package}',
  as the path it was supposed to be at, '${importPath}', wasn't found.
  Ensure a valid module was specified, as well as ensure that the build output cache is available
  and up-to-date, if it is used, or (otherwise) if the derivation being used to build the package
  is correct.
'';

import importPath
