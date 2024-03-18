use crate::analyse::TargetSupport;
use crate::ast::{
    Arg, BinOp, CallArg, SrcSpan, Statement, TypedArg, TypedAssignment, TypedExpr, TypedModule,
    TypedPattern, TypedRecordUpdateArg, TypedStatement,
};
use crate::docvec;
use crate::javascript::Output;
use crate::line_numbers::LineNumbers;
use crate::nix::{
    fun_args, is_nix_keyword, maybe_escape_identifier_doc, try_wrap_attr_set, INDENT,
};
use crate::pretty::{break_, join, line, Document, Documentable};
use crate::type_::{ModuleValueConstructor, Type, ValueConstructor, ValueConstructorVariant};
use ecow::{eco_format, EcoString};
use itertools::Itertools;
use std::sync::Arc;
use vec1::Vec1;

/// Generates a Nix expression.
pub(crate) struct Generator<'module> {
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
                        .map(|element| self.wrap_child_expression(element)),
                )?;

                match tail {
                    Some(tail) => Ok(docvec![head, " ++ ", self.wrap_child_expression(tail)?]),

                    None => Ok(head),
                }
            }

            TypedExpr::Tuple { elems, .. } => self.tuple(elems),
            TypedExpr::TupleIndex { tuple, index, .. } => self.tuple_index(tuple, *index),

            TypedExpr::Pipeline {
                assignments,
                finally,
                ..
            } => self.pipeline(assignments, finally),
            TypedExpr::Block { statements, .. } => self.block(statements),
            TypedExpr::Var {
                name, constructor, ..
            } => self.variable(name, constructor),

            TypedExpr::Fn { args, body, .. } => self.fn_(args, body),
            TypedExpr::Call { fun, args, .. } => self.call(fun, args),
            TypedExpr::Panic {
                location, message, ..
            } => self.panic(location, message.as_deref()),
            TypedExpr::Todo {
                location, message, ..
            } => self.todo(location, message.as_deref()),

            TypedExpr::BinOp {
                name, left, right, ..
            } => self.bin_op(name, left, right),
            TypedExpr::NegateBool { value, .. } => self.negate_with("!", value),
            TypedExpr::NegateInt { value, .. } => self.negate_with("-", value),

            TypedExpr::RecordAccess { label, record, .. } => self.record_access(record, label),
            TypedExpr::RecordUpdate { spread, args, .. } => self.record_update(spread, args),

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
        Ok(docvec![
            name,
            " =",
            break_("", " "),
            value.nest(INDENT),
            ";"
        ])
    }

    fn block<'a>(&mut self, statements: &'a Vec1<TypedStatement>) -> Output<'a> {
        if statements.len() == 1 {
            self.expression_from_statement(statements.first())
        } else {
            // Entering a new scope
            let scope = self.current_scope_vars.clone();
            let output = self.statements(statements)?;

            // Reset scope
            self.current_scope_vars = scope;
            Ok(output)
        }
    }

    /// In Nix, statements are translated to 'let ... in' syntax.
    fn statements<'a>(&mut self, statements: &'a [TypedStatement]) -> Output<'a> {
        let Some((trailing_statement, assignments)) = statements.split_last() else {
            // TODO: can we unwrap?
            return Ok(Document::Str(""));
        };

        if assignments.is_empty() {
            return self.expression_from_statement(trailing_statement);
        }

        let assignments = assignments
            .iter()
            .map(|statement| self.statement(statement))
            .collect::<Result<Vec<_>, _>>()?;

        let body = self.expression_from_statement(trailing_statement)?;

        self.let_in(assignments, body)
    }

    fn pipeline<'a>(
        &mut self,
        assignments: &'a [TypedAssignment],
        finally: &'a TypedExpr,
    ) -> Output<'a> {
        if assignments.is_empty() {
            return self.expression(finally);
        }

        // Entering a new scope
        let scope = self.current_scope_vars.clone();
        let assignments = assignments
            .iter()
            .map(|assignment| self.assignment(assignment))
            .collect::<Result<Vec<_>, _>>()?;

        let body = self.expression(finally)?;

        // Exiting scope
        self.current_scope_vars = scope;

        self.let_in(assignments, body)
    }

    fn let_in<'a>(
        &mut self,
        assignments: impl IntoIterator<Item = Document<'a>>,
        body: Document<'a>,
    ) -> Output<'a> {
        Ok(docvec![
            break_("", ""),
            "let",
            docvec![line(), join(assignments, line())].nest(INDENT),
            line(),
            docvec!["in", break_("", " "), body].group(),
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
    /// This function wraps the expression in parentheses if it would have spaces in
    /// its representation or if it could generate a potentially ambiguous
    /// expansion (such as with [`TypedExpr::NegateInt`]).
    pub fn wrap_child_expression<'a>(&mut self, expression: &'a TypedExpr) -> Output<'a> {
        // TODO: Recheck
        match expression {
            TypedExpr::Block { statements, .. } if statements.len() == 1 => {
                match statements.first() {
                    Statement::Expression(expression) => {
                        // A block with one expression is just that expression,
                        // so wrap it only if needed.
                        self.wrap_child_expression(expression)
                    }
                    Statement::Assignment(assignment) => {
                        self.wrap_child_expression(assignment.value.as_ref())
                    }
                    Statement::Use(_) => {
                        unreachable!("use statements must not be present for Nix generation")
                    }
                }
            },

            TypedExpr::Pipeline {
                assignments,
                finally,
                ..
            } if assignments.is_empty() => {
                self.wrap_child_expression(finally)
            }

            // Negative numbers can trip up the Nix parser in some positions,
            // such as lists:
            TypedExpr::Int { value, .. }
            | TypedExpr::Float { value, .. } if value.starts_with('-') => {
                Ok(docvec!["(", self.expression(expression)?, ")"])
            },

            TypedExpr::Block { .. }
            | TypedExpr::Pipeline { .. }
            | TypedExpr::Fn { .. }
            | TypedExpr::Call { .. }
            | TypedExpr::BinOp { .. }
            // Negated values are invalid in some positions, such as lists:
            | TypedExpr::NegateBool { .. }
            | TypedExpr::NegateInt { .. }
            // Expands into 'if':
            | TypedExpr::Case { .. }
            // Expand into calls:
            | TypedExpr::Todo { .. }
            | TypedExpr::Panic { .. } => Ok(docvec!["(", self.expression(expression)?, ")"]),

            _ => self.expression(expression),
        }
    }
}

