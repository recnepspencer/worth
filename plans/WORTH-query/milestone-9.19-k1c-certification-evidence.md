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

# R9: identity conflict through real admission

A compiled scoped control belongs to the actual schema registry receiver and
matches its installed format Arc. Only an actual duplicate admission consumes
it. The frame owner readmits a fresh original-funded copy, checks exclusive
access and flips one payload byte for comparison; that witness never enters
the directory. The same_content owner performs the exact comparison. No hash
owner, aspect shape, arbitrary identity override or portable snapshot changes.
The only added guard edge names this certification file.

The new identity_conflict court creates two genuine duplicate admissions and
expects literal ContentIdentityConflict from the real transaction. It checks
the complete selected descriptor and full original-memory restoration. Scoped
unwind cleanup is then proved by a real unarmed duplicate commit. The oracle is
a literal typed refusal, not the equality algorithm copied into the court.

| R9 exact command or experiment | Result |
| --- | --- |
| Fetch master/provider; merge each with `--no-edit` | Three successes; already up to date. |
| `cargo fmt -p worth-relational -p worth-court-relational-certification` | Success. |
| `cargo test -p worth-court-relational-certification --test worth_court_relational_certification duplicate_content_disagreement_refuses_real_admission_with_literal_identity_conflict -- --nocapture` | Baseline: 1 passed, 0 failed/ignored/measured, 213 filtered, 0.01s. |
| Same exact court, deleting only admission's prior.same_content call | Compiled mutant: 0 passed, 1 failed, 0 ignored/measured, 213 filtered. Real publication wrongly succeeded; unwrap_err saw Ok(CommitResult). |
| Python mutation install/restore, PYTHONUTF8=1 | One comparison anchor; one byte-exact source restoration assertion, timestamp refreshed. |
| `cargo test -p worth-court-relational-certification --test worth_court_relational_certification content:: -- --nocapture` after restoration | 65 passed, 0 failed/ignored/measured, 149 filtered, 25.53s; rebuilt restored source in 30.19s. |

| R9 final exact gates | Result |
| --- | --- |
| `cargo fmt --check -p worth-relational -p worth-court-relational-certification` | Success. |
| `cargo clippy -p worth-relational -p worth-court-relational-certification --all-targets` | Failed only untouched payload.rs:34, 1 error/8 warnings. |
| `cargo clippy -p worth-court-relational-certification --all-targets --no-deps` | Failed only four previously listed untouched court errors/2 warnings. |
| Boundary check; context check | One success each. |
| `bash scripts/ci/check_workspace_rust_line_caps.sh dirty` | Success, 7 Rust files. |
| `git diff --check origin/master...HEAD`; `git diff --check` | One success each, zero errors. |
| Python lint-scope check, PYTHONUTF8=1 | Four scope assertions passed. |
| `git diff --cached --check` | Success, zero errors. |

The same reviewer passed R9 under qa-loop, qa-tests and code-quality-qa after
independently checking the staged owner/court/guard, actual compiled kill, exact
restoration, rebuilt portfolio and final receipts. No R9 finding remains open.

# Full Native red repairs: production accounting, Source and recovery

Content evidence now settles through the existing commit RequestWorkBudget once
per initial/final evidence stage. Kernels do not receive or re-enter that
account. R7's precedence owner preserves exact inner refusals only against the
nested sentinel. Direct physical suspension uses physical-after content; only
real matching Source custody can supply logical-before content. Journal FieldText
semantic corruption receives CorruptCanonicalArtifact at that receiver, while
checkpoint classification and typed resource/storage causes stay unchanged.

The kind-forgery fixture now supplies the actual attached journal alongside its
physical members, and proves zero member reads at the same exact corruption
refusal. No semantic expectation was changed. The two allocation courts now
expect the actual first Native SDK framework admission: literal 408 bytes before
its callback. The earlier first-member 161 and first-envelope 3922 input quotes
were real but unreachable on original budgets 1/0. Both our initial reading and
the reviewer's payload-first advice missed that enclosing scope admission; the
failed runs are recorded below. No production bypass was added to force the
old allocation site. Capture's first substage can be digest or sizing depending
on residency. Readmission's first stage hashes a physical member.

Checkpoint fixture Work 1 could no longer fund controlled hashing/emission;
it is now bounded at 1, 000, 000. Positive peak memory is bounded at 1, 048,576 bytes,
allowing simultaneous envelope/outer-frame/framework custody; this is an upper
bound, not a measured minimal peak. Actual retained original custody is 12,886
outer bytes plus 0 newly charged member bytes, exactly matching parent charges.
Existing member backing is external to that parent. Clone pointer equality,
exact denied over-reservation, genuine restore/edit, unchanged Allocation
cancellation cause and final parent charge 0 all pass. The original lower-memory
refusals retain exact original PolicyMemoryExhausted(408,1/0, 0) causes.
No new court function or aspect shape is introduced.

