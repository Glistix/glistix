# Module to expose the "builders" option from the flake.
# Passed to 'flake-parts'.
{ lib, flake-parts-lib, ... }:
let
  inherit (lib)
    mkOption
    types
    ;
  inherit (flake-parts-lib)
    mkTransposedPerSystemModule
    ;
in
mkTransposedPerSystemModule {
  name = "builders";
  option = mkOption {
    type = types.lazyAttrsOf types.anything;
    default = { };
    description = ''
      An attribute set of builder functions.
      This is used to expose `buildGlistixPackage`.
    '';
  };
  file = ./flake-builders.nix;
}
