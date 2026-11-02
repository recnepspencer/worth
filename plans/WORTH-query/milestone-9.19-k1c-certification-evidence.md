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

The same reviewer passed R7's separate repair under qa-loop, qa-tests and
code-quality-qa after inspecting final source and receipts. The public failure
above remains active for the immediate R5 follow-up; it was neither excluded as
debt nor called a passing public portfolio. Final working-tree whitespace
passed twice and staged whitespace passed once, with zero errors.

The same reviewer found a shifted-tag risk from inserting the new enum case; it
closed by appending the case. It approved the corrected compositional cold proof
and directory distinction, inspected the compiled mutation and restored 63-test
run, and passed R2 under all three QA skills. No R2 findings remain open.

## R3: refuse dead top-level content at installation

The real entity/relation registration matrix expects literal
`InvalidAspectDeclaration { kind_id: KindId(42), detail: "content references require a format-bound struct field" }`.
Its oracle is the specified literal, independently of declaration validation.
Successful integer registration and the real two-field content fixture protect
the lawful neighbors. No AspectShape or additional runtime door was added.

| Exact command or experiment | Result |
| --- | --- |
| `git fetch origin master codex/919-provider-lifecycle-20261027`; `git merge --no-edit origin/master`; `git merge --no-edit origin/codex/919-provider-lifecycle-20261027` | All three succeeded; both merges already up to date. |
| `cargo fmt -p worth-relational -p worth-court-relational-certification` | One success. |
| `cargo test -p worth-court-relational-certification --test worth_court_relational_certification content_install_refuses_both_unbound_top_level_forms_on_both_kind_doors -- --nocapture` | Baseline: 1 passed, 212 filtered. Compiled mutant: 0 passed, 1 failed, 212 filtered; scalar installation incorrectly returned Ok at schema_install.rs:58. |
| Python mutation installation/restoration, with PYTHONUTF8=1 | Both succeeded; one exact restored-byte equality assertion passed, timestamp refreshed. |
| `cargo test -p worth-court-relational-certification --test worth_court_relational_certification content:: -- --nocapture` | First restored run: 63 passed, 1 failed, 149 filtered. The old raw_write court panicked installing the now-deleted scalar form. Retired that court in favor of the install matrix. Final restored run: 63 passed, 0 failed, 149 filtered. Both runs had 0 ignored/measured. |
| `cargo fmt --check -p worth-relational -p worth-court-relational-certification` | Two successes. |
| `cargo clippy -p worth-relational -p worth-court-relational-certification --all-targets` | Initial failure: 2 errors, 8 warnings. Scoped reconstruction.rs:99 single_match fixed to if-let. Untouched payload.rs:34 let_and_return remains repository debt. Final failure is detailed below. |
| `cargo clippy -p worth-court-relational-certification --all-targets --no-deps` | Failed on the same 4 untouched court lint errors in 3 files, 2 warnings, as R1/R2. |
| `cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .` | One success; repository boundaries valid. |
| `cargo run --manifest-path tools/agent-context/Cargo.toml -- check` | One success. |
| `bash scripts/ci/check_workspace_rust_line_caps.sh dirty` | Two successes: 4 Rust files each. |
| `git diff --check origin/master...HEAD`; `git diff --check` | Both succeeded with zero whitespace errors. |
| `python target/k1c-repair-logs/r1-lint-scope.py`, with PYTHONUTF8=1 | Four untouched lint-file scope assertions passed. |

The same reviewer passed R3 under qa-loop, qa-tests and code-quality-qa after independently inspecting the install boundary, exact matrix, retired dead-form court, field checkpoints, compiled mutation and final receipts. No R3 findings remain.

The final Native/court Clippy rerun failed only on untouched payload.rs:34
`let_and_return` (1 error, 8 warnings). The scoped single_match failure is closed.
Line caps passed twice, checking four Rust files each time. Branch whitespace
checks passed twice; working-tree whitespace checks passed four times, all with
zero whitespace errors. These repetitions include the final source correction.

## R4: one owner of exact frame equality

