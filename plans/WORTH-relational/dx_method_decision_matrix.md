# WORTH Relational DX Method Decision Matrix

## Purpose

This is the method-level pass.

The older matrix in
[`dx_export_decision_matrix.md`](./dx_export_decision_matrix.md)
handles modules.

This one handles verbs.

That matters because the real public facade is not just the names re-exported in
[`facade.rs`](../../crates/worth-relational/src/facade.rs).

A lot of the real boundary also comes from public methods on
`RelationalRuntime`,
`RelationalRuntimeBuilder`,
`RelationalTransaction`,
and the access or authority helpers they hand back.

The goal here is not to flatten hard stuff.

The goal is:

- keep real architectural power
- stop leaking half-shaped boundary seams
- make the facade teach one honest mental model

---

## Scope Rule

This doc covers:

- public methods on facade-exported types
- public methods on public helper types returned by facade-exported methods
- public runtime verbs that currently leak non-facade helper surfaces

This doc does not individually classify every tiny getter on every passive data
carrier.

Those inherit the decision of their owning surface unless the getter itself
creates a wrong mental model.

Example:

- a digest struct getter is not a separate DX product decision
- `runtime.history()` absolutely is

---

## Action Legend

- `Keep`
  - keep public and keep visible
- `Condense`
  - keep the capability, but prefer a more guided or more declarative path
- `Contain`
  - keep public, but move it out of the main path
- `Promote Or Remove`
  - current public method is boundary-inconsistent; either promote the returned
    lane into the actual facade story or stop exposing it publicly

## Boundary Legend

- `Primary`
  - main product memory shape
- `Guided`
  - still central, but should be taught through a narrower workflow
- `Contained`
  - architecturally real, but not part of first-use memory
- `Leak`
  - public seam that does not currently earn its exposure

---

## Root Runtime

Owner:
`RelationalRuntime`

| Method | Boundary | Action | Decision |
| --- | --- | --- | --- |
| `new` | `Guided` | `Contain` | Keep available, but push normal setup toward `RelationalRuntimeApi::builder()` and profile-first construction. |
| `new_with_custom_invariants` | `Contained` | `Contain` | Real power, but not the first story. Better as an explicit extension lane. |
| `new_with_extensions` | `Contained` | `Contain` | Same as above. Powerful, not first-path. |
| `fork` | `Contained` | `Contain` | Architecturally real runtime operation. Keep public, but not on the main quick-start path. |
| `set_execution_model` | `Contained` | `Contain` | Real tuning knob, but mutable execution-shape switching should not be a main-path runtime memory. |
| `config` | `Primary` | `Keep` | Resolved config is real architecture and should stay easy to inspect. |
| `commit_strategy_registry` | `Contained` | `Contain` | Good read surface for strategy introspection, but not central runtime memory. |
| `commit_strategies` | `Guided` | `Keep` | This is the right public entry into strategy execution. |
| `commit_strategies_authority` | `Contained` | `Contain` | Real authority surface. Keep public, but clearly specialist. |
| `snapshots` | `Guided` | `Keep` | This is the right controlled-view door for current or pinned truth. |
| `read_truth` | `Primary` | `Keep` | This is the primary current-truth read lane now. |
| `validation` | `Contained` | `Keep` | Real public validation lane and the right name for it. |
| `compiled_artifacts` | `Contained` | `Keep` | Real read-side compiled-artifact lane. |
| `compiled_artifacts_authority` | `Contained` | `Contain` | Real authority side of the compiled-artifact lane. |
| `retention` | `Contained` | `Keep` | Real retention lane and the right public name for it. |
| `history` | `Guided` | `Keep` | This is the right read door into history. |
| `history_authority` | `Contained` | `Contain` | Real authority, but should stay out of the main runtime path. |
| `inspect_what_happened` | `Contained` | `Keep` | Good question-shaped inspection door and a much clearer top-level name. |
| `index_access` | `Contained` | `Contain` | Keep public as subsystem access, not main product memory. |
| `index_authority` | `Contained` | `Contain` | Same. Real authority, specialist lane. |
| `publication` | `Contained` | `Keep` | This is the clean read entry for publication state. |
| `replay` | `Contained` | `Keep` | Good read-side replay lane. |
| `replay_authority` | `Contained` | `Contain` | Keep public, but clearly specialist and authority-shaped. |
| `durability` | `Contained` | `Keep` | Legit read lane for recovery and storage durability state. |
| `durability_authority` | `Contained` | `Contain` | Keep public, clearly specialist. |
| `storage_access` | `Contained` | `Contain` | Useful support lane. Keep available, not central. |
| `merge` | `Contained` | `Keep` | Good specialist read/planning door. |
| `prepare_merge_execution` | `Contained` | `Keep` | Strong top-level guided verb for merge execution prep. |
| `execute_prepared_merge` | `Contained` | `Keep` | Strong top-level guided verb for merge execution. |
| `certify_current_state` | `Contained` | `Keep` | This is a good top-level authority verb and should stay public. |

