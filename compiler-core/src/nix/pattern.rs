use ecow::EcoString;
use std::sync::OnceLock;

use crate::ast::TypedExpr;
use crate::nix::expression;
use crate::pretty::Document;

pub static ASSIGNMENT_VAR: &str = "_pat'";

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
            let subject = expression_generator
                .next_local_var(ASSIGNMENT_VAR_ECO_STR.get_or_init(|| ASSIGNMENT_VAR.into()));
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
