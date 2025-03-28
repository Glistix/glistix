use camino::{Utf8Path, Utf8PathBuf};
use clap::ValueEnum;
use glistix_core::{
    erlang, error,
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

#[allow(dead_code)]
const GLEAM_STDLIB_REQUIREMENT: &str = ">= 0.44.0 and < 2.0.0";
const GLISTIX_STDLIB_REQUIREMENT: &str = ">= 0.34.0 and < 2.0.0";
const GLEEUNIT_REQUIREMENT: &str = ">= 1.0.0 and < 2.0.0";
#[allow(dead_code)]
const ERLANG_OTP_VERSION: &str = "27.1.2";
#[allow(dead_code)]
const REBAR3_VERSION: &str = "3";
#[allow(dead_code)]
const ELIXIR_VERSION: &str = "1";
const GLISTIX_HANDBOOK_LINK: &str = "https://glistix.github.io/book/";

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
# Use Glistix-maintained fork of the Gleam standard library with support for the
# Nix target.
#
# Consider depending on gleam_stdlib instead if you're publishing a package to
# non-Glistix users on Hex (Glistix users can still patch gleam_stdlib on their
# projects). Otherwise, you can depend on the fork directly.
glistix_stdlib = "{GLISTIX_STDLIB_REQUIREMENT}"

[dev-dependencies]
glistix_gleeunit = "{GLEEUNIT_REQUIREMENT}"

# The [glistix.preview] namespace contains useful settings which will be needed
# during Glistix beta. In the future, we hope these won't be necessary anymore.
# None of the settings below are recognized by the official Gleam compiler.
#
# For more information on those options, check out the Glistix handbook at
# this link: {GLISTIX_HANDBOOK_LINK}

# The section below allows replacing transitive dependencies with other packages,
# such as forks providing support for the Nix target. For example, `gleam_stdlib`
# does not support the Nix target, so we replace it with the `glistix_stdlib` fork.
# Replacing Hex packages with local packages is also supported (and Git packages
# in a future Glistix version).
#
# Specifying a version (or local path) is always required.
#
# NOTE: This section is ignored when publishing to Hex. It is only read on top-level
# Glistix projects. However, it can still be useful on packages to allow running unit
# tests, so you can keep this here regardless. Just keep this in mind if a user
# complains about a missing dependency: they are responsible for patching.
[glistix.preview.patch]
# Replaces 'gleam_stdlib' with 'glistix_stdlib' on all transitive dependencies.
# This is needed so stdlib will work on the Nix target.
gleam_stdlib = {{ name = "glistix_stdlib", version = "{GLISTIX_STDLIB_REQUIREMENT}" }}
# otherpkg = {{ version = "3.4.5" }} # replace with another Hex version
# anotherpkg = {{ path = "./external/submodule1" }} # replace with local package
# renamedpkg = {{ name = "differentpkg", path = "./external/submodule2" }}
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
    glistix.url = "github:glistix/glistix/v0.7.0";

    # Submodules
    # Add any submodules which you use as dependencies here,
    # and then add them to "submodules = [ ... ]" below.
    # This is optional and only necessary to use dependencies
    # from outside Hex, such as Git dependencies.

    # submodule1 = {{
    #   url = "github:author/submodule1";
    #   flake = false;
    # }};
  }};

  outputs =
    inputs@{{ self, nixpkgs, flake-parts, systems, glistix, ... }}:
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
        # {{
        #   src = inputs.submodule1;
        #   dest = "external/submodule1";
        # }}
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
        let project_name = get_valid_project_name(options.name.clone(), &options.project_root)?;
        let root = get_current_directory()?.join(&options.project_root);
        let src = root.join("src");
        let test = root.join("test");
        let github = root.join(".github");
        let workflows = github.join("workflows");
        let me = Self {
            root: root.clone(),
            src,
            test,
            github,
            workflows,
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

        if !self.options.skip_git && !self.options.skip_github {
            crate::fs::mkdir(&self.github)?;
            crate::fs::mkdir(&self.workflows)?;
        }

        if !self.options.skip_git {
            crate::fs::git_init(&self.root)?;
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

Please note that Glistix is still in beta and may not have all features you need.

If you need help, check out the Glistix handbook at the website below:
{GLISTIX_HANDBOOK_LINK}
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
    } else if regex::Regex::new("^[a-z][a-z0-9_]*$")
        .expect("failed regex to match valid name format")
        .is_match(name)
    {
        Ok(())
    } else if regex::Regex::new("^[a-zA-Z][a-zA-Z0-9_]*$")
        .expect("failed regex to match valid but non-lowercase name format")
        .is_match(name)
    {
        Err(Error::InvalidProjectName {
            name: name.to_string(),
            reason: InvalidProjectNameReason::FormatNotLowercase,
        })
    } else {
        Err(Error::InvalidProjectName {
            name: name.to_string(),
            reason: InvalidProjectNameReason::Format,
        })
    }
}

fn suggest_valid_name(invalid_name: &str, reason: &InvalidProjectNameReason) -> Option<String> {
    match reason {
        InvalidProjectNameReason::GleamPrefix => match invalid_name.strip_prefix("gleam_") {
            Some(stripped) if invalid_name != "gleam_" => {
                let suggestion = stripped.to_string();
                match validate_name(&suggestion) {
                    Ok(_) => Some(suggestion),
                    Err(_) => None,
                }
            }
            _ => None,
        },
        InvalidProjectNameReason::ErlangReservedWord => Some(format!("{}_app", invalid_name)),
        InvalidProjectNameReason::ErlangStandardLibraryModule => {
            Some(format!("{}_app", invalid_name))
        }
        InvalidProjectNameReason::GleamReservedWord => Some(format!("{}_app", invalid_name)),
        InvalidProjectNameReason::GleamReservedModule => Some(format!("{}_app", invalid_name)),
        InvalidProjectNameReason::FormatNotLowercase => Some(invalid_name.to_lowercase()),
        InvalidProjectNameReason::Format => {
            let suggestion = regex::Regex::new(r"[^a-z0-9]")
                .expect("failed regex to match any non-lowercase and non-alphanumeric characters")
                .replace_all(&invalid_name.to_lowercase(), "_")
                .to_string();

            let suggestion = regex::Regex::new(r"_+")
                .expect("failed regex to match consecutive underscores")
                .replace_all(&suggestion, "_")
                .to_string();

            match validate_name(&suggestion) {
                Ok(_) => Some(suggestion),
                Err(_) => None,
            }
        }
    }
}

fn get_valid_project_name(name: Option<String>, project_root: &str) -> Result<String, Error> {
    let initial_name = match name {
        Some(name) => name,
        None => get_foldername(project_root)?,
    }
    .trim()
    .to_string();

    let invalid_reason = match validate_name(&initial_name) {
        Ok(_) => return Ok(initial_name),
        Err(Error::InvalidProjectName { reason, .. }) => reason,
        Err(error) => return Err(error),
    };

    let suggested_name = match suggest_valid_name(&initial_name, &invalid_reason) {
        Some(suggested_name) => suggested_name,
        None => {
            return Err(Error::InvalidProjectName {
                name: initial_name,
                reason: invalid_reason,
            })
        }
    };
    let prompt_for_suggested_name = error::format_invalid_project_name_error(
        &initial_name,
        &invalid_reason,
        &Some(suggested_name.clone()),
    );
    match crate::cli::confirm(&prompt_for_suggested_name)? {
        true => Ok(suggested_name),
        false => Err(Error::InvalidProjectName {
            name: initial_name,
            reason: invalid_reason,
        }),
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