### Root Runtime Take

The biggest runtime cleanup was not deleting power.

It was making a hard call on the seam methods that exposed public helper
types without giving them a real facade lane.

That lane-ownership call is now resolved in code:

- `read_truth()` is the primary current-truth read lane
- `validation()` is the contained validation lane
- `compiled_artifacts()` and `compiled_artifacts_authority()` are the contained
  compiled-artifact lane
- `retention()` is the contained retention lane
- `snapshots()` is the controlled-view lane

The remaining job is polish and consistency, not basic lane ownership.

---

## API Entry

Owner:
[`RelationalRuntimeApi`](../../crates/worth-relational/src/presentation/api.rs)

| Method | Boundary | Action | Decision |
| --- | --- | --- | --- |
| `builder` | `Primary` | `Keep` | This should stay the default public entry. |

---

## Builder

Owner:
`RelationalRuntimeBuilder`

### Core Builder Spine

| Method | Boundary | Action | Decision |
| --- | --- | --- | --- |
| `new` | `Primary` | `Keep` | Good starting point. |
| `profile` | `Primary` | `Keep` | Profile-first setup should stay central. |
| `runtime_name` | `Guided` | `Contain` | Fine as metadata, not core runtime identity. |
| `build` | `Primary` | `Keep` | Essential close to the builder story. |

### Execution And Authority Knobs

| Method | Boundary | Action | Decision |
| --- | --- | --- | --- |
| `execution_model` | `Guided` | `Keep` | Real setup knob and should stay easy to reach. |
| `planning` | `Contained` | `Contain` | Keep available, but this feels more planner-contract tuning than day-one runtime setup. |
| `commit_authority` | `Contained` | `Contain` | Real authority knob, but not first-path. |
| `durability_mode` | `Guided` | `Keep` | Important runtime-shape choice. |
| `diagnostics` | `Guided` | `Keep` | Worth keeping close because diagnostics are part of the product, not an afterthought. |

### Schema And Invariant Setup

| Method | Boundary | Action | Decision |
| --- | --- | --- | --- |
| `schema_registry` | `Primary` | `Keep` | Core setup input. |
| `invariant_catalog` | `Guided` | `Keep` | Real architecture. Keep it visible, but explain it as runtime contract setup, not random advanced flavor. |
| `custom_invariant` | `Contained` | `Contain` | Keep public, but clearly an extension lane. |

### Commit Strategy Setup

| Method | Boundary | Action | Decision |
| --- | --- | --- | --- |
| `commit_strategy` | `Contained` | `Contain` | Real subsystem registration. |
| `commit_strategy_executor` | `Contained` | `Contain` | Same. Important, but not first-path. |

### Capacity And Storage Knobs

