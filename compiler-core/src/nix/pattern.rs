use crate::analyse::Inferred;
use ecow::EcoString;
use itertools::Itertools;
use std::sync::OnceLock;

use crate::ast::{AssignName, ClauseGuard, Pattern, TypedClauseGuard, TypedExpr, TypedPattern};
use crate::docvec;
use crate::nix::{
    expression, maybe_escape_identifier_doc, module_var_name_doc, syntax, Error, Output,
    UsageTracker,
};
use crate::pretty::{nil, Document, Documentable};
use crate::type_::{FieldMap, PatternConstructor};

pub static ASSIGNMENT_VAR: &str = "_pat'";

#[derive(Debug)]
enum Index<'a> {
    Int(usize),
    Tuple(usize),
    String(&'a str),
    ByteAt(usize),
    IntFromSlice(usize, usize),
    FloatAt(usize),
    BinaryFromSlice(usize, usize),
    SliceAfter(usize),
    StringPrefixSlice(usize),
}

#[derive(Debug)]
pub struct Subjects<'a> {
    pub values: Vec<Document<'a>>,
    pub assignments: Vec<(Document<'a>, Document<'a>)>,
}

/// Compiles clauses with patterns into individual checks.
#[derive(Debug)]
pub(crate) struct Generator<'module_ctx, 'expression_gen, 'a> {
    pub expression_generator: &'expression_gen mut expression::Generator<'module_ctx>,
    /// The current transformations to the subject being tested.
    /// That is, oftentimes there is a pattern which verifies what a subject's
    /// field matches, and not the subject itself. This indicates which fields
    /// we need to take from the target to reach what we actually want to
    /// compare from the subject.
    path: Vec<Index<'a>>,
    /// The compiled checks. For example, an equality between the subject
    /// (or a path within it) and some value. Each clause will have at least one
    /// associated check.
    checks: Vec<Check<'a>>,
    assignments: Vec<Assignment<'a>>,
}

struct Offset {
    bytes: usize,
    open_ended: bool,
}

impl Offset {
    pub fn new() -> Self {
        Self {
            bytes: 0,
            open_ended: false,
        }
    }
    // This should never be called on an open ended offset
    // However previous checks ensure bit_array segments without a size are only
    // allowed at the end of a pattern
    pub fn increment(&mut self, step: usize) {
        self.bytes += step
    }
    pub fn set_open_ended(&mut self) {
        self.open_ended = true
    }
}

impl<'module_ctx, 'expression_gen, 'a> Generator<'module_ctx, 'expression_gen, 'a> {
    pub fn new(
        expression_generator: &'expression_gen mut expression::Generator<'module_ctx>,
    ) -> Self {
        Self {
            path: vec![],
            checks: vec![],
            assignments: vec![],
            expression_generator,
        }
    }

