# Landing 2a cold test claim review

Read-only independent review of the 80-file closure in `bc-cold-module-closure.json`: all 86 named tests have an individual entry below. No builds, source edits, or assertion removals. This report is review guidance, not authorization to retire evidence.

Class **1** moves public semantic/provider behavior to an external public court preserving assertions. A private getter or private setup helper alone does not make a claim C2. Class **2** preserves private resident state, exact dictionary, pin actions, allocation-point or accepted-round accounting in C2, with the supplied `restore from 2a51552fc5:path` pointer. Class **3** is a genuine exact observation gap: preserve the test while designing a minimal typed owner observation, or split its public and private claims explicitly. A class-3 row does not approve a new observer, shim, sealed door, constructor reach or caller-list change.

Reading boundary: AGENTS.md; testing laws and QA review guide; qa-tests skill; court destination and authority passages of `plans/WORTH-relational/iteration-cutover.md`; C2 moved-court entry of `plans/database-foundation/milestone-3.md`; actual named test assertions, their action adapters and causally relevant fixtures; facade exports and selected production operations. This is source inspection, not compilation evidence. Lines refer to the inspected worktree; all C2 restore pointers use the coordinator-specified commit.

## Existing public paths

The reusable external fixture is `crates/worth-testkit-relational/src/cold_recovery.rs::recovered_with_history`: actual native members, an externally owned source/medium, and recovery through production APIs. `durable_medium/immutable_members/mod.rs` already implements the public durable-source trait with provider-owned member/commit logs and failure injection. These provider observations are public boundary evidence; they do not justify a new runtime observer and do not force C2. The same is true for provider-owned custody reference counts in the index placement test. Public immutable read/metadata pins are distinct from private partition physical-pin counts.

An existing external analogue is `crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/durable_publication/cold_envelope_custody.rs`, proving public canonical-envelope identity/custody and retry through `runtime.replay().canonical_commit_envelope`. That API replaces private `history().demand_commit_envelope` when the assertion concerns the public envelope rather than resident catalog internals.

Public API definitions are available through `crates/worth-relational/src/facade.rs`, `facade/durability.rs`, `facade/branch.rs`, `facade/indexes.rs`, and `facade/inspection.rs`; runtime convenience entry points are in `runtime/guided.rs`. `publication().diagnostics_snapshot()` (`publication/access/mod.rs:110`) preserves equality against the same diagnostic type without private field reach. `patch_position_reservation_counters` (`mvcc/publication/port.rs:159`) is already public semantic observation; unchanged public reservation counts alone do not force C2.

Public fixture construction must retain emitted bytes, exact locators, authentic record/commit results and original source custody. Court-local wire reading or existing public checkpoint inspection may identify emitted members. Do not export private `RelationalCheckpointManifest`, checkpoint selection tables, `member_frame::decode`, roots, interner or residency guards to make fixtures compile. Use proper owner-issued branch bases/ports and owner-bound merge requests; cfg(test) raw-request overloads are not the external contract.

## Genuine gaps and dormant traversals

`lineage_access()` is `pub(crate)` at `src/lineage/access/mod.rs:19`; `LineageAccess` is not facade-exported. Exact node observations therefore need a typed node-for-record or node-for-admitted-lineage-selection operation on the existing lineage/runtime owner. It must carry an owner-issued branch/observation binding and return the semantic `LineageNode`, not root/arena/cell access, arbitrary authority or a test shim. An exact node lease/custody observation may need an owned semantic result; public ID-only visibility is not identical.

The public `record_history_at_observation` (`src/change_source/lineage_at_observation.rs:19`) first demands Full region and resolves selected-root ancestry. `visible_entities_for_lineages_at_observation` returns entity IDs and also demands regions. These are useful existing public behaviors, but are not automatically equivalent to raw branch-scoped `resolve_historical_lineage` that reads only event pages. Class-3 event cases need a typed branch-bound semantic lineage-history observation preserving the existing request's branch-scoped meaning, exact resolution/digest and event-only provider reads. Do not claim the current observation API preserves that no-region/no-symbol/no-node assertion.

Graph and divergence modules are explicitly `#[cfg(test)]` in `src/lineage/access/mod.rs`. They materialize/sort complete node/event vectors and sets in `graph.rs` and `divergence.rs` using `Vec`/`BTreeSet` collection, without explicit retained-result backing reservations for those collections. Promoting them is a production capability/funding design, not a minimal observation needed by this move. Preserve their exact graph-event/shared-lineage subcases in C2 until that owner work is specified. The aspect-history access wrappers, by contrast, directly forward to existing public history APIs and can use those equivalents without new doors.

`cold_merge_history_matrix.rs::HistoryPlan` uses `plan_history_scope_for_test` and inspects private source/target delta-envelope lists. Public `MergePlanningArtifactCore` exposes semantic ancestry/classification/decision artifacts, not those lists. Keep that internal delta evidence C2 while moving the already-public preparation and transaction-parent subcases. The separate intermediate-overlap test chooses only TransactionParents and is class1.

## Shared helper preservation

Helper assertions are not permission to delete evidence. Public fixtures must split the private posture checks into C2, retaining these exact restore locations:

- `restore from 2a51552fc5:crates/worth-relational/src/tests/durability/medium/cold_lineage_event_fixture.rs`: final `runtime.lineage.event_count()==0` and `node_count()==0`.
- `restore from 2a51552fc5:crates/worth-relational/src/tests/durability/medium/cold_lineage_fixture.rs`: final `runtime.lineage.node_count()==0`.
- `restore from 2a51552fc5:crates/worth-relational/src/tests/durability/medium/cold_candidates.rs`: recovered helper's `probe_full_partition(REMOTE)==Cold`, plus private arena slot baseline used by retry tests.
- `restore from 2a51552fc5:crates/worth-relational/src/tests/durability/medium/cold_merges.rs`: `recovered_shape(symbols=true)` asserts exact interner absence.
- `restore from 2a51552fc5:crates/worth-relational/src/tests/durability/medium/cold_historical_index_matrix.rs`: `assert_cold` exact historical-symbol absence and `canonical_envelope(...).is_none`.
- `restore from 2a51552fc5:crates/worth-relational/src/tests/durability/medium/cold_projection_adjacency_matrix.rs:81`: `Door::BoundedIncoming` asserts exact interner resolution even when its caller's body appears to use only source logs. It affects the selected matrix cases; the zero-frontier test does not invoke that helper.

## Per-test map

Counts: class1 = 39; class2 = 41; class3 = 6. The class1 released-live-history row also retains a narrow C2 resident-count subclaim. There are 27 support-only files in the 80-file closure; they are not miscounted as tests.

### 1. `checkpoint_restart_cycles_preserve_full_reads_and_retention` ? class2

Source: `crates/worth-relational/src/tests/durability/medium/checkpoint_restart_bound.rs:27`.

Private history capture limits and directory.fold_span; retain layer/input bounds and consolidation count.

`restore from 2a51552fc5:crates/worth-relational/src/tests/durability/medium/checkpoint_restart_bound.rs`.

Exact private assertions requiring C2:

- crates/worth-relational/src/tests/durability/medium/checkpoint_restart_bound.rs:99: `assert!(layers < HISTORY_CAPTURE_MAX_LAYERS + 1)`
- crates/worth-relational/src/tests/durability/medium/checkpoint_restart_bound.rs:100: `assert!(input < HISTORY_CAPTURE_INPUT_TARGET)`
- crates/worth-relational/src/tests/durability/medium/checkpoint_restart_bound.rs:129: `assert!( consolidations >= 2, "real runtime capture must repeatedly consolidate" )`

### 2. `accepted_pin_projections_survive_later_proven_member_damage` ? class1

Source: `crates/worth-relational/src/tests/durability/medium/cold_accepted_pins.rs:6`.

Public read_truth().pin_read_capability/project_snapshot/pin_resident_entity_metadata and accepted projection reads; exact metadata, fresh denial, no provider reacquisition and no effects. Public semantic pins, not physical accounting.

### 3. `paired_adjacency_propagates_current_head_storage_failure_without_history_fallback` ? class1

Source: `crates/worth-relational/src/tests/durability/medium/cold_adjacency_support.rs:6`.

Public VisibilityProjectionView::compare_adjacency_support; preserve exact Region subject/cause and provider no-history log.

### 4. `residency_failure_matrix_aspect_history_doors_cover_every_subject_and_class` ? class1

Source: `crates/worth-relational/src/tests/durability/medium/cold_aspect_history_matrix/exhaustive.rs:4`.

Public history().entity_aspect_history/relation_aspect_history/lineage_entity_aspect_history and traced variants; preserve all six doors, failure classes/retry/no-effects. Use publication().diagnostics_snapshot for identical diagnostic equality.

