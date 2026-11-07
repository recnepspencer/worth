# Full-construction census — CERTFIX4

Base: `71819d11a109af9ea8bbc23b51b54429a056701a`. Paths below are relative to
`crates/worth-relational/src/`. The committed inventory covers semantic full
construction, its callers, and explicit bulk/primitive false positives, rather
than treating a green growth court as evidence that an unmetered engine is safe.

## Method and boundary

Searched every Rust file in worth-relational, then traced the storage, overlay,
residency, partition, kind, adjacency, content, index and history matches through
branch publication, canonical routes, checkpoints, lineage and deferred index
readmission. Patterns were `previous: None` / `previous = None`, `None,` seeds,
`capture_seeded`, `occupied_slots`, `live_slots`, `.iter()`, `.values()`, `.keys()`,
`.into_iter()`, `rebuild`, `cold`, `from_scratch`, `from_sorted*` and `Sha256::`.
The raw grep is saved as `commitcost-evidence/certfix4/full-census-grep.txt` in the
external evidence directory; `census.py` reproduces this inventory and verifies
that every named function exists. Callers were searched by name and read in
owner scope, including the private recursive builders and alternate Head/full
entrances. Tests are classified separately; a generic serde/container operation
is not silently claimed to be a sealed model reconstruction engine.

D = requires `ReconstructionLane` by type directly. C = private whole-structure
helper reached only from the named lane-taking caller. V = shared index
orchestration accepts an optional **typed cold capability/work pair**; its cold
branch cannot run without that pair, and its actual cold engines are D.
B = real fixed trust-boundary operation, accepts the original request (or an
already typed admitted recovery input) and mints locally; no caller callback or
lane in its result. T = cfg(test) oracle, unavailable to ordinary compilation.
R = no lane: explicit requested bulk read/inspection/compaction, predecessor-required
delta maintenance, or generic input-sized container construction; not a semantic
full-construction engine.
Raw iteration is intentionally still available to its storage owner; the claim
is that an existing cold engine cannot be silently reached from publication,
not that Rust prevents someone from writing a new raw loop.

## Caller trace

Ordinary `PreparedResidentRegion::incremental` calls
`PartitionContentCommitment::maintain` with a mandatory commitment, old partition
and journal. It cannot supply `None` or reach `capture_seeded`. Records and
adjacency maintenance require their old arena/table, and record history update
requires its old arena. All cold content, record, adjacency, history and sorted
digest-tree engines instead require the lane. Cold content entry methods open
`lane.recover` on the **original** request, then carry `ReconstructionWork` down
to every row, history cell, membership and native value hash. Hash writers poll
the same request; each admitted value hash is a native operation, so digest-byte
entropy and MessagePack integer widths do not change the work quote. Cold record,
history and adjacency trees batch sorted entries into new unique owners instead
of quoting repeated persistent path copies. No per-commit fixed invoice is used.

Initial no-prior-region/root publication, region byte readmission and checkpoint
export/verification propagate that capability; no residency helper mints and
hands it out. Head frontier reconstruction uses the same controlled cold engine.
The content-reference directory has separate changed-record preparation and
lane-taking schema-transition/reconstruction functions. Its full live-record
visitor also requires the lane; schema transition checks the incoming schema's
identity against the retained predecessor before minting locally. Checkpoint
metadata/history and symbol-dictionary construction carry the export lane down
to the serializer-owned tree builders. Symbol snapshot restoration also requires
the lane; its snapshot reader remains an explicitly named bulk observation.
Resident-content verification now takes a request all the way from the public
inspection caller, rather than manufacturing a request inside the helper.

Index recovery/schema transition/checkpoint validation call the typed cold
projection/packet engines. Their field, ordering and join builders, join-leg
construction and transported index-image builders are sealed below those entry
points. A deferred index member mints only while it actually reads untrusted
bytes, and returns a state, never the capability. History ancestry, recovered
catalog/route trees, lineage resolution and branch-head indexes likewise carry
the lane from the real admitted-plan/tail/checkpoint recovery boundaries.

Nine exact registry additions name those newly covered **actual** boundaries:
explicit selected authored-history recovery; admitted runtime-plan recovery;
canonical tail replay; checkpoint snapshot capture; complete portable image
demand; lazy derived-index member readmission; native checkpoint decoding; and
native checkpoint export-directory verification; and changed content-schema authority. No ordinary path, wildcard,
generic callback trampoline or engine helper was registered. Each new entry was
reviewed for callback parameters and capability-bearing results. Existing G/J
seals remain in place; no threshold, whitelist exemption or court expectation
was changed.

Log compaction no longer cold-rebuilds its lookup tree: lookup values carry the
same resident-envelope handles, dropped keys are removed directly, and the log
uses a deque so front trimming does not move every survivor. Full log installation
still rebuilds only under its typed recovery capability. Retention consumes
carried generation bindings directly and no longer serializes a portable
checkpoint just to discover ids.

## Counts

321 named entries: D=175, C=11, V=1, B=26, T=4, R=104.

D/C/V cover the full-construction engines and typed propagation listed below. B entries identify fixed trust boundaries, T entries identify the source oracle exceptions, and R entries make the non-reconstruction scan/primitive boundary explicit. Counts are inventory totals, never performance assertions.

## Direct lane requirements