    fn next_local_var(&mut self, name: &'a EcoString) -> Document<'a> {
        self.expression_generator.next_local_var(name, false)
    }

    fn local_var(&mut self, name: &'a EcoString) -> Document<'a> {
        self.expression_generator.local_var(name)
    }

    /// For this pattern, access a certain field within the target.
    fn push_string(&mut self, s: &'a str) {
        self.path.push(Index::String(s));
    }

    /// For this pattern, access a record's numbered field.
    fn push_int(&mut self, i: usize) {
        self.path.push(Index::Int(i));
    }

    /// For this pattern, access a tuple index field.
    fn push_tuple_index(&mut self, i: usize) {
        self.path.push(Index::Tuple(i));
    }

    fn push_string_prefix_slice(&mut self, i: usize) {
        self.path.push(Index::StringPrefixSlice(i));
    }

    fn push_byte_at(&mut self, i: usize) {
        self.path.push(Index::ByteAt(i));
    }

    fn push_int_from_slice(&mut self, start: usize, end: usize) {
        self.path.push(Index::IntFromSlice(start, end));
    }

    fn push_float_at(&mut self, i: usize) {
        self.path.push(Index::FloatAt(i));
    }

    fn push_binary_from_slice(&mut self, start: usize, end: usize) {
        self.path.push(Index::BinaryFromSlice(start, end));
    }

    fn push_rest_from(&mut self, i: usize) {
        self.path.push(Index::SliceAfter(i));
    }

    /// Access a certain field within the subject several times deep.
    fn push_string_times(&mut self, s: &'a str, times: usize) {
        for _ in 0..times {
            self.push_string(s);
        }
    }

    /// Remove the latest access in the path.
    fn pop_segment(&mut self) {
        let _ = self.path.pop();
    }

    fn pop_times(&mut self, times: usize) {
        for _ in 0..times {
            self.pop_segment();
        }
    }

    fn finish_path(&self) -> SubjectPath<'a> {
        let mut path = SubjectPath::new();

        for segment in &self.path {
            match segment {
                Index::Int(i) => {
                    path = path.add_right_component(Document::String(format!("._{i}")))
                }
                Index::Tuple(i) => {
                    path = path
                        .add_component("(builtins.elemAt ".to_doc(), docvec!(" ", i.to_doc(), ")"))
                }
                Index::String(s) => {
                    path = path.add_right_component(docvec!(".", maybe_escape_identifier_doc(s)))
                }
                Index::ByteAt(_i) => todo!("bitarray"),
                Index::IntFromSlice(_start, _end) => todo!("bitarray"),
                Index::FloatAt(_i) => todo!("bitarray"),
                Index::BinaryFromSlice(_start, _end) => todo!("bitarray"),
                Index::SliceAfter(_i) => todo!("bitarray"),
                Index::StringPrefixSlice(i) => {
                    path = path
                        .add_component(docvec!("(builtins.substring ", i, " (-1) "), ")".to_doc())
                }
            }
        }

        path
    }

    /// Compile one of the patterns for a single clause, producing
    /// the necessary assignments, checks and indicating the guard.
    /// The `expression` module is then responsible for converting the compiled
    /// patterns into valid Nix syntax, by performing the assignments,
    /// then `if (guard && check1 && check2 && ...)`, and finally the body.
    ///
    /// For example, with the code
    ///
    /// ```rs
    /// case x, y {
    ///   Ok(x), Ok(y) | Error(x), Error(y) -> x + y
    ///   Ok(x), Error(y) if x == y -> x - y
    ///   _, _ -> 0
    /// }
    /// ```
    ///
    /// There are two subjects, `x` and `y`, so each clause will have two
    /// patterns to check at a time.
    /// For the first clause, this function is run twice: once for
    /// `Ok(x), Ok(y)`, and once for `Error(x), Error(y)`, both with
    /// two subjects and patterns, but no guard.
    /// For the second clause, this function is run once (for
    /// `Ok(x), Error(y)`), with two subjects, two patterns, and a guard.
    /// For the third clause (with just `_, _`), this function is run once,
    /// with two subjects, two (identical) patterns and no guard.
    pub fn generate(
        &mut self,
        subjects: &[Document<'a>],
        patterns: &'a [TypedPattern],
        guard: Option<&'a TypedClauseGuard>,
    ) -> Result<CompiledPattern<'a>, Error> {
        for (subject, pattern) in subjects.iter().zip_eq(patterns) {
            self.traverse_pattern(subject, pattern)?;
        }
        if let Some(guard) = guard {
            self.push_guard_check(guard)?;
        }

        Ok(self.take_compiled())
    }

    pub fn take_compiled(&mut self) -> CompiledPattern<'a> {
        CompiledPattern {
            checks: std::mem::take(&mut self.checks),
            assignments: std::mem::take(&mut self.assignments),
        }
    }

    fn push_guard_check(&mut self, guard: &'a TypedClauseGuard) -> Result<(), Error> {
        let expression = self.guard(guard)?;
        self.checks.push(Check::Guard { expression });
        Ok(())
    }