### 5. `candidate_invariant_discovery_refuses_stickily_without_publishing_round_effects` ? class2

Source: `crates/worth-relational/src/tests/durability/medium/cold_candidates.rs:149`.

Private performance_access counters and services.symbols.snapshot equality, alongside typed public invariant-preparation refusal.

`restore from 2a51552fc5:crates/worth-relational/src/tests/durability/medium/cold_candidates.rs`.

Exact private assertions requiring C2:

- crates/worth-relational/src/tests/durability/medium/cold_candidates.rs:189: `assert_eq!( after.preparation_packet_count, before.preparation_packet_count )`
- crates/worth-relational/src/tests/durability/medium/cold_candidates.rs:193: `assert_eq!( after.invariant_entity_slot_scans, before.invariant_entity_slot_scans )`
- crates/worth-relational/src/tests/durability/medium/cold_candidates.rs:198: `assert_eq!(runtime.services.symbols.snapshot(), dictionary)`

### 6. `a_region_retry_spends_the_applied_draft_identity_and_settles_one_round` ? class2

Source: `crates/worth-relational/src/tests/durability/medium/cold_candidates.rs:205`.

Private entity_arena.slot_count floor, abandoned identity slot inequalities/headroom, and cold/warm packet/scan deltas.

`restore from 2a51552fc5:crates/worth-relational/src/tests/durability/medium/cold_candidates.rs`.

Exact private assertions requiring C2:

- crates/worth-relational/src/tests/durability/medium/cold_candidates.rs:225: `assert!( id.slot_index() > floor, "the discarded applied candidate cannot reuse an identity" )`
- crates/worth-relational/src/tests/durability/medium/cold_candidates.rs:229: `assert!( id.slot_index() < floor + 16, "retry growth has bounded headroom" )`
- crates/worth-relational/src/tests/durability/medium/cold_candidates.rs:234: `assert!((warm_floor..warm_floor + 16).contains(&warm_slot))`
- crates/worth-relational/src/tests/durability/medium/cold_candidates.rs:235: `assert!( id.slot_index() > warm_slot, "cold discovery abandons an extra applied identity" )`
- crates/worth-relational/src/tests/durability/medium/cold_candidates.rs:241: `assert!(after_cold.preparation_packet_count > before_cold.preparation_packet_count)`
- crates/worth-relational/src/tests/durability/medium/cold_candidates.rs:242: `assert_eq!( after_cold.preparation_packet_count - before_cold.preparation_packet_count, after_warm.preparation_packet_count - before_warm.preparation_packet_count )`
- crates/worth-relational/src/tests/durability/medium/cold_candidates.rs:246: `assert_eq!( after_cold.invariant_entity_slot_scans - before_cold.invariant_entity_slot_scans, after_warm.invariant_entity_slot_scans - before_warm.invariant_entity_slot_scans )`

### 7. `residency_failure_matrix_declared_current_doors_preserve_every_failure_class` ? class1

Source: `crates/worth-relational/src/tests/durability/medium/cold_current_failure_matrix/exhaustive.rs:4`.

CurrentDoor inventory calls public projected reads/exact snapshot reads, transaction read/validate/prepare/commit/delete; preserve failure class/subject, callback suppression, retry and effect assertions. Use publication().diagnostics_snapshot.

### 8. `residency_failure_matrix_current_doors_never_consult_region_history` ? class1

Source: `crates/worth-relational/src/tests/durability/medium/cold_current_matrix/exhaustive.rs:4`.

Same public CurrentDoor inventory; external provider logs prove no unavailable history requested, returned records and frontier/offers prove effects.

### 9. `head_write_capture_and_full_promotion_preserve_current_and_retained_reference_reads` ? class1

Source: `crates/worth-relational/src/tests/durability/medium/cold_current_matrix.rs:37`.

Public native_checkpoint_with_policy and read_truth().visible_entity_at_version/read_observation; preserve cold capture no-load, typed Full failure, warm-reference current/retained equality.

### 10. `settled_live_nonhead_envelopes_release_only_when_a_durable_copy_exists` ? class2

Source: `crates/worth-relational/src/tests/durability/medium/cold_envelope_lifetime.rs:5`.

Private history retired-root reclamation and retired_branch_root_count plus exact body Weak residency distinctions durable vs memory-only.

`restore from 2a51552fc5:crates/worth-relational/src/tests/durability/medium/cold_envelope_lifetime.rs`.

Exact private assertions requiring C2:

- crates/worth-relational/src/tests/durability/medium/cold_envelope_lifetime.rs:29: `assert!( runtime.history.retired_branch_root_count() < 4, "a single retired version fits reclamation headroom" )`
- crates/worth-relational/src/tests/durability/medium/cold_envelope_lifetime.rs:34: `assert!( weak.upgrade().is_none(), "a settled nonhead without a reader must release its journal-backed body" )`
- crates/worth-relational/src/tests/durability/medium/cold_envelope_lifetime.rs:39: `assert!( weak.upgrade().is_some(), "memory-only history has no other copy" )`

### 11. `cold_source_is_bound_before_a_durable_runtime_publishes_its_first_commit` ? class3 (revised)

Source: `crates/worth-relational/src/tests/durability/medium/cold_envelope_lifetime.rs:60`.

Revised classification: semantic source-origin/attachment ordering, not C2. Before first publication the attached source must already exist; it then reads the first exact artifact and matches position, commit identity and entity target. The memory-only variant asserts source absence. Existing public analogue: retain the real medium/source outside the runtime, observe RelationalDurableMedium::cold_source before publish, make the first commit nonhead, then demand replay().canonical_commit_envelope or durability().canonical_commit_artifacts and require an actual read from that same provider, matching all original artifact fields. A read from a still-resident envelope is insufficient. This behavioral public variant may be class1 if it preserves the accepted source-origin claim; literal immediate private Option-presence/absence lacks an existing public observation. Minimal class3 seam, if required, is typed read-only attached-source origin/posture on the durability owner (no source getter, raw trait-object export, shim or new sealed door). No resident-state, pin or allocation claim justifies C2 here.

The old classification treated a private getter as C2 evidence. That was too broad. Original exact artifact assertions at lines72,80,88 and93 remain public semantic requirements; line97's memory-only source absence is an origin/posture requirement, not a cost/residency assertion.

### 12. `released_live_history_remains_in_external_capture_and_retained_recovery` ? class1 plus a narrow C2 subclaim (revised)

Source: `crates/worth-relational/src/tests/durability/medium/cold_envelope_lifetime.rs:104`.

Revised classification: mixed public recovery with a narrow C2 resident subclaim. Preserve BOTH actual recovery paths: durability().recovery_plan -> durability_recovery().recover, then external native capture -> recovery_plan_from_source -> recover. Enumerate authentic commit IDs from captured create/update results or public history/export; for every ID compare public replay().canonical_commit_envelope(...).commit.commit_id after both recoveries. Preserve provider commit_reads.len()<32; this is externally observed I/O, not private resident accounting. Only resident_envelope count<4 at line125 and its explicit reclaim_retired_branch_roots posture move to C2 with the restore pointer; do not defer/drop the public loops at lines142 and169. Replace private commit_artifacts and demand_commit_envelope rather than losing the recovery coverage.

Public assertions which must move now:

- L142: every captured commit ID is readable and equal after retained recovery.
- L169: the same IDs are readable and equal after external capture and source recovery.
- L180: provider `commit_reads.len()<32` keeps the existing externally measured read headroom.

Private subclaim only: `restore from 2a51552fc5:crates/worth-relational/src/tests/durability/medium/cold_envelope_lifetime.rs`; L125: `assert!( resident < 4, "released journal bodies fit head and reclamation headroom" )`. Preserve its private reclamation setup in C2. This is a split, not permission to defer the public recovery loops.

### 13. `recovered_nonhead_tail_envelopes_release_their_actual_journal_backed_bodies` ? class2

Source: `crates/worth-relational/src/tests/durability/medium/cold_envelope_lifetime.rs:187`.

Private commit_artifact resident_envelope and envelope_residency resident_payload_bytes plus exact Weak release.

`restore from 2a51552fc5:crates/worth-relational/src/tests/durability/medium/cold_envelope_lifetime.rs`.

Exact private assertions requiring C2:

- crates/worth-relational/src/tests/durability/medium/cold_envelope_lifetime.rs:220: `assert!(runtime .history .commit_artifact(head.commit_id) .unwrap() .resident_envelope() .is_some())`
- crates/worth-relational/src/tests/durability/medium/cold_envelope_lifetime.rs:226: `assert!( runtime .history .commit_artifacts() .iter() .all(|artifact| artifact .envelope_residency() .resident_payload_bytes() .is_none()), "journal placement releases encoded duplicate buffers too" )`
- crates/worth-relational/src/tests/durability/medium/cold_envelope_lifetime.rs:248: `assert!( weak.upgrade().is_none(), "replay must not turn a nonhead journal body into permanent memory truth" )`

