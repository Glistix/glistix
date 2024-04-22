{ lib
, rustPlatform
, stdenv
, openssl
, git
, pkg-config
, darwin ? { }
, Security ? darwin.Security
, SystemConfiguration ? darwin.SystemConfiguration
}:

rustPlatform.buildRustPackage {
  pname = "glistix";
  version = "0.1.0";

  src = lib.sourceByRegex ./.. [
    ''(bin|test(|-community-packages|-package-compiler)|compiler-(cli|core|wasm))(/.*)?''
    ''Cargo\.(toml|lock)''
  ];

  nativeBuildInputs = [ git pkg-config ];

  buildInputs = [ openssl ] ++
    lib.optionals stdenv.isDarwin [ Security SystemConfiguration ];

  cargoHash = "sha256-RxHkBOIMtXrr4Up7oY8K3jszdWiy9BF1nyv1fE/be48=";

  meta = with lib; {
    description = "A fork of the Gleam compiler with a Nix backend";
    mainProgram = "glistix";
    homepage = "https://github.com/Glistix/glistix";
    license = licenses.asl20;
  };
}