    fn wrapped_guard(&mut self, guard: &'a TypedClauseGuard) -> Output<'a> {
        match guard {
            // Some constants need to be wrapped, e.g. record construction: `(Ok 1)`
            ClauseGuard::Constant(constant) => expression::wrap_child_guard_constant_expression(
                &mut self.assignments,
                self.expression_generator.tracker,
                constant,
            ),

            ClauseGuard::Var { .. }
            | ClauseGuard::Not { .. }
            | ClauseGuard::FieldAccess { .. }
            | ClauseGuard::ModuleSelect { .. } => self.guard(guard),

            ClauseGuard::Equals { .. }
            | ClauseGuard::NotEquals { .. }
            | ClauseGuard::GtInt { .. }
            | ClauseGuard::GtEqInt { .. }
            | ClauseGuard::LtInt { .. }
            | ClauseGuard::LtEqInt { .. }
            | ClauseGuard::GtFloat { .. }
            | ClauseGuard::GtEqFloat { .. }
            | ClauseGuard::LtFloat { .. }
            | ClauseGuard::LtEqFloat { .. }
            | ClauseGuard::Or { .. }
            | ClauseGuard::And { .. }
            | ClauseGuard::TupleIndex { .. } => Ok(docvec!("(", self.guard(guard)?, ")")),
        }
    }

    fn guard(&mut self, guard: &'a TypedClauseGuard) -> Output<'a> {
        Ok(match guard {
            ClauseGuard::Equals { left, right, .. } => {
                let left = self.wrapped_guard(left)?;
                let right = self.wrapped_guard(right)?;
                docvec!(left, " == ", right)
            }

            ClauseGuard::NotEquals { left, right, .. } => {
                let left = self.wrapped_guard(left)?;
                let right = self.wrapped_guard(right)?;
                docvec!(left, " != ", right)
            }

            ClauseGuard::GtFloat { left, right, .. } | ClauseGuard::GtInt { left, right, .. } => {
                let left = self.wrapped_guard(left)?;
                let right = self.wrapped_guard(right)?;
                docvec!(left, " > ", right)
            }

            ClauseGuard::GtEqFloat { left, right, .. }
            | ClauseGuard::GtEqInt { left, right, .. } => {
                let left = self.wrapped_guard(left)?;
                let right = self.wrapped_guard(right)?;
                docvec!(left, " >= ", right)
            }

            ClauseGuard::LtFloat { left, right, .. } | ClauseGuard::LtInt { left, right, .. } => {
                let left = self.wrapped_guard(left)?;
                let right = self.wrapped_guard(right)?;
                docvec!(left, " < ", right)
            }

            ClauseGuard::LtEqFloat { left, right, .. }
            | ClauseGuard::LtEqInt { left, right, .. } => {
                let left = self.wrapped_guard(left)?;
                let right = self.wrapped_guard(right)?;
                docvec!(left, " <= ", right)
            }

            ClauseGuard::Or { left, right, .. } => {
                let left = self.wrapped_guard(left)?;
                let right = self.wrapped_guard(right)?;
                docvec!(left, " || ", right)
            }

            ClauseGuard::And { left, right, .. } => {
                let left = self.wrapped_guard(left)?;
                let right = self.wrapped_guard(right)?;
                docvec!(left, " && ", right)
            }

            ClauseGuard::Var { name, .. } => self
                .path_doc_from_assignments(name)
                .unwrap_or_else(|| self.local_var(name)),

            ClauseGuard::TupleIndex { tuple, index, .. } => {
                let tuple = self.wrapped_guard(tuple)?;

                syntax::fn_call("builtins.elemAt".to_doc(), [tuple, index.to_doc()])
            }

            ClauseGuard::FieldAccess {
                label, container, ..
            } => {
                docvec!(
                    self.wrapped_guard(container)?,
                    ".",
                    maybe_escape_identifier_doc(label)
                )
            }

            ClauseGuard::ModuleSelect {
                module_name, label, ..
            } => docvec!(
                module_var_name_doc(module_name),
                ".",
                maybe_escape_identifier_doc(label)
            ),

            ClauseGuard::Not { expression, .. } => {
                docvec!["!", self.wrapped_guard(expression)?]
            }

            ClauseGuard::Constant(constant) => {
                return expression::guard_constant_expression(
                    &mut self.assignments,
                    self.expression_generator.tracker,
                    constant,
                )
            }
        })
    }

    /// Get the path that would assign a variable, if there is one for the given name.
    /// This is in used in clause guards where may use variables defined in
    /// patterns can be referenced, but in the compiled Nix they have not
    /// yet been defined.
    fn path_doc_from_assignments(&self, name: &str) -> Option<Document<'a>> {
        self.assignments
            .iter()
            .find(|assignment| assignment.name == name)
            .map(|assignment| {
                assignment
                    .path
                    .clone()
                    .into_doc_with_subject(assignment.subject.clone())
            })
    }

    pub fn traverse_pattern(
        &mut self,
        subject: &Document<'a>,
        pattern: &'a TypedPattern,
    ) -> Result<(), Error> {
        match pattern {
            Pattern::String { value, .. } => {
                let string = expression::string(value, self.expression_generator.tracker);
                self.push_equality_check(subject.clone(), string);
                Ok(())
            }
            Pattern::Int { value, .. } => {
                let integer = expression::int(value, self.expression_generator.tracker);
                self.push_equality_check(subject.clone(), integer);
                Ok(())
            }
            Pattern::Float { value, .. } => {
                self.push_equality_check(subject.clone(), expression::float(value));
                Ok(())
            }

            Pattern::Discard { .. } => Ok(()),

            Pattern::Variable { name, .. } => {
                self.push_assignment(subject.clone(), name);
                Ok(())
            }

            Pattern::Assign { name, pattern, .. } => {
                self.push_assignment(subject.clone(), name);
                self.traverse_pattern(subject, pattern)
            }

            Pattern::List { elements, tail, .. } => {
                self.push_list_length_check(subject.clone(), elements.len(), tail.is_some());
                for pattern in elements {
                    self.push_string("head");
                    self.traverse_pattern(subject, pattern)?;
                    self.pop_segment();
                    self.push_string("tail");
                }
                self.pop_times(elements.len());
                if let Some(pattern) = tail {
                    self.push_string_times("tail", elements.len());
                    self.traverse_pattern(subject, pattern)?;
                    self.pop_times(elements.len());
                }
                Ok(())
            }

            Pattern::Tuple { elems, .. } => {
                // We don't check the length because type system ensures it's a
                // tuple of the correct size
                for (index, pattern) in elems.iter().enumerate() {
                    self.push_tuple_index(index);
                    self.traverse_pattern(subject, pattern)?;
                    self.pop_segment();
                }
                Ok(())
            }

            Pattern::Constructor {
                type_,
                constructor: Inferred::Known(PatternConstructor { name, .. }),
                ..
            } if type_.is_bool() && name == "True" => {
                self.push_bool_check(subject.clone(), true);
                Ok(())
            }

            Pattern::Constructor {
                type_,
                constructor: Inferred::Known(PatternConstructor { name, .. }),
                ..
            } if type_.is_bool() && name == "False" => {
                self.push_bool_check(subject.clone(), false);
                Ok(())
            }

            Pattern::Constructor {
                type_,
                constructor: Inferred::Known(PatternConstructor { .. }),
                ..
            } if type_.is_nil() => {
                self.push_equality_check(subject.clone(), "null".to_doc());
                Ok(())
            }

            Pattern::Constructor {
                constructor: Inferred::Unknown,
                ..
            } => {
                panic!("Nix generation performed with uninferred pattern constructor");
            }

            Pattern::StringPrefix {
                left_side_string,
                right_side_assignment,
                left_side_assignment,
                ..
            } => {
                self.push_string_prefix_check(subject.clone(), left_side_string);
                if let AssignName::Variable(right) = right_side_assignment {
                    self.push_string_prefix_slice(no_escape_bytes_len(left_side_string));
                    self.push_assignment(subject.clone(), right);
                    // After pushing the assignment we need to pop the prefix slicing we used to
                    // check the condition.
                    self.pop_segment();
                }
                if let Some((left, _)) = left_side_assignment {
                    // "foo" as prefix <> rest
                    //       ^^^^^^^^^ In case the left prefix of the pattern matching is given an
                    //                 alias we bind it to a local variable so that it can be
                    //                 correctly referenced inside the case branch.
                    // let prefix = "foo";
                    // ^^^^^^^^^^^^^^^^^^^ we're adding this assignment inside the if clause
                    //                     the case branch gets translated into.
                    let left_side_string =
                        expression::string(left_side_string, self.expression_generator.tracker);
                    self.push_assignment(left_side_string, left);
                }
                Ok(())
            }

            Pattern::Constructor {
                constructor:
                    Inferred::Known(PatternConstructor {
                        field_map,
                        name: record_name,
                        ..
                    }),
                arguments,
                name,
                type_,
                ..
            } => {
                // Ensure the subject has the correct constructor before
                // proceeding with a per-field check.
                if type_.is_result() {
                    self.push_result_check(subject.clone(), record_name == "Ok");
                } else {
                    self.push_variant_check(subject.clone(), name);
                }

                // For each matched field, append the field access to the path and then
                // traverse its own pattern.
                // E.g., for `MyType(a, Ok(c), True)`, we first check above if the subject
                // has the constructor MyType, and, if so, goes through each field,
                // traversing patterns as follows:
                // 1. The pattern "a" is traversed with the path "subject._0", and will result in
                // us pushing an assignment "a = subject._0;".
                // 2. The pattern "Ok(c)" is traversed with the path "subject._1". This leads to
                // a recursive traversal of the pattern "c" with path "subject._1._0", which leads
                // to an assignment being pushed: "c = subject._0._0".
                // 3. The pattern "True" is traversed with the path "subject._2".
                // Here we simply push a check "if subject._2".
                for (index, arg) in arguments.iter().enumerate() {
                    match field_map {
                        None => self.push_int(index),
                        Some(FieldMap { fields, .. }) => {
                            let find = |(key, &val)| {
                                if val as usize == index {
                                    Some(key)
                                } else {
                                    None
                                }
                            };
                            let label = fields.iter().find_map(find);
                            match label {
                                Some(label) => self.push_string(label),
                                None => self.push_int(index),
                            }
                        }
                    }
                    self.traverse_pattern(subject, &arg.value)?;
                    self.pop_segment();
                }
                Ok(())
            }

            Pattern::BitArray {
                segments: _,
                location,
            } => Err(Error::Unsupported {
                feature: "Pattern matching on bit arrays".into(),
                location: *location,
            }),
            Pattern::VarUsage { location, .. } => Err(Error::Unsupported {
                feature: "Bit array matching".into(),
                location: *location,
            }),
        }
    }

    fn push_assignment(&mut self, subject: Document<'a>, name: &'a EcoString) {
        let var = self.next_local_var(name);
        let path = self.finish_path();
        self.assignments.push(Assignment {
            subject,
            path,
            var,
            name,
        });
    }

    fn push_string_prefix_check(&mut self, subject: Document<'a>, prefix: &'a str) {
        self.checks.push(Check::StringPrefix {
            prefix,
            subject,
            path: self.finish_path(),
        })
    }

    fn push_bool_check(&mut self, subject: Document<'a>, expected_to_be_true: bool) {
        self.checks.push(Check::Bool {
            expected_to_be_true,
            subject,
            path: self.finish_path(),
        })
    }

    fn push_equality_check(&mut self, subject: Document<'a>, to: Document<'a>) {
        self.checks.push(Check::Equal {
            to,
            subject,
            path: self.finish_path(),
        })
    }

    fn push_variant_check(&mut self, subject: Document<'a>, kind: &'a str) {
        self.checks.push(Check::Variant {
            kind,
            subject,
            path: self.finish_path(),
        })
    }

    fn push_result_check(&mut self, subject: Document<'a>, is_ok: bool) {
        self.checks.push(Check::Result {
            is_ok,
            subject,
            path: self.finish_path(),
        })
    }

    fn push_list_length_check(
        &mut self,
        subject: Document<'a>,
        expected_length: usize,
        has_tail_spread: bool,
    ) {
        self.checks.push(Check::ListLength {
            expected_length,
            has_tail_spread,
            subject,
            path: self.finish_path(),
        })
    }

    fn push_bit_array_length_check(
        &mut self,
        subject: Document<'a>,
        expected_bytes: usize,
        has_tail_spread: bool,
    ) {
        self.checks.push(Check::BitArrayLength {
            expected_bytes,
            has_tail_spread,
            subject,
            path: self.finish_path(),
        })
    }
}

/// A path of field accesses and function calls on top of the subject.
/// Patterns may attach accesses to the path in order to reason about
/// a particular field in the subject being matched.
///
/// For example, when matching on a tuple originally assigned to the variable
/// `x` (the subject), for each element in the matching pattern `#(a, b, #(c, d))`,
/// we assign `a = x._0` (here `._0` is the path), `b = x._1`, `c = x._2._0` (extended path)
/// and `d = x._2._1`.
#[derive(Debug, Clone)]
pub struct SubjectPath<'a> {
    /// What is attached to the left of the subject.
    left: Document<'a>,
    /// What is attached to the right of the subject.
    right: Document<'a>,
}

