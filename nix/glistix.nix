{ lib
, rustPlatform
, stdenv
, openssl
, git
, pkg-config
, darwin
, Security ? darwin.apple_sdk.frameworks.Security
, SystemConfiguration ? darwin.apple_sdk.frameworks.SystemConfiguration
}:

let
  sourcePaths = [
    "test"
    "test-package-compiler"
    "test-project-compiler"
    "test-helpers-rs"
    "test-community-packages"
    "compiler-cli"
    "compiler-core"
    "compiler-wasm"
    "Cargo.toml"
    "Cargo.lock"
  ];

  filterPaths = path: type:
    builtins.any (accepted: lib.path.hasPrefix (./../. + "/${accepted}") (/. + path)) sourcePaths;
in

rustPlatform.buildRustPackage {
  pname = "glistix";
  version = "0.8.0";

  src = lib.cleanSourceWith {
    filter = filterPaths;
    src = ./..;
  };

  nativeBuildInputs = [ git pkg-config ];

  buildInputs = [ openssl ] ++
    lib.optionals stdenv.isDarwin [ Security SystemConfiguration ];

  cargoHash = "sha256-073LUpMgDJ2N/MSGae8741ScF/dxQXBykWlInJ5uHEs=";

  meta = with lib; {
    description = "A fork of the Gleam compiler with a Nix backend";
    mainProgram = "glistix";
    homepage = "https://github.com/Glistix/glistix";
    license = licenses.asl20;
  };
}
