mod expression;
mod import;
#[cfg(test)]
mod tests;

use crate::analyse::TargetSupport;
use crate::ast::{
    AssignName, CustomType, Definition, Function, Import, ModuleConstant, Publicity, SrcSpan,
    TypeAlias, TypedArg, TypedConstant, TypedDefinition, TypedFunction, TypedModule,
    TypedRecordConstructor, TypedRecordConstructorArg, UnqualifiedImport,
};
use crate::build::Target;
use crate::docvec;
use crate::javascript::Error;
use crate::line_numbers::LineNumbers;
use crate::nix::expression::string;
use crate::nix::import::{Imports, Member};
use crate::pretty::{break_, concat, join, line, Document, Documentable};
use camino::Utf8Path;
use ecow::EcoString;
use itertools::Itertools;

pub const INDENT: isize = 2;

struct Generator<'module> {
    module: &'module TypedModule,
    line_numbers: &'module LineNumbers,
    target_support: TargetSupport,
    module_scope: im::HashMap<EcoString, usize>,
    tracker: UsageTracker,
    /// Used when determining relative import paths.
    current_module_name_segments_count: usize,
}

pub type Output<'a> = Result<Document<'a>, Error>;

/// Some declaration at the top-level of a module.
struct ModuleDeclaration<'a> {
    /// If the variable being declared must be exported by this module.
    exported: bool,
    /// The name of the variable being declared.
    name: Document<'a>,
    /// The value of the variable being declared.
    value: Document<'a>,
}

impl<'module> Generator<'module> {
    pub fn new(
        line_numbers: &'module LineNumbers,
        module: &'module TypedModule,
        target_support: TargetSupport,
    ) -> Self {
        let current_module_name_segments_count = module.name.split('/').count();

        Self {
            module,
            line_numbers,
            target_support,
            module_scope: im::HashMap::new(),
            tracker: UsageTracker::default(),
            current_module_name_segments_count,
        }
    }

    pub fn compile(&mut self) -> Output<'module> {
        // Determine Nix import code to generate.
        let imports = self.collect_imports();

        // Determine what names are defined in the module scope so we know to
        // rename any variables that are defined within functions using the same
        // names.
        self.register_module_definitions_in_scope();

        // Generate Nix code for each statement.
        let statements: Vec<_> = self
            .collect_definitions()
            .into_iter()
            .chain(
                self.module
                    .definitions
                    .iter()
                    .flat_map(|s| self.statement(s)),
            )
            .try_collect()?;

        let no_imports = imports.is_empty();
        let (import_lines, exported_names) = imports.finish();

        // Exported names. Those will be `inherit`ed in the final exported dictionary.
        let mut exported_names = exported_names
            .into_iter()
            .map(Document::String)
            .chain(
                statements
                    .iter()
                    .filter(|declaration| declaration.exported)
                    .map(|declaration| &declaration.name)
                    .cloned(),
            )
            .peekable();

        let exports = if exported_names.peek().is_some() {
            docvec![
                "{",
                break_("", " "),
                inherit(exported_names),
                break_("", " "),
                "}"
            ]
        } else {
            "{}".to_doc()
        };

        // Assignment of top-level module names, exported or not.
        let assignments: Vec<_> = statements
            .into_iter()
            .map(|declaration| expression::assignment_line(declaration.name, declaration.value))
            .collect();