Query's byte comparison loop and its reexports are deleted. The existing actual
request receivers supply erased genuine-frame callbacks to Native same_content;
no actual caller has a Source read budget, so no unused Native Source API was
added. Native's exact denial conversion now serves the receiving boundaries.
The two removed raw-byte helper tests belonged to the deleted duplicate checker.

The new pure latch family injects literal Cancelled, WorkExhausted,
ContentIdentityConflict and SizeLimit(97,31), checks both hostile boolean results,
one callback contact and no final poll after refusal. It tests production's
first-error mechanism, without claiming Native admission in a product-src test.
The new pure receiver matrix independently expects literal final categories and
exact Native causes for cancellation, deadline, work, conflict and size. Actual
Native content/Source integration remains in the external public content court.

| Exact command or experiment | Result |
| --- | --- |
| `git fetch origin master codex/919-provider-lifecycle-20261027`; `git merge --no-edit origin/master`; `git merge --no-edit origin/codex/919-provider-lifecycle-20261027` | All three succeeded; both merges already up to date. |
| `cargo fmt -p worth-relational` | Two successes. |
| `cargo fmt --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution` | Two successes. |
| `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --lib content_comparison:: -- --nocapture` | Initial compile failed: one E0004 in application_checkpoint/header.rs for the new Content retention arm, two execution-test warnings; no tests ran. Added the exact Native execution receiver. |
| `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --lib comparison:: -- --nocapture` | Baseline and restored-source runs: 19 passed, 0 failed/ignored/measured, 2220 filtered each. |
| `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --lib native_comparison_refusals_keep_literal_attempt_categories_and_causes -- --nocapture` | Baseline and restored-source runs: 1 passed, 0 failed/ignored/measured, 2238 filtered each. Mutant: 0 passed, 1 failed, 2238 filtered; actual cause None versus literal Some(Cancelled). |
| `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --lib erased_comparison_latches_literal_native_refusals_before_hostile_boolean_results -- --nocapture` | Compiled mutant: 0 passed, 1 failed, 0 ignored/measured, 2238 filtered; literal panic `no poll after refusal` at comparison/tests.rs:129. |
| Paired mutation Python install/restore, with PYTHONUTF8=1 | Two pre-edit anchor assertions and two exact restored-byte assertions passed; both timestamps refreshed. The two pure tests have disjoint owner call graphs. Both mutants shared one build, each test ran separately, then both originals were restored and rebuilt. The reviewer approved this attribution and compile saving. |
| `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-court-decl-host --test worth_query_court_decl_host content_door:: -- --nocapture` | 21 passed, 0 failed/ignored/measured, 327 filtered. This is K1c's actual public content owner; the separately existing query-decl-host aggregate has no content_door. |
| `cargo test -p worth-relational --lib semantic_content_causes_keep_literal_payloads_and_execution_keeps_its_original_cause -- --nocapture` | 1 passed, 0 failed/ignored/measured, 2272 filtered. |
| Debug rlib measurement, same artifact/profile | libworth_query_execution-b7016cf434752ce1.rlib: 319,059,810 before, 319,335,294 after; increase 275,484 bytes (about 0.086%). This is an artifact-size observation, not isolated monomorphization attribution. The frame engine is concrete in Native; new generic failure-policy bodies are small. |

The reviewer found callback/stack name shadowing, orphaned cfg(test) attributes
after deleting reexports, inappropriate runtime construction in the first test
draft, and receiver classification that would hide original execution causes.
The corrected traversal names, production reexports, pure latch, NativeExecution
normalization and normal-path MaterialStorage mapping close those source
findings. The same reviewer passed R4 under qa-loop, qa-tests and code-quality-qa after inspecting the final source, compiled mutations, exact restoration, rebuilt tests, external court, required receipts and rlib measurement. No R4 findings remain.

| Final exact command | Result |
| --- | --- |
| `cargo fmt --check -p worth-relational` | One success. |
| `cargo fmt --check --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution` | One success. |
| `cargo clippy -p worth-relational --all-targets` | Failed solely on untouched payload.rs:34 let_and_return, 1 error and 8 warnings; no scoped lint error. |
| `cargo clippy --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --all-targets` | One success. |
| `cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .` | One success; repository boundaries valid. |
| `cargo run --manifest-path tools/agent-context/Cargo.toml -- check` | One success. |
| `bash scripts/ci/check_workspace_rust_line_caps.sh dirty` | One success; 24 Rust files. |
| `git diff --check origin/master...HEAD`; `git diff --check` | One success each; zero whitespace errors. |
| `python target/k1c-repair-logs/r1-lint-scope.py`, with PYTHONUTF8=1 | Four untouched lint-file scope assertions passed. |
# R5: staged attachments must belong to final candidate closure

