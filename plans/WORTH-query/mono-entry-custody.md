# Slice 3 entry custody and cause corrections

B1 and findings 1–6, 8 and 9 are addressed on top of `2b87991577`. Finding 7 remains incomplete because its expired-authentication federation court needs a clock-owner ruling outside this lane; the exact conflict is recorded below. This is not a full-round approval. Work stops before Slice 4. Finding 3 delivers the unit-kind/full-cause representation; its five measured source families remove 17658 of the previous 26274-byte growth, leaving 8616 bytes above certified Slice 3. The approximate 26 KB target is only partly recovered. The governing engine-once design remains read-only. No installation migration, compatibility path, crate role, profile, MSRV or changes to existing public Decl/Host function signatures or bounds are made.

## Boundaries and shape

Commit bodies now require private custody of the exact owned program and runtime. `ReadmittedCommitProgram` carries installation validation before funding; `CurrentCommitProgram` also borrows the exact advancement. A sealed `CommitCarrier` accepts exactly the seven existing effect/lifecycle/delegation/revocation carriers. Callers cannot construct, clone or extract the raw program. The old raw preparation request and body paths are deleted; thin raw entry signatures validate once and hand the checked value to the only body.

Same-moment forwarding carries immutable runtime, brand, seal and input-type facts. Opening/funding advancement can change lapse, so the inner transition refreshes concrete lapse/phase validation; preparation refreshes lapse again before program-fact work. Both moments have ordering courts. The comparison and authority composition remain non-generic core methods; the generic factories only extract the brand/core and wrap the exact carrier or typed refusal.

Fresh source preparation requires `CheckedSourceAdmission`; bind and pending consume require `CheckedMutableSourceAdmission`, with exact shared/unique loans of the runtime and admission. The existing move-only prepared-source value continues to carry its already-checked admission identity for pure fact attachment. Source entry composition calls the existing admitted-operation projection-entry helper. No source retention, affinity work, fact read or admission mutation precedes ownership, seal, type and lapse at the fresh doors.

`WorthQuerySourceExpectationDenialKind::AdmissionAuthorization` is unit-only. The concrete denial owns the complete authorization refusal behind `authorization_denial()`. Its owner supplies a typed internal inconsistency if constructed without an authorization cause. Producer cutoff, output availability and publication recovery match every source kind explicitly and preserve the exact new cause. This public cause/reader crosses the Decl/Host reexports and is documented in `docs/capability-index.md`; the old unchanged-public-surface claim is corrected. No AspectShape form is added.

The runtime-order corrections move recovery handle expiry ahead of admission authority, matching redispatch, and keep history model ownership before lapse. Transaction production ownership gates were already correct after the preceding certification; finding 7’s additional second-door courts remain incomplete for the reason below.

## Courts and independent oracles

| Boundary | Court | Oracle |
|---|---|---|
| Commit producers | `foreign_installed_seal_is_stale_at_commit_validation_before_work` and `..._in_advancement_before_work` | A genuine other installed seal yields literal `StaleInstalledOperation`, with zero work. Both own producer calls are mutated separately. |
| Commit/source consuming doors | 35 actual caller omissions listed below | One compiler invocation emits a primary E0308/E0599 at every consuming function. Signatures remain intact in the batch, so each raw value fails its own typed-door contract. The compiler is independent of runtime validation. |
| Funding and preparation moments | `entry_custody_refreshes_lapse_after_advancement_is_funded`; `entry_custody_preparation_refreshes_lapse_before_program_fact_work` | Literal Cancelled refusal after funding, and Cancelled before a deliberately inconsistent fact route. Source/consult/charge observations independently measure absence of work. |
| Source fresh entries | `entry_custody_lapsed_source_bind_is_refused_before_work`, `...source_prepare...`, `...pending_consume...` | Literal Cancelled at all three doors; unchanged partition and zero fact visits/consults/charges. Fresh and input-type controls remain. Separate bind, prepare and pending foreign-seal courts catch their producers. |
| Source cause owner | `entry_custody_source_authorization_kind_is_unit_and_keeps_the_original_cause` | Unit copied kind plus literal exact original authorization cause; reader corruption is caught. |
| Producer/output cause consumers | `entry_custody_producer_source_cancellation_and_expiry_keep_their_causes`; `entry_custody_output_source_cancellation_and_expiry_keep_their_causes` | Literal Cancelled and RequestAuthorization(ExpiredAuthentication), without copying classifier logic. |
| Publication actual Source branch | `entry_custody_cancelled_source_dependency_keeps_retryable_recovery` | A real authenticated Host source binding refuses cancellation. The real denial enters the production-owned Host From conversion, preserving Dependencies::Source; Publication recovery classification must be literal Retryable. This proves the consumer boundary, not end-to-end retry execution. |
| History ownership | `entry_custody_foreign_history_model_precedes_authentication_lapse` | Real distinct roots/models and real receiver expiry; literal ForeignModel fresh and expired. Moving the model gate yields ExpiredAuthentication and fails. |
| Handle custody twins | `entry_custody_expired_handle_precedes_foreign_seal_at_recovery_authority_issuance`; `..._at_redispatch` | A fresh handle control detects StaleInstalledOperation. The owned wall clock then expires the handle: issuance yields Expired, redispatch FreshAuthority(Expired). Each local ordering mutation fails. |
| Existing transaction door | `transaction_application_mismatch_precedes_fresh_and_lapsed_admission_checks` and existing branch court | Real-clock sleeps are removed. Fresh-before and literal ExpiredAuthentication-after setup assertions verify the receiver control; existing literal transaction refusals remain. |

## Mutation results

The 16 retained isolated runtime mutants compiled successfully and produced 22 intended named test failures. Three additional transaction mutants produced five failures in newly added courts later rejected by BC9002; those courts and their fixture-only changes were removed, so those results do not certify finding 7. The fixture-expiry mutation checks attenuation setup; it is not a removed production guard. Every mutant restores exact source bytes. The separate compiler batch blocks all 35 actual caller omissions with 55 diagnostics (50 E0599, 5 E0308), restoring all 16 modified files. The reviewer independently matched the patches, assertion failures, compiler primary spans and restoration in ignored logs.

