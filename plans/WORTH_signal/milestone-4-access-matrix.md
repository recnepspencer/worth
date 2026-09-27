# worth-signal Milestone 4 Access Matrix

> **Status:** Phase 1 exact read-path matrix and closure artifact
>
> **Parent milestone:** [milestone-4.md](./milestone-4.md)
> **Interior audit:** [milestone-4-interior-heat-audit.md](./milestone-4-interior-heat-audit.md)

## Purpose

This document is the Phase 1 exact read-path access matrix required by
Milestone 4. It records the critical-path lanes that must stop depending on
broad `NodeEntry` reads before storage splitting begins, the exact fields they
consume, and the residual compatibility surfaces that remain explicitly
deferred.

Gate 3 closure now builds on this matrix rather than replacing it. The node and
artifact lanes are structurally split in code, and this document remains the
named baseline for why those split boundaries exist.

The point of this file is to keep Milestone 4 grounded in named read/write
surfaces rather than intuition. Every later storage move must trace back to an
access pattern named here or to an explicitly added matrix update.

## Lane Inventory

### Serial Apply and Suppression

Primary files:

- [apply.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/evaluation/engine/apply.rs)
- [prepared_apply.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/evaluation/engine/prepared_apply.rs)
- [effect.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/graph/runtime/effect.rs)

Current field accesses:

- `dependency_inputs_match_graph`
  reads: `dependencies_id`, `dep_snapshot_id`
  accessor: `node_dependency_ids`
- `resolve_effect_comparator`
  reads: `eval_config.comparator`
  accessor: `node_eval_config`
- `verdict_for_evaluated_result`
  reads: runtime artifact `output_identity`, `continuity_token`
  accessor: `node_runtime_artifact_warm`
- `build_effect_dependency_inputs`
  reads: `dependencies_id`, `dep_snapshot_id`
  accessor: `node_dependency_ids`
- `build_effect_dependency_inputs_for_dependencies`
  reads: source scoped versions
  accessor: `node_version_for_scope`
- `apply_effect_suppression`
  reads: `state`
  mutates: clean-state transition
  accessor: `get_state`, broad mutable write still present pending hot mutable view
- `check_upstream_unchanged_ignoring_source`
  reads: comparator config, source state, source scoped versions, source runtime artifact state
  accessor: `node_eval_config`, `get_state`, `node_version_for_scope`, `node_runtime_artifact_state`

Current classification:

- mandatory hot reads: state, aspect versions, dependency snapshot id, runtime artifact hot facts
- likely warm reads: comparator config, warm artifact continuity and identity metadata
- cold reads should be absent in this lane except through explicit artifact reconstruction paths

Current compatibility pressure:

- broad mutable writes still exist in runtime effect publication until hot mutable views land
- runtime artifact reads still flow through broad `RuntimeArtifactState` rather than split hot/warm forms

### Serial Finalize and Stage Lowering

Primary files:

- [serial_batch.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/planner/apply/serial_batch.rs)
- [stage.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/planner/apply/stage.rs)
- [semantic/mod.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/planner/semantic/mod.rs)

Current field accesses:

- `AppliedSerialTask::from_apply_result`
  reads: `state`, runtime artifact `memoized_origin`, `reuse_basis`
  accessor: `get_state`, `node_runtime_artifact_warm`
- `publish_group_local_task_commit`
  reads: `state`, runtime artifact `memoized_origin`, `reuse_basis`
  accessor: `get_state`, `node_runtime_artifact_warm`
- `lower_serial_task_patch`
  reads: pre-apply `state`, pre-apply finalize image
  accessor: `get_state`, `node_runtime_artifact_finalize_image`
- `lower_task_patch`
  reads: pre-apply `state`, pre-apply finalize image, comparator config
  accessor: `get_state`, `node_runtime_artifact_finalize_image`, `node_eval_config`
- `finalize_stage_batch`
  reads: after finalize image
  accessor: `node_runtime_artifact_finalize_image`
- `finalize_serial_stage_batch`
  reads: after finalize image
  accessor: `node_runtime_artifact_finalize_image`

Current classification:

