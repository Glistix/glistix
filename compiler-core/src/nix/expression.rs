use crate::analyse::TargetSupport;
use crate::ast::{
    Arg, BinOp, CallArg, Constant, SrcSpan, Statement, TypedArg, TypedAssignment, TypedClause,
    TypedConstant, TypedExpr, TypedModule, TypedPattern, TypedRecordUpdateArg, TypedStatement,
};
use crate::docvec;
use crate::line_numbers::LineNumbers;
use crate::nix::syntax::is_nix_keyword;
use crate::nix::{
    maybe_escape_identifier_doc, module_var_name_doc, pattern, syntax, Error, Output, UsageTracker,
    INDENT,
};
use crate::pretty::{break_, join, nil, Document, Documentable};
use crate::type_::{ModuleValueConstructor, Type, ValueConstructor, ValueConstructorVariant};
use ecow::{eco_format, EcoString};
use itertools::Itertools;
use std::borrow::Cow;
use std::sync::{Arc, OnceLock};
use vec1::Vec1;

/// Generates a Nix expression.
#[derive(Debug)]
pub(crate) struct Generator<'module> {
    module: &'module TypedModule,
    line_numbers: &'module LineNumbers,
    target_support: TargetSupport,
    current_scope_vars: im::HashMap<EcoString, usize>,
    // We register whether these features are used within an expression so that
    // the module generator can output a suitable function if it is needed.
    pub(crate) tracker: &'module mut UsageTracker,
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
    ///
    /// If there is no assignment to be printed, returns `Ok(None)`. This can only happen
    /// when using `let` with a pattern with no assignments. Then, nothing effectively happens,
    /// due to the fact that Nix is lazily evaluated, so a variable that isn't used or assigned
    /// is never evaluated. See [`Generator::assignment`] for more information.
    ///
    /// In all other cases, returns either `Ok(Some(...))` or `Error(...)`.
    fn statement<'a>(
        &mut self,
        statement: &'a TypedStatement,
    ) -> Result<Option<Document<'a>>, Error> {
        match statement {
            Statement::Expression(expression) => {
                let subject = self.expression(expression)?;
                let name = self.next_anonymous_var();
                // Convert expression to assignment with irrelevant name
                Ok(Some(syntax::assignment_line(name, subject)))
            }
            Statement::Assignment(assignment) => self.assignment(assignment, false),
            Statement::Use(_use) => {
                unreachable!("Use must not be present for Nix generation")
            }
        }
    }

    pub fn expression<'a>(&mut self, expression: &'a TypedExpr) -> Output<'a> {
        match expression {
            TypedExpr::String { value, .. } => Ok(string(value)),
            TypedExpr::Int { value, .. } => Ok(int(value, self.tracker)),
            TypedExpr::Float { value, .. } => Ok(float(value)),
            TypedExpr::List { elements, tail, .. } => match tail {
                Some(tail) => {
                    // A tail without prepended elements is a syntax error.
                    // Therefore, we can assume we will have to use prepend here.
                    self.tracker.prepend_used = true;
                    let tail = self.wrap_child_expression(tail)?;
                    prepend(elements.iter().map(|e| self.wrap_child_expression(e)), tail)
                }
                None => {
                    self.tracker.list_used = true;
                    list(elements.iter().map(|e| self.wrap_child_expression(e)))
                }
            },

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
                subjects, clauses, ..
            } => self.case(subjects, clauses),

            TypedExpr::BitArray { location, .. } => Err(Error::Unsupported {
                feature: "The bit array type".into(),
                location: *location,
            }),
        }
    }

    /// Renders an assignment, or just the value being assigned if we're in trailing position
    /// (that is, this assignment is the last statement in a function or block, so it is returned).
    ///
    /// Returns `Ok(None)` if no assignment should be printed. That never occurs within trailing
    /// position.
    ///
    /// ```nix
    /// # Before trailing position
    /// name = value;  # without assert
    /// # with assert:
    /// pat' = value;  # assuming the value is complex so we assign it to a var, for example
    /// assert' = if pat'.__gleam_tag' != "Ok" then throw "..." else null;
    /// name = builtins.seq assert' pat'; # only return pat' if assertion succeeds
    ///
    /// # In trailing position
    /// value  # without assert
    /// if (checks) then value else throw  # with assert
    /// ```
    pub fn assignment<'a>(
        &mut self,
        assignment: &'a TypedAssignment,
        in_trailing_position: bool,
    ) -> Result<Option<Document<'a>>, Error> {
        static ASSERTION_VAR_ECO_STR: OnceLock<EcoString> = OnceLock::new();

        let TypedAssignment {
            pattern,
            kind,
            value,
            ..
        } = assignment;

        // If it is a simple assignment to a variable we can generate a normal
        // Nix assignment
        if let TypedPattern::Variable { name, .. } = pattern {
            // Subject must be rendered before the variable for variable numbering
            let subject = self.expression(value)?;
            if in_trailing_position {
                // No need to assign, we are being returned.
                return Ok(Some(subject));
            }
            let nix_name = self.next_local_var(name);
            return Ok(Some(syntax::assignment_line(nix_name, subject)));
        }

        // Otherwise we need to compile the patterns
        let (subject, subject_assignment) = pattern::assign_subject(self, value);
        // Value needs to be rendered before traversing pattern to have correctly incremented variables.
        let value = self.wrap_child_expression(value)?;
        let mut pattern_generator = pattern::Generator::new(self);
        pattern_generator.traverse_pattern(&subject, pattern)?;
        let compiled = pattern_generator.take_compiled();
        let has_assertion = kind.is_assert() && !compiled.checks.is_empty();
        let pattern_location = pattern.location();

        if in_trailing_position {
            // Note that, even though the variables used within trailing position are separate
            // from those in the outer block or function (which would normally warrant a scope
            // reset to indicate that those variables are now out of scope), we can assume that
            // the block or function is about to end, and thus the scope is about to be reset,
            // since we are at the trailing statement position. Therefore, there is no problem
            // in not resetting the scope here, even though we are using a new variable (the
            // subject).
            if !has_assertion {
                // No assertions, so we don't use the subject (it would be used in the checks).
                // Just return the value directly.
                return Ok(Some(value));
            }

            // No need to add any assignments when we are in trailing position (i.e. the 'let'
            // assignment is the last statement in the parent block or function).
            // Simply return the subject (with the value assigned to it) if the check succeeds.
            let checked_subject = self.pattern_checks_or_throw_doc(
                compiled.checks,
                subject.clone(),
                subject,
                pattern_location,
            );

            // If the value being assigned is complex and needs a subject variable,
            // assign it so it can be used within the check.
            Ok(Some(match subject_assignment {
                Some(name) => syntax::let_in(
                    [syntax::assignment_line(name, value)],
                    checked_subject,
                    false,
                ),
                None => checked_subject,
            }))
        } else if compiled.assignments.is_empty() {
            // No assignments, so don't print anything, not even the value at right-hand side.
            // Since Nix is lazily-evaluated, the value being assigned is effectively ignored.
            // It would have to be used through some variable to have any effect.
            // TODO: Consider adding a "strict assertion" mode.
            Ok(None)
        } else {
            // If the value being assigned is complex and needs a subject variable,
            // assign it so it can be used within patterns.
            let subject_assignment = match subject_assignment {
                Some(name) => syntax::assignment_line(name, value).append(break_("", " ")),
                None => nil(),
            };

            Ok(Some(if has_assertion {
                // We first assign a dummy value to a variable whose only purpose is performing
                // an assertion. The idea is that, if the assertion fails, the variable will
                // throw an error upon evaluation instead of returning the dummy value.
                let assertion_var_name =
                    ASSERTION_VAR_ECO_STR.get_or_init(|| pattern::ASSERTION_VAR.into());
                let assertion_var = self.next_local_var(assertion_var_name);
                let assertion_assignment = syntax::assignment_line(
                    assertion_var.clone(),
                    self.pattern_checks_or_throw_doc(
                        compiled.checks,
                        subject,
                        "null".to_doc(),
                        pattern_location,
                    ),
                )
                .append(break_("", " "));

                // Then, we ensure that evaluating any of the assignments first evaluates the
                // assertion variable through `builtins.seq assertion_var assigned_value`.
                // Therefore, accessing any of the assignments later will cause an error
                // if the checks failed. If they aren't accessed, no error occurs, since
                // Nix is lazily evaluated.
                // TODO: Strict assertions mode (always check, even without assignments etc.)
                let assignments = join(
                    compiled.assignments.into_iter().map(|assignment| {
                        assignment.into_doc_with_assertion(assertion_var.clone())
                    }),
                    break_("", " "),
                );

                // Finally, we place the assignments.
                // Subject goes first so checks can be done on the subject.
                // Then we generate the assertion which will evaluate either
                // to an error (if checks fail) or to some dummy value.
                // Finally, we generate assignments which will first evaluate
                // the assertion before completing the assignments for the
                // given pattern.
                //
                // For example, for `let assert Ok(x) = something(1)`, we'd generate
                //
                // ```nix
                // _pat' = something 1;
                // _assert' = if _pat'.__gleam_tag' != "Ok" then throw "..." else null;
                // x = builtins.seq _assert' _pat'._0; # access the field after the assertion
                // ```
                docvec![subject_assignment, assertion_assignment, assignments]
            } else {
                // No assertions, so the type system tells us the pattern is exhaustive.
                // We can perform the assignments directly.
                let assignments = join(
                    compiled
                        .assignments
                        .into_iter()
                        .map(pattern::Assignment::into_doc),
                    break_("", " "),
                );

                docvec![subject_assignment, assignments]
            }))
        }
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
            .flat_map(|statement| self.statement(statement).transpose())
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
            .flat_map(|assignment| self.assignment(assignment, false).transpose())
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
    ) -> Output<'a> {
        let (subjects, subject_assignments): (Vec<_>, Vec<_>) =
            pattern::assign_subjects(self, subject_values)
                .into_iter()
                .unzip();

        let mut gen = pattern::Generator::new(self);

        let mut doc = nil();

        // We wish to be able to know whether this is the first or clause being
        // processed, so record the index number. We use this instead of
        // `Iterator.enumerate` because we are using a nested for loop.
        let mut clause_number = 0;
        let total_patterns: usize = clauses
            .iter()
            .map(|c| c.alternative_patterns.len())
            .sum::<usize>()
            + clauses.len();

        // A case has many clauses `pattern -> consequence`
        for clause in clauses {
            let multipattern = std::iter::once(&clause.pattern);
            let multipatterns = multipattern.chain(&clause.alternative_patterns);

            // A clause can have many patterns `pattern, pattern ->...`
            for multipatterns in multipatterns {
                let scope = gen.expression_generator.current_scope_vars.clone();
                let mut compiled = gen.generate(&subjects, multipatterns, clause.guard.as_ref())?;
                let consequence = gen.expression_generator.expression(&clause.then)?;

                // We've seen one more clause
                clause_number += 1;

                // Reset the scope now that this clause has finished, causing the
                // variables to go out of scope.
                gen.expression_generator.current_scope_vars = scope;

                // If the pattern assigns any variables we need to render assignments
                let body = if compiled.has_assignments() {
                    let assignments = std::mem::take(&mut compiled.assignments)
                        .into_iter()
                        .map(pattern::Assignment::into_doc);

                    syntax::let_in(assignments, consequence, false)
                } else {
                    consequence
                };

                let is_final_clause = clause_number == total_patterns;
                let is_first_clause = clause_number == 1;
                let is_only_clause = is_final_clause && is_first_clause;

                doc = if is_only_clause {
                    // If this is the only clause and there are no checks then we can
                    // render just the body as the case does nothing
                    doc.append(body)
                } else if is_final_clause {
                    doc.append(break_("", " "))
                        .append("else")
                        .append(docvec!(break_("", " "), body).nest(INDENT).group())
                } else {
                    let condition = gen
                        .expression_generator
                        .pattern_take_checks_doc(&mut compiled, true);

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

            Statement::Assignment(assignment) => {
                // Trailing position assignment must always be kept.
                // It is evaluated to the right-hand side of the assignment,
                // optionally with some checks if `let assert` was used.
                // Hence, we unwrap.
                self.assignment(assignment, true).map(Option::unwrap)
            }

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

            // Integers with 0x, 0o, 0b require a function call to 'parseNumber'
            TypedExpr::Int { value, .. } if int_requires_parsing(value) => {
                Ok(docvec!["(", self.expression(expression)?, ")"])
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
            | TypedExpr::List { .. }
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

        if arguments.is_empty() {
            // A function without args only has its body.
            return result;
        }

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

/// Methods related to patterns.
impl Generator<'_> {
    fn pattern_take_checks_doc<'a>(
        &mut self,
        compiled_pattern: &mut pattern::CompiledPattern<'a>,
        match_desired: bool,
    ) -> Document<'a> {
        let checks = std::mem::take(&mut compiled_pattern.checks);
        self.pattern_checks_doc(checks, match_desired)
    }

    fn pattern_checks_doc<'a>(
        &mut self,
        checks: Vec<pattern::Check<'a>>,
        match_desired: bool,
    ) -> Document<'a> {
        if checks.is_empty() {
            return "true".to_doc();
        };
        let operator = if match_desired {
            break_(" &&", " && ")
        } else {
            break_(" ||", " || ")
        };

        let checks_len = checks.len();
        join(
            checks.into_iter().map(|check| {
                if checks_len > 1 && check.may_require_wrapping() {
                    docvec!["(", check.into_doc(self.tracker, match_desired), ")"]
                } else {
                    check.into_doc(self.tracker, match_desired)
                }
            }),
            operator,
        )
        .group()
    }

    /// Given the compiled pattern checks, if the checks fail, throws an error.
    /// Otherwise, returns the given value.
    fn pattern_checks_or_throw_doc<'a>(
        &mut self,
        checks: Vec<pattern::Check<'a>>,
        subject: Document<'a>,
        success_value: Document<'a>,
        location: SrcSpan,
    ) -> Document<'a> {
        let checks = self.pattern_checks_doc(checks, false);
        docvec![
            "if",
            docvec![break_("", " "), checks].nest(INDENT).group(),
            break_("", " "),
            "then",
            docvec![break_("", " "), self.assignment_no_match(location, subject)]
                .nest(INDENT)
                .group(),
            break_("", " "),
            "else",
            docvec![break_("", " "), success_value].nest(INDENT).group(),
        ]
        .group()
    }

    fn assignment_no_match<'a>(
        &mut self,
        location: SrcSpan,
        subject: Document<'a>,
    ) -> Document<'a> {
        self.throw_error(
            "assignment_no_match",
            &string("Assignment pattern did not match"),
            location,
            [("value", subject)],
        )
    }
}