The new external transaction court uses an independently frozen frame digest,
not the writer's returned identity, as its refusal oracle. A consumed failed
transaction releases the original-funded staging and candidate scratch, and the
entire selected descriptor stays unchanged. Its positive transactions also prove
that a staged duplicate referenced by an unchanged prior record remains legal.
The final directory owns the paid traversal; reconstruction reuses it without
changing its existing NonCanonicalContent classification. No AspectShape was added.

| Exact command or experiment | Result |
| --- | --- |
| `git fetch origin master codex/919-provider-lifecycle-20261027`; `git merge --no-edit origin/master`; `git merge --no-edit origin/codex/919-provider-lifecycle-20261027` | All three succeeded; both merges already up to date. |
| `cargo fmt -p worth-relational -p worth-court-relational-certification` | Two successes. |
| `cargo test -p worth-court-relational-certification --test worth_court_relational_certification unreferenced_attachment_refuses_with_literal_identity_and_releases_custody -- --nocapture` | Initial compile: three E0433 errors for the same missing RelationalTransactionIntent import, two warnings; no tests ran. Corrected baseline: 1 passed, 212 filtered. Compiled guard-deletion mutant: 0 passed, 1 failed, 212 filtered; unwrap_err observed an incorrectly successful publication. Both executed runs had 0 ignored/measured. |
| Python mutation install/restore, with PYTHONUTF8=1 | One pre-edit anchor and one byte-exact restored-source assertion passed; restored timestamp refreshed. |
| `cargo test -p worth-court-relational-certification --test worth_court_relational_certification content:: -- --nocapture` | Restored source: 64 passed, 0 failed/ignored/measured, 149 filtered, 26.80 seconds. The local-directory court measured n=1,000 read 29,272/edit 275,431 and n=10,000 read 29,784/edit 283,639; its independent growth bounds passed. |

Administrative receipts from prior committed slices: R2, R3 and R4 each also
ran `git diff --cached --check` successfully once before their commit, with zero
whitespace errors. These checks supplement their recorded branch/dirty checks.

| R5 final exact command | Result |
| --- | --- |
| `cargo fmt --check -p worth-relational -p worth-court-relational-certification` | One success. |
| `cargo clippy -p worth-relational -p worth-court-relational-certification --all-targets` | Failed solely on untouched payload.rs:34 let_and_return: 1 error, 8 warnings. |
| `cargo clippy -p worth-court-relational-certification --all-targets --no-deps` | Failed on the same 4 untouched errors in 3 court files, 2 warnings, recorded under R1. |
| `cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .` | One success; repository boundaries valid. |
| `cargo run --manifest-path tools/agent-context/Cargo.toml -- check` | One success. |
| `bash scripts/ci/check_workspace_rust_line_caps.sh dirty` | One success; 7 Rust files. |
| `git diff --check origin/master...HEAD`; `git diff --check` | One success each; zero whitespace errors. |
| `cargo test -p worth-relational --lib semantic_content_causes_keep_literal_payloads_and_execution_keeps_its_original_cause -- --nocapture` | 1 passed, 0 failed/ignored/measured, 2272 filtered. |
| `python target/k1c-repair-logs/r1-lint-scope.py`, with PYTHONUTF8=1 | Four untouched lint-file scope assertions passed. |

The reviewer independently checked final-phase placement, literal identity,
unchanged-record acceptance, consumed-transaction memory release, the compiled
mutation and exact restoration. The missing import was corrected before tests.
The same reviewer passed R5 under qa-loop, qa-tests and code-quality-qa; no R5
finding remains. Working-tree whitespace passed twice and staged whitespace
passed once, all with zero errors.
# R6: require and digest the content authority block

