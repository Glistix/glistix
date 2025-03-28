mod expression;
mod import;
mod pattern;
pub mod syntax;
#[cfg(test)]
mod tests;

use crate::analyse::TargetSupport;
use crate::ast::{
    AssignName, CustomType, Definition, Function, Import, ModuleConstant, Publicity, SrcSpan,
    TypeAlias, TypedConstant, TypedDefinition, TypedFunction, TypedModule, TypedRecordConstructor,
    TypedRecordConstructorArg, UnqualifiedImport,
};
use crate::build::Target;
use crate::docvec;
use crate::line_numbers::LineNumbers;
use crate::nix::import::{Imports, Member};
use crate::pretty::{break_, concat, line, nil, Document, Documentable};
use crate::type_::PRELUDE_MODULE_NAME;
use camino::Utf8Path;
use ecow::{eco_format, EcoString};
use itertools::Itertools;

pub const INDENT: isize = 2;

pub const PRELUDE: &str = include_str!("../templates/prelude.nix");

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
        let mut imports = self.collect_imports();

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

        self.register_used_prelude_functions(&mut imports);

        let no_imports = imports.is_empty();
        let (import_lines, exported_names) = imports.finish();

        // Exported names. Those will be `inherit`ed in the final exported dictionary.
        let mut exported_names = exported_names
            .into_iter()
            .map(EcoString::to_doc)
            .chain(
                statements
                    .iter()
                    .filter(|declaration| declaration.exported)
                    .map(|declaration| &declaration.name)
                    .cloned(),
            )
            .peekable();

        let exports = if exported_names.peek().is_some() {
            syntax::attr_set(syntax::inherit(exported_names))
        } else {
            "{ }".to_doc()
        };

        // Assignment of top-level module names, exported or not.
        let assignments: Vec<_> = statements
            .into_iter()
            .map(|declaration| syntax::assignment_line(declaration.name, declaration.value))
            .collect();

        // Finish up the module.
        if no_imports && assignments.is_empty() {
            Ok(docvec![exports, line()])
        } else if no_imports {
            Ok(docvec![
                syntax::let_in(assignments, exports, true).group(),
                line()
            ])
        } else {
            Ok(docvec![
                syntax::let_in(
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
                // If there's an external Nix implementation then it will be imported,
                // so we don't need to generate a function definition.
                if function.external_nix.is_some() {
                    return None;
                }

                // If the function does not support Nix then we don't need to generate
                // a function definition.
                if !function.implementations.supports(Target::Nix) {
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
        let (_, name) = function
            .name
            .as_ref()
            .expect("A module's function must be named");
        let mut generator = expression::Generator::new(
            self.module,
            self.line_numbers,
            Some(name.clone()),
            self.module_scope.clone(),
            &mut self.tracker,
        );

        let name = maybe_escape_identifier_doc(name.as_ref());

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
            .map(|constructor| Ok(Self::record_definition(constructor, publicity, opaque)))
            .collect()
    }

    /// Returns a record definition, of the form:
    ///
    /// ```nix
    /// Ctor = named1: named2: x0: x1: { __gleamTag = "Ctor"; inherit named1 named2; _0 = x0; _1 = x1; }
    /// ```
    fn record_definition(
        constructor: &TypedRecordConstructor,
        publicity: Publicity,
        opaque: bool,
    ) -> ModuleDeclaration<'_> {
        const GLEAM_TAG_FIELD_NAME: &str = "__gleamTag";

        fn parameter((i, arg): (usize, &TypedRecordConstructorArg)) -> Document<'_> {
            arg.label
                .as_ref()
                .map(|(_, s)| maybe_escape_identifier_doc(s))
                .unwrap_or_else(|| eco_format!("x{i}").to_doc())
        }

        let should_export = !(publicity.is_private() || opaque);
        let name = maybe_escape_identifier_doc(&constructor.name);
        let tag_field = syntax::assignment_line(
            GLEAM_TAG_FIELD_NAME.to_doc(),
            syntax::string_without_escapes_or_backslashes(&constructor.name),
        );

        if constructor.arguments.is_empty() {
            let result = syntax::attr_set(tag_field);
            return ModuleDeclaration {
                exported: should_export,
                name,
                value: result,
            };
        }

        let args = syntax::wrap_args(constructor.arguments.iter().enumerate().map(parameter));

        // Named fields will always correspond to their parameters, unless
        // they are keywords (in which case the parameter will be escaped
        // and the associated attribute will be quoted) or important names (in
        // which case the parameter will be escaped but the attribute won't).
        // Thus, named fields with ordinary names are added through 'inherit'
        // instead of 'field = field' to make the declaration shorter and more
        // concise.
        let (inherited_fields, other_fields) = constructor
            .arguments
            .iter()
            .enumerate()
            .partition::<Vec<_>, _>(|(_, arg)| {
                arg.label
                    .as_ref()
                    .is_some_and(|(_, l)| is_usable_nix_identifier(l))
            });

        let inherited_fields = if inherited_fields.is_empty() {
            nil()
        } else {
            docvec![
                break_("", " "),
                syntax::inherit(inherited_fields.iter().map(|(_, arg)| {
                    (&arg
                        .label
                        .as_ref()
                        .expect("presence of label should already have been checked")
                        .1)
                        .to_doc()
                }))
            ]
        };

        let other_fields = concat(other_fields.iter().map(|(i, arg)| {
            let parameter = parameter((*i, arg));
            let assignment = if let Some((_, label)) = &arg.label {
                // Add "quotes" around attribute name if it would be a keyword.
                // The quotes are needed on declaration site and also on field
                // access, but the underlying attribute name isn't changed.
                syntax::assignment_line(
                    syntax::maybe_quoted_attr_set_label_from_identifier(label),
                    parameter,
                )
            } else {
                // Positional arguments are represented as '_NUMBER'.
                // They might not start at zero.
                syntax::assignment_line(eco_format!("_{i}").to_doc(), parameter)
            };

            docvec![break_("", " "), assignment]
        }));

        let returned_set = syntax::attr_set(docvec![tag_field, inherited_fields, other_fields]);

        let constructor_fun = docvec![args, break_("", " "), returned_set]
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
                Definition::ModuleConstant(ModuleConstant { name, .. }) => {
                    self.register_in_scope(name)
                }

                Definition::Function(Function { name, .. }) => self.register_in_scope(
                    name.as_ref()
                        .map(|(_, name)| name)
                        .expect("Function in a definition must be named"),
                ),

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
                    name: Some((_, name)),
                    publicity,
                    external_nix: Some((module, function, _location)),
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

    fn import_path<'a>(&self, package: &'a str, module: &'a str) -> EcoString {
        // TODO: strip shared prefixed between current module and imported
        // module to avoid descending and climbing back out again
        if package == self.module.type_info.package || package.is_empty() {
            // Same package
            match self.current_module_name_segments_count {
                1 => eco_format!("./{module}.nix"),
                _ => {
                    let prefix = "../".repeat(self.current_module_name_segments_count - 1);
                    eco_format!("{prefix}{module}.nix")
                }
            }
        } else {
            // Different package
            let prefix = "../".repeat(self.current_module_name_segments_count);
            eco_format!("{prefix}{package}/{module}.nix")
        }
    }

    /// Register an import from a Gleam module.
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

            // Here we escape as an identifier, not as an arbitrary string,
            // since we're using Gleam's built-in import mechanism, which can
            // only import identifiers (and variables can be bound to the
            // imported names). This is different from functions imported through
            // @external, which can have arbitrary names, but that's OK since
            // we always rename the imported function when it wouldn't match
            // the chosen identifier for the function receiving @external.
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
            // External functions can have arbitrary names in Nix,
            // including keywords and whatnot, so we escape them
            // using quotes.
            name: syntax::maybe_quoted_attr_set_label(fun),
            alias: if name == fun && !needs_escaping {
                None
            } else if needs_escaping {
                Some(escape_identifier(name).to_doc())
            } else {
                Some(name.to_doc())
            },
        };
        if publicity.is_importable() {
            imports.register_export(maybe_escape_identifier_string(name))
        }
        imports.register_module(EcoString::from(module), [], [member]);
    }

    /// Add prelude imports based on the used functions.
    fn register_used_prelude_functions(&mut self, imports: &mut Imports<'_>) {
        let path = self.import_path(&self.module.type_info.package, PRELUDE_MODULE_NAME);
        let mut register_prelude_member = |name: &'static str, alias: Option<&'static str>| {
            debug_assert!(
                is_usable_nix_identifier(name) && syntax::is_nix_identifier_or_keyword(name)
                    || alias.is_some()
            );

            let member = Member {
                // Assumes we will always provide an alias when the name
                // would be an invalid identifier or would otherwise clash with
                // a keyword or important variable name (checked above).
                name: syntax::maybe_quoted_attr_set_label(name),
                alias: alias.map(|a| a.to_doc()),
            };
            imports.register_module(path.clone(), [], [member]);
        };

        if self.tracker.ok_used {
            register_prelude_member("Ok", None);
        }

        if self.tracker.error_used {
            register_prelude_member("Error", None);
        }

        if self.tracker.str_has_prefix_used {
            register_prelude_member("strHasPrefix", None);
        }

        if self.tracker.parse_escape_used {
            register_prelude_member("parseEscape", None);
        }

        if self.tracker.parse_number_used {
            register_prelude_member("parseNumber", None);
        }

        if self.tracker.list_used {
            register_prelude_member("toList", None);
        }

        if self.tracker.prepend_used {
            register_prelude_member("prepend", Some("listPrepend"));
        }

        if self.tracker.list_has_at_least_length_used {
            register_prelude_member("listHasAtLeastLength", None);
        }

        if self.tracker.list_has_length_used {
            register_prelude_member("listHasLength", None);
        }

        if self.tracker.make_error_used {
            register_prelude_member("makeError", None);
        }

        if self.tracker.int_remainder_used {
            register_prelude_member("remainderInt", None);
        }

        if self.tracker.float_division_used {
            register_prelude_member("divideFloat", None);
        }

        if self.tracker.int_division_used {
            register_prelude_member("divideInt", None);
        }

        // if self.tracker.object_equality_used {
        //     self.register_prelude_usage(&mut imports, "isEqual", None);
        // }

        if self.tracker.bit_array_literal_used {
            register_prelude_member("toBitArray", None);
        }

        if self.tracker.sized_integer_segment_used {
            register_prelude_member("sizedInt", None);
        }

        if self.tracker.string_bit_array_segment_used {
            register_prelude_member("stringBits", None);
        }

        if self.tracker.codepoint_bit_array_segment_used {
            register_prelude_member("codepointBits", None);
        }

        if self.tracker.bit_array_byte_size_used {
            register_prelude_member("byteSize", None);
        }

        // if self.tracker.float_bit_array_segment_used {
        //     self.register_prelude_usage(&mut imports, "float64Bits", None);
        // }

        if self.tracker.bit_array_byte_at_used {
            register_prelude_member("byteAt", None);
        }

        if self.tracker.bit_array_int_from_slice_used {
            register_prelude_member("intFromBitSlice", None);
        }

        if self.tracker.bit_array_binary_from_slice_used {
            register_prelude_member("binaryFromBitSlice", None);
        }

        if self.tracker.bit_array_slice_after_used {
            register_prelude_member("bitSliceAfter", None);
        }

        if self.tracker.seq_all_used {
            register_prelude_member("seqAll", None);
        }
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
        .map_err(|error| crate::Error::Nix {
            path: path.to_path_buf(),
            src: src.clone(),
            error,
        })?;
    Ok(document.to_pretty_string(80))
}