| File:function | Lane/test posture and caller |
| --- | --- |
| `branch/root_capture.rs:build_initial_regions` | D: required lane argument; caller family traced above |
| `branch/root_capture.rs:build_initial_regions_owned` | D: required lane argument; caller family traced above |
| `branch/root_region_access.rs:reconstruct_region_head` | D: required lane argument; caller family traced above |
| `branch/root_axes.rs:storage_root_from_authoritative_regions` | D: required lane argument; caller family traced above |
| `branch/root_region.rs:new` | D: required lane argument; caller family traced above |
| `branch/root_readmission.rs:reconstruct_storage_root_for_test` | D: required lane argument; caller family traced above |
| `durability/derived_index_artifacts.rs:restore_checkpoint_derived_index_artifacts` | D: required lane argument; caller family traced above |
| `content/reconstruction.rs:visit_records` | D: required lane argument; caller family traced above |
| `indexes/projected_field_values/relation_field_index_values.rs:build_relation_aspect_field_index` | D: required lane argument; caller family traced above |
| `indexes/projected_field_values/relation_field_index_values.rs:build_relation_aspect_field_index_checked` | D: required lane argument; caller family traced above |
| `indexes/projected_field_values/entity_field_index_values.rs:build_entity_aspect_field_index` | D: required lane argument; caller family traced above |
| `indexes/projected_field_values/entity_field_index_values.rs:build_entity_aspect_field_index_checked` | D: required lane argument; caller family traced above |
| `indexes/projected_field_values/relation_join.rs:build_relation_join_index_checked` | D: required lane argument; caller family traced above |
| `indexes/projected_field_values/relation_join.rs:build_relation_join_index` | D: required lane argument; caller family traced above |
| `indexes/projected_field_values/related_entity_ordering.rs:build_related_entity_ordering_index_checked` | D: required lane argument; caller family traced above |
| `indexes/projected_field_values/related_entity_ordering.rs:build_related_entity_ordering_index` | D: required lane argument; caller family traced above |
| `indexes/authority/build_execution.rs:execute_index_packets` | D: required lane argument; caller family traced above |
| `indexes/authority/build_execution.rs:execute_index_inputs` | D: required lane argument; caller family traced above |
| `indexes/authority/build_execution.rs:index_packet_result_checked` | D: required lane argument; caller family traced above |
| `indexes/authority/build_execution.rs:index_packet_result` | D: required lane argument; caller family traced above |
| `indexes/authority/basis_build.rs:reconstruct_basis_image` | D: required lane argument; caller family traced above |
| `indexes/authority/checkpoint_admission.rs:checkpoint_index_projections` | D: required lane argument; caller family traced above |
| `indexes/authority/checkpoint_admission.rs:verify_in_reconstruction` | D: required lane argument; caller family traced above |
| `indexes/authority/commit_build.rs:reconstruct_commit_image` | D: required lane argument; caller family traced above |
| `indexes/data/join_leg_state.rs:JoinLegState::cold` | D: required lane argument; caller family traced above |
| `indexes/data/join_leg_state.rs:consistent` | D: required lane argument; caller family traced above |
| `indexes/data/join_leg_state.rs:JoinLegsState::cold` | D: required lane argument; caller family traced above |
| `indexes/data/relation_join_index.rs:consistent_legs` | D: required lane argument; caller family traced above |
| `indexes/authority/maintenance/cold_projection.rs:project` | D: required lane argument; caller family traced above |
| `indexes/authority/maintenance/cold_projection.rs:collect_join` | D: required lane argument; caller family traced above |
| `indexes/authority/maintenance/recovery.rs:prepare_recovery` | D: required lane argument; caller family traced above |
| `indexes/authority/maintenance/changes.rs:cold` | D: required lane argument; caller family traced above |
| `indexes/authority/maintenance/maintained_selection.rs:cold` | D: required lane argument; caller family traced above |
| `symbols/data/string_interner.rs:restore_snapshot` | D: required lane argument; caller family traced above |
| `storage/residency/checkpoint_kind_images.rs:capture` | D: required lane argument; caller family traced above |
| `storage/residency/checkpoint_kind_images.rs:restore` | D: required lane argument; caller family traced above |
| `storage/residency/content_capture.rs:capture_resident_content` | D: required lane argument; caller family traced above |
| `storage/residency/kind_commitment.rs:digest` | D: required lane argument; caller family traced above |
| `storage/residency/kind_commitment.rs:digest_controlled` | D: required lane argument; caller family traced above |
| `storage/residency/kind_commitment.rs:verify` | D: required lane argument; caller family traced above |
| `storage/residency/region_initialization.rs:initial` | D: required lane argument; caller family traced above |
| `storage/residency/region_image.rs:direct_bases` | D: required lane argument; caller family traced above |
| `storage/residency/region_image.rs:encode` | D: required lane argument; caller family traced above |
| `storage/residency/region_image.rs:decode` | D: required lane argument; caller family traced above |
| `storage/residency/resident_history.rs:restore_rows` | D: required lane argument; caller family traced above |
| `storage/residency/kind_index.rs:rebuild` | D: required lane argument; caller family traced above |
| `storage/residency/region_load.rs:verify_buckets` | D: required lane argument; caller family traced above |
| `storage/residency/kind_incremental.rs:capture` | D: required lane argument; caller family traced above |
| `storage/residency/kind_incremental.rs:capture_controlled` | D: required lane argument; caller family traced above |
| `storage/residency/history_fold.rs:read` | D: required lane argument; caller family traced above |
| `storage/residency/history_fold.rs:fold_rows` | D: required lane argument; caller family traced above |
| `storage/residency/history_fold.rs:fold_row` | D: required lane argument; caller family traced above |
| `storage/residency/history_fold.rs:verify_directory` | D: required lane argument; caller family traced above |
| `storage/residency/kind_admission.rs:reconstruct` | D: required lane argument; caller family traced above |
| `storage/residency/kind_admission.rs:recover` | D: required lane argument; caller family traced above |
| `storage/residency/kind_admission.rs:normalize_value` | D: required lane argument; caller family traced above |
| `storage/residency/kind_history.rs:capture` | D: required lane argument; caller family traced above |
| `storage/residency/history_capture.rs:split_rows` | D: required lane argument; caller family traced above |
| `storage/residency/history_capture.rs:directory` | D: required lane argument; caller family traced above |
| `storage/residency/region_capture.rs:capture_region_members` | D: required lane argument; caller family traced above |
| `storage/residency/region_capture.rs:capture_kinds` | D: required lane argument; caller family traced above |
| `storage/residency/retention.rs:reconstruct_head` | D: required lane argument; caller family traced above |
| `storage/residency/history_presence.rs:from_cold` | D: required lane argument; caller family traced above |
| `storage/residency/required_symbols.rs:collect_head` | D: required lane argument; caller family traced above |
| `storage/residency/required_symbols.rs:collect` | D: required lane argument; caller family traced above |
| `storage/residency/required_symbols.rs:collect_counted` | D: required lane argument; caller family traced above |
| `storage/residency/required_symbols.rs:capture` | D: required lane argument; caller family traced above |
| `storage/residency/history_kind_cache.rs:verify` | D: required lane argument; caller family traced above |
| `storage/residency/kind_authority.rs:bind` | D: required lane argument; caller family traced above |
| `storage/residency/kind_projection.rs:capture` | D: required lane argument; caller family traced above |
| `storage/residency/kind_projection.rs:capture_controlled` | D: required lane argument; caller family traced above |
| `storage/residency/kind_projection.rs:rebuild_region_history_kind_buckets` | D: required lane argument; caller family traced above |
| `storage/substrate/digest_index.rs:cold` | D: required lane argument; caller family traced above |
| `storage/substrate/digest_index.rs:from_sorted_entries` | D: required lane argument; caller family traced above |
| `storage/substrate/digest_index.rs:build_sorted` | D: required lane argument; caller family traced above |
| `storage/partition/bitsets.rs:from_words` | D: required lane argument; caller family traced above |
| `storage/partition/bitsets.rs:from_sparse_words` | D: required lane argument; caller family traced above |
| `storage/partition/adjacency.rs:compressed_from_checkpoint` | D: required lane argument; caller family traced above |
| `storage/authority/publication.rs:projected_partitions_from_access` | D: required lane argument; caller family traced above |
| `storage/overlay/partition_symbols.rs:visit_retained_symbols` | D: required lane argument; caller family traced above |
| `storage/overlay/partition_content/record_history.rs:rebuild` | D: required lane argument; caller family traced above |
| `storage/overlay/partition_content/head_commitment.rs:history_frontiers` | D: required lane argument; caller family traced above |
| `storage/overlay/partition_content/head_commitment.rs:capture_head` | D: required lane argument; caller family traced above |
| `storage/overlay/partition_content/records.rs:history_frontiers` | D: required lane argument; caller family traced above |
| `storage/overlay/partition_content/reconstruction.rs:capture` | D: required lane argument; caller family traced above |
| `storage/overlay/partition_content/reconstruction.rs:capture_layered` | D: required lane argument; caller family traced above |
| `storage/overlay/partition_content/reconstruction.rs:capture_seeded` | D: required lane argument; caller family traced above |
| `storage/overlay/partition_content/records/reconstruction.rs:rebuild` | D: required lane argument; caller family traced above |
| `storage/overlay/partition_content/records/reconstruction.rs:from_history_frontiers` | D: required lane argument; caller family traced above |
| `storage/overlay/partition_content/adjacency/reconstruction.rs:rebuild` | D: required lane argument; caller family traced above |
| `storage/partition/adjacency/checkpoint.rs:checkpoint_kind_memberships` | D: required lane argument; caller family traced above |
| `storage/substrate/record_arena/arena/slot_directory.rs:restore` | D: required lane argument; caller family traced above |
| `storage/residency/cell/reconstruction_head.rs:reconstruct_head` | D: required lane argument; caller family traced above |
| `storage/residency/record_directory/maintenance.rs:cold` | D: required lane argument; caller family traced above |
| `storage/residency/record_directory/membership.rs:cold` | D: required lane argument; caller family traced above |
| `storage/residency/record_directory/portable.rs:portable` | D: required lane argument; caller family traced above |
| `storage/residency/record_directory/portable.rs:portable_controlled` | D: required lane argument; caller family traced above |
| `storage/residency/record_directory/portable.rs:from_portable` | D: required lane argument; caller family traced above |
| `storage/residency/record_directory/portable.rs:capture` | D: required lane argument; caller family traced above |
| `storage/residency/record_directory/portable.rs:verify_member` | D: required lane argument; caller family traced above |
| `storage/residency/kind_projection/reconstruction.rs:rebuild` | D: required lane argument; caller family traced above |
| `storage/residency/kind_projection/reconstruction.rs:rebuild_controlled` | D: required lane argument; caller family traced above |
| `storage/residency/kind_projection/reconstruction.rs:rebuild_head_history` | D: required lane argument; caller family traced above |
| `storage/residency/kind_projection/reconstruction.rs:install_projection` | D: required lane argument; caller family traced above |
| `storage/residency/kind_projection/reconstruction.rs:canonical_history` | D: required lane argument; caller family traced above |
| `storage/residency/kind_projection/reconstruction.rs:canonical_history_controlled` | D: required lane argument; caller family traced above |
| `storage/residency/kind_admission/maintained.rs:cold` | D: required lane argument; caller family traced above |
| `storage/residency/kind_projection/reconstruction/owned_history.rs:project_owned_history` | D: required lane argument; caller family traced above |
| `content/directory/reconstruction.rs:reconstruct` | D: required lane argument; caller family traced above |
| `content/directory/reconstruction.rs:prepare_schema_transition` | D: required lane argument; caller family traced above |
| `runtime/state/subsystems/history_canonical_routes.rs:rebuild_authored_write_generation` | D: required lane argument; caller family traced above |
| `runtime/state/subsystems/history_canonical_routes.rs:rebuild_recovered_canonical_routes` | D: required lane argument; caller family traced above |
| `runtime/state/subsystems/history_checkpoint_catalog.rs:rebuild_checkpoint_catalog` | D: required lane argument; caller family traced above |
| `runtime/state/subsystems/history_recovery.rs:restore_branch_cells` | D: required lane argument; caller family traced above |
| `runtime/state/subsystems/indexing.rs:retained_generations` | D: required lane argument; caller family traced above |
| `runtime/state/subsystems/lineage.rs:replace_events` | D: required lane argument; caller family traced above |
| `runtime/state/subsystems/durability.rs:set_log` | D: required lane argument; caller family traced above |
| `runtime/state/subsystems/history_branch_cells.rs:rebuild_branch_head_version_index` | D: required lane argument; caller family traced above |
| `runtime/state/subsystems/history_branch_cells.rs:admit_recovered_branch_cell` | D: required lane argument; caller family traced above |
| `runtime/state/subsystems/history_catalog_recovery.rs:install_recovered_canonical_inventory` | D: required lane argument; caller family traced above |
| `runtime/state/subsystems/history_catalog_recovery.rs:rebuild_catalog_with_live_roots` | D: required lane argument; caller family traced above |
| `runtime/state/subsystems/history_catalog_recovery.rs:rebuild_catalog_with_live_roots_and_descriptors` | D: required lane argument; caller family traced above |
| `runtime/state/subsystems/history_catalog_recovery.rs:readmit_replayed_root_descriptor` | D: required lane argument; caller family traced above |
| `runtime/state/subsystems/history_catalog_recovery.rs:record_recovered_commit` | D: required lane argument; caller family traced above |
| `runtime/state/subsystems/history_head_versions.rs:rebuild` | D: required lane argument; caller family traced above |
| `runtime/state/subsystems/durability/state.rs:set_log` | D: required lane argument; caller family traced above |
| `runtime/state/subsystems/durability/state.rs:rebuild_log_commit_index` | D: required lane argument; caller family traced above |
| `runtime/state/subsystems/lineage/state.rs:rebuild_derived_indexes` | D: required lane argument; caller family traced above |
| `runtime/state/subsystems/lineage/state.rs:replace_events` | D: required lane argument; caller family traced above |
| `runtime/state/subsystems/lineage/resolution_indexes.rs:rebuild` | D: required lane argument; caller family traced above |
| `runtime/state/subsystems/indexing/cold_checkpoint.rs:install_cold_checkpoint` | D: required lane argument; caller family traced above |
| `runtime/state/subsystems/indexing/basis_selection/checkpoint.rs:basis_checkpoint` | D: required lane argument; caller family traced above |
| `runtime/state/subsystems/indexing/basis_selection/checkpoint.rs:restore_basis_selection` | D: required lane argument; caller family traced above |
| `runtime/state/subsystems/indexing/basis_selection/empty_basis.rs:prepare_reconstruction_basis` | D: required lane argument; caller family traced above |
| `runtime/state/subsystems/canonical_publication_routes/authored_writes/rebuild.rs:rebuild_authored_generation` | D: required lane argument; caller family traced above |
| `durability/derived_index_artifacts/checkpoint_delta.rs:capture` | D: required lane argument; caller family traced above |
| `durability/derived_index_artifacts/checkpoint_delta.rs:readmit` | D: required lane argument; caller family traced above |
| `durability/checkpoints/images.rs:partition_to_image_with_contracts` | D: required lane argument; caller family traced above |
| `durability/checkpoints/images.rs:partition_from_image` | D: required lane argument; caller family traced above |
| `durability/data/checkpoint_manifest.rs:readmit_history_members` | D: required lane argument; caller family traced above |
| `durability/data/history_member_directory.rs:readmit` | D: required lane argument; caller family traced above |
| `durability/data/checkpoint_images.rs:branch_root_partition_image_digest` | D: required lane argument; caller family traced above |
| `durability/data/checkpoint_images.rs:branch_root_image_digest` | D: required lane argument; caller family traced above |
| `durability/data/native_checkpoint/roster_corruption/commitments.rs:rebind` | D: required lane argument; caller family traced above |
| `durability/authority/runtime_rebuild/recovered_lineage_artifacts.rs:reconcile_recovered_lineage_artifacts` | D: required lane argument; caller family traced above |
| `durability/authority/runtime_rebuild/history_parity.rs:apply_authoritative_commit_artifacts` | D: required lane argument; caller family traced above |
| `durability/authority/runtime_rebuild/checkpoint_restore.rs:prepare_symbols` | D: required lane argument; caller family traced above |
| `durability/authority/runtime_rebuild/checkpoint_restore.rs:prepare_lineage` | D: required lane argument; caller family traced above |
| `durability/authority/runtime_rebuild/checkpoint_restore.rs:prepare_indexes` | D: required lane argument; caller family traced above |
| `durability/authority/runtime_rebuild/checkpoint_restore.rs:finalize_restored_runtime` | D: required lane argument; caller family traced above |
| `durability/authority/runtime_rebuild/envelope_replay/effect.rs:replay_envelope_effect` | D: required lane argument; caller family traced above |
| `durability/authority/runtime_rebuild/envelope_replay/effect.rs:restore_authoritative_artifacts_when_required` | D: required lane argument; caller family traced above |
| `durability/authority/runtime_rebuild/envelope_replay/preparation.rs:prepare_replay_context` | D: required lane argument; caller family traced above |
| `durability/authority/runtime_rebuild/envelope_replay/preparation.rs:admit_carried_branch_checkpoint` | D: required lane argument; caller family traced above |
| `durability/authority/runtime_rebuild/checkpoint_restore/cold_directory.rs:restore` | D: required lane argument; caller family traced above |
| `durability/authority/runtime_rebuild/checkpoint_restore/partition_images.rs:restore_partition_mirror` | D: required lane argument; caller family traced above |
| `durability/authority/runtime_rebuild/checkpoint_restore/partition_images.rs:restore_unique_partition_images_with_schema` | D: required lane argument; caller family traced above |
| `durability/authority/runtime_rebuild/checkpoint_restore/partition_images.rs:restore_owned_partition_images_with_schema` | D: required lane argument; caller family traced above |
| `durability/authority/runtime_rebuild/checkpoint_restore/branch_root_images.rs:restore_branch_root_images` | D: required lane argument; caller family traced above |
| `durability/authority/runtime_rebuild/checkpoint_restore/history_readmission.rs:prepare_history` | D: required lane argument; caller family traced above |
| `durability/authority/runtime_rebuild/checkpoint_restore/root_schema_readmission.rs:readmit_root` | D: required lane argument; caller family traced above |
| `durability/authority/runtime_rebuild/checkpoint_restore/root_schema_readmission.rs:validate_root_image_integrity` | D: required lane argument; caller family traced above |
| `durability/log/native_checkpoint_codec/manifest_capture.rs:capture_metadata` | D: required lane argument; caller family traced above |
| `durability/log/native_checkpoint_codec/manifest_capture.rs:capture` | D: required lane argument; caller family traced above |
| `durability/log/native_checkpoint_codec/manifest_capture.rs:capture_expanded` | D: required lane argument; caller family traced above |
| `durability/log/native_checkpoint_codec/manifest_validation.rs:validate` | D: required lane argument; caller family traced above |
| `durability/log/native_checkpoint_codec/dictionary_capture.rs:capture` | D: required lane argument; caller family traced above |
| `durability/log/native_checkpoint_codec/manifest_capture/expanded_regions.rs:restore_images` | D: required lane argument; caller family traced above |
| `durability/log/native_checkpoint_codec/manifest_capture/expanded_regions.rs:capture_partitions` | D: required lane argument; caller family traced above |
| `durability/checkpoints/images/bitset_image.rs:restore_bitset` | D: required lane argument; caller family traced above |
| `durability/checkpoints/images/arena_images.rs:arena_to_image` | D: required lane argument; caller family traced above |
| `durability/checkpoints/images/arena_images.rs:arena_from_image` | D: required lane argument; caller family traced above |
| `visibility/pins/pin_authority/branch_head_residency.rs:rebuild_branch_head_visibility_residency` | D: required lane argument; caller family traced above |
| `branch/root_regions/publication.rs:initial` | D: required lane argument; caller family traced above |
| `branch/root_readmission/required_text.rs:validate` | D: required lane argument; caller family traced above |

