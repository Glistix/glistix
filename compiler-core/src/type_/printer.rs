use ecow::EcoString;
use im::HashMap;
use std::{collections::HashSet, sync::Arc};

use crate::type_::{Type, TypeVar};

use super::PRELUDE_MODULE_NAME;

/// This class keeps track of what names are used for modules in the current
/// scope, so they can be printed in errors, etc.
///
#[derive(Debug, Default)]
pub struct Names {
    /// Types that exist in the current module, either defined or imported in an
    /// unqualified fashion.
    ///
    /// key:   (Defining module name, type name)
    /// value: Alias name
    ///
    /// # Example 1
    ///
    /// ```gleam
    /// type Wibble = wobble.Woo
    /// ```
    /// would result in
    /// - key:   `("wibble", "Woo")`
    /// - value: `"Wibble"`
    ///
    /// # Example 2
    ///
    /// ```gleam
    /// import some/module.{type Wibble}
    /// ```
    /// would result in
    /// - key:   `("some/module", "Wibble")`
    /// - value: `"Wibble"`
    ///
    /// # Example 3
    ///
    /// ```gleam
    /// import some/module.{type Wibble as Wobble}
    /// ```
    /// would result in
    /// - key:   `("some/module", "Wibble")`
    /// - value: `"Wobble"`
    ///
    local_types: HashMap<(EcoString, EcoString), EcoString>,

    /// Types which exist in the prelude, and haven't been shadowed by a local type.
    /// These are a special case, because they are unqualified by default, but can be
    /// shadowed and then must be qualified.
    unshadowed_prelude_types: HashSet<EcoString>,

    /// Mapping of imported modules to their locally used named
    ///
    /// key:   The name of the module
    /// value: The name the module is aliased to
    ///
    /// # Example 1
    ///
    /// ```gleam
    /// import mod1 as my_mod
    /// ```
    /// would result in:
    /// - key:   "mod1"
    /// - value: "my_mod"
    ///
    /// # Example 2
    ///
    /// ```gleam
    /// import mod1
    /// ```
    /// would result in:
    /// - key:   "mod1"
    /// - value: "mod1"
    ///
    imported_modules: HashMap<EcoString, EcoString>,

    /// Generic type parameters that have been annotated in the current
    /// function.
    ///
    /// key:   The id of generic type that was annotated
    /// value: The name that is used for the generic type in the annotation.
    ///
    /// # Example 1
    ///
    /// ```gleam
    /// fn equal(x: something, y: something) -> Bool {
    ///   arg1 == arg2
    /// }
    /// ```
    ///
    /// key:   <some id int>
    /// value: `"something"`
    ///
    type_variables: HashMap<u64, EcoString>,

    /// Constructors which are imported in the current module in an
    /// unqualified fashion.
    ///
    /// key:   (Defining module name, type name)
    /// value: Alias name
    ///
    /// # Example 1
    ///
    /// ```gleam
    /// import wibble.{Wobble}
    /// ```
    /// would result in
    /// - key:   `("wibble", "Wobble")`
    /// - value: `"Wobble"`
    ///
    /// # Example 2
    ///
    /// ```gleam
    /// import wibble.{Wobble as Woo}
    /// ```
    /// would result in
    /// - key:   `("wibble", "Wobble")`
    /// - value: `"Woo"`
    ///
    local_constructors: HashMap<(EcoString, EcoString), EcoString>,

    /// A map from local constructor names to the modules which they refer to.
    /// This helps resolve cases like:
    /// ```gleam
    /// import wibble.{Wobble}
    /// type Wibble { Wobble }
    /// ```
    /// Here, `Wobble` is shadowed, causing `Wobble` not to be valid syntax
    /// for `wibble.Wobble`.
    ///
    /// Each key is the local name of the constructor, and the value is the module
    /// for which the unqualified version is valid. In the above example,
    /// it would result in
    /// - key:   `"Wobble"`
    /// - value: `"module"` (Whatever the current module is)
    ///
    /// But in this case:
    /// ```gleam
    /// import wibble.{Wobble as Wubble}
    /// type Wibble { Wobble }
    /// ```
    /// No shadowing occurs, so this isn't needed.
    ///
    constructor_names: HashMap<EcoString, EcoString>,
}