| Isolated mutation | Failed tests | Command |
|---|---:|---|
| `source-cause-retention` | 1 | `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --lib entry_custody_source_authorization_kind_is_unit_and_keeps_the_original_cause -- --test-threads=1` |
| `clock-lapse-control` | 1 | `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --lib transaction_application_mismatch_precedes_fresh_and_lapsed_admission_checks -- --test-threads=1` |
| `handle-redispatch` | 1 | `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --lib entry_custody_expired_handle_precedes_foreign_seal_at_redispatch -- --test-threads=1` |
| `commit-producer-before-funding` | 1 | `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --lib foreign_installed_seal_is_stale_at_commit_validation_before_work -- --test-threads=1` |
| `commit-producer-in-advancement` | 1 | `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --lib foreign_installed_seal_is_stale_at_commit_validation_in_advancement_before_work -- --test-threads=1` |
| `source-shared-producer` | 1 | `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --lib entry_custody_foreign_seal_is_refused_at_source_preparation -- --test-threads=1` |
| `source-unique-producer` | 2 | `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --lib foreign_installed_seal_is_stale_at_ -- --test-threads=1` |
| `source-producer` | 4 | `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --lib source_admission -- --test-threads=1` |
| `source-lapse` | 3 | `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --lib entry_custody_lapsed_ -- --test-threads=1` |
| `funded-lapse` | 1 | `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --lib entry_custody_refreshes_lapse_after_advancement_is_funded -- --test-threads=1` |
| `preparation-lapse` | 1 | `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --lib entry_custody_preparation_refreshes_lapse_before_program_fact_work -- --test-threads=1` |
| `producer-cause` | 1 | `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --lib entry_custody_producer_source_ -- --test-threads=1` |
| `output-cause` | 1 | `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --lib entry_custody_output_source_ -- --test-threads=1` |
| `history-model` | 1 | `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --lib entry_custody_foreign_history_model_precedes_authentication_lapse -- --test-threads=1` |
| `handle-issuance` | 1 | `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --lib entry_custody_expired_handle_precedes_foreign_seal_at_recovery_authority_issuance -- --test-threads=1` |
| `publication-cause` | 1 | `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-court-query-decl-host --test worth_query_court_query_decl_host entry_custody_cancelled_source_dependency_ -- --test-threads=1` |

The mutation lines and compiler spans below describe the exact source at the time each mutation ran; exact patches and restoration hashes remain in the ignored run records. Later changes only narrow two helper visibilities and format two courts; the checked-value producers, consuming contracts and constructor privacy are unchanged. The reviewer rechecked those differences and did not require repeating the omission batch. Historical restoration hashes are not claimed to authenticate every final byte. Ignored logs retain exact unified mutations, named failures, commands and compiler primary diagnostics. No source fingerprint or mutation receipt ledger is added to the repository. These are runtime mutation failures and compile-level omissions, respectively; they are not failures of the restored implementation.

| Consuming function | Omitted factory line | Primary compiler error |
|---|---:|---|
| `workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/provider_execution/entry.rs::compare_and_commit_conditional_operation` | 118 | E0599 |
| `workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/provider_execution/entry/application_commit.rs::compare_and_commit_application_in_advancement_with_history` | 56 | E0599 |
| `workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/provider_execution/entry/application_commit.rs::compare_and_commit_application_in_advancement` | 33 | E0599 |
| `workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/provider_execution/entry/application_commit.rs::compare_and_commit_application` | 14 | E0599 |
| `workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/provider_execution/entry/output_source.rs::compare_and_commit_application_for_program_action` | 87 | E0599 |
| `workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/provider_execution/entry/output_source.rs::compare_and_commit_application_for_program_output_producer` | 56 | E0599 |
| `workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/provider_execution/entry/output_source.rs::compare_and_commit_application_for_required_output_source` | 22 | E0599 |
| `workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/provider_execution/delegation_activation.rs::compare_and_commit_capability_delegation_for_program` | 71 | E0599 |
| `workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/provider_execution/delegation_activation.rs::compare_and_commit_capability_delegation` | 23 | E0599 |
| `workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/provider_execution/capability_revocation.rs::compare_and_commit_capability_revocation_for_program` | 71 | E0599 |
| `workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/provider_execution/capability_revocation.rs::compare_and_commit_capability_revocation` | 23 | E0599 |
| `workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_installation/program/specialized_action.rs::compare_and_commit_capability_revocation` | 332 | E0599 |
| `workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_installation/program/specialized_action.rs::compare_and_commit_capability_delegation` | 286 | E0599 |
| `workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_installation/program/specialized_action.rs::compare_and_commit_mandatory_review` | 236 | E0599 |
| `workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_installation/program/specialized_action.rs::compare_and_commit_elevation_close` | 186 | E0599 |
| `workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_installation/program/specialized_action.rs::compare_and_commit_elevation_approval` | 136 | E0599 |
| `workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_installation/program/specialized_action.rs::compare_and_commit_elevation_request` | 87 | E0599 |
| `workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/workflow_definition_program/publication.rs::compare_and_commit_workflow_definition_publication` | 203 | E0599 |
| `workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/workflow_definition_program/retirement/outcome.rs::compare_and_commit_workflow_definition_retirement` | 104 | E0599 |
| `workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/provider_execution/elevation_lifecycle/elevation_request.rs::compare_and_commit_elevation_request_for_program` | 65 | E0599 |
| `workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/provider_execution/elevation_lifecycle/elevation_request.rs::compare_and_commit_elevation_request` | 16 | E0599 |
| `workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/provider_execution/elevation_lifecycle/elevation_approval.rs::compare_and_commit_elevation_approval_for_program` | 66 | E0599 |
| `workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/provider_execution/elevation_lifecycle/elevation_approval.rs::compare_and_commit_elevation_approval` | 16 | E0599 |
| `workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/provider_execution/elevation_lifecycle/elevation_close.rs::compare_and_commit_elevation_close_for_program` | 66 | E0599 |
| `workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/provider_execution/elevation_lifecycle/elevation_close.rs::compare_and_commit_elevation_close` | 16 | E0599 |
| `workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/provider_execution/elevation_lifecycle/mandatory_review.rs::compare_and_commit_mandatory_review_for_program` | 66 | E0599 |
| `workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/provider_execution/elevation_lifecycle/mandatory_review.rs::compare_and_commit_mandatory_review` | 16 | E0599 |
| `workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/provider_execution/elevation_lifecycle/installed_readmission.rs::readmit_mandatory_review_installation` | 75 | E0308 |
| `workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/provider_execution/elevation_lifecycle/installed_readmission.rs::readmit_elevation_close_installation` | 57 | E0308 |
| `workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/provider_execution/elevation_lifecycle/installed_readmission.rs::readmit_elevation_approval_installation` | 39 | E0308 |
| `workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/provider_execution/elevation_lifecycle/installed_readmission.rs::readmit_elevation_request_installation` | 21 | E0308 |
| `workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_query/observed_source/prepared_expectation.rs::bind_checked_source_expectation` | 290 | E0599 |
| `workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_query/observed_source/prepared_expectation.rs::prepare_application_source_expectation` | 275 | E0599 |
| `workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_query/observed_source/prepared_expectation/pending.rs::consume_into` | 30 | E0599 |
| `workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/provider_execution/phase/prepared/entry_custody/progression.rs::compare_and_commit_application_inner_with_currentness_and_aftermath` | 33 | E0308 |

