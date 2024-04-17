# A builder for Glistix packages.
#
{ stdenv, fetchurl, gleam }:

{ pname ? null
, version ? null
, sourceRoot ? "."
, gleamToml
, manifestFile
, ...
}@args:

let
  parseManifest = import ./manifest.nix;
  gleamTomlContents =
    if builtins.isPath gleamToml
    then builtins.readFile gleamToml
    else gleamToml;
  gleamConfig = builtins.fromTOML gleamTomlContents;
  pkgVersion = gleamConfig.version;
  pkgName = gleamConfig.name;

  manifest = parseManifest { manifestContents = builtins.readFile manifestFile; };

  hexPackages = builtins.filter (pkg: pkg.source == "hex") manifest;

  hexTarballURL = name: version: "https://repo.hex.pm/tarballs/${name}-${version}.tar";

  # Fetch a hex package, but only the tarball, as that's what Gleam caches
  fetchHex = { pkg, version, sha256 }: fetchurl { url = hexTarballURL pkg version; inherit sha256; };

  fetchedHexPackages =
    map
    ({ name, version, sha256, ... }: fetchHex { inherit version sha256; pkg = name; })
    hexPackages;

  fetchedHexPackagePaths = builtins.concatStringsSep "\n" fetchedHexPackages;
in

assert gleamToml != null;
assert builtins.match "[a-zA-Z0-9_-]+" pkgName != null;

stdenv.mkDerivation (args // {
  inherit fetchedHexPackagePaths;
  pname = if pname != null then pname else pkgName;
  version = if version != null then version else pkgVersion;

  buildInputs = [ gleam ];

  # Symlink dependencies to a directory which Gleam has access to
  configurePhase = ''
    runHook preConfigure

    export XDG_CACHE_HOME="$TEMPDIR/.gleam_cache"
    hexdir="''${XDG_CACHE_HOME}/gleam/hex/hexpm/packages"
    mkdir -p "$hexdir"

    IFS=$'\n'
    for package in $fetchedHexPackagePaths;
    do
      dest="$hexdir/$(basename "$package" | cut -d '-' -f1 --complement)"
      echo "Linking $package to $dest"
      ln -s "$package" -T "$dest"
    done

    runHook postConfigure
  '';

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
    gleamMain = "nix/${pkgName}/${pkgName}.nix";

    # Relative path to the package's testing entrypoint
    # from the derivation's root directory.
    gleamTest = "nix/${pkgName}/${pkgName}_test.nix";
  };
})
