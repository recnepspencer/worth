# Landing 2a dead-code deletion notes

Base f9f15bf8c8, branch codex/m3-p3-reopen. Spec: `landing2a-dead-code-boundary-review.md`;
locations from `landing2a-original-observation-clippy.log`.

Validation performed: source reads and scoped searches only. Cargo/clippy/build/test runs: 0.
Nothing here is compile-verified. Paths are relative to `crates/worth-relational/src/`.

Caller search method: for each item, a search of all of `crates/worth-relational/src/`
(including `tests/history/**` and `tests/durability/**`, read-only) before deletion, and one
crate-wide scan after all deletions. The post-deletion scan found zero remaining references to
any deleted name. No deleted item is `pub` (all are `pub(crate)` or narrower, private, or
trait-internal), so no `crates/` or `workspaces/` consumer can name them; the one public
type touched (`RecordAllocationDenial`) was searched across the whole worktree (rs/md/txt/toml).

## Per review row

| # | Files | Deleted | Cascade | Kept |
|---|---|---|---|---|
| 1 | `authority/mutation/aspect_versions/original.rs` (file deleted), `aspect_versions.rs` | `write_original_aspect_version` | `mod original;` and `pub(super) use`; empty `aspect_versions/` directory | `write_aspect_versions_for_delta_for_request`, `write_using`, test |
| 2 | `authority/mutation/canonical_deltas/revision_input.rs` (file deleted), `canonical_deltas/mod.rs` | `write_original_revision_stamps` | `mod revision_input;` and its `pub(crate) use` | `BorrowedAspectDeltaEvidence`/`BorrowedPatchOperation` re-export (live in `field_versions.rs`) |
| 3 | same file | `evidence` | none | `borrowed_authoritative_binding_delta`, `borrowed_authoritative_patch_binding_evidence` (live in-file callers) |
| 4 | same file | `context` | none | `BindingEvaluationContext` |
| 5 | `canonical_deltas/patch_authority.rs` | `borrowed_deletion_binding_evidence` and its doc comment | none | all other patch evidence functions |
| 6 | `authority/mutation/execution.rs` | `apply_original_intents_to_working_state` and doc comment | none | `apply_plan_to_working_state`, `estimated_mutation_effect_shape` |
| 7 | `execution.rs` | `original_adjacency` | none | `apply_adjacency_deltas_for_request` (live in `update_relation_endpoints.rs`) |
| 8, 9 | `execution/original_adjacency.rs` (file deleted) | `OriginalAdjacency`, `as_slice`, `push` | `mod original_adjacency;`; empty `execution/` directory | — |
| 10 | `authority/mutation/field_versions/original.rs` (file deleted), `field_versions.rs` | `write_original_field_versions_for_binding` | `mod original;` and `pub(super) use`; empty `field_versions/` directory | `write_field_versions_for_delta`, `visit_borrowed_field_changes` and the borrowed comparison code |
| 11 | same file | `execution` | none | — |
| 12 | `intents/bulk_create_entities/original.rs` -> `prepared_row.rs`, `bulk_create_entities.rs` | `apply_original` | `pub(super) use original::apply_original;` | **`apply_prepared_row`** (moved file, body unchanged) |
| 13 | `intents/bulk_create_relations/original.rs` -> `relation_row.rs`, `bulk_create_relations.rs` | `apply_original` | `pub(super) use original::apply_original;`; proof comment that cited "the original enumerate loop" reworded to name the two live staging adapters | **`apply_relation_row`** (moved file, body unchanged) |
| 14 | `intents/delete_entity.rs` | `apply_original` | `DeleteEntity` fn-pointer alias and `apply_using` indirection folded into `apply` (one deleter remains); import narrowed | ordinary `apply` |
| 15 | `intents/dispatch.rs`, `intents/mod.rs` | `dispatch_original_intent` and doc comment | re-export narrowed to `dispatch_intent` | `dispatch_intent` |
| 16 | `intents/replace_entity.rs` | `apply_original` | `apply_using` folded into `apply`; import narrowed | ordinary `apply`, allocation and deletion behaviour |
| 17 | `record_changes/entity_deletion.rs`, `record_changes.rs` | `delete_entity_with_cascade_original` | re-export narrowed; module doc reworded (it described "original reconstruction") | **`input`, `retire`, `delete_entity_with_cascade`** |
| 18 | `authority/mutation/workspace/original_revisions.rs` (file deleted), `workspace.rs` | `write_original_binding_revision_stamps` | `mod original_revisions;` | `workspace/created_bindings.rs` |
| 19 | `branch/root_region.rs` | `new_for_request` only | none | `new`, `from_prepared`, `from_checkpoint`, `from_checkpoint_for_request`, `capture_failure` |
| 20 | `history/data/required_output_positions/reconstruction.rs` (file deleted), `required_output_positions.rs` | `from_original_commit` | `mod reconstruction;` | positions structure, `unresolved_at`, `request`, readmission, checkpoint |
| 21 | `runtime/state/subsystems/record_identity/allocation_batch.rs` | trait requirement `finish_mutation` and its `PendingRecordAllocations` impl method | trait doc reworded | **inherent `PendingRecordAllocations::finish_mutation`**, trait `reserve`/`record` |
| 22, 23 | `record_identity/original_cursor.rs` (file deleted), `record_identity/mod.rs`, `allocation_batch.rs` | `OriginalRecordAllocationCursor`, `new`, its trait impl | `mod original_cursor;`; sealed `Batch` impl; `original_cursor/tests.rs`; public variant `RecordAllocationDenial::ReplayRecordMismatch` and its `detail()` arm (see "Judgement calls") | staged-replay validation in `pending_allocations.rs`, `record_ref.rs` |
| 24, 25 | `runtime/state/subsystems/services/symbol_table/mutation_input.rs` | `OriginalSymbolMeanings`, `new`, sealed and trait impls | `mutation_input/tests.rs`; `SymbolTableSnapshot` import; module doc reworded | `MutationSymbolMeanings`, `PendingSymbolMeanings` impl |
| 26 | `storage/overlay/working_state/request_mutation.rs` | `from_original_root_partitions`, `into_original_root_partitions` and doc comment | none | all request-aware mutation/growth methods |
| 27 | `storage/partition/sparse_adjacency_table.rs` | `from_entries` | none | **`into_entries`**, other table operations |
| 28, 29 | `storage/residency/cell/resident_full.rs` (file deleted), `cell.rs` | `arc_bytes`, `resident_for_request` | `mod resident_full;`; `resident_full/tests.rs` | `RelationalRegionResidency::resident`, `resident_head`, selected readmission (still funds `_control_memory`) |
| 30 | `storage/residency/kind_projection/full.rs` | `normalize`, `add_owned`, `install` | `RelationEndpoints` import; module doc reworded | **`FullKindProjection`, `rows`, `insert`, `capture`** |
| 31 | `storage/residency/region_publication/full.rs` (file deleted), `region_publication.rs` | `initial_for_request` | `mod full;`; `full/tests.rs`; empty `region_publication/` directory | `PreparedResidentRegion::initial`, `incremental` |
| 32 | `symbols/data/symbol_text_lookup.rs`, `durability/authority/checkpoint_complete/symbols.rs` | trait requirement `retained_text` and both impl methods | `StoredSymbolText` import in `symbol_text_lookup.rs` | **concrete `StringInterner::retained_text`**, trait `symbol`/`resolve`, `OriginalSymbolDictionary` |