### 14. `residency_failure_matrix_envelope_doors_preserve_every_failure_class` ? class1

Source: `crates/worth-relational/src/tests/durability/medium/cold_envelope_matrix/exhaustive.rs:4`.

EnvelopeDoor uses public history summaries/inspection/compiled artifact, bounded patches, patch stream, canonical artifact export, checkpoints, replay and mint_change_receipt. Replace private demand_commit_envelope with public replay().canonical_commit_envelope. Preserve all failure classes and provider logs.

### 15. `bounded_history_reads_only_the_selected_envelope_and_refuses_sticky_body_damage` ? class1

Source: `crates/worth-relational/src/tests/durability/medium/cold_envelopes.rs:9`.

Public history().bounded_canonical_commit_patches and immutable_commit_receipt_at_patch_stream_position; source commit/journal/member logs retain exact selected-position locality, no reload, refusal and contents.

### 16. `patch_stream_demands_only_the_requested_batch_and_refuses_sticky_journal_damage` ? class1

Source: `crates/worth-relational/src/tests/durability/medium/cold_envelopes.rs:92`.

Public publication().read_patch_stream(max_commits=1); preserve exact selected position/batch/frontier, damaged refusal, no extra provider requests or publication.

### 17. `envelope_surfaces_keep_index_facts_readable_and_refuse_cold_body_damage` ? class1

Source: `crates/worth-relational/src/tests/durability/medium/cold_envelopes.rs:260`.

Public historical_committed_version, inspect_commit/inspect_recent_commits and compile_execution_artifact; preserve readable metadata, typed damaged body, genuine result counts, requested envelope only.

### 18. `head_probes_admit_each_forks_shared_region_without_reading_history` ? class2

Source: `crates/worth-relational/src/tests/durability/medium/cold_fork_edits.rs:5`.

Private selected_branch_state and probe_head_region_with_symbols Cold->Resident assertions in each fork; public record read alone loses this state claim.

`restore from 2a51552fc5:crates/worth-relational/src/tests/durability/medium/cold_fork_edits.rs`.

Exact private assertions requiring C2:

- crates/worth-relational/src/tests/durability/medium/cold_fork_edits.rs:21: `assert!(matches!( selected.state().root().probe_head_region_with_symbols( cold.entities[0].partition_id, &runtime.services.symbols, ), crate::storage::residency::PartitionHeadProbe::Cold ))`
- crates/worth-relational/src/tests/durability/medium/cold_fork_edits.rs:37: `assert!(matches!( selected.state().root().probe_head_region_with_symbols( cold.entities[0].partition_id, &runtime.services.symbols, ), crate::storage::residency::PartitionHeadProbe::Resident(_) ))`

### 19. `cold_forks_edit_existing_ring_records_in_both_orders_and_checkpoint` ? class1

Source: `crates/worth-relational/src/tests/durability/medium/cold_fork_edits.rs:49`.

Public fork, transaction read/update/delete/create, native_checkpoint/restore_native_checkpoint. Obtain old/new relation IDs from public commit envelope/adjacency instead of private arena iteration; retain len>=2 and every entity/relation equality.

### 20. `residency_failure_matrix_historical_index_builds_preserve_full_source_failures` ? class2

Source: `crates/worth-relational/src/tests/durability/medium/cold_historical_index_matrix/exhaustive.rs:4`.

Private assert_cold helper exact interner absence and canonical_envelope absence; direct matrix canonical_envelope is_none also material.

`restore from 2a51552fc5:crates/worth-relational/src/tests/durability/medium/cold_historical_index_matrix/exhaustive.rs`.

Exact private assertions requiring C2:

- crates/worth-relational/src/tests/durability/medium/cold_historical_index_matrix/exhaustive.rs:35: `assert!(fixture .runtime .history .canonical_envelope(fixture.generation.source_commit_id) .is_none())`

### 21. `historical_index_reconstruction_uses_the_old_value_and_full_cold_history` ? class2

Source: `crates/worth-relational/src/tests/durability/medium/cold_historical_index_matrix.rs:69`.

Private assert_cold helper and exact interner.resolve(history_symbol)==Some(paged-view-value-0300); public old index-entry behavior can split.

`restore from 2a51552fc5:crates/worth-relational/src/tests/durability/medium/cold_historical_index_matrix.rs`.

Exact private assertions requiring C2:

- crates/worth-relational/src/tests/durability/medium/cold_historical_index_matrix.rs:77: `assert_eq!( fixture .runtime .services .symbols .interner_snapshot() .resolve(fixture.history_symbol.unwrap()), Some("paged-view-value-0300") )`

### 22. `an_older_index_directory_retains_its_cold_frame_after_home_replacement` ? class1

Source: `crates/worth-relational/src/tests/durability/medium/cold_index_custody.rs:97`.

Public durable-source custody trait and pin_view_read_capability/index_generation; provider-owned owners count is external lifecycle evidence, not internal runtime physical pins. Preserve member demand and last-owner release.

### 23. `residency_cold_index_frontier_prevents_reissue_before_any_image_demand` ? class2

Source: `crates/worth-relational/src/tests/durability/medium/cold_index_frontier.rs:5`.

Direct runtime.indexes reserve_generation_ids/next_generation_id/publish_generation[publish_generations_admitted], generation_frontier and demand_checkpoint_image; private owner actions and allocator frontier.

`restore from 2a51552fc5:crates/worth-relational/src/tests/durability/medium/cold_index_frontier.rs`.

Exact private assertions requiring C2:

- crates/worth-relational/src/tests/durability/medium/cold_index_frontier.rs:16: `assert!(reserved.0 > fixture.generation.generation_id.0)`
- crates/worth-relational/src/tests/durability/medium/cold_index_frontier.rs:29: `assert_eq!(owner.snapshot().generation_frontier(), reserved.0 + 1)`
- crates/worth-relational/src/tests/durability/medium/cold_index_frontier.rs:37: `assert_eq!( loaded .iter() .find(|generation| generation.generation_id == fixture.generation.generation_id) .map(Arc::as_ref), Some(fixture.generation.as_ref()) )`
- crates/worth-relational/src/tests/durability/medium/cold_index_frontier.rs:44: `assert_eq!( loaded .iter() .find(|generation| generation.generation_id == reserved) .map(Arc::as_ref), Some(&published) )`

### 24. `residency_failure_matrix_index_doors_preserve_every_failure_class` ? class1

Source: `crates/worth-relational/src/tests/durability/medium/cold_index_matrix/exhaustive.rs:4`.

Public generation_residency_for_observation, read_latest_generation/read_published_generation_for_commit/read_published_generation_for_observation/read_generations_for_version; public builds/pins/field prep/validation/prepare/commit. Preserve DeferredImage/Absent and every failure/effect assertion.

### 25. `selected_branch_views_preserve_cold_probe_and_demand` ? class2

Source: `crates/worth-relational/src/tests/durability/medium/cold_invariants.rs:53`.

Private selected_branch_state and probe_full_sources protocol; assertion observes a private view partition, not merely public record success.

`restore from 2a51552fc5:crates/worth-relational/src/tests/durability/medium/cold_invariants.rs`.

Exact private assertions requiring C2:

- crates/worth-relational/src/tests/durability/medium/cold_invariants.rs:63: `assert!(answer)`

### 26. `invariant_storage_refusal_is_sticky_and_does_not_settle_provisional_counts` ? class2

Source: `crates/worth-relational/src/tests/durability/medium/cold_invariants.rs:68`.

Private preparation_packet_count/invariant_entity_slot_scans/invariant_authoritative_entity_records_materialized unchanged on refusal.

`restore from 2a51552fc5:crates/worth-relational/src/tests/durability/medium/cold_invariants.rs`.

Exact private assertions requiring C2:

- crates/worth-relational/src/tests/durability/medium/cold_invariants.rs:105: `assert_eq!( after.preparation_packet_count, before.preparation_packet_count )`
- crates/worth-relational/src/tests/durability/medium/cold_invariants.rs:109: `assert_eq!( after.invariant_entity_slot_scans, before.invariant_entity_slot_scans )`
- crates/worth-relational/src/tests/durability/medium/cold_invariants.rs:113: `assert_eq!( after.invariant_authoritative_entity_records_materialized, before.invariant_authoritative_entity_records_materialized )`

### 27. `cold_and_warm_invariant_reads_settle_one_round_of_evidence` ? class2