impl Names {
    pub fn new() -> Self {
        Self {
            local_types: Default::default(),
            unshadowed_prelude_types: Default::default(),
            imported_modules: Default::default(),
            type_variables: Default::default(),
            local_constructors: Default::default(),
            constructor_names: Default::default(),
        }
    }

    /// Record a named type in this module.
    pub fn named_type_in_scope(
        &mut self,
        module_name: EcoString,
        type_name: EcoString,
        local_alias: EcoString,
    ) {
        // If this is a type in the prelude, it is now shadowed.
        _ = self.unshadowed_prelude_types.remove(&local_alias);

        _ = self
            .local_types
            .insert((module_name, type_name), local_alias);
    }

    pub fn prelude_type(&mut self, name: EcoString) {
        _ = self.unshadowed_prelude_types.insert(name);
    }

    /// Record a type variable in this module.
    pub fn type_variable_in_scope(&mut self, id: u64, local_alias: EcoString) {
        _ = self.type_variables.insert(id, local_alias.clone());
    }

    /// Record an imported module in this module.
    pub fn imported_module(&mut self, module_name: EcoString, module_alias: EcoString) {
        _ = self.imported_modules.insert(module_name, module_alias)
    }

    /// Get the name and optional module qualifier for a named type.
    pub fn named_type<'a>(
        &'a self,
        module: &'a EcoString,
        name: &'a EcoString,
    ) -> NameQualifier<'a> {
        let key = (module.clone(), name.clone());

        // There is a local name for this type, use that.
        if let Some(name) = self.local_types.get(&key) {
            return NameQualifier::Unqualified(name.as_str());
        }

        if module == PRELUDE_MODULE_NAME {
            if let Some(prelude_type) = self.unshadowed_prelude_types.get(name) {
                return NameQualifier::Unqualified(prelude_type.as_str());
            }
        }

        // This type is from a module that has been imported
        if let Some(module) = self.imported_modules.get(module) {
            return NameQualifier::Qualified(module, name.as_str());
        };

        NameQualifier::Unimported(name.as_str())
    }

    /// Record a named value in this module.
    pub fn named_constructor_in_scope(
        &mut self,
        module_name: EcoString,
        value_name: EcoString,
        local_alias: EcoString,
    ) {
        _ = self
            .local_constructors
            .insert((module_name.clone(), value_name), local_alias.clone());
        _ = self.constructor_names.insert(local_alias, module_name);
    }

    /// Get the name and optional module qualifier for a named constructor.
    pub fn named_constructor<'a>(
        &'a self,
        module: &'a EcoString,
        name: &'a EcoString,
    ) -> NameQualifier<'a> {
        let key: (EcoString, EcoString) = (module.clone(), name.clone());

        // There is a local name for this value, use that.
        if let Some(name) = self.local_constructors.get(&key) {
            // Only return unqualified syntax if the constructor is not shadowed,
            // and unqualified syntax is valid.
            if self
                .constructor_names
                .get(name)
                .expect("Constructors must be added to both maps")
                == module
            {
                return NameQualifier::Unqualified(name.as_str());
            }
        }

        // This value is from a module that has been imported
        if let Some(module) = self.imported_modules.get(module) {
            return NameQualifier::Qualified(module, name.as_str());
        };

        NameQualifier::Unimported(name.as_str())
    }
}

#[derive(Debug)]
pub enum NameQualifier<'a> {
    /// This type is from a module that has not been imported in this module.
    Unimported(&'a str),
    /// This type has been imported in an unqualifid fashion in this module.
    Unqualified(&'a str),
    /// This type is from a module that has been imported.
    Qualified(&'a str, &'a str),
}

/// A type printer that does not wrap and indent, but does take into account the
/// names that types and modules have been aliased with in the current module.
#[derive(Debug)]
pub struct Printer<'a> {
    names: &'a Names,
    uid: u64,

    /// Some type variables aren't bound to names, so when trying to print those,
    /// we need to create our own names which don't overlap with existing type variables.
    /// These two data structures store a mapping of IDs to created type-variable names,
    /// to ensure consistent printing, and the set of all printed names so that we don't
    /// create a type variable name which matches an existing one.
    ///
    /// Note: These are stored per printer, not per TypeNames struct, because:
    /// - It doesn't really matter what these are, as long as they are consistent.
    /// - We would need mutable access to the names struct, which isn't really possible
    ///   in many contexts.
    ///
    printed_type_variables: HashMap<u64, EcoString>,
    printed_type_variable_names: HashSet<EcoString>,
}

