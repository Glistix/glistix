use crate::analyse::TargetSupport;
use crate::ast::{
    Arg, BinOp, CallArg, Constant, Pattern, SrcSpan, Statement, TypedArg, TypedAssignment,
    TypedClause, TypedConstant, TypedExpr, TypedModule, TypedPattern, TypedRecordUpdateArg,
    TypedStatement,
};
use crate::docvec;
use crate::line_numbers::LineNumbers;
use crate::nix::syntax::is_nix_keyword;
use crate::nix::{
    maybe_escape_identifier_doc, module_var_name_doc, pattern, syntax, Error, Output, UsageTracker,
    INDENT,
};
use crate::pretty::{break_, nil, Document, Documentable};
use crate::type_::{ModuleValueConstructor, Type, ValueConstructor, ValueConstructorVariant};
use ecow::{eco_format, EcoString};
use itertools::Itertools;
use std::borrow::Cow;
use std::sync::Arc;
use vec1::Vec1;

/// Generates a Nix expression.
pub(crate) struct Generator<'module> {
    module: &'module TypedModule,
    line_numbers: &'module LineNumbers,
    target_support: TargetSupport,
    current_scope_vars: im::HashMap<EcoString, usize>,
    // We register whether these features are used within an expression so that
    // the module generator can output a suitable function if it is needed.
    tracker: &'module mut UsageTracker,
}

impl<'module> Generator<'module> {
    pub fn new(
        module: &'module TypedModule,
        line_numbers: &'module LineNumbers,
        target_support: TargetSupport,
        current_scope_vars: im::HashMap<EcoString, usize>,
        tracker: &'module mut UsageTracker,
    ) -> Self {
        Self {
            module,
            line_numbers,
            target_support,
            current_scope_vars,
            tracker,
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
                Ok(syntax::assignment_line(name, subject))
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
            TypedExpr::Call { fun, args, .. } => self.call(fun, args, false),
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

            TypedExpr::ModuleSelect {
                module_alias,
                label,
                constructor,
                ..
            } => Ok(self.module_select(module_alias, label, constructor)),

            TypedExpr::Case {
                subjects,
                clauses,
                location,
                ..
            } => self.case(subjects, clauses, location),

            TypedExpr::BitArray { location, .. } => Err(Error::Unsupported {
                feature: "The bit array type".into(),
                location: *location,
            }),
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
            return Ok(syntax::assignment_line(nix_name, subject));
        }

        // Patterns
        Err(Error::Unsupported {
            feature: "This kind of pattern in variable declarations".into(),
            location: pattern.location(),
        })
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

        Ok(syntax::let_in(assignments, body, false))
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

        Ok(syntax::let_in(assignments, body, false))
    }