Source: `crates/worth-relational/src/tests/durability/medium/cold_invariants.rs:122`.

Private exact cold/warm packet and invariant slot-scan settlement deltas; keep public result equality too.

`restore from 2a51552fc5:crates/worth-relational/src/tests/durability/medium/cold_invariants.rs`.

Exact private assertions requiring C2:

- crates/worth-relational/src/tests/durability/medium/cold_invariants.rs:147: `assert!(cold_packets > 0)`
- crates/worth-relational/src/tests/durability/medium/cold_invariants.rs:148: `assert_eq!( cold_packets, warm_packets, "a discarded round must not publish its counters" )`
- crates/worth-relational/src/tests/durability/medium/cold_invariants.rs:152: `assert_eq!( after_cold.invariant_entity_slot_scans - before.invariant_entity_slot_scans, after_warm.invariant_entity_slot_scans - after_cold.invariant_entity_slot_scans, )`

### 28. `the_invariant_orchestrator_uses_head_while_full_history_failure_stays_typed` ? class1

Source: `crates/worth-relational/src/tests/durability/medium/cold_invariants.rs:159`.

Public validation().mutation_sensitive_state and storage_access().storage_stats; source member log distinguishes Head vs history and results/typed sticky failure/frontier remain public.

### 29. `a_relation_bearing_cold_commit_loads_only_its_partition` ? class1

Source: `crates/worth-relational/src/tests/durability/medium/cold_isolation.rs:189`.

Public transaction commit plus external source count reads<12. Provider I/O locality is already public observation, not private allocation accounting.

### 30. `missing_history_damages_only_its_own_region` ? class1

Source: `crates/worth-relational/src/tests/durability/medium/cold_isolation.rs:199`.

Public read_truth().visible_entity_at_version for damaged region Full demand yields same StorageUnavailable subject/cause; repeat without provider reload, preserve healthy transaction reads/writes. Replace private demand_regions_full, not its failure/locality claims.

### 31. `fork_of_a_cold_runtime_shares_cells_and_both_copies_can_read_and_write` ? class1

Source: `crates/worth-relational/src/tests/durability/medium/cold_isolation.rs:241`.

Public fork and transaction reads/commits; preserve zero-I/O fork and readable/writable contents in both copies.

### 32. `replay_pin_of_a_cold_partition_is_typed_and_has_no_effect` ? class2

Source: `crates/worth-relational/src/tests/durability/medium/cold_isolation.rs:277`.

Private fabricated SnapshotState/DenseSlotBitSet, visibility_pins resident_replay_pins/pin_replay_state, exact replay_pin_count 2->1->0.

`restore from 2a51552fc5:crates/worth-relational/src/tests/durability/medium/cold_isolation.rs`.

Exact private assertions requiring C2:

- crates/worth-relational/src/tests/durability/medium/cold_isolation.rs:323: `assert_eq!( failure.subject, crate::storage::data::RelationalStorageSubject::Region(PartitionId(1)) )`
- crates/worth-relational/src/tests/durability/medium/cold_isolation.rs:327: `assert_eq!(failure.cause, Cause::SourceUnavailable)`
- crates/worth-relational/src/tests/durability/medium/cold_isolation.rs:328: `assert_eq!(cold.source.reads.load(Ordering::Relaxed), 0)`
- crates/worth-relational/src/tests/durability/medium/cold_isolation.rs:331: `assert_eq!( cold.runtime .visibility_pins() .resident_replay_pins(state.clone()) .unwrap_err() .cause, Cause::SourceUnavailable )`
- crates/worth-relational/src/tests/durability/medium/cold_isolation.rs:375: `assert_eq!(pin_count(), Some(2))`
- crates/worth-relational/src/tests/durability/medium/cold_isolation.rs:379: `assert_eq!(pin_count(), Some(1))`
- crates/worth-relational/src/tests/durability/medium/cold_isolation.rs:383: `assert_eq!(pin_count(), Some(0))`

### 33. `an_older_lineage_directory_retains_its_cold_pages_after_home_replacement` ? class3

Source: `crates/worth-relational/src/tests/durability/medium/cold_lineage_custody.rs:131`.

Action runtime.lineage.demand_node returns an exact node lease; no public node-by-lineage observation exists. Minimal seam: typed node observation from owner-issued branch/lineage selection. visible_entities_for_lineages_at_observation returns IDs and demands selected regions, not the asserted node lease. Preserve provider custody and final-owner release.

### 34. `changed_event_bytes_damage_only_their_page_and_share_the_refusal_with_export` ? class3

Source: `crates/worth-relational/src/tests/durability/medium/cold_lineage_event_integrity.rs:91`.

Export is public; raw branch-scoped resolve_historical_lineage(seed), healthy exact traversal and shared query/export damage have no exact public door. Minimal seam: typed branch-bound lineage-history observation with existing request semantics; preserve no extra source reads and healthy unrelated result. record_history_at_observation changes demand/selection semantics.

### 35. `residency_failure_matrix_lineage_event_doors_preserve_every_failure_class` ? class3

Source: `crates/worth-relational/src/tests/durability/medium/cold_lineage_event_matrix/exhaustive.rs:4`.

Mixed EventDoor matrix: Observation/Aspect/AspectTrace/MemoryCheckpoint/EmbeddedCheckpoint are public; AccessAspect/AccessAspectTrace are cfg(test) forwarding aliases of the existing public history().lineage_entity_aspect_history[_with_trace], so use those exact public equivalents. Historical/Record need typed branch-bound history observation. Graph/Divergence are cfg(test) private full-graph traversals; split those C2, preserving their graph-event/shared-lineage assertions. Do not promote dormant collectors into public code to retain ALL. No new sealed door is required for a typed semantic history observation; its design still needs the owner ruling.

### 36. `a_reopened_lineage_resolution_follows_only_its_actual_event_pages` ? class2

Source: `crates/worth-relational/src/tests/durability/medium/cold_lineage_events.rs:5`.

Private lineage.event_count()<601/2 is exact resident event-index evidence, beyond semantic resolved/traversed IDs and digest equality.

`restore from 2a51552fc5:crates/worth-relational/src/tests/durability/medium/cold_lineage_events.rs`.

Exact private assertions requiring C2:

- crates/worth-relational/src/tests/durability/medium/cold_lineage_events.rs:20: `assert!( fixture.runtime.lineage.event_count() < 601 / 2, "a lineage query leaves unrelated node ranges' events cold" )`

### 37. `a_shared_event_page_is_admitted_once_into_each_forked_lineage_authority` ? class2

Source: `crates/worth-relational/src/tests/durability/medium/cold_lineage_events.rs:27`.

Private fork.lineage.event_count==0, bounded admitted count and unchanged warm event count prove per-runtime index admission.

`restore from 2a51552fc5:crates/worth-relational/src/tests/durability/medium/cold_lineage_events.rs`.

Exact private assertions requiring C2:

- crates/worth-relational/src/tests/durability/medium/cold_lineage_events.rs:38: `assert_eq!( fork.lineage.event_count(), 0, "the fork's index has not admitted shared bytes yet" )`
- crates/worth-relational/src/tests/durability/medium/cold_lineage_events.rs:49: `assert!(fork.lineage.event_count() < 601 / 2)`
- crates/worth-relational/src/tests/durability/medium/cold_lineage_events.rs:58: `assert_eq!( fork.lineage.event_count(), admitted, "a warm query never reinstalls its page" )`

### 38. `concurrent_lineage_history_reads_share_one_actual_event_member_load` ? class3

Source: `crates/worth-relational/src/tests/durability/medium/cold_lineage_events.rs:67`.

Eight concurrent raw resolve_historical_lineage operations assert identical digest, one event-member load and no node/symbol/region reads. Existing record_history_at_observation first demands Full region. Minimal seam: typed branch-bound lineage-history observer retaining raw request semantics, not a snapshot that loads unrelated payloads.

### 39. `node_range_boundaries_keep_neighboring_event_members_cold` ? class2

Source: `crates/worth-relational/src/tests/durability/medium/cold_lineage_events.rs:126`.

Private lineage.event_count()<601/2 and raw seed selection; preserve neighboring-event provider no-read and exact unchanged lineage result.

`restore from 2a51552fc5:crates/worth-relational/src/tests/durability/medium/cold_lineage_events.rs`.

Exact private assertions requiring C2:

- crates/worth-relational/src/tests/durability/medium/cold_lineage_events.rs:164: `assert!(fixture.runtime.lineage.event_count() < 601 / 2)`

### 40. `native_capture_from_a_fork_uses_shared_resident_event_pages_without_losing_rows` ? class2

Source: `crates/worth-relational/src/tests/durability/medium/cold_lineage_events.rs:168`.