impl<'a> SubjectPath<'a> {
    fn new() -> Self {
        Self {
            left: nil(),
            right: nil(),
        }
    }

    /// Adds a component to this path, which may attach some document content
    /// to the left and to the right of the subject.
    fn add_component(mut self, left: Document<'a>, right: Document<'a>) -> Self {
        self.left = left.append(self.left);
        self.right = self.right.append(right);
        self
    }

    /// Adds a component to this path which only attaches content to the right
    /// of the subject.
    fn add_right_component(mut self, component: Document<'a>) -> Self {
        self.right = self.right.append(component);
        self
    }

    /// Applies the path to a subject, generating a Document with code which
    /// extracts the value within the subject after applying the path.
    pub(crate) fn into_doc_with_subject(self, subject: Document<'a>) -> Document<'a> {
        docvec![self.left, subject, self.right]
    }
}

#[derive(Debug)]
pub struct CompiledPattern<'a> {
    pub checks: Vec<Check<'a>>,
    pub assignments: Vec<Assignment<'a>>,
}

impl<'a> CompiledPattern<'a> {
    pub fn has_assignments(&self) -> bool {
        !self.assignments.is_empty()
    }
}

#[derive(Debug)]
pub struct Assignment<'a> {
    pub name: &'a str,
    var: Document<'a>,
    pub subject: Document<'a>,
    pub path: SubjectPath<'a>,
}