| Method | Boundary | Action | Decision |
| --- | --- | --- | --- |
| `entity_capacity` | `Contained` | `Contain` | Tuning knob, not main facade memory. |
| `relation_capacity` | `Contained` | `Contain` | Same. |
| `mvcc` | `Guided` | `Contain` | Real config, but should eventually live in a clearer config section story. |
| `storage_layout` | `Contained` | `Contain` | Keep public, clearly deployment or durability shaped. |
| `publication` | `Guided` | `Contain` | Real config, but should eventually read as part of grouped config sections rather than a flat pile. |
| `payload_policy` | `Guided` | `Contain` | Same. |
| `symbol_policy` | `Guided` | `Contain` | Same. |
| `visibility_cache_policy` | `Contained` | `Contain` | Support tuning, not main path. |
| `durable_log_policy` | `Contained` | `Contain` | Durability lane. |
| `durability_policy` | `Contained` | `Contain` | Durability lane. |
| `durable_store_layout` | `Contained` | `Contain` | Durability lane. |
| `adjacency_policy` | `Contained` | `Contain` | Real storage/graph knob, not central setup memory. |
| `cross_context_policy` | `Guided` | `Contain` | Important semantics knob, but it belongs in a grouped config story. |
| `cascade_delete_policy` | `Guided` | `Contain` | Same. |
| `compiled_lane_policy` | `Contained` | `Contain` | Specialist performance or execution knob. |
| `relation_integrity_scope_budget` | `Contained` | `Contain` | Useful hard-safety tuning, but not general setup memory. |

### Builder Take

The builder should stay powerful.

The DX problem is not "too many knobs exist."

The problem is that the knobs currently arrive as a flat stream instead of a
clear subsystem config story.

---

## Commit Strategies

Owners:

- [`CommitStrategiesFacade`](../../crates/worth-relational/src/commit_strategies/facade.rs)
- [`CommitStrategiesAuthorityFacade`](../../crates/worth-relational/src/commit_strategies/facade.rs)
- [`FrozenCommitStrategyRegistry`](../../crates/worth-relational/src/commit_strategies/frozen_registry.rs)

### Read And Execution Entry

| Method | Boundary | Action | Decision |
| --- | --- | --- | --- |
| `canonicalize_request` | `Guided` | `Keep` | Good entry into the strategy pipeline. |
| `execute` | `Guided` | `Keep` | Good execution-stage verb. |

### Authority Lane

| Method | Boundary | Action | Decision |
| --- | --- | --- | --- |
| `lower_execution` | `Contained` | `Contain` | Keep public. This is real pipeline power, but specialist. |
| `execute_lowered_commit` | `Contained` | `Contain` | Same. |
| `validate_lowered_plan` | `Contained` | `Contain` | Same. |
| `execute_validated_commit` | `Contained` | `Contain` | Same. |

### Registry Introspection

| Method | Boundary | Action | Decision |
| --- | --- | --- | --- |
| `is_empty` | `Contained` | `Contain` | Fine helper on a contained surface. |
| `len` | `Contained` | `Contain` | Same. |
| `iter` | `Contained` | `Contain` | Same. |
| `get_by_id` | `Contained` | `Contain` | Same. |
| `get_by_name` | `Contained` | `Contain` | Same. |
| `registry_digest` | `Contained` | `Contain` | Same. |

---

## Transactions

Owner:
`RelationalTransaction`

| Method | Boundary | Action | Decision |
| --- | --- | --- | --- |
| `transaction_id` | `Guided` | `Keep` | Basic transaction identity is normal and should stay easy. |
| `push_batch` | `Primary` | `Keep` | Core mutation building verb. |
| `create_savepoint` | `Guided` | `Keep` | Good transactional control verb. |
| `rollback_to_savepoint` | `Guided` | `Keep` | Same. |
| `merged_plan` | `Contained` | `Contain` | Real planning insight, but clearly deeper than normal transaction use. |
| `plan_bulk_mutation_batch` | `Guided` | `Keep` | Worth keeping visible because it condenses a complicated workflow. |
| `admit_naming_stable_bulk_mutation_batch` | `Contained` | `Contain` | Real phase step, but too implementation-shaped for the main path. |
| `admit_lineage_safe_bulk_mutation_batch` | `Contained` | `Contain` | Same. |
| `admit_provenance_complete_bulk_mutation_batch` | `Contained` | `Contain` | Same. |
| `inspect_staging` | `Contained` | `Contain` | Useful debug and introspection verb, not main flow. |
| `commit` | `Primary` | `Keep` | Core transaction close verb. |