impl<'a> Printer<'a> {
    pub fn new(names: &'a Names) -> Self {
        Printer {
            names,
            uid: Default::default(),
            printed_type_variables: Default::default(),
            printed_type_variable_names: names.type_variables.values().cloned().collect(),
        }
    }

    pub fn print_type(&mut self, type_: &Type) -> EcoString {
        let mut buffer = EcoString::new();
        self.print(type_, &mut buffer);
        buffer
    }

    fn print(&mut self, type_: &Type, buffer: &mut EcoString) {
        match type_ {
            Type::Named {
                name, args, module, ..
            } => {
                let (module, name) = match self.names.named_type(module, name) {
                    NameQualifier::Qualified(m, n) => (Some(m), n),
                    NameQualifier::Unqualified(n) => (None, n),
                    // TODO: indicate that the module is not import and as such
                    // needs to be, as well as how.
                    NameQualifier::Unimported(n) => {
                        (Some(module.split('/').last().unwrap_or(module)), n)
                    }
                };

                if let Some(module) = module {
                    buffer.push_str(module);
                    buffer.push('.');
                }
                buffer.push_str(name);

                if !args.is_empty() {
                    buffer.push('(');
                    self.print_arguments(args, buffer);
                    buffer.push(')');
                }
            }

            Type::Fn { args, retrn } => {
                buffer.push_str("fn(");
                self.print_arguments(args, buffer);
                buffer.push_str(") -> ");
                self.print(retrn, buffer);
            }

            Type::Var { type_, .. } => match *type_.borrow() {
                TypeVar::Link { ref type_, .. } => self.print(type_, buffer),
                TypeVar::Unbound { id, .. } | TypeVar::Generic { id, .. } => {
                    buffer.push_str(&self.type_variable(id))
                }
            },

            Type::Tuple { elems, .. } => {
                buffer.push_str("#(");
                self.print_arguments(elems, buffer);
                buffer.push(')');
            }
        }
    }

    fn print_arguments(&mut self, args: &[Arc<Type>], typ_str: &mut EcoString) {
        for (i, arg) in args.iter().enumerate() {
            self.print(arg, typ_str);
            if i < args.len() - 1 {
                typ_str.push_str(", ");
            }
        }
    }

    /// A suitable name of a type variable.
    pub fn type_variable(&mut self, id: u64) -> EcoString {
        if let Some(name) = self.names.type_variables.get(&id) {
            return name.clone();
        }

        if let Some(name) = self.printed_type_variables.get(&id) {
            return name.clone();
        }

        loop {
            let name = self.next_letter();
            if !self.printed_type_variable_names.contains(&name) {
                _ = self.printed_type_variable_names.insert(name.clone());
                _ = self.printed_type_variables.insert(id, name.clone());
                return name;
            }
        }
    }

    fn next_letter(&mut self) -> EcoString {
        let alphabet_length = 26;
        let char_offset = 97;
        let mut chars = vec![];
        let mut n;
        let mut rest = self.uid;

        loop {
            n = rest % alphabet_length;
            rest /= alphabet_length;
            chars.push((n as u8 + char_offset) as char);

            if rest == 0 {
                break;
            }
            rest -= 1
        }

        self.uid += 1;
        chars.into_iter().rev().collect()
    }
}

#[test]
fn test_local_type() {
    let mut names = Names::new();
    names.named_type_in_scope("mod".into(), "Tiger".into(), "Cat".into());
    let mut printer = Printer::new(&names);

    let type_ = Type::Named {
        name: "Tiger".into(),
        args: vec![],
        module: "mod".into(),
        publicity: crate::ast::Publicity::Public,
        package: "".into(),
    };

    assert_eq!(printer.print_type(&type_), "Cat");
}