Both actual serialized owners require content_formats and emit its block even
when empty. The preceding positional slots are explicit (seven snapshot fields,
ten carrier fields). extension.rs has no serde field; the second actual owner
is durability/data/branch_root_schema_images.rs. The older
projected_member_domains serde defaults remain existing debt, as requested;
older derived defaults were also left outside this repair.

The independent Python oracle specifies the digest domains, empty u64 counts
and MessagePack tokens manually, without loading Rust output. It freezes empty
authority d88918c5d7c1c31bc3df72d45eac0c683488dbff018bca1726e03b7004685707
and carrier f3821900deb7ea73bd91b4777899c003a4c5218a571909352421a99a497246b2.
The existing pure families also use literal 92 90 90 as the empty content tail,
remove it from valid compact envelopes, and remove the named JSON field.

Harness failures are not evidence: the filename filter
`root_schema_content_tests::` selected zero tests in baseline, compiled mutant
and restored runs (2273 filtered each). The first attempted module correction,
`branch::root_schema::content_tests::`, also selected zero in all three runs.
The reviewer missed the parent path alias as well and corrected that statement
after independently tracing both mounts. The compiled binary's `--list`
identified `branch::root::schema::content_tests::`; the corrected harness refuses
to install a mutant unless its baseline reports exactly four passing tests.
Neither zero-selected mutant is credited as a kill.

The reviewer's earlier one-line Python oracle attempt exited 1 with
`SyntaxError: unexpected character after line continuation character` at
`<string>`, line 1 because Windows argument handling stripped its quotes. Its
stdin retry succeeded. The durable .py oracle rerun uses PYTHONUTF8=1 and passed
three literal hash assertions. No Rust test or production file was involved in
that Python quoting failure.

| R6 exact command or experiment | Result |
| --- | --- |
| `git fetch origin master codex/919-provider-lifecycle-20261027`; `git merge --no-edit origin/master`; `git merge --no-edit origin/codex/919-provider-lifecycle-20261027` | All three succeeded; both merges already up to date. |
| `cargo fmt -p worth-relational` | One success. |
| `python target/k1c-repair-logs/r6-digest-oracle.py`, with PYTHONUTF8=1 | Three independent hash assertions passed. |
| `cargo test -p worth-relational --lib root_schema_content_tests:: -- --nocapture` | Three invalid coverage attempts: 0 passed/failed/ignored/measured, 2273 filtered each; no tests selected. |
| `cargo test -p worth-relational --lib branch::root_schema::content_tests:: -- --nocapture` | Three further invalid coverage attempts: 0 passed/failed/ignored/measured, 2273 filtered each; no tests selected. |
| `C:/worth-build-cache/target-cut-k1c-content/debug/deps/worth_relational-bc0d68c1db642786.exe --list` (output filtered for content_schema_/content_recorded_declarations) | Listed all four actual branch::root::schema::content_tests test names. This was discovery, not a passing test run. |
| `cargo test -p worth-relational --lib branch::root::schema::content_tests:: -- --nocapture` | Correct baseline: 4 passed, 0 failed/ignored/measured, 2269 filtered. Compiled paired mutant: 1 passed, 3 failed, 0 ignored/measured, 2269 filtered. The authority literal differed at line 70; snapshot/carrier unwrap_err observed wrongly accepted missing fields at lines 159/198. |
| Python paired mutation installation/restoration, with PYTHONUTF8=1 | Three installations and three restorations succeeded across the two invalid sequences and the valid sequence. Each restoration checked both files byte-for-byte and refreshed both timestamps: six exact equality assertions total. Only the verified four-test sequence supplies mutation evidence. |
| `cargo test -p worth-relational --lib branch::root::schema::content_tests:: -- --nocapture` after exact restoration | 4 passed, 0 failed/ignored/measured, 2269 filtered; fresh rebuild 1m08s, tests 0.00s. |
| `cargo fmt --check -p worth-relational` | Three successes, including the final restored source. |
| `cargo clippy -p worth-relational --all-targets` | Three failures, each solely untouched payload.rs:34 let_and_return, 1 error and 8 warnings; no scoped error. |
| `cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .` | Three successes; repository boundaries valid each time. |
| `cargo run --manifest-path tools/agent-context/Cargo.toml -- check` | Three successes. |
| `bash scripts/ci/check_workspace_rust_line_caps.sh dirty` | Three successes, 5 Rust files each. |
| `git diff --check origin/master...HEAD`; `git diff --check` | Three successes each, zero whitespace errors. |
| `python target/k1c-repair-logs/r1-lint-scope.py`, with PYTHONUTF8=1 | Three successes, four untouched lint-file assertions each (12 total). |