- hot reads: node state and compact runtime artifact truth needed for finalize decisions
- warm escalation candidates: output identity, continuity token, reuse metadata for reporting surfaces

Current compatibility pressure:

- finalize now consumes `RuntimeArtifactFinalizeImage` instead of broad
  `RuntimeArtifactState` snapshots
- lowered task execution now carries the compact finalize image rather than the
  compatibility carrier
- compatibility pressure remains at the storage mutation boundary and in
  non-hot explain/merge/reporting surfaces, not in the main finalize lane

### Planning and Precompute

Primary files:

- [planning/mod.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/planner/planning/mod.rs)
- [precompute/mod.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/planner/precompute/mod.rs)

Current field accesses:

- `visit_node`
  reads: `state`, `dirty_aspects`, `dirty_partition_scopes`
  accessor: `get_state`, `node_dirty_aspects`, `node_dirty_partition_scopes`
- `admit_planned_node`
  reads: `state`, dirty-scope presence
  accessor: `get_state`, `node_dirty_partition_scopes_present`
- `admit_direct_task_with_policy_resolver`
  reads: `state`
  accessor: `get_state`
- `populate_plan_buffers`
  reads: node validity through state lookup
  accessor: `get_state`
- `classify_reason`
  reads: `state`, dirty-scope presence, runtime artifact `output_change`, `reuse_basis.source`
  accessor: `get_state`, `node_dirty_partition_scopes_present`, `node_runtime_artifact_hot`, `node_runtime_artifact_warm`
- `prepare_condition_outcome_if_blocked`
  reads: `dirty_aspects`, `eval_config.condition`
  accessor: `node_dirty_aspects`, `node_eval_config`
- `max_dependency_delta`
  reads: source scoped versions
  accessor: `node_version_for_scope`

Current classification:

- mixed lane: not as hot as apply, but still on the critical path
- must not regress to broad `NodeEntry` reads after Phase 1 narrowing lands

Current compatibility pressure:

- dirty-scope reads still materialize scoped payload collections rather than a future compact hot header
- maybe-stale validation now uses narrowed config/state/version/hot-artifact
  accessors rather than broad `NodeEntry` reads

### Snapshot Commit

Primary files:

- [entries.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/graph/storage/entries.rs)
- [apply.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/evaluation/engine/apply.rs)

Current field accesses:

- `set_dep_snapshot`
  reads: prior `dep_snapshot_id`
  mutates: next `dep_snapshot_id`
  accessor: broad read/write still present inside storage boundary
- `replace_dep_snapshot_committed`
  reads: prior `dep_snapshot_id`
  mutates: next `dep_snapshot_id`
  accessor: broad write still present inside storage boundary
- `apply_stable_shape_snapshot_batch_commit`
  mutates: `dep_snapshot_id`
  accessor: broad write still present inside storage boundary
- `apply_mixed_snapshot_batch_commit`
  mutates: `dep_snapshot_id`
  accessor: broad write still present inside storage boundary

Current classification:

- authoritative commit lane should converge toward hot-only
- commit-adjacent continuity/report assembly may require explicit warm escalation

Current compatibility pressure:

- snapshot commit is still inside the broad storage mutation boundary
- this is acceptable for Phase 1 because the closure target is read-path discipline, not final hot mutable store layout

### Merge Adoption

Primary file:

- [execute.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/transaction/runtime/state/merge/execute.rs)

Current broad access patterns:

- clones full source `NodeEntry`
- applies carry policy over broad runtime artifact and cold payload state
- mutates target entry through `get_entry_mut`

Current classification:

- not a primary apply hot lane
- merge-semantics-sensitive lane that must remain named because it constrains what may later move warm, cold, or reconstructable

### Checkpoint Capture and Restore

Primary files:

- [graph.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/graph/runtime/graph.rs)
- [snapshotting.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/transaction/runtime/state/branching/snapshotting.rs)
- [mod.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/state/mod.rs)

Current field accesses:

- checkpoint slots now store `node: Option<CheckpointNodeImage>` instead of serializing `NodeEntry` directly
- checkpoint slot deserialization accepts both legacy `entry` payloads and current `node` image payloads
- checkpoint authority deserialization accepts legacy slot payloads through the same bridge
- restore reconstructs `NodeEntry` from `CheckpointNodeImage` rather than treating the persisted schema as the in-memory entry layout