All commands below use `cargo test -p worth-relational --lib <filter> -- --nocapture`.
Each of the seven reproduction runs before repair selected one test and failed with 0 passed,
1 failed, 0 ignored/measured, 2273 filtered. Every corrected row selected one and
passed with 1 passed, 0 failed/ignored/measured, 2273 filtered. Literal semantic
expectations stay unchanged; only the explained memory invoices changed.

| Exact test filter | Corrected focused result |
| --- | --- |
| `one_work_account_bounds_the_same_proposal_on_every_entry` | 1 passed, 0 failed, 2273 filtered. |
| `durability_replays_suspended_and_restored_materialization_roots` | 1 passed, 0 failed, 2273 filtered. |
| `forged_cross_region_kind_commitment_is_refused_before_member_reads` | 1 passed, 0 failed, 2273 filtered; exact class/detail and0 member reads. |
| `native_field_text_checkpoint_readmission_denial_keeps_bytes_and_target_root_intact` | 1 passed, 0 failed, 2273 filtered; exact408/1/0 invoice, bytes/root unchanged and funded restore passes. |
| `native_field_text_missing_tail_dictionary_refuses_without_target_or_source_effect` | 1 passed, 0 failed, 2273 filtered. |
| `native_checkpoint_admits_exact_backing_retains_clones_and_restores_editable_truth` | 1 passed, 0 failed, 2273 filtered; exact408/0/0 invoice and12886 retained bytes. |
| `request_includes_rematerialized_relation_as_a_planned_edge` | 1 passed, 0 failed, 2273 filtered. |

| Further exact command or attempt | Result |
| --- | --- |
| Fetch master/provider; merge both with `--no-edit` | Three successes; already up to date. |
| Initial `cargo fmt -p worth-relational` | Success. |
| First work-account correction build | E0603: private request_work module, no tests. Used its existing execution facade re-export. |
| Next materialization and kind correction builds | Two E0308 failures: AttachedTestMedium wrapper where Arc ColdSource required; no tests. Used existing attached_durable_medium Arc accessor. |
| First managed-text correction run | Compiled: 0 passed/1 failed, 2273 filtered: actual408/1/0 differed from input-derived161/1/0. |
| First allocation correction run | Compiled: 0 passed/1 failed, 2273 filtered: actual408/0/0 differed from input-derived3922/0/0. |
| First missing-dictionary and rematerialized correction runs | Each 1 passed, 0 failed/ignored/measured, 2273 filtered (additional to corrected seven-row run). |
| Corrected `cargo fmt -p worth-relational` | Success. |
| `cargo test -p worth-relational --lib empty_commit_request_refuses_before_publication -- --nocapture` | 1 passed, 0 failed/ignored/measured, 2273 filtered. |
| `cargo test -p worth-relational --lib unfunded_native_validation_preserves_the_branch_and_releases_request_memory -- --nocapture` | 1 passed, 0 failed/ignored/measured, 2273 filtered. |
| `cargo test -p worth-court-relational-certification --test worth_court_relational_certification leased_preparation -- --nocapture` | Before this slice 16 passed, 0 failed/ignored/measured, 197 filtered; after 16 passed, 0 failed/ignored/measured, 198 filtered, 2.96s. R9 added one unrelated court. |
| `cargo test -p worth-court-relational-certification --test worth_court_relational_certification content:: -- --nocapture` | 65 passed, 0 failed/ignored/measured, 149 filtered, 17.53s. |

| Native repair final exact gates | Result |
| --- | --- |
| Final `cargo fmt -p worth-relational` | Third formatting success in this slice. |
| `cargo test -p worth-relational --lib one_work_account_bounds_the_same_proposal_on_every_entry -- --nocapture` after measurement/comment refinement | 1 passed, 0 failed/ignored/measured, 2273 filtered, 0.06s. Independent whole Work 658 equals settled 658; previous recorded 268 was missing 390. No work expectation changed. |
| `cargo fmt --check -p worth-relational` | Success. |
| `cargo clippy -p worth-relational --all-targets` | Failed only untouched payload.rs:34,1 error/8 warnings. |
| Boundary check; context check | One success each. |
| `bash scripts/ci/check_workspace_rust_line_caps.sh dirty` | Success,10 Rust files. |
| `git diff --check origin/master...HEAD`; `git diff --check` | One success each, zero errors. |
| Python lint-scope check, PYTHONUTF8=1 | Four scope assertions passed. |

Staged whitespace passed with zero errors. The same reviewer passed this Native
repair under qa-loop, qa-tests and code-quality-qa after independently checking
the final source, unchanged semantic oracles, exact resource invoices, all scoped
receipts and the corrected funding reading. Full-binary qualification and the
Store withdrawal/Native lifecycle proof remain required.

## Native lifecycle and Store withdrawal

The out-of-scope Store content court, module declaration, testkit dependency and
its eleven lockfile additions are removed. Against master, Store retains only
the two source compile fixes necessitated by deleting Bytes. No Store test
binary was run during this repair. Store Clippy below compiles its targets.