        // Finish up the module.
        if no_imports && assignments.is_empty() {
            Ok(docvec![exports, line()])
        } else if no_imports {
            Ok(docvec![
                expression::let_in(assignments, exports, true).group(),
                line()
            ])
        } else {
            Ok(docvec![
                expression::let_in(
                    std::iter::once(import_lines).chain(assignments),
                    exports,
                    true
                )
                .group(),
                line()
            ])
        }
    }

    /// Outputs the name and the value of the module item.
    /// The boolean is true if the item is public (exported).
    pub fn statement<'a>(
        &mut self,
        statement: &'a TypedDefinition,
    ) -> Option<Result<ModuleDeclaration<'a>, Error>> {
        match statement {
            Definition::TypeAlias(TypeAlias { .. }) => None,

            // Handled in collect_imports
            Definition::Import(Import { .. }) => None,

            // Handled in collect_definitions
            Definition::CustomType(CustomType { .. }) => None,

            Definition::ModuleConstant(ModuleConstant {
                publicity,
                name,
                value,
                ..
            }) => Some(self.module_constant(*publicity, name.as_ref(), value)),

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

                self.module_function(function)
            }
        }
    }

    fn collect_definitions(&mut self) -> Vec<Result<ModuleDeclaration<'module>, Error>> {
        self.module
            .definitions
            .iter()
            .flat_map(|statement| match statement {
                Definition::CustomType(CustomType {
                    publicity,
                    constructors,
                    opaque,
                    ..
                }) => self.custom_type_definition(constructors, *publicity, *opaque),

                Definition::Function(Function { .. })
                | Definition::TypeAlias(TypeAlias { .. })
                | Definition::Import(Import { .. })
                | Definition::ModuleConstant(ModuleConstant { .. }) => vec![],
            })
            .collect()
    }

    fn module_constant<'a>(
        &mut self,
        publicity: Publicity,
        name: &'a str,
        value: &'a TypedConstant,
    ) -> Result<ModuleDeclaration<'a>, Error> {
        Ok(ModuleDeclaration {
            exported: !publicity.is_private(),
            name: maybe_escape_identifier_doc(name),
            value: expression::constant_expression(&mut self.tracker, value)?,
        })
    }

    fn module_function<'a>(
        &mut self,
        function: &'a TypedFunction,
    ) -> Option<Result<ModuleDeclaration<'a>, Error>> {
        let mut generator = expression::Generator::new(
            self.module,
            self.line_numbers,
            self.target_support,
            self.module_scope.clone(),
            &mut self.tracker,
        );

        let name = maybe_escape_identifier_doc(function.name.as_ref());

        // A module-level function, in Nix, will have the exact same syntax as a lambda function.
        let def_body = match generator.fn_(function.arguments.as_slice(), &function.body) {
            // No error, let's continue!
            Ok(body) => body,

            // There is an error coming from some expression that is not supported on JavaScript
            // and the target support is not enforced. In this case we do not error, instead
            // returning nothing which will cause no function to be generated.
            Err(error) if error.is_unsupported() && !self.target_support.is_enforced() => {
                return None;
            }

            // Some other error case which will be returned to the user.
            Err(error) => return Some(Err(error)),
        };

        Some(Ok(ModuleDeclaration {
            exported: !function.publicity.is_private(),
            name,
            value: def_body,
        }))
    }

    fn custom_type_definition<'a>(
        &mut self,
        constructors: &'a [TypedRecordConstructor],
        publicity: Publicity,
        opaque: bool,
    ) -> Vec<Result<ModuleDeclaration<'a>, Error>> {
        // If there's no constructors then there's nothing to do here.
        if constructors.is_empty() {
            return vec![];
        }

        constructors
            .iter()
            .map(|constructor| Ok(self.record_definition(constructor, publicity, opaque)))
            .collect()
    }

    fn record_definition<'a>(
        &self,
        constructor: &'a TypedRecordConstructor,
        publicity: Publicity,
        opaque: bool,
    ) -> ModuleDeclaration<'a> {
        fn parameter((i, arg): (usize, &TypedRecordConstructorArg)) -> Document<'_> {
            arg.label
                .as_ref()
                .map(|s| maybe_escape_identifier_doc(s))
                .unwrap_or_else(|| Document::String(format!("x{i}")))
        }

        let should_export = !(publicity.is_private() || opaque);
        let name = maybe_escape_identifier_doc(&constructor.name);
        let tag_field = ("__gleam_tag'".to_doc(), Some(string(&constructor.name)));

        if constructor.arguments.is_empty() {
            let result = wrap_attr_set([tag_field]);
            return ModuleDeclaration {
                exported: should_export,
                name,
                value: result,
            };
        }

        let args = wrap_args(constructor.arguments.iter().enumerate().map(parameter));
        let returned_fields = constructor.arguments.iter().enumerate().map(|(i, arg)| {
            let parameter = parameter((i, arg));
            if let Some(label) = &arg.label {
                (label.to_doc(), Some(parameter))
            } else {
                (Document::String(format!("_{i}")), Some(parameter))
            }
        });

        let returned_set = wrap_attr_set(std::iter::once(tag_field).chain(returned_fields));
        let constructor_fun = docvec!(args, break_("", " "), returned_set)
            .nest(INDENT)
            .group();

        ModuleDeclaration {
            exported: should_export,
            name,
            value: constructor_fun,
        }
    }

    fn register_in_scope(&mut self, name: &str) {
        let _ = self.module_scope.insert(name.into(), 0);
    }

    fn register_module_definitions_in_scope(&mut self) {
        for statement in self.module.definitions.iter() {
            match statement {
                Definition::ModuleConstant(ModuleConstant { name, .. })
                | Definition::Function(Function { name, .. }) => self.register_in_scope(name),

                Definition::Import(Import {
                    unqualified_values: unqualified,
                    ..
                }) => unqualified
                    .iter()
                    .for_each(|unq_import| self.register_in_scope(unq_import.used_name())),

                Definition::TypeAlias(TypeAlias { .. })
                | Definition::CustomType(CustomType { .. }) => (),
            }
        }
    }
}

/// Import-related methods.
impl<'module> Generator<'module> {
    fn collect_imports(&mut self) -> Imports<'module> {
        let mut imports = Imports::new();

        for statement in &self.module.definitions {
            match statement {
                Definition::Import(Import {
                    module,
                    as_name,
                    unqualified_values: unqualified,
                    package,
                    ..
                }) => {
                    self.register_import(&mut imports, package, module, as_name, unqualified);
                }

                Definition::Function(Function {
                    name,
                    publicity,
                    external_javascript: Some((module, function)),
                    ..
                }) => {
                    self.register_external_function(
                        &mut imports,
                        *publicity,
                        name,
                        module,
                        function,
                    );
                }

                Definition::Function(Function { .. })
                | Definition::TypeAlias(TypeAlias { .. })
                | Definition::CustomType(CustomType { .. })
                | Definition::ModuleConstant(ModuleConstant { .. }) => (),
            }
        }

