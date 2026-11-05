# Queue resume lane evidence

Counts are selected pass/fail/ignored. Each full log contains exact command, HEAD, dirty state, UTC times and exit. Compiler/owned runner logs remain at the paths printed in their parent lane. No ignored test was selected or executed; nextest libtest JSON also reports unselected ignored inventory entries.

|Full log under target/lane-evidence/|Exit|P/F/I|
|---|---:|---|
|queue-owner-preflight.log|0|0/0/0|
|queue1-same-directory-direct-unset.log|0|1/0/0|
|queue4-agent-context.log|0|0/0/0|
|queue4-boundary.log|1|0/0/0|
|queue4-envelope-check.log|101|0/0/0|
|queue4-envelope-format.log|0|0/0/0|
|queue4-envelope-public-basis-check.log|0|0/0/0|
|queue4-envelope-public-basis-format.log|0|0/0/0|
|queue4-line-caps.log|0|0/0/0|
|queue4-native-unit-check.log|0|0/0/0|
|queue4-owner-clippy.log|101|0/0/0|
|queue4-public-envelope-custody.log|0|1/0/0|
|queue4-scope-audit.log|0|0/0/0|

## Result limits and failed attempts

- queue1 is the exact direct same-directory test with RUST_MIN_STACK removed. Its1/0/0 result records non-reproduction atb8d9b28608, not a source fix or whole integration/default-stack qualification. Independent APPROVE is committed withd0061f1327.
- queue4-envelope-check failed at an old private branch_head call copied from the source court. The corrected public-basis check passes; it uses the actual admitted main-branch basis and changes no product API. This was a test integration error. Native unit target check also passes.
- queue4-public-envelope-custody passes1/0/0 on frozen5 source files. This is the one migrated test; no broader migration/test packet is claimed. Format, five-file line caps, context and scope/name-parity audit pass.
- Scoped owner Clippy exits101 with exactly10 unchanged master-changed sites/7paths below. These remain standing-rule3 skips; no new lint is reported in the candidate. It is not a green full Clippy result.
- Boundary exits1 with the exact3 still-visible BC9002 provider sites. ZeroBC9002 is NOT achieved. Full provider removal is blocked by the explicitly forbidden branch/root_* consumers; no baseline suppression or new Native observer is introduced.
- D13 and named-digest source changes/qualification are UNRUN: prerequisite stops are recorded in the separate queue2/3 handoffs. No integration merge/head was authorized.

## Actual Clippy skips

|Exact path:line|Lint|Rule|
|---|---|---|
|crates/worth-court-relational-certification/tests/worth_court_relational_certification/process/serial/relational/managed_source_work_account/public_only.rs:38|int_plus_one|3: locally unchanged; master differs from7d|
|crates/worth-court-relational-certification/tests/worth_court_relational_certification/process/serial/relational/managed_source_work_account.rs:292|int_plus_one|3: locally unchanged; master differs from7d|
|crates/worth-court-relational-certification/tests/worth_court_relational_certification/process/serial/relational/managed_source_work_account.rs:104|unnecessary_lazy_evaluations|3: locally unchanged; master differs from7d|
|crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/durable_publication/index_catalog/recovery.rs:109|needless_borrow|3: locally unchanged; master differs from7d|
|crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/durable_publication/index_catalog/recovery.rs:340|needless_borrow|3: locally unchanged; master differs from7d|
|crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/durable_publication/index_catalog.rs:169|needless_borrow|3: locally unchanged; master differs from7d|
|crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/durable_publication/index_catalog.rs:212|needless_borrow|3: locally unchanged; master differs from7d|
|crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/managed_source/nested.rs:110|err_expect|3: locally unchanged; master differs from7d|
|crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/managed_source/row_read.rs:48|err_expect|3: locally unchanged; master differs from7d|
|crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/managed_source/owner.rs:267|collapsible_if|3: locally unchanged; master differs from7d|
