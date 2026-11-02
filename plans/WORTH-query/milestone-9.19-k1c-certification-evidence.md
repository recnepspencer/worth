# K1c resumed certification evidence

This report records the resumed repair checks, including unsuccessful runs.
The earlier filtered closeout does not qualify the full binaries. Commands run
from the lane worktree; Query commands use its explicit manifest. Long Cargo
commands run hidden in the background with logs and exit receipts under
`target/k1c-repair-logs`. Python checks set `PYTHONUTF8=1`; Query court/build
commands set `WORTH_IMPLEMENTATION_BUILD_ID=11111111-1111-1111-1111-111111111111`.
Store test binaries are excluded by the revised brief.

## Baseline discovery

| Command | Exact result |
| --- | --- |
| `cargo test -p worth-relational --lib` | 2,232 passed; nine failed; 31 ignored; zero measured; zero filtered; 461.90 seconds. |
| `cargo test -p worth-court-relational-certification --test worth_court_relational_certification` | Started after R1 edits; compilation failed on ten missing residency Content arms; no tests executed. This was not an untouched baseline court run. |

The Native failures reproduce all nine names in the resumed brief. Representative
literal output: work-account `left: 268`, `right: 658`; empty commit work refusal
`left: NestedStopped`, `right: WorkExhausted`; rematerialization staging
`ContentAdmission { denial: EncodingInconsistency }`. The other six failures are
materialization-root recovery, forged kind authentication, both named managed
field-text recovery cases, exact checkpoint allocation, and unfunded acyclicity.
They remain required repairs; no expectation has been changed to hide them.

## R1: exact cold-envelope causes

Native owns the semantic rejection vocabulary and its conversion into execution
and recovery carriers. Query preserves the exact payload through its existing
denial facade. No aspect shape is added. A single closed variant declaration
keeps flat content-denial spellings and nonrecursive rejection payloads aligned.

`content_cold_envelope_preserves_a_literal_version_refusal_without_poisoning_history`
uses an actual external checkpoint and older journal body. The court changes the
content-envelope version, independently repairs the outer artifact and index
digest, proves the body was cold, and demands it twice through public history.
Both reads must return literal `Content(UnsupportedVersion)` and leave the
selected descriptor unchanged. Its literal refusal and test-side physical
MessagePack/SHA framing are independent of the content decoder and cause mapper.
The pure conversion family separately checks every semantic case, nontrivial
payloads and unchanged execution causes. Query's existing literal receiving
family adds an exact SizeLimit payload case.