## Scoped checks

All Query commands use the all-ones `WORTH_IMPLEMENTATION_BUILD_ID`, `CARGO_TARGET_DIR=C:/worth-build-cache/target-cut-pub-mono` and `PYTHONUTF8=1`. Long Cargo commands run in background processes into logs, are awaited, and are read after completion. No workspace-wide build or test runs. The following final checks ran on the integrated working source; intermediate commit SHAs are not claimed to have been separately built.

| Exact command | Result |
|---|---|
| `cargo fmt --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution -p worth-query-publication -p worth-query-court-query-decl-host --check` | Exit 0 |
| `cargo clippy --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution -p worth-query-publication -p worth-query-court-query-decl-host --all-targets --message-format=json` | Exit 0 |
| `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-installation --lib` | 371 passed / 0 failed / 0 ignored / 0 filtered, 5.0 s |
| `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --lib` | 2270 passed / 0 failed / 1 ignored / 0 filtered, 723.22 s |
| `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --lib erased_core_forgery -- --test-threads=1` | 25 passed / 0 failed / 0 ignored / 2246 filtered, 6.73 s |
| `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-publication --lib` | 28 passed / 0 failed / 0 ignored / 0 filtered, 0.0 s |
| `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-court-decl-host --test worth_query_court_decl_host` | 325 passed / 0 failed / 2 ignored / 0 filtered, 23.39 s |
| `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-court-query-decl-host --test worth_query_court_query_decl_host` | 341 passed / 0 failed / 2 ignored / 0 filtered, 127.04 s |
| `cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .` | Exit 0 |
| `cargo run --manifest-path tools/agent-context/Cargo.toml -- check` | Exit 0 |
| `'C:/Program Files/Git/bin/bash.exe' scripts/ci/check_workspace_rust_line_caps.sh dirty` | Exit 0 |
| `git diff --check origin/master...HEAD` | Exit 0 |
| `git diff --check` | Exit 0 |

Clippy: 87 Execution warning emissions / 38 distinct, identical to the certified correction baseline; zero new warning keys and zero warnings in touched files. Seven Relational warnings and one document-retention warning remain in untouched files. No allow attributes are added. Six forwarding accessors made dead by custody are deleted; meaningful cause fields remain intact. The final function advisory scans 81 files, reports 39 candidates and zero scan errors. The reviewer independently verified every advisory difference: removal of the blocked fixture accounts for 13 removed candidates; unchanged 62-line `request_effects` is context-only, surfaced by getter deletion; the genuine consumer court grows from 86 to 88 lines for the lawful owner conversion. No new split is justified.

A direct mounted-file rustfmt check initially found one malformed adapter; formatting it and expanding the new dense producer court preserves all lexical tokens/literals apart from optional trailing commas. Explicit checking of all 81 touched Rust files passes. The moved occurrence-read helpers are restricted to provider_execution, their actual common owner. Initial focused checks before boundary correction: Execution entry_custody 17 passed / 2259 filtered (including five newly added transaction courts later rejected and removed); source_admission 9 / 2267 filtered; erased_core_forgery 25 / 2251 filtered; Publication 28; actual consumer court 1 / 342 filtered. All had zero failures/ignored. Final broader courts above independently rerun the lawful restored implementation. Pre-commit formatter, Clippy, boundary/context, dirty caps and diff gates are replayed against unchanged integrated source for each separately staged finding; index whitespace is checked too.

## Earlier failed runs and closure

- Exit 101: two compile errors, E0061 and E0599. Repaired forwarding arguments and the duplicated unwrap.

- Exit 101: nine owner-test visibility/placement errors. Moved preparation court under its owner and exposed the existing operation fixture within PrimaryGraph.

- Exit 101: one missing request-extension trait import in the consumer court. Added the import.

- Three failed module-resolution invocations: one program_required path and two authentication_expiry path attempts. Added correct explicit module paths; later fmt and fmt --check passed.

- First run:10 passed/5 failed. Incomplete real roster, report read before advancement ended, and wall hold mistaken for monotonic expiry. Second run:12 passed/5 failed, all five nested advancement fixture errors. Complete roster, after-scope report assertion, owned deadline attenuation and real outer serial request repaired these; restored focused run17 passed/0 failed.

- First driver stopped because the source-producer filter selected two seal courts but expected three; both selected tests failed, source restored. Filter changed to source_admission, covering prepare too. Historical driver caught all19 mutants; certification retains16mutants/22failures after rejecting finding7 courts.

The initial boundary check failed with BC2001_BAND_DEPENDENCY_VIOLATION (the new consumer directly named Execution publication_boundary) and BC9002_TEST_TOPOLOGY (the new transaction source court constructed ProgramApplicationRuntime and ExecutionRequest). Publication now owns concrete lossless From conversions, used by production retain/combine and the Host consumer court. The five newly added source-federation courts and fixture-only changes were removed; no pre-existing court was weakened. A line-cap preflight initially failed to launch with WSL `execvpe(/bin/bash) failed: No such file or directory`; the installed Git Bash runs the same guard successfully.

`entry-custody-check.log`:
```text
error[E0061]: this method takes 2 arguments but 4 arguments were supplied
error[E0599]: no method named `into_inner` found for struct `WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>` in the current scope
error: could not compile `worth-query-execution` (lib) due to 2 previous errors; 26 warnings emitted
```

`entry-custody-first-clippy.log`:
```text
E0425: cannot find type `ProgramRequiredOperation` in this scope
E0425: cannot find type `ProgramRequiredOperation` in this scope
E0425: cannot find type `ProgramRequiredOperation` in this scope
E0603: module `provider_execution` is private
E0603: module `provider_execution` is private
E0616: field `read_set` of struct `domain_computation::primary_graph::application_attempt::effect_program::model::WorthQueryApplicationEffectProgram` is private
E0616: field `read_set` of struct `domain_computation::primary_graph::application_attempt::effect_program::model::WorthQueryApplicationEffectProgram` is private
E0616: field `read_set` of struct `domain_computation::primary_graph::application_attempt::effect_program::model::WorthQueryApplicationEffectProgram` is private
E0616: field `read_set` of struct `domain_computation::primary_graph::application_attempt::effect_program::model::WorthQueryApplicationEffectProgram` is private
error: could not compile `worth-query-execution` (lib test) due to 9 previous errors; 3 warnings emitted
```

`entry-custody-second-clippy.log`:
```text
E0599: no method named `request` found for reference `&WorthQueryPrimaryGraphApplicationRuntime<CanonicalSchema>` in the current scope
error: could not compile `worth-query-court-query-decl-host` (test "worth_query_court_query_decl_host") due to 1 previous error
```