## Private helpers under typed callers

| File:function | Lane/test posture and caller |
| --- | --- |
| `runtime/state/subsystems/history_recovery.rs:recorded_postures` | C: HistorySubsystem::restore_branch_cells(lane, ..) |
| `storage/residency/kind_projection/reconstruction/claim_copy.rs:claims_match` | C: kind_projection::rebuild_controlled |
| `storage/residency/history_kind_cache.rs:verify_direction` | C: history_kind_cache::verify(lane, image) |
| `storage/residency/history_presence.rs:count` | C: HistoryPresence::from_cold(lane, partition) |
| `storage/overlay/partition_symbols.rs:visit_columns` | C: PartitionState::visit_retained_symbols(lane, visitor) |
| `storage/residency/record_directory/portable.rs:keys` | C: RegionRecordDirectory::from_portable(lane, ..) |
| `indexes/authority/maintenance/cold_projection.rs:seal` | C: cold_projection::project(lane, ..) |
| `indexes/projected_field_values/relation_join.rs:collect_join_relations` | C: build_relation_join_index(lane, ..) |
| `indexes/projected_field_values/relation_join.rs:collect_join_relations_checked` | C: build_relation_join_index_checked(lane, ..) |
| `durability/data/history_member_directory.rs:verify_reachable_acyclic` | C: RelationalHistoryMemberDirectory::readmit(lane, ..) |
| `durability/derived_index_artifacts/checkpoint_delta.rs:digest` | C: DerivedIndexCheckpointArtifacts::{capture,readmit}(lane, ..) |

