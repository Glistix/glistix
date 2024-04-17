<h1><p align="center">✨ <i> Glistix</i> ❄️</p></h1>
<h3><p align="center">The power of Nix meets the simplicity and safety of Gleam</p></h3>

<h3><p align="center"><i><a href="https://glistix.github.io/book">Read the official Glistix book here!</a></i></p></h3>

<!-- A spacer -->
<div>&nbsp;</div>

Glistix is a **fork of [the Gleam compiler](https://github.com/gleam-lang/gleam)** which adds a **[Nix](https://nixos.org/) backend**, so that you can **compile Gleam code into Nix** and use it in your configurations!

This allows you to leverage Gleam's **type-safety** and **simplicity** to write reasonable and more correct code. You will also be able to use **Gleam's tooling** in your favor, such as unit tests and easy package management.

For more information on Gleam, including tutorials, please check [the Gleam language's website](https://gleam.run). Likewise, to learn more about the Nix language, check [the NixOS website](https://nixos.org/) as well.

**NOTE:** Glistix is **beta software**. You shouldn't rely on it on production just yet, but **feel free to give it a shot on personal or smaller projects** and **report any issues and bugs you find**. It should be functional, but we still need people to try it out and report any bugs. Also check the Roadmap at the bottom.

**NOTE:** Glistix's latest stable version currently tracks **Gleam v1.1.0.**

## Sponsors

Please consider [supporting the team behind Gleam on GitHub](https://github.com/sponsors/lpil). Without their awesome project, Glistix wouldn't exist!

If you also want to support the Glistix project directly, feel free to [support me on GitHub](https://github.com/sponsors/PgBiel) as well!

## Installation

There are a few ways you can install Glistix.

1. **Through Nix:** you can use `nix run github:glistix/glistix/stable -- --help` to quickly install and use the latest stable Glistix compiler. (You can also add this repository as an input to your flake, or otherwise fetch this repository in your configuration, in order to persist the install.) Note that this will compile from source.

2. **Through Cargo:** If you have a Rust toolchain installed, you can use `cargo install --git https://github.com/glistix/glistix --branch stable --locked` to compile and install the latest stable Glistix compiler.

Glistix is not currently available through Nixpkgs.

## Usage

### Creating your first project

1. Create a new project with `glistix new name`.
2. Adjust `gleam.toml` to your liking; add dependencies from Hex with `glistix add name`.
3. Modify the code at `src/name.gleam`. Ideally, have your main function return something useful, such as `5 + 5`.
4. You can use `glistix build` (or `glistix build --target nix` when not specified in your `gleam.toml`) to compile your Gleam code into a Nix expression.
5. Use `glistix run` (optionally with `--target nix` if not specified in your configuration) to check the output of `main`. This calls `nix-instantiate` by default.
6. Use `glistix test` to run tests in the `test/` directory using the `gleeunit` test runner.

### Using in a configuration

<!-- TODO -->

### Using with editors / Gleam LSP

To properly use Glistix with editors and LSPs, you will need to configure them to use the `glistix` program instead of `gleam`. Otherwise, they won't recognize `@target(nix)` and `@external(nix, ..., ...)` in your projects.

The VSCode Extension for Gleam does not have this option at the moment, so you will have to alias `gleam` to `glistix`. Which should be OK as `glistix` also supports the Erlang and JavaScript targets (we currently aim to be fully compatible with existing non-Nix projects).

However, you can also apply this alias change in a per-project basis through `direnv`. You can do this by
linking the `glistix` executable as `gleam` to a folder
and then adding an `.envrc` file to your project with contents such as

```sh
export PATH="path/to/folder/with/link:$PATH"
```

which will let you use the `direnv` extension (which requires `direnv` to be installed) to automatically load that alias as needed. This applies to other editors supporting `direnv` as well; for those which don't, you will have to run `direnv` through the command line and start the editor there (e.g. `cd project && direnv allow && zed` for the Zed editor).

## Important notes

### Overriding packages incompatible with Nix

While the entire Gleam ecosystem is at our disposal, truth is that **many Gleam packages depend on Gleam's usual targets** (Erlang and JavaScript) by depending (heavily or not) on FFI. This makes using them a non-starter at first. However, there is a way to dodge this issue: you can **override dependencies and transitive dependencies on your top-level project** in order to **point them to forks which have Nix support.**

The most important such package is `gleam_stdlib`. It is an essential package for all things Gleam, and it **heavily depends on FFI.** Therefore, Glistix made its own fork at https://github.com/glistix/stdlib. To use it on your top-level project, you can:

1. Add it as a git submodule with `git submodule add https://github.com/glistix/stdlib` (optionally followed by `git submodule init`, which must be run when cloning your repository again)
    - Alternatively, when not using a Git repository for your project, you can just clone it: `git clone https://github.com/glistix/stdlib`
2. Replace `gleam_stdlib = "some numbers"` in your `gleam.toml`, under `[dependencies]`, with `gleam_stdlib = { path: "./stdlib" }`, in order to have it use your local clone instead of the Hex version

Repeat the procedure for any Gleam dependencies you might need to patch to have proper Nix support. The Glistix project maintains forks for most core packages, including not only the standard library (mentioned above), but also gleeunit (https://github.com/glistix/gleeunit) and json (https://github.com/glistix/json).

The procedure above might change based on the outcome of upstream issue https://github.com/gleam-lang/gleam/issues/2899.

**Note:** Only follow the procedure above **for top-level packages**, that is, **don't do it on libraries** (especially when publishing them to Hex). It makes sense to do so **when testing libraries**, however - but never hardcode a local path for consumers of your library; let the consumers patch your dependency by themselves instead.

### Strictness and laziness

Nix is lazy by default, which means that values aren't evaluated until they are used. This means that side effects are often suppressed when the values that trigger them aren't used. However, Glistix includes **opt-in strictness** through **discarded expressions.** Consider:

```gleam
pub fn main() {
  let a = panic as "a" // this won't panic ("a" not used)
  let _ = panic as "b" // this will panic
  panic as "c" // this would also panic
  let Nil = panic as "d" // this would also panic
  let e = panic as "e"  // this would also panic ("e" is returned)
  let f = panic as "f"  // this would also panic ("f" is discarded, albeit not returned)
  f

  case False {
    True -> 5
    False -> panic as "g" // will panic
  }

  e
}
```

Additionally, **assertions (`let assert`) are always strict, and are evaluated before the function's returned value.** This ensures you can use assertions to check if certain invariants were met in your function:

```gleam
pub fn func(x: Int, y: Bool) {
  let assert True = y  // will panic if y is False
  let assert 5 = x // will panic if x isn't 5
  let assert Ok(x) = Ok(10) // x will be bound to 10
  let assert Ok(_) = Error(10) // will always panic

  10 // unreachable return
}
```

However, note that expressions evaluated strictly like above are only **shallowly evaluated**, meaning evaluation doesn't recurse into lists, tuples etc.:

```gleam
pub fn main() {
  #(panic as "A", panic as "B")  // won't panic

  Nil
}
```

**This might change in the future** (we'd love to hear your opinion at the relevant issue). In the meantime, you can use Nix's `builtins.deepSeq` function to recursively evaluate values. This can be done through [the `glistix_nix` package](https://github.com/glistix/nix):

```gleam
// Run `glistix add glistix_nix` to use this package
import glistix/nix

pub fn main() {
  nix.deep_eval(#(panic as "A", panic as "B"))  // will panic

  Nil
}
```

## Contributing

Make sure to take a look at `CONTRIBUTING.md`. In particular, please **discuss first** (open an issue) before opening a large PR. Do note that we generally try to follow an **upstream-first policy**. This means that any feature ideas which are not related to Nix should generally be brought up on [the upstream Gleam repository](https://github.com/gleam-lang/gleam) first; if accepted there, the feature should be implemented (/contributed) upstream. Otherwise, we can consider implementing it on Glistix if it would significantly improve the experience of using Glistix, especially regarding usage with Nix, but not before proper discussions. We do this so that we don't "drift off" of the Gleam ecosystem - **Glistix is not a hard fork**, so we try to keep up with the upstream versions of Gleam.

## License

Glistix is licensed under Apache-2.0, and is largely based on work done by the Gleam team, which is also licensed under Apache-2.0.