`entry-custody-focused-execution.log`:
```text
test result: FAILED. 10 passed; 5 failed; 0 ignored; 0 measured; 2259 filtered out; finished in 5.53s
error: test failed, to rerun pass `-p worth-query-execution --lib`
```

`entry-custody-repaired-execution.log`:
```text
test result: FAILED. 12 passed; 5 failed; 0 ignored; 0 measured; 2259 filtered out; finished in 3.91s
error: test failed, to rerun pass `-p worth-query-execution --lib`
```

`entry-custody-mutation-run.err`:
```text
RuntimeError: Mutant was not caught as a test failure: source-producer
```

Counts before that boundary correction (not final certification):

- `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-installation --lib`: [{'outcome': 'ok', 'passed': 371, 'failed': 0, 'ignored': 0, 'measured': 0, 'filtered': 0, 'seconds': 3.17}]

- `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --lib`: [{'outcome': 'ok', 'passed': 2275, 'failed': 0, 'ignored': 1, 'measured': 0, 'filtered': 0, 'seconds': 698.9}]

- `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --lib erased_core_forgery -- --test-threads=1`: [{'outcome': 'ok', 'passed': 25, 'failed': 0, 'ignored': 0, 'measured': 0, 'filtered': 2251, 'seconds': 11.01}]

- `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-publication --lib`: [{'outcome': 'ok', 'passed': 28, 'failed': 0, 'ignored': 0, 'measured': 0, 'filtered': 0, 'seconds': 0.0}]

- `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-court-decl-host --test worth_query_court_decl_host`: [{'outcome': 'ok', 'passed': 325, 'failed': 0, 'ignored': 2, 'measured': 0, 'filtered': 0, 'seconds': 26.02}]

- `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-court-query-decl-host --test worth_query_court_query_decl_host`: [{'outcome': 'ok', 'passed': 341, 'failed': 0, 'ignored': 2, 'measured': 0, 'filtered': 0, 'seconds': 183.07}]

The fully passing, subsequently rejected hoist variant also ran all 13 scoped commands (exit 0 each):

- `cargo fmt --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution -p worth-query-publication -p worth-query-court-query-decl-host --check`: Exit 0.

- `cargo clippy --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution -p worth-query-publication -p worth-query-court-query-decl-host --all-targets --message-format=json`: Exit 0.

- `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-installation --lib`: 371 passed / 0 failed / 0 ignored / 0 filtered, 4.41 s.

- `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --lib`: 2270 passed / 0 failed / 1 ignored / 0 filtered, 583.22 s.

- `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --lib erased_core_forgery -- --test-threads=1`: 25 passed / 0 failed / 0 ignored / 2246 filtered, 6.5 s.

- `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-publication --lib`: 28 passed / 0 failed / 0 ignored / 0 filtered, 0.0 s.

- `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-court-decl-host --test worth_query_court_decl_host`: 325 passed / 0 failed / 2 ignored / 0 filtered, 24.45 s.

- `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-court-query-decl-host --test worth_query_court_query_decl_host`: 341 passed / 0 failed / 2 ignored / 0 filtered, 131.77 s.

- `cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .`: Exit 0.

- `cargo run --manifest-path tools/agent-context/Cargo.toml -- check`: Exit 0.

- `'C:/Program Files/Git/bin/bash.exe' scripts/ci/check_workspace_rust_line_caps.sh dirty`: Exit 0.

- `git diff --check origin/master...HEAD`: Exit 0.

- `git diff --check`: Exit 0.

## Readings I chose

- Authentication expiry uses Instant; authorization_time.hold controls SystemTime. A cfg(test) control on the actual admitted receiver attenuates only its own deadline to Instant::now(); an effect program forwards to its own admission. The real lapse checker still produces ExpiredAuthentication. No sleep, production clock change, global override or new caller-list exemption.

- Each finding is committed separately by staging index-only subsets of the final integrated tree. The checks listed here ran on the integrated working source; intermediate commit SHAs are not claimed to have been separately built.

- Seal, runtime, brand and type facts are immutable inside exact private custody; same-moment repeated seal lookups are removed. Cancellation/lapse may change after funding, so concrete lapse checks remain after funding and immediately before preparation work.

- This is one Slice 3 correction round with separately reviewable finding commits. The two required sync branches were fetched and merged at the start; both were already ancestors. No other worktree or repository is changed.

## Measurement

The unchanged `scripts/quality/query_monomorphization.py` selects the canonical fixture archive from successful scoped Cargo artifact messages. Independent llvm-nm/readobj symbol counts and llvm-size/section byte counts agree at each new point. Exact commands, artifact/raw-report/assembly hashes, owner/module counts, slopes and rankings are in `mono-entry-custody-measurement.json`. Both original baseline raw-report checksums were verified before comparison. This is the same opt-level-0, debug-info-0 archive method; no build settings change.

| Point | Exported symbols | Archive bytes | Code bytes | Slope all owners | Slope Execution |
|---|---:|---:|---:|---:|---:|
| Certified Slice 3 | 150256 | 619896574 | 44872006 | 1792398.212355 | 1375552.419328 |
| Previous corrections | 150287 | 620056630 | 44925524 | 1797565.608913 | 1380356.120015 |
| Experiment A: original getter | 150471 | 620985096 | 44933063 | 1797438.497199 | 1380142.099883 |
| Experiment B: rejected hoist | 150471 | 621001940 | 44937111 | 1797614.497199 | 1380318.099883 |
| Accepted final | 150471 | 620985096 | 44933063 | 1797438.497199 | 1380142.099883 |

Against certified Slice 3: `{"exported_symbols": 215, "rlib_bytes": 1088522, "code_bytes": 61057}`; slope delta `{"all_owners": 5040.284844, "execution": 4589.680556}`.

Against previous corrections: `{"exported_symbols": 184, "rlib_bytes": 928466, "code_bytes": 7539}`; slope delta `{"all_owners": -127.111714, "execution": -214.020132}`.

Against isolated experiment A to rejected experiment B: `{"exported_symbols": 0, "rlib_bytes": 16844, "code_bytes": 4048}`; slope delta `{"all_owners": 176.0, "execution": 176.0}`.

The slope is sum(bytes/copies) over normalized families with at least seven copies, a fixture proxy rather than an independently measured multi-application derivative. Full absolute and delta owner/module counts are retained. The five requested main source families total 225721 bytes previously and 208063 after the unit-kind change: -17658 bytes. Against certified Slice 3 their total remains +8616 bytes. These are measured generic-family totals, not a claim that all residual layout/codegen growth was removed. No growth test is added: reproducing this fixture archive is a large build, so the committed measurement is the growth record.

### Source families and progression

