# Milestone 4 Closeout: Historical And Branch-Aware Evaluation

## Status

Milestone 4 is closed as of 2026-04-06.

The runtime bridge now treats historical and branch-aware evaluation as a
first-class, replay-safe bridge protocol rather than as incidental snapshot
reuse.

The semantic center shipped in this milestone is:

one explicit historical-evaluation declaration lowers through resolved
truth-view policy into one planned truth-view packet, that packet materializes
through one observation-scoped truth-view authority basis, lowering produces
one canonical historical evaluation artifact, and diagnostics/replay consume
canonical historical records and typed failure records instead of rediscovering
truth-view selection from latest state, ambient branch context, or adapter
accident.

This is not "the bridge can sometimes read old snapshots."
Milestone 4 made truth-view authority itself explicit, typed, and replay-safe.

The bridge now owns:

- one declarative historical-evaluation surface
- canonical truth-view selector, declaration, policy, packet, observation, and
  lowered-artifact surfaces
- explicit branch-aware and historical authority resolution over commit,
  branch-head, and snapshot-bound truth views
- canonical historical decision-log, record, replay-summary, explanation, and
  failure-record artifacts
- typed historical failure classification rather than generic string-shaped
  drift
- harness certification for branch divergence, unavailable truth views,
  replay-after-newer-publication, diagnostics-tier parity, and historical
  counter assertions

## Shipped Scope

Milestone 4 delivered:

- `HistoricalEvaluationDeclaration` as the single bridge-owned declaration
  surface for selector, replay mode, diagnostics mode, and delivery intent
- closed-world `BridgeTruthViewSelector` support for committed snapshot,
  branch snapshot, historical commit, branch commit, and branch head views
- `ResolvedTruthViewPolicy` and typed rejection surfaces so selector support,
  replay admission, and source capability resolve before execution
- `PlannedTruthViewPacket` and `BridgeTruthViewAuthorityBasis` so historical
  and branch-local evaluation run from lowered, explicit authority rather than
  ad hoc lookups
- `MaterializedTruthViewObservation` as the phase-typed read surface over
  resolved truth views
- `LoweredHistoricalEvaluationArtifact` as the canonical lowered artifact for
  historical/branch evaluation
- canonical historical decision-log, record, replay-summary, and explanation
  artifacts
- typed historical failure recording for unsupported selectors, unavailable
  truth views, branch mismatch, snapshot mismatch, historical-resolution
  rejection, policy conflict, and replay mismatch
- first-class historical evaluation counters including selector width, branch
  width, replay mismatch count, and materialization path counts
- runtime-backed truth authority resolution for commit-bound and branch-head
  truth views
- harness coverage for historical commit execution, branch-head execution,
  branch divergence, unavailable truth-view rejection, replay parity, replay
  after newer publication, diagnostics-tier invariance, and certification
  matrix parity
- final structural cleanup into dedicated internal `historical/` and
  `diagnostics/history/` subdomains rather than leaving milestone-4 logic
  collapsed into facade-sized files

## Acceptance Mapping

Milestone 4 is considered closed against the roadmap and the engineering spec
because the required acceptance surfaces are now covered directly.

### `Historical and branch-local evaluation uses the intended truth surface`

Covered by:

- `facade::tests::runtime_materializes_snapshot_bound_truth_view_observation`
- `facade::tests::runtime_materializes_commit_bound_truth_view_observation`
- `facade::tests::runtime_materializes_branch_head_truth_view_observation`
- `harness::tests::history::bridge_harness_executes_historical_commit_view`
- `harness::tests::history::bridge_harness_executes_branch_head_view`
- `harness::tests::history::bridge_harness_branch_divergence_changes_selected_truth_view_explicitly`

What is proven:

- snapshot-bound, commit-bound, and branch-head truth views are explicit bridge
  concepts rather than latest-state convenience reads
- branch divergence changes the selected truth view canonically instead of
  flattening branch-local truth into one apparent history
- planned authority carries explicit branch/commit/snapshot basis into
  materialization

### `Branch-local truth does not leak into unrelated derived runs`

Covered by:

- `harness::tests::history::bridge_harness_branch_divergence_changes_selected_truth_view_explicitly`
- `harness::tests::certification::bridge_historical_certification_matrix_reports_candidate_profile_parity`
- `harness::tests::parity::bridge_harness_parity_proves_historical_truth_is_invariant_across_diagnostics_tiers`

What is proven:

- branch-local evaluations remain branch-scoped and digest-distinct under
  divergence
- diagnostics richness changes explanation surfaces only, not truth-view
  selection
- candidate execution profiles preserve the same historical/branch truth

### `Historical bridge evaluation remains replayable and diagnosable`

Covered by:

- `facade::tests::runtime_canonicalizes_historical_evaluation_record`
- `facade::tests::runtime_replays_canonical_historical_evaluation_record`
- `facade::tests::runtime_replay_rejects_historical_authority_drift`
- `facade::tests::runtime_replay_rejects_incompatible_historical_record_version`
- `harness::tests::history::bridge_harness_replays_historical_record`
- `harness::tests::history::bridge_harness_replays_historical_record_after_newer_publication_arrives`
- `harness::tests::diagnostics::bridge_diagnostics_retain_queryable_historical_records_by_record_and_decision_log_identity`

What is proven:

- replay proceeds from canonical declaration, policy, authority, and
  decision-log truth rather than from live latest state
- authority drift and schema incompatibility reject explicitly and typed
- canonical historical records remain queryable by both record identity and
  decision-log identity
- historical replay remains pinned to the original truth view after newer
  publication arrives

### `Unavailable or unsupported truth views fail explicitly and typed`

Covered by:

- `facade::tests::runtime_rejects_required_replay_when_runtime_policy_disallows_replay_artifacts`
- `harness::tests::history::bridge_harness_rejects_unavailable_historical_view_explicitly`
- `harness::tests::planning::bridge_historical_lineage_packet_preserves_typed_unsupported_class_failure`

What is proven:

- unsupported or unavailable historical requests do not degrade into latest
  reachable truth
- replay-policy rejection is explicit at planning time rather than deferred
  into execution ambiguity
- historical failures are retained as canonical diagnostic truth with typed
  failure classes

### `Truth-view planning, counters, and canonicality remain deterministic`

Covered by:

- `facade::tests::runtime_plans_truth_view_packet_from_admitted_policy`
- `facade::tests::runtime_lowers_identical_historical_requests_to_identical_artifacts`
- `harness::tests::counters::historical_evaluation_counters_capture_selector_branch_and_materialization_width`
- `snapshot::selection::tests::planned_truth_view_packet_is_canonical_for_same_inputs`
- `adapter::tests::historical_lineage_authority_digest_is_canonical_for_same_inputs`

What is proven:

- identical admitted requests lower to identical planned packets and lowered
  historical artifacts
- counter surfaces expose selector width, branch width, materialization path,
  and replay mismatch accounting
- authority and packet digest construction are canonical rather than
  host-order-dependent

## Additional Hardening Added Before Close

Milestone 4 closeout includes these extra hardening outcomes beyond the
minimum phase labels:

- milestone-4 historical orchestration was extracted out of
  `facade.rs` into dedicated internal `historical/` submodules for policy,
  planning, materialization, replay, and failures
- historical diagnostics artifacts were split into dedicated
  `diagnostics/history/` submodules for counters, failures, records, and
  replay summaries instead of one broad file
- lowered historical artifacts were split away from observation materialization
  so lowering and materialization no longer share one file responsibility
- historical failure diagnostics were promoted to first-class typed records and
  queryable retained history instead of remaining generic runtime errors
- commit-bound and branch-head truth views now resolve through real authority
  sources rather than placeholder admissions that fail later
- historical materialization continues to reuse the shared planned-snapshot
  opening path rather than forking snapshot-opening logic into a second
  execution path

These changes were made because the closeout bar was not "historical reads seem
to work." The closeout bar was production-grade authority honesty, proof-chain
clarity, replay safety, and subdomain structure that future milestones can
extend cleanly.

## Explicit Deviations And Honest Notes

Milestone 4 closes with a few intentional implementation-shape deviations from
the spec's example file list:

- the spec's expected `delivery/historical.rs` did not become necessary because
  historical materialization reuses the shared delivery snapshot-opening seam
  honestly
- the spec's suggested `snapshot/authority.rs` and `snapshot/decision_log.rs`
  surfaces are realized through existing `snapshot/selection.rs`,
  `snapshot/history.rs`, and `diagnostics/history/records.rs` boundaries
  without collapsing responsibilities back together
- historical harness support lives in the existing harness adapter and test
  component layout rather than dedicated `fixtures/historical_*.rs` files

These are implementation-shape deviations, not semantic gaps. The required
authority boundaries, proof-bearing artifacts, failure taxonomy, counters, and
certification lanes are present.

## Verification Baseline

At closeout, the verification baseline for the milestone implementation is:

- `cargo test -p worth-runtime-bridge`

This passes cleanly and includes:

- 128 unit and harness tests
- compile-fail phase-boundary coverage
- historical and branch-local runtime tests
- parity and certification matrix coverage
- replay, diagnostics, and typed-failure coverage

## Operational Conclusion

Milestone 4 is now closed at the bridge level.

The runtime bridge no longer treats historical or branch-local evaluation as a
best-effort consequence of stable snapshots. It now owns a real truth-view
contract: explicit declaration, pre-resolved policy, planned authority,
phase-typed observation, lowered historical artifacts, canonical records,
typed failures, replay-safe decision logs, and certification evidence strong
enough to support later bridge planning, reactive source contracts, and
branch-coordination milestones without reopening truth-view authority.