#[test]
fn test_prelude_type() {
    let mut names = Names::new();
    names.prelude_type("Int".into());
    let mut printer = Printer::new(&names);

    let type_ = Type::Named {
        name: "Int".into(),
        args: vec![],
        module: "gleam".into(),
        publicity: crate::ast::Publicity::Public,
        package: "".into(),
    };

    assert_eq!(printer.print_type(&type_), "Int");
}

#[test]
fn test_shadowed_prelude_type() {
    let mut names = Names::new();

    names.prelude_type("Int".into());
    names.named_type_in_scope("mod".into(), "Int".into(), "Int".into());

    let mut printer = Printer::new(&names);

    let type_ = Type::Named {
        name: "Int".into(),
        args: vec![],
        module: "gleam".into(),
        publicity: crate::ast::Publicity::Public,
        package: "".into(),
    };

    assert_eq!(printer.print_type(&type_), "gleam.Int");
}

#[test]
fn test_generic_type_annotation() {
    let mut names = Names::new();
    names.type_variable_in_scope(0, "one".into());
    let mut printer = Printer::new(&names);

    let type_ = Type::Var {
        type_: Arc::new(std::cell::RefCell::new(TypeVar::Generic { id: 0 })),
    };

    assert_eq!(printer.print_type(&type_), "one");
}

#[test]
fn test_generic_type_var() {
    let names = Names::new();
    let mut printer = Printer::new(&names);

    let type_ = Type::Var {
        type_: Arc::new(std::cell::RefCell::new(TypeVar::Unbound { id: 0 })),
    };

    let typ2 = Type::Var {
        type_: Arc::new(std::cell::RefCell::new(TypeVar::Unbound { id: 1 })),
    };

    assert_eq!(printer.print_type(&type_), "a");
    assert_eq!(printer.print_type(&typ2), "b");
}

#[test]
fn test_tuple_type() {
    let names = Names::new();
    let mut printer = Printer::new(&names);

    let type_ = Type::Tuple {
        elems: vec![
            Arc::new(Type::Named {
                name: "Int".into(),
                args: vec![],
                module: "gleam".into(),
                publicity: crate::ast::Publicity::Public,
                package: "".into(),
            }),
            Arc::new(Type::Named {
                name: "String".into(),
                args: vec![],
                module: "gleam".into(),
                publicity: crate::ast::Publicity::Public,
                package: "".into(),
            }),
        ],
    };

    assert_eq!(printer.print_type(&type_), "#(gleam.Int, gleam.String)");
}

#[test]
fn test_fn_type() {
    let mut names = Names::new();
    names.prelude_type("Int".into());
    names.prelude_type("Bool".into());
    let mut printer = Printer::new(&names);

    let type_ = Type::Fn {
        args: vec![
            Arc::new(Type::Named {
                name: "Int".into(),
                args: vec![],
                module: "gleam".into(),
                publicity: crate::ast::Publicity::Public,
                package: "".into(),
            }),
            Arc::new(Type::Named {
                name: "String".into(),
                args: vec![],
                module: "gleam".into(),
                publicity: crate::ast::Publicity::Public,
                package: "".into(),
            }),
        ],
        retrn: Arc::new(Type::Named {
            name: "Bool".into(),
            args: vec![],
            module: "gleam".into(),
            publicity: crate::ast::Publicity::Public,
            package: "".into(),
        }),
    };

    assert_eq!(printer.print_type(&type_), "fn(Int, gleam.String) -> Bool");
}

#[test]
fn test_module_alias() {
    let mut names = Names::new();
    names.imported_module("mod1".into(), "animals".into());
    let mut printer = Printer::new(&names);

    let type_ = Type::Named {
        name: "Cat".into(),
        args: vec![],
        module: "mod1".into(),
        publicity: crate::ast::Publicity::Public,
        package: "".into(),
    };

    assert_eq!(printer.print_type(&type_), "animals.Cat");
}

#[test]
fn test_type_alias_and_generics() {
    let mut names = Names::new();

    names.named_type_in_scope("mod".into(), "Tiger".into(), "Cat".into());

    names.type_variable_in_scope(0, "one".into());

    let mut printer = Printer::new(&names);

    let type_ = Type::Named {
        name: "Tiger".into(),
        args: vec![Arc::new(Type::Var {
            type_: Arc::new(std::cell::RefCell::new(TypeVar::Generic { id: 0 })),
        })],
        module: "mod".into(),
        publicity: crate::ast::Publicity::Public,
        package: "".into(),
    };

    assert_eq!(printer.print_type(&type_), "Cat(one)");
}

