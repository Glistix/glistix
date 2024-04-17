{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    systems.url = "github:nix-systems/default";
  };

  outputs = inputs@{ nixpkgs, flake-parts, systems, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = import systems;

      imports = [
        flake-parts.flakeModules.easyOverlay
        ./nix/flake-builders.nix
      ];

      perSystem = { self', pkgs, lib, ... }:
        let
          expressions = import ./nix;

          glistix = pkgs.callPackage expressions.glistix { };
        in
        {
          formatter = pkgs.nixpkgs-fmt;

          builders.buildGlistixPackage =
            expressions.buildGlistixPackage
            { inherit (pkgs) stdenv fetchurl; glistix = self'.packages.default; };

          packages = {
            default = glistix;
            glistix-dev = self'.packages.default;
          };

          apps.default = {
            type = "app";
            program = lib.getExe glistix;
          };

          devShells.default = pkgs.mkShell {
            packages = with pkgs; [ rustc cargo ];

            buildInputs = (lib.optionals pkgs.stdenv.isDarwin [
              pkgs.Security
              pkgs.SystemConfiguration
            ]) ++ [
              pkgs.openssl
            ];

            nativeBuildInputs = [
              pkgs.pkg-config
              pkgs.openssl.dev
            ];
          };
        };
    };
}
