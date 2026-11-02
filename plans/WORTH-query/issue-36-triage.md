# Issue 36: measured court triage

Status: triage and named repairs complete. All 19 issue-named courts have passing
results. Classification: 11 stale fixtures, three production defects, five fixed
already, no measured load flake. Untouched formatting and Clippy debt remains
reported below; this is not a claim that every repository check is green.

## Baseline and scope

Read [issue 36](https://github.com/recnepspencer/worth/issues/36), including its
eight-fixture comment, without modifying the issue. Fetch and merge of both
required sync branches reported already up to date before this slice:

- Lane baseline: `567b4f8abbb38a96e651fb8bdadeaf2a1d2c9e66`.
- `origin/master`: `7131b1d4cf5e04be108417f199a602f0641723b9`.
- `origin/codex/919-provider-lifecycle-20261027`:
  `567b4f8abbb38a96e651fb8bdadeaf2a1d2c9e66` at initial reproduction.
  The original table sampled the later remote ref and incorrectly associated it
  with this baseline. Remote reflog confirms that `d5d9456637cff28cfa19cfd92cee9cecd3596370`
  arrived during reproduction; it was merged in `7d83d1bdfa` before repair.
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

After the provider sync, both formerly compiler-blocked courts execute and fail:
the measured runtime total is five passing courts and fourteen failing courts.
This includes ten passing executions (five isolated parent/child pairs) and
fourteen failing executions, excluding diagnostic reruns and build refusals.

Classification is a conclusion about the cause, separate from the measured
outcome. An unresolved cause or a compiler refusal is not forced into a
fixture/production/flake category without evidence.

| Key | Court | Initial outcome | Classification and evidence |
|---|---|---|---|
| R | `certification::application_graph::document_retention_model::record_history::court::supplied_equal_value_retains_its_occurrence_and_bounded_pages_match_authored_commits` | 0 passed, 1 failed; `World(History(RetiredHistoryGap))` at `mod.rs:58` | Stale fixture. The fork creation occurrence is not held before its first mutation. Automatic changed-slot retirement in `01a0e3b962` legitimately makes that unprotected raw parent retire. Keep a genuine observation of the fork root; preserve the expected record sequence and the typed-gap negative twin. |
| R | `certification::native_operation_bodies::document_retention_model::record_history::court::supplied_equal_value_retains_its_occurrence_and_bounded_pages_match_authored_commits` | 0 passed, 1 failed; same typed gap at `mod.rs:58` | Same stale fixture custody; independently executed. |
| R | `certification::native_workflow_operations::document_retention_model::record_history::court::supplied_equal_value_retains_its_occurrence_and_bounded_pages_match_authored_commits` | 0 passed, 1 failed; same typed gap at `mod.rs:58` | Same stale fixture custody; independently executed. |
| H | `application_query::tests::canonical_basis_residue::warm_consumers_cannot_hide_hashing_behind_a_helper` | 0 passed, 1 failed; `projected_tree/content.rs` contains `Sha256` | Stale source guard after `10cdd5b0bb` and `6af8ebc1aa`. The first repair exposed a second hit in `authorization/retained_capability_request/material.rs` (0 passed, 1 failed, 360 filtered). Both admitted content/capability-use descriptor writers meter original-caller framing and bytes; retained comparison requires exact raw content or full request meaning. Restrict exceptions to these exact owners and SHA tokens; semantic identity helpers remain forbidden. No claim is made that every admitted capture is a cold rebuild. |
| E | `domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::tests::unobservable::a_few_adjacency_keys_whose_worst_case_passes_the_request_work_run_in_full` | 0 passed, 1 failed; shared `projected_attempt.rs:215` expected zero-memory exhaustion | Stale fixture after `a2a9ce5d99` replaced a separate declared kernel quota with the original request ceiling. Its copied 4096-unit ceiling now underfunds FirstRun's complete host pipeline. Corrected diagnostic prints `retained prior: false`, `InvariantAdmission(ExecutionRequest(WorkExhausted))`, one field read and zero other projection counters. Preserve full-uncompared fallback, gathered partitions and equal semantic outcome when funding the whole pipeline. |
| E | `domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::tests::unobservable::more_retained_keys_than_the_request_work_runs_in_full_uncompared` | 0 passed, 1 failed; same shared assertion | Same stale request-funding transition in `a2a9ce5d99`; separately executed and diagnostic-rerun. Corrected `retained prior: false` proves FirstRun fails before the adversarial retained-state attempt. Its retained key bound must still exceed the actual original request ceiling after repair. |
| E | `domain_computation::primary_graph::provider::application_touch_admission::tests::whole_clear_requires_grants_for_every_removed_prior_field` | 0 passed, 1 failed; `complete actual removal grants: Contract` | Stale fixture after `eea53fe428`: the purported complete operation lacks `RemoveAspect`. That commit makes whole removal depend on the explicit parent grant rather than field writes. The repaired court is named `whole_clear_requires_explicit_aspect_removal_grant`; a genuinely installed all-fields-without-removal operation must still refuse. Required aspect removal remains a Native denial. |
| E | `domain_computation::primary_graph::provider::application_touch_admission::tests::whole_sequence::committed_prior_fields_cannot_be_hidden_by_a_successive_whole_write` | 0 passed, 1 failed; `full installed grants before effects: Contract` | Same stale complete-grant declaration. Keep real committed successive writes and the incomplete-grant refusal. |
| E | `domain_computation::primary_graph::tests::inbound_admission::cost_tests::post_preparation_cancellation_releases_inbound_dispatch_reservation` | 0 passed, 1 failed; `Denied(ProviderRejected)` at `ProviderCommit`, detail `Relational execution stopped: Cancelled at None`, typed cause `Execution(Err(Cancelled))` | Production defect. Successor-observation reservation can stop after native preparation; normalization of the original caller interruption currently covers only the native-prepare call. Preserve live-caller execution-preparation retry denials while classifying actual caller cancellation correctly. |
| Q | `domain_installation::operation_execution::workflow_parallel_progression::frontier::tests::dispatch::reversed_completion_still_applies_canonical_effects_and_receipts` | Initially compile-blocked; post-sync 0 passed, 1 failed, 2487 filtered | Stale fixture after `c478117afa` reduced native preparation charges. `dispatch.rs:225` expects the default 1024-unit request to stop, but it completes. Preserve reversed completion, canonical effects and serial/parallel receipt equivalence; test resource refusal separately. |
| C | `certification::application_graph::canonical_transition::related::companion_transition::companion_target_rule_rejects_actual_surviving_relation_before_publication` | Initially compile-blocked; post-sync 0 passed, 1 failed, 328 filtered | Stale diagnostic after `3dc89054fd` moved the genuine invariant check to seed publication. Actual subject is `application seed transaction`; expected subject is `application principal bootstrap transaction`. The custom invariant still refuses before publication. |
| F | `application_invariant_acceptance::candidate_index::boxes::candidate_allocation::pruned_completed_source_allocates_no_dense_candidate_backing` | 1 passed, 0 failed, in parent and isolated child | Fixed already on this branch. Physical interval: zero allocations, zero allocated bytes; one deallocation of eight bytes. Existing zero-allocation assertions unchanged. |
| F | `application_invariant_acceptance::managed_index::boxes::candidate_allocation::pruned_completed_source_allocates_no_dense_candidate_backing` | 1 passed, 0 failed, in parent and isolated child | Fixed already; same measured zero-allocation interval. |
| F | `application_invariant_acceptance::candidate_index::boxes::lifecycle::cancelled_cold_box_build_releases_partial_bvh_and_rebuilds_with_fresh_request` | 1 passed, 0 failed, in parent and isolated child | Fixed already. Existing exact allocated/deallocated-byte equality, original cancellation, Failed lifecycle and fresh successful rebuild all pass. |
| F | `application_invariant_acceptance::managed_index::boxes::lifecycle::cancelled_cold_box_build_releases_partial_bvh_and_rebuilds_with_fresh_request` | 1 passed, 0 failed, in parent and isolated child | Fixed already; same unchanged release/rebuild court. |
| F | `application_invariant_acceptance::managed_index::lifecycle::cancelled_cold_prefix_build_releases_partial_backing_and_rebuilds_with_fresh_request` | 1 passed, 0 failed, in parent and isolated child | Fixed already. Existing cancellation, partial backing release and fresh rebuild assertions pass. |
| F | `proposal_acceptance::courts::acceptance::disposal::partial_attempt_disposal_retries_actual_retired_cleanup_until_its_reader_releases` | 0 passed, 1 failed; `NestedStopped` at `disposal.rs:117` after reader release | Production defect: application cleanup lacked the product owner's admitted-phase retry. Its standalone retry opened a different serial request inside advancement. The new receiver-owned original-phase door and caller correction make the unchanged reader-release/completion assertions pass; no retention bypass was needed. |
| F | `proposal_acceptance::courts::acceptance::previous::previous_head_cleanup_recovers_without_discarding_adoption_evidence_or_adopting_again` | 0 passed, 1 failed; expected completion at `previous.rs:131` did not occur | Same production request gap, now causally verified. Original completion, retired-component count, kept adoption evidence and no-second-adoption assertions pass with the original admitted phase. No residual retention refusal remains. |
| T | `checkpoint_recovery::current_output::serial::restored_current_output_is_tracked_before_any_producer_demand_and_rejects_source_change` | 0 passed, 1 failed; `InvariantAdmission(ExecutionRequest(WorkExhausted))` at `current_output.rs:63` | Stale request funding introduced by `facc9d1197`: the legitimate projection-quota removal expanded this very court from one read to 8193 without changing its eight-million-unit host grant. No evidence establishes that expanded fixture ever passed at eight million; the issue's earlier green report covers the original eleven courts. Preserve 8193 reads and zero producer contacts, and retain the original grant as an explicit typed-refusal twin. This is not evidence of a later production read-cost regression. |

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

On the initial baseline, both Q and C stopped on these unchanged production sources:

```text
error[E0063]: missing field `source_kind` in initializer of `WorthQueryReadGraphRelationView<'_>`
crates/worth-query/src/runtime/surface/read_graph_planning_contract.rs:180:10

error[E0004]: non-exhaustive patterns: WorthQueryGraphReadAccessRequirementKind::DerivedSource not covered
crates/worth-query/src/runtime/graph_read_access/access_admission/case_inventory.rs:166:11
```

Cargo reports `could not compile worth-query` due to two previous errors.
These are outside the named court failures and were not repaired by this lane.
Provider sync `d5d9456637` fixes them; the required merge `7d83d1bdfa` lets both
courts run. Q then reports the complete workflow where its old assertion expects
work exhaustion (zero passed, one failed, 2487 filtered).

C's first post-sync command still executes zero tests: three missing
`WORTH_IMPLEMENTATION_BUILD_ID` compile-time inputs in two testkit packages.
The documented `python scripts/application_build.py cargo test ...` wrapper
issues that required build identity. With the same single-court filter, C runs
and reports the diagnostic subject mismatch (zero passed, one failed, 328
filtered). Both post-sync runtime runs have zero ignored and zero measured.

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

## Readings I chose

- Use the nested owning manifest for excluded external consumer packages;
  selecting them from the Query workspace would not execute their courts.
- A compile refusal is a blocked measurement, not one of the issue's reported
  runtime failures. An unresolved cause remains explicit rather than guessed.
- Preserve exact World ancestry refusal. Retaining the missing real fork
  occurrence repairs fixture custody; splicing history would weaken correctness.
- Keep the intentional 8193 current-output population and every allocation
  equality assertion until evidence justifies a production or fixture repair.
- Supply the documented fresh application build identity for courts that embed
  it. A missing build input is a command refusal and executes no court.
- The application cleanup receiver needs the original phase, just as the
  existing product cleanup receiver does. Source/facade documentation, branch
  guidance, worked cleanup and known gaps show only a standalone application
  retry. Add the missing receiver method at that owner and index it; do not
  expose a runtime getter or create another cleanup service.

## Repair and verification status

The initial and repair-attempt failures remain recorded rather than overwritten.
Named repairs are complete; focused court qualification is reported per slice.
No new `AspectShape`, shape form, authority type or production cost API was added.
The application cleanup receiver gains one method at its existing owner.

### Publication custody repair

The three record-history fixtures now hold a genuine observation of the fork's
creation occurrence until exact paging completes. Production history behavior
and the expected four authored record coordinates are unchanged. Caller
interruption classification now surrounds the complete pre-publication commit
transition rather than only Native preparation. Only a returned control stop
with an actually interrupted original caller is normalized; successful or
unpublished publication and live-caller retry custody are unchanged. No shape
or public API was added.

Focused commands all use the Query manifest above and `-- --nocapture`:

| Package / target / filter | Passed | Failed | Filtered |
|---|---:|---:|---:|
| R / `--test worth_query_court_decl_host` / `record_history::court` | 9 | 0 | 185 |
| E / `--lib` / `post_preparation_cancellation_releases_inbound_dispatch_reservation` | 1 | 0 | 2189 |
| E / `--lib` / `preparation_control_refusal_and_ordinary_interruption_keep_different_head_postures` | 1 | 0 | 2189 |
| E / `--lib` / `stops::leased_requests::request_control` | 2 | 0 | 2188 |

All have zero ignored and measured tests. History's nine include three genuine
retired-parent refusal twins and three grant checks. The posture court checks
both cancellation and timeout: an interrupted work lease with a live original
caller remains a retryable execution-preparation denial. The two public-door
courts keep original typed cancellation/deadline causes. The cancellation
regression's independent native commit and inbound reservation observations
remain unchanged; the actual post-preparation pause/cancel now returns Cancelled.

Publication-custody footer commands select `-p worth-query-execution
-p worth-query-court-decl-host` with the Query manifest for both formatting and
Clippy. Formatting exits zero. Clippy exits 101 with the same untouched debt:
46 errors / 49 warnings for lib and 37 errors / 48 warnings for lib test
(40 warning duplicates). No diagnostic location names either changed commit
file or a changed history court. The reports overlap and are not summed as
unique errors. No lint is suppressed.
The exact boundary-check and agent-context commands listed below both exit
zero, with valid boundaries and no context mismatch. The dirty line-cap guard
passes five Rust files. Both branch and working diff checks exit zero with zero
whitespace errors. The independent reviewer found no material code finding in
this slice; it verified raw-parent custody and the enclosing pre-effect stop
boundary, including unchanged live-caller retry and unpublished-owner posture.

### Original-request cleanup repair

Application cleanup now exposes `retry_in_advancement(&phase)`, using the
existing product owner to admit exact runtime affinity and original request
liveness before history or conditional cleanup. Both scoped consumer retry
callers pass that phase. Refusals return the same cleanup; deferred retirement
receipts retain their existing drop order. Successful cleanup is not turned
into a new retry owner by a trailing poll. The standalone retry remains.
No shape, proof type or new authority seam was added. The capability index and
usage guide document the new receiver method.

Each of these F binary commands reports one passed, zero failed, 217 filtered,
zero ignored and zero measured, with the same nested manifest as reproduction:

- `partial_attempt_disposal_retries_actual_retired_cleanup_until_its_reader_releases`
- `previous_head_cleanup_recovers_without_discarding_adoption_evidence_or_adopting_again`
- `registered_disposal_uses_owned_custody_after_storage_has_really_retired`

The first two preserve every original business and custody assertion. The third
uses a genuine retired cleanup from actual storage retirement. A different
installed runtime's phase returns typed `ForeignAdvancementPhase`; cancellation
inside the own callback returns typed `Interrupted(Cancelled)` from the receiver
and outer handoff. Both return the original owner with unchanged pending work.
After releasing the independent real reader, a fresh own phase retires exactly
one component. The independent reviewer found no material source or test issue;
it checked the early liveness boundary, failure custody and drop order.

Cleanup footer checks run the exact initial commands below with selected E and
F packages, separately with their owning manifests. Both formatting commands
exit zero. E Clippy exits 101 with the unchanged 46-error/49-warning lib and
37-error/48-warning lib-test reports (40 warning duplicates); no diagnostic
location names either changed cleanup owner. F Clippy exits zero; its binary
test target reports 121 warnings, 17 duplicates. Boundary and context checks
both exit zero, with valid boundaries and no mismatch. The dirty line-cap guard
passes six Rust files. Both diff checks exit zero with zero whitespace errors.

### Stale fixture qualification

Compact triples in this section mean passed / failed / filtered.

The source guard permits only SHA tokens at the exact complete-content writer
(`10cdd5b0bb`) and capability-use material owner (`6af8ebc1aa`), with explicit
framing/checkpoint constraints. Semantic digest helpers remain forbidden.
Capability-use capture meters original caller work; retained comparison carries
the descriptor and checks the complete request. The initial content-only repair
still failed the same court on the second owner: 0 passed, 1 failed, 360 filtered.
The completed guard passes 1/0/360. Eight existing raw-content courts pass
8/0/2182, including literal protocol bytes, 14 shallow and 17 descendant
corruptions, support/order/generation distinctions and bounded refusals. Three
capability material courts pass 3/0/2187, including 22 independently selected
request mutations, framing/context backing and original first/final stops.

`eea53fe428` requires the explicit optional-parent removal grant and complete
parent field reads. The complete fixture now declares those reads and removal.
A genuinely installed `FieldsOnly` operation keeps all A/B writes without parent
removal and must refuse. Required parent clear remains a Native refusal. The
old whole-clear test name is mapped to its corrected removal-grant name above.
The first repair omitted the complete read closure: the focused family reported
1 passed, 10 failed, 2179 filtered with `MissingOperationProgramDependency` for
`Both` / `RemoveAspect(Node, Notes)`. This was a repair-attempt declaration error,
not a newly attributed production defect; the reads were added at the real
schema owner rather than bypassing validation.

The two retained-observation fixtures preserve the actual request ceiling
transition in `a2a9ce5d99`. A first repair at 8192 still failed FirstRun: each
reported 0 passed, 1 failed, 2189 filtered, `WorkExhausted`, `retained prior: false`
and three field reads. A deliberately failing tagged calibration ran with a
finite million-unit grant and stopped before the adversary. It reported two
accounted setup-through-FirstRun bills: 23,558 work / 23,142 span (peak memory
23,372 bytes) and 9,540 work / 9,539 span (peak 26,904 bytes), neither refused.
That diagnostic reports 0 passed, 1 failed, 2189 filtered; it is not a passing
court or regression. The probe is removed before final qualification.
The fixed fixture grant is 65,536, with ample headroom. Hostile key count and
adjacency worst-case work are derived from that same original grant. A batch
mutation at the existing test-only fact owner rebuilds metadata once, avoiding
quadratic per-key fixture reconstruction; actual key count is asserted above
the grant. Both courts retain zero actual retained/native comparison assertions.
The clean large-key court passes 1/0/2189 in 2.82 seconds after an actual
`worth-query-execution` rebuild. The reviewer found an unused single-copy
forwarding method after this batch change; it was removed before that rebuild.
The clean adjacency court passes 1/0/2189 in 1.68 seconds. The complete installed
touch-admission family passes 11/0/2179, including both original issue cases,
the all-fields-without-removal refusal, request/memory stops, omitted prior fields
and genuine Native required-parent clear refusal.

`c478117afa` reduced Native preparation charges. The reversed-completion court
now completes at the unchanged default 1024-unit grant at widths 0/1/2/4,
preserving the literal canonical receipt order, reset serial repeat and genuine
out-of-order overlap. It passes 1/0/2487. The separate width work-refusal court
also passes 1/0/2487; no production accounting was changed.

`3dc89054fd` moved the companion invariant check to seed publication. Updating
only that diagnostic subject passes 1/0/328; typed refusal, unchanged original
data/checkpoint bytes and zero target decodes remain asserted.

`facc9d1197` expanded the restored-output fixture to 8193 reads. The complete
journey now has a finite 16-million-unit host request. A separately named court
keeps the original eight-million-unit grant, proves typed `WorkExhausted` after
actual partial field/role reads, checks the exact World basis is unchanged and
requires zero producer contacts. Both pass together: 2 passed, 0 failed,
309 filtered. The positive court still requires all 8193 reads, zero producer
warmup, typed stale-source refusal after a real mutation, actual producer refresh
and fresh output verification. No shape or production cost API was added.

Every fixture test result above has zero ignored and zero measured tests.
The existing rename-control court also passes 1/0/2189, exercising the original
single-key rename through the same batch reconstruction owner.

Additional focused selection commands use the same owning manifests and
`-- --nocapture` as initial reproduction:

| Package and target | Filter | Passed | Failed | Filtered |
|---|---|---:|---:|---:|
| E / `--lib` | `application_touch_admission::tests` | 11 | 0 | 2179 |
| E / `--lib` | `content_comparison::tests::shape` | 8 | 0 | 2182 |
| E / `--lib` | `authorization::retained_capability_request::material::tests` | 3 | 0 | 2187 |
| Q / `--lib` | `work_ceiling_settles_the_same_boundary_at_all_widths` | 1 | 0 | 2487 |
| T / `--lib` | `checkpoint_recovery::current_output::serial` | 2 | 0 | 309 |
| E / `--lib` | `an_unobservable_key_gathers_its_partition_again_and_denies_nothing` | 1 | 0 | 2189 |

The seven original F selections were rerun individually after the production
cleanup repair. Each reports one pass, zero failures, 217 filtered and zero
ignored/measured. Each of the five allocator/lifecycle courts also reports its
one passing isolated child, with the same counts. This final F batch therefore
contains twelve passing test executions, zero failures; original predicates and
populations are unchanged. No load-flake rerun or allocation relaxation was needed.

| Named family (full names in the initial table) | Courts | Latest result and independent evidence |
|---|---:|---|
| Exact record history through three public forms | 3 | Pass; authored commit receipts determine expected coordinates, independently of paging. Three genuine retired-gap twins also pass. |
| Warm-path hashing guard | 1 | Pass; exact-owner source guard, independent literal byte framing and manually corrupted raw/request dimensions. A digest never authorizes equality. |
| Unobservable retained keys and adjacency bounds | 2 | Pass; literal partition roster, full-run semantic/work oracle and zero actual comparison counters. Hostile bounds exceed the actual finite request. |
| Whole clear and committed prior fields | 2 | Pass; real installed grants and genuine Native proposals/commits, independently read committed truth and unchanged-basis checks. The whole-clear court has its corrected name. |
| Post-preparation cancellation | 1 | Pass; external cancellation at a real pause, independent Native commit and inbound-reservation observations. |
| Reversed completion | 1 | Pass; prescribed receipt order, independent schedule rendezvous and serial/parallel full-result equivalence. Separate budget refusal remains tested. |
| Companion transition | 1 | Pass; actual target invariant refusal, unchanged literal original data and checkpoint bytes, zero target decodes. |
| Allocator and cancelled cold-build lifecycle | 5 | Pass in parent and child; independent physical allocator, exact release accounting and fixture full scans for fresh rebuilt results. |
| Disposal and previous-head cleanup | 2 | Pass; separately held/released real readers, actual owner receipts, kept adoption evidence and unchanged selected head. |
| Restored current output | 1 | Pass; genuine checkpoint, zero provider contacts, real source mutation with typed stale refusal and literal refreshed value. The original finite grant has a separate no-effects refusal court. |

### Fixture footer checks and untouched debt

Formatting ran with the Query manifest and selected H/E/Q/C packages together:
exit 1, three formatter diff blocks in the untouched
`worth-query-court-query-decl-host/tests/worth_query_court_query_decl_host/host/application_runtime_surface/mod.rs`.
They contain four formatting changes (import placement/order, function wrapping,
module order). The file is unchanged from the lane baseline. Formatting the
selected H/E/Q packages alone exits zero; T with its nested owning manifest
also exits zero. No changed file has a formatter diff.

All Clippy commands use `cargo clippy --manifest-path <owning manifest>
-p <package> --all-targets`; Query packages use the Query manifest and T uses
the consumer-entry manifest. Required build identities use the documented
`python scripts/application_build.py cargo ...` wrapper.

| Package selection | Exit | Exact target report |
|---|---:|---|
| H / `worth-query-installation` | 101 | Two errors and two warnings in lib test. Both errors are untouched `application_query/inspection/tests.rs:51:26` and `:103:30`: `unnecessary use of clone to create a slice from a reference`. The file is unchanged from the lane baseline. |
| E / `worth-query-execution` | 101 | Same untouched 46-error/49-warning lib and 37-error/48-warning lib-test reports, 40 warning duplicates. No changed fixture or mutation owner is a diagnostic location. |
| Q / `worth-query` | 101 | Blocked by the same dependency E lib's 46 errors / 49 warnings; no claim that Q's own lint targets completed. |
| C / `worth-query-court-query-decl-host` | 101 | Same E dependency barrier, 46 errors / 49 warnings; no claim that C's own lint targets completed. |
| T / `worth-query-topology-entry`, without wrapper | 101 | Two missing `WORTH_IMPLEMENTATION_BUILD_ID` compile-time errors from the shared-execution source, one per test/example target. This is a command input failure. |
| T, corrected wrapper invocation | 0 | Zero errors. Lib reports one warning, lib test 35; required-feature and invariant-read example/test targets and the shared-execution example each report two warnings. Two reports mark two duplicates each; reports are not summed as unique warnings. |

Boundary-check exits zero with valid boundaries. Agent-context check exits zero
with no mismatch. The dirty line-cap guard passes nine Rust files, all at or
below 400 lines. Both branch and working diff checks exit zero with zero
whitespace errors. No lint or boundary rule was suppressed. Untouched failures
are reported under AGENTS.md's scoped-debt rule and were not absorbed into this
lane.

The persistent reviewer independently read every slice's code and tests using
qa-loop, qa-tests and code-quality-qa. Its corrections closed before commit:
FirstRun diagnostic ambiguity; whole-removal naming and isolated field-only
negative control; complete parent read closure; exact capability material
comparison trace; finite whole-host funding and batch setup; removal of the
obsolete single-copy method. The final temporary probe is absent, and the
clean affected artifact was rebuilt and executed successfully.

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
