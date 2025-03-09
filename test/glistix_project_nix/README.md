# Project

A test Glistix project with the Nix target.

It covers these features:

- Downloading packages
  - Specified in config.dependencies
  - Patched in config.glistix.preview.patch
- Importing packages
  - Specified in config.dependencies
- Compilation of Gleam code
  - in src
  - in test
- Importing Gleam src code into test

These features are not tested yet

- Downloading packages
  - Specified in config.dev-dependencies
- Importing packages
  - Specified in config.dev-dependencies
- Importing Gleam src code into test

## Quick start

```sh
glistix run
glistix test
```