Private demand_event_pages/demand_symbol_ids/demand_node/full-root warming creates partial per-runtime admission; exact fork.lineage.event_count==0 before/after no-load export is resident-only.

`restore from 2a51552fc5:crates/worth-relational/src/tests/durability/medium/cold_lineage_events.rs`.

Exact private assertions requiring C2:

- crates/worth-relational/src/tests/durability/medium/cold_lineage_events.rs:181: `assert_eq!( fork.lineage.event_count(), 0, "shared residency does not populate another runtime's index" )`
- crates/worth-relational/src/tests/durability/medium/cold_lineage_events.rs:253: `assert_eq!( fork.lineage.event_count(), 0, "capture reads resident pages without rewriting the runtime's query index" )`

### 41. `residency_failure_matrix_lineage_doors_preserve_every_failure_class` ? class2

Source: `crates/worth-relational/src/tests/durability/medium/cold_lineage_matrix/exhaustive.rs:4`.

Direct lineage.node(...).is_some retry assertion; NodeDoor Graph/Divergence cfg(test) and Point/RecordHistory lack public raw doors. Split public observation/history/checkpoint cases, keep exact node residency C2.

`restore from 2a51552fc5:crates/worth-relational/src/tests/durability/medium/cold_lineage_matrix/exhaustive.rs`.

Exact private assertions requiring C2:

- crates/worth-relational/src/tests/durability/medium/cold_lineage_matrix/exhaustive.rs:66: `assert!( fixture .runtime .lineage .node(fixture.expected.lineage_id()) .is_some(), "retry must admit the requested node" )`

### 42. `a_cold_lineage_point_read_loads_one_node_page_and_no_region_history` ? class2

Source: `crates/worth-relational/src/tests/durability/medium/cold_lineage_nodes.rs:9`.

Private lineage.node_count<=256 and <POPULATION/2; direct lineage point action has no public exact-node observer. Retain one-member/no-region-history/no-envelope-read too.

`restore from 2a51552fc5:crates/worth-relational/src/tests/durability/medium/cold_lineage_nodes.rs`.

Exact private assertions requiring C2:

- crates/worth-relational/src/tests/durability/medium/cold_lineage_nodes.rs:21: `assert!(fixture.runtime.lineage.node_count() <= 256)`
- crates/worth-relational/src/tests/durability/medium/cold_lineage_nodes.rs:22: `assert!( fixture.runtime.lineage.node_count() < POPULATION / 2, "point demand leaves unrelated node pages cold" )`

### 43. `every_node_guard_preserves_a_sticky_typed_page_failure_without_publication` ? class2

Source: `crates/worth-relational/src/tests/durability/medium/cold_lineage_nodes.rs:143`.

Private lineage.node_count zero/unchanged, node(...).is_none on damage, direct commit-artifact warming and dormant graph/divergence cases.

`restore from 2a51552fc5:crates/worth-relational/src/tests/durability/medium/cold_lineage_nodes.rs`.

Exact private assertions requiring C2:

- crates/worth-relational/src/tests/durability/medium/cold_lineage_nodes.rs:164: `assert_eq!(fixture.runtime.lineage.node_count(), 0)`
- crates/worth-relational/src/tests/durability/medium/cold_lineage_nodes.rs:180: `assert!( fixture .runtime .lineage .node(fixture.expected.lineage_id()) .is_none(), "damaged page admits no node" )`
- crates/worth-relational/src/tests/durability/medium/cold_lineage_nodes.rs:191: `assert_eq!(fixture.runtime.lineage.node_count(), admitted_nodes)`

### 44. `simultaneous_lineage_point_reads_share_one_page_load` ? class3

Source: `crates/worth-relational/src/tests/durability/medium/cold_lineage_nodes.rs:204`.

lineage_access().for_record is pub(crate), LineageAccess unexported; public visible-entities API returns IDs rather than asserted LineageNode. Minimal seam: typed node-for-record observation on existing owner selected by admitted branch observation/entity; preserve exact node equality and one provider page under concurrency.

### 45. `residency_materialization_source_manifest_demands_before_reading_or_refusing_damage` ? class1

Source: `crates/worth-relational/src/tests/durability/medium/cold_materialization_doors.rs:11`.

Public owner_component_services().materialization_port().prepare_generated_materialization_suspension; completion records, typed Region damage and provider-read count are externally observable.

### 46. `residency_retained_kind_demands_only_its_cold_head_and_preserves_damage` ? class2

Source: `crates/worth-relational/src/tests/durability/medium/cold_materialization_doors.rs:59`.

Private basis.inner.root.probe_head_region Cold and probe_full_region Cold; retained_entity_kind is public and its kind/failure semantics can split out, but exact posture assertions remain C2.

`restore from 2a51552fc5:crates/worth-relational/src/tests/durability/medium/cold_materialization_doors.rs`.

Exact private assertions requiring C2:

- crates/worth-relational/src/tests/durability/medium/cold_materialization_doors.rs:129: `assert!( matches!( basis.inner.root.probe_head_region(retained.partition_id), PartitionHeadProbe::Cold ), "the unrelated retained identity must still be cold at this public door" )`
- crates/worth-relational/src/tests/durability/medium/cold_materialization_doors.rs:149: `assert!( matches!( basis.inner.root.probe_full_region(retained.partition_id), PartitionProbe::Cold ), "a value-free kind read must not demand Full history" )`

### 47. `residency_load_free_sharing_and_ledger_refuse_cold_payloads_without_inventing_absence` ? class1

Source: `crates/worth-relational/src/tests/durability/medium/cold_materialization_doors.rs:173`.

Public observe_branch_sharing, inspect_owner_allocation_ledger and observe_retained_storage; typed unavailable-owner refusals and unchanged provider reads preserve load-free evidence.

### 48. `a_damaged_cold_member_is_a_sticky_typed_transaction_read_refusal` ? class1

Source: `crates/worth-relational/src/tests/durability/medium/cold_members.rs:98`.

Public transaction read_entity/commit, Preparation reason, history().branch_head and external source counter; preserve typed sticky damage, no publication and no source retry.

### 49. `tail_replay_refuses_a_damaged_touched_member_as_cold_storage_unavailable` ? class1

Source: `crates/worth-relational/src/tests/durability/medium/cold_members.rs:171`.

Public recovery_plan_from_source/recover; preserve ColdStorageUnavailable class, IntegrityMismatch cause and untouched receiver branch.

### 50. `a_cold_point_read_keeps_cross_partition_kind_authority_after_an_unrelated_write` ? class1

Source: `crates/worth-relational/src/tests/durability/medium/cold_members.rs:206`.

Public transactions and checkpoint paths; replace private Head lease outgoing with projected try_visit_adjacency_ids or storage_access().outgoing_relations_for_entity; preserve exact receipt relation IDs and readable cross-partition entity.

### 51. `residency_failure_matrix_nonhead_merge_history_covers_every_door_and_class` ? class2

Source: `crates/worth-relational/src/tests/durability/medium/cold_merge_history_matrix/exhaustive.rs:4`.

Private HistoryPlan invokes plan_history_scope_for_test and asserts target_delta/source_delta touched record, intermediate commit_ids and row.envelopes length. Public MergePlanningArtifactCore exposes no such envelope lists. Split IndependentPreparation/TransactionParents public subfamilies; retain exact delta evidence C2.

`restore from 2a51552fc5:crates/worth-relational/src/tests/durability/medium/cold_merge_history_matrix/exhaustive.rs`.

Exact private assertions requiring C2:

- crates/worth-relational/src/tests/durability/medium/cold_merge_history_matrix.rs:45: `assert!(plan .target_delta .touched_records .iter() .any(|row| row.target == overlap))`
- crates/worth-relational/src/tests/durability/medium/cold_merge_history_matrix.rs:56: `assert!(row .commit_ids() .any(|id| id == fixture.intermediate.commit_id))`

### 52. `a_cold_intermediate_source_commit_cannot_disappear_from_merge_parent_conflicts` ? class1

Source: `crates/worth-relational/src/tests/durability/medium/cold_merge_history_matrix.rs:109`.

This test selects only TransactionParents, not helper HistoryPlan. Public owner-issued merge transaction/commit preserves DiagnosticCode::MergeConflictOverlap and unchanged frontier.

### 53. `residency_failure_matrix_merge_planning_and_commit_leave_unconsulted_generations_cold` ? class1

Source: `crates/worth-relational/src/tests/durability/medium/cold_merge_index_matrix/exhaustive.rs:4`.

Public owner-bound prepare_merge_execution/execute_prepared_merge, index_authority().build_for_commit/read_latest_generation, read_snapshot; preserve source no-generation-demand until explicit rebuild and exact warm-reference index keys/entity parity.