The new `checkpoint_tail_replacement_and_retained_history_keep_exact_native_content`
court uses actual Native checkpoint members, one canonical journal-tail artifact,
two destroyed/reopened runtimes and a replacement checkpoint. It retains the
original admitted historical envelope across both reopens. Selected fields name
the new frame; the current directory refuses the old identity as MissingContent;
the retained historical reader returns the old exact bytes. C2 carries nine
members and emits four new ones. Original custody retains exactly 1628 bytes
until that reader drops, then releases the complete 536870912-byte budget.
These are Native lifecycle assertions, not physical Store publication or GC.

The independent byte oracle is the literal 75-byte frame in expected.rs, with
Python hashlib verifying its frozen digest and the separately frozen digest
after replacing the first sample at the fixed format offset. The selected-field
reader checks field identity separately from the content-directory reader.

| Lifecycle command or attempt | Exact result |
| --- | --- |
| Fetch master/provider and merge both | Three successes, already up to date. |
| `cargo fmt -p worth-court-relational-certification` | Three successes. |
| Exact lifecycle filter, initial 64 MiB original | 0 passed, 1 failed, 214 filtered: RequestPolicyMemoryExhausted requested268435456/admitted67100398/ancestor0. The Native cold source reserves a bounded 256 MiB read ceiling before its callback. |
| Exact lifecycle filter, 512 MiB original but empty C2 membership | 0 passed, 1 failed, 214 filtered: ColdStorageUnavailable, Lineage1..1, SourceUnavailable. C2 must receive genuine C1 membership and its source. |
| Exact lifecycle filter, corrected genuine membership/source | 1 passed, 0 failed/ignored/measured, 214 filtered, 0.17s. |
| Compiled mutation: journal recovery installs an empty attachment directory instead of admitted envelope content | 0 passed, 1 failed, 214 filtered, 0.10s; typed Content(MissingContent) kills the lifecycle court. |
| Byte-exact source restoration and fresh modification time | One restoration assertion passed. |
| `cargo test -p worth-court-relational-certification --test worth_court_relational_certification content:: -- --nocapture` rebuilt after restore | 66 passed, 0 failed/ignored/measured, 149 filtered, 18.74s. |
| `PYTHONUTF8=1` lifecycle-oracle.py | Three assertions passed: frame length and both independent SHA-256 digests. |
| Native court `cargo fmt --check -p worth-court-relational-certification` | Success. |
| Store `cargo fmt --manifest-path workspaces/worth-store/Cargo.toml --check -p worth-store-court-relational` | Success. |
| Native court Clippy all-targets | Failed at untouched Native payload.rs:34, 1 error/8 warnings. |
| Native court Clippy all-targets `--no-deps` | Failed at the four previously recorded untouched court lint sites, 4 errors/2 warnings. |
| Store `cargo clippy --manifest-path workspaces/worth-store/Cargo.toml -p worth-store-court-relational --all-targets` | Success; compilation only, no Store test execution. |
| Boundary check; context check | One success each. |
| `bash scripts/ci/check_workspace_rust_line_caps.sh dirty` | Success, 5 Rust files. |
| `git diff --check origin/master...HEAD`; `git diff --check` | One success each, zero errors. |
| `PYTHONUTF8=1` lint-scope check | Four assertions passed. |

The same reviewer passed this slice under qa-loop, qa-tests and code-quality-qa.
The first full qualification run has Native 2241 passed, 2 failed, 31 ignored
(296.67s) and Native certification 214 passed, 0 failed, 1 ignored (61.81s).
Both Native failures remain active follow-up work: the explicitly retired old
serialization shape expectation and a persisted CDC checkpoint leaf invariant.
All nine original reported Native failures pass. Public qualification is still
running at this slice's evidence cut; this is not an overall green claim.

Process deviation: sync merge 99b9277d25 used Git's default merge message without
the required full sentence and coauthor trailer. The no-amend/no-rewrite rule
prevents repairing that existing commit. It is preserved and disclosed; future
merge commits must supply the required message explicitly.

## Complete Native qualification repairs

The first full public run completed: `cargo test --manifest-path
workspaces/worth-query/Cargo.toml -p worth-query-court-decl-host --test
worth_query_court_decl_host`, with the prescribed build ID, passed 350 tests,
0 failed, 2 ignored, 0 measured/filtered, 45.89s. The two Native failures were
not treated as debt.

The retired-shape test now expects required seven-field schema authority and
ten-field durable carrier forms. It retains an independently declared serializer,
literal positional tuple lengths, empty content authority, domain-only preceding
tail checks and all named/compact/JSON/readmission roundtrips. This is the explicit
deleted-old-form exception to changing expectations, not compatibility support.

The frozen CDC scenario reproduced a historical-kind mismatch in partition31:
partition identity, content digest, entity slots, relation slots, unavailable
records and the current-kind digest all agreed; only historical-kind digest
differed. Empty endpoint-incidence journals did not certify unchanged relation
row/history meaning. Root publication now includes the relation-row journal in
that reuse proof. All six independent checkpoint leaf checks remain intact.