    fn variable<'a>(
        &mut self,
        name: &'a EcoString,
        constructor: &'a ValueConstructor,
    ) -> Output<'a> {
        match &constructor.variant {
            ValueConstructorVariant::LocalConstant { literal } => {
                constant_expression(self.tracker, literal)
            }
            ValueConstructorVariant::Record { .. } => {
                Ok(self.record_constructor(constructor.type_.clone(), None, name))
            }
            ValueConstructorVariant::ModuleFn { .. }
            | ValueConstructorVariant::ModuleConstant { .. }
            | ValueConstructorVariant::LocalVariable { .. } => Ok(self.local_var(name)),
        }
    }

    fn case<'a>(
        &mut self,
        subject_values: &'a [TypedExpr],
        clauses: &'a [TypedClause],
        location: &SrcSpan,
    ) -> Output<'a> {
        let (subjects, subject_assignments): (Vec<_>, Vec<_>) =
            pattern::assign_subjects(self, subject_values)
                .into_iter()
                .unzip();

        let mut doc = nil();

        if subjects.len() > 1 {
            return Err(Error::Unsupported {
                feature: "A case with multiple subjects".into(),
                location: subject_values
                    .last()
                    .map(TypedExpr::location)
                    .unwrap_or_default(),
            });
        }

        let Some(subject) = subjects.into_iter().next() else {
            return Err(Error::Unsupported {
                feature: "A case without subjects".into(),
                location: *location,
            });
        };

        let total_patterns: usize = clauses
            .iter()
            .map(|c| c.alternative_patterns.len())
            .sum::<usize>()
            + clauses.len();

        // A case has many clauses `pattern -> consequence`
        for (clause_number, clause) in clauses.iter().enumerate() {
            if !clause.alternative_patterns.is_empty() || clause.guard.is_some() {
                return Err(Error::Unsupported {
                    feature: "A clause with alternative patterns or guards".into(),
                    location: clause.location(),
                });
            }
            let pattern = &clause.pattern;
            if pattern.len() > 1 {
                return Err(Error::Unsupported {
                    feature: "A case with multiple subjects".into(),
                    location: pattern.last().map(Pattern::location).unwrap_or_default(),
                });
            }
            let Some(pattern) = pattern.first() else {
                continue;
            };
            let clause_number = clause_number + 1;
            let is_final_clause = clause_number == total_patterns;
            let is_first_clause = clause_number == 1;
            let is_only_clause = is_final_clause && is_first_clause;

            let condition = match pattern {
                Pattern::Int { value, .. } => {
                    docvec!(subject.clone(), " == ", int(value))
                }
                Pattern::Float { value, .. } => {
                    docvec!(subject.clone(), " == ", float(value))
                }
                Pattern::String { value, .. } => {
                    docvec!(subject.clone(), " == ", string(value))
                }
                Pattern::Constructor { name, type_, .. } if type_.is_bool() => {
                    if name == "True" {
                        subject.clone()
                    } else {
                        docvec!("!", subject.clone())
                    }
                }
                Pattern::Constructor { type_, .. } if type_.is_nil() => {
                    docvec!(subject.clone(), " == null")
                }
                Pattern::Constructor {
                    name,
                    arguments,
                    with_spread,
                    ..
                } if arguments.is_empty() && !with_spread => {
                    docvec!(subject.clone(), ".__gleam_tag' == ", string(name))
                }
                Pattern::Discard { .. } => "true".to_doc(),
                unsupported_pattern => {
                    return Err(Error::Unsupported {
                        feature: "This kind of pattern".into(),
                        location: unsupported_pattern.location(),
                    })
                }
            };

            let body = self.expression(&clause.then)?;

            doc = if is_only_clause {
                // If this is the only clause and there are no checks then we can
                // render just the body as the case does nothing
                doc.append(body)
            } else if is_final_clause {
                doc.append(break_("", " "))
                    .append("else")
                    .append(docvec!(break_("", " "), body).nest(INDENT).group())
            } else {
                doc.append(if is_first_clause {
                    "if".to_doc()
                } else {
                    docvec!(break_("", " "), "else if")
                })
                .append(docvec!(break_("", " "), condition).nest(INDENT).group())
                .append(docvec!(break_("", " "), "then"))
                .append(docvec!(break_("", " "), body).nest(INDENT).group())
            };
        }

        // If there is a subject name given create a variable to hold it for
        // use in patterns
        let subject_assignments: Vec<_> = subject_assignments
            .into_iter()
            .zip(subject_values)
            .flat_map(|(assignment_name, value)| assignment_name.map(|name| (name, value)))
            .map(|(name, value)| Ok(syntax::assignment_line(name, self.expression(value)?)))
            .try_collect()?;

        Ok(if subject_assignments.is_empty() {
            doc
        } else {
            syntax::let_in(subject_assignments, doc, false)
        })
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
            // Expand into binary operators (x OP y):
            | TypedExpr::BinOp { .. }
            | TypedExpr::RecordUpdate { .. }
            // Negated values are invalid in some positions, such as lists:
            | TypedExpr::NegateBool { .. }
            | TypedExpr::NegateInt { .. }
            // Expands into 'if':
            | TypedExpr::Case { .. }
            // Expand into calls:
            | TypedExpr::Todo { .. }
            | TypedExpr::Panic { .. } => Ok(docvec!["(", self.expression(expression)?, ")"]),

            TypedExpr::Call { fun, args, .. } => {
                if args.is_empty() {
                    // When the args are empty, only the function or
                    // constructor remains. Therefore, parentheses might or
                    // might be necessary, depending on which expression is
                    // used for the function or constructor. We thus delegate
                    // this job to 'self.call' by informing that it is in a
                    // child expression position.
                    self.call(fun, args, true)
                } else {
                    // There's at least one argument, so the call will
                    // certainly have spaces.
                    Ok(docvec!["(", self.expression(expression)?, ")"])
                }
            }

            TypedExpr::Var {
                constructor: ValueConstructor {
                    variant: ValueConstructorVariant::LocalConstant {
                        literal: Constant::Record { args, .. }
                    },
                    ..
                },
                ..
            } if !args.is_empty() => {
                // Expands into a call to the record's constructor.
                // The arguments aren't empty, so there will be spaces.
                // When they are indeed empty, it's just a reference to the
                // record's constructor, either by name or by 'module.name'.
                Ok(docvec!["(", self.expression(expression)?, ")"])
            },

            _ => self.expression(expression),
        }
    }
}

