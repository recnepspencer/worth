# Merge of P-A (`e4d3b01c8d`) into `codex/m3-host-home` @ `8283728aff`: conflict resolution

Base `2a51552fc5`. 50 conflicted paths resolved and staged. Not committed, no cargo run.

## Per-file decisions

| Path (crates/worth-relational/src unless noted) | Decision | Why |
|---|---|---|
| authority/mutation/intents/bulk_create_entities.rs, bulk_create_relations.rs | HEAD | Same rename on both sides; HEAD names (`row`) kept |
| bulk_create_entities/original.rs, bulk_create_relations/original.rs (DD) | delete | Both sides deleted |
| bulk_create_entities/row.rs + prepared_row.rs | HEAD `row.rs`; `prepared_row.rs` removed | Contents identical |
| bulk_create_relations/row.rs + relation_row.rs | HEAD `row.rs` plus P-A's corrected `.expect` proof comment; `relation_row.rs` removed | Only difference was the comment |
| authority/mutation/intents/delete_entity.rs, replace_entity.rs | P-A shape (no `DeleteEntity` fn-pointer indirection) + HEAD's `context.deferred_publication_edges` argument | Both removed `apply_original`; indirection now has one caller |
| history/data/mod.rs | union of the re-export lists | HEAD's `CanonicalOwnedGraphSelection` + P-A's `RelationalIntroducedSchemaObservation`, `RelationalOriginalInputObservation` |
| runtime/state/subsystems/record_identity/{allocation_batch,mod}.rs | P-A | HEAD kept `original_cursor` only under cfg(test); no remaining user; file already gone |
| runtime/.../symbol_table/mutation_input.rs | P-A | `OriginalSymbolMeanings` has no remaining user (HEAD cfg(test)-gated it) |
| storage/residency/kind_projection/full.rs | HEAD | `FullKindProjection::normalize` is still called by HEAD's production `region_initialization.rs` |
| storage/residency/region_initialization.rs (UD) | kept (HEAD) | Production callers: `branch/root_capture.rs:138`, `branch/root_region.rs:38` (`PreparedResidentRegion::initial(lane, request, ..)`) |
| storage/residency/cell/resident_full.rs (UD) | kept (HEAD), `mod resident_full;` re-added in `cell.rs`, its `mod tests;` dropped | `resident_for_request` is called by `region_initialization.rs:86` |
| storage/residency/cell/resident_full/tests.rs (UD) | delete | Both tests moved out to C2 by P-A (restore row from `f9f15bf8c8`) |
| storage/residency/region_publication.rs | HEAD + P-A's test module, placed at `#[path = "region_publication/full/tests.rs"] mod kind_commitment_preimage_tests;` | HEAD's light `initial` is gone; the cfg(test) `StringInterner` import is dropped because the test imports it |
| storage/residency/region_publication/kind_commitment_preimage_tests.rs (rename vs modify of `full/tests.rs`) | P-A's one preimage test, rewritten to HEAD's API, kept at HEAD path `region_publication/full/tests.rs` with HEAD's `full/tests/kind_oracle.rs` | See "HEAD changes carried" below; the HEAD path keeps its existing src-court baseline row |
| storage/residency/checkpoint_capture/complete/tests.rs, resident_history_tests.rs | HEAD | P-A only re-pointed `initial_for_request` to the light `initial`, which HEAD removed when it unified both into `initial(lane, request, ..)` |
| branch/root_tests/request.rs | P-A (fork B) | HEAD only edited `request_root_refusal_preserves_original..`, which P-A moved out |
| tests/durability/contracts/branch_root_checkpoint.rs | P-A body + HEAD's `mod unsupported_root_version;` | HEAD added that test after base; `foreign_targets` was moved out by P-A |
| tests/durability/contracts/branch_root_content_binding.rs, branch_root_schema_binding.rs | P-A | HEAD changes (root format 4→5, `ReconstructionLane::oracle()`) only touched moved-out tests |
| tests/durability/contracts/derived_index_checkpoint.rs | P-A + `#[path] mod artifact_affinity;` | P-A already carries index-checkpoint format 2→3; other HEAD edits are in moved-out tests |
| derived_index_checkpoint/scoped_affinity.rs | P-A | HEAD edits only in the moved-out digest test |
| derived_index_checkpoint/artifact_affinity.rs (UD) | kept, holding only the 2 tests HEAD added; baseline row restored from HEAD | P-A's 2 tests moved out; HEAD's 2 had no port |
| derived_index_checkpoint/historical_reader.rs, contracts/global_generation_reclamation.rs, contracts/native_checkpoint_restore_benchmark.rs, medium/checkpoint_restart_bound.rs (UD) | delete | Moved out by P-A; HEAD changes were only internal API updates inside them |
| tests/durability/medium/cold_{index_checkpoint_door,index_fixture,index_frontier,isolation,members,merges,read_door,visibility}.rs, cold_read_matrix/exhaustive.rs (UD) | delete; HEAD semantics carried into the cold-residency court (fork A) | Ported by P-A or moved out |
| medium/cold_index_selections.rs, complete_index_selections.rs, cold_read_door/inventory.rs (HEAD-added, children of deleted `cold_members.rs`) | delete after porting | Public halves ported; private halves get C2 rows (8283728aff) |
| tests/history/milestone_7e_phase_a.rs | P-A | HEAD only rewrote a helper used by the moved-out graph half |
| court-relational-certification/.../merge_descriptive_touches.rs (rename vs modify) | P-A port + HEAD's explicit `build_for_basis` index build before forking | Behaviour change HEAD made |
| court-relational-cold-residency/.../merge_index_reads.rs (rename vs modify) | P-A port, renamed to HEAD's `..._native_commit_requires_explicit_index_recovery` with HEAD's refusal + recovery flow | Behaviour change HEAD made |
| court-relational-cold-residency/.../merge_member_matrix.rs (rename vs modify) | P-A port with recipe `indexed = false` (HEAD) | |
| docs/capability-index.md | union: HEAD paragraph, then P-A's two paragraphs | Both inserted at the same point |
| plans/database-foundation/milestone-3.md | HEAD structure; P-A's "Moved courts" C4/C2 blocks moved under HEAD's `### Moved courts` (headings demoted to ####), plus merge rows | HEAD dropped `## Phase map`; both sides' entries kept |
| tools/boundary-check/config/certification-construction-callers.txt | HEAD + P-A's one row (`merge_descriptive_touches.rs`) | Union |
| tools/boundary-check/config/court-frontier-splits.toml | both `[[splits]]` (native-fragments, cold-residency) | Union |

## Edits outside the conflict list
- `src-court-baseline.json`: restored HEAD rows for `derived_index_checkpoint/artifact_affinity.rs` and `region_publication/full/tests.rs`. Both are in the union of the two sides; no new rows.
- `indexes/access/pinning/definition_inventory.rs`: restored HEAD's cfg(test) `definitions_snapshot`. P-A deleted it, but HEAD's `tests/query/indexes/candidate_index_publication.rs:348` still calls it.
- `facade.rs`: auto-merge reached 401 lines; removed one blank line between two `pub mod` items, so it is now 400.
- Cold-residency court files that git auto-merged, edited by fork A: `full_read_inventory.rs`, `full_read_failure_matrix.rs`, `full_visibility_reads.rs`, `merge_source.rs`, `merge_reads.rs`, `index_source.rs`, `mod.rs`, new `index_selection_reads.rs`. `symbol_read_doors.rs` lost one private interner-absence line pulled in from HEAD; that claim is already covered by P-A's C2 row.

## HEAD changes carried into P-A court ports
- `medium/cold_merges.rs` → `cold_residency/merge_source.rs` and `merge_reads.rs`: optional index (`MergeRecipe::new(.., indexed)`), `expected_merge_version`, the 4-argument `assert_merge_records`, and `build_for_basis` before `execute_prepared_merge`.
- `cold_merge_index_matrix/exhaustive.rs` → `merge_index_reads.rs`: refuse with `ColdReconstructionRequired`, then explicit recovery, re-plan and commit.
- `cold_merge_member_matrix/exhaustive.rs` → `merge_member_matrix.rs`: unindexed recipe.
- `medium/cold_index_fixture.rs` (carried variant) → `index_source.rs`: `IndexRecipe::carried()`.
- `medium/cold_read_door.rs` and `cold_read_door/inventory.rs` → `full_read_inventory.rs`: `is_current_relation_scan`, `is_metadata_only_read`.
- `medium/cold_visibility.rs` → `full_visibility_reads.rs`; `cold_read_matrix/exhaustive.rs` → `full_read_failure_matrix.rs`: the metadata-only and relation-scan carve-out.
- `milestone_7d_phase_d/merge_descriptive_touches.rs` → court `merge_descriptive_touches.rs`: index built before the branch.
- `region_publication/full/tests.rs` (P-A's preimage test): HEAD's `initial(lane, request, ..)`, the v3 kind oracle, and `canonical_history`/`capture` with the oracle lane. P-A's v2 rmp preimage no longer matches production.

## Files P-A deleted that were kept
- `storage/residency/region_initialization.rs`: production callers `branch/root_capture.rs`, `branch/root_region.rs`.
- `storage/residency/cell/resident_full.rs`: caller `region_initialization.rs`.
- `FullKindProjection::normalize` in `kind_projection/full.rs`: caller `region_initialization.rs`.
- cfg(test) `IndexAccess::definitions_snapshot`: caller `candidate_index_publication.rs`.

Follow-up for P-A: P-A's orphan analysis predates the master merge. Master made the Full constructor the single production `initial`, so these are live, not orphan.

## Tests HEAD added that were preserved
- `contracts/branch_root_checkpoint/unsupported_root_version.rs`: kept in-source.
- `derived_index_checkpoint/artifact_affinity.rs`: kept in-source with only `digest_valid_join_checkpoint_requires_complete_rows_and_leg_state` and `digest_valid_basis_binding_cannot_carry_an_obsolete_generation_across_an_edit`.
- `cold_checkpoint_preserves_carried_index_selection_and_reads_postings_without_a_full_warmup`, `complete_checkpoint_saves_the_selection_carried_to_its_current_root`, `reclose_before_any_image_demand_keeps_the_selections_the_image_carried`: public halves are in `cold_residency/index_selection_reads.rs`. Private halves are C2 rows restoring from `8283728aff`.

## Uncertain / expected compile risks
- None of this has been compiled. Fork A flagged:
  - `outcome.commit.version_id` read through `CommitResult` deref;
  - `VersionId` coming in via `facade::identity::*`;
  - `read_published_generation_for_observation`;
  - `string_aspect_value` at `aspect_storage::publications`;
  - the court calling `build_for_basis`, `project_observation` and `prepare_entity_field_lookup`.
- Fork B flagged `IndexAuthority::build_for_basis` as the first use of that method from a court crate (it is `pub`).
- Preimage test: HEAD's 2 tests in `region_publication/full/tests.rs` were replaced by P-A's single preimage court, per P-A's C2 restore row. That drops HEAD's content-digest, history-kind and budget assertions on what is now the production constructor. An alternative is to keep HEAD's `controlled_full_region_matches_current_kind_preimages_and_truth` and amend P-A's row. Coordinator decision.
- Kept in-source tests: `unsupported_root_version.rs` roots `ExecutionRequest` but has no src-court baseline row on either side, so BC9002 may report it. This was already true on HEAD. The `artifact_affinity` tests are fault-class claims that P-A would normally send to C4.
- Stale strings, the same on P-A's branch: `performance/data/complexity_contract_inventory/history_capture.rs` names the moved-out `cold_members::checkpoint_restart_bound::..`, and `complexity_contract_inventory.rs:385` names the moved-out `verify_prepared_merge_execution_reports_verification_counters_without_planning_work`.
- Restore rows at `f9f15bf8c8` predate HEAD's later fixture changes: branch-root format 5, index-checkpoint format 3, `ReconstructionLane` arguments, and the unified `initial`. Restorers must apply them.
- Unexpected working-tree dirt appeared during the session and was not touched or staged: `.cargo/config.toml`, `workspaces/worth-query/Cargo.lock`, `worth-query-court-decl-host-authored-ownership/{Cargo.toml,build.rs}`, and untracked `scripts/windows_linker.cmd`.