Review found missing admission on that newly required reconstruction. The same
original request now crosses the existing Native root-capture path, with exact
Execution refusals through root/publication/transaction and recovery receivers.
Its top-level root stage settles once into the existing RequestWorkBudget.
Single projection engines take erased per-item admission, use borrowed rows,
poll empty kind buckets/directions, admit traversal backing before allocation,
and stream MessagePack hashing under bounded chunk checkpoints. The old unused
whole-index mutable materializers are deleted, including their complete-Vec
construction. Ordinary scalar entity edits retain the existing reuse path.

Scratch funding is separate from the historical-kind result's retained backing.
Each rebuilt entry receives a fresh original reservation before persistent path,
value and index allocation. Its existing StorageAllocation owns the custody
mutex, so actual entry clones/adoptions retain exactly the same grant. Inventories
and allocation visitors include its physical token. Reset never grows a prior
root's ticket. The SDK's own charged-memory counter is the independent custody
oracle; the court observes fresh replacement, cloned aliasing, partial Native
table adoption and final release. Explicit native-only None accounting remains
the SDK's existing mode; no free replacement account is created.

| Focused command/attempt before final qualification | Exact result |
| --- | --- |
| Fetch and both explicitly messaged sync merges | Three successes; both merges already up to date. |
| `cargo fmt -p worth-relational` before diagnostic and initial correction | Two successes. |
| `cargo test -p worth-relational --lib retained_relation_history_checkpoint_matches_published_leaf_and_recovers -- --nocapture`, diagnostic | 0 passed, 1 failed, 2274 filtered, 0.46s; exact checkpoint CorruptCheckpoint leaf mismatch, diagnostic axes `[true,true,true,true,true,false]`. Temporary diagnostics removed byte-exactly before correction. |
| Same deterministic court with relation journal correction | 1 passed, 0 failed/ignored/measured, 2274 filtered, 0.89s. |
| `cargo test -p worth-relational --lib required_content_authority_keeps_exact_shape_and_positional_empty_tails -- --nocapture` | 1 passed, 0 failed/ignored/measured, 2274 filtered, 0.03s. |
| `cargo test -p worth-relational --lib cdc_property_persisted_random_operation_matrix_recovers -- --nocapture` | 1 passed, 0 failed/ignored/measured, 2274 filtered, 36.37s; saved regression seed retained. |
| First compiled relation-journal-guard deletion mutant | 0 passed, 1 failed, 2274 filtered, 0.55s; same checkpoint leaf mismatch. |
| Byte-exact guard restoration and rebuilt deterministic court | Restoration assertion passed; 1 passed, 0 failed/ignored/measured, 2274 filtered, 0.86s. |
| Control implementation first compile | E0308: exact execution cause required the existing CommitExecutionDenial wrapper, no tests ran. |
| Control implementation second compile | Library compiled; unit target E0061 at the two existing publication_identity fixture callers requiring the original request, no tests ran. |
| Control implementation third compile | Three E0308 Option-membership callback types and two closure lifetime errors; no tests ran. |
| Control implementation fourth compile | Library compiled; unit target E0308/E0277 because packet recorder returns a Vec of recorded stages, not the SDK whole Work total; no tests ran. Replaced with the SDK's actual whole-request report. |
| `cargo fmt -p worth-relational` during those four control attempts | Four successes. |
| `cargo fmt -p worth-relational` before first compiled control courts | Success. |
| `cargo test -p worth-relational --lib storage::residency::tests::kind_admission_tests:: -- --nocapture`, first compiled courts | 1 passed, 1 failed, 2275 filtered, 0.05s. Custody court passed: SDK whole Work501, retained12864, elapsed350.7 microseconds. New stop court's guessed scratch-only invoice failed: actual PolicyMemoryExhausted requested65984/admitted65551/ancestor0, expected65536/65535/0. |
| `PYTHONUTF8=1` independent empty-projection digest check | One literal assertion passed: SHA-256 `eca5e71347d5448811e5333ba940265daa5435b573f498127d8c3886a020903c` for the v2 domain separator followed by MessagePack empty-array byte0x90. This literal is also asserted by the Rust custody court. |

The new memory court now expects its reached original-owner invoice exactly:
the existing measured-stage loan is448 bytes, so adding65536 requests65984;
the enclosing SDK scope has admitted392 of65943 policy bytes, leaving65551.
Its Work0 and cancellation expectations are unchanged. No pre-existing semantic
expectation was weakened. An initial attempt to stop the childless qualification
waiter conservatively refused because it also had a conhost child; after proving
that was its only child, only the waiting launcher was stopped. Active Cargo and
the compiled mutation/restoration sequence were left running. No build was killed.

