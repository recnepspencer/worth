# Vendored CEL conformance cases

These `.textproto` files are copied unmodified from
[google/cel-spec](https://github.com/google/cel-spec) at commit
`40a3c9007a9305d1f2638d5f538a4e011732cace` (2026-09-16), path
`tests/simple/testdata/`, under the Apache License 2.0 in `LICENSE`.

`classification.tsv` assigns every case in these files one disposition:

- `equivalent`: the CEL source is also Worth expression source, and it admits
  and evaluates to the value CEL expects.
- `divergent`: the source admits in both languages, but Worth deliberately
  produces a different outcome. The row names the governing rule.
- `unsupported`: the source uses CEL syntax, types, or functions that Worth
  expressions V1 does not have, and it is denied at parse or admission.

`tests/expressions/cel.rs` re-derives each disposition and fails when a case
is missing from the table or its recorded disposition no longer holds.