## Capability-bearing cold variant

| File:function | Lane/test posture and caller |
| --- | --- |
| `indexes/authority/maintenance/pipeline_preparation.rs:prepare_index_publication` | V: required lane argument; caller family traced above |

## Fixed trust-boundary operations

| File:function | Lane/test posture and caller |
| --- | --- |
| `content/schema_transition.rs:prepare_directory` | B: changed schema identity checked against predecessor root; unchanged identity can only use delta maintenance |
| `storage/residency/region_load.rs:load_head` | B: cold untrusted region bytes; original request -> content/kind admission |
| `storage/residency/region_load.rs:load_full` | B: cold history member fold and independent content check |
| `storage/residency/resident_history.rs:load_full` | B: readmit authenticated Head plus cold history |
| `storage/residency/checkpoint_capture.rs:capture_checkpoint` | B: checkpoint export, only absent-carried-member/consolidation branch |
| `storage/residency/checkpoint_capture.rs:verify_checkpoint_directory` | B: fixed checkpoint export consistency verification |
| `branch/root_readmission/region_binding.rs:readmit` | B: checkpoint root image readmission |
| `branch/root_readmission/region_binding.rs:readmit_region_directory` | B: checkpoint leaf directory readmission |
| `branch/root_readmission/region_binding.rs:verify_resident_storage_content` | B: fixed resident-byte verification, original request required |
| `branch/root_capture_preparation.rs:prepare_regions` | B: no committed predecessor only; ordinary branch calls incremental engine |
| `branch/root_capture.rs:build_incremental_regions` | B: missing prior region only; existing regions use typed predecessor |
| `indexes/authority/recovery_build.rs:build_for_basis` | B: fixed public exact-basis recovery; no callback/lane result |
| `indexes/authority/recovery_build.rs:build_for_commit` | B: fixed public exact-commit recovery; no callback/lane result |
| `indexes/authority/maintenance/schema_transition.rs:prepare_schema_transition` | B: explicit changed schema authority |
| `indexes/authority/maintenance/recovery.rs:reconstruct_for_basis` | B: fixed public index recovery |
| `indexes/authority/checkpoint_admission.rs:verify_checkpoint_index_bases` | B: full checkpoint-derived index equality verification |
| `durability/authority/runtime_rebuild.rs:rebuild_runtime_from_plan` | B: admitted cold recovery plan; capability remains local |
| `durability/authority/runtime_rebuild/envelope_replay.rs:replay_readmitted_envelope` | B: readmitted canonical tail replay; capability remains local |
| `history/access/relevant_writes/maintenance.rs:rebuild_authored_write_generation` | B: explicit owner-exact selected ancestry rebuild; original request charged by history stage |
| `runtime/state/subsystems/indexing/cold_checkpoint.rs:readmit` | B: actual derived-index member byte readmission; no callback |
| `durability/authority/checkpoint_capture.rs:capture` | B: actual checkpoint snapshot/export |
| `durability/authority/checkpoint_memory_demand.rs:demand_memory_image` | B: explicit complete portable checkpoint image export |
| `durability/authority/checkpoint_native_capture.rs:build_native_capture` | B: complete native checkpoint export-directory verification |
| `durability/log/native_checkpoint_codec/checkpoint_decoding.rs:decode_checkpoint_from_source` | B: actual native checkpoint byte decode/manifest verification |
| `durability/authority/checkpoint_image.rs:build_checkpoint_image` | B: explicit full portable image export |
| `durability/log/native_checkpoint_codec/expanded_checkpoint.rs:expand` | B: explicit portable checkpoint expansion |