| Final control and mutation receipts | Exact result |
| --- | --- |
| Sixth control formatting | Success. |
| Sixth control compilation | Two E0599 errors exposed a cfg(test)-only iter_mut fixture caller, no tests ran. The fixture now resets the same keys through get_mut; all original assertions remain. The remaining reset convenience is cfg(test)-only. |
| Seventh `cargo fmt -p worth-relational` control formatting | Success. |
| Corrected `cargo test -p worth-relational --lib storage::residency::tests::kind_admission_tests:: -- --nocapture` | 2 passed, 0 failed/ignored/measured, 2275 filtered, 0.04s. Whole Work501, retained12864, measured positive reconstruction297.2 microseconds. These are actual SDK reports, not a public cost signature. |
| Final mutation installer, invalid attempts | Two attempts stopped before source mutation or Cargo: CRLF anchor mismatch, then a Python source-escaping syntax error. Neither is mutation evidence. The corrected raw-byte-aware installer matched exactly one protection for each compiled mutant below. |
| Compiled final root-reuse mutant, exact deterministic CDC filter | 0 passed, 1 failed, 2276 filtered, 0.36s; exact CorruptCheckpoint, captured region differs from selected root leaf. |
| Compiled original-Work mutant, exact literal-stop court filter | 0 passed, 1 failed, 2276 filtered, 0.04s; expected WorkExhausted but got an otherwise successful RelationalRegionKindCommitment. |
| Compiled retained-custody omission mutant, exact clone/adoption court filter | 0 passed, 1 failed, 2276 filtered, 0.05s; `funded > 0` fails on the independent SDK charge counter. |
| Each mutant source restore | Three byte-exact restoration assertions passed, modification times refreshed. |
| Rebuilt `cargo test -p worth-relational --lib storage::residency::tests::kind_admission_tests:: -- --nocapture` | 2 passed, 0 failed/ignored/measured, 2275 filtered, 0.04s. |
| Rebuilt `cargo test -p worth-relational --lib retained_relation_history_checkpoint_matches_published_leaf_and_recovers -- --nocapture` | 1 passed, 0 failed/ignored/measured, 2276 filtered, 0.58s. |
| Rebuilt `cargo test -p worth-relational --lib required_content_authority_keeps_exact_shape_and_positional_empty_tails -- --nocapture` | 1 passed, 0 failed/ignored/measured, 2276 filtered, 0.02s. |

The same reviewer independently read the materializer removal and fixture change,
the per-item admission loops, exact refusal transport, cache-token inventory and
fresh-ticket lifetime. It found no further source blocker and awaits complete
full-binary and footer receipts before its final gate. No new AspectShape form
or Query engine body is added by this Native qualification follow-up.

The final complete Native library run passed: `cargo test -p worth-relational
--lib`, 2246 passed, 0 failed, 31 ignored, 0 measured/filtered, 282.06s. The
complete certification run then exposed one positive fixture's bounded grant:
`cargo test -p worth-court-relational-certification --test
worth_court_relational_certification`, 213 passed, 1 failed, 1 ignored,
0 measured/filtered, 60.02s. Its isolated child reported 0 passed, 1 failed,
214 filtered, 2.28s; that is the same failure, not a second failed court.

`preimage_endpoint_slots` reached exact original PolicyMemoryExhausted
requested6648/admitted4384/ancestor0 while committing1024 changed records and
1023 new relations. Its positive grant was32MiB. The existing authority ceiling
is64MiB, so the fixture now funds that bounded amount for newly admitted scratch
and retained reconstruction paths. The1million Work ceiling, literal1024/1025
endpoint boundary, snapshot/source assertions and separate refusal courts stay
unchanged. This is not a minimal-peak claim. The same reviewer independently
checked the reached invoice and test-only change and accepted the funding
choice pending the affected and complete certification reruns. Native production
remains frozen at the complete passing run above.

The public rebuild's `libworth_query_execution-b7016cf434752ce1.rlib` measures
319361570 bytes (2026-11-02, local build target). The preceding recorded artifact
was319365618 bytes, a net decrease of4048 bytes. This is an aggregate measurement
across the intervening Native/dependency changes, not an isolated monomorphization
attribution. This qualification slice adds no Query engine code; the earlier
Query-touching slices' measured rlib changes remain recorded individually above.

The first memory-funded exact rerun compiled and reached the next original
bound: `cargo test -p worth-court-relational-certification --test
worth_court_relational_certification preimage_endpoint_slots -- --nocapture`,
0 passed, 1 failed, 214 filtered, 4.30s (isolated child4.27s), exact WorkExhausted.
Its preceding `cargo fmt -p worth-court-relational-certification` succeeded.
Memory admission now clears, but the newly admitted projection traversal,
retained-path reconstruction and streamed hashing exhaust the old1million Work
grant. This positive journey now uses the existing lease_request_with_work
helper with bounded8million Work and64MiB. All semantic assertions and separate
refusal courts remain intact; neither grant is a minimal-cost claim. This
supersedes the earlier intent to leave its Work grant unchanged. No production
source changes between the full Native pass and these test-only corrections.

The complete final public rerun passed: `cargo test --manifest-path
workspaces/worth-query/Cargo.toml -p worth-query-court-decl-host --test
worth_query_court_decl_host`, with the prescribed build ID, 350 passed,
0 failed, 2 ignored, 0 measured/filtered, 51.84s. Its build completed in6m22s.

