//! A width-aware formatter for Snowflake SQL.
//!
//! The pipeline is the one proven by Prettier, `biome_formatter` and `ruff_formatter`:
//!
//! ```text
//! source ──parse──▶ lossless CST ──lower──▶ Doc IR ──print──▶ formatted text
//! ```
//!
//! * [`sql`] lowers the CST into a [`doc::Doc`] — a tree of *intent* (groups, indents, line
//!   breaks) rather than characters.
//! * [`printer`] resolves that intent against [`FormatOptions::line_width`], keeping short
//!   constructs on one line and exploding long ones.
//!
//! ## Guarantees & current scope
//!
//! [`format`] is **total and conservative**: it never panics and never produces invalid SQL. To
//! uphold that while comment attachment is still unimplemented, it returns the input **unchanged**
//! when either of these holds:
//!
//! * the input does not parse cleanly (it has syntax errors), or
//! * the input contains comments.
//!
//! So today snow-fmt reformats *comment-free, syntactically valid* SQL and leaves everything else
//! byte-for-byte intact. That keeps the headline invariants true now —
//! **idempotent** (`format(format(x)) == format(x)`) and **never drops a comment** — and the
//! comment path is a focused next step rather than a correctness risk. See `HANDOFF.md`.

mod builder;
mod doc;
mod options;
mod printer;
mod sql;

pub use options::{FormatOptions, KeywordCase};

use snow_fmt_syntax::SyntaxNode;

/// Format Snowflake SQL with the default options.
///
/// Equivalent to [`format_with`] using [`FormatOptions::default`].
pub fn format(source: &str) -> String {
    format_with(source, &FormatOptions::default())
}

/// Format Snowflake SQL with explicit [`FormatOptions`].
///
/// Returns `source` unchanged if it has syntax errors or contains comments (see the crate-level
/// docs); otherwise returns the reformatted SQL, which always ends in a single trailing newline.
pub fn format_with(source: &str, opts: &FormatOptions) -> String {
    let parse = snow_fmt_parser::parse(source);

    // Conservative fallbacks: never reformat broken SQL, and never risk dropping a comment.
    if !parse.errors().is_empty() {
        return source.to_string();
    }
    let root = parse.syntax();
    if has_comments(&root) {
        return source.to_string();
    }

    let formatted = printer::print(sql::format_source(&root, opts), opts);
    normalize_trailing_newline(formatted)
}

/// Does the tree contain any comment token? (Comment attachment is not implemented yet, so the
/// formatter declines to touch such inputs rather than drop the comment.)
fn has_comments(root: &SyntaxNode) -> bool {
    root.descendants_with_tokens()
        .filter_map(|e| e.into_token())
        .any(|t| t.kind().is_comment())
}

/// Ensure the output ends with exactly one `\n` — unless it is empty (e.g. whitespace-only input).
fn normalize_trailing_newline(mut s: String) -> String {
    if s.is_empty() {
        return s;
    }
    while s.ends_with('\n') {
        s.pop();
    }
    s.push('\n');
    s
}
