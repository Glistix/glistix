{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    systems.url = "github:nix-systems/default";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    rust-manifest = {
      url = "https://static.rust-lang.org/dist/channel-rust-1.83.0.toml";
      flake = false;
    };
  };

  outputs = inputs@{ nixpkgs, flake-parts, systems, fenix, rust-manifest, ... }:
    let
      expressions = import ./nix;
    in
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = import systems;

      flake = {
        lib = { inherit (expressions) loadGlistixPackage; };
      };

      imports = [
        flake-parts.flakeModules.easyOverlay
        ./nix/flake-builders.nix
      ];

      perSystem = { self', pkgs, lib, system, ... }:
        let
          rust-toolchain = (fenix.packages.${system}.fromManifestFile rust-manifest).defaultToolchain;

          rustPlatform = pkgs.makeRustPlatform {
            cargo = rust-toolchain;
            rustc = rust-toolchain;
          };

          glistix = pkgs.callPackage expressions.glistix { inherit rustPlatform; };
        in
        {
          formatter = pkgs.nixpkgs-fmt;

          builders.buildGlistixPackage =
            pkgs.callPackage
            expressions.buildGlistixPackage
            { glistix = self'.packages.default; };

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
              pkgs.darwin.apple_sdk.frameworks.Security
              pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
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