### Transaction Take

The bulk mutation trio with `admit_*` names is real architecture, but it is a
great example of a surface that should stay available while getting condensed
behind a better top-level workflow.

---

## History

Owners:

- `HistoryAccess`
- `HistoryAuthority`

### Read Lane

| Method | Boundary | Action | Decision |
| --- | --- | --- | --- |
| `latest_commit` | `Guided` | `Keep` | Good everyday history read. |
| `branch_head` | `Guided` | `Keep` | Same. |
| `branches` | `Guided` | `Keep` | Same. |
| `version_graph` | `Contained` | `Contain` | Real history shape view, but deeper than normal reads. |
| `ancestor_closure_by_commit_id_order` | `Contained` | `Contain` | Specialist ancestry tooling. |
| `latest_common_ancestor_between_branches` | `Contained` | `Contain` | Keep public, merge-adjacent. |
| `can_merge_branch_into` | `Contained` | `Contain` | Keep public, merge-adjacent. |
| `inspect_merge` | `Contained` | `Contain` | Merge path, not general history path. |
| `entity_aspect_history` | `Contained` | `Keep` | Real and useful historical query. |
| `relation_aspect_history` | `Contained` | `Keep` | Same. |
| `entity_aspect_history_with_trace` | `Contained` | `Contain` | Keep, but specialist trace lane. |
| `relation_aspect_history_with_trace` | `Contained` | `Contain` | Same. |
| `lineage_entity_aspect_history` | `Contained` | `Contain` | Real power, but clearly deeper. |
| `lineage_entity_aspect_history_with_trace` | `Contained` | `Contain` | Same. |

### Authority Lane

| Method | Boundary | Action | Decision |
| --- | --- | --- | --- |
| `create_branch` | `Contained` | `Keep` | Important authority verb and should stay public. |
| `retain_version_for_replay` | `Contained` | `Contain` | Real retention authority, but specialist. |
| `release_version_replay_retention` | `Contained` | `Contain` | Same. |

---

## Inspection

Owner:
`InspectionAccess`

| Method | Boundary | Action | Decision |
| --- | --- | --- | --- |
| `inspect_commit` | `Contained` | `Keep` | Good question-shaped entry. |
| `inspect_recent_commits` | `Contained` | `Keep` | Same. |
| `inspect_branch_head` | `Contained` | `Keep` | Same. |
| `connectivity_summary` | `Contained` | `Keep` | Real inspection job. |
| `neighbors` | `Contained` | `Keep` | Same. |
| `graph_summary` | `Contained` | `Keep` | Same. |
| `kind_summary` | `Contained` | `Keep` | Same. |
| `open_historical_view` | `Contained` | `Keep` | Real and helpful. |
| `inspect_historical_record` | `Contained` | `Keep` | Same. |
| `retention_summary` | `Contained` | `Keep` | Good inspection-shaped read. |
| `inspect_record_retention` | `Contained` | `Keep` | Same. |
| `inspect_snapshot_pinning` | `Contained` | `Keep` | Same. |
| `inspect_retention_execution` | `Contained` | `Keep` | Same. |
| `structural_identity` | `Contained` | `Keep` | Good inspection job. |
| `compare_structural_identity` | `Contained` | `Keep` | Same. |
| `query_structural_identity` | `Contained` | `Keep` | Same. |

### Inspection Take

Inspection is actually in decent shape at the verb level.

Its problem is not bad verbs.

Its problem is that the surrounding noun cloud makes the lane harder to
recognize than it should be.

---

## Merge

Owner:
`MergeAccess`

