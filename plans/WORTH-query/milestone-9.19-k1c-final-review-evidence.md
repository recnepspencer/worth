# K1c final review evidence

This ledger records the new certification repair attempts after4adcea7b55.
Long Cargo commands run hidden with logs/exit receipts under
target/k1c-repair-logs. Python uses PYTHONUTF8=1. Query builds use the prescribed
all-ones implementation build ID. No Store binary is authorized or run.

Initial sync fetch and both explicitly messaged merges succeeded; both sync
branches were already current. The supplied independent Execution log reports
2230 passed,8 failed,1 ignored,308.20s. This is external baseline evidence,
not a claim of an agent-run full baseline. All eight failures remain in scope.

## Finding 2: production custody admission

Initial `cargo fmt -p worth-relational`: exit 0. Initial focused Native run
`cargo test -p worth-relational --lib storage::residency::kind_admission::tests:: -- --nocapture`:
4 passed, 0 failed/ignored/measured, 2274 filtered, 0.00s (compile 1m10s).
The subsequent fixture refinement replaces an invented fallback cause with a
loud positive-fixture assertion; no production refusal is changed.

`cargo fmt -p worth-relational` rerun: exit 0. Compiled mutant changes the real
admit prior selection to `prior.filter(|_| false)`. Exact private custody test:
0 passed, 1 failed, 2277 filtered, 0.00s (compile 2m04s), exit 101. Failure is
literal allocation identity inequality at kind_admission_tests.rs:74; a fresh
allocation cannot masquerade as growth of the original ticket. Mutation script
restored source byte-exactly and refreshed its timestamp (receipt exit 0).

Restored focused command: 4 passed, 0 failed/ignored/measured, 2274 filtered,
0.00s (compile 1m43s), exit 0. `cargo fmt --check -p worth-relational`: exit 0.
`cargo clippy -p worth-relational --all-targets`: exit 101, one excluded error,
`returning the result of a let binding from a block` at
mvcc/retained_entity_selection/payload.rs:34, and eight warnings. No action per
ruling; four independent lint-file scope assertions pass, excluding dirty overlap.
Boundary-check exit 0, repository boundaries valid. Agent-context check exit 0.
Dirty line caps exit 0, 4 Rust files checked. Both `git diff --check
origin/master...HEAD` and `git diff --check` exit 0. No full binary is claimed
for this intermediate slice; all four remain required after final source.
`/root/reviewer` independently passed the staged slice under qa-loop, qa-tests and code-quality-qa; no remaining in-scope finding.