### 54. `residency_failure_matrix_merge_planning_preserves_versioned_footprints` ? class1

Source: `crates/worth-relational/src/tests/durability/medium/cold_merge_member_matrix/exhaustive.rs:4`.

Same public owner-bound merge paths, exact typed refusals/retry, no reservation/effects, provider member/commit reuse and adopted-source record evidence. Shared recovered_shape(symbols=true) fixture exact interner-absence assertion must be retained separately C2.

### 55. `cold_merge_preparation_refuses_region_damage_without_journal_effects` ? class1

Source: `crates/worth-relational/src/tests/durability/medium/cold_merges.rs:206`.

Public owner-bound prepare_merge_execution; exact typed sticky region denial, provider no reread and no journal/frontier effects.

### 56. `independent_merge_executes_from_a_real_cold_checkpoint` ? class1

Source: `crates/worth-relational/src/tests/durability/medium/cold_merges.rs:251`.

Public owner-bound merge prepare/execute, structural_summary and changed public patches/read_snapshot; preserve adopted one, changed one and actual three-entity result.

### 57. `head_resident_metadata_refuses_to_open_unavailable_history` ? class1

Source: `crates/worth-relational/src/tests/durability/medium/cold_metadata_matrix.rs:4`.

Public projected exact entity state/relation metadata/aspect revision/adjacency structural revision; source log proves unavailable history never consulted.

### 58. `metadata_pin_requires_resident_head_and_never_opens_history` ? class1

Source: `crates/worth-relational/src/tests/durability/medium/cold_metadata_pins.rs:5`.

Public pin_resident_entity_metadata RegionNotResident and pinned live_kind/aspect_version/retirement; public transaction read warms Head. Preserve old metadata after handle release/later commit and no history demand. No physical pin accounting.

### 59. `metadata_pin_refuses_shared_head_until_its_runtime_has_the_aspect_meanings` ? class2

Source: `crates/worth-relational/src/tests/durability/medium/cold_metadata_pins.rs:89`.

Private selected_root().demand_region_head_cached deliberately creates shared Head bytes without runtime dictionary admission; preserve SymbolsNotResident and subsequent exact metadata/revision assertions C2.

`restore from 2a51552fc5:crates/worth-relational/src/tests/durability/medium/cold_metadata_pins.rs`.

Exact private assertions requiring C2:

- crates/worth-relational/src/tests/durability/medium/cold_metadata_pins.rs:112: `assert_eq!(fixture.source.reads.load(Ordering::Relaxed), reads)`
- crates/worth-relational/src/tests/durability/medium/cold_metadata_pins.rs:113: `assert!( matches!(fork.read_truth().pin_resident_entity_metadata(&snapshot, fixture.entity), Err(Denial::SymbolsNotResident(partition)) if partition == fixture.entity.partition_id) )`

### 60. `an_admitted_head_read_never_becomes_absence_after_transport_damage` ? class1

Source: `crates/worth-relational/src/tests/durability/medium/cold_metadata_pins.rs:157`.

Public entity_field_revision_admitted callback retains accepted custody; native_checkpoint discovers true source damage; fresh entity_field_revision still refuses. Preserve nonabsence and typed fresh refusal.

### 61. `retained_observation_region_guards_preserve_sticky_storage_failure` ? class1

Source: `crates/worth-relational/src/tests/durability/medium/cold_observation_reads.rs:68`.

Public entity_record_at_observation/relation_record_at_observation/record_history_at_observation/visible_entities_for_lineages_at_observation; preserve exact sticky region failures, source logs/frontier/offers. Private prerequisite node warming needs separate fixture treatment, not a new exposed demand_node.

### 62. `lineage_point_guard_preserves_a_cold_mirror_head_failure` ? class2

Source: `crates/worth-relational/src/tests/durability/medium/cold_observation_reads.rs:107`.

Private lineage.node_count()==0 and private lineage_access().for_record action; public ID-only analogue does not preserve exact node evidence.

`restore from 2a51552fc5:crates/worth-relational/src/tests/durability/medium/cold_observation_reads.rs`.

Exact private assertions requiring C2:

- crates/worth-relational/src/tests/durability/medium/cold_observation_reads.rs:123: `assert_eq!(fixture.runtime.lineage.node_count(), 0)`

### 63. `replay_pinning_refuses_a_damaged_mirror_without_installing_retention` ? class2

Source: `crates/worth-relational/src/tests/durability/medium/cold_pins.rs:146`.

Private visibility.replay_retention_versions empty before/after denial, cached historical-state diagnostics. Public retention refusal alone loses installed-retention proof.

`restore from 2a51552fc5:crates/worth-relational/src/tests/durability/medium/cold_pins.rs`.

Exact private assertions requiring C2:

- crates/worth-relational/src/tests/durability/medium/cold_pins.rs:153: `assert!( runtime.visibility.replay_retention_versions().is_empty(), "fixture has no prior replay retention" )`
- crates/worth-relational/src/tests/durability/medium/cold_pins.rs:184: `assert!(runtime.visibility.replay_retention_versions().is_empty())`
- crates/worth-relational/src/tests/durability/medium/cold_pins.rs:199: `assert!(runtime.visibility.replay_retention_versions().is_empty())`

### 64. `resident_pin_custody_refuses_unread_mirror_frames_after_exact_source_admission` ? class2

Source: `crates/worth-relational/src/tests/durability/medium/cold_pins.rs:216`.

Private SnapshotState and visibility_pins().demand_replay_pins plus replay_retention_versions().is_empty; exact source admission/pin action is private.

`restore from 2a51552fc5:crates/worth-relational/src/tests/durability/medium/cold_pins.rs`.

Exact private assertions requiring C2:

- crates/worth-relational/src/tests/durability/medium/cold_pins.rs:245: `assert!(fixture .runtime .visibility .replay_retention_versions() .is_empty())`

### 65. `replay_retention_releases_from_owned_resident_custody_after_cache_clear` ? class2

Source: `crates/worth-relational/src/tests/durability/medium/cold_pins.rs:261`.

Private visibility.cached_historical_states.clear and replay_pin_count sequence1,1,1,0; preserve public retain/release/extra-release false and provider no loads.

`restore from 2a51552fc5:crates/worth-relational/src/tests/durability/medium/cold_pins.rs`.

Exact private assertions requiring C2:

- crates/worth-relational/src/tests/durability/medium/cold_pins.rs:272: `assert_eq!(replay_pin_count(&fixture), 1)`
- crates/worth-relational/src/tests/durability/medium/cold_pins.rs:279: `assert_eq!( replay_pin_count(&fixture), 1, "additional retention references share physical pins" )`
- crates/worth-relational/src/tests/durability/medium/cold_pins.rs:290: `assert_eq!(replay_pin_count(&fixture), 1)`
- crates/worth-relational/src/tests/durability/medium/cold_pins.rs:294: `assert_eq!(replay_pin_count(&fixture), 0)`

### 66. `residency_failure_matrix_remote_invariant_heads_cover_every_preparation_door_and_class` ? class2

Source: `crates/worth-relational/src/tests/durability/medium/cold_preparation_failure_matrix/exhaustive.rs:4`.

Private services.symbols.snapshot equality plus preparation_packet_count/invariant_entity_slot_scans preserved for each failure class, beyond public refusal/no-effects.

`restore from 2a51552fc5:crates/worth-relational/src/tests/durability/medium/cold_preparation_failure_matrix/exhaustive.rs`.

Exact private assertions requiring C2:

- crates/worth-relational/src/tests/durability/medium/cold_preparation_failure_matrix/exhaustive.rs:34: `assert_eq!(runtime.services.symbols.snapshot(), symbols)`
- crates/worth-relational/src/tests/durability/medium/cold_preparation_failure_matrix/exhaustive.rs:36: `assert_eq!( after.preparation_packet_count, before.preparation_packet_count )`
- crates/worth-relational/src/tests/durability/medium/cold_preparation_failure_matrix/exhaustive.rs:40: `assert_eq!( after.invariant_entity_slot_scans, before.invariant_entity_slot_scans )`
- crates/worth-relational/src/tests/durability/medium/cold_preparation_failure_matrix/exhaustive.rs:52: `assert_eq!(runtime.services.symbols.snapshot(), symbols)`

### 67. `residency_failure_matrix_remote_current_invariants_never_consult_history` ? class1

Source: `crates/worth-relational/src/tests/durability/medium/cold_preparation_failure_matrix/exhaustive.rs:70`.

Public validate/prepare_branch_transaction/commit plus provider member log prove remote current invariant never reads history. Shared preparation fixture Cold-probe/arena-floor assertions remain separately C2.

