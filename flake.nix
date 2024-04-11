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

          makePackage = pkgs:
            pkgs.rustPlatform.buildRustPackage {
              pname = "gleam";
              version = "1.0.0";

              src = lib.sourceByRegex ./. [
                ''(bin|test(|-community-packages|-package-compiler)|compiler-(cli|core|wasm))(/.*)?''
                ''Cargo\.(toml|lock)''
              ];

              nativeBuildInputs = with pkgs; [ git pkg-config ];

              buildInputs = with pkgs; [ openssl ] ++
                lib.optionals stdenv.isDarwin (with pkgs; [ Security SystemConfiguration ]);

              cargoHash = "sha256-o2V4SKkF/GNGuKZ/tYeNqkFwYyXJJTzQuojF9Y3l3go=";

              meta = with lib; {
                description = "A statically typed language for the Erlang VM";
                mainProgram = "gleam";
                homepage = "https://gleam.run/";
                license = licenses.asl20;
              };
            };

          gleam = makePackage pkgs;
        in
        {
          formatter = pkgs.nixpkgs-fmt;

          builders.buildGlistixPackage =
            expressions.buildGlistixPackage
            { inherit (pkgs) stdenv fetchurl; gleam = self'.packages.default; };

          packages = {
            default = gleam;
            gleam-dev = self'.packages.default;
          };

          apps.default = {
            type = "app";
            program = lib.getExe gleam;
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
