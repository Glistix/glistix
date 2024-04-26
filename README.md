<h1><p align="center">✨ <i> Glistix</i> ❄️</p></h1>
<h3><p align="center">The power of Nix meets the simplicity and type-safety of Gleam</p></h3>

<p align="center">
  <a href="https://github.com/glistix/glistix/releases"><img src="https://img.shields.io/github/release/glistix/glistix" alt="GitHub release"></a>
  <a><img src="https://github.com/Glistix/glistix/actions/workflows/ci.yaml/badge.svg?branch=main"></a>
</p>

<h3><p align="center"><i><a href="https://glistix.github.io/book">Read the Glistix book here!</a></i></p></h3>

**Mirrors:** [**GitHub**](https://github.com/glistix/glistix) | [**Codeberg**](https://codeberg.org/glistix/glistix)

<!-- A spacer -->
<div>&nbsp;</div>

## NOTE: We are releasing very soon, check back later!

Glistix is a **fork of [the Gleam compiler](https://github.com/gleam-lang/gleam)** which adds a **[Nix](https://nixos.org/) backend**, so that you can **compile Gleam code into Nix** and **use it in your configurations!**

This allows you to leverage Gleam's **type-safety** and **simplicity** to write reasonable and more correct code. You will also be able to use **Gleam's tooling** in your favor, such as unit tests and easy package management.

For more information on Gleam, including tutorials, please check [the Gleam language's website](https://gleam.run) and [its language tour](https://tour.gleam.run/). Likewise, to learn more about the Nix language, check [the NixOS website](https://nixos.org/) as well.

**NOTE:** Glistix is **beta software**, and **may have breaking changes**. You shouldn't rely on it on production just yet, but **feel free to give it a shot on personal or smaller projects** and **report any issues and bugs you find**. It should be functional, but we still need people to try it out and report any bugs.

**NOTE:** Glistix's latest stable version currently tracks **Gleam v1.1.0,** meaning features and fixes from up to that Gleam version are available.

**NOTE:** Glistix is **an unofficial project** and is therefore **not affiliated with the Gleam project**.

## Sponsors

Please consider [supporting the team behind Gleam on GitHub](https://github.com/sponsors/lpil). Without their awesome project, Glistix wouldn't exist!

If you also want to support the Glistix project directly, feel free to [support me on GitHub](https://github.com/sponsors/PgBiel) as well!

## We need help!

If you like the project and know some Nix, **consider contributing!** At the time of writing, we have a few important issues open at [`glistix/stdlib`](https://github.com/glistix/stdlib) which could use some help. Additionally, feel free to explore issues at the Glistix compiler's repository (make sure to read the ["Contributing"](#contributing) section below).

## Installation

Glistix officially supports **Linux, MacOS and Windows.** (Note, however, that Nix doesn't support Windows yet, so you won't be able to test your projects, but `glistix build` should work at least.)

You can install Glistix in one of the following ways.

1. **From GitHub Releases (non-NixOS):** If you're not using NixOS, you can install Glistix by downloading the latest precompiled binary for your platform at [https://github.com/glistix/glistix/releases](https://github.com/glistix/glistix/releases).

2. **With Nix flakes (NixOS):** Invoke the command below in the command line to download, compile and run a specific release of Glistix - here the latest at the time of writing (v0.1.0).

    ```sh
    nix run 'github:Glistix/glistix/v0.1.0' -- --help
    ```

    To install permanently, you can either add `github:Glistix/glistix/v0.1.0` as an input to your system/Home Manager configuration, or use `nix profile`:

    ```sh
    nix profile install 'github:Glistix/glistix/v0.1.0'
    ```

3. **With Cargo:** You can use Cargo to compile and install Glistix's latest release (v0.1.0 at the time of writing):

    ```sh
    cargo install --git https://github.com/glistix/glistix --tag v0.1.0 --locked
    ```

## Table of Contents

- [Motivation](#motivation)
- [Features](#features)
    - [Compiling Gleam to Nix](#compiling-gleam-to-nix)
    - [Using Nix code from Gleam via FFI](#using-nix-code-from-gleam-via-ffi)
    - [Nix tools to compile and import Gleam code](#nix-tools-to-compile-and-import-gleam-code)
    - [Ports of several Gleam core packages to the Nix target](#ports-of-several-core-gleam-packages-to-the-nix-target)
- [Basic Usage](#basic-usage)
    - [Creating your first project](#creating-your-first-project)
    - [Using Glistix within editors / with Gleam LSP](#using-glistix-within-editors--with-gleam-lsp)
        - [Using `direnv`](#using-direnv)
- [Important notes](#important-notes)
- [Contributing](#contributing)
- [License](#license)


## Motivation

Glistix aims to **improve the development experience of working with more complex Nix code.** Whereas Nix can be fairly straightforward to use at its core (the language itself is tiny in terms of syntax), it can be observed in reality that writing more involved Nix code **can take considerable time and/or effort to get right** due to the **lack of type-safety in Nix,** which leads to **limitations in Nix LSPs** and often **hard to understand errors** which could have been avoided with static type-checking - that is, checking the correctness of your Nix code before you even try to evaluate it.

Personally, I (author of Glistix) have faced this kind of problem while **writing my own NixOS and Home Manager modules** - oftentimes the logic can get a bit involved and become particularly hard to debug and/or expand on without some form of static type-checking (and thus proper autocomplete and so on).

While there have been **several attempts to add some form of static type-checking** to Nix, very few have had great amounts of success so far. It always seemed like Nix needed some quite large (and thus unfeasible) changes to make it possible.

**That's where Gleam comes in**. Gleam is a **functional and statically-typed language** with **friendly syntax** and **great tooling** (including the compiler and its **helpful errors,** great support for **unit testing**, good **LSP support** across editors, and so on). What's more, **Gleam was designed for transpilation to other dynamically-typed languages** (namely Erlang and JavaScript). So, **it was the perfect match:** we could leverage the Gleam compiler and its tooling to **write more complicated logic with Gleam,** transpile that to Nix, and **attempt to keep complexity of actual Nix source code at a minimum** - the Nix code would then **focus on calling the transpiled Gleam code as much as possible**.

Thus **Glistix was born:** it adds a **Nix backend to the Gleam compiler**, alongside **auxiliary features and tooling to make Gleam easier to use within the Nix ecosystem.** The main goal is precisely to **integrate Gleam and Nix together as much as possible,** so that you may **write Gleam code and use it within Nix**, and **vice-versa** (Gleam code can invoke Nix functions through FFI).

Importantly, this is very much **inspired by [PureNix's approach](https://github.com/purenix-org/purenix)**. That project, which allows compiling PureScript to Nix, was one of the options considered before settling on Glistix's design, and has similar goals regarding type-checking and tooling. The **main difference** in our approach is **the usage of Gleam,** whose **simplicity in syntax, concepts and usage makes it stand out** among other languages. Nevertheless, make sure to check out PureNix as well if you're interested in learning more about alternatives - it is an awesome project as well!

**We hope you will enjoy using Glistix for your Nix and NixOS projects!**

## Features

Glistix provides the following features on top of Gleam's compiler and tooling:

### Compiling Gleam to Nix

You can **write Gleam code and have Glistix produce equivalent Nix code.** Almost all of the base language's features are supported (bar a few `BitArray` specifics).

The **generated Nix code** aims to be **as readable as possible,** and **attempts to follow common formatting standards** within the Nix ecosystem. In particular, we have **taken a few concepts from [Nix RFC 0166](https://github.com/NixOS/rfcs/pull/166)** and plan to be more conformant with said RFC in the future.

This also allows **using packages from the Gleam ecosystem** within Nix. Note that **pure Gleam packages are instantly usable,** while packages which require Erlang/JS FFI need to be manually ported.

- These ports are made possible thanks to the function attributes `@external(nix, "./file.nix", "functionName")` and `@target(nix)`. The first one allows **importing a function declared in Nix** (see the next section for an example), while the second allows **specializing a function's implementation for each target supported by Glistix** (one implementation for `@target(erlang)`, one for `@target(javascript)` and another for `@target(nix)`).

Consider this example code in `src/example.gleam`, added after running `glistix new example`:

```gleam
pub type Time {
  Seconds(Int)
  Minutes(amount: Int)
}

pub fn as_seconds(time: Time) -> Int {
  case time {
    Seconds(amount) -> amount
    Minutes(amount) -> 60 * amount
  }
}

pub fn main() {
  let example1 = Seconds(5)
  let example2 = Minutes(30)
  let seconds: Int = as_seconds(example1)
  let minutes = as_seconds(example2)

  #(seconds, minutes)
}
```

Running `glistix build`, Glistix will compile this to:

```nix
let
  Seconds = x0: { __gleamTag = "Seconds"; _0 = x0; };

  Minutes = amount: { __gleamTag = "Minutes"; inherit amount; };

  as_seconds =
    time:
    if time.__gleamTag == "Seconds" then let amount = time._0; in amount
    else let amount = time.amount; in 60 * amount;

  main =
    { }:
    let
      example1 = Seconds 5;
      example2 = Minutes 30;
      seconds = as_seconds example1;
      minutes = as_seconds example2;
    in
    [ seconds minutes ];
in
{ inherit Seconds Minutes as_seconds main; }
```

We can use `glistix run` (in this case with `-- --strict` so the tuple is fully evaluated), which runs `nix-instantiate`, to verify that `main { }` evaluates to `[ 5 1800 ]`.

Note that:
- Generated Nix code **is formatted and readable**;
- **Gleam records can be easily created and manipulated from Nix code** through the generated constructor functions and the resulting attribute sets (the field names are preserved, and have the form `_INDEX` when strictly positional);
- **Gleam's built-in types are adequately mapped to Nix's built-in types** where possible (there are notable non-obvious exceptions, such as `List` and `BitArray` - check the relevant book page for more info);
- By convention, **functions without arguments take an empty set `{ }` as argument** (you'd call `main { }` for example), whereas **functions with one or more arguments take them positionally** (for example, `f(a, b, c)` in Gleam translates to `f a b c` in Nix).

Make sure to check out the chapter on the Nix target of [the official Glistix book](https://glistix.github.io/book) for more information!

### Using Nix code from Gleam via FFI

You can add `.nix` files to your `src` and import them from Gleam. For example, consider the Gleam code below at `src/example.gleam`:

```gleam
// We will need some help from Nix to check
// if a string contains a comma. (In practice,
// you'd use Gleam's stdlib for this.)
@external(nix, "./ffi.nix", "containsComma")
fn contains_comma(string: String) -> Bool

/// Converts True/False to "Yes"/"No"
fn yes_or_no(boolean: Bool) -> String {
  case boolean {
    True -> "Yes"
    False -> "No"
  }
}

pub fn main() {
  let text = "abc,d"

  // Yes, you can use piping!
  contains_comma(text)
  |> yes_or_no
}
```

Also consider that we add `src/ffi.nix` as follows:

```nix
let
  containsComma = s: builtins.match ".*,.*" s != null;
in
{ inherit containsComma; }
```

With `glistix run` (which uses `nix-instantiate` to evaluate the main function), we get the expected output of `"Yes"` (`text` does indeed contain a comma).

### Nix tools to compile and import Gleam code

When you create a new Glistix project with `glistix new name`, you get `flake.nix`, `default.nix` and `shell.nix` for free (the latter two files [use `flake-compat`](https://github.com/edolstra/flake-compat) to work). With those, you can, for example, run `nix develop` (flakes) or `nix-shell` (no flakes) to enter a development shell where Glistix is installed.

Most importantly, the flake and `default.nix` both export a `lib.loadGlistixPackage { }` function (which optionally takes `{ system = some system; }`) which **allows Nix code to import modules from your Glistix project** (by default, it imports the names exported by the compiled `packagename.gleam` as an attribute set, but you can choose another module with `{ module = "some/module"; }`). It does that through a **Nix derivation which calls Glistix to compile your Gleam code automatically,** and then imports from the derivation's `out` output (which is basically the resulting `build` directory produced by the compiler). For example, if your project is at folder `./my/project`, Nix code can import your Glistix code as such:

```nix
let
  yourProject = (import ./my/project).lib.loadGlistixPackage { };
  mainOutput = yourProject.main { };
  # Using some exported function defined at 'src/package.gleam'
  otherFunction = yourProject.contains_comma "abc,d";
in { inherit mainOutput otherFunction; }  # The sky is the limit!
```

#### Build caching

You can optionally **cache the built Nix files in your repository** (or somewhere else) so Nix users **don't depend on the Glistix compiler to import your Glistix project** (if they don't have Glistix installed already, **the compiler will be compiled from scratch, which can take several minutes**). To do so, you can create an `output` folder in your repository and, **after each (relevant) build** with `glistix build --target nix`, run:

```sh
# Create destination
mkdir -p output/dev

# Copy just the resulting Nix files
# (IMPORTANT: -L flag used to deep copy symlinked folders in 'build')
cp -rL build/dev/nix -t output/dev

# Check it in so the flake can access the folder
git add output
```

The default `flake.nix` and `default.nix` files in Glistix projects will **automatically recognize** the newly-created `output` directory as the cached output folder, and `lib.loadGlistixPackage { }` **will use the contents of `output` instead of building your project on import**, which will be much faster for downstream Nix users (with the downside that `output` needs to be **manually updated** independently of your source Gleam code).
- If you want to use a directory other than `output`, or even e.g. GitHub Releases, make sure to change the relevant setting in the `flake.nix` file to point to a different directory (which can also be a derivation exporting the files).

For more information, check the [relevant chapter in the Glistix book.](https://glistix.github.io/book/recipes/import-from-nix.html)

### Ports of several core Gleam packages to the Nix target

The Glistix project officially maintains a few ports of core Gleam packages to work with Glistix and its Nix target. This includes the standard library (at [`glistix/stdlib`](https://github.com/glistix/stdlib)), the `gleeunit` test runner (at [`glistix/gleeunit`](https://github.com/glistix/gleeunit)), `json` (at [`glistix/json`](https://github.com/glistix/json)) and `birl` for datetime manipulation (at [`glistix/birl`](https://github.com/glistix/birl)), with more to come eventually.

## Basic Usage

### Creating your first project

0. Ensure you [have Glistix installed](#installation).
1. Create a new project with `glistix new name`.
2. Adjust `gleam.toml` to your liking; add dependencies from Hex with `glistix add name`. _(Git dependencies have to be added as local dependencies on Git submodules for now.)_
3. Modify the code at `src/name.gleam`. Have your main function return something useful, such as `5 + 5`.
4. You can use `glistix build` (or `glistix build --target nix` when not specified in your `gleam.toml`) to compile your Gleam code into a Nix expression.
5. Use `glistix run` (optionally with `--target nix` if not specified in your configuration) to check the output of `main { }`. This calls `nix-instantiate` by default.
6. Use `glistix test` to run tests in the `test/` directory using the [`glistix_gleeunit` test runner](https://github.com/glistix/gleeunit).

Great job! You now have a basic project up and running. You can **import its modules from Nix** by importing `default.nix` elsewhere (or adding your project's repository as an input to another `flake.nix` elsewhere) and using `lib.loadGlistixPackage { }` (or `{ module = "module/name"; }` to pick a module). You can also cache the build folder in your repository by copying it to `output`, as [described in the previous sections](#build-caching).

[Check out the Glistix book](https://glistix.github.io/book/getting-started/basic-usage.html) for more information!

### Using Glistix within editors / with Gleam LSP

To properly use Glistix with editors and LSPs, you will need to configure them to **use the `glistix` program instead of `gleam`** for LSP functionality. Otherwise, they won't recognize Glistix-only syntax, specifically `@target(nix)` and `@external(nix, ..., ...)`, in your code.

The VSCode Extension for Gleam does not have this option at the moment, so **you will have to alias `glistix` as `gleam` in your system while using VSCode** (or other editors in a similar situation). Note that `glistix` **also supports Gleam's Erlang and JavaScript targets,** maintained at the upstream compiler (we currently aim to be compatible with existing non-Nix projects as much as possible - **please open an issue if something breaks**).

#### Using `direnv`

If you'd like, **you can also apply this alias in a per-project (or per-folder) basis through `direnv`.** You can do this by linking the `glistix` executable to a folder, naming the link as `gleam` (e.g. through `ln -s $(which glistix) -T /chosen/folder/gleam`), and then adding an `.envrc` file to your project with contents such as

```sh
export PATH="/chosen/folder:$PATH"
```

With that in place, you can, for example install the [`direnv` VSCode extension](https://marketplace.visualstudio.com/items?itemName=mkhl.direnv) (which requires `direnv` to be installed in your system) which will **automatically load that alias each time you open the project,** and the Gleam LSP extension will properly use the Glistix executable while you're working on your project, giving correct diagnostics.

The above applies to other editors supporting `direnv` as well. For those which don't, you will have to run `direnv` through the command line and open the editor through it (or have some other solution so that `glistix` is aliased to `gleam` in the `PATH`).

## Important notes

### Overriding packages incompatible with Nix

While many Gleam packages can, in principle, be used with Glistix projects, it is also a fact that **a large number of Gleam packages depend on Gleam's usual targets** (Erlang and JavaScript) by depending (heavily or not) on FFI. This makes using them with Glistix a non-starter at first. The way around this is to create a **fork with Nix support** (adding Nix FFI together with Erlang and JavaScript FFI), and then use it through **Git submodules** as local dependencies of your root package (at least while Gleam doesn't support Git dependencies).

### Tail-call optimization

Glistix **does not implement tail-call optimization** as this is not yet implemented by Nix itself, so make sure to **avoid recursion over large data structures** (e.g. a `List` with thousands of elements) or, in general, excessive recursion to avoid hitting memory limits.

### Strictness and laziness

Nix is lazy by default, which means that values aren't evaluated until they are used. This means that side effects are often suppressed when the values that trigger them aren't used. However, Glistix will **always evaluate discarded expressions.** That is, any expressions which appear in the body of your function but are not bound to any variables **are evaluated before the function's return value** through Nix's `builtins.seq` function, which is useful when side-effects are needed. **The same applies to `let assert` statements** (the assertions always run).

For more details on the topics above, see the ["Limitations" page of the Glistix book.](https://glistix.github.io/book/about/limitations.html)

## Contributing

Please **discuss first** (open an issue) before opening a large PR. Additionally, make sure to [check out the book](https://glistix.github.io/book/) for useful information on how the compiler works.

Please note that we generally try to follow an **upstream-first policy**. This means that **any feature ideas which are not related to Nix** should generally **be brought to the [the upstream Gleam repository](https://github.com/gleam-lang/gleam) first**; if accepted there, the feature should be implemented (/contributed) upstream. Otherwise, we can consider implementing it on Glistix if it would significantly improve the experience of using Glistix, especially regarding usage with Nix, but not before proper discussions. The idea is to not only keep up with the Gleam ecosystem at large, but also to **ensure most improvements also benefit users of the upstream Gleam compiler.**

## License

Glistix is licensed under Apache-2.0, and is largely based on work done by the Gleam team, which is also licensed under Apache-2.0.

Glistix is an unofficial project and is not affiliated with the team behind the Gleam compiler.
