//! Subquery / CTE / set-op / quantified-comparison parsing. Every case parses with no diagnostics
//! and round-trips byte-for-byte; structural assertions pin the key node kinds.

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
fn scalar_subqueries_in_every_position() {
    clean("SELECT (SELECT MAX(x) FROM u) AS m FROM t");
    clean("SELECT a FROM t WHERE a > (SELECT AVG(b) FROM u WHERE u.k = t.k)");
    clean("SELECT a FROM t WHERE a BETWEEN (SELECT MIN(x) FROM u) AND (SELECT MAX(x) FROM u)");
    clean("SELECT a, COUNT(*) c FROM t GROUP BY a HAVING COUNT(*) > (SELECT AVG(c) FROM u)");
    clean("SELECT a FROM t ORDER BY (SELECT x FROM u LIMIT 1)");
    clean("SELECT * FROM a JOIN b ON a.id = (SELECT MAX(id) FROM c WHERE c.x = a.x)");
    clean("SELECT x + (SELECT SUM(y) FROM u) * 2 FROM t");
}

#[test]
fn in_exists_and_quantified() {
    clean("SELECT a FROM t WHERE a IN (SELECT id FROM u)");
    clean("SELECT a FROM t WHERE a NOT IN (SELECT id FROM u WHERE u.flag)");
    clean("SELECT a FROM t WHERE EXISTS (SELECT 1 FROM u WHERE u.id = t.id)");
    assert!(has(
        "SELECT a FROM t WHERE EXISTS (SELECT 1 FROM u)",
        SyntaxKind::EXISTS_EXPR
    ));

    for q in ["ALL", "ANY", "SOME"] {
        clean(&format!("SELECT a FROM t WHERE a > {q} (SELECT x FROM u)"));
    }
    clean("SELECT a FROM t WHERE a <> ANY (SELECT x FROM u)");
    clean("SELECT * FROM t WHERE name LIKE ANY ('a%', 'b%', 'c%')");
}

#[test]
fn derived_tables_and_set_ops() {
    clean("SELECT * FROM (SELECT * FROM (SELECT a FROM t) x) y");
    clean("SELECT * FROM (SELECT a FROM t UNION ALL SELECT a FROM u) x");
    clean("SELECT * FROM (SELECT a, b FROM t) x (c, d)");
    clean("(SELECT a FROM t ORDER BY a) UNION ALL (SELECT b FROM u ORDER BY b)");
    clean("SELECT a FROM t UNION ALL SELECT a FROM u EXCEPT SELECT a FROM v");
}

#[test]
fn with_clauses() {
    clean("WITH a AS (SELECT 1 AS n), b AS (SELECT n + 1 AS m FROM a) SELECT m FROM b");
    clean("WITH RECURSIVE r AS (SELECT 1 AS n UNION ALL SELECT n + 1 FROM r WHERE n < 10) SELECT n FROM r");
    clean("WITH RECURSIVE r (n) AS (SELECT 1 UNION ALL SELECT n + 1 FROM r WHERE n < 5) SELECT n FROM r");
    clean("WITH a (x, y) AS (SELECT 1, 2) SELECT x, y FROM a");
    clean("WITH a AS (VALUES (1), (2)) SELECT * FROM a");
    assert!(has(
        "WITH a AS (SELECT 1) SELECT * FROM a",
        SyntaxKind::WITH_QUERY
    ));
}

#[test]
fn nested_with_in_every_position() {
    clean("SELECT * FROM (WITH x AS (SELECT 1 AS n) SELECT n FROM x) y");
    clean("SELECT a FROM t WHERE a IN (WITH x AS (SELECT id FROM u) SELECT id FROM x)");
    clean("WITH a AS (WITH b AS (SELECT 1 AS n) SELECT n FROM b) SELECT n FROM a");
    clean("SELECT (WITH x AS (SELECT 1 AS n) SELECT MAX(n) FROM x) FROM t");
    clean("SELECT a FROM t WHERE a > ALL (WITH x AS (SELECT v FROM u) SELECT v FROM x)");
}

#[test]
fn deeply_nested_and_correlated() {
    clean("SELECT * FROM (SELECT * FROM (SELECT * FROM (SELECT a FROM t) x) y) z");
    clean("SELECT a FROM t WHERE a IN (SELECT id FROM u WHERE id IN (SELECT id FROM v WHERE id > (SELECT MIN(id) FROM w)))");
    clean("WITH RECURSIVE tree AS (SELECT id, parent FROM nodes WHERE parent IS NULL UNION ALL SELECT n.id, n.parent FROM nodes n JOIN tree t ON n.parent = t.id) SELECT * FROM tree WHERE id IN (SELECT id FROM active)");
}
