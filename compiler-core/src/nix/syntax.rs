//! Gathers functions which generate Nix syntax.

use crate::docvec;
use crate::nix::{Output, INDENT};
use crate::pretty::{break_, concat, join, nil, Document, Documentable};
use ecow::EcoString;
use itertools::Itertools;
use regex::Regex;
use std::borrow::Cow;
use std::sync::OnceLock;

/// Attempts to generate a valid Nix path.
/// Not always possible when the value is surrounded by <...> (Nix store path).
///
/// Valid Nix paths include:
/// 1. Those starting with `/` (absolute paths).
/// 2. Those starting with `./` (relative paths).
/// 3. Those starting with `~/` (user home paths).
/// 4. Those surrounded by `<...>` (Nix store paths - don't support interpolation with ${...}).
///
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
                current_prefix = value.get(0..2).expect("string should have two characters");
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
    invalid_path_segment_pattern().replace_all(value, |captures: &regex::Captures<'_>| {
        format!("${{\"{}\"}}", sanitize_string(captures.extract::<0>().0))
    })
}

/// Pattern with invalid characters in a typical Nix path.
/// Here we also consider ${...} interpolation as invalid.
fn invalid_path_segment_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| Regex::new(r"[^a-zA-Z0-9./_\-+]+").expect("regex should be valid"))
}

/// Sanitize a Nix string.
pub fn sanitize_string(value: &str) -> Cow<'_, str> {
    if value.contains('\n') || value.contains("${") {
        Cow::Owned(value.replace('\n', r"\n").replace("${", "\\${"))
    } else {
        Cow::Borrowed(value)
    }
}

/// Generates a valid Nix string from some string contents.
/// Assumes said contents won't include any escape sequences,
/// any backslashes or any double quotes.
/// This is meant to create strings from valid Gleam identifiers
/// without having to check for backslashes and double quotes each time.
pub fn string_without_escapes_or_backslashes(value: &str) -> Document<'_> {
    debug_assert!(!value.contains(['\\', '"']));

    match sanitize_string(value) {
        Cow::Owned(string) => EcoString::from(string).to_doc(),
        Cow::Borrowed(value) => value.to_doc(),
    }
    .surround("\"", "\"")
}

/// Generates a valid Nix string escaping any backslashes or quotes,
/// thus printing escape sequences literally. Additionally,
/// sanitizes the string such that newlines become `\n` and
/// `${` becomes `\${`.
pub fn string_escaping_backslashes_and_quotes(value: &str) -> Document<'_> {
    if value.contains('\\') || value.contains('"') {
        let replaced_value = EcoString::from(value)
            .replace("\\", "\\\\")
            .replace("\"", "\\\"");

        match sanitize_string(&replaced_value) {
            Cow::Owned(string) => EcoString::from(string).to_doc(),
            Cow::Borrowed(_) => replaced_value.to_doc(),
        }
        .surround("\"", "\"")
    } else {
        string_without_escapes_or_backslashes(value)
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
        break_("", " "),
        body
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
    join(args.into_iter().map(|arg| arg.append(":")), break_("", " ")).group()
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
    docvec![fun, args].nest(INDENT).group()
}

/// Generates `inherit` in a Nix attribute set or let...in declaration:
///
/// ```nix
/// inherit x y z;
/// ```
pub fn inherit<'a>(items: impl IntoIterator<Item = Document<'a>>) -> Document<'a> {
    let mut spaced_items = items
        .into_iter()
        .map(|name| docvec!(break_("", " "), name))
        .peekable();

    if spaced_items.peek().is_none() {
        // Note: an 'inherit' without items is valid Nix syntax.
        return "inherit;".to_doc();
    }

    docvec!["inherit", concat(spaced_items), break_("", ""), ";"]
        .nest(INDENT)
        .group()
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
            Some(value) => assignment_line(key, value),
            None => docvec!["inherit ", key.to_doc(), ";"],
        }
    });
    let fields = join(fields, break_("", " "));

    if empty {
        "{ }".to_doc()
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

    Ok(attr_set(fields.to_doc()))
}

/// Generates an attribute set's delimiters around its inner contents:
///
/// ```nix
/// { inner }
/// ```
pub fn attr_set(inner: Document<'_>) -> Document<'_> {
    docvec![
        docvec!["{", break_("", " "), inner]
            .nest(INDENT)
            .append(break_("", " "))
            .group(),
        "}"
    ]
}

/// Generates a Nix list:
///
/// ```nix
/// [ element1 element2 ... ]
/// ```
pub fn list<'a, Elements: IntoIterator<Item = Document<'a>>>(elements: Elements) -> Document<'a> {
    let spaced_elements = elements
        .into_iter()
        .map(|element| break_("", " ").append(element));

    docvec![
        docvec!["[", concat(spaced_elements)]
            .nest(INDENT)
            .append(break_("", " "))
            .group(),
        "]"
    ]
}

pub fn is_nix_keyword(word: &str) -> bool {
    matches!(
        word,
        // Keywords and reserved words
        "if" | "then" | "else" | "assert" | "with" | "let" | "in" | "rec" | "inherit" | "or"
    )
}

/// Pattern with valid characters in a Nix identifier.
/// Make sure to combine this with `is_nix_keyword`.
fn nix_is_identifier_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN
        .get_or_init(|| Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_'\-]*$").expect("regex should be valid"))
}

/// Checks if the given string is syntactically a valid Nix identifier.
/// Combine this with `is_nix_keyword` to make sure the string is not a keyword.
pub fn is_nix_identifier_or_keyword(string: &str) -> bool {
    nix_is_identifier_pattern().is_match(string)
}

/// If the label would be a keyword, it is quoted.
/// Assumes the label is a valid Gleam identifier, thus doesn't check for other
/// invalid attribute names.
pub fn maybe_quoted_attr_set_label_from_identifier(label: &str) -> Document<'_> {
    if is_nix_keyword(label) {
        string_without_escapes_or_backslashes(label)
    } else {
        label.to_doc()
    }
}

/// If the label would be a keyword or not an identifier, it is quoted.
/// Otherwise, it is kept as is.
pub fn maybe_quoted_attr_set_label(label: &str) -> Document<'_> {
    if is_nix_keyword(label) || !is_nix_identifier_or_keyword(label) {
        string_escaping_backslashes_and_quotes(label)
    } else {
        label.to_doc()
    }
}