Staged whitespace also passed once with zero errors. The same reviewer passed
R6 under qa-loop, qa-tests and code-quality-qa after independently checking the
final source, literal oracles, three intended compiled failures, exact restored
bytes, rebuilt four-test run and complete gate ledger. Its wrong filter advice
is corrected above; no R6 finding remains open.
# R7: typed invariants and exact final stop precedence

The two Native content-control shells share one concrete refusal-selection
owner. Only NestedStopped yields to an already returned inner denial; concrete
outer stops stay authoritative. Missing invocation/result invariants return
EncodingInconsistency. Source structural refusals keep their existing dedicated
channel. Query's missing locator and latch refuse with sticky typed causes,
and content field work uses the existing checked consume. Checked frame-width
conversions replace the remaining Native content layout expects. The named
registry/readmission.rs currently has no expect and needed no invented change.
No AspectShape or generic engine was added.

The new pure court has seven literal expected outcomes, independent of request
execution and of the merge implementation. The two existing actual Native
request courts protect the nested SDK boundary and unchanged selected state.

| R7 exact command or experiment | Result |
| --- | --- |
| `git fetch origin master codex/919-provider-lifecycle-20261027`; `git merge --no-edit origin/master`; `git merge --no-edit origin/codex/919-provider-lifecycle-20261027` | All three succeeded; both merges already up to date. |
| `cargo fmt -p worth-relational` | One success. |
| `cargo fmt --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution` | One success. |
| `cargo test -p worth-relational --lib content_stop_precedence_preserves_exact_inner_only_for_nested_handoff -- --nocapture` | Baseline and restored: 1 passed, 0 failed/ignored/measured, 2273 filtered each. Compiled always-outer mutant: 0 passed, 1 failed, 0 ignored/measured, 2273 filtered; actual Execution(NestedStopped) differed from literal Execution(WorkExhausted). |
| Python merge mutation install/restore, with PYTHONUTF8=1 | Owner located successfully; one byte-exact restored-source assertion passed, timestamp refreshed. |
| `cargo test -p worth-relational --lib empty_commit_request_refuses_before_publication -- --nocapture` | 1 passed, 0 failed/ignored/measured, 2273 filtered. Original expectation unchanged. |
| `cargo test -p worth-relational --lib unfunded_native_validation_preserves_the_branch_and_releases_request_memory -- --nocapture` | 1 passed, 0 failed/ignored/measured, 2273 filtered. Original expectation unchanged. |

An additional reviewer source audit traced empty/no-op transactions through
proposal_round, apply_mutation_draft and unconditional artifact preparation.
They retain staged attachments and reach R5's final closure check; no empty
commit bypass or production follow-up was found. This was source inspection,
not a further executed court.

| R7 public integration and artifact observation | Result |
| --- | --- |
| `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-court-decl-host --test worth_query_court_decl_host content_door:: -- --nocapture` | FAILED: 20 passed, 1 failed, 0 ignored/measured, 327 filtered, 1.85 seconds. content_recorded_inverse_and_reapplication_preserve_exact_source_bytes failed at revision_courts.rs:86 with ProviderRejected / Native UnreferencedAttachment for original identity 7bcb0e089d83cadc112d673d0f68e3200fdde9f3b4b46626f60e6f83667d165a. |
| Debug rlib measurement, same file/profile | libworth_query_execution-b7016cf434752ce1.rlib: 319,335,294 before; 319,337,486 after; increase 2,192 bytes. This is an artifact observation, not isolated monomorphization attribution. |

The public failure is an active caused R5 consumer regression, not untouched
debt and not a passing public portfolio. Independent review traced correction
custody's deliberate preimage/output union into unconditional authored staging
at commit_preparation.rs:163-176. Reapplication writes B but stages old A too.
The Native surplus guard is correct and the court's independent literal IDs
remain unchanged. The immediate separate R5 follow-up must keep retained
correction custody as read availability and stage only actual missing reads,
while preserving every explicitly authored attachment. R7 stays separate to
keep finding commits coherent; final qualification requires this failure closed.