pub(crate) fn guard_constant_expression<'a>(
    assignments: &mut Vec<pattern::Assignment<'a>>,
    tracker: &mut UsageTracker,
    expression: &'a TypedConstant,
) -> Output<'a> {
    match expression {
        Constant::Tuple { elements, .. } => tuple(
            elements
                .iter()
                .map(|e| guard_constant_expression(assignments, tracker, e)),
        ),

        Constant::List { elements, .. } => {
            tracker.list_used = true;
            list(
                elements
                    .iter()
                    .map(|e| wrap_child_guard_constant_expression(assignments, tracker, e)),
            )
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
            let field_values: Vec<_> = args
                .iter()
                .map(|arg| wrap_child_guard_constant_expression(assignments, tracker, &arg.value))
                .try_collect()?;
            Ok(construct_record(module.as_deref(), tag, field_values))
        }

        Constant::Var { name, .. } => Ok(assignments
            .iter()
            .find(|assignment| assignment.name == name)
            .map(|assignment| {
                assignment
                    .path
                    .clone()
                    .into_doc_with_subject(assignment.subject.clone())
            })
            .unwrap_or_else(|| name.to_doc())),
        expression => constant_expression(tracker, expression),
    }
}

pub(crate) fn constant_expression<'a>(
    tracker: &mut UsageTracker,
    expression: &'a TypedConstant,
) -> Output<'a> {
    match expression {
        Constant::Int { value, .. } => Ok(int(value, tracker)),
        Constant::Float { value, .. } => Ok(float(value)),
        Constant::String { value, .. } => Ok(string(value)),
        Constant::Tuple { elements, .. } => {
            tuple(elements.iter().map(|e| constant_expression(tracker, e)))
        }

        Constant::List { elements, .. } => {
            tracker.list_used = true;
            list(
                elements
                    .iter()
                    .map(|e| wrap_child_constant_expression(tracker, e)),
            )
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
                .map(|arg| wrap_child_constant_expression(tracker, &arg.value))
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

/// Same as [`constant_expression`], but wraps the result in parentheses if needed.
fn wrap_child_constant_expression<'a>(
    tracker: &mut UsageTracker,
    expression: &'a TypedConstant,
) -> Output<'a> {
    match expression {
        Constant::Int { value, .. } if int_requires_parsing(value) => {
            // Will call 'parseNumber'
            Ok(docvec!("(", constant_expression(tracker, expression)?, ")"))
        }
        Constant::List { .. } => Ok(docvec!("(", constant_expression(tracker, expression)?, ")")),
        Constant::Record { args, .. } if !args.is_empty() => {
            Ok(docvec!("(", constant_expression(tracker, expression)?, ")"))
        }
        _ => constant_expression(tracker, expression),
    }
}