/// Function-related code generation.
impl Generator<'_> {
    fn call<'a>(&mut self, fun: &'a TypedExpr, arguments: &'a [CallArg<TypedExpr>]) -> Output<'a> {
        let arguments = arguments
            .iter()
            .map(|argument| self.wrap_child_expression(&argument.value))
            .try_collect()?;

        self.call_with_doc_args(fun, arguments)
    }

    fn call_with_doc_args<'a>(
        &mut self,
        fun: &'a TypedExpr,
        arguments: Vec<Document<'a>>,
    ) -> Output<'a> {
        match fun {
            // Qualified record construction
            TypedExpr::ModuleSelect {
                constructor: ModuleValueConstructor::Record { name: _, .. },
                module_alias: _,
                ..
            } => Ok(todo!("construct_record")),

            // Record construction
            TypedExpr::Var {
                constructor:
                    ValueConstructor {
                        variant: ValueConstructorVariant::Record { .. },
                        type_: _,
                        ..
                    },
                name: _,
                ..
            } => {
                todo!("construct_record, Ok/Err")
            }

            _ => {
                let fun = self.wrap_child_expression(fun)?;
                let arguments = join(arguments, break_("", " "));
                Ok(docvec![fun, break_("", " "), arguments])
            }
        }
    }

    pub fn fn_<'a>(&mut self, arguments: &'a [TypedArg], body: &'a [TypedStatement]) -> Output<'a> {
        let scope = self.current_scope_vars.clone();
        for name in arguments.iter().flat_map(Arg::get_variable_name) {
            let _ = self.current_scope_vars.insert(name.clone(), 0);
        }

        // Generate the function body
        let result = self.statements(body);

        // Reset scope
        self.current_scope_vars = scope;

        Ok(docvec!(fun_args(arguments), break_("", " "), result?)
            .nest(INDENT)
            .group())
    }

    fn todo<'a>(&mut self, location: &'a SrcSpan, message: Option<&'a TypedExpr>) -> Output<'a> {
        let message = match message {
            Some(m) => self.expression(m)?,
            None => string("This has not yet been implemented"),
        };

        Ok(self.throw_error("todo", &message, *location, vec![]))
    }

    fn panic<'a>(&mut self, location: &'a SrcSpan, message: Option<&'a TypedExpr>) -> Output<'a> {
        let message = match message {
            Some(m) => self.expression(m)?,
            None => string("panic expression evaluated"),
        };

        Ok(self.throw_error("todo", &message, *location, vec![]))
    }

    fn throw_error<'a, Fields>(
        &mut self,
        _error_name: &'a str,
        message: &Document<'a>,
        _location: SrcSpan,
        _fields: Fields,
    ) -> Document<'a>
    where
        Fields: IntoIterator<Item = (&'a str, Document<'a>)>,
    {
        // let module = self.module.name.clone().to_doc().surround('"', '"');
        // TODO: Function name
        // let line = self.line_numbers.line_number(location.start).to_doc();

        // TODO: Use prelude error, pass fields
        // let fields = wrap_attr_set(fields.into_iter().map(|(k, v)| (k.to_doc(), Some(v))));

        // TODO: Insert module and line
        docvec!["builtins.throw", break_("", " "), message.clone()]
    }
}