| Command or check | Completed attempts and exact result |
| --- | --- |
| `git fetch origin master codex/919-provider-lifecycle-20261027` | One success. |
| `git merge --no-edit origin/master` | One success; already up to date. |
| `git merge --no-edit origin/codex/919-provider-lifecycle-20261027` | One success; already up to date. |
| `cargo fmt -p worth-relational -p worth-court-relational-certification` | Two successes. |
| `cargo fmt --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution -p worth-query-publication` | Two successes. |
| `cargo test -p worth-court-relational-certification --test worth_court_relational_certification content::cold_envelope:: -- --nocapture` | First compile: ten non-exhaustive residency receivers; no tests. After correction: one passed, zero failed/ignored/measured, 210 filtered. |
| `cargo test -p worth-relational --lib semantic_content_causes_keep_literal_payloads_and_execution_keeps_its_original_cause -- --nocapture` | One passed, zero failed/ignored/measured, 2,272 filtered. |
| `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --lib content_tests -- --nocapture` | First compile failed: `E0432`, missing primary-graph Rejection re-export; no tests. Corrected run: two passed, zero failed/ignored/measured, 2,237 filtered. |
| `cargo check --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-publication --all-targets` | One success; compile check, no tests. |
| First attempted cold-court mutation | Invalid experiment: five `bare CR not allowed in doc-comment` compilation errors from a script newline conversion; no tests. This is not a killed mutant. Script corrected and source restored. |
| Restored cold court after invalid experiment | One passed, zero failed/ignored/measured, 210 filtered; rebuilt from restored source. |
| `cargo test -p worth-court-relational-certification --test worth_court_relational_certification content:: -- --nocapture` | 62 passed, zero failed/ignored/measured, 149 filtered. |
| Corrected cold-court mutation with the same focused Cargo command | Compiled mutant: zero passed, one failed, zero ignored/measured, 210 filtered. Actual `StorageUnavailable { subject: Envelope(PatchStreamPosition(1)), cause: Damaged }`; expected `Content(UnsupportedVersion)`. Killed for the intended literal mismatch. |
| Restored cold court after corrected mutation | One passed, zero failed/ignored/measured, 210 filtered; source restored with a fresh timestamp and target rebuilt. |
| First converter mutation attempt with the focused pure-unit command above | Invalid experiment: one `error: unreachable pattern` at `content/denial.rs:27`; no tests. A duplicate special-case arm was rejected by the crate's deny rule. Source restored; unit rebuilt and passed one, zero failed/ignored/measured, 2,272 filtered. Not a killed mutant. |
| Second converter harness dispatch and direct diagnostic | Dispatch produced no Cargo receipt. Direct diagnostic reproduced an installation guard `AssertionError` because the macro pattern occurs twice; exit one, no source edit and no tests. Target narrowed to the forward conversion; installation now has explicit logs/exit receipts. |
| Corrected converter mutation with the focused pure-unit command above | Compiled mutant: zero passed, one failed, zero ignored/measured, 2,272 filtered. Actual `Content(SizeLimit { observed: 96, maximum: 31 })`; literal expected observed 97. Killed for the intended payload loss. |
| Restored pure conversion court after corrected mutation | One passed, zero failed/ignored/measured, 2,272 filtered; original bytes restored with a fresh timestamp and target rebuilt. |
| `cargo fmt --check -p worth-relational -p worth-court-relational-certification` | One success; zero formatting differences. |
| `cargo fmt --check --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution -p worth-query-publication` | One success; zero formatting differences. |
| `cargo clippy -p worth-relational -p worth-court-relational-certification --all-targets` | Failed on one untouched-file lint: `mvcc/retained_entity_selection/payload.rs:34`, `returning the result of a let binding from a block`. Eight warnings; court linting was not reached. |
| `cargo clippy --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution -p worth-query-publication --all-targets` | One success. |
| `cargo clippy -p worth-court-relational-certification --all-targets --no-deps` | Four errors in three untouched court files, two warnings: `managed_source_work_account/public_only.rs:38` and `managed_source_work_account.rs:292` unnecessary `<= EXTRA - 1`; `managed_source_work_account.rs:104` unnecessary closure in `bool::then`; `relational/managed_source/nested.rs:110` `.err().expect()`. Zero error locations overlap dirty files. |
| `cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .` | Two successes: repository boundaries valid; tool supplies no case count. |
| `cargo run --manifest-path tools/agent-context/Cargo.toml -- check` | Two successes: exit zero; tool supplies no case count. |
| `bash scripts/ci/check_workspace_rust_line_caps.sh dirty` | Two successes: 36 Rust files each. |
| `git diff --check origin/master...HEAD` | Three successes: zero whitespace errors. |
| `git diff --check` | Three successes: zero whitespace errors. |
| `git diff --cached --check` | One success after staging all 38 R1 files; zero whitespace errors. |
| `python target/k1c-repair-logs/r1-verify-restored.py` | Two runs; one exact restored-source equality assertion passed each time. |
| Converter harness restoration checks | One exact byte-equality assertion passed after each of the invalid-pattern and corrected compiled mutations. |
| `python target/k1c-repair-logs/r1-lint-scope.py` | Four untouched lint-file scope assertions passed. |

The lint failures above are existing untouched-file debt under AGENTS.md's scoped
dirty-set rule. No lint was suppressed and no unrelated source was changed.
The nine explicitly named full-binary failures remain in scope independently of
that rule and will be repaired before final qualification.

The same independent reviewer passed R1 under qa-loop, qa-tests and
code-quality-qa. It found no code defect and required the missing mutation for
the new pure conversion court; the compiled payload mutation, exact restoration
and rebuilt passing unit closed that gap. It independently inspected both valid
mutation failures, restored sources and passing rebuilds. Approval covers R1's
separate commit; full-binary qualification remains required later.

