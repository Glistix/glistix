<h1><p align="center">✨ <i> Glistix</i> ❄️</p></h1>
<h3><p align="center">The power of Nix meets the simplicity and safety of Gleam</p></h3>

<h3><p align="center"><i><a href="https://glistix.github.io/book">Read the official Glistix book here!</a></i></p></h3>

<!-- A spacer -->
<div>&nbsp;</div>

Glistix is a **fork of [the Gleam compiler](https://github.com/gleam-lang/gleam)** which adds a **[Nix](https://nixos.org/) backend**, so that you can **compile Gleam code into Nix** and use it in your configurations!

This allows you to leverage Gleam's **type-safety** and **simplicity** to write reasonable and more correct code. You will also be able to use **Gleam's tooling** in your favor, such as unit tests and easy package management.

For more information on Gleam, including tutorials, please check [the Gleam language's website](https://gleam.run) and [its language tour](https://tour.gleam.run/). Likewise, to learn more about the Nix language, check [the NixOS website](https://nixos.org/) as well.

**NOTE:** Glistix is **beta software**. You shouldn't rely on it on production just yet, but **feel free to give it a shot on personal or smaller projects** and **report any issues and bugs you find**. It should be functional, but we still need people to try it out and report any bugs. Also check the Roadmap at the bottom.

**NOTE:** Glistix's latest stable version currently tracks **Gleam v1.1.0,** meaning features and fixes from up to that Gleam version are available.

**NOTE:** Glistix is **an unofficial project** and is therefore **not affiliated with the Gleam project**.

## Sponsors

Please consider [supporting the team behind Gleam on GitHub](https://github.com/sponsors/lpil). Without their awesome project, Glistix wouldn't exist!

If you also want to support the Glistix project directly, feel free to [support me on GitHub](https://github.com/sponsors/PgBiel) as well!

## Installation

There are a few ways you can install Glistix.

1. **Through Nix:** you can use `nix run github:glistix/glistix/v0.1.0 -- --help` to quickly install and use the latest stable Glistix compiler. (You can also add this repository as an input to your flake, or otherwise fetch this repository in your configuration, in order to persist the install.) Note that this will compile from source.

2. **Through Cargo:** If you have a Rust toolchain installed, you can use `cargo install --git https://github.com/glistix/glistix --tag v0.1.0 --locked` to compile and install the latest stable Glistix compiler.

Glistix is not currently available through Nixpkgs.

## Usage

### Creating your first project

1. Create a new project with `glistix new name`.
2. Adjust `gleam.toml` to your liking; add dependencies from Hex with `glistix add name`.
3. Modify the code at `src/name.gleam`. Ideally, have your main function return something useful, such as `5 + 5`.
4. You can use `glistix build` (or `glistix build --target nix` when not specified in your `gleam.toml`) to compile your Gleam code into a Nix expression.
5. Use `glistix run` (optionally with `--target nix` if not specified in your configuration) to check the output of `main`. This calls `nix-instantiate` by default.
6. Use `glistix test` to run tests in the `test/` directory using the [`glistix_gleeunit` test runner](https://github.com/glistix/gleeunit).

### Using in a configuration

<!-- TODO -->

### Using with editors / Gleam LSP

To properly use Glistix with editors and LSPs, you will need to configure them to use the `glistix` program instead of `gleam`. Otherwise, they won't recognize Glistix-only syntax, specifically `@target(nix)` and `@external(nix, ..., ...)`, in your code.

The VSCode Extension for Gleam does not have this option at the moment, so you will have to alias `gleam` to `glistix` in your system while using VSCode (or while working on your project). Note that `glistix` also supports the Erlang and JavaScript targets, maintained at the upstream compiler (we currently aim to be compatible with existing non-Nix projects as much as possible - please open an issue if something breaks).

However, you can also apply this alias change in a per-project (or per-folder) basis through `direnv`. You can do this by linking the `glistix` executable to a folder, naming the link as `gleam` (e.g. through `ln -s $(which glistix) -T some/folder/gleam`), and then adding an `.envrc` file to your project with contents such as

```sh
export PATH="/path/to/folder/with/link:$PATH"
```

With that in place, you can, for example install the [`direnv` VSCode extension](https://marketplace.visualstudio.com/items?itemName=mkhl.direnv) (which requires `direnv` to be installed in your system) which will automatically load that alias each time you open the project, and the Gleam LSP extension will properly use the Glistix executable.

The above applies to other editors supporting `direnv` as well. For those which don't, you will have to run `direnv` through the command line and open the editor through it.

## Important notes

### Overriding packages incompatible with Nix

While many Gleam packages can, in principle, be used with Glistix projects, it is also a fact that **a large number of Gleam packages depend on Gleam's usual targets** (Erlang and JavaScript) by depending (heavily or not) on FFI. This makes using them with Glistix a non-starter at first. The way around this is to create a **fork with Nix support** (adding Nix FFI together with Erlang and JavaScript FFI), and then use it through **Git submodules** (at least while Gleam doesn't support Git dependencies).

The most important such package is `gleam_stdlib`. It is an essential package for all things Gleam, depended on by almost all Gleam packages, and it **heavily depends on FFI.** Therefore, Glistix made its own Nix-compatible fork at https://github.com/glistix/stdlib. `glistix new` will automatically clone it for new projects, but if it failed to do so or you'd like to use it for an existing project, you can follow the steps below:

1. Add it as a git submodule with `git submodule add https://github.com/glistix/stdlib external/stdlib` (make sure you have an `external` folder, as a convention)
    - Alternatively, when not using a Git repository for your project, you can just clone it: `git clone https://github.com/glistix/stdlib external/stdlib`
2. Replace `gleam_stdlib = "some numbers"` in your `gleam.toml`, under `[dependencies]`, with `gleam_stdlib = { path = "./external/stdlib" }`, in order to have it use your local clone instead of the Hex version

Repeat the procedure for any Gleam dependencies you might need to patch to have proper Nix support. The Glistix project maintains forks for several core packages, including not only the standard library (mentioned above), but also gleeunit (https://github.com/glistix/gleeunit) and json (https://github.com/glistix/json).

The procedure above might change based on the outcome of upstream issue https://github.com/gleam-lang/gleam/issues/2899.

**Note:** When publishing a package to Hex, you will have to remove the patch above from your `gleam.toml` (you can just comment out the line containing `dependency = { path = ... }` with a leading `#`, and uncomment the line with `dependency = "numbers"`). This is because you cannot publish packages with local dependencies to Hex. However, your package will still work for end users as they will also have patched the dependencies by themselves (make sure to add a disclaimer in your README if you have any unusual dependencies that need patching).

### Strictness and laziness

Nix is lazy by default, which means that values aren't evaluated until they are used. This means that side effects are often suppressed when the values that trigger them aren't used. However, Glistix includes **opt-in strictness** through **discarded expressions.** That is, any expressions which appear in the body of your function but are not bound to any variables **are evaluated before the function's return value** through Nix's `builtins.seq` function.

For more info, see the ["Known Issues" page of the Glistix book.](https://glistix.github.io/book/about/known-issues.html)

## Contributing

Make sure to take a look at `CONTRIBUTING.md`. In particular, please **discuss first** (open an issue) before opening a large PR.

Please note that we generally try to follow an **upstream-first policy**. This means that any feature ideas which are not related to Nix should generally be brought up on [the upstream Gleam repository](https://github.com/gleam-lang/gleam) first; if accepted there, the feature should be implemented (/contributed) upstream. Otherwise, we can consider implementing it on Glistix if it would significantly improve the experience of using Glistix, especially regarding usage with Nix, but not before proper discussions. The idea is to not only keep up with the Gleam ecosystem at large, but also to ensure our progress also benefits users of Gleam's Erlang and JavaScript targets.

## License

Glistix is licensed under Apache-2.0, and is largely based on work done by the Gleam team, which is also licensed under Apache-2.0.

Glistix is an unofficial project and is not affiliated with the team behind the Gleam compiler.
