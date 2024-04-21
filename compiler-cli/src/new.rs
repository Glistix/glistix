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
const ERLANG_OTP_VERSION: &str = "26.0.2";
const REBAR3_VERSION: &str = "3";
const ELIXIR_VERSION: &str = "1.15.4";

const GLISTIX_STDLIB_URL: &str = "https://github.com/glistix/stdlib";
const GLISTIX_GLEEUNIT_URL: &str = "https://github.com/glistix/gleeunit";

#[derive(
    Debug, Serialize, Deserialize, Display, EnumString, VariantNames, ValueEnum, Clone, Copy,
)]
#[strum(serialize_all = "kebab_case")]
pub enum Template {
    Lib,
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
    external_gleeunit: Utf8PathBuf,
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
        }
    }

    pub fn contents(&self, creator: &Creator) -> Option<String> {
        let project_name = &creator.project_name;
        let skip_git = creator.options.skip_git;
        let skip_github = creator.options.skip_github;
        let gleam_version = creator.gleam_version;

        match self {
            Self::Readme => Some(format!(
                r#"# {project_name}

[![Package Version](https://img.shields.io/hexpm/v/{project_name})](https://hex.pm/packages/{project_name})
[![Hex Docs](https://img.shields.io/badge/hex-docs-ffaff3)](https://hexdocs.pm/{project_name}/)

```sh
gleam add {project_name}
```
```gleam
import {project_name}

pub fn main() {{
  // TODO: An example of the project in use
}}
```

Further documentation can be found at <https://hexdocs.pm/{project_name}>.

## Development

```sh
gleam run   # Run the project
gleam test  # Run the tests
gleam shell # Run an Erlang shell
```
"#,
            )),

            Self::Gitignore if !skip_git => Some(
                "*.beam
*.ez
/build
erl_crash.dump
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
target = "nix"

# Fill out these fields if you intend to generate HTML documentation or publish
# your project to the Hex package manager.
#
# description = ""
# licences = ["Apache-2.0"]
# repository = {{ type = "github", user = "username", repo = "project" }}
# links = [{{ title = "Website", href = "https://gleam.run" }}]
#
# For a full reference of all the available options, you can have a look at
# https://gleam.run/writing-gleam/gleam-toml/.

[dependencies]
# Run 'git submodule add --name stdlib -- https://github.com/glistix/stdlib external/stdlib'
# to clone Glistix's stdlib patch to the local path specified below.
gleam_stdlib = {{ path = "./external/stdlib" }}
# Uncomment (i.e. remove '#' from) the line below when publishing your package to Hex, as Hex
# packages cannot have dependencies on local packages.
# gleam_stdlib = "{GLEAM_STDLIB_REQUIREMENT}"

[dev-dependencies]
# Run 'git submodule add --name gleeunit -- https://github.com/glistix/gleeunit external/gleeunit'
# to clone Glistix's gleeunit patch to the local path specified below.
gleeunit = {{ path = "./external/gleeunit" }}
# Uncomment the line below if needed.
# gleeunit = "{GLEEUNIT_REQUIREMENT}"
"#,
            )),

            Self::GithubCi if !skip_git && !skip_github => Some(format!(
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
      - uses: erlef/setup-beam@v1
        with:
          otp-version: "{ERLANG_OTP_VERSION}"
          gleam-version: "{gleam_version}"
          rebar3-version: "{REBAR3_VERSION}"
          # elixir-version: "{ELIXIR_VERSION}"
      - run: gleam deps download
      - run: gleam test
      - run: gleam format --check src test
"#,
            )),
            Self::GithubCi | Self::Gitignore => None,
            Self::NixFlake => Some(format!(
                r#"
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

    # Pick your Glistix version here.
    glistix.url = "github:glistix/glistix/v0.1.0";

    # Submodules
    # Add any submodules which you use as dependencies here,
    # and then add them to "submodules = [ ... ]" below.
    stdlib = {{
      url = "github:glistix/stdlib";
      flake = false;
    }};
    gleeunit = {{
      url = "github:glistix/gleeunit";
      flake = false;
    }};
  }};

  outputs =
    inputs@{{ self, nixpkgs, flake-parts, systems, glistix, stdlib, gleeunit, ... }}:
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
        {{
          src = gleeunit;
          dest = "external/gleeunit";
        }}
      ];

      # Set this to 'true' if you created an 'output' folder where you're storing
      # build outputs and you'd like to ensure Nix consumers will use it.
      # It will be used even if this is 'false', but Glistix will fallback to
      # building your package from scratch upon load if the output folder is
      # missing. If you set this to 'true', it will error instead.
      forceLoadFromOutput = false;

      # --- IMPLEMENTATION ---

      inherit (nixpkgs) lib;
      sourceFileRegex =
        builtins.concatStringsSep "|" (map lib.escapeRegex sourceFiles);
      src = lib.sourceByRegex ./. [ "(${{sourceFileRegex}})(/.*)?" ];

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
      # Specify 'forceLoadFromOutput = true;' above to opt into erroring
      # if the 'output/dev/nix' folder isn't found instead of invoking
      # the Glistix compiler.
      #
      # Pass 'system' to use the derivation for the given system.
      # Other arguments are passed through to Glistix's 'loadGlistixPackage'.
      # For example, 'lib.loadGlistixPackage {{ module = "ops/create"; }}'
      # will load what's exported by your package's 'ops/create' module
      # as an attribute set.
      loadGlistixPackage =
        args@{{ system ? builtins.currentSystem or null, ... }}:
        let
          derivation = if forceLoadFromOutput || system == null then
            null
          else
            self.packages.${{system}}.default or null;

          overrides = builtins.removeAttrs args [ "system" ];
          loaderArgs = {{ inherit src derivation; }} // overrides;
        in glistix.lib.loadGlistixPackage loaderArgs;

    in flake-parts.lib.mkFlake {{ inherit inputs; }} {{
      systems = import systems;

      flake = {{ lib = {{ inherit loadGlistixPackage; }}; }};

      perSystem = {{ self', pkgs, lib, system, ... }}:
        let
          inherit (glistix.builders.${{system}}) buildGlistixPackage;

          # This derivation will build Glistix itself if needed
          # (using Rust), and then use Glistix to build this particular
          # package into Nix files.
          # The derivation's "out" output will contain the resulting
          # 'build' directory. You can use
          #   "${{derivation}}/${{derivation.glistixMain}}"
          # for a path to this package's main '.nix' file, which can
          # be imported through Nix's `import`.
          derivation = buildGlistixPackage {{ inherit src submodules; }};
        in {{
          packages.default = derivation;

          # Run 'nix develop' to create a shell where 'glistix' is available.
          devShells.default = pkgs.mkShell {{
            nativeBuildInputs = [ glistix.packages.${{system}}.default ];
          }};
        }};
    }};
}}
"#,
            )),
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
        // External folder: we will clone stdlib and gleeunit there if possible.
        let external = root.join("external");
        let external_stdlib = external.join("stdlib");
        let external_gleeunit = external.join("gleeunit");
        let me = Self {
            root: root.clone(),
            src,
            test,
            github,
            workflows,
            external,
            external_stdlib,
            external_gleeunit,
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
Glistix patches for 'stdlib' and 'gleeunit'. You can do so with the command \
'git clone -- https://github.com/glistix/NAME external/NAME', where NAME is one of \
stdlib or gleeunit. You can also use 'git submodule add --name NAME' instead of 'git clone' \
if you plan on creating a git repository at your new project's directory."
            )
        } else {
            crate::fs::git_init(&self.root)?;
            crate::fs::git_submodule_add(
                "stdlib",
                GLISTIX_STDLIB_URL,
                &self.root,
                &self.external_stdlib,
            )?;
            crate::fs::git_submodule_add(
                "gleeunit",
                GLISTIX_GLEEUNIT_URL,
                &self.root,
                &self.external_gleeunit,
            )?;
        }

        match self.options.template {
            Template::Lib => {
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
        if full_path.exists() {
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
