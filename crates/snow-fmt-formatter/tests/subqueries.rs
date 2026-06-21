//! Exhaustive subquery / CTE / set-operation / quantified-comparison coverage.
//!
//! Every case below is asserted to (1) parse with no errors, (2) format to valid SQL, (3) be
//! idempotent, and (4) preserve its meaningful tokens (formatting only changes trivia and keyword
//! casing). The matrix deliberately crosses *position* (SELECT scalar, WHERE, HAVING, ORDER BY,
//! QUALIFY, JOIN ON, FROM, IN, EXISTS, quantifier, arithmetic, CASE) with *shape* (correlated,
//! nested, set-op body, WITH/RECURSIVE/column-list, value list) so regressions surface here.

use snow_fmt_formatter::format;
use snow_fmt_lexer::{tokenize, SyntaxKind};
use snow_fmt_parser::parse;

/// The signature a faithful formatter must preserve: meaningful tokens, upper-cased, with the
/// synthesized `;` dropped.
fn signature(sql: &str) -> Vec<String> {
    tokenize(sql)
        .tokens
        .into_iter()
        .filter(|t| !t.kind.is_trivia() && t.kind != SyntaxKind::SEMICOLON)
        .map(|t| t.text.to_ascii_uppercase())
        .collect()
}

const CASES: &[&str] = &[
    // scalar subquery in every position
    "select (select max(x) from u) from t",
    "select (select max(x) from u) as m from t",
    "select a, (select count(*) from u where u.id = t.id) c from t",
    "select x + (select sum(y) from u) * 2 from t",
    "select case when a > (select avg(b) from u) then (select 1) else (select 0) end from t",
    "select coalesce((select x from u limit 1), 0) from t",
    "select a from t where a > (select avg(b) from u)",
    "select a from t where (select count(*) from u) > 0",
    "select a from t where a > (select avg(b) from u where u.k = t.k)",
    "select a from t where a between (select min(x) from u) and (select max(x) from u)",
    "select a, count(*) c from t group by a having count(*) > (select avg(c) from u)",
    "select a from t order by (select x from u limit 1)",
    "select a from t qualify row_number() over (order by a) <= (select k from cfg)",
    "select * from a join b on a.id = (select max(id) from c where c.x = a.x)",
    // IN / EXISTS / NOT, correlated
    "select a from t where a in (select id from u)",
    "select a from t where a not in (select id from u where u.flag)",
    "select a from t where exists (select 1 from u where u.id = t.id)",
    "select a from t where not exists (select 1 from u where u.id = t.id)",
    "select a from t where a in (select id from u) and b not in (select id from v)",
    // quantified ALL / ANY / SOME, LIKE ANY
    "select a from t where a > all (select x from u)",
    "select a from t where a = any (select x from u)",
    "select a from t where a < some (select x from u)",
    "select a from t where a >= all (select x from u where u.k = t.k)",
    "select a from t where a <> any (select x from u)",
    "select * from t where name like any ('a%', 'b%', 'c%')",
    "select * from t where x not in (select id from u) and y = some (select z from v)",
    // derived tables / joins / set ops inside / column-aliased
    "select * from (select a from t) x",
    "select * from (select a from t)",
    "select * from (select * from (select a from t) x) y",
    "select * from a join (select id from b) sub on a.id = sub.id",
    "select * from (select a from t union all select a from u) x",
    "select * from (select a from t) x join (select b from u) y on x.a = y.b",
    "select * from (select a from t order by a limit 10) x",
    "select * from (select a, b from t) x (c, d)",
    // set operations + parens
    "select a from t union select a from u",
    "select a from t union all select a from u except select a from v",
    "(select a from t) union (select a from u)",
    "select a from t intersect select a from u",
    "select a from t minus select a from u",
    "(select a from t order by a) union all (select b from u order by b)",
    // WITH variations
    "with a as (select 1 as n) select n from a",
    "with a as (select 1 as n), b as (select 2 as m) select n, m from a, b",
    "with a as (select 1 as n), b as (select n + 1 as m from a) select m from b",
    "with a as (select 1 as n), b as (select 2 as m), c as (select 3 as p) select * from a, b, c",
    "with recursive r as (select 1 as n union all select n + 1 from r where n < 10) select n from r",
    "with recursive r (n) as (select 1 union all select n + 1 from r where n < 5) select n from r",
    "with a (x, y) as (select 1, 2) select x, y from a",
    "with a as (select 1 union all select 2) select * from a",
    "with a as (values (1), (2)) select * from a",
    "with a as (select 1 as n) select n from a union all select 2",
    "with a as (select 1 as n) (select n from a) union all (select n from a)",
    // nested WITH (every position)
    "select * from (with x as (select 1 as n) select n from x) y",
    "select a from t where a in (with x as (select id from u) select id from x)",
    "with a as (with b as (select 1 as n) select n from b) select n from a",
    "select (with x as (select 1 as n) select max(n) from x) from t",
    "select a from t where a > all (with x as (select v from u) select v from x)",
    // deep nesting / monsters
    "select * from (select * from (select * from (select a from t) x) y) z",
    "select a from t where a in (select id from u where id in (select id from v where id > (select min(id) from w)))",
    "with a as (select id, v from t where v > (select avg(v) from t)) select id from a where id in (select id from u) order by id",
    "select t.a, (select count(*) from u where u.t_id = t.id and u.v > all (select v from w)) from t",
    "with recursive tree as (select id, parent from nodes where parent is null union all select n.id, n.parent from nodes n join tree t on n.parent = t.id) select * from tree where id in (select id from active)",
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
        let errors = parse(&formatted).errors().to_vec();
        assert!(
            errors.is_empty(),
            "formatted output is invalid for {sql:?}: {errors:?}\n---\n{formatted}"
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
            "token sequence changed:\n{sql}\n---\n{formatted}"
        );
    }
}