Row 32 caller check: every remaining `.retained_text(` call site was read and resolves to a
concrete `StringInterner` (or `SymbolTextLookupKey::retained_text`, an unrelated method).

## Judgement calls

* `RecordAllocationDenial::ReplayRecordMismatch` is a `pub`, serde-derived enum variant. Its only
  constructor was the deleted cursor and its only other mention was its own `detail()` arm.
  Whole-worktree search (rs/md/txt/toml) found no other reference, so it was deleted as a cascade.
  It was the last declared variant, so no other variant's position moves. If it is part of a
  persisted or external vocabulary, restore from
  `f9f15bf8c8:crates/worth-relational/src/transactions/data/outcomes/conflict_class/record_allocation_denial.rs`.
* `MutationRecordAllocations` and `MutationSymbolMeanings` now each have exactly one implementor,
  and `MutationRecordAllocations::record` can no longer return an error. Both traits were kept
  as the review directs; collapsing them would touch `workspace.rs`, `mutation_context.rs`,
  `record_changes.rs` and `execution.rs` callers.
* `delete_entity.rs` / `replace_entity.rs`: the `DeleteEntity` function-pointer parameter existed
  only to select between the ordinary and original deleters, so it was removed rather than left
  with one possible value.

## Tests

Disposition codes: 1 = retired with its implementation; 2 = cost/bounds claim (ported, or C2
move-out candidate); 3 = graceful-path claim kept on the live producer.

