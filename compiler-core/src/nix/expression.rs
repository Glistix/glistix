use crate::analyse::TargetSupport;
use crate::ast::{
    Statement, TypedAssignment, TypedExpr, TypedModule, TypedPattern, TypedStatement,
};
use crate::docvec;
use crate::javascript::Output;
use crate::line_numbers::LineNumbers;
use crate::nix::{maybe_escape_identifier_doc, INDENT};
use crate::pretty::{break_, Document, Documentable};
use ecow::EcoString;
use itertools::Itertools;

/// Generates a Nix expression.
struct Generator<'output, 'module> {
    module: &'module TypedModule,
    line_numbers: &'module LineNumbers,
    target_support: TargetSupport,
    current_scope_vars: im::HashMap<EcoString, usize>,
    /// If this is not empty, we need to insert a 'let ... in'.
    assignments: Vec<Document<'output>>,
}

impl<'output, 'module> Generator<'output, 'module> {
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
            assignments: vec![],
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

    fn statement(&mut self, statement: &'output TypedStatement) -> Output<'output> {
        match statement {
            Statement::Expression(expression) => self.expression(expression),
            Statement::Assignment(assignment) => {
                let assignment = self.assignment(assignment)?;
                self.assignments.push(assignment);
                Ok(Document::Str(""))
            }
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
            return Ok(docvec![nix_name, " =", break_("", " "), subject, ";"]);
        }

        // Patterns
        todo!()
    }

    /// Some expressions in Nix may be displayed with spaces.
    /// Those expressions need to be wrapped in parentheses so that they aren't
    /// parsed as separate list elements or function call arguments, for example.
    pub fn wrap_expression_with_spaces<'a>(&mut self, expression: &'a TypedExpr) -> Output<'a> {
        match expression {
            TypedExpr::Fn { .. }
            | TypedExpr::Call { .. }
            | TypedExpr::BinOp { .. }
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