| Method | Boundary | Action | Decision |
| --- | --- | --- | --- |
| `inspect_history_scope` | `Contained` | `Keep` | Good specialist planning verb. |
| `inspect_planning_scope` | `Contained` | `Keep` | Good specialist planning verb. |
| `prepare_merge_execution` | `Contained` | `Keep` | Core merge prep verb. |
| `verify_prepared_merge_execution` | `Contained` | `Contain` | Keep public for specialists, but not part of the everyday merge story. |

---

## Replay

Owners:

- `ReplayAccess`
- `ReplayAuthority`

| Method | Boundary | Action | Decision |
| --- | --- | --- | --- |
| `canonical_commit_envelope` | `Contained` | `Keep` | Good read-side replay primitive. |
| `compare_outcome` | `Contained` | `Keep` | Same. |
| `replay_commit` | `Contained` | `Keep` | Core replay authority verb. |
| `replay_range` | `Contained` | `Contain` | Keep public, but clearly a deeper specialist operation. |

---

## Publication

Owners:

- `PublicationAccess`
- [`RelationalDiagnosticsFacade`](../../crates/worth-relational/src/diagnostics/data/mod.rs)
- [`PublicationError`](../../crates/worth-relational/src/publication/data/publication_error.rs)

### Publication Reads

| Method | Boundary | Action | Decision |
| --- | --- | --- | --- |
| `diagnostics` | `Contained` | `Keep` | Good top-level publication read. |
| `diagnostic_artifacts` | `Contained` | `Contain` | Keep public, but artifact-level raw access is a deeper lane. |
| `diagnostics_since` | `Contained` | `Keep` | Useful job-shaped read. |
| `latest_bundle` | `Contained` | `Keep` | Good publication summary entry. |
| `latest_patch` | `Contained` | `Keep` | Good publication read. |
| `latest_replay` | `Contained` | `Keep` | Good publication read. |
| `read_patch_stream` | `Contained` | `Keep` | Real product surface and should stay public. |
| `read_subscriber_stream` | `Contained` | `Keep` | Same. |

### Diagnostics View Helper

| Method | Boundary | Action | Decision |
| --- | --- | --- | --- |
| `artifacts` | `Contained` | `Contain` | Fine helper on a contained diagnostics lane. |
| `by_scope` | `Contained` | `Contain` | Same. |
| `minimal_summaries` | `Contained` | `Contain` | Same. |

### Error Helper

| Method | Boundary | Action | Decision |
| --- | --- | --- | --- |
| `new` | `Contained` | `Contain` | Constructor is fine, but `PublicationError` is not where the DX fight is. |

---

## Durability

Owners:

- [`DurabilityAccess`](../../crates/worth-relational/src/durability/access/mod.rs)
- [`DurabilityAuthority`](../../crates/worth-relational/src/durability/authority.rs)
- `SnapshotGuard`
- [`VisibilityAuthority`](../../crates/worth-relational/src/visibility/authority.rs)

### Durability Reads And Writes

| Method | Boundary | Action | Decision |
| --- | --- | --- | --- |
| `recovery_plan` | `Contained` | `Keep` | Core durability read verb. |
| `durable_log` | `Contained` | `Contain` | Keep public, but more specialist than `recovery_plan`. |
| `durable_branch_heads` | `Contained` | `Contain` | Same. |
| `checkpoint` | `Contained` | `Keep` | Core durability authority verb. |
| `compact_store` | `Contained` | `Contain` | Keep public, but more operational. |
| `recover` | `Contained` | `Keep` | Core authority verb. |

### Snapshot Authority

| Method | Boundary | Action | Decision |
| --- | --- | --- | --- |
| `snapshot` | `Contained` | `Keep` | Real, simple, useful authority verb. |
| `pin_snapshot` | `Contained` | `Contain` | Keep, but more specialist. |
| `release_snapshot` | `Contained` | `Contain` | Same. |
| `handle` | `Contained` | `Contain` | Fine helper on a contained type. |
| `snapshot_id` | `Contained` | `Contain` | Fine helper on a contained type. |

---

## Indexes

Owners:

- `IndexAccess`
- `IndexAuthority`

