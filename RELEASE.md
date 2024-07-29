# Release checklist

## Glistix checklist

Additional instructions for Glistix:

1. Follow "Gleam checklist" below.
2. Ensure `compiler-core/src/version.rs` reports the correct Gleam version we're tracking.
3. Bump Glistix version in each `Cargo.toml` (as well as `Cargo.lock` by running `cargo check` once), in the new project flake template at `compiler-cli/src/new.rs` and in `nix/glistix.nix`, as well as in `README.md`.
4. Update the `cargoHash` in `nix/glistix.nix` by setting it to `""` and trying to build. The error will inform the new hash.
5. Add changelog to the Glistix book (initially with "(Unreleased)" as the release date).
6. Update the book as needed (there are several compiler internals documented there, including which patches we applied on top of the base Gleam compiler).
7. Update the Glistix version in the book's installation instructions in "Getting Started".

After release:

1. Update the new version's changelog in the book to display the release date in UTC (both in the changelog's page and in the `src/SUMMARY.md`).
2. Push a book tag with the new release version.
3. Update Glistix library forks' flakes to point to the new Glistix version.

## Gleam checklist

1. Update the version in each `Cargo.toml`.
2. Update versions in `src/new.rs` for stdlib etc if required.
3. Run `make test build`.
4. Git commit, tag, push, push tags.
5. Wait for CI release build to finish.
6. Publish release on GitHub from draft made by CI.
7. Update version in `Cargo.toml` to next-dev.
8. Update clone target version in `getting-started.md` in `website`.