Current classification:

- closure-gate lane for Phase 0
- persistence boundary is now explicitly layout-decoupled from future in-memory SoA storage

Current compatibility pressure:

- restore internals still operate through broad graph mutation boundaries after image decode
- future split-store restore must preserve this image boundary while changing in-memory reconstruction

## Residual Broad Entry Hotspots

The following modules still contain important broad `get_entry` / `get_entry_mut`
usage after Phase 1 closure. They are now explicit residual dependencies, not
unknown debt:

- [effect.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/graph/runtime/effect.rs)
  residual: two known broad mutable writes
- [entries.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/graph/storage/entries.rs)
  residual: storage-boundary snapshot id writes and broad storage assembly
- [execute.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/transaction/runtime/state/merge/execute.rs)
  residual: merge still clones broad entries and carries warm/cold payload
- [observer.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/graph/runtime/observer.rs)
  residual: observer and inspection APIs intentionally consume broad and cold surfaces
- [context_resolution.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/evaluation/reuse/context_resolution.rs)
  residual: reuse context resolution still uses broad entry access
- [graph.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/graph/runtime/graph.rs)
  residual: restore and snapshot authority internals still straddle broad graph mutation boundaries

## Phase 0 Closure Decisions

These Phase 0 decisions are now locked:

- checkpoint serialization uses the explicit `CheckpointNodeImage` boundary rather than persisted `NodeEntry`
- legacy checkpoint slot payloads using `entry` remain load-compatible
- legacy checkpoint authority payloads remain load-compatible through the slot bridge
- current serialization emits the `node` field and does not continue emitting the legacy `entry` field
- runtime artifact state is now explicitly split into hot and warm lanes while preserving a flat serialized schema

## Phase 1 Closure Decisions

These Phase 1 decisions are now in force:

- no new hot-path code may be introduced on broad entry accessors
- apply, finalize, suppression, planning, and precompute read paths are narrowed away from broad `NodeEntry` reads
- residual broad mutable writes are explicitly enumerated and are not allowed to spread silently
- merge and checkpoint/restore remain mandatory named lanes even when they are not part of the main read-path enforcement seam
- conditional fields do not get promoted to hot storage without explicit workload evidence and interior heat audit support
- Phase 1 closure is about read-path discipline and residual-boundary enumeration, not final hot mutable store shape

## Gate 3 Closure Decisions

These Gate 3 decisions are now in force:

- `RuntimeArtifactHot` and `RuntimeArtifactWarm` are real runtime lanes, not
  taxonomy-only documentation
- `NodeEntry` is structurally split into hot, warm, and boxed cold data while
  preserving the serialized compatibility boundary
- planner apply/finalize paths carry `RuntimeArtifactFinalizeImage` instead of
  broad `RuntimeArtifactState` snapshots
- node and artifact lane inline sizes are surfaced through observer metrics as
  lane inventory, not as proof that the node-side lanes already live in fully
  separate physical stores

## Mechanical Enforcement

Phase 1 closure is certified in code by:

- [phase1_api.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/tests/phase1_api.rs)
  `hot_apply_modules_do_not_use_broad_entry_accessors_for_reads`
- [phase1_api.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/tests/phase1_api.rs)
  `hot_effect_runtime_path_avoids_broad_entry_reads`
- [phase1_api.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/tests/phase1_api.rs)
  `hot_stage_path_avoids_broad_entry_reads`
- [phase1_api.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/tests/phase1_api.rs)
  `maybe_stale_validation_path_uses_narrowed_hot_accessors`
- [phase1_api.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/tests/phase1_api.rs)
  `gate3_finalize_paths_use_compact_artifact_images_instead_of_broad_runtime_state_snapshots`

These tests do not prove the final storage split. They prove the narrower but
critical Phase 1 contract:

- the main read-path hot and adjacent-critical modules no longer drift back to broad `get_entry` convenience reads
- the remaining broad mutable writes are explicit and counted
