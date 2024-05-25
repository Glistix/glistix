# Expects manifest packages in the form { name, version, sha256, ... }.
{ packages, fetchurl }:
let
  hexTarballURL = name: version: "https://repo.hex.pm/tarballs/${name}-${version}.tar";

  # Fetch a hex package, but only the tarball, as that's what Gleam caches
  fetchHex = { pkg, version, sha256 }: fetchurl { url = hexTarballURL pkg version; inherit sha256; };

  fetchedHexPackages =
    map
    ({ name, version, sha256, ... }: fetchHex { inherit version sha256; pkg = name; })
    packages;

  # Environment variable which will pass one hex package path per line
  # to ./hex.sh.
  fetchedHexPackagePaths = builtins.concatStringsSep "\n" fetchedHexPackages;
in
{
  inherit fetchedHexPackagePaths;
}
