//! Gathers functions which generate Nix syntax.

use crate::docvec;
use crate::nix::{Output, INDENT};
use crate::pretty::{break_, concat, join, nil, Document, Documentable};
use itertools::Itertools;
use std::borrow::Cow;

/// Attempts to generate a valid Nix path.
/// Not always possible when the value is surrounded by <...> (Nix store path).
///
/// Valid Nix paths include:
/// 1. Those starting with `/` (absolute paths).
/// 2. Those starting with `./` (relative paths).
/// 3. Those starting with `~/` (user home paths).
/// 4. Those surrounded by `<...>` (Nix store paths - don't support interpolation with ${...}).
/// Anything not in the four categories above is converted to a relative path.
pub fn path(value: &str) -> Cow<'_, str> {
    // TODO: Consider introducing fallibility somewhere here.
    match value {
        "" => Cow::Borrowed(value),
        "~" | "~/" => Cow::Borrowed("~/."),
        "." | "./" => Cow::Borrowed("./."),
        "/" => Cow::Borrowed("/."),
        _ if value.starts_with('<') && value.ends_with('>') => {
            // Can't sanitize further (Nix doesn't support ${...} insertions here),
            // so just remove newlines and extra '>' as an "emergency measure" to
            // guarantee that invalid syntax will crash Nix.
            if value.contains('\n')
                || value
                    .get(..value.len() - 1)
                    .unwrap_or_default()
                    .contains('>')
            {
                Cow::Owned(format!("{}>", value.replace(['\n', '>'], "")))
            } else {
                Cow::Borrowed(value)
            }
        }
        _ => {
            let new_prefix;
            let current_prefix;
            if value.starts_with('/') {
                new_prefix = "";
                current_prefix = "/";
            } else if value.starts_with("./") || value.starts_with("~/") {
                new_prefix = "";
                current_prefix = &value.get(0..2).expect("string should have two characters");
            } else {
                // Assume a relative path when the prefix is valid
                new_prefix = "./";
                current_prefix = "";
            };

            // Nix restriction: paths must not end with a trailing slash
            let suffix = if value.ends_with('/') { "." } else { "" };

            match sanitize_path(value.get(current_prefix.len()..).unwrap_or_default()) {
                Cow::Owned(sanitized) => {
                    Cow::Owned(format!("{new_prefix}{current_prefix}{sanitized}{suffix}"))
                }
                Cow::Borrowed(_) if new_prefix.is_empty() && suffix.is_empty() => {
                    Cow::Borrowed(value)
                }
                Cow::Borrowed(_) => Cow::Owned(format!("{new_prefix}{value}{suffix}")),
            }
        }
    }
}

/// Sanitize a Nix path's contents.
/// Replaces any invalid path syntax with ${"... string ..."}.
pub fn sanitize_path(value: &str) -> Cow<'_, str> {
    let path_regex = regex::Regex::new(r"[^a-zA-Z0-9./_\-+]+").expect("regex should be valid");
    path_regex.replace_all(value, |captures: &regex::Captures<'_>| {
        format!("${{\"{}\"}}", sanitize_string(captures.extract::<0>().0))
    })
}

/// Sanitize a Nix string.
pub fn sanitize_string(value: &str) -> Cow<'_, str> {
    if value.contains('\n') || value.contains("${") {
        Cow::Owned(value.replace('\n', r"\n").replace("${", "\\${"))
    } else {
        Cow::Borrowed(value)
    }
}

/// Produces an assignment line in Nix:
///
/// ```nix
/// let
///   name = value;  // <--- that line is generated by this function
/// in ...
/// ```
pub fn assignment_line<'a>(name: Document<'a>, value: Document<'a>) -> Document<'a> {
    docvec![
        name,
        " =",
        docvec![break_("", " "), value, ";"].nest(INDENT).group()
    ]
}

/// Generates a Nix expression in the form
///
/// ```nix
/// let
///   assignment1 = value;
///   assignment2 = value;
/// in body
/// ```
pub fn let_in<'a>(
    assignments: impl IntoIterator<Item = Document<'a>>,
    body: Document<'a>,
    extra_assign_break: bool,
) -> Document<'a> {
    let extra_assign_break = if extra_assign_break {
        break_("", "")
    } else {
        nil()
    };

    docvec![
        "let",
        docvec![
            break_("", " "),
            join(assignments, break_("", " ").append(extra_assign_break))
        ]
        .nest(INDENT),
        break_("", " "),
        "in",
        docvec![break_("", " "), body].nest(INDENT).group(),
    ]
}

/// Generates the arguments in a function declaration:
///
/// ```nix
/// arg1: arg2:
/// ```
pub(crate) fn wrap_args<'a, I>(args: I) -> Document<'a>
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

/// Generates a function call in Nix:
///
/// ```nix
/// fun arg1 arg2 arg3
/// ```
pub fn fn_call<'a>(
    fun: Document<'a>,
    args: impl IntoIterator<Item = Document<'a>>,
) -> Document<'a> {
    let args = concat(args.into_iter().map(|arg| break_("", " ").append(arg)));
    docvec![fun, args]
}

/// Generates `inherit` in a Nix attribute set or let...in declaration:
///
/// ```nix
/// inherit x y z;
/// ```
pub fn inherit<'a>(items: impl IntoIterator<Item = Document<'a>>) -> Document<'a> {
    let spaced_items = items.into_iter().map(|name| docvec!(break_("", " "), name));

    // Note: an 'inherit' without items is valid Nix syntax.
    docvec!["inherit", concat(spaced_items).nest(INDENT).group(), ";"]
}

/// Generates an attribute set with the given attribute/value pairs.
/// If the value is `None`, it is inherited from the variable with the same
/// name as attribute (instead of `attr = value;`, prints `inherit attr;`).
pub fn wrap_attr_set<'a>(
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

/// Similar to [`wrap_attr_set`], but handles set values which may error.
pub fn try_wrap_attr_set<'a>(
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

pub fn is_nix_keyword(word: &str) -> bool {
    matches!(
        word,
        // Keywords and reserved words
        "if" | "then" | "else" | "assert" | "with" | "let" | "in" | "rec" | "inherit" | "or"
    )
}
