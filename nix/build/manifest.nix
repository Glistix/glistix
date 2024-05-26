# Parses Gleam's manifest.toml.
{ manifestContents }:
let
  manifest = builtins.fromTOML manifestContents;

  packages = manifest.packages or [];

  parsedPackages =
    map
    (pkg:
      if pkg.source == "local"
      then { inherit (pkg) name version source path; }
      else if pkg.source == "hex"
      then { inherit (pkg) name version source; sha256 = pkg.outer_checksum; }
      else throw "Unexpected Gleam package source '${pkg.source}', expected 'hex' or 'local'")
    packages;
in
parsedPackages
