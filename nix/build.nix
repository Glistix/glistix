# A builder for Glistix packages.
#
{ stdenv, gleam }:

{ pname ? null
, version ? null
, sourceRoot ? "."
, src
, gleamToml
, ...
}@args:

let
  gleamTomlContents =
    if builtins.isPath gleamToml
    then builtins.readFile gleamToml
    else gleamToml;
  gleamConfig = builtins.fromTOML gleamTomlContents;
  pkgVersion = gleamConfig.version;
  pkgName = gleamConfig.name;
in

assert gleamToml != null;
assert builtins.match "[a-zA-Z0-9_-]+" pkgName != null;

stdenv.mkDerivation (args // {
  inherit src sourceRoot;
  pname = if pname != null then pname else pkgName;
  version = if version != null then version else pkgVersion;

  buildInputs = [ gleam ];

  # TODO: prefetch deps
  # Build using Glistix
  buildPhase = ''
    runHook preBuild

    gleam build --target nix

    runHook postBuild
  '';

  # The generated Nix files are our output
  installPhase = ''
    runHook preInstall

    cp build/dev/nix -rT "$out"

    runHook postInstall
  '';

  passthru = {
    gleamPackage = pkgName;

    # Relative path to the package's entrypoint
    # from the derivation's root directory.
    gleamMain = "${pkgName}/${pkgName}.nix";

    # Relative path to the package's testing entrypoint
    # from the derivation's root directory.
    gleamTest = "${pkgName}/${pkgName}_test.nix";
  };
})