impl<'a> Assignment<'a> {
    /// Create a new assignment which simply reassigns the subject to some other name.
    pub fn reassign_subject(name: &'a str, var: Document<'a>, subject: Document<'a>) -> Self {
        Self {
            name,
            var,
            subject,
            path: SubjectPath::new(),
        }
    }

    /// Converts this assignment into a document ready for use within `let...in`:
    ///
    /// ```nix
    /// var = value;
    /// ```
    pub fn into_doc(self) -> Document<'a> {
        syntax::assignment_line(self.var, self.path.into_doc_with_subject(self.subject))
    }

    /// Similar to [`Assignment::into_doc`]; however, only evaluates the assigned value
    /// if a given assertion succeeds. Useful in `let assert Pat = val`. Compiles to:
    ///
    /// ```nix
    /// var = builtins.seq assertion value;
    /// ```
    pub fn into_doc_with_assertion(self, assertion: Document<'a>) -> Document<'a> {
        syntax::assignment_line(
            self.var,
            syntax::fn_call(
                "builtins.seq".to_doc(),
                [assertion, self.path.into_doc_with_subject(self.subject)],
            ),
        )
    }
}

#[derive(Debug)]
pub enum Check<'a> {
    Result {
        subject: Document<'a>,
        path: SubjectPath<'a>,
        is_ok: bool,
    },
    Variant {
        subject: Document<'a>,
        path: SubjectPath<'a>,
        kind: &'a str,
    },
    Equal {
        subject: Document<'a>,
        path: SubjectPath<'a>,
        to: Document<'a>,
    },
    ListLength {
        subject: Document<'a>,
        path: SubjectPath<'a>,
        expected_length: usize,
        has_tail_spread: bool,
    },
    BitArrayLength {
        subject: Document<'a>,
        path: SubjectPath<'a>,
        expected_bytes: usize,
        has_tail_spread: bool,
    },
    StringPrefix {
        subject: Document<'a>,
        path: SubjectPath<'a>,
        prefix: &'a str,
    },
    Bool {
        subject: Document<'a>,
        path: SubjectPath<'a>,
        expected_to_be_true: bool,
    },
    Guard {
        expression: Document<'a>,
    },
}