## Test-only reconstruction oracles

| File:function | Lane/test posture and caller |
| --- | --- |
| `storage/residency/kind_commitment.rs:capture` | T: required lane argument; caller family traced above |
| `storage/overlay/partition_content.rs:authoritative_content_digest` | T: cfg(test); invokes same lane-taking cold engine |
| `storage/overlay/partition_content/reconstruction.rs:capture_oracle` | T: cfg(test); invokes same lane-taking cold engine |
| `storage/residency/record_directory/portable.rs:portable_oracle` | T: cfg(test); invokes same lane-taking cold engine |

## Explicit bulk operations, delta entrances and generic primitives

| File:function | Lane/test posture and caller |
| --- | --- |
| `storage/overlay/partition_content/record_history.rs:update` | R: delta entrance with mandatory predecessor and journal/changed keys; a new record or changed authenticated history basis may seed its changed intrinsic granule, but no unchanged partition/history fallback exists |
| `symbols/data/string_interner.rs:snapshot` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `content/reconstruction.rs:schema_has_references` | R: schema/retained-contract inspection for full content readmission; installation declarations, not model records/history |
| `content/directory/reconstruction.rs:retained_records` | R: explicit requested retained-content export/read, charged by original content control; does not build the directory |
| `content/directory/reconstruction.rs:visit_frames` | R: explicit requested retained-content export/read, charged by original content control; does not build the directory |
| `validation/engine/evaluator/relation_cardinality/minimum_current_index.rs:build` | R: whole invariant validation scratch scan on its original context; ordinary publication uses merged_plan delta counts; no maintained field index is reconstructed |
| `storage/authority/retention/history_trim.rs:trim_full_live_history` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `visibility/materialization/read_records/reader/paid_current_record_walk.rs:paid_current_entities` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `visibility/materialization/read_records/reader/paid_current_record_walk.rs:paid_current_relations` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `storage/substrate/shared_map/traits.rs:from_iter` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `storage/substrate/shared_column/traits.rs:from_iter` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `storage/residency/views/consequence_materialization.rs:materialize_consequence_partition` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `storage/overlay/working_state/consequence_materialization.rs:materialize_consequence_records` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `history/data/codec_retirement_preservation.rs:validate_metered` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `history/data/codec_retirement_closure.rs:derive_metered` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `history/data/codec_retirement_closure/relations.rs:derive_closed_relations` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `validation/engine/state_view/logical_slots.rs:try_entity_slots` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `validation/engine/state_view/logical_slots.rs:try_relation_slots` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `validation/engine/evaluator/record_surface_rules/live_record_sidecars.rs:evaluate_live_record_sidecar` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `validation/engine/evaluator/relation_cardinality/minimum_visible_counts.rs:collect_selected_root_relation_counts` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `validation/engine/evaluator/relation_cardinality/minimum_visible_counts.rs:collect_selected_root_candidate_entities` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `aggregate_identity/pinned/lookup.rs:find` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `visibility/materialization/read_records/read_work/record_memory.rs:occupied_slots` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `visibility/materialization/read_records/visibility/slot_sets.rs:visible_relation_slots_in_partition_from_state` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `visibility/materialization/read_records/visibility/slot_sets.rs:visible_slots_in_partition_from_state` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `visibility/retention/retention_authority/partition_pass.rs:inspect_partition_retention` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `visibility/retention/retention_authority/partition_pass.rs:run_partition_retention_pass` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `visibility/retention/retention_authority/maintenance.rs:requested_slots` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `visibility/materialization/read_records/projection/streamed_basis_reads/partition_stream.rs:entity` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `visibility/materialization/read_records/projection/streamed_basis_reads/partition_stream.rs:relation` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `visibility/materialization/read_records/reader/truth_kind_scan.rs:scan_entity_kind_in_partition` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `visibility/materialization/read_records/reader/truth_relation_kind_scan.rs:scan_relation_kind_in_partition` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `visibility/materialization/read_records/reader/paid_record_walk.rs:paid_entities_in_partition` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `visibility/materialization/read_records/reader/paid_record_walk.rs:paid_relations_in_partition` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `storage/access.rs:record_slots_in` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `storage/residency/head_arena.rs:try_visit_occupied` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `storage/residency/head_arena.rs:try_occupied_slots` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `storage/residency/head_arena.rs:occupied_slots_iter` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `storage/residency/head_arena.rs:live_slots` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `storage/residency/head_arena.rs:try_live_slots` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `storage/substrate/record_arena/arena/mod.rs:try_occupied_slots` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `storage/substrate/record_arena/arena/mod.rs:occupied_slots` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `storage/substrate/record_arena/arena/mod.rs:occupied_slots_iter` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `storage/substrate/record_arena/arena/slot_directory.rs:try_occupied_entries` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `storage/substrate/record_arena/arena/slot_directory.rs:try_occupied_slots` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `storage/substrate/record_arena/arena/slot_directory.rs:occupied_slots` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `storage/substrate/record_arena/arena/slot_directory.rs:occupied_slots_iter` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `storage/partition/chunks.rs:summarize_entity_chunks` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `storage/partition/chunks.rs:summarize_relation_chunks` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `storage/partition/chunks.rs:summarize_arena_chunks` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `storage/partition/storage_stats.rs:partition_storage_stats` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `storage/partition/storage_stats.rs:storage_stats` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `storage/overlay/partition.rs:allocation_inventory` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `storage/overlay/partition_content.rs:visit_allocations` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `storage/overlay/partition_content.rs:maintain` | R: delta entrance with mandatory predecessor and journal/changed keys; a new record or changed authenticated history basis may seed its changed intrinsic granule, but no unchanged partition/history fallback exists |
| `storage/overlay/partition_content/records.rs:visit_allocations` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `storage/overlay/partition_content/records.rs:update` | R: delta entrance with mandatory predecessor and journal/changed keys; a new record or changed authenticated history basis may seed its changed intrinsic granule, but no unchanged partition/history fallback exists |
| `storage/overlay/partition_content/adjacency.rs:visit_allocations` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `storage/overlay/partition_content/adjacency.rs:update` | R: delta entrance with mandatory predecessor and journal/changed keys; a new record or changed authenticated history basis may seed its changed intrinsic granule, but no unchanged partition/history fallback exists |
| `storage/substrate/digest_index.rs:visit_allocations` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `storage/residency/kind_index.rs:visit_allocations` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `storage/residency/kind_incremental.rs:visit_allocations` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `storage/residency/kind_history.rs:visit_allocations` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `storage/substrate/shared_column.rs:into_vec` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `storage/overlay/partition_allocation_walk.rs:reclaimable_unique_authoritative_bytes` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `storage/overlay/partition_allocation_walk.rs:visit_authoritative_allocations` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `storage/overlay/partition_allocation_walk.rs:visit_diagnostic_allocations` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `storage/overlay/partition_allocation_walk.rs:visit_retention_allocations` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `storage/overlay/partition_allocation_walk.rs:visit_cache_allocations` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `storage/substrate/record_arena/arena/allocation.rs:allocation_inventory` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `storage/substrate/record_arena/arena/allocation_walk.rs:visit_authoritative_allocations` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `storage/substrate/record_arena/arena/allocation_walk.rs:visit_diagnostic_allocations` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `storage/substrate/record_arena/arena/allocation_walk.rs:visit_retention_allocations` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `storage/partition/sparse_adjacency_table.rs:visit_cache_allocations` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `storage/partition/sparse_adjacency_table.rs:from_entries` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `storage/partition/adjacency/kind_delta.rs:visit_changed_kind_memberships` | R: generic one-adjacency-set diff; absent old set represents a newly created changed slot, not a partition rebuild; kind traversal polls original admission |
| `storage/partition/adjacency/allocation.rs:visit_authoritative_allocations` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `storage/partition/adjacency/allocation.rs:visit_cache_allocations` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `storage/partition/adjacency.rs:structural_revisions` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `storage/partition/adjacency.rs:visit_kind_memberships` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `storage/partition/adjacency.rs:ids` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `storage/partition/adjacency.rs:extend_into` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `storage/partition/adjacency.rs:changed_current_memberships` | R: generic one-adjacency-set diff; absent old set represents a newly created changed slot, not a partition rebuild; kind traversal polls original admission |
| `storage/partition/bitsets.rs:sparse_words` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `storage/partition/bitsets.rs:set_slots` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `storage/partition/bitsets.rs:iter_set_slots` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `storage/partition/bitsets.rs:try_set_slots` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `storage/partition/bitsets.rs:count_ones_in_range` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `storage/partition/bitsets.rs:visit_allocations` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `storage/partition/bitsets.rs:changed_words_since` | R: generic word diff; cold semantic builders do not call this function; ordinary content maintenance always supplies the old bitset |
| `storage/substrate/shared_map.rs:from_sorted_unique` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `storage/substrate/shared_map/node.rs:from_sorted_unique` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `storage/substrate/shared_column/bulk_bootstrap.rs:from_dense_values` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `indexes/projected_field_values/index_projection_source/native_fields.rs:try_for_each_entity_field` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `indexes/projected_field_values/index_projection_source/native_fields.rs:try_for_each_relation_field` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `indexes/projected_field_values/index_projection_source.rs:for_each_entity` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `indexes/projected_field_values/index_projection_source.rs:for_each_relation` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `storage/residency/cell/member_export.rs:checkpoint_export_sources` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `durability/data/history_member_directory.rs:members` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `durability/data/history_member_directory.rs:declarations_for` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `durability/data/history_member_directory.rs:fold_span` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `runtime/state/subsystems/durability/state.rs:retain_log_after` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `runtime/state/subsystems/durability/state.rs:trim_log_front` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |
| `runtime/state/subsystems/indexing/basis_selection/checkpoint.rs:visit_retained_bindings` | R: explicit bulk read/inspection/compaction or generic supplied-input construction; not a semantic cold engine |

