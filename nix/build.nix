# A builder for Glistix packages.
#
{ stdenv, glistix, lib }@defaults:

let
  # Converts a "sourceRoot" string to a store path we can
  # retrieve files from.
  # For a single src, it is in the form "source/<something>",
  # so we just strip the first path segment.
  # For srcs, it is in the form "<src name>/<...>", so we take
  # the first segment and find the corresponding derivation
  # in srcs.
  # Consider this to be a best effort.
  sourceRootPath =
    args: add:
      let
        sourceRoot = args.sourceRoot or "source";
        matchedRoot = builtins.match "^([^/]*)(/.*)?$" sourceRoot;
        matchedRestOfRoot =
          if builtins.length matchedRoot > 1
          then builtins.elemAt matchedRoot 1
          else null;
        # Normalize so we don't add null to path
        # When there is nothing to add after the first segment,
        # just don't add anything.
        restOfRoot =
          if matchedRestOfRoot == null
          then ""
          else matchedRestOfRoot;
        correspondingSourcePath =
          name:
            let
              filteredSources =
                builtins.filter (drv: drv.name or drv == name) args.srcs;
            in
            if builtins.length filteredSources == 0
            then
              builtins.throw
                "Expected source root to correspond to a valid path within given `srcs`"
            else "${builtins.head filteredSources}" + restOfRoot;
        effectiveRoot =
          if args ? src
          then "${args.src}" + restOfRoot
          else if args ? srcs
          then correspondingSourcePath
          else builtins.throw "Expected Glistix builder args to have either src or srcs";
      in effectiveRoot + add;
in

{ pname ? null
, version ? null
, gleamToml ? sourceRootPath args "/gleam.toml"
, manifestToml ? { file = sourceRootPath args "/manifest.toml"; }
, stdenv ? defaults.stdenv
, fetchurl ? defaults.lib.fetchurl
, glistix ? defaults.glistix
, ...
}@args:

let
  parseManifest = import ./manifest.nix;
  gleamTomlContents = gleamToml.contents or builtins.readFile gleamToml;
  projectConfig = builtins.fromTOML gleamTomlContents;
  pkgVersion = projectConfig.version;
  pkgName = projectConfig.name;

  manifest = parseManifest { manifestContents = builtins.readFile manifestToml.file; };

  hexPackages = builtins.filter (pkg: pkg.source == "hex") manifest;

  hexTarballURL = name: version: "https://repo.hex.pm/tarballs/${name}-${version}.tar";

  # Fetch a hex package, but only the tarball, as that's what Gleam caches
  fetchHex = { pkg, version, sha256 }: lib.fetchurl { url = hexTarballURL pkg version; inherit sha256; };

  fetchedHexPackages =
    map
    ({ name, version, sha256, ... }: fetchHex { inherit version sha256; pkg = name; })
    hexPackages;

  fetchedHexPackagePaths = builtins.concatStringsSep "\n" fetchedHexPackages;
in

assert gleamToml != null;
assert builtins.match "[a-zA-Z0-9_-]+" pkgName != null;

stdenv.mkDerivation (self': args // {
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

  # The build directory is our output
  installPhase = ''
    runHook preInstall

    cp build -rT "$out"

    runHook postInstall
  '';

  passthru = {
    glistixPackage = pkgName;

    glistixNixRoot = "dev/nix";

    # Relative path to the package's entrypoint
    # from the derivation's root directory.
    glistixMain = "${self'.passthru.glistixNixRoot}/${pkgName}/${pkgName}.nix";

    # Relative path to the package's testing entrypoint
    # from the derivation's root directory.
    glistixTest = "${self'.passthru.glistixNixRoot}/${pkgName}/${pkgName}_test.nix";
  };
})