// Operators.
impl Generator<'_> {
    fn bin_op<'a>(
        &mut self,
        name: &'a BinOp,
        left: &'a TypedExpr,
        right: &'a TypedExpr,
    ) -> Output<'a> {
        match name {
            BinOp::And => self.print_bin_op(left, right, "&&"),
            BinOp::Or => self.print_bin_op(left, right, "||"),
            BinOp::LtInt | BinOp::LtFloat => self.print_bin_op(left, right, "<"),
            BinOp::LtEqInt | BinOp::LtEqFloat => self.print_bin_op(left, right, "<="),
            BinOp::Eq => self.equal(left, right, true),
            BinOp::NotEq => self.equal(left, right, false),
            BinOp::GtInt | BinOp::GtFloat => self.print_bin_op(left, right, ">"),
            BinOp::GtEqInt | BinOp::GtEqFloat => self.print_bin_op(left, right, ">="),
            BinOp::Concatenate | BinOp::AddInt | BinOp::AddFloat => {
                self.print_bin_op(left, right, "+")
            }
            BinOp::SubInt | BinOp::SubFloat => self.print_bin_op(left, right, "-"),
            BinOp::MultInt | BinOp::MultFloat => self.print_bin_op(left, right, "*"),
            BinOp::RemainderInt => todo!("use remainder from prelude"),
            BinOp::DivInt => todo!("possibly use div int from prelude"),
            BinOp::DivFloat => todo!("possibly use div float from prelude"),
        }
    }

    fn print_bin_op<'a>(
        &mut self,
        left: &'a TypedExpr,
        right: &'a TypedExpr,
        op: &'a str,
    ) -> Output<'a> {
        let left = self.wrap_child_expression(left)?;
        let right = self.wrap_child_expression(right)?;
        Ok(docvec!(left, " ", op, " ", right))
    }

    fn equal<'a>(
        &mut self,
        left: &'a TypedExpr,
        right: &'a TypedExpr,
        should_be_equal: bool,
    ) -> Output<'a> {
        // If it is a simple scalar type then we can use Nix's simple equality
        if is_nix_scalar(left.type_()) {
            return self.print_bin_op(left, right, if should_be_equal { "==" } else { "!=" });
        }

        // Other types must be compared using structural equality
        todo!("track prelude equals call")
        // Ok(self.prelude_equal_call(should_be_equal, left, right))
    }

    fn negate_with<'a>(&mut self, with: &'static str, value: &'a TypedExpr) -> Output<'a> {
        Ok(docvec!(with, self.wrap_child_expression(value)?))
    }
}

/// Record-related methods.
impl Generator<'_> {
    fn tuple<'a>(&mut self, elements: &'a [TypedExpr]) -> Output<'a> {
        let fields = elements
            .iter()
            .enumerate()
            .map(|(i, element)| (Document::String(format!("_{i}")), self.expression(element)));

        try_wrap_attr_set(fields)
    }

    fn tuple_index<'a>(&mut self, tuple: &'a TypedExpr, index: u64) -> Output<'a> {
        let tuple = self.wrap_child_expression(tuple)?;
        Ok(docvec![tuple, Document::String(format!("._{index}"))])
    }

    fn record_access<'a>(&mut self, record: &'a TypedExpr, label: &'a str) -> Output<'a> {
        let record = self.wrap_child_expression(record)?;
        Ok(docvec![record, ".", maybe_quoted_attr_set_label(label)])
    }

    fn record_update<'a>(
        &mut self,
        record: &'a TypedExpr,
        updates: &'a [TypedRecordUpdateArg],
    ) -> Output<'a> {
        let record = self.wrap_child_expression(record)?;
        let fields = updates
            .iter()
            .map(|TypedRecordUpdateArg { label, value, .. }| {
                (
                    maybe_quoted_attr_set_label(label),
                    self.wrap_child_expression(value),
                )
            });
        let set = try_wrap_attr_set(fields)?;
        Ok(docvec![record, " // ", set])
    }
}

/// Types which are trivially comparable for equality.
pub fn is_nix_scalar(t: Arc<Type>) -> bool {
    t.is_int() || t.is_float() || t.is_bool() || t.is_nil() || t.is_string()
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
        docvec![break_("", " "), elements].nest(INDENT),
        break_("", " "),
        "]"
    ]
    .group())
}

/// If the label would be a keyword, it is quoted.
/// Assumes the label is a valid Gleam identifier, thus doesn't check for other
/// invalid attribute names.
pub fn maybe_quoted_attr_set_label(label: &str) -> Document<'_> {
    if is_nix_keyword(label) {
        string(label)
    } else {
        label.to_doc()
    }
}