The bounded positive fixture now passes: `cargo test -p
worth-court-relational-certification --test worth_court_relational_certification
preimage_endpoint_slots -- --nocapture`, 1 passed, 0 failed/ignored/measured,
214 filtered, 4.07s. Its formatting command succeeded. The subsequent complete
`cargo test -p worth-court-relational-certification --test
worth_court_relational_certification` passed214, failed0, ignored1,
measured/filtered0, 79.03s. The reviewer independently confirmed that the
separate refusal fixture still uses its original32MiB/default1million Work.

The Native-only footer's `cargo fmt --check -p worth-relational` succeeded.
`cargo clippy -p worth-relational --all-targets` reported2 errors and8 warnings:
the existing untouched payload.rs:34 let-and-return, and this slice's new
allocation.rs:41 collapsible-if. The latter was corrected by the exact
equivalent if-let/condition chain; `cargo fmt -p worth-relational` succeeded.
The reviewer confirmed no changed evaluation order, allocation, result or
refusal behavior. Complete Native/public receipts above precede only that
mechanical style change; this is not a claim of another complete rerun.

The same footer's boundary command failed with two exact violations:
BC9002_TEST_TOPOLOGY at storage/residency/kind_admission_tests.rs::file and
tests/publication/cdc/certification/historical_kind_checkpoint.rs::file,
both naming ExecutionRequest runtime-root construction in new src test modules.
This is a hard-rule failure, not repository debt. No source-baseline exemption,
test relocation into an existing source file, or weakened proof is used to
silence it. Existing external-facade feasibility is under independent review;
the complete qualification slice remains uncommitted while this gate is red.
That footer's context command passed, dirty line caps passed37 Rust files,
branch and dirty whitespace checks passed, and its Python scope check passed
four lint-file assertions. The four pre-existing lint files also have no
branch diff against origin/master; the now-corrected allocation lint was in
scope and was not claimed as existing debt.

Independent final boundary ruling: BLOCK / required stop. Existing external
Native doors can support a redesigned checkpoint/recovery integration scenario,
but cannot preserve this exact private normalization and per-entry/partial
SparseAdjacencyTable adoption proof. kind_admission::normalize is pub(super),
PartitionState/SparseAdjacencyTable and custody accessors are crate-private,
and no existing certification facade lends those owners. The private CDC
scenario harness is not available externally unchanged. Adding a new access
door requires a new boundary decision; it is not inferred as an exemption.
test-convention section1 forbids new src runtime roots, and section7 permits
private-access tests only within section1. The reviewer expressly refused
grandfathered-file relocation, baseline widening, suppression or committing
this slice. The user's hard-rule stop applies. R1–R9 and the earlier regression
and Native lifecycle commits remain intact; the final qualification repair
remains uncommitted for a subsequent boundary decision. Full-binary green
results do not constitute overall certification while this guard is red.

The already-running final two-package footer completed after that stop:

| Exact final footer command | Result |
| --- | --- |
| `cargo fmt --check -p worth-relational -p worth-court-relational-certification` | Success, zero formatting differences. |
| `cargo clippy -p worth-relational -p worth-court-relational-certification --all-targets` | Exit101: one existing payload.rs:34 error, eight warnings; the in-scope allocation style error is gone. |
| `cargo clippy -p worth-court-relational-certification --all-targets --no-deps` | Exit101: four existing errors, two warnings, in the same three untouched court files identified in R1. |
| `cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .` | Exit1: the same two BC9002 violations; no exemption or suppression. |
| `cargo run --manifest-path tools/agent-context/Cargo.toml -- check` | Success; no case count supplied. |
| `bash scripts/ci/check_workspace_rust_line_caps.sh dirty` | Success,37 Rust files. |
| `git diff --check origin/master...HEAD` | Success, zero whitespace errors. |
| `git diff --check` | Success, zero whitespace errors. |
| `PYTHONUTF8=1` Python lint-scope check | Four assertions passed; no existing lint error file overlaps the dirty set. Separate branch-diff check also empty for all four lint files. |

No Store binary ran. No blocked-slice staging or commit occurred. The same
reviewer issued BLOCK using qa-loop, qa-tests and code-quality-qa. No extra
build was started after the hard-rule stop, and all previously running checks
were allowed to finish. The style-only if-let collapse has no subsequent
rebuilt control-court receipt, because the required boundary stop superseded
that proposed focused rerun. Do not infer it was run.

## Resumed split-layer qualification

The user explicitly ruled that private decisions use value-only unit tests,
while runtime/request/durability facts use existing external Native doors.
No new access door, root-registry edit, exemption or grandfathered relocation
is authorized. The same reviewer accepted SerialMemoryBudget/reservation values
for the actual private custody owner: budgets are explicitly excluded from the
unchanged root registry. The production driver and private fixtures now call one
extracted normalize_value engine with the existing erased callbacks. The old
root-driving src CDC test is replaced by a public Native certification scenario;
the original behavior name is retained externally. The original stop court's
runtime facts move externally; literal private claims/refusal and custody
adoption tests remain value-only. All previous stopped-run receipts remain
historical evidence, not current boundary qualification.

