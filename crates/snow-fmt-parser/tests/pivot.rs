//! PIVOT / UNPIVOT table operators. Each must parse cleanly, round-trip, and form a PIVOT_CLAUSE.
//!
//! Reference: <https://docs.snowflake.com/en/sql-reference/constructs/pivot>

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
fn pivot_with_value_list() {
    let sql =
        "SELECT * FROM monthly_sales PIVOT(SUM(amount) FOR month IN ('JAN', 'FEB', 'MAR')) AS p";
    clean(sql);
    assert!(has(sql, SyntaxKind::PIVOT_CLAUSE));
}

#[test]
fn unpivot_with_column_list() {
    clean("SELECT * FROM sales UNPIVOT(amount FOR month IN (jan, feb, mar))");
    assert!(has(
        "SELECT * FROM sales UNPIVOT(amount FOR month IN (jan, feb, mar))",
        SyntaxKind::PIVOT_CLAUSE,
    ));
}

#[test]
fn pivot_any_and_subquery() {
    clean("SELECT * FROM t PIVOT(SUM(x) FOR k IN (ANY ORDER BY k))");
    clean("SELECT * FROM t PIVOT(AVG(x) FOR k IN (SELECT DISTINCT k FROM t))");
}

#[test]
fn pivot_then_alias_and_where() {
    clean(
        "SELECT dept FROM emp PIVOT(SUM(sal) FOR mon IN ('jan', 'feb')) p WHERE dept IS NOT NULL",
    );
}

#[test]
fn pivot_values_with_aliases() {
    clean("SELECT * FROM t PIVOT(SUM(amount) FOR month IN ('JAN' AS jan, 'FEB' AS feb))");
    clean("SELECT * FROM t PIVOT(SUM(amount) FOR m IN (1 AS jan, 2 AS feb))");
}

#[test]
fn unpivot_include_nulls() {
    clean("SELECT * FROM t UNPIVOT INCLUDE NULLS (val FOR name IN (a, b, c))");
    clean("SELECT * FROM t UNPIVOT EXCLUDE NULLS (val FOR name IN (a, b))");
}

#[test]
fn round_trips_when_incomplete() {
    for s in [
        "SELECT * FROM t PIVOT(",
        "SELECT * FROM t UNPIVOT(x FOR y IN (",
    ] {
        assert_eq!(
            parse(s).syntax().to_string(),
            s,
            "round-trip failed for {s:?}"
        );
    }
}
