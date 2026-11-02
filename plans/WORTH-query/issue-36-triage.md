# Issue 36: measured court triage

Status: initial reproduction, not qualification for adoption. The issue's
reported failure counts are not this branch's measured result.

## Baseline and scope

Read [issue 36](https://github.com/recnepspencer/worth/issues/36), including its
eight-fixture comment, without modifying the issue. Fetch and merge of both
required sync branches reported already up to date before this slice:

- Lane baseline: `567b4f8abbb38a96e651fb8bdadeaf2a1d2c9e66`.
- `origin/master`: `7131b1d4cf5e04be108417f199a602f0641723b9`.
- `origin/codex/919-provider-lifecycle-20261027`:
  `d5d9456637cff28cfa19cfd92cee9cecd3596370`.
- Environment: Windows x86-64, repository-pinned Rust and repository build
  configuration; existing lane target directory and six build jobs.

Every named court was selected individually. Seventeen courts executed:
**five passed and twelve failed**. Two further courts could not execute because
their Query dependency does not compile. No load flake has been established.
The five passing allocation courts also run their one selected test in an
isolated child process: each parent and child reports one pass, zero failures.
Thus the initial successful commands contain ten passing test executions;
the twelve failing courts contain twelve failing executions.

No court predicate or budget was changed for the initial reproduction. A later
diagnostic change preserves the shared projection assertion and prints the
actual typed denial and whether a real retained state was supplied.

## Commands

Each table row uses `cargo test`, `-p`, the indicated target and its unique
court filter, followed by `-- --nocapture`. Query packages use
`--manifest-path workspaces/worth-query/Cargo.toml`. The external consumer
packages use their actual owning manifest:
`workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/Cargo.toml`.
They are excluded from the Query workspace.

| Key | Package | Target |
|---|---|---|
| R | `worth-query-court-decl-host` | `--test worth_query_court_decl_host` |
| H | `worth-query-installation` | `--lib` |
| E | `worth-query-execution` | `--lib` |
| Q | `worth-query` | `--lib` |
| C | `worth-query-court-query-decl-host` | `--test worth_query_court_query_decl_host` |
| F | `worth-query-consumer-root` | `--bin worth-query-consumer-root` |
| T | `worth-query-topology-entry` | `--lib` |

The three R filters were their complete names below. Other filters were the
unique final function name, except the two aliased F allocation and box-build
families, which used the complete names below. Each executed selection reports
one selected test. Filtered counts: R 193, H 360, E 2189, F 217, T 309.
All ignored and measured counts were zero.

Seven initial F commands incorrectly selected `--lib`; Cargo reported
`no library targets found in package worth-query-consumer-root`. These executed
zero tests and are command errors, not court failures. All seven were rerun
with the correct binary target; only those corrected outcomes appear below.

## Court table

Classification is a conclusion about the cause, separate from the measured
outcome. An unresolved cause or a compiler refusal is not forced into a
fixture/production/flake category without evidence.

| Key | Court | Initial outcome | Classification and evidence |
|---|---|---|---|
| R | `certification::application_graph::document_retention_model::record_history::court::supplied_equal_value_retains_its_occurrence_and_bounded_pages_match_authored_commits` | 0 passed, 1 failed; `World(History(RetiredHistoryGap))` at `mod.rs:58` | Stale fixture. The fork creation occurrence is not held before its first mutation. Automatic changed-slot retirement in `01a0e3b962` legitimately makes that unprotected raw parent retire. Keep a genuine observation of the fork root; preserve the expected record sequence and the typed-gap negative twin. |
| R | `certification::native_operation_bodies::document_retention_model::record_history::court::supplied_equal_value_retains_its_occurrence_and_bounded_pages_match_authored_commits` | 0 passed, 1 failed; same typed gap at `mod.rs:58` | Same stale fixture custody; independently executed. |
| R | `certification::native_workflow_operations::document_retention_model::record_history::court::supplied_equal_value_retains_its_occurrence_and_bounded_pages_match_authored_commits` | 0 passed, 1 failed; same typed gap at `mod.rs:58` | Same stale fixture custody; independently executed. |
| H | `application_query::tests::canonical_basis_residue::warm_consumers_cannot_hide_hashing_behind_a_helper` | 0 passed, 1 failed; `projected_tree/content.rs` contains `Sha256` | Stale source guard after `10cdd5b0bb`. The metered complete-content hash is a rejection filter followed by exact raw-content comparison, not equality authority. A repair must keep semantic identity helpers forbidden and constrain the content-owner exception. |
| E | `domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::tests::unobservable::a_few_adjacency_keys_whose_worst_case_passes_the_request_work_run_in_full` | 0 passed, 1 failed; shared `projected_attempt.rs:215` expected zero-memory exhaustion | Stale fixture after `a2a9ce5d99` replaced a separate declared kernel quota with the original request ceiling. Its copied 4096-unit ceiling now underfunds FirstRun's complete host pipeline. Corrected diagnostic prints `retained prior: false`, `InvariantAdmission(ExecutionRequest(WorkExhausted))`, one field read and zero other projection counters. Preserve full-uncompared fallback, gathered partitions and equal semantic outcome when funding the whole pipeline. |
| E | `domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::tests::unobservable::more_retained_keys_than_the_request_work_runs_in_full_uncompared` | 0 passed, 1 failed; same shared assertion | Same stale request-funding transition in `a2a9ce5d99`; separately executed and diagnostic-rerun. Corrected `retained prior: false` proves FirstRun fails before the adversarial retained-state attempt. Its retained key bound must still exceed the actual original request ceiling after repair. |
| E | `domain_computation::primary_graph::provider::application_touch_admission::tests::whole_clear_requires_grants_for_every_removed_prior_field` | 0 passed, 1 failed; `complete actual removal grants: Contract` | Stale fixture after `eea53fe428`: the purported complete operation lacks the separate `RemoveAspect` grant. Field-write grants must remain insufficient. |
| E | `domain_computation::primary_graph::provider::application_touch_admission::tests::whole_sequence::committed_prior_fields_cannot_be_hidden_by_a_successive_whole_write` | 0 passed, 1 failed; `full installed grants before effects: Contract` | Same stale complete-grant declaration. Keep real committed successive writes and the incomplete-grant refusal. |
| E | `domain_computation::primary_graph::tests::inbound_admission::cost_tests::post_preparation_cancellation_releases_inbound_dispatch_reservation` | 0 passed, 1 failed; `Denied(ProviderRejected)` at `ProviderCommit`, detail `Relational execution stopped: Cancelled at None`, typed cause `Execution(Err(Cancelled))` | Production defect. Successor-observation reservation can stop after native preparation; normalization of the original caller interruption currently covers only the native-prepare call. Preserve live-caller execution-preparation retry denials while classifying actual caller cancellation correctly. |
| Q | `domain_installation::operation_execution::workflow_execution::progression::frontier::tests::dispatch::reversed_completion_still_applies_canonical_effects_and_receipts` | 0 tests executed; Cargo exit 101 | Compile-blocked, unclassified. The issue's former work-ceiling assertion cannot be measured on this baseline. See the exact compiler failures below. |
| C | `certification::application_graph::canonical_transition::related::companion_transition::companion_target_rule_rejects_actual_surviving_relation_before_publication` | 0 tests executed; Cargo exit 101 | Compile-blocked, unclassified. The issue's reported subject mismatch is not a measured result here. |
| F | `application_invariant_acceptance::candidate_index::boxes::candidate_allocation::pruned_completed_source_allocates_no_dense_candidate_backing` | 1 passed, 0 failed, in parent and isolated child | Fixed already on this branch. Physical interval: zero allocations, zero allocated bytes; one deallocation of eight bytes. Existing zero-allocation assertions unchanged. |
| F | `application_invariant_acceptance::managed_index::boxes::candidate_allocation::pruned_completed_source_allocates_no_dense_candidate_backing` | 1 passed, 0 failed, in parent and isolated child | Fixed already; same measured zero-allocation interval. |
| F | `application_invariant_acceptance::candidate_index::boxes::lifecycle::cancelled_cold_box_build_releases_partial_bvh_and_rebuilds_with_fresh_request` | 1 passed, 0 failed, in parent and isolated child | Fixed already. Existing exact allocated/deallocated-byte equality, original cancellation, Failed lifecycle and fresh successful rebuild all pass. |
| F | `application_invariant_acceptance::managed_index::boxes::lifecycle::cancelled_cold_box_build_releases_partial_bvh_and_rebuilds_with_fresh_request` | 1 passed, 0 failed, in parent and isolated child | Fixed already; same unchanged release/rebuild court. |
| F | `application_invariant_acceptance::managed_index::lifecycle::cancelled_cold_prefix_build_releases_partial_backing_and_rebuilds_with_fresh_request` | 1 passed, 0 failed, in parent and isolated child | Fixed already. Existing cancellation, partial backing release and fresh rebuild assertions pass. |
| F | `proposal_acceptance::courts::acceptance::disposal::partial_attempt_disposal_retries_actual_retired_cleanup_until_its_reader_releases` | 0 passed, 1 failed; `NestedStopped` at `disposal.rs:117` after reader release | Cause attribution pending. A production application-cleanup request gap is supported by source: its retry opens a new serial request inside an existing advancement, while the product owner already supports the admitted phase. This run does not yet isolate that gap from a residual cleanup refusal. |
| F | `proposal_acceptance::courts::acceptance::previous::previous_head_cleanup_recovers_without_discarding_adoption_evidence_or_adopting_again` | 0 passed, 1 failed; expected completion at `previous.rs:131` did not occur | Unresolved. The same request gap is present, but its causal contribution and any additional retention cause require the actual residual denial. Keep the genuine reader-release and retained adoption-evidence requirements. |
| T | `checkpoint_recovery::current_output::serial::restored_current_output_is_tracked_before_any_producer_demand_and_rejects_source_change` | 0 passed, 1 failed; `InvariantAdmission(ExecutionRequest(WorkExhausted))` at `current_output.rs:63` | Unresolved cost failure. Equality lookups/candidates 5091, field reads/role lookups 5090. `facc9d1197` deliberately used 8193 expectations to cross a former 8192 ceiling while retaining a finite eight-million-unit request. Do not lower that population or raise its budget without establishing the legitimate production change. |

## Diagnostic reruns

The two unobservable courts were each rerun twice for diagnosis. Every rerun
reports zero passed, one failed, 2189 filtered, zero ignored and zero measured.
The corrected diagnostic distinguishes an actual retained state from a supplied
FirstRun token: both print `retained prior: false`. Their FirstRun setup exhausts
work before the adversarial retained-state attempt begins. This does not establish
a production observation-fallback defect. The identified `a2a9ce5d99` transition
removed the separate kernel quota; fund the whole host request without restoring
that retired quota or reducing the adversarial bound relative to the request.

## Out-of-scope compiler blockers

Both Q and C stop on the following unchanged production sources:

```text
error[E0063]: missing field `source_kind` in initializer of `WorthQueryReadGraphRelationView<'_>`
crates/worth-query/src/runtime/surface/read_graph_planning_contract.rs:180:10

error[E0004]: non-exhaustive patterns: WorthQueryGraphReadAccessRequirementKind::DerivedSource not covered
crates/worth-query/src/runtime/graph_read_access/access_admission/case_inventory.rs:166:11
```

Cargo reports `could not compile worth-query` due to two previous errors.
These are outside the named court failures. Do not repair these files in this
lane or describe either blocked court as passing or reproduced at runtime.

## Evidence boundaries

- History uses actual authored commit receipts as its expected coordinates,
  independent of the history pager. Its retired-parent twin must still refuse.
- Content comparison has existing independent literal framing evidence and
  corrupted raw-shape/descendant twins. A content digest alone must never
  authorize equality. No new shape is required.
- Grant courts cross real Native validation and publication; literal expected
  presence/value and unchanged committed snapshots distinguish denied effects.
- Cancellation uses a real preparation pause and external cancellation source;
  actual commit and inbound-reservation observations prove absence of effects.
- Allocation uses isolated physical allocator observations. Cancellation tests
  compare rebuilt query results against independent fixture full scans.
- Cleanup uses a genuine separately held reader, then its actual release,
  independently of the cleanup owner. Retained adoption evidence must survive.
- Restored-output testing starts from a genuine checkpoint, requires zero
  producer contacts, and then changes the source to require a typed refusal.

## Readings chosen

- Use the nested owning manifest for excluded external consumer packages;
  selecting them from the Query workspace would not execute their courts.
- A compile refusal is a blocked measurement, not one of the issue's reported
  runtime failures. An unresolved cause remains explicit rather than guessed.
- Preserve exact World ancestry refusal. Retaining the missing real fork
  occurrence repairs fixture custody; splicing history would weaken correctness.
- Keep the intentional 8193 current-output population and every allocation
  equality assertion until evidence justifies a production or fixture repair.

## Repair and verification status

Initial reproduction is complete. Diagnostic classification and repairs remain
open. No implementation slice is qualified by this initial table.

### Initial triage slice checks

| Command | Result |
|---|---|
| `cargo fmt --check --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution` | Exit 0; one selected package. |
| `cargo clippy --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --all-targets` | Exit 101. Compiler reports 46 errors / 49 warnings for lib, and 37 errors / 48 warnings for lib test (40 warning duplicates). These target reports overlap; do not sum them as unique diagnostics. No diagnostic location names the changed `projected_attempt.rs`. |
| `cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .` | Exit 0; `repository boundaries are valid`. |
| `cargo run --manifest-path tools/agent-context/Cargo.toml -- check` | Exit 0; no mismatch reported. |
| `bash scripts/ci/check_workspace_rust_line_caps.sh dirty` | Exit 0; checked one dirty Rust file, 400-line cap. |
| `git diff --check origin/master...HEAD` | Exit 0; zero whitespace errors. |
| `git diff --check` | Exit 0; zero whitespace errors. |

The Clippy failures are in untouched files. Representative exact failures:

```text
error: unneeded unit expression
application_query/managed_index/realized_footprint/candidate_rows.rs:108:33

error: use of a disallowed method `std::any::TypeId::of`
application_contribution/producer/output_equivalence_contract.rs:167:23
```

They are reported repository debt, not edits authorized by this lane's court
list. No lint was suppressed and no unrelated file was changed.
