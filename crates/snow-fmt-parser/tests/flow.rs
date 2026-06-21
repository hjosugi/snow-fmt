//! Snowflake flow operator `->>`: a chain of statements where `$n` (in a FROM clause) references
//! a previous step. Each chain must parse cleanly, round-trip byte-for-byte, and form a FLOW_STMT.
//!
//! Reference: <https://docs.snowflake.com/en/sql-reference/operators-flow>

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
fn two_step_chain_parses_and_round_trips() {
    clean("SELECT a FROM t ->> SELECT b FROM $1");
    assert!(has(
        "SELECT a FROM t ->> SELECT b FROM $1",
        SyntaxKind::FLOW_STMT
    ));
}

#[test]
fn three_step_chain() {
    clean(
        "SELECT * FROM dept WHERE dname = 'SALES' \
         ->> SELECT * FROM emp WHERE deptno IN (SELECT deptno FROM $1) \
         ->> SELECT ename, sal FROM $1 ORDER BY 2 DESC",
    );
}

#[test]
fn dollar_reference_is_a_valid_table_ref() {
    clean("SELECT b FROM $1");
    clean("SELECT b FROM $2 AS prev");
}

#[test]
fn values_then_select_chain() {
    clean("VALUES (1), (2) ->> SELECT column1 FROM $1");
}

#[test]
fn a_lone_statement_is_not_wrapped_in_a_flow_stmt() {
    assert!(!has("SELECT a FROM t", SyntaxKind::FLOW_STMT));
}
