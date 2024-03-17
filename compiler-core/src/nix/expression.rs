use crate::analyse::TargetSupport;
use crate::ast::{
    Statement, TypedAssignment, TypedExpr, TypedModule, TypedPattern, TypedStatement,
};
use crate::docvec;
use crate::javascript::Output;
use crate::line_numbers::LineNumbers;
use crate::nix::{maybe_escape_identifier_doc, INDENT};
use crate::pretty::{break_, concat, line, Document, Documentable};
use crate::type_::{ValueConstructor, ValueConstructorVariant};
use ecow::{eco_format, EcoString};
use itertools::Itertools;
use vec1::Vec1;

/// Generates a Nix expression.
struct Generator<'module> {
    module: &'module TypedModule,
    line_numbers: &'module LineNumbers,
    target_support: TargetSupport,
    current_scope_vars: im::HashMap<EcoString, usize>,
}

impl<'module> Generator<'module> {
    pub fn new(
        module: &'module TypedModule,
        line_numbers: &'module LineNumbers,
        target_support: TargetSupport,
        current_scope_vars: im::HashMap<EcoString, usize>,
    ) -> Self {
        Self {
            module,
            line_numbers,
            target_support,
            current_scope_vars,
        }
    }

    pub fn local_var<'a>(&mut self, name: &'a EcoString) -> Document<'a> {
        match self.current_scope_vars.get(name) {
            None => {
                let _ = self.current_scope_vars.insert(name.clone(), 0);
                maybe_escape_identifier_doc(name)
            }
            Some(0) => maybe_escape_identifier_doc(name),
            Some(n) if name == "_''" => Document::String(format!("_''{n}")),
            Some(n) => Document::String(format!("{name}'{n}")),
        }
    }

    pub fn next_local_var<'a>(&mut self, name: &'a EcoString) -> Document<'a> {
        let next = self.current_scope_vars.get(name).map_or(0, |i| i + 1);
        let _ = self.current_scope_vars.insert(name.clone(), next);
        self.local_var(name)
    }

    fn next_anonymous_var<'a>(&mut self) -> Document<'a> {
        let name = "_''";
        let next = self.current_scope_vars.get(name).map_or(0, |i| i + 1);
        let _ = self.current_scope_vars.insert(eco_format!("{name}"), next);
        Document::String(format!("{name}{next}"))
    }

    /// Every statement, in Nix, must be an assignment, even an expression.
    fn statement<'a>(&mut self, statement: &'a TypedStatement) -> Output<'a> {
        match statement {
            Statement::Expression(expression) => {
                let subject = self.expression(expression)?;
                let name = self.next_anonymous_var();
                // Convert expression to assignment with irrelevant name
                self.assignment_line(name, subject)
            }
            Statement::Assignment(assignment) => self.assignment(assignment),
            Statement::Use(_use) => {
                unreachable!("Use must not be present for Nix generation")
            }
        }
    }

    pub fn expression<'a>(&mut self, expression: &'a TypedExpr) -> Output<'a> {
        match expression {
            TypedExpr::String { value, .. } => Ok(string(value)),
            TypedExpr::Int { value, .. } => Ok(int(value)),
            TypedExpr::Float { value, .. } => Ok(float(value)),
            TypedExpr::List { elements, tail, .. } => {
                let head = list(
                    elements
                        .iter()
                        .map(|element| self.wrap_expression_with_spaces(element)),
                )?;

                match tail {
                    Some(tail) => Ok(docvec![
                        head,
                        " ++",
                        break_("", " "),
                        self.wrap_expression_with_spaces(tail)?
                    ]
                    .group()),

                    None => Ok(head),
                }
            }
            TypedExpr::Block { statements, .. } => self.block(statements),
            TypedExpr::Var {
                name, constructor, ..
            } => self.variable(name, constructor),
            _ => todo!(),
        }
    }

    pub fn assignment<'a>(&mut self, assignment: &'a TypedAssignment) -> Output<'a> {
        let TypedAssignment {
            pattern,
            kind: _,
            value,
            annotation: _,
            location: _,
        } = assignment;

        // If it is a simple assignment to a variable we can generate a normal
        // JS assignment
        if let TypedPattern::Variable { name, .. } = pattern {
            // Subject must be rendered before the variable for variable numbering
            let subject = self.expression(value)?;
            let nix_name = self.next_local_var(name);
            return self.assignment_line(nix_name, subject);
        }

        // Patterns
        todo!()
    }

    fn assignment_line<'a>(&mut self, name: Document<'a>, value: Document<'a>) -> Output<'a> {
        Ok(docvec![name, " =", break_("", " "), value, ";"])
    }

    fn block<'a>(&mut self, statements: &'a Vec1<TypedStatement>) -> Output<'a> {
        if statements.len() == 1 {
            self.expression_from_statement(statements.first())
        } else {
            self.statements(statements)
        }
    }

    /// In Nix, statements are translated to 'let ... in' syntax.
    fn statements<'a>(&mut self, statements: &'a [TypedStatement]) -> Output<'a> {
        let Some(trailing_statement) = statements.last() else {
            // TODO: can we unwrap?
            return Ok(Document::Str(""));
        };

        let assignments = Itertools::intersperse_with(
            statements
                .iter()
                .take(statements.len().saturating_sub(1))
                .map(|statement| self.statement(statement)),
            || Ok(line()),
        )
        .collect::<Result<Vec<_>, _>>()?;

        Ok(docvec![
            break_("", ""),
            "let",
            line(),
            concat(assignments).nest(INDENT).group(),
            line(),
            "in",
            break_("", " "),
            self.expression_from_statement(trailing_statement)?,
        ])
    }

    fn variable<'a>(
        &mut self,
        name: &'a EcoString,
        constructor: &'a ValueConstructor,
    ) -> Output<'a> {
        match &constructor.variant {
            ValueConstructorVariant::LocalConstant { literal: _ } => {
                todo!()
            }
            ValueConstructorVariant::Record { arity: _, .. } => {
                todo!()
            }
            ValueConstructorVariant::ModuleFn { .. }
            | ValueConstructorVariant::ModuleConstant { .. }
            | ValueConstructorVariant::LocalVariable { .. } => Ok(self.local_var(name)),
        }
    }

    /// Outputs the expression which would replace a statement if it were the
    /// last one.
    fn expression_from_statement<'a>(&mut self, statement: &'a TypedStatement) -> Output<'a> {
        match statement {
            Statement::Expression(expression) => self.expression(expression),

            Statement::Assignment(assignment) => self.expression(assignment.value.as_ref()),

            Statement::Use(_) => {
                unreachable!("use statements must not be present for Nix generation")
            }
        }
    }

    /// Some expressions in Nix may be displayed with spaces.
    /// Those expressions need to be wrapped in parentheses so that they aren't
    /// parsed as separate list elements or function call arguments, for example.
    pub fn wrap_expression_with_spaces<'a>(&mut self, expression: &'a TypedExpr) -> Output<'a> {
        // TODO: Recheck
        match expression {
            TypedExpr::Block { statements, .. } if statements.len() == 1 => {
                match statements.first() {
                    Statement::Expression(expression) => {
                        // A block with one expression is just that expression,
                        // so wrap it only if needed.
                        self.wrap_expression_with_spaces(expression)
                    }
                    Statement::Assignment(assignment) => {
                        self.wrap_expression_with_spaces(assignment.value.as_ref())
                    }
                    Statement::Use(_) => {
                        unreachable!("use statements must not be present for Nix generation")
                    }
                }
            },

            TypedExpr::Block { .. }
            | TypedExpr::Pipeline { .. }
            | TypedExpr::Fn { .. }
            | TypedExpr::Call { .. }
            | TypedExpr::BinOp { .. }
            // Expands into 'if':
            | TypedExpr::Case { .. }
            // Expand into calls:
            | TypedExpr::TupleIndex { .. }
            | TypedExpr::Todo { .. }
            | TypedExpr::Panic { .. } => Ok(docvec!["(", self.expression(expression)?, ")"]),

            _ => self.expression(expression),
        }
    }
}