| Split-layer attempt | Exact result |
| --- | --- |
| Fetch and both explicitly messaged sync merges | Three successes; both branches already up to date. |
| First two-package `cargo fmt` | Success. |
| First `cargo test -p worth-relational --lib kind_admission_tests -- --nocapture` | E0599, AdjacencyPolicy has no Default; no tests ran. Replaced with explicit policy values. |
| First external `content::historical_kind::` court filter | Three E0624 errors: snapshot is private; no tests ran. Fixture moved to the existing public observation-based snapshot door. |
| Second two-package `cargo fmt` | Success. |
| Second private-unit filter | 3 passed,0 failed/ignored/measured,2274 filtered,0.00s. |
| Second external filter | One E0308: branch descriptor is not an observation; no tests ran. Use the admitted basis's existing observation method. |
| Third two-package `cargo fmt` | Success. |
| Third private-unit filter | 3 passed,0 failed/ignored/measured,2274 filtered,0.00s. |
| Third external filter | Two E0609 record kind fields and one E0599 text accessor; no tests ran. Complete identity/value observer now uses resolved kind fields and the pinned reader's symbol authority. |
| Fourth two-package `cargo fmt` and private-unit filter | Formatting passed;3 private tests passed,0 failed/ignored/measured,2274 filtered,0.00s. |
| Fourth external filter | Two E0599 errors: materialized RelationalReadView does not own a symbols door; no tests ran. Use the existing pinned-read capability's frozen symbols. |
| Fifth two-package `cargo fmt` | Success. |
| Fifth external diagnostic filter | 3 passed,0 failed/ignored/measured,215 filtered,0.74s. This establishes candidate invoices, not final assertions. Public original whole SDK Work944, live-commit retained11696, post-runtime successor-basis retained8576. |
| Unchanged early boundary command | Success: repository boundaries valid; no case count supplied. No root registry/baseline/exemption changes. |
| Literal two-package `cargo fmt` | Success. |
| Literal external filter | 3 passed,0 failed/ignored/measured,215 filtered,0.60s. Work943 returns literal WorkExhausted before selected-root change and releases all original memory; Work1024 succeeds with SDK bill944 and retained11696. Actual successor basis retains8576 across physical checkpoint/one-tail reopen and releases at its last drop. |

The external runtime now matches the original regression's explicit
CertificationCore/MVCC values: explicit snapshot release, auto reclamation,
batch8 and EpochChunkRetention. It drives the frozen sequence through actual
public Native operations, using the original read ordering after refresh.
Its literal oracle records every complete selected identity/generation, kind,
field text, relation endpoint and label; both live and recovered truth must
match independently of the writer/recovery output. The selected final relation
is(31,slot0,generation2), with endpoints(7,1,1) and(29,0,1), label third-edge.
No cold recapture invents membership. The separate committed lifecycle court
continues to prove replacement checkpoints with genuine carried membership.

The same reviewer provisionally cleared the final split source and fixture
honesty. Its attribution correction is retained:943 proves whole-original
prepublication Work admission, not the exact internal point of exhaustion;
the Work-to-zero mutation must establish dependence on the root stage. Final
review still requires all compiled mutants, restored-source reruns, complete
binaries on final source and scoped footer/staged checks.

| Compiled split mutation or restored command | Exact result |
| --- | --- |
| Remove relation journal from root-reuse guard; external retained_relation_history_checkpoint_matches_published_leaf_and_recovers | 0 passed,1 failed,217 filtered,0.48s. Exact CorruptCheckpoint: captured region differs from its selected root leaf. |
| Byte-exact root restoration, fresh timestamp, rebuilt same court | Restoration passed;1 passed,0 failed/ignored/measured,217 filtered,0.57s. |
| Charge root checkpoint Work0; external historical_kind_reconstruction_preserves_literal_original_stops_and_releases_scratch | 0 passed,1 failed,217 filtered,0.02s. The943 Work request incorrectly returns Ok(CommitResult), status Published. |
| Byte-exact Work restoration, fresh timestamp, rebuilt same court | Restoration passed;1 passed,0 failed/ignored/measured,217 filtered,0.04s. |
| Drop entry custody assignment; private historical_kind_funding_follows_clones_partial_adoption_and_replacement | 0 passed,1 failed,2276 filtered,0.00s. Independent SDK budget counter fails funded>0. |
| Same compiled custody mutant; external original_kind_custody_survives_successor_basis_checkpoint_tail_and_reopen | 0 passed,1 failed,217 filtered,0.11s. Literal retained original charge is0, expected8576. |
| Byte-exact custody restoration, fresh timestamp, rebuilt private custody court | Restoration passed;1 passed,0 failed/ignored/measured,2276 filtered,0.00s. |
| `PYTHONUTF8=1` final mutation restoration verification | Three byte-equality assertions passed, with SHA-256 witnesses. Same reviewer independently compared all three current files to backups and confirmed equality. |
| Rebuilt `cargo test -p worth-relational --lib kind_admission_tests -- --nocapture` | 3 passed,0 failed/ignored/measured,2274 filtered,0.00s. |
| Rebuilt `cargo test -p worth-court-relational-certification --test worth_court_relational_certification content::historical_kind:: -- --nocapture` | 3 passed,0 failed/ignored/measured,215 filtered,0.52s. |

