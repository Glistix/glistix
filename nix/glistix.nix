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
  version = "1.0.0";

  src = lib.sourceByRegex ./.. [
    ''(bin|test(|-community-packages|-package-compiler)|compiler-(cli|core|wasm))(/.*)?''
    ''Cargo\.(toml|lock)''
  ];

  nativeBuildInputs = [ git pkg-config ];

  buildInputs = [ openssl ] ++
    lib.optionals stdenv.isDarwin [ Security SystemConfiguration ];

  cargoHash = "sha256-Ldwy8JVjnDPgm5d95MSeAUtGoW2tGMrCJZDskZZuY9A=";

  meta = with lib; {
    description = "A fork of the Gleam compiler with a Nix backend";
    mainProgram = "glistix";
    homepage = "https://github.com/Glistix/glistix";
    license = licenses.asl20;
  };
}
