# Landing 2a Native dead-code and Complete/ACK boundary review

Read-only review, 2026-11-09. Source inspection only; no Cargo, builds, tests,
formatters or source edits. Diagnostic inventory is the complete 32-warning
`bc-final-root-clippy.log` (recorded e7f2490cb1, 2026-11-07). Current source was
inspected separately; that old log is not evidence of the current HEAD's build.
Paths below are relative to `crates/worth-relational/src/` unless stated otherwise.

## Result

The named diagnostics describe orphan production code, not 32 independently
missing callers. No missing necessary production caller was established by this
review. In particular, wiring the orphan original-intent engine into current
Complete capture or journal recovery would add a second execution route to paths
which already have working owners. Delete the orphan members in coherent families,
preserving the live shared code called out below. This is a source/caller finding,
not qualification of deletion or a replacement for the parent's required checks.

The landing instructions name a separate "Complete/ACK contract" gate but do not
identify its concrete defect. The governing contract and current production
ownership are identified below; the label alone does not justify inventing a new
ACK route or weakening a test.

## Nearest live paths

1. Ordinary commits call `authority/mutation/execution.rs:20`
   `apply_plan_to_working_state`, through
   `authority/commit/phases/applied_mutation.rs:115` and
   `authority/commit/pipeline/mutation_execution/aggregate_consequences.rs:60`.
   This calls `dispatch_intent` then `assemble_effect`; the ordinary aspect/field
   writers and canonical deltas remain live. The aggregate path still calls the
   inherent `PendingRecordAllocations::finish_mutation` at line 121.
2. Real journal recovery calls
   `durability/authority/runtime_rebuild/envelope_replay.rs`
   `replay_readmitted_envelope`. It imports canonical symbol DATA, stages recorded
   allocation evidence, and checks staged evidence consumption. Its `effect.rs`
   `replay_ordinary_commit` calls `execute_authoritative_commit` at line 167
   (merge path at 255). It does not call the orphan original-intent executor.
3. Current Complete capture is
   `durability/authority/checkpoint_complete/prepare.rs:11`. It freezes actual
   original inputs and actual held component roots, reads only uncovered canonical
   frames (`source.rs::read_canonical` skips the selected covered prefix), builds
   schemas and roles, and captures those real regions. Region capture calls
   `storage/residency/checkpoint_capture/complete.rs::capture_complete_for_request`
   which either leases Full or invokes ordinary `demand_full`. It then projects
   prefix-free Full DATA through `region_capture/complete.rs`. It does not rebuild
   component state by rerunning the orphan original-intent engine.
4. Selected-image readmission uses
   `branch/root_readmission/checkpoint.rs:7` and
   `RelationalRootRegion::from_checkpoint_for_request`, preserving actual selected
   cells. The ordinary root construction path is `branch/root_capture.rs:51`
   `RelationalRootRegion::new`; incremental publication uses
   `PreparedResidentRegion::incremental`, including the live
   `storage/residency/kind_admission.rs::normalize`.
5. Required output positions are carried by actual roots, exported by
   `checkpoint_complete/admission_projection/root_directory.rs:58`, and readmitted
   by the checkpoint path. `branch/root_capture_preparation.rs:90` carries prior
   positions or explicitly records unresolved provenance; ordinary publication
   binds its actual positions. There is no caller for the alternative
   `RequiredOutputPublicationPositions::from_original_commit` constructor.

## All 32 diagnostics

"Dead family" means no live production or test call chain reaches its root.
"Test-only" identifies direct construction/use by owner unit tests; it does not
mean the corresponding production requirement may be deleted without review.