| Method | Boundary | Action | Decision |
| --- | --- | --- | --- |
| `latest_generation` | `Contained` | `Keep` | Good direct read. |
| `generations_for_version` | `Contained` | `Keep` | Same. |
| `execute_query_plan_with_fallback_parity` | `Contained` | `Contain` | Real power, but very specialist. |
| `register` | `Contained` | `Contain` | Real authority, not main path. |
| `build_for_commit` | `Contained` | `Contain` | Same. |

---

## Storage And Performance

Owners:

- `StorageAccess`
- `PerformanceAccess`

### Storage Reads

| Method | Boundary | Action | Decision |
| --- | --- | --- | --- |
| `partition_ids` | `Contained` | `Contain` | Support-level read. |
| `partition_storage_stats` | `Contained` | `Contain` | Same. |
| `storage_stats` | `Contained` | `Contain` | Same. |
| `chunked_storage_summary` | `Contained` | `Contain` | Same. |
| `chunk_diagnostics` | `Contained` | `Contain` | Same. |
| `plan_read_explicit_query_packet` | `Contained` | `Contain` | Specialist planning support. |
| `outgoing_relations_for_entity` | `Contained` | `Contain` | Structural support helper. |
| `incoming_relations_for_entity` | `Contained` | `Contain` | Structural support helper. |
| `all_relations_for_entity` | `Contained` | `Contain` | Structural support helper. |

### Performance Reads

| Method | Boundary | Action | Decision |
| --- | --- | --- | --- |
| `contracts` | `Contained` | `Contain` | Keep public as a support lane. |
| `counters` | `Contained` | `Contain` | Same. |
| `reset_counters` | `Contained` | `Contain` | Same. |

---

## Biggest Method-Level Calls

If we want the method surface to feel honest before bridge work, the biggest
calls are:

1. Keep the clean top-level verbs visible.
2. Contain the specialist authority and planning verbs, not by hiding them, but
   by giving them a clearly specialist lane.
3. Keep the remaining subsystem lanes explicit without letting helper naming
   leak back into the published story.

The clean keepers are things like:

- `RelationalRuntimeApi::builder`
- `RelationalRuntime::config`
- `RelationalRuntime::read_truth`
- `RelationalRuntime::snapshots`
- `RelationalRuntime::history`
- `RelationalRuntime::publication`
- `RelationalRuntime::commit_strategies`
- `RelationalRuntime::prepare_merge_execution`
- `RelationalRuntime::execute_prepared_merge`
- `RelationalRuntime::certify_current_state`
- `RelationalTransaction::push_batch`
- `RelationalTransaction::commit`
- `PublicationAccess::read_patch_stream`
- `PublicationAccess::read_subscriber_stream`
- `DurabilityAuthority::recover`

The clearest condensation or containment targets are things like:

- the `RelationalRuntimeBuilder` long flat knob list
- the `RelationalTransaction::admit_*` trio
- `MergeAccess::verify_prepared_merge_execution`
- `HistoryAccess` trace-heavy and merge-adjacent verbs
- `IndexAccess::execute_query_plan_with_fallback_parity`

The clearest remaining condensation targets are:

- the flat `RelationalRuntimeBuilder` knob list
- the write-truth split between transaction nouns and admission-phase detail
- how current truth, snapshots, query, inspection, history, and publication are
  taught as one coherent story
- how the deep lanes stay explicit without feeling like random leftovers

Those are now mostly workflow-shape problems, not "what lane does this belong
to?" problems.

---

## Follow-Through

This method matrix means the next DX docs should now be written against the real
verb surface, not just the noun inventory.

Immediate follow-through:

1. update
   [`dx_export_decision_matrix.md`](./dx_export_decision_matrix.md)
   so it explicitly points at this method-level pass
2. use this doc to write
   [`dx_canonical_surface_spec.md`](./dx_canonical_surface_spec.md)
3. use
   [`dx_boundary_cleanup_list.md`](./dx_boundary_cleanup_list.md)
   as the resolved lane-ownership record before condensation and naming work
   continue
