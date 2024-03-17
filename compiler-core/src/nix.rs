mod expression;

use crate::analyse::TargetSupport;
use crate::ast::TypedModule;
use crate::docvec;
use crate::javascript::Error;
use crate::line_numbers::LineNumbers;
use crate::pretty::{break_, concat, line, Document, Documentable};
use camino::Utf8Path;
use ecow::EcoString;

pub const INDENT: isize = 2;

struct Generator<'module> {
    module: &'module TypedModule,
    line_numbers: &'module LineNumbers,
    target_support: TargetSupport,
}

pub type Output<'a> = Result<Document<'a>, Error>;

impl<'module> Generator<'module> {
    pub fn new(
        line_numbers: &'module LineNumbers,
        module: &'module TypedModule,
        target_support: TargetSupport,
    ) -> Self {
        Self {
            module,
            line_numbers,
            target_support,
        }
    }

    pub fn compile(&mut self) -> Output<'module> {
        todo!()
    }
}

pub fn module(
    module: &TypedModule,
    line_numbers: &LineNumbers,
    path: &Utf8Path,
    src: &EcoString,
    target_support: TargetSupport,
) -> Result<String, crate::Error> {
    let document = Generator::new(line_numbers, module, target_support)
        .compile()
        .map_err(|error| crate::Error::JavaScript {
            path: path.to_path_buf(),
            src: src.clone(),
            error,
        })?;
    Ok(document.to_pretty_string(80))
}

fn is_usable_nix_identifier(word: &str) -> bool {
    !matches!(
        word,
        // Keywords and reserved words
        "if"
            | "then"
            | "else"
            | "assert"
            | "with"
            | "let"
            | "in"
            | "rec"
            | "inherit"
            | "or"
            // Some non-keywords for fundamental types
            | "true"
            | "false"
            | "null"
            // Built-ins let us access fundamental functions anywhere
            | "builtins"
    )
}

fn maybe_escape_identifier_string(word: &str) -> String {
    if is_usable_nix_identifier(word) {
        word.to_string()
    } else {
        escape_identifier(word)
    }
}

fn escape_identifier(word: &str) -> String {
    format!("{word}'")
}

fn maybe_escape_identifier_doc(word: &str) -> Document<'_> {
    if is_usable_nix_identifier(word) {
        word.to_doc()
    } else {
        Document::String(escape_identifier(word))
    }
}