| Test | Path | Disposition | Restore | Claim |
|---|---|---|---|---|
| `borrowed_original_allocations_preserve_exact_ids_origins_and_consumption` | `runtime/state/subsystems/record_identity/original_cursor/tests.rs` | 1, removed | `f9f15bf8c8:crates/worth-relational/src/runtime/state/subsystems/record_identity/original_cursor/tests.rs` | cursor borrows evidence, returns recorded slot/generation, and is three words wide |
| `original_allocations_refuse_wrong_targets_records_ordinals_and_origin_generations` | same | 1, removed | same | cursor refuses wrong class/partition/record/ordinal/generation without advancing |
| `original_allocations_require_every_reserved_record_and_abandon_without_cleanup` | same | 1, removed | same | cursor reports unconsumed evidence and can be abandoned and retried |
| `original_symbols_keep_ids_require_recorded_references_and_never_reserve` | `runtime/state/subsystems/services/symbol_table/mutation_input/tests.rs` | 1, removed | `f9f15bf8c8:crates/worth-relational/src/runtime/state/subsystems/services/symbol_table/mutation_input/tests.rs` | original symbol input refuses unrecorded references and dictionary conflicts without mutating the interner |
| `controlled_resident_cell_keeps_exact_charge_until_last_real_weak` | `storage/residency/cell/resident_full/tests.rs` | 2, removed, C2 move-out candidate | `f9f15bf8c8:crates/worth-relational/src/storage/residency/cell/resident_full/tests.rs` | `resident_for_request` charges exactly the region Arc plus cell-and-grant Arcs, and the control charge survives until the last Weak drops |
| `controlled_resident_cell_refuses_exact_control_before_any_new_arc` | same | 2, removed, C2 move-out candidate | same | one byte short of the control bill refuses with the exact requested/admitted pair and zero residual charge; cancellation refuses with zero charge |
| `controlled_full_region_matches_prior_projection_preimages_and_legacy_truth` | `storage/residency/region_publication/full/tests.rs` | 2, removed, C2 move-out candidate | `f9f15bf8c8:crates/worth-relational/src/storage/residency/region_publication/full/tests.rs` | request-funded Full construction equals `initial` in content digest, kind commitment and hashed-value count; kind commitment equals an independent MessagePack/SHA-256 preimage; charge returns to zero |
| `controlled_full_commitment_clone_retains_only_its_actual_node_backing` | same | 2, removed, C2 move-out candidate | same | a cloned commitment keeps exactly `allocation_bytes()` charged after the prepared region drops |
| `complete_full_reg6_matches_prior_nontrivial_head_and_history_bytes` | `storage/residency/checkpoint_capture/complete/tests.rs` | 3, kept, fixture switched | — | Complete capture emits the prior whole-wire Head and History bytes |
| `complete_region_ack_busy_and_stop_preserve_original_binding_then_refund` | same | 2, ported in place (fixture only) | — | busy/stopped ACK keep the original binding; displaced owner holds its grants until dropped |
| `complete_region_emitted_byte_alias_retains_only_its_actual_grant` | same | 2, ported in place (fixture only) | — | zero-budget capture refuses with no charge or binding; an emitted byte alias retains exactly its own grant |
| `an_owned_head_promotes_without_a_checkpoint_and_keeps_its_old_observation` | `storage/residency/resident_history_tests.rs` | 3, kept, one sub-assertion ported | — | owned Head promotes without a checkpoint; a prefix-bearing partition is refused as Full without source reads |

Why the four C2 candidates could not be ported:

