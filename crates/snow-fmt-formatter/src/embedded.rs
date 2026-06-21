//! The seam between the SQL formatter and a language-specific formatter for embedded bodies.
//!
//! Snowflake UDFs/procedures carry a body in another language inside `$$ … $$`:
//!
//! ```sql
//! CREATE FUNCTION add(a FLOAT, b FLOAT) RETURNS FLOAT LANGUAGE JAVASCRIPT AS $$
//!   return A + B;
//! $$;
//! ```
//!
//! Formatting that body well means handing it to a real formatter for that language (Biome for
//! JavaScript). To keep the core SQL formatter pure, dependency-free, and deterministic, that
//! formatter is injected through this trait rather than hard-wired. [`crate::format`] /
//! [`crate::format_with`] use no embedded formatter (bodies are emitted verbatim);
//! [`crate::format_with_embedded`] lets a caller plug one in — e.g. one that shells out to an
//! installed `biome` (then `prettier`, then `deno fmt`), or a future in-process engine.

/// Formats an embedded-language body. Implementors decide which languages they handle.
pub trait EmbeddedFormatter {
    /// Format `body` (the text *between* the `$$` delimiters) written in `language` (lower-cased,
    /// e.g. `"javascript"`). Return `Some(formatted)` to replace the body, or `None` to leave it
    /// untouched. Must not panic; returning `None` is the safe default for anything unsupported.
    fn format(&self, language: &str, body: &str) -> Option<String>;
}
