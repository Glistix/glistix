# A builder for Glistix packages.
#
{ stdenv, fetchurl, glistix }@defaults:

{ pname ? null
, version ? null
, src
, sourceRoot ? "."
, gleamToml ? src + "/gleam.toml"
, manifestFile ? src + "/manifest.toml"
, stdenv ? defaults.stdenv
, fetchurl ? defaults.fetchurl
, glistix ? defaults.glistix
, ...
}@args:

let
  parseManifest = import ./manifest.nix;
  gleamTomlContents =
    if builtins.isPath gleamToml
    then builtins.readFile gleamToml
    else gleamToml;
  projectConfig = builtins.fromTOML gleamTomlContents;
  pkgVersion = projectConfig.version;
  pkgName = projectConfig.name;

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

  buildInputs = [ glistix ];

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

    glistix build --target nix

    runHook postBuild
  '';

  # The generated Nix files are our output
  installPhase = ''
    runHook preInstall

    cp build/dev/nix -rT "$out"

    runHook postInstall
  '';

  passthru = {
    glistixPackage = pkgName;

    # Relative path to the package's entrypoint
    # from the derivation's root directory.
    glistixMain = "nix/${pkgName}/${pkgName}.nix";

    # Relative path to the package's testing entrypoint
    # from the derivation's root directory.
    glistixTest = "nix/${pkgName}/${pkgName}_test.nix";
  };
})