impl<'a> Check<'a> {
    pub fn into_doc(self, tracker: &mut UsageTracker, match_desired: bool) -> Document<'a> {
        match self {
            Check::Guard { expression } => {
                if match_desired {
                    expression
                } else {
                    docvec!["!", expression]
                }
            }

            Check::Bool {
                expected_to_be_true,
                subject,
                path,
            } => {
                if expected_to_be_true == match_desired {
                    path.into_doc_with_subject(subject)
                } else {
                    docvec!["!", path.into_doc_with_subject(subject)]
                }
            }

            Check::Variant {
                subject,
                path,
                kind,
            } => {
                let operator = if match_desired { " == " } else { " != " };
                docvec![
                    path.into_doc_with_subject(subject),
                    ".__gleamTag",
                    operator,
                    syntax::string_without_escapes_or_backslashes(kind)
                ]
            }

            Check::Result {
                subject,
                path,
                is_ok,
            } => {
                let kind = if is_ok { "Ok" } else { "Error" };
                let operator = if match_desired { " == " } else { " != " };
                docvec![
                    path.into_doc_with_subject(subject),
                    ".__gleamTag",
                    operator,
                    "\"",
                    kind,
                    "\""
                ]
            }

            Check::Equal { subject, path, to } => {
                let operator = if match_desired { " == " } else { " != " };
                docvec![path.into_doc_with_subject(subject), operator, to]
            }

            Check::ListLength {
                subject,
                path,
                expected_length,
                has_tail_spread,
            } => {
                let resolved_subject = path.into_doc_with_subject(subject);
                let length_check_fun = if has_tail_spread {
                    tracker.list_has_at_least_length_used = true;
                    "listHasAtLeastLength".to_doc()
                } else {
                    tracker.list_has_length_used = true;
                    "listHasLength".to_doc()
                };
                let length_check = syntax::fn_call(
                    length_check_fun,
                    [resolved_subject, expected_length.to_doc()],
                );

                if match_desired {
                    length_check
                } else {
                    docvec!["!(", length_check, ")"]
                }
            }
            Check::BitArrayLength { .. } => todo!("bit array"),
            Check::StringPrefix {
                subject,
                path,
                prefix,
            } => {
                tracker.str_has_prefix_used = true;

                let prefix = expression::string(prefix, tracker);
                let has_prefix = syntax::fn_call(
                    "strHasPrefix".to_doc(),
                    [prefix, path.into_doc_with_subject(subject)],
                );
                if match_desired {
                    has_prefix
                } else {
                    docvec!["!(", has_prefix, ")"]
                }
            }
        }
    }

    /// If the check, within chains of '&&' and '||', may require wrapping in (...).
    pub(crate) fn may_require_wrapping(&self) -> bool {
        match self {
            Check::Result { .. }
            | Check::Variant { .. }
            | Check::Equal { .. }
            | Check::Bool { .. }
            | Check::ListLength { .. }
            | Check::BitArrayLength { .. }
            | Check::StringPrefix { .. } => false,
            Check::Guard { .. } => true,
        }
    }
}

