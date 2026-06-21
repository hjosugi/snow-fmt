//! GROUP BY extensions. CUBE/ROLLUP/GROUPING parse as ordinary call expressions (they are not
//! reserved); only GROUPING SETS needs a dedicated rule. Each must round-trip.

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
fn grouping_sets() {
    let sql = "SELECT a, SUM(x) FROM t GROUP BY GROUPING SETS ((a, b), (a), ())";
    clean(sql);
    assert!(has(sql, SyntaxKind::GROUPING_SETS));
}

#[test]
fn grouping_sets_mixed_with_plain_columns() {
    clean("SELECT a FROM t GROUP BY a, GROUPING SETS ((b), (c))");
}

#[test]
fn cube_and_rollup_are_call_expressions() {
    clean("SELECT a, SUM(x) FROM t GROUP BY CUBE(a, b)");
    clean("SELECT a, SUM(x) FROM t GROUP BY ROLLUP(a, b)");
    // …and not mistaken for a GROUPING SETS construct.
    assert!(!has(
        "SELECT a FROM t GROUP BY CUBE(a, b)",
        SyntaxKind::GROUPING_SETS
    ));
}

#[test]
fn grouping_function_is_not_a_keyword() {
    // GROUPING(x) / GROUPING_ID(x) must still parse as function calls.
    clean("SELECT GROUPING(a), GROUPING_ID(a, b) FROM t GROUP BY a, b");
}
