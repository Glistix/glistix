use std::collections::{HashMap, HashSet};

use ecow::EcoString;
use itertools::Itertools;

use crate::nix::{maybe_escape_identifier_string, syntax};
use crate::pretty::{break_, join, nil};
use crate::{
    docvec,
    pretty::{Document, Documentable},
};

/// A collection of Nix import statements from Gleam imports and from
/// external functions, to be rendered into a Nix module.
/// Analogous to [`crate::javascript::Imports`].
#[derive(Debug, Default)]
pub(crate) struct Imports<'a> {
    imports: HashMap<EcoString, Import<'a>>,
    exports: HashSet<EcoString>,
}

impl<'a> Imports<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_export(&mut self, export: EcoString) {
        let _ = self.exports.insert(export);
    }

    pub fn register_module(
        &mut self,
        path: EcoString,
        aliases: impl IntoIterator<Item = EcoString>,
        unqualified_imports: impl IntoIterator<Item = Member<'a>>,
    ) {
        // Sanitize path
        let path = EcoString::from(syntax::path(&path));
        let import = self
            .imports
            .entry(path.clone())
            .or_insert_with(|| Import::new(path));
        import.aliases.extend(aliases);
        import.unqualified.extend(unqualified_imports)
    }

    /// Finishes import declarations.
    /// Returns assignments to perform and names to export.
    pub fn finish(self) -> (Document<'a>, impl IntoIterator<Item = EcoString>) {
        let imports = join(
            self.imports
                .into_values()
                .filter(|import| !import.is_empty())
                .sorted_by(|a, b| a.path.cmp(&b.path))
                .map(Import::into_doc),
            break_("", " "),
        );

        (imports, self.exports.into_iter().sorted())
    }

    pub fn is_empty(&self) -> bool {
        self.imports.is_empty() && self.exports.is_empty()
    }
}

#[derive(Debug)]
struct Import<'a> {
    path: EcoString,
    aliases: HashSet<EcoString>,
    unqualified: Vec<Member<'a>>,
}

impl<'a> Import<'a> {
    /// Assumes the path is already sanitized.
    fn new(path: EcoString) -> Self {
        Self {
            path,
            aliases: Default::default(),
            unqualified: Default::default(),
        }
    }

    pub fn into_doc(self) -> Document<'a> {
        let path = self.path.to_doc();
        let no_aliases = self.aliases.is_empty();
        let alias_imports = join(
            self.aliases.into_iter().sorted().map(|alias| {
                // Alias is equivalent to just importing again:
                // alias = import path;
                syntax::assignment_line(
                    maybe_escape_identifier_string(&alias).to_doc(),
                    docvec!["builtins.import ", path.clone()],
                )
            }),
            break_("", " "),
        );

        if self.unqualified.is_empty() {
            alias_imports
        } else {
            // 'inherit (#import_source) #members' will import the members from the source in Nix.
            let import_source = docvec!["(builtins.import ", path, ")"];
            let (aliased_members, unaliased_members) = self
                .unqualified
                .into_iter()
                .partition::<Vec<Member<'_>>, _>(|member| member.alias.is_some());

            let (inherit_unaliased, no_unaliased) = if unaliased_members.is_empty() {
                (nil(), true)
            } else {
                let alias_import_break = if no_aliases { nil() } else { break_("", " ") };

                let unaliased_names = unaliased_members
                    .into_iter()
                    .map(|member| member.name.to_doc());

                (
                    docvec![
                        alias_import_break,
                        syntax::inherit(
                            std::iter::once(import_source.clone()).chain(unaliased_names)
                        )
                    ],
                    false,
                )
            };

            let aliased_assignments = if aliased_members.is_empty() {
                nil()
            } else {
                let unaliased_break = if no_unaliased && no_aliases {
                    nil()
                } else {
                    break_("", " ")
                };
                unaliased_break.append(join(
                    aliased_members.into_iter().map(|member| {
                        // Generate:
                        // `alias = (import ...).name;\n`
                        syntax::assignment_line(
                            member.alias.to_doc(),
                            docvec![import_source.clone(), ".", member.name],
                        )
                    }),
                    break_("", ""),
                ))
            };

            docvec![alias_imports, inherit_unaliased, aliased_assignments]
        }
    }

    pub fn is_empty(&self) -> bool {
        self.aliases.is_empty() && self.unqualified.is_empty()
    }
}

#[derive(Debug)]
pub struct Member<'a> {
    pub name: Document<'a>,
    pub alias: Option<Document<'a>>,
}

#[test]
fn finish() {
    let mut imports = Imports::new();
    imports.register_module("./gleam/empty".into(), [], []);
    imports.register_module(
        "./multiple/times".into(),
        ["wibble".into(), "wobble".into()],
        [],
    );
    imports.register_module("./multiple/times".into(), ["wubble".into()], []);
    imports.register_module(
        "./multiple/times".into(),
        [],
        [Member {
            name: "one".to_doc(),
            alias: None,
        }],
    );

    imports.register_module(
        "./other".into(),
        [],
        [
            Member {
                name: "one".to_doc(),
                alias: None,
            },
            Member {
                name: "one".to_doc(),
                alias: Some("onee".to_doc()),
            },
            Member {
                name: "two".to_doc(),
                alias: Some("twoo".to_doc()),
            },
        ],
    );

    imports.register_module(
        "./other".into(),
        [],
        [
            Member {
                name: "three".to_doc(),
                alias: None,
            },
            Member {
                name: "four".to_doc(),
                alias: None,
            },
        ],
    );

    imports.register_module(
        "./zzz".into(),
        [],
        [
            Member {
                name: "one".to_doc(),
                alias: None,
            },
            Member {
                name: "two".to_doc(),
                alias: None,
            },
        ],
    );

    assert_eq!(
        crate::pretty::line()
            .append(imports.finish().0.group())
            .to_pretty_string(40),
        r#"
wibble =
  builtins.import ./multiple/times;
wobble =
  builtins.import ./multiple/times;
wubble =
  builtins.import ./multiple/times;
inherit
  (builtins.import ./multiple/times)
  one
  ;
inherit
  (builtins.import ./other)
  one
  three
  four
  ;
onee = (builtins.import ./other).one;
twoo = (builtins.import ./other).two;
inherit (builtins.import ./zzz) one two;"#
            .to_string()
    );
}