| # | Diagnostic (log location) | Actual caller / boundary disposition |
|---|---|---|
| 1 | `aspect_versions/original.rs:2 write_original_aspect_version` | Only `workspace/original_revisions.rs:16`, inside the dead original executor family. Remove with that family; preserve ordinary aspect writers. |
| 2 | `canonical_deltas/revision_input.rs:14 write_original_revision_stamps` | Only original executor at `execution.rs:142`. Remove family and its private re-export. |
| 3 | `revision_input.rs:50 evidence` | Only preceding dead revision writer. Remove with it. |
| 4 | `revision_input.rs:103 context` | Only preceding dead revision writer. Remove with it. |
| 5 | `canonical_deltas/patch_authority.rs:164 borrowed_deletion_binding_evidence` | Only `revision_input.rs:61`. Remove this function, retain other borrowed patch evidence used by live canonical delta code. |
| 6 | `execution.rs:113 apply_original_intents_to_working_state` | No caller. Root of the dead alternate original-row execution family. Current Complete and journal routes are paths 2 and 3 above. |
| 7 | `execution.rs:170 original_adjacency` | Only preceding dead executor. Remove with it. |
| 8 | `execution/original_adjacency.rs:7 OriginalAdjacency` | Only preceding dead executor constructs it. Remove module. |
| 9 | `OriginalAdjacency::{as_slice,push}` | Only preceding dead executor uses them. Remove with module. |
| 10 | `field_versions/original.rs:7 write_original_field_versions_for_binding` | Only `workspace/original_revisions.rs:27`. Remove family, keep ordinary and borrowed field comparison/writer code. |
| 11 | `field_versions/original.rs:101 execution` | Error conversion used only within that dead original writer. Remove with it. |
| 12 | `intents/bulk_create_entities/original.rs:4 apply_original` | Only `dispatch_original_intent`. Remove function; **keep `apply_prepared_row` at line 46**, called from live `bulk_create_entities.rs:64`. |
| 13 | `intents/bulk_create_relations/original.rs:3 apply_original` | Only `dispatch_original_intent`. Remove function; **keep `apply_relation_row` at line 36**, called by live ordered/staged adapters in `bulk_create_relations.rs:81,149`. |
| 14 | `intents/delete_entity.rs:25 apply_original` | Only original dispatch. Remove this adapter; preserve ordinary apply and shared delete algorithm. |
| 15 | `intents/dispatch.rs:69 dispatch_original_intent` | Only original executor at `execution.rs:140`. Remove it and its re-export, keep `dispatch_intent`. |
| 16 | `intents/replace_entity.rs:19 apply_original` | Only original dispatch. Remove adapter; preserve ordinary replace and shared allocation/deletion behavior. |
| 17 | `record_changes/entity_deletion.rs:71 delete_entity_with_cascade_original` | Only original delete/replace adapters. Remove this entry and re-export; **keep `input`, `retire`, and ordinary `delete_entity_with_cascade`**. |
| 18 | `workspace/original_revisions.rs:3 write_original_binding_revision_stamps` | Only original revision writer. Remove module and declaration. |
| 19 | `branch/root_region.rs:27 new_for_request` | No callers, including tests. Root of orphan service-free Full root construction chain; real paths use `new`, incremental preparation, and checkpoint region readmission. |
| 20 | `required_output_positions/reconstruction.rs:8 from_original_commit` | No callers, including tests. Remove reconstruction child and declaration; keep the live output position structure, readmission, checkpoint export and ordinary bind operations. |
| 21 | `record_identity/allocation_batch.rs:27 trait finish_mutation` | Only original executor invokes through this trait; cursor tests invoke on cursor. **Keep inherent `PendingRecordAllocations::finish_mutation`**, used by live aggregate commit path and its tests. Preserve live trait `reserve`/`record` unless consciously simplifying their real callers. |
| 22 | `record_identity/original_cursor.rs:18 OriginalRecordAllocationCursor` | Constructors only in `original_cursor/tests.rs`; sealed implementation declaration is not a caller. Test-only alternative input for orphan original executor. |
| 23 | `OriginalRecordAllocationCursor::new` | Only that owner test module, at lines 24,59,92,119,146,161,174. Remove along with the unused cursor and its obsolete tests; do not remove live staged-replay allocation validation. |
| 24 | `symbol_table/mutation_input.rs:32 OriginalSymbolMeanings` | Only constructed by `mutation_input/tests.rs`; no live production constructor. Remove original-only branch; retain live `MutationSymbolMeanings` / `PendingSymbolMeanings` contract and implementations. |
| 25 | `OriginalSymbolMeanings::new` | Only owner test at `mutation_input/tests.rs:14,29`. Test checks dictionary conflict, missing references and no interner mutation on this unused input. Removing it changes no production symbol reservation path. |
| 26 | `working_state/request_mutation.rs:34,50 from_original_root_partitions / into_original_root_partitions` | No callers, including tests. Remove those alternative original-root map conversion methods; preserve all live request-aware mutation/growth methods in the same file. |
| 27 | `partition/sparse_adjacency_table.rs:124 from_entries` | No callers in owner crate, including tests. Remove constructor; live table construction uses default/ensure/request-aware growth and existing `From<Vec<AdjacencySet>>`. Keep `into_entries` and other live table operations. |
| 28 | `residency/cell/resident_full.rs:6 arc_bytes` | Called only by orphan `resident_for_request`; owner tests reach it indirectly. Remove with dead construction chain, not other same-named allocation helpers elsewhere. |
| 29 | `resident_full.rs:16 resident_for_request` | Production caller only orphan `PreparedResidentRegion::initial_for_request`. Direct owner test calls at `resident_full/tests.rs:38,81,106`; that tests allocation refusal/retention of an unused constructor. Existing demand/full materialization paths remain necessary. |
| 30 | `kind_projection/full.rs:69,118,157 normalize / add_owned / install` | `normalize` only from orphan `initial_for_request`; it calls `add_owned`/`install`. Remove those methods, **retain `FullKindProjection`, `rows`, `insert`, `capture`**. `capture` is live at `region_capture/complete.rs:78,80`; ordinary incremental normalization uses the separate real `kind_admission` owner. |
| 31 | `region_publication/full.rs:7 initial_for_request` | Only production caller orphan root constructor (#19). Direct tests: `region_publication/full/tests.rs:106,175`, `checkpoint_capture/complete/tests.rs:112`, and `resident_history_tests.rs:44`. Remove/port these tests coherently with retiring that construction path; preserve public graceful-path assertions through a genuine live producer. |
| 32 | `symbols/data/symbol_text_lookup.rs:7 trait retained_text` | No dispatch through `SymbolTextLookup` uses this method. Remove this trait requirement and the corresponding methods in both implementations (`StringInterner` and `checkpoint_complete/symbols.rs:73 OriginalSymbolDictionary`). **Keep concrete `StringInterner::retained_text`**, used by `commit_texts.rs:21`, `branch/root_readmission/required_text.rs:15`, and other concrete callers. Trait `symbol` and `resolve` and the actual original dictionary remain live. |

## Deletion precautions and proof boundaries

Do not replace these with `cfg(test)` or allow attributes. Tests which solely
exercise a retired implementation do not create a production caller. Tests with
real graceful-path assertions should continue through the real owner paths;
allocation-point/private residency observations follow the explicit C2 ruling.
The parent is separately migrating those tests; this report does not reclassify
their individual claims by filename alone.

Several filenames encode the retired original-input strategy while containing
live row algorithms. Once their dead wrappers are removed, rename/place the
remaining row responsibility honestly. In particular, wholesale deletion of
either bulk `original.rs` file would break ordinary mutation and journal recovery.

The current selected admission cutover explicitly keeps original exports reading
actual frames, while image rows never become fabricated positioned frames
(`plans/database-foundation/milestone-3-durable-owner-contracts.md:3405+`). Keep
that distinction. Wiring the orphan executor or selector reconstruction merely
to silence a lint would need an actual missing requirement and authority handoff;
none was found in this source trace.

## Complete/ACK contract: exact evidence

External orchestration root is
`C:/programming/worth worktees/orchestration/m3`.

* `spec/landing_2a.md:27-33`,
  `prompts/p3_reopen_landing2a_gate.md:24-26`, and
  `prompts/integrate_merge_pc_stacker.md:50` list the held Complete/ACK item.
  They state no particular defect, failing court, changed API or additional
  acceptance requirement for it. `sessions/p3-reopen.r27.jsonl` item 22 records
  the earlier agent asking which finding defines this label. This review did
  not find a more precise ruling in the scoped orchestration Markdown sources.
* Repository `milestone-3-durable-owner-contracts.md:1992-1995`: stopped/refused
  admission retains the original owning Home/source and prior selection; ACK
  continues the same selected attempt and cursor, with no replacement request,
  fresh-source fallback or reconstructed performed proof.
* Same document `:3381-3389`: Home captures actual World then Native under the
  same request; `CapturedHomeRelationalSection` owns the actual image, Complete
  capture and member-selection custody. Its consuming binding frames one final
  package and returns `PreparedCompleteRelationalSection`, accepting no replacement
  Native image. Selection/interrupted ACK retain that same attempt, subject,
  package backing and funding.
* Same document `:3428-3434` names
  `selected_checkpoint_survives_observer_drop_online_history_and_mutation_before_ack_recovery`
  for interrupted ACK / zero Native member reads. The earlier real filesystem
  acceptance list `:2049` names
  `interrupted_owner_ack_resumes_original_attempt_and_reopens_unacknowledged_selection`.
  These are obligations, not new pass evidence from this review.

## Existing Native ownership implementing that contract

* `checkpoint_complete/prepare.rs:163+` moves actual component holds, region cells
  and original read protection into `with_complete_component_holds`; bytes,
  members and publications retain original admitted owners.
* `durability/data/native_checkpoint.rs:53+` clones the shared custody and backing;
  it does not deep-copy/reconstruct capture. Its `acknowledge_selected` at line
  183 delegates to the bound publication, refusing `CaptureNotBound` otherwise.
* `native_checkpoint/publication_binding.rs` binds the exact byte range/member/
  initialization subject and original checkpoint clone to the attached medium.
* `durability/medium/binding/checkpoint_selection.rs:14+` owns that immutable
  subject, weak binding, funded phase and original capture. `advance` at line 91
  queries the actual selected receipt only in `Captured`; ownership moves to
  `Acknowledging` **before** fallible work. The retained phase keeps original
  capture, selected identity, directory admission, next canonical/region cursors,
  and displaced owners. Refusal exits leave that same state in place.
* Only successful `AcknowledgingCheckpoint::advance` releases shared component
  holds. It then becomes `Acknowledged` while retaining the completed owner.
  `native_checkpoint/component_custody.rs` shares a single `Arc<Backing>` across
  checkpoint clones; `release` takes the actual held DATA once and drops outside
  its mutex. It is not tied to the lifetime of one observer handle.
* `runtime/state/subsystems/durability/checkpoint_selection.rs:315,331` separates
  directory replacement from final acknowledged index frontier. A later
  incomplete data pointer does not become proof of an earlier ACK's completion.
* Existing source tests `native_checkpoint/tests/selected_ack.rs:24,119` exercise
  partial transfer and resumed same capture. Component custody tests exercise
  observer drop, retained exact roots, interrupted ACK and abandonment. Some of
  their observations are private/C2 claims; merely inspecting these tests does
  not certify the public Landing 2a gate.

No defect in this existing phase ownership was established by this read-only
caller review. The coordinator's held label remains underspecified beyond the
explicit governing contract and named courts above. Do not report the held item
as implemented merely because the source resembles the requirement; use the
actual final-source acceptance evidence or obtain the specific missing finding.

Validation performed here: source reads and scoped searches only. Cargo/clippy
runs: 0. Focused builds: 0. Test runs: 0.