#[test]
fn test_unqualified_import_and_generic() {
    let mut names = Names::new();

    names.named_type_in_scope("mod".into(), "Cat".into(), "C".into());

    names.type_variable_in_scope(0, "one".into());

    let mut printer = Printer::new(&names);

    let type_ = Type::Named {
        name: "Cat".into(),
        args: vec![Arc::new(Type::Var {
            type_: Arc::new(std::cell::RefCell::new(TypeVar::Generic { id: 0 })),
        })],
        module: "mod".into(),
        publicity: crate::ast::Publicity::Public,
        package: "".into(),
    };

    assert_eq!(printer.print_type(&type_), "C(one)");
}

#[test]
fn nested_module() {
    let names = Names::new();
    let mut printer = Printer::new(&names);
    let type_ = Type::Named {
        name: "Cat".into(),
        args: vec![],
        module: "one/two/three".into(),
        publicity: crate::ast::Publicity::Public,
        package: "".into(),
    };

    assert_eq!(printer.print_type(&type_), "three.Cat");
}

#[test]
fn test_unqualified_import_and_module_alias() {
    let mut names = Names::new();

    names.imported_module("mod1".into(), "animals".into());

    let _ = names
        .local_types
        .insert(("mod1".into(), "Cat".into()), "C".into());

    let mut printer = Printer::new(&names);

    let type_ = Type::Named {
        name: "Cat".into(),
        args: vec![],
        module: "mod1".into(),
        publicity: crate::ast::Publicity::Public,
        package: "".into(),
    };

    assert_eq!(printer.print_type(&type_), "C");
}

#[test]
fn test_module_imports() {
    let mut names = Names::new();
    names.imported_module("mod".into(), "animals".into());
    let _ = names
        .local_types
        .insert(("mod2".into(), "Cat".into()), "Cat".into());

    let mut printer = Printer::new(&names);

    let type_ = Type::Named {
        name: "Cat".into(),
        args: vec![],
        module: "mod".into(),
        publicity: crate::ast::Publicity::Public,
        package: "".into(),
    };

    let typ1 = Type::Named {
        name: "Cat".into(),
        args: vec![],
        module: "mod2".into(),
        publicity: crate::ast::Publicity::Public,
        package: "".into(),
    };

    assert_eq!(printer.print_type(&type_), "animals.Cat");
    assert_eq!(printer.print_type(&typ1), "Cat");
}

#[test]
fn test_multiple_generic_annotations() {
    let mut names = Names::new();

    names.type_variable_in_scope(0, "one".into());
    names.type_variable_in_scope(1, "two".into());

    let mut printer = Printer::new(&names);

    let type_ = Type::Named {
        name: "Tiger".into(),
        args: vec![
            Arc::new(Type::Var {
                type_: Arc::new(std::cell::RefCell::new(TypeVar::Generic { id: 0 })),
            }),
            Arc::new(Type::Var {
                type_: Arc::new(std::cell::RefCell::new(TypeVar::Generic { id: 1 })),
            }),
        ],
        module: "tigermodule".into(),
        publicity: crate::ast::Publicity::Public,
        package: "".into(),
    };

    let typ1 = Type::Var {
        type_: Arc::new(std::cell::RefCell::new(TypeVar::Generic { id: 2 })),
    };

    assert_eq!(printer.print_type(&type_), "tigermodule.Tiger(one, two)");
    assert_eq!(printer.print_type(&typ1), "a");
}

#[test]
fn test_variable_name_already_in_scope() {
    let mut names = Names::new();

    names.type_variable_in_scope(1, "a".into());
    names.type_variable_in_scope(2, "b".into());

    let mut printer = Printer::new(&names);

    let type_ = |id| Type::Var {
        type_: Arc::new(std::cell::RefCell::new(TypeVar::Generic { id })),
    };

    assert_eq!(printer.print_type(&type_(0)), "c");
    assert_eq!(printer.print_type(&type_(1)), "a");
    assert_eq!(printer.print_type(&type_(2)), "b");
    assert_eq!(printer.print_type(&type_(3)), "d");
}
