use camino::{Utf8Path, Utf8PathBuf};
use clap::ValueEnum;
use glistix_core::{
    erlang,
    error::{Error, FileIoAction, FileKind, InvalidProjectNameReason},
    parse, Result,
};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::{env, io::Write};
use strum::{Display, EnumIter, EnumString, IntoEnumIterator, VariantNames};

#[cfg(test)]
mod tests;

use crate::{fs::get_current_directory, NewOptions};

const GLEAM_STDLIB_REQUIREMENT: &str = ">= 0.34.0 and < 2.0.0";
const GLEEUNIT_REQUIREMENT: &str = ">= 1.0.0 and < 2.0.0";
#[allow(dead_code)]
const ERLANG_OTP_VERSION: &str = "27.1.2";
#[allow(dead_code)]
const REBAR3_VERSION: &str = "3";
#[allow(dead_code)]
const ELIXIR_VERSION: &str = "1.15.4";

const GLISTIX_STDLIB_URL: &str = "https://github.com/glistix/stdlib";

#[derive(
    Debug, Serialize, Deserialize, Display, EnumString, VariantNames, ValueEnum, Clone, Copy,
)]
#[strum(serialize_all = "lowercase")]
#[clap(rename_all = "lower")]
pub enum Template {
    #[clap(skip)]
    Lib,
    Erlang,
    JavaScript,
    Nix,
}

#[derive(Debug)]
pub struct Creator {
    root: Utf8PathBuf,
    src: Utf8PathBuf,
    test: Utf8PathBuf,
    github: Utf8PathBuf,
    workflows: Utf8PathBuf,
    external: Utf8PathBuf,
    external_stdlib: Utf8PathBuf,
    #[allow(dead_code)]
    gleam_version: &'static str,
    options: NewOptions,
    project_name: String,
}

#[derive(EnumIter, PartialEq, Eq, Debug, Hash)]
enum FileToCreate {
    Readme,
    Gitignore,
    SrcModule,
    TestModule,
    GleamToml,
    GithubCi,
    NixFlake,
    NixDefault,
    NixShell,
}

impl FileToCreate {
    pub fn location(&self, creator: &Creator) -> Utf8PathBuf {
        let project_name = &creator.project_name;

        match self {
            Self::Readme => creator.root.join(Utf8PathBuf::from("README.md")),
            Self::Gitignore => creator.root.join(Utf8PathBuf::from(".gitignore")),
            Self::SrcModule => creator
                .src
                .join(Utf8PathBuf::from(format!("{project_name}.gleam"))),
            Self::TestModule => creator
                .test
                .join(Utf8PathBuf::from(format!("{project_name}_test.gleam"))),
            Self::GleamToml => creator.root.join(Utf8PathBuf::from("gleam.toml")),
            Self::GithubCi => creator.workflows.join(Utf8PathBuf::from("test.yml")),
            Self::NixFlake => creator.root.join(Utf8PathBuf::from("flake.nix")),
            Self::NixDefault => creator.root.join(Utf8PathBuf::from("default.nix")),
            Self::NixShell => creator.root.join(Utf8PathBuf::from("shell.nix")),
        }
    }

