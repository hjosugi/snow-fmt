//! FLATTEN / LATERAL / TABLE table functions and named arguments. Round-trip + structure.
//!
//! Reference: <https://docs.snowflake.com/en/sql-reference/functions/flatten>

use snow_fmt_parser::{parse, SyntaxKind};

fn clean(s: &str) {
    let p = parse(s);
    assert!(
        p.errors().is_empty(),
        "unexpected errors for {s:?}: {:?}",
        p.errors()
    );
    assert_eq!(p.syntax().to_string(), s, "round-trip failed for {s:?}");
}

fn has(s: &str, kind: SyntaxKind) -> bool {
    parse(s).syntax().descendants().any(|n| n.kind() == kind)
}

#[test]
fn lateral_flatten() {
    let sql = "SELECT f.value FROM t, LATERAL FLATTEN(input => t.col) f";
    clean(sql);
    assert!(has(sql, SyntaxKind::NAMED_ARG));
}

#[test]
fn flatten_all_named_args() {
    clean("SELECT * FROM t, TABLE(FLATTEN(input => t.c, path => 'a', outer => true, recursive => false, mode => 'both')) f");
}

#[test]
fn table_wrapper_and_udtf() {
    clean("SELECT * FROM TABLE(FLATTEN(input => parse_json('[1,2]'))) f");
    clean("SELECT * FROM my_udtf(1, 2) x");
    clean("SELECT * FROM TABLE(GENERATOR(rowcount => 10))");
}

#[test]
fn join_lateral_flatten() {
    clean("SELECT * FROM t JOIN LATERAL FLATTEN(input => t.c) f ON f.value IS NOT NULL");
    clean("SELECT * FROM t LEFT JOIN LATERAL FLATTEN(input => t.c) f ON TRUE");
}

#[test]
fn chained_lateral_flatten() {
    clean("SELECT * FROM t, LATERAL FLATTEN(input => t.a) a, LATERAL FLATTEN(input => a.value) b");
}

#[test]
fn flatten_output_columns_are_plain_names() {
    // VALUE / KEY / INDEX / PATH / THIS / SEQ are not reserved, so `f.value` etc. parse cleanly.
    clean("SELECT f.seq, f.key, f.path, f.index, f.value, f.this FROM t, LATERAL FLATTEN(input => t.c) f");
}