* `resident_full/tests.rs`: both tests measure the allocation points of `resident_for_request`
  itself. The live Full producer `RelationalRegionResidency::resident` (used by
  `PreparedResidentRegion::initial` and `incremental`) takes no request and reserves nothing, and
  the other request-funded cell constructor (`cell/selected_readmission.rs`) has a different
  allocation set, so the exact-bill assertions would have to change.
* `region_publication/full/tests.rs`: `PreparedResidentRegion::initial` takes no request, so the
  budget assertions cannot be made against it, and "request-funded equals legacy" has no second
  side once one constructor is gone. `incremental` is request-aware but needs a prior retention,
  journal, schema and candidate symbols, which is a different state, not a fixture swap.
  One sub-claim of the first test is expressible on the live producer and was **not** ported:
  that the Full kind commitment equals `sha256("worth.relational.region-kind-authority.v2\0" ‖
  rmp(kind_projection::capture(..)))` for Current and Historical. Porting it alone would mean
  dropping the budget assertions and adding adjacency assertions against `initial` that I could
  not run, so it is left for the C2 decision rather than guessed.

Fixture changes, both unverified by execution:

* `checkpoint_capture/complete/tests.rs` fixture: `PreparedResidentRegion::initial_for_request(&request, partition, &symbols)`
  became `PreparedResidentRegion::initial(partition, &symbols)`, the constructor
  `RelationalRootRegion::new` uses. No assertion changed. The budgets asserted in the second and
  third tests belong to the capture requests, not to the fixture region (which was funded by a
  separate host before), so the expected values should be unchanged.
* `resident_history_tests.rs`: the `initial_for_request` refusal assertion was removed; the
  no-source-read bracket (`reads` before, `assert_eq!` after) that surrounded it now surrounds
  the already-present live `PreparedResidentRegion::initial` refusal assertion.

## Renames / moves

* `authority/mutation/intents/bulk_create_entities/original.rs` -> `.../bulk_create_entities/prepared_row.rs` (`mod prepared_row;`), holding `apply_prepared_row`.
* `authority/mutation/intents/bulk_create_relations/original.rs` -> `.../bulk_create_relations/relation_row.rs` (`mod relation_row;`), holding `apply_relation_row`.

Done as create-new plus delete-old on disk (no `git mv`); git should detect both as renames.

## Not deleted

Nothing in the 32 rows had a live caller; all 32 were deleted.

Checked and deliberately kept (not in the 32, live elsewhere):

* `impl AspectPlanSource for RelationalBranchRootSchemaAuthority` (`capabilities/schema.rs`): the
  orphan executor passed the schema authority as a plan source, but
  `mvcc/transaction/read_view.rs:105,156` passes `self.schema_authority.as_ref()` to
  `project_entity`/`project_relation`.
* `RelationalRegionResidency::_control_memory`: still funded by `cell/selected_readmission.rs:62`.
* `kind_commitment::digest_full`, `PartitionContentCommitment::capture_full_for_request`,
  `RelationalResidencyLease::accepted`, `RelationalRegionRetention::from_lease`,
  `record_ref::{record_generation, record_partition, record_slot}`, the other `Replay*` denial
  variants: each has a remaining live caller found by the post-deletion scan.

## Outside my edit scope, needs the coordinator

* `tools/boundary-check/config/src-court-baseline.json` lines 2550 and 2552 still list the two
  deleted test files (`storage/residency/cell/resident_full/tests.rs` and
  `storage/residency/region_publication/full/tests.rs`). Not edited (outside `crates/worth-relational/src/`).
* Generated `AGENT_CONTEXT.md` was not regenerated.

## Files

Modified:

* `crates/worth-relational/src/authority/mutation/aspect_versions.rs`
* `crates/worth-relational/src/authority/mutation/canonical_deltas/mod.rs`
* `crates/worth-relational/src/authority/mutation/canonical_deltas/patch_authority.rs`
* `crates/worth-relational/src/authority/mutation/execution.rs`
* `crates/worth-relational/src/authority/mutation/field_versions.rs`
* `crates/worth-relational/src/authority/mutation/intents/bulk_create_entities.rs`
* `crates/worth-relational/src/authority/mutation/intents/bulk_create_relations.rs`
* `crates/worth-relational/src/authority/mutation/intents/delete_entity.rs`
* `crates/worth-relational/src/authority/mutation/intents/dispatch.rs`
* `crates/worth-relational/src/authority/mutation/intents/mod.rs`
* `crates/worth-relational/src/authority/mutation/intents/replace_entity.rs`
* `crates/worth-relational/src/authority/mutation/record_changes.rs`
* `crates/worth-relational/src/authority/mutation/record_changes/entity_deletion.rs`
* `crates/worth-relational/src/authority/mutation/workspace.rs`
* `crates/worth-relational/src/branch/root_region.rs`
* `crates/worth-relational/src/durability/authority/checkpoint_complete/symbols.rs`
* `crates/worth-relational/src/history/data/required_output_positions.rs`
* `crates/worth-relational/src/runtime/state/subsystems/record_identity/allocation_batch.rs`
* `crates/worth-relational/src/runtime/state/subsystems/record_identity/mod.rs`
* `crates/worth-relational/src/runtime/state/subsystems/services/symbol_table/mutation_input.rs`
* `crates/worth-relational/src/storage/overlay/working_state/request_mutation.rs`
* `crates/worth-relational/src/storage/partition/sparse_adjacency_table.rs`
* `crates/worth-relational/src/storage/residency/cell.rs`
* `crates/worth-relational/src/storage/residency/checkpoint_capture/complete/tests.rs`
* `crates/worth-relational/src/storage/residency/kind_projection/full.rs`
* `crates/worth-relational/src/storage/residency/region_publication.rs`
* `crates/worth-relational/src/storage/residency/resident_history_tests.rs`
* `crates/worth-relational/src/symbols/data/symbol_text_lookup.rs`
* `crates/worth-relational/src/transactions/data/outcomes/conflict_class/record_allocation_denial.rs`

Created (renames):

* `crates/worth-relational/src/authority/mutation/intents/bulk_create_entities/prepared_row.rs`
* `crates/worth-relational/src/authority/mutation/intents/bulk_create_relations/relation_row.rs`

Deleted:

* `crates/worth-relational/src/authority/mutation/aspect_versions/original.rs`
* `crates/worth-relational/src/authority/mutation/canonical_deltas/revision_input.rs`
* `crates/worth-relational/src/authority/mutation/execution/original_adjacency.rs`
* `crates/worth-relational/src/authority/mutation/field_versions/original.rs`
* `crates/worth-relational/src/authority/mutation/intents/bulk_create_entities/original.rs`
* `crates/worth-relational/src/authority/mutation/intents/bulk_create_relations/original.rs`
* `crates/worth-relational/src/authority/mutation/workspace/original_revisions.rs`
* `crates/worth-relational/src/history/data/required_output_positions/reconstruction.rs`
* `crates/worth-relational/src/runtime/state/subsystems/record_identity/original_cursor.rs`
* `crates/worth-relational/src/runtime/state/subsystems/record_identity/original_cursor/tests.rs`
* `crates/worth-relational/src/runtime/state/subsystems/services/symbol_table/mutation_input/tests.rs`
* `crates/worth-relational/src/storage/residency/cell/resident_full.rs`
* `crates/worth-relational/src/storage/residency/cell/resident_full/tests.rs`
* `crates/worth-relational/src/storage/residency/region_publication/full.rs`
* `crates/worth-relational/src/storage/residency/region_publication/full/tests.rs`

Written outside the crate: this notes file only.

## Review fix (2026-11-10)

Independent review (claude-review-DC-B1.md, finding DC-1) found that
`controlled_full_region_matches_prior_projection_preimages_and_legacy_truth`
also held an ordinary oracle: the Full kind commitment equals an independent
MessagePack/SHA-256 preimage of the kind projection. That claim is kept in the
tree on the live producer `PreparedResidentRegion::initial`:
`storage/residency/region_publication/kind_commitment_preimage_tests.rs`
(`full_region_kind_commitment_matches_independent_projection_preimages`),
mounted from `storage/residency/region_publication.rs`. The C2 entry in
milestone-3.md is narrowed to the request-funded bills.