/// Function-related code generation.
impl Generator<'_> {
    fn call<'a>(
        &mut self,
        fun: &'a TypedExpr,
        arguments: &'a [CallArg<TypedExpr>],
        in_child_position: bool,
    ) -> Output<'a> {
        let arguments = arguments
            .iter()
            .map(|argument| self.wrap_child_expression(&argument.value))
            .try_collect()?;

        self.call_with_doc_args(fun, arguments, in_child_position)
    }

    fn call_with_doc_args<'a>(
        &mut self,
        fun: &'a TypedExpr,
        arguments: Vec<Document<'a>>,
        in_child_position: bool,
    ) -> Output<'a> {
        match fun {
            // Qualified record construction
            TypedExpr::ModuleSelect {
                constructor: ModuleValueConstructor::Record { name, .. },
                module_alias,
                ..
            } => Ok(construct_record(Some(module_alias), name, arguments)),

            // Record construction
            TypedExpr::Var {
                constructor:
                    ValueConstructor {
                        variant: ValueConstructorVariant::Record { .. },
                        type_,
                        ..
                    },
                name,
                ..
            } => {
                if type_.is_result_constructor() {
                    if name == "Ok" {
                        self.tracker.ok_used = true;
                    } else if name == "Error" {
                        self.tracker.error_used = true;
                    }
                }
                Ok(construct_record(None, name, arguments))
            }

            _ => {
                if arguments.is_empty() {
                    return if in_child_position {
                        self.wrap_child_expression(fun)
                    } else {
                        self.expression(fun)
                    };
                }
                let fun = self.wrap_child_expression(fun)?;
                Ok(syntax::fn_call(fun, arguments))
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

        // Don't break after the function call if there are no arguments.
        let argument_break = if arguments.is_empty() {
            nil()
        } else {
            break_("", " ")
        };

        Ok(docvec!(fun_args(arguments), argument_break, result?)
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
        syntax::fn_call("builtins.throw".to_doc(), [message.clone()])
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
            BinOp::RemainderInt => self.remainder_int(left, right),
            BinOp::DivInt => self.div_int(left, right),
            BinOp::DivFloat => self.div_float(left, right),
        }
    }

    fn div_int<'a>(&mut self, left: &'a TypedExpr, right: &'a TypedExpr) -> Output<'a> {
        let left = self.wrap_child_expression(left)?;
        let right = self.wrap_child_expression(right)?;
        self.tracker.int_division_used = true;
        // This name can't be shadowed, as user variables must be in lowercase
        // or (for types) PascalCase.
        Ok(syntax::fn_call("divideInt".to_doc(), [left, right]))
    }

    fn remainder_int<'a>(&mut self, left: &'a TypedExpr, right: &'a TypedExpr) -> Output<'a> {
        let left = self.wrap_child_expression(left)?;
        let right = self.wrap_child_expression(right)?;
        self.tracker.int_remainder_used = true;
        Ok(syntax::fn_call("remainderInt".to_doc(), [left, right]))
    }

    fn div_float<'a>(&mut self, left: &'a TypedExpr, right: &'a TypedExpr) -> Output<'a> {
        let left = self.wrap_child_expression(left)?;
        let right = self.wrap_child_expression(right)?;
        self.tracker.float_division_used = true;
        Ok(syntax::fn_call("divideFloat".to_doc(), [left, right]))
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
        // Nix's equality is always structural.
        return self.print_bin_op(left, right, if should_be_equal { "==" } else { "!=" });
        // if is_nix_scalar(left.type_()) {
        // }
        // Other types must be compared using structural equality
        // Ok(self.prelude_equal_call(should_be_equal, left, right))
    }

    fn negate_with<'a>(&mut self, with: &'static str, value: &'a TypedExpr) -> Output<'a> {
        Ok(docvec!(with, self.wrap_child_expression(value)?))
    }
}