/// Generates the variable name in Nix for the given module.
pub fn module_var_name(name: &str) -> EcoString {
    eco_format!("{}'", maybe_escape_identifier_string(name))
}

/// Generates the variable name in Nix for the given module (as a document).
pub fn module_var_name_doc(name: &str) -> Document<'_> {
    docvec![maybe_escape_identifier_doc(name), "'"]
}

pub fn is_usable_nix_identifier(word: &str) -> bool {
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

pub fn maybe_escape_identifier_string(word: &str) -> EcoString {
    if is_usable_nix_identifier(word) {
        EcoString::from(word)
    } else {
        escape_identifier(word)
    }
}

pub fn escape_identifier(word: &str) -> EcoString {
    eco_format!("{word}'")
}

pub fn maybe_escape_identifier_doc(word: &str) -> Document<'_> {
    if is_usable_nix_identifier(word) {
        word.to_doc()
    } else {
        escape_identifier(word).to_doc()
    }
}

#[derive(Debug, Default)]
pub(crate) struct UsageTracker {
    pub ok_used: bool,
    pub list_used: bool,
    pub prepend_used: bool,
    pub list_has_at_least_length_used: bool,
    pub list_has_length_used: bool,
    pub error_used: bool,
    pub str_has_prefix_used: bool,
    pub parse_escape_used: bool,
    pub parse_number_used: bool,
    pub int_remainder_used: bool,
    pub make_error_used: bool,
    // pub custom_type_used: bool,
    pub int_division_used: bool,
    pub float_division_used: bool,
    // pub object_equality_used: bool,
    pub bit_array_literal_used: bool,
    pub sized_integer_segment_used: bool,
    pub string_bit_array_segment_used: bool,
    pub codepoint_bit_array_segment_used: bool,
    // pub float_bit_array_segment_used: bool,
    pub bit_array_byte_size_used: bool,
    pub bit_array_byte_at_used: bool,
    pub bit_array_int_from_slice_used: bool,
    pub bit_array_binary_from_slice_used: bool,
    pub bit_array_slice_after_used: bool,
    pub seq_all_used: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Unsupported { feature: String, location: SrcSpan },
}

impl Error {
    /// Returns `true` if the error is [`Unsupported`].
    ///
    /// [`Unsupported`]: crate::nix::Error::Unsupported
    #[must_use]
    pub fn is_unsupported(&self) -> bool {
        matches!(self, Self::Unsupported { .. })
    }
}