        imports
    }

    fn import_path<'a>(&self, package: &'a str, module: &'a str) -> String {
        // TODO: strip shared prefixed between current module and imported
        // module to avoid descending and climbing back out again
        if package == self.module.type_info.package || package.is_empty() {
            // Same package
            match self.current_module_name_segments_count {
                1 => format!("./{module}.nix"),
                _ => {
                    let prefix = "../".repeat(self.current_module_name_segments_count - 1);
                    format!("{prefix}{module}.nix")
                }
            }
        } else {
            // Different package
            let prefix = "../".repeat(self.current_module_name_segments_count);
            format!("{prefix}{package}/{module}.nix")
        }
    }

    fn register_import<'a>(
        &mut self,
        imports: &mut Imports<'a>,
        package: &'a str,
        module: &'a str,
        as_name: &'a Option<(AssignName, SrcSpan)>,
        unqualified: &'a [UnqualifiedImport],
    ) {
        let get_name = |module: &'a str| {
            module
                .split('/')
                .last()
                .expect("Nix generator could not identify imported module name.")
        };

        let (discarded, module_name) = match as_name {
            None => (false, get_name(module)),
            Some((AssignName::Discard(_), _)) => (true, get_name(module)),
            Some((AssignName::Variable(name), _)) => (false, name.as_str()),
        };

        let module_name = module_var_name(module_name);
        let path = self.import_path(package, module);
        let unqualified_imports = unqualified.iter().map(|i| {
            let alias = i.as_name.as_ref().map(|n| {
                self.register_in_scope(n);
                maybe_escape_identifier_doc(n)
            });
            let name = maybe_escape_identifier_doc(&i.name);
            Member { name, alias }
        });

        let aliases = if discarded { vec![] } else { vec![module_name] };
        imports.register_module(path, aliases, unqualified_imports);
    }

    fn register_external_function<'a>(
        &mut self,
        imports: &mut Imports<'a>,
        publicity: Publicity,
        name: &'a str,
        module: &'a str,
        fun: &'a str,
    ) {
        let needs_escaping = !is_usable_nix_identifier(name);
        let member = Member {
            name: fun.to_doc(),
            alias: if name == fun && !needs_escaping {
                None
            } else if needs_escaping {
                Some(Document::String(escape_identifier(name)))
            } else {
                Some(name.to_doc())
            },
        };
        if publicity.is_importable() {
            imports.register_export(maybe_escape_identifier_string(name))
        }
        imports.register_module(module.to_string(), [], [member]);
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
    // Add spaces after all but the last argument.
    break_("", "")
        .append(join(
            args.into_iter().map(|arg| arg.append(":")),
            " ".to_doc(),
        ))
        .append(break_("", ""))
        .group()
}

fn inherit<'a>(items: impl IntoIterator<Item = Document<'a>>) -> Document<'a> {
    let spaced_items = items.into_iter().map(|name| docvec!(break_("", " "), name));

    // Note: an 'inherit' without items is valid Nix syntax.
    docvec!["inherit", concat(spaced_items).nest(INDENT).group(), ";"]
}

/// Generates the variable name in Nix for the given module.
fn module_var_name(name: &str) -> String {
    format!("mod''{}", maybe_escape_identifier_string(name))
}

fn wrap_attr_set<'a>(
    items: impl IntoIterator<Item = (Document<'a>, Option<Document<'a>>)>,
) -> Document<'a> {
    let mut empty = true;
    let fields = items.into_iter().map(|(key, value)| {
        empty = false;
        match value {
            Some(value) => docvec![
                key,
                " =",
                docvec![break_("", " "), value, ";"].nest(INDENT).group()
            ],
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
    let fields = items.into_iter().map(|(key, value)| {
        Ok(docvec![
            key,
            " =",
            docvec![break_("", " "), value?, ";"].nest(INDENT).group()
        ])
    });
    let fields: Vec<_> = Itertools::intersperse(fields, Ok(break_("", " "))).try_collect()?;

    Ok(docvec![
        docvec!["{", break_("", " "), fields]
            .nest(INDENT)
            .append(break_("", " "))
            .group(),
        "}"
    ])
}

fn is_nix_keyword(word: &str) -> bool {
    matches!(
        word,
        // Keywords and reserved words
        "if" | "then" | "else" | "assert" | "with" | "let" | "in" | "rec" | "inherit" | "or"
    )
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

#[derive(Debug, Default)]
pub(crate) struct UsageTracker {
    pub ok_used: bool,
    // pub list_used: bool,
    // pub prepend_used: bool,
    pub error_used: bool,
    pub int_remainder_used: bool,
    // pub make_error_used: bool,
    // pub custom_type_used: bool,
    // pub int_division_used: bool,
    // pub float_division_used: bool,
    // pub object_equality_used: bool,
    // pub bit_array_literal_used: bool,
    // pub sized_integer_segment_used: bool,
    // pub string_bit_array_segment_used: bool,
    // pub codepoint_bit_array_segment_used: bool,
    // pub float_bit_array_segment_used: bool,
}