/// Record-related methods.
impl Generator<'_> {
    fn tuple<'a>(&mut self, elements: &'a [TypedExpr]) -> Output<'a> {
        tuple(elements.iter().map(|element| self.expression(element)))
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
        let set = syntax::try_wrap_attr_set(fields)?;
        Ok(docvec![record, " // ", set])
    }

    fn record_constructor<'a>(
        &mut self,
        type_: Arc<Type>,
        qualifier: Option<&'a str>,
        name: &'a str,
    ) -> Document<'a> {
        if qualifier.is_none() && type_.is_result_constructor() {
            if name == "Ok" {
                self.tracker.ok_used = true;
            } else if name == "Error" {
                self.tracker.error_used = true;
            }
        }
        if type_.is_bool() && name == "True" {
            "true".to_doc()
        } else if type_.is_bool() {
            "false".to_doc()
        } else if type_.is_nil() {
            "null".to_doc()
        } else {
            // Use the record constructor directly.
            match qualifier {
                Some(module) => docvec![module_var_name_doc(module), ".", name],
                None => name.to_doc(),
            }
        }
    }

    fn module_select<'a>(
        &mut self,
        module: &'a str,
        label: &'a str,
        constructor: &'a ModuleValueConstructor,
    ) -> Document<'a> {
        match constructor {
            ModuleValueConstructor::Fn { .. } | ModuleValueConstructor::Constant { .. } => {
                docvec!(
                    module_var_name_doc(module),
                    ".",
                    maybe_escape_identifier_doc(label)
                )
            }

            ModuleValueConstructor::Record { name, type_, .. } => {
                self.record_constructor(type_.clone(), Some(module), name)
            }
        }
    }
}

/// Types which are trivially comparable for equality.
pub fn is_nix_scalar(t: Arc<Type>) -> bool {
    t.is_int() || t.is_float() || t.is_bool() || t.is_nil() || t.is_string()
}

pub(crate) fn constant_expression<'a>(
    tracker: &mut UsageTracker,
    expression: &'a TypedConstant,
) -> Output<'a> {
    match expression {
        Constant::Int { value, .. } => Ok(int(value)),
        Constant::Float { value, .. } => Ok(float(value)),
        Constant::String { value, .. } => Ok(string(value)),
        Constant::Tuple { elements, .. } => {
            tuple(elements.iter().map(|e| constant_expression(tracker, e)))
        }

        Constant::List { elements, .. } => {
            list(elements.iter().map(|e| constant_expression(tracker, e)))
        }

        Constant::Record { typ, name, .. } if typ.is_bool() && name == "True" => {
            Ok("true".to_doc())
        }
        Constant::Record { typ, name, .. } if typ.is_bool() && name == "False" => {
            Ok("false".to_doc())
        }
        Constant::Record { typ, .. } if typ.is_nil() => Ok("null".to_doc()),

        Constant::Record {
            tag,
            typ,
            args,
            module,
            ..
        } => {
            if typ.is_result() {
                if tag == "Ok" {
                    tracker.ok_used = true;
                } else {
                    tracker.error_used = true;
                }
            }
            let field_values = args
                .iter()
                .map(|arg| constant_expression(tracker, &arg.value))
                .try_collect()?;

            Ok(construct_record(module.as_deref(), tag, field_values))
        }

        Constant::BitArray {
            segments: _,
            location,
            ..
        } => Err(Error::Unsupported {
            feature: "The bit array type".into(),
            location: *location,
        }),

        Constant::Var { name, module, .. } => Ok({
            match module {
                None => name.to_doc(),
                Some(module) => docvec![module_var_name_doc(module), ".", name],
            }
        }),
    }
}

/// A record in Nix is represented with the following format:
///
/// ```nix
/// { __gleam_tag' = "Ctor", field_name = value, field2_name = value, ... }
/// ```
fn construct_record<'a>(
    module: Option<&'a str>,
    name: &'a str,
    arguments: Vec<Document<'a>>,
) -> Document<'a> {
    let name = if let Some(module) = module {
        docvec![module_var_name_doc(module), ".", name]
    } else {
        name.to_doc()
    };

    syntax::fn_call(name, arguments)
}

/// Generates a valid Nix string.
pub fn string(value: &str) -> Document<'_> {
    match syntax::sanitize_string(value) {
        Cow::Owned(string) => Document::String(string),
        Cow::Borrowed(value) => value.to_doc(),
    }
    .surround("\"", "\"")
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

pub fn tuple<'a>(elements: impl IntoIterator<Item = Output<'a>>) -> Output<'a> {
    let fields = elements
        .into_iter()
        .enumerate()
        .map(|(i, element)| (Document::String(format!("_{i}")), element));

    syntax::try_wrap_attr_set(fields)
}

pub fn fun_args(args: &'_ [TypedArg]) -> Document<'_> {
    let mut discards = 0;
    syntax::wrap_args(args.iter().map(|a| match a.get_variable_name() {
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