### 68. `residency_failure_matrix_projection_adjacency_guards_selected_heads` ? class2

Source: `crates/worth-relational/src/tests/durability/medium/cold_projection_adjacency_matrix/exhaustive.rs:4`.

Inherited cold_projection_adjacency_matrix.rs Door::BoundedIncoming asserts exact interner.resolve(remote_symbol)==Some(remote-paged-edge). Preserve this dictionary claim C2; public callback/typed-source-failure matrix portions may split.

`restore from 2a51552fc5:crates/worth-relational/src/tests/durability/medium/cold_projection_adjacency_matrix/exhaustive.rs`.

Exact private assertions requiring C2:

- crates/worth-relational/src/tests/durability/medium/cold_projection_adjacency_matrix.rs:81: `assert_eq!(cold.runtime.services.symbols.interner_snapshot().resolve(symbol), Some("remote-paged-edge"), "actual incoming value={value:?}; member reads={:?}", cold.source.member_reads.lock().unwrap())`

### 69. `projection_adjacency_discovery_settles_only_one_accepted_round` ? class2

Source: `crates/worth-relational/src/tests/durability/medium/cold_projection_adjacency_matrix.rs:174`.

Private performance_access reset/counters: exact adjacency_kind_slices_leased and visible_authoritative_relation_records_materialized cold/warm equality.

`restore from 2a51552fc5:crates/worth-relational/src/tests/durability/medium/cold_projection_adjacency_matrix.rs`.

Exact private assertions requiring C2:

- crates/worth-relational/src/tests/durability/medium/cold_projection_adjacency_matrix.rs:187: `assert!(warm.adjacency_kind_slices_leased > 0)`
- crates/worth-relational/src/tests/durability/medium/cold_projection_adjacency_matrix.rs:188: `assert_eq!( initial.adjacency_kind_slices_leased, warm.adjacency_kind_slices_leased, "{door:?} historical={historical}" )`
- crates/worth-relational/src/tests/durability/medium/cold_projection_adjacency_matrix.rs:192: `assert_eq!( initial.visible_authoritative_relation_records_materialized, warm.visible_authoritative_relation_records_materialized )`

### 70. `an_exhausted_frontier_never_demands_an_unvisited_anchor` ? class1

Source: `crates/worth-relational/src/tests/durability/medium/cold_projection_adjacency_matrix.rs:200`.

Public bounded_incoming_relations_for_frontier(...Some(0)) rejects before any source demand; preserve zero reads despite armed damaged source. This test does not enter private-dictionary Door::read.

### 71. `residency_failure_matrix_full_read_subjects_cover_every_door_and_class` ? class2

Source: `crates/worth-relational/src/tests/durability/medium/cold_read_matrix/exhaustive.rs:4`.

Direct private interner.resolve(symbol).is_none asserts exact historical meaning remains absent for current scans; retain all47 public consumer/failure cells when splitting.

`restore from 2a51552fc5:crates/worth-relational/src/tests/durability/medium/cold_read_matrix/exhaustive.rs`.

Exact private assertions requiring C2:

- crates/worth-relational/src/tests/durability/medium/cold_read_matrix/exhaustive.rs:41: `assert!(fixture .runtime .services .symbols .interner_snapshot() .resolve(symbol) .is_none())`

### 72. `a_reserved_candidate_published_after_another_branch_checkpoint_keeps_its_pages` ? class2

Source: `crates/worth-relational/src/tests/durability/medium/cold_reserved_checkpoint.rs:32`.

Private exact services.symbols.snapshot entries/demand_symbol_ids equality across checkpoints and private lineage node observation; retain public reserved-publication/reopened entity behavior too.

`restore from 2a51552fc5:crates/worth-relational/src/tests/durability/medium/cold_reserved_checkpoint.rs`.

Exact private assertions requiring C2:

- crates/worth-relational/src/tests/durability/medium/cold_reserved_checkpoint.rs:102: `assert_eq!( reopened .services .symbols .snapshot() .entries .iter() .find(|(id, _)| *id == symbol) .map(|(_, value)| value), Some(&text) )`

### 73. `cold_retirement_metadata_carries_original_creation_and_deletion_in_head` ? class1

Source: `crates/worth-relational/src/tests/durability/medium/cold_retirement_metadata.rs:4`.

Public projected read_entity_retirement and pin_resident_entity_metadata live_kind/retirement; preserve authentic creation/deletion versions and source no-history-read.

### 74. `the_remote_ring_label_stays_cold_until_its_incoming_edge_is_read` ? class2

Source: `crates/worth-relational/src/tests/durability/medium/cold_ring_dictionary.rs:84`.

Private interner.resolve(remote_symbol).is_none at reopen and after anchor read is central; preserve incoming typed page refusal along with these exact dictionary assertions.

`restore from 2a51552fc5:crates/worth-relational/src/tests/durability/medium/cold_ring_dictionary.rs`.

Exact private assertions requiring C2:

- crates/worth-relational/src/tests/durability/medium/cold_ring_dictionary.rs:89: `assert!( cold.runtime .services .symbols .interner_snapshot() .resolve(symbol) .is_none(), "reopen must not admit the remote field text" )`
- crates/worth-relational/src/tests/durability/medium/cold_ring_dictionary.rs:102: `assert!( cold.runtime .services .symbols .interner_snapshot() .resolve(symbol) .is_none(), "the anchor's dictionary closure excludes the remote label" )`

### 75. `residency_storage_adjacency_doors_demand_before_reading_and_preserve_cold_warm_answers` ? class1

Source: `crates/worth-relational/src/tests/durability/medium/cold_storage_adjacency.rs:31`.

Public storage_access outgoing/incoming/all_relations_for_entity; exact vec[relation] cold/warm parity and provider loads once remain public.

### 76. `residency_failure_matrix_symbol_buckets_cover_every_preparation_door_and_class` ? class2

Source: `crates/worth-relational/src/tests/durability/medium/cold_symbol_failure_matrix/exhaustive.rs:4`.

Private services.symbols.snapshot equality before/after each preparation failure. Diagnostics can use public accessor; exact dictionary contents cannot be replaced with source logs.

`restore from 2a51552fc5:crates/worth-relational/src/tests/durability/medium/cold_symbol_failure_matrix/exhaustive.rs`.

Exact private assertions requiring C2:

- crates/worth-relational/src/tests/durability/medium/cold_symbol_failure_matrix/exhaustive.rs:46: `assert_eq!(fixture.runtime.services.symbols.snapshot(), symbols)`
- crates/worth-relational/src/tests/durability/medium/cold_symbol_failure_matrix/exhaustive.rs:64: `assert_eq!(fixture.runtime.services.symbols.snapshot(), symbols)`

### 77. `every_full_read_door_admits_its_runtime_dictionary_or_preserves_page_damage` ? class2

Source: `crates/worth-relational/src/tests/durability/medium/cold_symbol_read_doors.rs:8`.

Private interner historical absence for current scans and exact resolution paged-view-value-0300 after Full; keep typed page failure, stickiness and healthy Head assertions too.

`restore from 2a51552fc5:crates/worth-relational/src/tests/durability/medium/cold_symbol_read_doors.rs`.

Exact private assertions requiring C2:

- crates/worth-relational/src/tests/durability/medium/cold_symbol_read_doors.rs:44: `assert!(fixture .runtime .services .symbols .interner_snapshot() .resolve(fixture.historical) .is_none())`
- crates/worth-relational/src/tests/durability/medium/cold_symbol_read_doors.rs:94: `assert_eq!( fixture .runtime .services .symbols .interner_snapshot() .resolve(fixture.historical), Some("paged-view-value-0300"), "{door:?} must admit Full meanings into its own runtime" )`

### 78. `head_symbols_never_cover_full_history_or_inherit_its_page_damage` ? class2

Source: `crates/worth-relational/src/tests/durability/medium/cold_symbol_view_levels.rs:19`.

Private exact interner/Symbol identity, dictionary entries.len<400, metadata_history_at and Head/Full coverage probes. Entire distinction is internal meaning admission evidence.

`restore from 2a51552fc5:crates/worth-relational/src/tests/durability/medium/cold_symbol_view_levels.rs`.

Exact private assertions requiring C2:

- crates/worth-relational/src/tests/durability/medium/cold_symbol_view_levels.rs:46: `assert!( matches!(value, AspectValue::String(InternedString::Symbol(symbol)) if symbol.0 == fixture.current.0) )`
- crates/worth-relational/src/tests/durability/medium/cold_symbol_view_levels.rs:50: `assert_eq!( symbols.resolve(fixture.current), Some("paged-view-value-0600") )`
- crates/worth-relational/src/tests/durability/medium/cold_symbol_view_levels.rs:54: `assert!( symbols.resolve(fixture.historical).is_none(), "Head must not admit the history-only page of a physically Full cell" )`
- crates/worth-relational/src/tests/durability/medium/cold_symbol_view_levels.rs:58: `assert!( fixture.runtime.services.symbols.snapshot().entries.len() < 400, "one current closure has ample headroom below the 601 admitted texts" )`
- crates/worth-relational/src/tests/durability/medium/cold_symbol_view_levels.rs:99: `assert!( read_full().unwrap().unwrap() > 1, "Full must retain both real committed field versions" )`
- crates/worth-relational/src/tests/durability/medium/cold_symbol_view_levels.rs:103: `assert_eq!( fixture .runtime .services .symbols .interner_snapshot() .resolve(fixture.historical), Some("paged-view-value-0300") )`
- crates/worth-relational/src/tests/durability/medium/cold_symbol_view_levels.rs:118: `assert!(probe_full(execution, &source, |view| view .get_partition(fixture.entity.partition_id) .is_some()) .unwrap())`
- crates/worth-relational/src/tests/durability/medium/cold_symbol_view_levels.rs:122: `assert!( matches!( selected.state().root().probe_head_region_with_symbols( fixture.entity.partition_id, &covered.services.symbols, ), crate::storage::residency::PartitionHeadProbe::Resident(_) ), "a Full symbol guard must cover Head without recollecting the view" )`

### 79. `cold_symbol_buckets_refuse_each_preparation_door_without_journal_effects_and_stay_damaged` ? class2

Source: `crates/worth-relational/src/tests/durability/medium/cold_symbols.rs:210`.

Private services.symbols.snapshot exact equality after failed preparation; preserve typed sticky bucket denial and frontier/offers too.

`restore from 2a51552fc5:crates/worth-relational/src/tests/durability/medium/cold_symbols.rs`.

Exact private assertions requiring C2:

- crates/worth-relational/src/tests/durability/medium/cold_symbols.rs:241: `assert_eq!( fixture.runtime.services.symbols.snapshot(), symbols, "failed bucket or draft publishes no meaning" )`

### 80. `a_cold_symbol_probe_spends_the_abandoned_record_identity_and_reads_only_touched_buckets` ? class2

Source: `crates/worth-relational/src/tests/durability/medium/cold_symbols.rs:258`.

Private relation-arena slot floor, spent-draft slot/headroom inequalities and dictionary entries.len<POPULATION/2; provider I/O bound retained alongside.

`restore from 2a51552fc5:crates/worth-relational/src/tests/durability/medium/cold_symbols.rs`.

Exact private assertions requiring C2:

- crates/worth-relational/src/tests/durability/medium/cold_symbols.rs:273: `assert!( relation.slot_index() > fixture.relation_floor, "a discarded symbol-probe draft cannot reuse its identity" )`
- crates/worth-relational/src/tests/durability/medium/cold_symbols.rs:277: `assert!( relation.slot_index() < fixture.relation_floor + 16, "retry growth uses headroom" )`
- crates/worth-relational/src/tests/durability/medium/cold_symbols.rs:285: `assert!( fixture.runtime.services.symbols.snapshot().entries.len() < POPULATION / 2, "operation must not load the dictionary population" )`

### 81. `residency_failure_matrix_tail_recovery_stages_every_consumed_subject_before_effects` ? class1

Source: `crates/worth-relational/src/tests/durability/medium/cold_tail_matrix/exhaustive.rs:4`.

Public recovery/copy original transport, typed ColdStorageUnavailable subject/cause, reservation/frontier observations, provider logs/offers and returned entity member-after value. Producer public admit_field_text/record payload can identify target symbol without private interner lookup.

### 82. `cold_head_warm_and_full_warm_uniqueness_reuse_the_original_paged_field_identity` ? class2

Source: `crates/worth-relational/src/tests/durability/medium/cold_unique_symbols.rs:10`.

Private interner original symbol/absence, direct demand_all_full setup, equality with exact original Symbol and duplicate conflict value; preserve no-effects/public uniqueness too.

`restore from 2a51552fc5:crates/worth-relational/src/tests/durability/medium/cold_unique_symbols.rs`.

Exact private assertions requiring C2:

- crates/worth-relational/src/tests/durability/medium/cold_unique_symbols.rs:52: `assert!(receiving .services .symbols .interner_snapshot() .resolve(symbol) .is_none())`
- crates/worth-relational/src/tests/durability/medium/cold_unique_symbols.rs:88: `assert_eq!( value, worth_foundational::facade::AspectValue::String( worth_foundational::facade::InternedString::Symbol( worth_foundational::facade::Symbol(symbol.0) ) ) )`
- crates/worth-relational/src/tests/durability/medium/cold_unique_symbols.rs:131: `assert_eq!(duplicate, value)`

### 83. `cold_visibility_doors_preserve_storage_failure_and_never_advance_publication` ? class1

Source: `crates/worth-relational/src/tests/durability/medium/cold_visibility.rs:8`.

Public ReadDoor inventory in cold_read_door.rs (re-exported by cold_visibility.rs): projected/snapshot/version/observation reads, inspection and retention. Preserve typed region IntegrityMismatch, sticky provider no-retry and no publication.

### 84. `cold_and_warm_visibility_preserve_an_observation_across_a_later_commit` ? class1

Source: `crates/worth-relational/src/tests/durability/medium/cold_visibility.rs:52`.

Public visible_entity_at_version/read_observation plus later commit; preserve producer-record equality, cold/warm no-reload, retained original record after newer write.

### 85. `discarded_visibility_rounds_settle_only_accepted_materialization_and_leases` ? class2

Source: `crates/worth-relational/src/tests/durability/medium/cold_visibility_round_counters.rs:4`.

Private performance_access reset/counters and exact materialization/lease counts prove discarded rounds settle nothing.

`restore from 2a51552fc5:crates/worth-relational/src/tests/durability/medium/cold_visibility_round_counters.rs`.

Exact private assertions requiring C2:

- crates/worth-relational/src/tests/durability/medium/cold_visibility_round_counters.rs:51: `assert!( warm.visible_authoritative_entity_records_materialized > 0, "the selected paid scan actually settles materialization instrumentation" )`
- crates/worth-relational/src/tests/durability/medium/cold_visibility_round_counters.rs:56: `assert!( warm.adjacency_kind_slices_leased > 0, "the selected traversal actually settles a substrate lease" )`
- crates/worth-relational/src/tests/durability/medium/cold_visibility_round_counters.rs:61: `assert_eq!( initial.visible_authoritative_entity_records_materialized, warm.visible_authoritative_entity_records_materialized, "discarded scan rounds have no materialization effect" )`
- crates/worth-relational/src/tests/durability/medium/cold_visibility_round_counters.rs:66: `assert_eq!( initial.adjacency_kind_slices_leased, warm.adjacency_kind_slices_leased, "discarded adjacency rounds have no lease-ledger effect" )`

### 86. `concurrent_replay_references_share_one_physical_pin_set` ? class2

Source: `crates/worth-relational/src/tests/durability/medium/replay_pin_concurrency.rs:5`.

Private partition.replay_pin_count exact1 then0 establishes one physical pin set under concurrent references; keep public retain/release result assertions.

`restore from 2a51552fc5:crates/worth-relational/src/tests/durability/medium/replay_pin_concurrency.rs`.

Exact private assertions requiring C2:

- crates/worth-relational/src/tests/durability/medium/replay_pin_concurrency.rs:60: `assert_eq!( count(), 1, "concurrent references share exactly one physical pin set" )`
- crates/worth-relational/src/tests/durability/medium/replay_pin_concurrency.rs:80: `assert_eq!(count(), 0)`

## Implementation limits

Every C2 row is a move with original assertion strength retained, not a retirement. Mixed files must split. Mixed parameterized tests must separate public doors from private state/action checks rather than silently reducing their tables. Preserve all original subject/cause classes, sticky/retry behavior, actual successful contents, callback suppression, no-effects, retained-reference equality and independent warm-reference comparisons. Public read-log assertions are not substitutes for exact private dictionary or resident-state assertions.

The JSON companion contains all86 entries and every extracted original test-body assertion, plus selected private helper assertions. It is generated review output under target, not source-controlled certification infrastructure. Support modules follow the claims they serve: shared public setup belongs with external fixtures; private dictionary/arena/node/event/root/cache setup follows C2. Original module mounts can disappear only after their dependent claims are preserved or explicitly moved.

No builds, exact-name discovery, formatting, tests or repository gates were run, per the read-only/no-build assignment. Source inspection does not establish that any proposed migration compiles. The six primary class3 rows remain open until the minimal owner observation or explicit claim split is reviewed; no new sealed door is approved here.
