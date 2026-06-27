; Indentation query for Snowflake SQL.
;
; The grammar is deliberately permissive, so indentation is anchored on the
; balanced nodes it does expose: calls and parenthesized expressions. Editors
; that understand Tree-sitter's common @indent/@dedent captures can use this for
; argument lists, grouped queries, and clause option lists without claiming a
; full formatter.

[
  (argument_list)
  (parenthesized_expression)
] @indent

")" @dedent