| R7 final exact command | Result |
| --- | --- |
| `cargo fmt --check -p worth-relational` | One success. |
| `cargo fmt --check --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution` | One success. |
| `cargo clippy -p worth-relational --all-targets` | Failed solely on untouched payload.rs:34 let_and_return, 1 error and 8 warnings. |
| `cargo clippy --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --all-targets` | Success; 47 lib warnings, 44 lib-test warnings (42 duplicates), no error; 4m04s. |
| `cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .` | One success; repository boundaries valid. |
| `cargo run --manifest-path tools/agent-context/Cargo.toml -- check` | One success. |
| `bash scripts/ci/check_workspace_rust_line_caps.sh dirty` | One success; 6 Rust files. |
| `git diff --check origin/master...HEAD`; `git diff --check` | One success each, zero whitespace errors. |
| `python target/k1c-repair-logs/r1-lint-scope.py`, with PYTHONUTF8=1 | Four untouched lint-file scope assertions passed. |

# R5 follow-up: correction availability is not authored staging

Native's concrete read closure admits all available retained input under the
original request, then selects only demanded identities absent from authored
staging and the genuine current root. Every explicitly authored attachment
survives; Native's existing final candidate closure remains the publication
check. Query neither parses content nor adds a second domain-write seal.
The untouched correction union remains retained availability. No AspectShape
or generic engine body is added.

The extended existing surplus court uses the independent frozen grid identity,
a real transaction refusal, complete selected descriptor equality and exact
original memory restoration. With 10/100/1000 unrelated available frames,
complete acquisition Work is 401460/3060430/30289554; selector Work is 960 in all
three. These are separate stages, not a constant-cost claim for admission.
Final diagnostic timings (microseconds, observations only) were acquisition
1047/11307/124212 and selector132/190/118. The tenfold growth inequalities use
an independent bound, and selection must spend positive Work. The existing
public inverse/reapplication court keeps its independent literal IDs unchanged.

| R5 follow-up exact command or experiment | Result |
| --- | --- |
| `git fetch origin master codex/919-provider-lifecycle-20261027`; `git merge --no-edit origin/master`; `git merge --no-edit origin/codex/919-provider-lifecycle-20261027` | All succeeded. Master advanced to e04dc5b176 and merged ten files in 99b9277d25; provider already up to date. |
| `cargo fmt -p worth-relational -p worth-court-relational-certification`; Query execution fmt with its manifest | Two successes each. |
| `cargo test -p worth-court-relational-certification --test worth_court_relational_certification unreferenced_attachment_refuses_with_literal_identity_and_releases_custody -- --nocapture` | Two baseline runs: 1 passed, 0 failed/ignored/measured, 212 filtered each (0.25s/0.26s). |
| Same exact court with authored clone replaced by empty staging | Compiled mutant: 0 passed, 1 failed, 0 ignored/measured, 212 filtered. Real transaction wrongly published and unwrap_err failed before the redundant selected-ID assertion. |
| Python mutation install/restore with PYTHONUTF8=1 | One anchor checked; one byte-exact restored-source assertion, fresh timestamp. |
| `cargo test -p worth-court-relational-certification --test worth_court_relational_certification content:: -- --nocapture` after restoration | 64 passed, 0 failed/ignored/measured, 149 filtered, 20.35s; rebuilt restored source. |
| Public content_door:: and Query execution Clippy first attempts | Both failed compilation with E0061 at effect_program/content.rs:51; no tests. Master's parent-removal guard gained the entity-name argument. K1c caller now passes name and actual ID consistently with ordinary authoring. |
| `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-court-decl-host --test worth_query_court_decl_host content_door:: -- --nocapture` after merge adaptation | 21 passed, 0 failed/ignored/measured, 331 filtered, 2.50s. Master added four unrelated tests. The previously red inverse/reapplication court passes unchanged. |
| Debug rlib observation, same path/profile | libworth_query_execution-b7016cf434752ce1.rlib: 319337486 before, 319365618 after, +28132 bytes. Includes master sync; not isolated generic-code attribution. New engine code is concrete. |