## Deliberate non-matches

Empty `Default`/empty-root creation is constant work; full nonempty root construction
is a separate lane-taking function. Digest/commitment getters hash fixed-size
carried child roots, not all records. `DigestIndex::from_append_frontier` and
frontier quoting walk only a bounded right spine (at most native key width),
not history length. Journal loops, changed-key/changed-word iterators and
`KindRegionDirectory::replaced` visit the supplied change or newly imported
record's intrinsic granule, not all model slots. The record-history delta entrance
requires the old arena: an unchanged prefix reuses its carried tree and visits
changed metadata cells only. Replacing an authenticated history basis reseeds
that explicitly replaced record's intrinsic history, as the mixed-prefix equality
courts require; it is not an unchanged-history cold fallback. Single-record native encoding
and field lookup are the changed/read record's granule. Canonical envelope
encoding hashes one commit's artifact, not the retained timeline. Frame hashes
consume explicitly supplied bytes; semantic model serialization constructors
are separately sealed above. Schema plans/registration and installed routing
are installation work, not a per-commit full-record cold fallback.

Allocation inventory/walks and complete chunk/statistics reads remain explicitly
named bulk observation operations. They are listed R, not claimed to be delta
maintenance. Ordinary publication uses `allocation_inventory_since`, persistent
path updates, cached populations and journal-selected content. The R validation/codec-transition and retention scans are explicit declared
read/transition/pass workloads, not cold commitment fallbacks. The R compaction
operation is explicit durable retention, not content/index reconstruction.

Source-private oracle helpers use cfg(test) `ReconstructionLane::inspect_oracle`
to invoke the exact cold engine without constructing a foreign execution runtime
inside pure source unit tests. The production lane constructor/registry cannot
reach that helper. Runtime/certification courts continue to pass their original
Execution request; no budget, scale, assertion or expected identity was reduced.