    pub fn contents(&self, creator: &Creator) -> Option<String> {
        let project_name = &creator.project_name;
        let skip_git = creator.options.skip_git;
        let skip_github = creator.options.skip_github;
        let target = match creator.options.template {
            Template::JavaScript => "target = \"javascript\"\n",
            Template::Lib | Template::Nix => "target = \"nix\"\n",
            Template::Erlang => "target = \"erlang\"\n",
        };

        match self {
            Self::Readme => Some(format!(
                r#"# {project_name}

[![Package Version](https://img.shields.io/hexpm/v/{project_name})](https://hex.pm/packages/{project_name})
[![Hex Docs](https://img.shields.io/badge/hex-docs-ffaff3)](https://hexdocs.pm/{project_name}/)

```sh
glistix add {project_name}@1
```
```gleam
import {project_name}

pub fn main() {{
  // TODO: An example of the project in use
}}
```

**Note:** This is a Glistix project, and as such may require the
[Glistix compiler](https://github.com/glistix/glistix) to be used.

Further documentation can be found at <https://hexdocs.pm/{project_name}>.

## Importing from Nix

To import this project from Nix, first fetch its source (through a Flake input,
using `builtins.fetchGit`, cloning to a local path, or some other way), import
the Flake or the `default.nix` file, and run `lib.loadGlistixPackage {{ }}`.
For example:

```nix
let
  # Assuming the project was cloned to './path/to/project'
  {project_name} = import ./path/to/project;
  # Use 'loadGlistixPackage {{ module = "module/name"; }}' to pick a module
  package = {project_name}.lib.loadGlistixPackage {{ }};
  result = package.main {{ }};
in result
```

## Development

```sh
nix develop   # Optional: Spawn a shell with glistix
glistix run   # Run the project
glistix test  # Run the tests
```
"#,
            )),

            Self::Gitignore if !skip_git => Some(
                "*.beam
*.ez
/build
erl_crash.dump
_gleam_artefacts
"
                .into(),
            ),

            Self::SrcModule => Some(format!(
                r#"import gleam/io

pub fn main() {{
  io.println("Hello from {project_name}!")
}}
"#,
            )),

            Self::TestModule => Some(
                r#"import gleeunit
import gleeunit/should

pub fn main() {
  gleeunit.main()
}

// gleeunit test functions end in `_test`
pub fn hello_world_test() {
  1
  |> should.equal(1)
}
"#
                .into(),
            ),

            Self::GleamToml => Some(format!(
                r#"name = "{project_name}"
version = "1.0.0"
{target}
# Fill out these fields if you intend to generate HTML documentation or publish
# your project to the Hex package manager.
#
# description = ""
# licences = ["Apache-2.0"]
# repository = {{ type = "github", user = "", repo = "" }}
# links = [{{ title = "Website", href = "" }}]
#
# For a full reference of all the available options, you can have a look at
# https://gleam.run/writing-gleam/gleam-toml/.

[dependencies]
# Run 'git submodule add --name stdlib -- https://github.com/glistix/stdlib external/stdlib'
# to clone Glistix's stdlib patch to the local path specified below. This is needed so stdlib
# will work on the Nix target. Hex dependents will use the stdlib version specified below,
# in [glistix.preview.hex-patch], instead.
gleam_stdlib = {{ path = "./external/stdlib" }}

[dev-dependencies]
glistix_gleeunit = "{GLEEUNIT_REQUIREMENT}"

# The [glistix.preview] namespace contains useful settings which will be needed
# during Glistix beta. In the future, it's likely these won't be necessary
# anymore.
[glistix.preview]
# If you're patching a package using a local dependency/Git submodule and you
# get a local dependency conflict error, add it to the list below.
local-overrides = ["gleam_stdlib"]

# The section below allows publishing your package to Hex despite having
# local dependencies, by declaring that you depend on another Hex package
# instead.
# This is needed to be able to patch stdlib etc. locally during development
# and at the same time publish to Hex without the patch.
# The section below should only be used for this purpose. Please do not abuse
# this feature, as it is mostly a temporary workaround while Gleam doesn't have
# a proper dependency patching system.
[glistix.preview.hex-patch]
gleam_stdlib = "{GLEAM_STDLIB_REQUIREMENT}"
"#,
            )),

            Self::GithubCi if !skip_git && !skip_github => Some(
                r#"name: test

on:
  push:
    branches:
      - master
      - main
  pull_request:

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          submodules: 'recursive'
      - uses: cachix/install-nix-action@v26
      - uses: DeterminateSystems/magic-nix-cache-action@v4
      - name: Ensure flake.lock was committed
        run: ls flake.lock
      - run: nix flake check -L
      - run: nix build -L
      - run: glistix deps download
        shell: nix develop --command bash -e {0}
      - run: glistix test
        shell: nix develop --command bash -e {0}
      - run: glistix format --check src test
        shell: nix develop --command bash -e {0}
"#.into(),
            ),
            Self::GithubCi | Self::Gitignore => None,
            Self::NixFlake => Some(format!(
                r#"# Make sure to run "nix flake update" at least once to generate your flake.lock.
# Run your main function from Nix by importing this flake as follows:
#
# let
#   yourProject = builtins.getFlake "URL";  # for example
#   mainResult = (yourProject.lib.loadGlistixPackage {{}}).main {{}};
# in doSomethingWith mainResult;
#
# See below to customize your flake.
{{
  description = "{project_name} - A Glistix project";

  inputs = {{
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    systems.url = "github:nix-systems/default";

    # Used for default.nix and shell.nix.
    flake-compat = {{
      url = "github:edolstra/flake-compat";
      flake = false;
    }};

    # Pick your Glistix version here.
    glistix.url = "github:glistix/glistix/v0.6.0";

    # Submodules
    # Add any submodules which you use as dependencies here,
    # and then add them to "submodules = [ ... ]" below.
    stdlib = {{
      url = "github:glistix/stdlib";
      flake = false;
    }};
  }};

  outputs =
    inputs@{{ self, nixpkgs, flake-parts, systems, glistix, stdlib, ... }}:
    let
      # --- CUSTOMIZATION PARAMETERS ---

      # Add any source files to keep when building your Glistix package here.
      # This includes anything which might be necessary at build time.
      sourceFiles = [
        "gleam.toml"
        "manifest.toml" # make sure to build locally at least once so Glistix generates this file
        "src"
        "test"
        "external" # cloned package repositories
        "output" # checked-in build output, if it exists
        "priv" # assets and other relevant files
        ".gitignore"
        ".gitmodules"
      ];

      # Submodules have to be specified here, even if they are cloned locally.
      # Add them as inputs to the flake, and then specify where to clone them to
      # during build below.
      submodules = [
        {{
          src = stdlib;
          dest = "external/stdlib";
        }}
      ];

      # If you cache your build output, this will specify the path
      # 'loadGlistixPackage' will check to find compiled Nix files in by default.
      # You usually don't have to change this, unless you cache your output in a
      # folder other than ./output, or if your output is fetched through a derivation
      # (e.g. `builtins.fetchGit`).
      # Don't forget to add the path to "sourceFiles" above if it's in the repo!
      outputPath = src + "/output";

      # Set this to 'true' if you created an 'output' folder where you're storing
      # build outputs and you'd like to ensure Nix consumers will use it.
      # It will be used even if this is 'false', but Glistix will fallback to
      # building your package from scratch upon load if the output folder is
      # missing. If you set this to 'true', it will error instead.
      forceLoadFromOutput = false;

      # --- IMPLEMENTATION ---

      inherit (nixpkgs) lib;
      glistixLib = glistix.lib;

      # Filter source files to only include the given files and folders.
      src = lib.cleanSourceWith {{
        filter = path: type:
          builtins.any (accepted: lib.path.hasPrefix (./. + "/${{accepted}}") (/. + path)) sourceFiles;
        src = ./.;
      }};

      # Prepare call to 'buildGlistixPackage', allowing for overrides.
      buildGlistixPackage =
        args@{{ system, ... }}:
        let
          inherit (glistix.builders.${{system}}) buildGlistixPackage;
          overrides = builtins.removeAttrs args [ "system" ];
          builderArgs = {{
            inherit src submodules;
          }} // overrides;
        in
        buildGlistixPackage builderArgs;

      # Prepare the call to 'loadGlistixPackage'. This is used to
      # easily run your Gleam code compiled to Nix from within Nix.
      #
      # This will try to read compiled code at 'output/dev/nix',
      # thus not invoking the Glistix compiler at all if possible,
      # avoiding the need to compile, install and run it.
      # If that path does not exist, however, uses Glistix to build your
      # Gleam package from scratch instead, using the derivation
      # created further below (exported by this flake under
      # the 'packages' output).
      #
      # Specify 'forceLoadFromOutput = true;' above to opt into erroring
      # if the 'output/dev/nix' folder isn't found instead of invoking
      # the Glistix compiler.
      #
      # Pass 'system' to use the derivation for that system for compilation.
      # Pass 'glistix' to override the glistix derivation used for compilation.
      # Other arguments are passed through to Glistix's 'loadGlistixPackage'.
      # For example, 'lib.loadGlistixPackage {{ module = "ops/create"; }}'
      # will load what's exported by your package's 'ops/create' module
      # as an attribute set.
      loadGlistixPackage =
        args@{{
          system ? builtins.currentSystem or null,
          glistix ? null,
          ...
        }}:
        let
          derivation =
            if forceLoadFromOutput || system == null then
              null
            else if glistix != null then
              buildGlistixPackage {{ inherit system glistix; }}
            else
              self.packages.${{system}}.default or null;

          overrides = builtins.removeAttrs args [
            "system"
            "glistix"
          ];
          loaderArgs = {{
            inherit src derivation;
            output = outputPath;
          }} // overrides;
        in
        glistixLib.loadGlistixPackage loaderArgs;

    in flake-parts.lib.mkFlake {{ inherit inputs; }} {{
      systems = import systems;

      flake = {{ lib = {{ inherit loadGlistixPackage; }}; }};

      perSystem = {{ self', pkgs, lib, system, ... }}:
        let
          # This derivation will build Glistix itself if needed
          # (using Rust), and then use Glistix to build this particular
          # package into Nix files.
          # The derivation's "out" output will contain the resulting
          # 'build' directory. You can use
          #   "${{derivation}}/${{derivation.glistixMain}}"
          # for a path to this package's main '.nix' file, which can
          # be imported through Nix's `import`.
          package = buildGlistixPackage {{ inherit system; }};
        in {{
          packages.default = package;

          # Run 'nix develop' to create a shell where 'glistix' is available.
          devShells.default = pkgs.mkShell {{
            nativeBuildInputs = [ glistix.packages.${{system}}.default ];
          }};
        }};
    }};
}}
"#,
            )),
            Self::NixDefault => Some(
                r#"# This will let Nix users import from your repository without flakes.
# Exposes the flake's outputs.
# Source: https://wiki.nixos.org/wiki/Flakes#Using_flakes_with_stable_Nix
#
# Usage:
#
# let
#   yourProject = import (builtins.fetchurl { url = "your url"; sha256 = ""; });  # for example
#   mainResult = (yourProject.lib.loadGlistixPackage {}).main {};
# in doSomethingWith mainResult;
(import (
  let
    lock = builtins.fromJSON (builtins.readFile ./flake.lock);
  in fetchTarball {
    url = "https://github.com/edolstra/flake-compat/archive/${lock.nodes.flake-compat.locked.rev}.tar.gz";
    sha256 = lock.nodes.flake-compat.locked.narHash; }
) {
  src = ./.;
}).defaultNix
"#.into()
            ),
            Self::NixShell => Some(
                r#"# This exposes the dev shell declared in the flake.
# Running `nix-shell` will thus be equivalent to `nix develop`.
# Source: https://wiki.nixos.org/wiki/Flakes#Using_flakes_with_stable_Nix
#
# Usage: Run `nix-shell`, and the `glistix` command will be available
# for you to use.
(import (
  let
    lock = builtins.fromJSON (builtins.readFile ./flake.lock);
  in fetchTarball {
    url = "https://github.com/edolstra/flake-compat/archive/${lock.nodes.flake-compat.locked.rev}.tar.gz";
    sha256 = lock.nodes.flake-compat.locked.narHash; }
) {
  src = ./.;
}).shellNix
"#.into()
            ),
        }
    }
}

impl Creator {
    fn new(options: NewOptions, gleam_version: &'static str) -> Result<Self, Error> {
        let project_name = if let Some(name) = options.name.clone() {
            name
        } else {
            get_foldername(&options.project_root)?
        }
        .trim()
        .to_string();

        validate_name(&project_name)?;

        let root = get_current_directory()?.join(&options.project_root);
        let src = root.join("src");
        let test = root.join("test");
        let github = root.join(".github");
        let workflows = github.join("workflows");
        // External folder: we will clone stdlib there if possible.
        let external = root.join("external");
        // Use a relative path as that is what is recorded to '.gitmodules'.
        let external_stdlib = Utf8PathBuf::from("external/stdlib");
        let me = Self {
            root: root.clone(),
            src,
            test,
            github,
            workflows,
            external,
            external_stdlib,
            gleam_version,
            options,
            project_name,
        };

        validate_root_folder(&me)?;

        Ok(me)
    }

    fn run(&self) -> Result<()> {
        crate::fs::mkdir(&self.root)?;
        crate::fs::mkdir(&self.src)?;
        crate::fs::mkdir(&self.test)?;
        crate::fs::mkdir(&self.external)?;

        if !self.options.skip_git && !self.options.skip_github {
            crate::fs::mkdir(&self.github)?;
            crate::fs::mkdir(&self.workflows)?;
        }

        if self.options.skip_git {
            eprintln!(
                "WARNING: Skipping Git procedures. You will have to manually clone the \
Glistix patch for 'stdlib'. If you create a Git repository at the new project's directory, \
you can do so with the command \
'git submodule add --name stdlib -- https://github.com/glistix/stdlib external/stdlib'. \
Otherwise, you can use 'git clone' instead of 'git submodule add --name stdlib'."
            )
        } else {
            crate::fs::git_init(&self.root)?;
            crate::fs::git_submodule_add(
                "stdlib",
                GLISTIX_STDLIB_URL,
                &self.root,
                &self.external_stdlib,
            )?;
        }

        match self.options.template {
            Template::Lib | Template::Erlang | Template::JavaScript | Template::Nix => {
                for file in FileToCreate::iter() {
                    let path = file.location(self);
                    if let Some(contents) = file.contents(self) {
                        write(path, &contents)?;
                    }
                }
            }
        }

        Ok(())
    }
}

pub fn create(options: NewOptions, version: &'static str) -> Result<()> {
    let creator = Creator::new(options.clone(), version)?;

    creator.run()?;

    let cd_folder = if options.project_root == "." {
        "".into()
    } else {
        format!("\tcd {}\n", creator.options.project_root)
    };

    println!(
        "Your Glistix project {} has been successfully created.
The project can be compiled and tested by running these commands:

{}\tglistix test
",
        creator.project_name, cd_folder,
    );
    Ok(())
}

fn write(path: Utf8PathBuf, contents: &str) -> Result<()> {
    let mut f = File::create(&path).map_err(|err| Error::FileIo {
        kind: FileKind::File,
        path: path.clone(),
        action: FileIoAction::Create,
        err: Some(err.to_string()),
    })?;

    f.write_all(contents.as_bytes())
        .map_err(|err| Error::FileIo {
            kind: FileKind::File,
            path,
            action: FileIoAction::WriteTo,
            err: Some(err.to_string()),
        })?;
    Ok(())
}

fn validate_root_folder(creator: &Creator) -> Result<(), Error> {
    let mut duplicate_files: Vec<Utf8PathBuf> = Vec::new();

    for t in FileToCreate::iter() {
        let full_path = t.location(creator);
        let content = t.contents(creator);
        if full_path.exists() && content.is_some() {
            duplicate_files.push(full_path);
        }
    }

    if !duplicate_files.is_empty() {
        return Err(Error::OutputFilesAlreadyExist {
            file_names: duplicate_files,
        });
    }

    Ok(())
}

fn validate_name(name: &str) -> Result<(), Error> {
    if name.starts_with("gleam_") {
        Err(Error::InvalidProjectName {
            name: name.to_string(),
            reason: InvalidProjectNameReason::GleamPrefix,
        })
    } else if erlang::is_erlang_reserved_word(name) {
        Err(Error::InvalidProjectName {
            name: name.to_string(),
            reason: InvalidProjectNameReason::ErlangReservedWord,
        })
    } else if erlang::is_erlang_standard_library_module(name) {
        Err(Error::InvalidProjectName {
            name: name.to_string(),
            reason: InvalidProjectNameReason::ErlangStandardLibraryModule,
        })
    } else if parse::lexer::str_to_keyword(name).is_some() {
        Err(Error::InvalidProjectName {
            name: name.to_string(),
            reason: InvalidProjectNameReason::GleamReservedWord,
        })
    } else if name == "gleam" {
        Err(Error::InvalidProjectName {
            name: name.to_string(),
            reason: InvalidProjectNameReason::GleamReservedModule,
        })
    } else if !regex::Regex::new("^[a-z][a-z0-9_]*$")
        .expect("new name regex could not be compiled")
        .is_match(name)
    {
        Err(Error::InvalidProjectName {
            name: name.to_string(),
            reason: InvalidProjectNameReason::Format,
        })
    } else {
        Ok(())
    }
}

fn get_foldername(path: &str) -> Result<String, Error> {
    match path {
        "." => env::current_dir()
            .expect("invalid folder")
            .file_name()
            .and_then(|x| x.to_str())
            .map(ToString::to_string)
            .ok_or(Error::UnableToFindProjectRoot {
                path: path.to_string(),
            }),
        _ => Utf8Path::new(path)
            .file_name()
            .map(ToString::to_string)
            .ok_or(Error::UnableToFindProjectRoot {
                path: path.to_string(),
            }),
    }
}