| R5 follow-up final gates | Result |
| --- | --- |
| Native/court fmt check; Query execution fmt check | Two successes each. |
| `cargo clippy -p worth-relational -p worth-court-relational-certification --all-targets` | Two failed attempts, each only untouched payload.rs:34, 1 error/8 warnings. |
| `cargo clippy -p worth-court-relational-certification --all-targets --no-deps` | Two failed attempts, each the four previously listed untouched court errors/2 warnings. |
| `cargo clippy --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --all-targets` | First failed E0061 (above); corrected run succeeded, 47 lib warnings/44 lib-test warnings (42 duplicates), 26.47s. |
| Boundary check; context check | Two successes each. |
| Dirty Rust line caps | Two successes: 4 files before merge-call adaptation, 5 after. |
| `git diff --check origin/master...HEAD` | Two successes. |
| `git diff --check` | First success; second failed only evidence.md new blank line at EOF. Removed that trailing blank; corrected check succeeded with zero errors. |
| Python lint-scope check with PYTHONUTF8=1 | Two successes, 4 assertions each; untouched lint errors do not overlap branch/dirty scope. |
| `git diff --cached --check` | Success, zero errors. |

The same reviewer independently read the complete production/court diff and
passed qa-loop, qa-tests and code-quality-qa after all receipts. It requested
only the staged-diff receipt above, now recorded. No behavior expectation was weakened. Remaining full Native reds
are tracked separately. This closes the R5 correction-staging regression.

# R8: atomic inline meaning and exhaustive consumer review

Retained InlineBytes for the three concrete generic certification bindings:
InlineCounterBinding, RatioBinding and BrokenBinaryBinding. The counter's real
conditional reader receives the complete scalar without Native resolver
capability. ContentRef cannot serve that contract without changing its authority;
this is a meaning distinction, never a size threshold. Capability index,
application guidance and both owners' READMEs name that consumer and reserve
ordered numerical data and splines at every size for Native content. The public
inventory separates 51 lexical scalar paths and 27 broader binary route paths
from semantic adopters and unrelated lexical matches. These are78 path
observations, not78 unique consumers. No code, aspect shape or test was added.

| R8 exact command or experiment | Result |
| --- | --- |
| Fetch master/provider; merge each with `--no-edit` | Three successes; already up to date. |
| `python target/k1c-repair-logs/r8-inventory-check.py`, PYTHONUTF8=1 | Four count/path assertions;78 path observations verified. |
| Initial `cargo fmt --check -p worth-relational`; Query execution fmt check with explicit manifest | One success each. |
| Initial generated launcher | Contained a later Clippy command missing its package selector. Caught before that command executed (no log/cargo child for it); stopped only launcher. Its active scoped Native Clippy child was left alive; no completion receipt, so no result credited. The first stop-selector also matched the current control shell, ending it with exit-1; corrected selector excludes its own PID. No build was killed. Replaced launcher with explicit scoped commands and a Python scope assertion. |
| `cargo fmt --check -p worth-foundational -p worth-relational` | Success. |
| `cargo clippy -p worth-relational --all-targets` | Failed only untouched payload.rs:34,1 error/8 warnings. |
| `cargo clippy -p worth-foundational --all-targets --no-deps` | Success; no errors. |

The reviewer found no ownership or composition defect and requested direct
links to the conditional reader and schema that establish the authority gap;
those are now present. This prose-only slice adds no court and claims no new
mutation result. Full-binary qualification remains pending subsequent repairs.

| R8 remaining exact gates | Result |
| --- | --- |
| Boundary check; context check | One success each. |
| `bash scripts/ci/check_workspace_rust_line_caps.sh dirty` | Success,0 Rust files. |
| `git diff --check origin/master...HEAD`; `git diff --check` | One success each, zero whitespace errors. |
| Python lint-scope check with PYTHONUTF8=1 | Four assertions passed. |
| `git diff --cached --check` | Success, zero errors. |

The same reviewer passed R8 under qa-loop, qa-tests and code-quality-qa after
independently checking the staged docs, consumer links and final receipts.