/// Generates a valid Nix string.
pub fn string(value: &str) -> Document<'_> {
    if value.contains('\n') {
        Document::String(value.replace('\n', r"\n")).surround("\"", "\"")
    } else {
        value.to_doc().surround("\"", "\"")
    }
}

/// Generates a valid Nix integer.
pub fn int(value: &str) -> Document<'_> {
    let mut out = EcoString::with_capacity(value.len());

    if value.starts_with('-') {
        out.push('-');
    }

    // Ignore '+' at the beginning (no Nix support)
    let value = value.trim_start_matches(['+', '-'].as_ref());

    let value = if value.starts_with("0x") {
        todo!("Implement 0x Nix support")
    } else if value.starts_with("0o") {
        todo!("Implement 0o Nix support")
    } else if value.starts_with("0b") {
        todo!("Implement 0b Nix support")
    } else {
        value
    };

    let value = value.trim_start_matches('0');
    if value.is_empty() {
        out.push('0');
    }
    out.push_str(value);

    out.to_doc()
}

/// Generates a valid Nix float.
pub fn float(value: &str) -> Document<'_> {
    let mut out = EcoString::with_capacity(value.len());

    if value.starts_with('-') {
        out.push('-');
    }

    // Ignore '+' at the beginning (no Nix support)
    let value = value.trim_start_matches(['+', '-'].as_ref());

    let value = value.trim_start_matches('0');
    if value.starts_with(['.', 'e', 'E']) {
        out.push('0');
    }
    out.push_str(value);

    out.to_doc()
}

pub fn list<'a, Elements: IntoIterator<Item = Output<'a>>>(elements: Elements) -> Output<'a> {
    let elements = Itertools::intersperse(elements.into_iter(), Ok(break_("", " ")))
        .collect::<Result<Vec<_>, _>>()?
        .to_doc();
    Ok(docvec![
        "[",
        docvec![break_("", ""), elements].nest(INDENT),
        break_("", " "),
        "]"
    ]
    .group())
}