All four compiled negative runs fail on the intended semantic/resource oracle,
not compilation or an unrelated panic. The same reviewer independently read
their raw failures and confirmed the root/Work rebuilt passes. The original
three protection mutations are therefore caught by the split proof: root reuse
externally, Work externally, and retained custody both privately and externally.
The complete three-binary qualification is now running on restored final source,
including the earlier mechanical Clippy correction. No source edit is planned
while those runs and their footer execute.

The final-source complete Native command, `cargo test -p worth-relational
--lib`, passed2246, failed0, ignored31, measured/filtered0,272.41s. This run
includes both the split value engine and the prior Clippy style correction.
The full external certification and public binaries and scoped footer are
still running at this evidence cut; no complete-lane green claim is made yet.

The final-source full relational certification command, `cargo test -p
worth-court-relational-certification --test worth_court_relational_certification`,
passed217, failed0, ignored1, measured/filtered0,59.90s. This includes the
three external split courts and the original bounded64MiB/8million endpoint
fixture, with every unchanged semantic assertion. Public qualification and
the final scoped footer remain pending.

The final-source full public command, `cargo test --manifest-path
workspaces/worth-query/Cargo.toml -p worth-query-court-decl-host --test
worth_query_court_decl_host`, with the prescribed all-ones build ID, passed350,
failed0, ignored2, measured/filtered0,54.18s. All three complete binaries now
pass on the final restored source, including the mechanical style correction.
The rebuilt Query execution rlib is319361570 bytes, unchanged from the preceding
measurement (net0). This split adds no Query engine code or public access door.
Final two-package formatting check passed with zero differences; the rest of
the scoped footer and staged review are still pending at this evidence cut.

| Final split footer command | Exact result |
| --- | --- |
| `cargo fmt --check -p worth-relational -p worth-court-relational-certification` | Success; zero formatting differences. |
| `cargo clippy -p worth-relational -p worth-court-relational-certification --all-targets` | Exit101: only the untouched payload.rs:34 let-and-return error; eight warnings. No in-scope error. |
| `cargo clippy -p worth-court-relational-certification --all-targets --no-deps` | Exit101: only the four untouched errors in the three named court files; two warnings. No in-scope error. |
| `cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .` | Success: repository boundaries valid; no case count supplied. The two prior BC9002 failures are resolved through the authorized layer split. |
| `cargo run --manifest-path tools/agent-context/Cargo.toml -- check` | Success; no case count supplied. |
| `bash scripts/ci/check_workspace_rust_line_caps.sh dirty` | Success;41 Rust files. |
| `git diff --check origin/master...HEAD` | Success; zero whitespace errors. |
| `git diff --check` | Success; zero whitespace errors. |
| `PYTHONUTF8=1` Python lint-scope check | Four assertions passed; none of the existing error files is in the dirty set. |

No action was taken on the explicitly excluded Clippy debt, older
projected_member_domains defaults or merge99b9277d25's missing trailer. Store's
branch diff remains exactly its two authorized src compile fixes; no Store
binary ran. No new AspectShape form, consumer capability, public access door,
root registry entry or exemption is introduced. The value engine is concrete
and reuses the existing erased callbacks; no Query engine code is added.

Readings chosen: private custody reservations are value facts and remain in
unit tests under the user's explicit ruling. Actual original-request billing,
publication, retained-basis lifetime and physical recovery are external Native
court facts. The literal Work refusal is attributed to whole-original
prepublication admission; the causal mutation establishes its dependence on
reconstruction charging. The64MiB/8million fixture grants are sufficient bounded
positive funding, not measured minima. The earlier hard-rule stop is resolved;
staged independent review remains the final commit gate.

The same reviewer completed that staged gate with PASS under qa-loop, qa-tests
and code-quality-qa. It independently confirmed all complete summaries, the
four intended negative runs, restored hashes/rebuilt courts, clean boundary,
context/format/caps/whitespace receipts and unchanged debt paths against
master, dirty and staged sets. The44 staged files comprise41 Rust files,
two documents and the saved regression seed. No source/evidence blocker remains.

The staged whitespace check passed. An auxiliary Python staged-scope verifier
initially completed four scope assertions, then raised UnicodeDecodeError while
incorrectly reading an ASCII exit receipt as UTF-16; no Cargo or source mutation
occurred. Its BOM-aware correction passed all five assertions with
PYTHONUTF8=1:44 files,41 Rust, only approved owners, no unstaged changes and the
successful cached-whitespace receipt. Both attempts are retained in the logs.
No compiling source changed after the final full binaries.