/// Same as [`guard_constant_expression`], but wraps the result in parentheses if needed.
fn wrap_child_guard_constant_expression<'a>(
    assignments: &mut Vec<pattern::Assignment<'a>>,
    tracker: &mut UsageTracker,
    expression: &'a TypedConstant,
) -> Output<'a> {
    match expression {
        Constant::Int { value, .. } if int_requires_parsing(value) => {
            // Will call 'parseNumber'
            Ok(docvec!(
                "(",
                guard_constant_expression(assignments, tracker, expression)?,
                ")"
            ))
        }
        Constant::List { .. } => Ok(docvec!(
            "(",
            guard_constant_expression(assignments, tracker, expression)?,
            ")"
        )),
        Constant::Record { args, .. } if !args.is_empty() => Ok(docvec!(
            "(",
            guard_constant_expression(assignments, tracker, expression)?,
            ")"
        )),
        _ => guard_constant_expression(assignments, tracker, expression),
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
///
/// An integer may use binary, octal or hexadecimal notation, in which case
/// importing `parseNumber` from the prelude is necessary. In those cases,
/// we will generate e.g. `parseNumber "0xff"`.
pub fn int<'a>(value: &str, tracker: &mut UsageTracker) -> Document<'a> {
    if int_requires_parsing(value) {
        tracker.parse_number_used = true;

        // Remove leading '+'
        let value = value.trim_start_matches('+');

        // 'parseNumber' does support '_' separators! They can be kept.
        let out = EcoString::from(value);

        return syntax::fn_call("parseNumber".to_doc(), [docvec!["\"", out, "\""]]);
    }

    let mut out = EcoString::with_capacity(value.len());

    if value.starts_with('-') {
        out.push('-');
    }

    // Ignore '+' at the beginning (no Nix support)
    let value = value.trim_start_matches(['+', '-'].as_ref());

    let value = value.trim_start_matches('0');
    if value.is_empty() {
        out.push('0');
    }

    out.push_str(value);

    // Remove '_' separators (no Nix support)
    let out = out.replace("_", "");

    out.to_doc()
}

/// Nix doesn't support hexadecimal, octal or binary integers by default.
/// Therefore, those will require a call to a parsing function in the prelude.
pub(super) fn int_requires_parsing(value: &str) -> bool {
    let value = value.trim_start_matches(['+', '-']);
    value.starts_with("0x") || value.starts_with("0o") || value.starts_with("0b")
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

    // Remove '_' separators (no Nix support)
    let out = out.replace("_", "");

    out.to_doc()
}

/// Constructs a Gleam List:
///
/// ```nix
/// toList [ elem1 elem2 elem3 ... elemN ]
/// ```
pub fn list<'a, Elements: IntoIterator<Item = Output<'a>>>(elements: Elements) -> Output<'a> {
    let elements: Vec<_> = elements.into_iter().try_collect()?;
    let element_list = syntax::list(elements);

    Ok(syntax::fn_call("toList".to_doc(), [element_list]))
}

/// Prepends elements before an existing list:
///
/// ```nix
/// listPrepend elem1 (listPrepend elem2 (... (listPrepend elemN tail) ...))
/// ```
fn prepend<'a, I: IntoIterator<Item = Output<'a>>>(elements: I, tail: Document<'a>) -> Output<'a>
where
    I::IntoIter: DoubleEndedIterator + ExactSizeIterator,
{
    elements.into_iter().rev().try_fold(tail, |tail, element| {
        Ok(syntax::fn_call("listPrepend".to_doc(), [element?, tail]))
    })
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
