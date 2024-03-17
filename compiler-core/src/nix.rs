mod expression;
#[cfg(test)]
mod tests;

use crate::analyse::TargetSupport;
use crate::ast::{
    CustomType, Definition, Function, Import, ModuleConstant, TypeAlias, TypedArg, TypedDefinition,
    TypedFunction, TypedModule,
};
use crate::build::Target;
use crate::docvec;
use crate::javascript::Error;
use crate::line_numbers::LineNumbers;
use crate::pretty::{break_, concat, join, line, lines, Document, Documentable};
use camino::Utf8Path;
use ecow::EcoString;
use itertools::Itertools;

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
        let mut generator = expression::Generator::new(
            self.module,
            self.line_numbers,
            self.target_support,
            im::HashMap::default(),
        );

        // Generate Nix code for each statement
        let statements = self.collect_definitions().into_iter().chain(
            self.module
                .definitions
                .iter()
                .flat_map(|s| self.statement(&mut generator, s)),
        );

        let attr_set = try_wrap_attr_set(statements)?;
        Ok(docvec![attr_set, line()])
    }

    /// Outputs the name and the value of the module item.
    pub fn statement<'a>(
        &mut self,
        generator: &mut expression::Generator<'module>,
        statement: &'a TypedDefinition,
    ) -> Option<(Document<'a>, Output<'a>)> {
        match statement {
            Definition::TypeAlias(TypeAlias { .. }) => None,

            // Handled in collect_imports
            Definition::Import(Import { .. }) => None,

            // Handled in collect_definitions
            Definition::CustomType(CustomType { .. }) => None,

            Definition::ModuleConstant(ModuleConstant {
                public,
                name,
                value,
                ..
            }) => todo!(),

            Definition::Function(function) => {
                // If there's an external JavaScript implementation then it will be imported,
                // so we don't need to generate a function definition.
                if function.external_javascript.is_some() {
                    // TODO: Specialize this to Nix
                    return None;
                }

                // If the function does not support JavaScript then we don't need to generate
                // a function definition.
                if !function.implementations.supports(Target::JavaScript) {
                    return None;
                }

                self.module_function(generator, function)
            }
        }
    }

    fn collect_definitions<'a>(&mut self) -> Vec<(Document<'a>, Output<'a>)> {
        self.module
            .definitions
            .iter()
            .flat_map(|statement| match statement {
                Definition::CustomType(CustomType {
                    public,
                    constructors,
                    opaque,
                    ..
                }) => todo!(), // self.custom_type_definition(constructors, *public, *opaque),

                Definition::Function(Function { .. })
                | Definition::TypeAlias(TypeAlias { .. })
                | Definition::Import(Import { .. })
                | Definition::ModuleConstant(ModuleConstant { .. }) => vec![],
            })
            .collect()
    }

    fn module_function<'a>(
        &mut self,
        generator: &mut expression::Generator<'module>,
        function: &'a TypedFunction,
    ) -> Option<(Document<'a>, Output<'a>)> {
        let name = function.name.as_ref().to_doc();
        let result = match generator.fn_(function.arguments.as_slice(), &function.body) {
            // No error, let's continue!
            Ok(body) => body,

            // There is an error coming from some expression that is not supported on JavaScript
            // and the target support is not enforced. In this case we do not error, instead
            // returning nothing which will cause no function to be generated.
            Err(error) if error.is_unsupported() && !self.target_support.is_enforced() => {
                return None
            }

            // Some other error case which will be returned to the user.
            Err(error) => return Some(("".to_doc(), Err(error))),
        };

        Some((name, Ok(result)))
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

fn fun_args(args: &'_ [TypedArg]) -> Document<'_> {
    let mut discards = 0;
    wrap_args(args.iter().map(|a| match a.get_variable_name() {
        None => {
            let doc = if discards == 0 {
                "_".to_doc()
            } else {
                Document::String(format!("_{discards}"))
            };
            discards += 1;
            doc
        }
        Some(name) => maybe_escape_identifier_doc(name),
    }))
}

fn wrap_args<'a, I>(args: I) -> Document<'a>
where
    I: IntoIterator<Item = Document<'a>>,
{
    break_("", "")
        .append(concat(args.into_iter().map(|arg| arg.append(": "))))
        .append(break_("", ""))
        .group()
}

fn wrap_attr_set<'a>(
    items: impl IntoIterator<Item = (Document<'a>, Option<Document<'a>>)>,
) -> Document<'a> {
    let mut empty = true;
    let fields = items.into_iter().map(|(key, value)| {
        empty = false;
        match value {
            Some(value) => docvec![key, " =", break_("", " "), value, ";"],
            None => docvec!["inherit ", key.to_doc(), ";"],
        }
    });
    let fields = join(fields, break_("", " "));

    if empty {
        "{}".to_doc()
    } else {
        docvec![
            docvec!["{", break_("", " "), fields]
                .nest(INDENT)
                .append(break_("", " "))
                .group(),
            "}"
        ]
    }
}

fn try_wrap_attr_set<'a>(
    items: impl IntoIterator<Item = (Document<'a>, Output<'a>)>,
) -> Output<'a> {
    let fields = items
        .into_iter()
        .map(|(key, value)| Ok(docvec![key, " =", break_("", " "), value?, ";"]));
    let fields: Vec<_> = Itertools::intersperse(fields, Ok(break_("", " "))).try_collect()?;

    Ok(docvec![
        docvec!["{", break_("", " "), fields]
            .nest(INDENT)
            .append(break_("", " "))
            .group(),
        "}"
    ])
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
            // This variable lets us access fundamental functions anywhere
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