pub(crate) fn assign_subject<'a>(
    expression_generator: &mut expression::Generator<'_>,
    subject: &'a TypedExpr,
) -> (Document<'a>, Option<Document<'a>>) {
    static ASSIGNMENT_VAR_ECO_STR: OnceLock<EcoString> = OnceLock::new();

    match subject {
        // If the value is a variable we don't need to assign it to a new
        // variable, we can the value expression safely without worrying about
        // performing computation or side effects multiple times.
        TypedExpr::Var {
            name, constructor, ..
        } if constructor.is_local_variable() => (expression_generator.local_var(name), None),
        // If it's not a variable we need to assign it to a variable
        // to avoid rendering the subject expression multiple times
        _ => {
            let subject = expression_generator.next_local_var(
                ASSIGNMENT_VAR_ECO_STR.get_or_init(|| ASSIGNMENT_VAR.into()),
                false,
            );
            (subject.clone(), Some(subject))
        }
    }
}

pub(crate) fn assign_subjects<'a>(
    expression_generator: &mut expression::Generator<'_>,
    subjects: &'a [TypedExpr],
) -> Vec<(Document<'a>, Option<Document<'a>>)> {
    let mut out = Vec::with_capacity(subjects.len());
    for subject in subjects {
        out.push(assign_subject(expression_generator, subject))
    }
    out
}

