# Milestone 1 Closeout: CDC and Subscriber Recovery

## Status

Milestone 1 is closed as of 2026-03-13.

The runtime now treats CDC as a subscriber-facing product contract rather than
an internal patch-emission side effect.

## Shipped Scope

Milestone 1 delivered:

- subscriber-facing checkpoint and resume semantics
- typed subscriber-visible failure classes
- deterministic resumed publication order
- replay-to-CDC and patch-to-CDC parity hardening
- durable subscriber recovery from canonical artifacts
- snapshot-stable CDC behavior under rewrite pressure
- restart-safe subscriber resume behavior across repeated reconnect loops
- hostile window-boundary, retention-pressure, rewrite-storm, and branch-pressure certification

The implementation was also split structurally so patch truth, bundle/publication
assembly, and subscriber CDC remain semantically distinct.

## Acceptance Mapping

Milestone 1 is considered closed against the roadmap because the required
acceptance surfaces are now covered directly.

### `Diff/CDC truth parity test`

Covered by:

- `tests::publication::cdc::replay_parity::subscriber_stream_matches_patch_stream_for_committed_history`
- `tests::publication::cdc::certification::cdc_property_random_operation_matrix_converges`
- `tests::publication::cdc::certification::cdc_certification_savepoint_abandoned_work_never_leaks_into_stream_truth`
- `tests::publication::cdc::savepoint_residue::savepoint_abandoned_work_never_appears_in_subscriber_cdc`

What is proven:

- subscriber CDC matches canonical patch truth
- abandoned savepoint / rollback work never appears in CDC
- stitched resume windows do not duplicate, drop, or reorder committed truth

### `Hostile commit/replay equivalence test`

Covered by:

- `tests::publication::cdc::replay_parity::durable_source_subscriber_stream_matches_recovered_runtime_patch_stream`
- `tests::publication::cdc::certification::cdc_certification_persisted_seeded_matrix_survives_checkpoint_compaction_and_recovery`
- `tests::publication::cdc::certification::cdc_property_persisted_random_operation_matrix_recovers`
- `tests::publication::cdc::certification::cdc_certification_concurrent_branch_merge_pressure_keeps_subscriber_order_stable`

What is proven:

- recovered runtimes reproduce canonical committed history
- subscriber-observed CDC remains aligned with replay/recovery outputs
- hostile branch and history shapes do not introduce scheduler-shaped CDC drift

### `Durable recovery and schema mismatch test`

Covered by:

- `tests::publication::cdc::recovery::subscriber_stream_recovers_from_durable_canonical_envelopes_when_checkpoint_is_not_in_memory`
- `tests::publication::cdc::subscriber_failures::subscriber_stream_rejects_schema_incompatible_checkpoint`
- `tests::publication::cdc::subscriber_failures::subscriber_stream_rejects_checkpoint_without_history_or_durable_coverage`
- `tests::publication::cdc::certification::cdc_certification_retention_truncation_recovers_exact_suffix_from_old_checkpoint`
- `tests::publication::cdc::certification::cdc_property_persisted_random_operation_matrix_recovers`

What is proven:

- durable recovery rebuilds subscriber-visible truth from canonical artifacts
- schema mismatch fails explicitly
- coverage gaps fail explicitly
- retention pressure does not silently corrupt resumed suffixes

### `Snapshot-stable concurrent read vs hot rewrite test`

Covered by:

- `tests::publication::cdc::snapshot_stability::subscriber_cdc_is_snapshot_stable_under_hot_rewrite_pressure`
- `tests::publication::cdc::certification::cdc_certification_snapshot_pinning_is_neutral_under_rewrite_churn`
- `tests::publication::cdc::certification::cdc_certification_rewrite_storm_preserves_exact_suffix_under_tiny_windows`

What is proven:

- pinned snapshots preserve old truth while latest reads see rewritten truth
- CDC output is independent of snapshot pinning state
- long rewrite chains and tiny resume windows preserve canonical suffix semantics

## Additional Hardening Added Before Close

Milestone 1 closeout also includes these extra certification lanes beyond the
minimum roadmap labels:

- subscriber API fuzz matrix over hostile checkpoint/window combinations
- thousand-step seeded convergence matrix
- restart-loop CDC session hardening
- explicit dependency graph resume exactness
- concurrent branch merge pressure

These were added because the codebase standard is closer to runtime
certification than to feature-only completion.

The deterministic seeded, checkpoint-window, dependency-graph, and
snapshot-stability proofs remain in the ordinary test lane. The full
rewrite-storm and thousand-step matrices retain their original workloads in
the daily/manual scheduled relational-certification lane and run explicitly
with `--ignored --exact`.

## Explicit Deferrals

Milestone 1 intentionally does not include:

- live schema-version transition inside an already-running subscriber contract
- schema renegotiation or dual-schema continuation for active CDC subscribers

Those are now tracked explicitly in
[worth_relational_roadmap.md](/Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/plans/worth-relational/worth_relational_roadmap.md)
under `Milestone 5: Schema Evolution and CDC Contract Evolution`.

Milestone 1 guarantees explicit schema mismatch handling, not live schema
evolution.

## Operational Conclusion

Milestone 1 can be treated as closed.

The next product milestone is
[worth_relational_roadmap.md](/Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/plans/worth-relational/worth_relational_roadmap.md)
`Milestone 2: Relational Aspect Semantics`.
