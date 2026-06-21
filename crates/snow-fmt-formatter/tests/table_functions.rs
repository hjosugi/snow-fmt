//! Table functions: FLATTEN / LATERAL / TABLE(...) / UDTFs, with named arguments (`input => x`).
//! Each case parses clean, formats to valid SQL, is idempotent, and preserves its tokens.

use snow_fmt_formatter::format;
use snow_fmt_lexer::{tokenize, SyntaxKind};
use snow_fmt_parser::parse;

fn signature(sql: &str) -> Vec<String> {
    tokenize(sql)
        .tokens
        .into_iter()
        .filter(|t| !t.kind.is_trivia() && t.kind != SyntaxKind::SEMICOLON)
        .map(|t| t.text.to_ascii_uppercase())
        .collect()
}

const CASES: &[&str] = &[
    "select f.value from t, lateral flatten(input => t.col) f",
    "select f.value from t, lateral flatten(input => t.col)",
    "select f.value, f.key from t, lateral flatten(input => t.col, path => 'a.b') as f",
    "select * from table(flatten(input => parse_json('[1,2]'))) f",
    "select * from t, table(flatten(input => t.c, outer => true, recursive => false, mode => 'both')) f",
    "select value from lateral flatten(input => parse_json('{}'))",
    "select * from my_udtf(1, 2) x",
    "select * from t, lateral flatten(input => t.a) a, lateral flatten(input => a.value) b",
    "select n.value:id::int from raw r, lateral flatten(input => r.payload:items) n",
    "select f.index, f.value from t, lateral flatten(input => t.arr) f where f.value > 0",
    // JOIN LATERAL
    "select * from t join lateral flatten(input => t.c) f on f.value is not null",
    "select * from t left join lateral flatten(input => t.c) f on true",
    // named args mixed with positional, nested flatten
    "select * from table(generator(rowcount => 10))",
    "select * from t, lateral flatten(input => t.a) a, lateral flatten(input => a.value, outer => true) b",
    "select g.value from t, lateral flatten(input => t.j, recursive => true) g where g.path like 'a%'",
    // flatten feeding a subquery / aggregate
    "select count(*) from (select f.value v from t, lateral flatten(input => t.c) f) where v > 0",
];

#[test]
fn all_cases_parse_clean() {
    for sql in CASES {
        let errors = parse(sql).errors().to_vec();
        assert!(errors.is_empty(), "parse errors for {sql:?}: {errors:?}");
    }
}

#[test]
fn formatting_is_idempotent() {
    for sql in CASES {
        let once = format(sql);
        assert_eq!(once, format(&once), "not idempotent:\n{sql}\n---\n{once}");
    }
}

#[test]
fn formatted_output_is_valid_sql() {
    for sql in CASES {
        let formatted = format(sql);
        assert!(
            parse(&formatted).errors().is_empty(),
            "invalid output for {sql:?}:\n{formatted}"
        );
    }
}

#[test]
fn formatting_preserves_tokens() {
    for sql in CASES {
        let formatted = format(sql);
        assert_eq!(
            signature(sql),
            signature(&formatted),
            "tokens changed:\n{sql}"
        );
    }
}

#[test]
fn flatten_golden() {
    assert_eq!(
        format("select f.value, f.key from t, lateral flatten(input => t.col, path => 'a.b') as f"),
        "SELECT f.value, f.key\nFROM t,\nLATERAL flatten(input => t.col, path => 'a.b') AS f;\n",
    );
}
