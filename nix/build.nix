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

# Path to gleam.toml which we can read from.
# This is used to automatically determine 'pname'
# and 'version' based on your package's name and version.
, gleamToml ? sourceRootPath args "/gleam.toml"

# The file attribute must be a path to a valid manifest.toml file.
# This specifies how to fetch each dependency before building.
, manifestToml ? { file = sourceRootPath args "/manifest.toml"; }

# Submodules in the format { src = PATH, dest = "RELATIVE PATH STRING" }.
# Basically any extra (usually Git) dependencies, which are copied to the
# given destinations at build-time.
, submodules ? [ ]

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

  convertedSubmodules = import ./submodules.nix { inherit submodules; };

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
  pname = if pname != null then pname else pkgName;
  version = if version != null then version else pkgVersion;

  # for hex.sh
  inherit fetchedHexPackagePaths;
  # for submodules.sh
  inherit (convertedSubmodules) submodules;

  buildInputs = [ glistix ];

  # Symlink dependencies to a directory which Gleam has access to
  configurePhase = ''
    runHook preConfigure

    ${builtins.readFile ./hex.sh}

    ${builtins.readFile ./submodules.sh}

    runHook postConfigure
  '';

  # TODO: prefetch and expand deps
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