## R2: historical admission is unavailable, not missing

Native's historical publication owner now refuses `HistoryNotResident` when its
admitted directory is absent. Actual directory lookup still owns MissingContent.
The new semantic case is appended, preserving existing serialized rejection tags.
No AspectShape or generic engine body is added.

`content_cold_history_is_unavailable_while_a_genuine_dangling_reference_is_missing`
composes existing public boundaries: an external checkpoint reopens without
reading an older journal body, counted cold artifact demand reads it once, and
controlled artifact readmission produces a decoded envelope whose field-content
read returns literal HistoryNotResident. This does not claim historical-directory
reconstruction. Current admitted bytes remain readable, while an actual dangling
field write refuses literal MissingContent and preserves the selected descriptor.
The refusal literals and external source read counter are independent of the
availability decision. The existing transport and conversion courts are extended.

| Command or check | Exact result |
| --- | --- |
| `git fetch origin master codex/919-provider-lifecycle-20261027` | One success. |
| `git merge --no-edit origin/master` | One success; already up to date. |
| `git merge --no-edit origin/codex/919-provider-lifecycle-20261027` | One success; already up to date. |
| `cargo fmt -p worth-relational -p worth-court-relational-certification` | Two successes. |
| `cargo test -p worth-court-relational-certification --test worth_court_relational_certification content::cold_envelope:: -- --nocapture` | First compile: one E0599, physical artifact has no envelope method, no tests. Corrected run through readmit: two passed, zero failed/ignored/measured, 210 filtered. |
| `cargo test -p worth-court-relational-certification --test worth_court_relational_certification content::historical_custody:: -- --nocapture` | First compile failed on the same E0599, no tests. Corrected run: two passed, zero failed/ignored/measured, 210 filtered. |
| `cargo test -p worth-relational --lib semantic_content_causes_keep_literal_payloads_and_execution_keeps_its_original_cause -- --nocapture` | Two successful runs: one passed, zero failed/ignored/measured, 2,272 filtered each. |
| `cargo test -p worth-court-relational-certification --test worth_court_relational_certification content_cold_history_is_unavailable_while_a_genuine_dangling_reference_is_missing -- --nocapture` | Compiled mutation restoring MissingContent: zero passed, one failed, zero ignored/measured, 211 filtered. Literal actual MissingContent versus expected HistoryNotResident. Killed. |
| `cargo test -p worth-court-relational-certification --test worth_court_relational_certification content:: -- --nocapture` | Restored source rebuilt: 63 passed, zero failed/ignored/measured, 149 filtered. |
| `cargo fmt --check -p worth-relational -p worth-court-relational-certification` | One success. |
| `cargo clippy -p worth-relational -p worth-court-relational-certification --all-targets` | One untouched-file let-and-return error at payload.rs:34, eight warnings, same diagnostic as R1. |
| `cargo clippy -p worth-court-relational-certification --all-targets --no-deps` | The same four untouched errors in three files and two warnings listed under R1. Zero dirty overlap. |
| `cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .` | One success; repository boundaries valid. |
| `cargo run --manifest-path tools/agent-context/Cargo.toml -- check` | One success; exit zero. |
| `bash scripts/ci/check_workspace_rust_line_caps.sh dirty` | One success; six Rust files. |
| `git diff --check origin/master...HEAD` | One success; zero whitespace errors. |
| `git diff --check` | One success; zero whitespace errors. |
| Mutation restoration Python check, with PYTHONUTF8=1 | One exact original-byte equality assertion passed; source timestamp refreshed before rebuild. |
| `python target/k1c-repair-logs/r1-lint-scope.py`, with PYTHONUTF8=1 | Four untouched lint-file scope assertions passed. |

The same reviewer found a shifted-tag risk from inserting the new enum case; it
closed by appending the case. It approved the corrected compositional cold proof
and directory distinction, inspected the compiled mutation and restored 63-test
run, and passed R2 under all three QA skills. No R2 findings remain open.