| Point | Family | Copies | Bytes |
|---|---|---:|---:|
| certified_slice3 | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::fact_conversion::PreparedSourceFactMaterialization<>::into_facts` | 11 | 76087 |
| certified_slice3 | `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::checked_source_affinity::<>` | 23 | 66211 |
| certified_slice3 | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSource<>::validate_fact_retention` | 11 | 34540 |
| certified_slice3 | `worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::progress_application_commit::<>` | 23 | 31648 |
| certified_slice3 | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSource<>::admit_fact_materialization` | 11 | 17644 |
| certified_slice3 | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSource<>::validate_affinity` | 5 | 4965 |
| certified_slice3 | `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::checked_source_affinity::<>::{closure#7}` | 23 | 2162 |
| certified_slice3 | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::fact_conversion::PreparedSourceFactMaterialization<>::into_facts::{closure#2}` | 11 | 1364 |
| certified_slice3 | `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::checked_source_affinity::<>::{closure#3}` | 23 | 1265 |
| certified_slice3 | `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::checked_source_affinity::<>::{closure#0}` | 23 | 1035 |
| certified_slice3 | `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::checked_source_affinity::<>::{closure#4}` | 23 | 1035 |
| certified_slice3 | `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::checked_source_affinity::<>::{closure#5}` | 23 | 1035 |
| certified_slice3 | `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::checked_source_affinity::<>::{closure#1}` | 23 | 1035 |
| certified_slice3 | `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::checked_source_affinity::<>::{closure#2}` | 23 | 897 |
| certified_slice3 | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSource<>::validate_fact_retention::{closure#0}` | 11 | 858 |
| certified_slice3 | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSource<>::admit_fact_materialization::{closure#0}` | 11 | 429 |
| certified_slice3 | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSource<>::admit_fact_materialization::{closure#1}` | 11 | 429 |
| certified_slice3 | `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::checked_source_affinity::<>::{closure#6}` | 23 | 368 |
| certified_slice3 | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSource<>::validate_fact_retention::{closure#2}` | 11 | 297 |
| certified_slice3 | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::fact_conversion::PreparedSourceFactMaterialization<>::into_facts::{closure#0}` | 11 | 286 |
| certified_slice3 | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::fact_conversion::PreparedSourceFactMaterialization<>::into_facts::{closure#1}` | 11 | 286 |
| certified_slice3 | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSource<>::validate_affinity::{closure#0}` | 5 | 245 |
| certified_slice3 | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSource<>::validate_fact_retention::{closure#3}` | 11 | 99 |
| certified_slice3 | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSource<>::validate_fact_retention::{closure#1}` | 11 | 99 |
| certified_slice3 | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSource<>::admit_fact_materialization::{closure#2}` | 11 | 99 |
| previous_corrections | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::fact_conversion::PreparedSourceFactMaterialization<>::into_facts` | 11 | 88231 |
| previous_corrections | `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::checked_source_affinity::<>` | 23 | 73203 |
| previous_corrections | `worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::progress_application_commit::<>` | 23 | 40112 |
| previous_corrections | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSource<>::validate_fact_retention` | 11 | 38236 |
| previous_corrections | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSource<>::admit_fact_materialization` | 11 | 19756 |
| previous_corrections | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSource<>::validate_affinity` | 5 | 6295 |
| previous_corrections | `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::checked_source_affinity::<>::{closure#7}` | 23 | 2162 |
| previous_corrections | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::fact_conversion::PreparedSourceFactMaterialization<>::into_facts::{closure#2}` | 11 | 1364 |
| previous_corrections | `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::checked_source_affinity::<>::{closure#3}` | 23 | 1265 |
| previous_corrections | `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::checked_source_affinity::<>::{closure#0}` | 23 | 1150 |
| previous_corrections | `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::checked_source_affinity::<>::{closure#4}` | 23 | 1150 |
| previous_corrections | `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::checked_source_affinity::<>::{closure#5}` | 23 | 1150 |
| previous_corrections | `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::checked_source_affinity::<>::{closure#1}` | 23 | 1150 |
| previous_corrections | `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::checked_source_affinity::<>::{closure#2}` | 23 | 1012 |
| previous_corrections | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSource<>::validate_fact_retention::{closure#0}` | 11 | 858 |
| previous_corrections | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSource<>::validate_affinity::{closure#0}` | 5 | 520 |
| previous_corrections | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSource<>::admit_fact_materialization::{closure#0}` | 11 | 484 |
| previous_corrections | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSource<>::admit_fact_materialization::{closure#1}` | 11 | 484 |
| previous_corrections | `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::checked_source_affinity::<>::{closure#6}` | 23 | 368 |
| previous_corrections | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSource<>::validate_fact_retention::{closure#2}` | 11 | 297 |
| previous_corrections | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::fact_conversion::PreparedSourceFactMaterialization<>::into_facts::{closure#0}` | 11 | 286 |
| previous_corrections | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::fact_conversion::PreparedSourceFactMaterialization<>::into_facts::{closure#1}` | 11 | 286 |
| previous_corrections | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSource<>::validate_fact_retention::{closure#3}` | 11 | 99 |
| previous_corrections | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSource<>::validate_fact_retention::{closure#1}` | 11 | 99 |
| previous_corrections | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSource<>::admit_fact_materialization::{closure#2}` | 11 | 99 |
| counterfactual_getter | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::fact_conversion::PreparedSourceFactMaterialization<>::into_facts` | 11 | 80135 |
| counterfactual_getter | `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::checked_source_affinity::<>` | 23 | 68787 |
| counterfactual_getter | `worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::progress_application_commit::<>` | 23 | 40112 |
| counterfactual_getter | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSource<>::validate_fact_retention` | 11 | 35772 |
| counterfactual_getter | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSource<>::admit_fact_materialization` | 11 | 18524 |
| counterfactual_getter | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSource<>::validate_affinity` | 5 | 4845 |
| counterfactual_getter | `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::checked_source_affinity::<>::{closure#7}` | 23 | 2162 |
| counterfactual_getter | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::fact_conversion::PreparedSourceFactMaterialization<>::into_facts::{closure#2}` | 11 | 1364 |
| counterfactual_getter | `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::checked_source_affinity::<>::{closure#3}` | 23 | 1265 |
| counterfactual_getter | `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::checked_source_affinity::<>::{closure#0}` | 23 | 1035 |
| counterfactual_getter | `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::checked_source_affinity::<>::{closure#4}` | 23 | 1035 |
| counterfactual_getter | `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::checked_source_affinity::<>::{closure#5}` | 23 | 1035 |
| counterfactual_getter | `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::checked_source_affinity::<>::{closure#1}` | 23 | 1035 |
| counterfactual_getter | `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::checked_source_affinity::<>::{closure#2}` | 23 | 897 |
| counterfactual_getter | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSource<>::validate_fact_retention::{closure#0}` | 11 | 858 |
| counterfactual_getter | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSource<>::admit_fact_materialization::{closure#0}` | 11 | 429 |
| counterfactual_getter | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSource<>::admit_fact_materialization::{closure#1}` | 11 | 429 |
| counterfactual_getter | `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::checked_source_affinity::<>::{closure#6}` | 23 | 368 |
| counterfactual_getter | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSource<>::validate_fact_retention::{closure#2}` | 11 | 297 |
| counterfactual_getter | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::fact_conversion::PreparedSourceFactMaterialization<>::into_facts::{closure#0}` | 11 | 286 |
| counterfactual_getter | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::fact_conversion::PreparedSourceFactMaterialization<>::into_facts::{closure#1}` | 11 | 286 |
| counterfactual_getter | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSource<>::validate_affinity::{closure#0}` | 5 | 245 |
| counterfactual_getter | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSource<>::admit_fact_materialization::{closure#2}` | 11 | 99 |
| counterfactual_getter | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSource<>::validate_fact_retention::{closure#3}` | 11 | 99 |
| counterfactual_getter | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSource<>::validate_fact_retention::{closure#1}` | 11 | 99 |
| after | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::fact_conversion::PreparedSourceFactMaterialization<>::into_facts` | 11 | 80135 |
| after | `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::checked_source_affinity::<>` | 23 | 68787 |
| after | `worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::progress_application_commit::<>` | 23 | 40112 |
| after | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSource<>::validate_fact_retention` | 11 | 35772 |
| after | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSource<>::admit_fact_materialization` | 11 | 18524 |
| after | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSource<>::validate_affinity` | 5 | 4845 |
| after | `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::checked_source_affinity::<>::{closure#7}` | 23 | 2162 |
| after | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::fact_conversion::PreparedSourceFactMaterialization<>::into_facts::{closure#2}` | 11 | 1364 |
| after | `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::checked_source_affinity::<>::{closure#3}` | 23 | 1265 |
| after | `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::checked_source_affinity::<>::{closure#0}` | 23 | 1035 |
| after | `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::checked_source_affinity::<>::{closure#4}` | 23 | 1035 |
| after | `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::checked_source_affinity::<>::{closure#5}` | 23 | 1035 |
| after | `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::checked_source_affinity::<>::{closure#1}` | 23 | 1035 |
| after | `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::checked_source_affinity::<>::{closure#2}` | 23 | 897 |
| after | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSource<>::validate_fact_retention::{closure#0}` | 11 | 858 |
| after | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSource<>::admit_fact_materialization::{closure#0}` | 11 | 429 |
| after | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSource<>::admit_fact_materialization::{closure#1}` | 11 | 429 |
| after | `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::checked_source_affinity::<>::{closure#6}` | 23 | 368 |
| after | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSource<>::validate_fact_retention::{closure#2}` | 11 | 297 |
| after | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::fact_conversion::PreparedSourceFactMaterialization<>::into_facts::{closure#0}` | 11 | 286 |
| after | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::fact_conversion::PreparedSourceFactMaterialization<>::into_facts::{closure#1}` | 11 | 286 |
| after | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSource<>::validate_affinity::{closure#0}` | 5 | 245 |
| after | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSource<>::admit_fact_materialization::{closure#2}` | 11 | 99 |
| after | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSource<>::validate_fact_retention::{closure#3}` | 11 | 99 |
| after | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSource<>::validate_fact_retention::{closure#1}` | 11 | 99 |
| rejected_getter_hoist | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::fact_conversion::PreparedSourceFactMaterialization<>::into_facts` | 11 | 80135 |
| rejected_getter_hoist | `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::checked_source_affinity::<>` | 23 | 68787 |
| rejected_getter_hoist | `worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::progress_application_commit::<>` | 23 | 44160 |
| rejected_getter_hoist | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSource<>::validate_fact_retention` | 11 | 35772 |
| rejected_getter_hoist | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSource<>::admit_fact_materialization` | 11 | 18524 |
| rejected_getter_hoist | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSource<>::validate_affinity` | 5 | 4845 |
| rejected_getter_hoist | `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::checked_source_affinity::<>::{closure#7}` | 23 | 2162 |
| rejected_getter_hoist | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::fact_conversion::PreparedSourceFactMaterialization<>::into_facts::{closure#2}` | 11 | 1364 |
| rejected_getter_hoist | `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::checked_source_affinity::<>::{closure#3}` | 23 | 1265 |
| rejected_getter_hoist | `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::checked_source_affinity::<>::{closure#0}` | 23 | 1035 |
| rejected_getter_hoist | `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::checked_source_affinity::<>::{closure#4}` | 23 | 1035 |
| rejected_getter_hoist | `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::checked_source_affinity::<>::{closure#5}` | 23 | 1035 |
| rejected_getter_hoist | `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::checked_source_affinity::<>::{closure#1}` | 23 | 1035 |
| rejected_getter_hoist | `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::checked_source_affinity::<>::{closure#2}` | 23 | 897 |
| rejected_getter_hoist | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSource<>::validate_fact_retention::{closure#0}` | 11 | 858 |
| rejected_getter_hoist | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSource<>::admit_fact_materialization::{closure#0}` | 11 | 429 |
| rejected_getter_hoist | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSource<>::admit_fact_materialization::{closure#1}` | 11 | 429 |
| rejected_getter_hoist | `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::checked_source_affinity::<>::{closure#6}` | 23 | 368 |
| rejected_getter_hoist | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSource<>::validate_fact_retention::{closure#2}` | 11 | 297 |
| rejected_getter_hoist | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::fact_conversion::PreparedSourceFactMaterialization<>::into_facts::{closure#0}` | 11 | 286 |
| rejected_getter_hoist | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::fact_conversion::PreparedSourceFactMaterialization<>::into_facts::{closure#1}` | 11 | 286 |
| rejected_getter_hoist | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSource<>::validate_affinity::{closure#0}` | 5 | 245 |
| rejected_getter_hoist | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSource<>::admit_fact_materialization::{closure#2}` | 11 | 99 |
| rejected_getter_hoist | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSource<>::validate_fact_retention::{closure#3}` | 11 | 99 |
| rejected_getter_hoist | `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSource<>::validate_fact_retention::{closure#1}` | 11 | 99 |

The getter-only experiment uses identical source except for moving the graph getter before destructuring the owned running carrier. Qualified captures bind both disassemblies to their measured archive hashes. The hoist makes the family larger: 40112 to 44160 code bytes across the same 23 copies, +4048 bytes (+176 per copy), +16844 archive bytes. It is rejected; the accepted source keeps the original expression. This method sees no saving. The accepted 40112-byte family is unchanged against previous corrections and +8464 bytes (+368 per copy) against certified Slice 3. The only prior source change inside this function was a direct graph field borrow becoming the live Option getter. Its potentially unwinding call amid moved aggregates changes opt-level-0 ownership, stack and unwind bookkeeping, rather than adding a semantic operation algorithm. All 23 accepted instruction/relocation dumps equal the saved prior-correction disassembly; the rejected hoist has fewer instructions/calls yet larger encoded sections. This identifies the changed source and emitted structure; without the old certified assembly, it is not a claim that one category of EH bytes independently accounts for every original byte.

### Top 20 appeared or growing families against certified Slice 3

| Family | Before bytes/copies | After bytes/copies | Delta bytes | Instantiation scope and cause |
|---|---:|---:|---:|---|
| `worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::entry_custody::preparation::prepare_application_commit::<>` | 0/0 | 307648/23 | +307648 | Per operation/carrier and application: Existing generic commit/preparation body relocated behind the exact owned custody value; the raw body is deleted in the same round. Currentness/lapse refresh calls concrete core validation, while program-specific effects retain their original typed forms. |
| `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::prepared_expectation::admission::CheckedSourceAdmission<>::prepare::<>` | 0/0 | 53382/14 | +53382 | Per operation/query payload and application: Existing source body moved under an exact checked shared or unique loan; the old raw body is deleted. Fresh-entry authorization is in the concrete producer, not repeated in the moved generic body. |
| `worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::entry_custody::CurrentCommitProgram<>::compare_and_commit_application_with_causality` | 0/0 | 32361/23 | +32361 | Per operation/carrier and application: Existing generic commit/preparation body relocated behind the exact owned custody value; the raw body is deleted in the same round. Currentness/lapse refresh calls concrete core validation, while program-specific effects retain their original typed forms. |
| `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::prepared_expectation::admission::CheckedMutableSourceAdmission<>::consume_pending::<>` | 0/0 | 22185/9 | +22185 | Per operation/query payload and application: Existing source body moved under an exact checked shared or unique loan; the old raw body is deleted. Fresh-entry authorization is in the concrete producer, not repeated in the moved generic body. |
| `worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::entry_custody::CurrentCommitProgram<>::compare_and_commit_application_inner_with_currentness_and_aftermath` | 0/0 | 21827/23 | +21827 | Per operation/carrier and application: Existing generic commit/preparation body relocated behind the exact owned custody value; the raw body is deleted in the same round. Currentness/lapse refresh calls concrete core validation, while program-specific effects retain their original typed forms. |
| `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::prepared_expectation::admission::CheckedMutableSourceAdmission<>::bind::<>` | 0/0 | 16425/9 | +16425 | Per operation/query payload and application: Existing source body moved under an exact checked shared or unique loan; the old raw body is deleted. Fresh-entry authorization is in the concrete producer, not repeated in the moved generic body. |
| `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::validate_commit_program_in_advancement::<>` | 0/0 | 12075/23 | +12075 | Per operation/carrier and application: Thin exact-carrier producer extracts core and brand, calls the non-generic runtime authority composition, then wraps the checked carrier or preserves its typed refusal. The authority comparison and cause conversion bodies remain concrete. |
| `worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::entry_custody::CurrentCommitProgram<>::compare_and_commit_application_in_advancement_with_history` | 0/0 | 9366/14 | +9366 | Per operation/carrier and application: Existing generic commit/preparation body relocated behind the exact owned custody value; the raw body is deleted in the same round. Currentness/lapse refresh calls concrete core validation, while program-specific effects retain their original typed forms. |
| `worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::entry_custody::CurrentCommitProgram<>::compare_and_commit_application_for_program_output_producer` | 0/0 | 8918/14 | +8918 | Per operation/carrier and application: Existing generic commit/preparation body relocated behind the exact owned custody value; the raw body is deleted in the same round. Currentness/lapse refresh calls concrete core validation, while program-specific effects retain their original typed forms. |
| `worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::progress_application_commit::<>` | 31648/23 | 40112/23 | +8464 | Per operation and application: Inherited getter-based live access changed opt-level-0 ownership, unwind and stack code generation around aggregate moves. This round adds zero bytes to this family. All 23 accepted instruction/relocation dumps equal previous corrections. Rejected getter hoist adds 176 bytes per copy; no EH saving is claimed. |
| `worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::entry_custody::CurrentCommitProgram<>::compare_and_commit_application_for_program_action` | 0/0 | 7360/4 | +7360 | Per operation/carrier and application: Existing generic commit/preparation body relocated behind the exact owned custody value; the raw body is deleted in the same round. Currentness/lapse refresh calls concrete core validation, while program-specific effects retain their original typed forms. |
| `worth_query_installation::application_operation::installed::WorthQueryInstalledApplicationOperation<>::install_executable_operation` | 120588/26 | 127244/26 | +6656 | Per operation/binding and application: Unchanged installation generic body; all growth was already present in previous corrections, with zero additional bytes this round. |
| `worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::entry_custody::CurrentCommitProgram<>::compare_and_commit_application_for_required_output_source` | 0/0 | 6000/5 | +6000 | Per operation/carrier and application: Existing generic commit/preparation body relocated behind the exact owned custody value; the raw body is deleted in the same round. Currentness/lapse refresh calls concrete core validation, while program-specific effects retain their original typed forms. |
| `worth_query_installation::application_schema::installed::WorthQueryInstalledApplicationSchema<>::installed_mutation_binding::<>` | 30900/25 | 36900/25 | +6000 | Per operation/binding and application: Unchanged installation generic body; all growth was already present in previous corrections, with zero additional bytes this round. |
| `worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::entry_custody::forwarding::plain_publication_posture::<>` | 0/0 | 5382/23 | +5382 | Per operation and application: Existing generic posture forwarding moved under the checked owner; the old forwarding definition is deleted. No installed comparison or authorization conversion algorithm is added. |
| `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::validate_source_admission::<>` | 0/0 | 4494/14 | +4494 | Per operation/input/scope and application: Thin borrowed producer extracts core and brand, calls concrete runtime source validation, and constructs the exact shared loan. The concrete owner delegates to the existing projection-entry authority composition. |
| `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::fact_conversion::PreparedSourceFactMaterialization<>::into_facts` | 76087/11 | 80135/11 | +4048 | Per application-defined source query type (no operation type parameter): Existing Query-generic fact-materialization body carries the concrete source denial. Unit-kind storage reduces this family by 8096 bytes versus previous corrections; 4048 bytes of inherited/new carried-denial layout cost remain versus certified Slice 3. No new authorization comparison body is here. |
| `worth_query_installation::application_operation::binding::catalog::compile_application_mutation_catalog::<>` | 16919/7 | 20391/7 | +3472 | Per application/schema: Unchanged installation catalog generic body; all growth was already present in previous corrections, with zero additional bytes this round. |
| `worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::entry_custody::CurrentCommitProgram<>::compare_and_commit_application_with_output_observation` | 0/0 | 3122/14 | +3122 | Per operation/carrier and application: Existing generic commit/preparation body relocated behind the exact owned custody value; the raw body is deleted in the same round. Currentness/lapse refresh calls concrete core validation, while program-specific effects retain their original typed forms. |
| `worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::entry_custody::preparation::prepare_application_commit::<>::{closure#1}` | 0/0 | 2921/23 | +2921 | Per operation/carrier and application: Existing generic commit/preparation body relocated behind the exact owned custody value; the raw body is deleted in the same round. Currentness/lapse refresh calls concrete core validation, while program-specific effects retain their original typed forms. |

### Top 40 disappeared families against certified Slice 3

Disappeared family count: 22. Renamed body owners are distinguished from eliminated work by the growth table and absolute totals.

| Family | Copies | Bytes |
|---|---:|---:|
| `worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::commit_preparation::prepare_application_commit::<>` | 23 | 317009 |
| `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::compare_and_commit_application_with_causality::<>` | 23 | 32821 |
| `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::compare_and_commit_application_inner_with_currentness_and_aftermath::<>` | 23 | 21827 |
| `worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::entry::plain_publication_posture::<>` | 23 | 5382 |
| `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::compare_and_commit_application_with_output_observation::<>` | 14 | 4116 |
| `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::validate_commit_program::<>` | 23 | 3496 |
| `worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::commit_preparation::prepare_application_commit::<>::{closure#1}` | 23 | 2921 |
| `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::prepare_application_source_expectation::<>::{closure#0}` | 14 | 2604 |
| `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::bind_checked_source_expectation::<>::{closure#0}` | 9 | 1674 |
| `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::prepare_application_source_expectation::<>::{closure#0}::{closure#0}` | 14 | 952 |
| `worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::commit_preparation::prepare_application_commit::<>::{closure#2}` | 23 | 920 |
| `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::compare_and_commit_application_with_causality::<>::{closure#0}` | 23 | 713 |
| `worth_query_execution::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime<>::bind_checked_source_expectation::<>::{closure#0}::{closure#0}` | 9 | 612 |
| `worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::commit_preparation::prepare_application_commit::<>::{closure#0}` | 23 | 506 |
| `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::prepared_expectation::pending::WorthQueryPendingSourceExpectation<>::consume_into::<>::{closure#0}` | 9 | 423 |
| `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::prepared_expectation::pending::WorthQueryPendingSourceExpectation<>::consume_into::<>::{closure#1}` | 9 | 351 |
| `worth_query_execution::domain_computation::primary_graph::application_query::observed_source::prepared_expectation::pending::WorthQueryPendingSourceExpectation<>::consume_into::<>::{closure#2}` | 9 | 351 |
| `worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::commit_preparation::prepare_application_commit::<>::{closure#3}` | 23 | 345 |
| `worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::commit_preparation::prepare_application_commit::<>::{closure#4}` | 23 | 345 |
| `worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::commit_preparation::prepare_application_commit::<>::{closure#5}` | 23 | 345 |
| `worth_relational::validation::data::execution::InvariantExecutionPoint::clone` | 1 | 3 |
| `worth_relational::validation::data::execution::InvariantFailureEffect::clone` | 1 | 3 |

## Finding 7: exact stop

No external owned control can expire the private retained Instant. Cancellation/deadline stops funding before the ownership gates; SystemTime holds do not expire authentication. The new source federation courts violate BC9002 and were removed, restoring the four fixture-only edits. A clock-owner redesign is outside this lane; no public testdoor, alias/inference hiding, baseline expansion, busy wait or real-clock sleep is used.

The runtime authority order itself remains as certified, and no new production refusal is introduced to manufacture this test. A real external court cannot use the cfg(test) receiver expiry control. Funding rejects cancelled/deadline-lapsed requests before commit_for_program reaches application/branch custody. An authorization-time SystemTime hold does not advance the private retained Instant. Moving the original source court to an external package therefore loses its deterministic authentication-expiry control; keeping it in source violates court topology. No admissible existing owner route was found independently by implementation and review. Finding 7 is unapproved, and fresh-only cases are not claimed as a substitute.

## Separate source commits

Each of the eight source commits matches its independently reviewed staged tree and passed the nine pre-commit commands: formatter, Clippy, boundary, context, dirty caps, range diff, working diff, explicit mounted-file rustfmt, and cached diff. No intermediate SHA is claimed to have been separately built.

| Group | Commit | Dirty Rust files checked | Gate results |
|---|---|---:|---|
| 0 | `ecff6f410c5d3e34a5db132d0f287b27e4aa3f05` | 81 | 9 commands, all exit 0; explicit rustfmt checks all 81 existing touched Rust files. |
| 1 | `8dc17a769dfec2a5bf38a1ab0b336c554a475973` | 79 | 9 commands, all exit 0; explicit rustfmt checks all 81 existing touched Rust files. |
| 2 | `b6fd4563781cd08600b1c238e4781e2ea0e1705f` | 79 | 9 commands, all exit 0; explicit rustfmt checks all 81 existing touched Rust files. |
| 3 | `c099c8062abae87830b2b9379be259f7d4d50c38` | 69 | 9 commands, all exit 0; explicit rustfmt checks all 81 existing touched Rust files. |
| 4 | `7c7056bf1bf9ad275822e898b1aa686ecace6b76` | 68 | 9 commands, all exit 0; explicit rustfmt checks all 81 existing touched Rust files. |
| 5 | `9afbaf691043028adde120f840729c6c9e998fc6` | 63 | 9 commands, all exit 0; explicit rustfmt checks all 81 existing touched Rust files. |
| 6 | `238b9007fb53daab7ab12bda16bdc68cdb370465` | 60 | 9 commands, all exit 0; explicit rustfmt checks all 81 existing touched Rust files. |
| 7 | `3eb430d2abd926db4711f84b02be89156fba54c5` | 57 | 9 commands, all exit 0; explicit rustfmt checks all 81 existing touched Rust files. |

## Reviewer closure

The same lane reviewer independently applied qa-loop, qa-tests and code-quality-qa. It required replacing a synthetic Publication helper court with the actual Host refusal/Dependencies::Source consumer court and production-owned conversion; removing the invalid new transaction federation; and honest receiver-owned monotonic expiry. It verified retained mutation/compile attribution and restoration, exact dead-forwarder deletions and the warning baseline. It also required narrowing two occurrence-read methods to provider_execution and explicitly formatting mounted test files. It corrected the initial 19-mutant accounting from 28 to 27; after rejecting finding-7 courts, 16 retained mutants account for 22 failures. Final staged approvals and commit SHAs are reported with the delivered commits. No pre-existing court is weakened, no meaningful unread cause is removed, and no pub-causes reserved file is edited. Work stops before Slice 4 with finding 7 incomplete.