/// Calculates the length of str as UTF-8 bytes without escape characters.
fn no_escape_bytes_len(str: &EcoString) -> usize {
    let mut filtered_str = String::new();
    let mut str_iter = str.chars().peekable();
    loop {
        match str_iter.next() {
            Some('\\') => match str_iter.next() {
                // Check for Unicode escape sequence, e.g. \u{00012FF}
                Some('u') => {
                    if str_iter.peek() != Some(&'{') {
                        // Invalid Unicode escape sequence
                        filtered_str.push('u');
                        continue;
                    }

                    // Consume the left brace after peeking
                    let _ = str_iter.next();

                    let codepoint_str = str_iter
                        .peeking_take_while(char::is_ascii_hexdigit)
                        .collect::<String>();

                    if codepoint_str.is_empty() || str_iter.peek() != Some(&'}') {
                        // Invalid Unicode escape sequence
                        filtered_str.push_str("u{");
                        filtered_str.push_str(&codepoint_str);
                        continue;
                    }

                    let codepoint = u32::from_str_radix(&codepoint_str, 16)
                        .ok()
                        .and_then(char::from_u32);

                    if let Some(codepoint) = codepoint {
                        // Consume the right brace after peeking
                        let _ = str_iter.next();

                        // Consider this codepoint's length instead of
                        // that of the Unicode escape sequence itself
                        filtered_str.push(codepoint);
                    } else {
                        // Invalid Unicode escape sequence
                        // (codepoint value not in base 16 or too large)
                        filtered_str.push_str("u{");
                        filtered_str.push_str(&codepoint_str);
                    }
                }
                Some(c) => filtered_str.push(c),
                None => break,
            },
            Some(c) => filtered_str.push(c),
            None => break,
        }
    }

    filtered_str.len()
}
