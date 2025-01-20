//! Gathers functions which generate Nix syntax.

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
