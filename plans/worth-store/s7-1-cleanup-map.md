# S.7.1 Cleanup Map — Phase 1 Structural Inventory And Concept Freeze

Milestone: `storage-foundation-s7-1.md`

Produced: Phase 1 implement turn (`worth-store-s7-1-cleanup`)

Status: **Concept frozen; implementation deferred to Phases 2–15**

## Concept freeze

S.7.1 adds **no new blob product capability**. Allowed work in later phases:

- Reorganize existing modules into lifecycle-shaped directories
- Narrow public facades and remove wildcard exports
- Decompose overloaded proof functions into named semantic steps
- Relocate helpers/test support to narrowest responsibility
- Seal or document construction boundaries with compile-fail evidence

**Forbidden without a new milestone:**

- New blob lifecycle phases, authority classes, or public proof nouns
- New Store physical law in certification crates
- New hostile-lane semantics (preserve existing denial cases)

---

## Failure-mode taxonomy

| Code | Mode | Closeout evidence |
|------|------|-------------------|
| T1 | Directory topology / ontology dump | Skeleton doc + root file count scan |
| T2 | Facade leakage / wildcard export | Public API diff; no `pub use foo::*` |
| T3 | Semantic collapse / god function | Named step functions + orchestration ≤25 lines |
| T4 | mod.rs / lib.rs business logic | Aggregation-only lib; logic in lifecycle modules |
| T5 | Helper / test_support misplacement | Topology map: helper → responsibility |
| T6 | Copied-field constructor | Compile-fail for field synthesis |
| T7 | Counter detachment | Counters only in receipt/denial builders |
| T8 | Certification-as-law | Production crates independent of cert vocabulary |
| T9 | Test authority on production facade | `certification-authority` feature gate |
| T10 | Cross-crate field copy | Sealed handoff types at boundaries |

### Evidence posture legend (per finding)

| Posture | Meaning |
|---------|---------|
| `structural-only` | Directory/facade move; close on skeleton diff + review |
| `structural + compile-fail` | Construction or visibility boundary changes; add UI compile-fail |
| `structural + runtime test` | Behavior or hostile lane touched; preserve/add focused test |
| `out-of-scope` | Not blocking S.7.1 or remaining S.7 continuation; no milestone work |

---

## Explicitly out of scope (concept freeze preserved)

| Item | Reason | Owner |
|------|--------|-------|
| Cosmetic renames without authority/facade impact | Surface-only; no auditability gain | — |
| `physical-isolation` existing 19 lifecycle dirs | Topology already teaches responsibility | Phase 12 (cert exports only) |
| Full S.2/S.3 rewrite of buffer-pool or physical-integrity semantics | Spec limits Phase 13 to structural readability | — |
| New blob lifecycle phases or authority classes | Concept freeze | — |
| Line-cap splits on files already under 400 lines with clear ownership | No structural failure mode | — |
| Reopening S.0–S.6 feature semantics | Milestone boundary | — |
| `worth-store-test-support` / `worth-store-physical-certification` crates | Phase 14 scope; referenced from Phase 1 routing only | Phase 14 |

---

## Baseline metrics (2026-07-06)

| Crate | Root `.rs` | Dirs | Total `.rs` | Primary defect |
|-------|-----------|------|-------------|----------------|
| `worth-store-blob-chunks` | 113 | 11 | 204 | T1, T2, T3 |
| `worth-store-certification` | 192 | 8 | 260 | T1, T2, T8 |
| `worth-store-physical-format` | 101 | 1 | 105 | T1, T2, T4 |
| `worth-store-physical-integrity` | 91 | 0 | 91 | T1 |
| `worth-store-buffer-pool` | 64 | 0 | 64 | T1 |
| `worth-store-physical-isolation` | 4 | 19 | 165 | T9 (targeted) |
| `worth-store-recovery-physics` | 22 | 14 | 209 | T1, T3, T6 |
| `worth-store-io-scheduler` | 5 | 8 | 115 | T2, T9 |
| `worth-store-physical-backend` | 7 | 7 | 66 | T1, T6 |

---

## Target directory skeleton (contract for Phases 2–15)

### `worth-store-blob-chunks` → Phase 4–5

```
src/
  lib.rs                    # compile-fail docs + mod only
  exports/lifecycle.rs      # ordered facade (replaces exports.rs + wildcard)
  identity/ integrity/ dedupe/ streaming/ publication/ recovery/
  reachability/ retention_reclaim/ placement/ compaction/ corruption/
  lifecycle/ generation/ milestone_handoff/ test_support/ compile_fail/
```

Root budget after Phase 4: ≤8 files at `src/` root.

### `worth-store-certification` → Phase 2

```
src/
  lib.rs                    # narrow courtroom facade
  courtroom/{scenario,plan,execution,transcript}/
  evidence/by_substrate/ replay/ harness/ closeout/by_milestone/
  residue/ compile_fail/
```

### `worth-store-physical-format` → Phase 3

```
src/
  lib.rs facade/ format_identity/ binary_format/ header/
  page_record/ extent_record/ manifest/ reference/ generation/
  checksum/ payload/ record_framing/ blob_manifest/ offline_verifier/
  security_metadata/ denial/ compile_fail/
```

### `worth-store-physical-integrity` → Phase 13

```
src/
  lib.rs pre_decode/ chunk_integrity/ container_integrity/
  manifest_integrity/ wal_frame_integrity/ damage_classification/
  quarantine/ scrub/ evidence_bundle/ compile_fail/
```

### `worth-store-buffer-pool` → Phase 13

```
src/
  lib.rs admission/ entry/ pin_lifecycle/ resident_frame/
  record_view/ eviction/ dirty_state/ background_envelope/
  speculative_work/ compile_fail/
```

### Lower seams → Phase 12

- **isolation:** keep 19 dirs; gate `*_for_certification_test` under `test_authority/`
- **recovery-physics:** nest provenance `s4_*`/`s5_*` under `milestone_handoff/`
- **io-scheduler:** privatize `background_pacing`/`foreground_reservation` modules
- **physical-backend:** nest `s6_*` under `queue_execution/`; split `blob_observation/`

---

## Inventory: wildcard and broad exports (T2)

| Location | Wildcard / leak | Target | Phase | Evidence posture |
|----------|-----------------|--------|-------|------------------|
| `blob-chunks/src/lib.rs:400` | `pub use exports::*` | `exports/lifecycle.rs` ordered groups | 5 | structural + compile-fail |
| `certification/src/lib.rs` | `binary_format_evidence::*`, `late_milestone_exports::*`, `physical_integrity_closeout_exports::*`, `runtime_verifier_comparison::*`, `s6_facade::*`, `scale_fixture::*`, `scenario_definition::*`, `scenario_plan::*`, `store_json_residue_exports::*` | Named courtroom modules | 2 | structural + compile-fail |
| `certification/src/s6_facade.rs` | Re-exports entire `s6`, `s6_evidence_materialization`, `s6_production_readiness_closeout`, etc. | `courtroom/closeout/s6/` | 2 | structural-only |
| `certification/src/late_milestone_exports.rs` | Chained `pub use crate::s4_*::*`, `s5_*::*`, `s6_facade::*` | Remove; explicit synthetic closeout only | 2 | structural + compile-fail |
| `certification/src/s6.rs:47` | `pub use io_qos_readiness_handoff::*` | Narrow handoff surface | 2 | structural-only |
| `certification/src/lib.rs:298` | `pub use worth_store_readiness::{S2PhysicalSubstrateReadiness, S3PhysicalIntegrityReadiness}` | Move to production crate | 2 | structural + compile-fail |
| `io-scheduler/src/lib.rs` | `pub mod background_pacing`, `pub mod foreground_reservation` | Private modules; export via lib facade | 12 | structural + compile-fail |
| `physical-format/src/lib.rs` | 82 `pub use` lines mirroring flat root | ≤40 grouped lifecycle exports | 3 | structural-only |

---

## Inventory: root-file swamps (T1)

### `worth-store-blob-chunks` (113 root files → Phase 4)

**Ungrouped root modules (move targets):**

| Current root file | Target dir | Phase |
|-------------------|------------|-------|
| `blob_chunk_identity.rs`, `blob_chunk_canonical_basis.rs`, `blob_chunk_scope.rs`, `blob_lifecycle_identity.rs` | `identity/` | 4 |
| `blob_chunk_integrity*.rs` | `integrity/` | 4 |
| `blob_chunk_dedupe*.rs` | `dedupe/` | 4, 6 |
| `blob_streaming_*.rs` (12 modules) | `streaming/` | 4, 8 |
| `blob_lifecycle_*.rs` | `lifecycle/` | 4 |
| `blob_reachability_*.rs` (except existing dirs) | `reachability/` | 4, 9 |
| `blob_placement_proof.rs` | `placement/` | 4 |
| `blob_generation_*.rs` | `generation/` | 4 |
| `s6_background_pressure.rs`, `s6_reclaim_handoff.rs`, `s7_*.rs` | `milestone_handoff/` | 4, 12 |

**Existing partial dirs (keep, complete):** `blob_publication_commit/`, `blob_retention_reclaim/`, `blob_resume_session/`, `blob_recovery_records/`, `blob_compaction/`, `blob_corruption/`, `blob_placement_admission/`, `blob_placement_movement/`.

### `worth-store-certification` (192 root → Phase 2)

All `*_evidence.rs`, `s[0-9]_*.rs`, `*_test_support.rs` at root → `courtroom/`, `evidence/by_substrate/`, or `closeout/by_milestone/`.

### `worth-store-physical-format` (101 root → Phase 3)

Group by artifact family per target skeleton. `facade.rs` (~355 lines) → split logic into `header/`, `page_record/`, etc.; facade aggregates only (T4).

### `worth-store-physical-integrity` (91 root → Phase 13)

Prefix clusters → dirs: `chunk_integrity_*` → `chunk_integrity/`, `container_integrity_*` → `container_integrity/`, `quarantine_*` → `quarantine/`, etc.

### `worth-store-buffer-pool` (64 root → Phase 13)

Prefix clusters → dirs: `resident_frame_*` → `resident_frame/`, `speculative_work_*` → `speculative_work/`, etc.

### `worth-store-recovery-physics` (22 root + 14 dirs → Phase 7, 12)

**Existing dirs (keep, extend):** `partial_publication/`, `checkpoint_cutover/`, `wal_topology/`, `wal_durability/`, `redo_replay/`, `offline_verifier/`, `recovery_budget/`, `recovery_evidence/`, `page_lsn_publication/`, `durable_publication/`, `source_precedence/`, `blob_replay/`, `s4_closeout/`, `security_scope_propagation/`.

**Root files → target (Phase 12):**

| Current root file | Target | Evidence posture |
|-------------------|--------|------------------|
| `s4_integrity_handoff_*.rs`, `s4_recovery_physics_integrity_readiness.rs` | `milestone_handoff/s4_integrity/` | structural-only |
| `s5_publication_recovery.rs` | `publication_recovery/` or `milestone_handoff/s5/` | structural-only |
| `recovery_entry_*.rs` (6 files) | `recovery_entry/` | structural + compile-fail |
| `recovery_integrity_handoff_receipt.rs` | `integrity_handoff/` | structural + compile-fail |
| `recovery_replay_entry_gate.rs`, `replay_receipt.rs` | `replay_entry/` | structural + runtime test |
| `integrity_damage_map.rs`, `integrity_vetted_records.rs`, `integrity_input.rs` | `integrity_vetted/` | structural + compile-fail |
| `recovery_blocking_integrity.rs` | `integrity_vetted/blocking.rs` | structural-only |
| `memory_envelope.rs` | `recovery_budget/` (already related) | structural-only |
| `security_metadata_admission.rs` | `security_scope_propagation/` | structural-only |

**Partial-publication proof-flow inventory (Phase 7):**

| Surface | Failure mode | Evidence posture |
|---------|--------------|------------------|
| `partial_publication/evidence.rs` — multi `from_*` constructors | T6 copied-field authority | structural + compile-fail |
| `partial_publication/replay_read_admission.rs` | T3 god admission + T6 `from_replay_read_artifact` | structural + compile-fail + runtime test |
| `partial_publication/replayed_crash_edge.rs` | T6 witness copy | structural + compile-fail |
| `partial_publication/no_undo_partial.rs` | T6 page mutation copy | structural + compile-fail |
| `partial_publication/before_wal_replay_read.rs` | T6 protected-bytes shortcut | structural + compile-fail |
| `PartialPublicationClassification` (scattered) | T3 state not topologically visible | structural-only (dir already exists) |

**Recovery state graph (target, Phase 7):**

`UnobservedPartial → PersistedCrashEdge → ReplayReadPending → ClassifiedPartial → RecoveredOrDeniedBeforeWal`

**Checkpoint / WAL inventory:**

| Surface | Phase | Evidence posture |
|---------|-------|------------------|
| `checkpoint_cutover/` (existing dir) | 7, 12 | structural-only |
| `wal_topology/replay_cursor_admission.rs` | 7, 12 | structural + compile-fail |
| `redo_replay/admitted_redo_frame.rs::admit` | 7 | structural + runtime test |
| `blob_replay/admission.rs` | 7 | structural + runtime test |

---

## Inventory: helper / test_support placement (T5)

| File | Current home | Target | Phase |
|------|-------------|--------|-------|
| `blob_chunk_test_support.rs` | blob root | `test_support/identity/` | 4, 14 |
| `blob_chunk_physical_test_support.rs` | blob root | `test_support/integrity/` | 4, 14 |
| `blob_publication_commit_test_support.rs` | blob root | `publication/test_support/` | 4, 14 |
| `blob_reachability_hold_test_support.rs` | blob root | `reachability/test_support/` | 4, 14 |
| `blob_retention_reclaim_test_support.rs` | blob root | `retention_reclaim/test_support/` | 4, 14 |
| `blob_generation_registry_test_support.rs` | blob root | `generation/test_support/` | 4, 14 |
| `blob_*/*_test_support.rs` (in dirs) | correct dir | keep under same lifecycle | 14 |
| `certification/src/*_test_support.rs` | cert root | `courtroom/harness/` | 2, 14 |
| `isolation/**/test_authority.rs` | mixed pub | `test_authority/` + feature gate | 12, 14 |
| `io-scheduler/**/test_authority.rs` | pub on lib | feature-gated only | 12, 14 |

---

## Inventory: copied constructors (T6)

| Symbol | Location | Risk | Phase | Evidence posture |
|--------|----------|------|-------|------------------|
| `from_chunk_write_replay`, `from_checksum_admitted_replay`, `from_chunk_tree_node_durable_replay`, `from_root_candidate_replay`, `from_reachability_staged_replay` | `blob_publication_commit/recovery.rs` | Recovery evidence assembly bypass | 7 | structural + compile-fail |
| `from_staged_reachability`, `from_replayable_wal_record` | `blob_publication_commit/wal_record.rs` | WAL authority copy | 7 | structural + compile-fail |
| `from_registry_observation` | `blob_publication_commit/root_candidate.rs` | Root candidate shortcut | 7 | structural + compile-fail |
| `from_published` | `blob_publication_commit/published.rs` | Visibility replay copy | 7 | structural + compile-fail |
| `from_store_backend_residue_scan`, `from_store_backend_manifest_traversal` | `physical-backend` blob_observation | Backend row as authority | 12 | structural + runtime test (preserve hostile) |
| `from_admitted_s6_posture` | `blob_retention_reclaim/candidate.rs` | Isolation field copy | 9, 12 | structural + compile-fail |
| `from_replay_read_artifact`, `from_replay_read_witness` | `recovery-physics/partial_publication/replay_read_admission.rs`, `replayed_crash_edge.rs` | Replay read bypasses classification | 7, 12 | structural + compile-fail |
| `from_persisted_crash_edge`, `from_backend_residue`, `from_unadmitted_durable_page_mutation`, `from_page_flush_recovery_receipt` | `recovery-physics/partial_publication/evidence.rs` | Partial publication evidence copy | 7 | structural + compile-fail |
| `from_integrity_report`, `from_manifest_report`, `from_page_report`, `from_frame_report` | `recovery-physics/integrity_vetted_records.rs` | Integrity report as vetted record | 12 | structural + compile-fail |
| `from_executed_evidence`, `from_quarantine_receipt_evidence` | `recovery-physics/recovery_integrity_handoff_receipt.rs` | Handoff receipt field copy | 12 | structural + compile-fail |
| `from_admitted_resume_session` | `blob_streaming_resume.rs` | Resume posture copy | 8 | structural + compile-fail |
| `from_read_corruption`, `from_detected_source`, `from_streaming_read_request` | `blob_corruption/localization.rs` | Localization without classification | 11 | structural + runtime test |
| `from_reachability_staging`, `from_admitted_shared_dedupe_reference` | `blob_corruption/reference_edges.rs` | Reference edge copy | 11 | structural + compile-fail |
| `from_rewritten_root_and_verified_read` | `blob_compaction/equivalence.rs` | Compaction equivalence copy | 10 | structural + compile-fail |
| `from_streaming_verified_read` | `blob_placement_movement/read_during_move.rs` | Movement read evidence copy | 10 | structural + compile-fail |

---

## Inventory: overloaded god functions (T3)

| Function | File | Lines (approx) | Decomposition | Phase | Evidence posture |
|----------|------|----------------|---------------|-------|------------------|
| `BlobChunkDedupeAdmission::admit` | `blob_chunk_dedupe.rs` | ~120 | `collect_dedupe_evidence` → `classify_dedupe_case` → `verify_dedupe_collision_and_scope` → `construct_dedupe_receipt` / `assemble_dedupe_denial` | 6 | structural + runtime test |
| `BlobStreamingIngest::run_bounded` | `blob_streaming_ingest.rs` | ~80 | `collect_streaming_evidence` → `classify_ingest_frame` → `advance_streaming_frontier` → `verify_chunk_window` → `construct_ingest_receipt` | 8 | structural + runtime test |
| `run_resumable_streaming_ingest` | `blob_streaming_resume.rs` | ~60 | `collect_resume_evidence` → `classify_resume_frontier` → `verify_resume_authority` → `construct_resumed_ingest_receipt` | 8 | structural + runtime test |
| `BlobResumeSession::admit` + `states::admit_chunk_integrity` | `blob_resume_session/` | ~100 | `collect_session_evidence` → `classify_session_state` → `verify_frontier` → `construct_checkpoint` / `stage_reachability` | 8 | structural + runtime test |
| `admit_reachability_orphan`, `admit_abandoned_resume_orphan` | `blob_retention_reclaim/admission.rs` | ~80 each | `classify_reclaim_case` → `verify_reclaim_eligibility` → `construct_reclaim_admission` | 9 | structural + runtime test |
| `BlobPublicationRecoveryEvidence::recover` + `from_*_replay` family | `blob_publication_commit/recovery.rs` | scattered | `classify_recovery_operation` → `verify_recovery_replay_admissible` → `construct_recovery_evidence` | 7 | structural + compile-fail + runtime test |
| `BlobLifecycleLowered::admit_reachability` | `blob_lifecycle_progression.rs` | ~40 | `verify_reachability_digest_match` → `construct_reachability_admitted` | 4, 7 | structural-only |
| `BlobLifecycleReachabilityAdmitted::admit_placement` | `blob_lifecycle_progression.rs` | ~30 | `verify_placement_reachability_basis` → `construct_placement_admitted` | 4, 10 | structural-only |
| `BlobPlacementAdmissionAuthority::admit` | `blob_placement_admission/admission.rs` | ~50 | `collect_placement_evidence` → `classify_placement_case` → `verify_stable_read_basis` → `construct_admitted_placement` | 10 | structural + runtime test |
| `BlobPlacementMovementAuthority::plan_movement` / `execute_with_receipt` | `blob_placement_movement/plan.rs`, `execution.rs` | ~120 | `collect_movement_evidence` → `classify_movement_posture` → `verify_movable_stability` → `execute_physical_movement` → `construct_movement_receipt` | 10 | structural + runtime test |
| `BlobCompactionAuthority::plan_compaction` / `execute_rewrite` / `publish_rewrite` | `blob_compaction/authority.rs` | ~80 | `collect_compaction_evidence` → `classify_compaction_intent` → `verify_interlock` → `construct_rewrite_receipt` | 10 | structural + runtime test |
| `BlobCorruptionGuard::seal` + `localization` paths | `blob_corruption/quarantine.rs`, `localization.rs` | ~80 | `collect_damage_evidence` → `classify_damage_case` (integrity first) → `verify_quarantine_locality` → `construct_quarantine_receipt` | 11 | structural + runtime test |
| `BlobCorruption::admit_derived_rebuild` | `blob_corruption/classification.rs` | ~40 | `classify_rebuild_posture` → `verify_derived_basis` → `construct_rebuild_readiness` | 11 | structural + runtime test |
| `BlobReachabilityRegistry::admit_*` (8 methods) | `blob_reachability_registry.rs` | ~100 | Named edge classifiers per hold type; shared `classify_edge_admission` | 9 | structural + runtime test |
| `RecoveryEntryAdmission::admit` | `recovery-physics/recovery_entry_admission.rs` | ~60 | `collect_entry_evidence` → `classify_entry_case` → `verify_entry_basis` → `construct_admitted_entry` | 7, 12 | structural + compile-fail |
| `PartialPublicationReplayReadAdmission::readmitted_before_wal_append` | `recovery-physics/partial_publication/replay_read_admission.rs` | ~50 | `classify_partial_publication_case` → `verify_replay_read_admissible` → `construct_replay_witness` | 7 | structural + compile-fail + runtime test |
| `RecoveryBudget::admit_recovery` | `recovery-physics/recovery_budget/budget.rs` | ~50 | `classify_recovery_scope` → `verify_bounded_plan` → `construct_bounded_receipt` | 12 | structural-only |
| `PlatformPhysicalFacade` methods | `physical-format/facade.rs` | ~300 | Per-operation: collect → classify → verify → construct report | 3 | structural + runtime test |
| `execute_read_during_compaction_cutover` | `isolation/compaction_interlock/` | decomposed | **Reference pattern** — no change | 12 | out-of-scope |

---

## Inventory: mod.rs / lib.rs logic (T4)

| File | Issue | Target | Phase |
|------|-------|--------|-------|
| `physical-format/src/facade.rs` | Orchestrates append/locate/publish | Delegate to subsystem authorities | 3 |
| `physical-format/src/lib.rs` | 82 re-exports + compile-fail docs | Aggregation only | 3 |
| `blob-chunks/src/lib.rs` | 140+ compile-fail lines + 80 mod declarations | Move compile-fail to `compile_fail/`; lib aggregates | 4, 5 |
| `certification/src/lib.rs` | 260+ lines mixed exports | Courtroom facade only | 2 |

**Clean reference:** `physical-isolation/src/compaction_interlock/mod.rs` — thin re-exports only.

---

## Inventory: certification test authority on production facade (T9)

| Export pattern | Crate | Phase |
|----------------|-------|-------|
| `*_for_certification_test` on `lib.rs` | `physical-isolation` (checkpoint, compaction) | 12 |
| `*_for_certification_test` on `lib.rs` | `io-scheduler` (background_pacing, s6_later_readiness_handoff) | 12 |
| `#[cfg(any(test, feature = "certification-authority"))]` pub use | isolation, backend | 12, 14 |

---

## Proof-flow grammar contract (all blob flows)

```
source_authority
  → collect_*_evidence()
  → classify_*_case()
  → verify_*_transition()
  → construct_*_receipt()      // counters attach here
  → expose_*_capability()
```

| Flow | Owner module (target) | Phase |
|------|----------------------|-------|
| Dedupe | `dedupe/` | 6 |
| Streaming | `streaming/` | 8 |
| Publication | `publication/` + `state_graph.rs` | 7 |
| Recovery replay | `publication/recovery.rs`, `recovery/` | 7 |
| Reachability/reclaim | `reachability/`, `retention_reclaim/` | 9 |
| Placement/movement | `placement/` | 10 |
| Compaction | `compaction/` | 10 |
| Corruption | `corruption/`, integrity `pre_decode/` | 11, 13 |

### Publication state graph (Phase 7)

`PreparedBytes → WalRecorded → ReachabilityStaged → CommittedVisible` with branches `PartialCrash`, `ReplayPending`, `RecoveredOrDenied`.

### Dedupe cases (Phase 6)

`ContentDigestMismatch | MissingEquivalence | PolicyDenied | CanonicalComparisonRequired | ByteComparisonRequired | VerifiedEquivalent | CollisionIndeterminate | ScopeMismatch`

### Damage classification (Phase 11 — preserve)

`DamageClassification` in `physical-integrity/damage_classification.rs` — promote to `damage_classification/` dir; block logical decode first.

---

## Public facade shape (target)

### Blob (`exports/lifecycle.rs` — Phase 5)

Ordered modules: `identity → integrity → dedupe → streaming → publication → recovery → reachability → placement → compaction → corruption → lifecycle`

### Certification (Phase 2)

`courtroom`, `evidence`, `harness`, `replay`, `closeout` — no wildcards; synthetic types prefixed.

### Physical-format (Phase 3)

`facade` + artifact-family modules; ≤40 named exports.

---

## Phase routing summary

| Phase | Primary targets from this map |
|-------|----------------------------|
| 2 | Certification wildcards, readiness re-exports, evidence swamp |
| 3 | physical-format 101-root, facade logic |
| 4 | blob-chunks 113-root → lifecycle tree |
| 5 | `pub use exports::*` → lifecycle facade |
| 6 | `BlobChunkDedupeAdmission::admit` decomposition |
| 7 | publication/recovery `from_*` constructors, state graph |
| 8 | `BlobStreamingIngest::run_bounded` decomposition |
| 9 | reclaim admission + reachability registry admits |
| 10 | placement movement + compaction |
| 11 | corruption + integrity pre-decode |
| 12 | lower seam cert exports, provenance root files |
| 13 | physical-integrity + buffer-pool flat roots |
| 14 | test_support topology across crates |
| 15 | Closeout bundle vs this map |

---

## Preserved behavior (must not regress)

- All existing `lib.rs` compile-fail doc tests in scoped crates
- S.7 hostile lanes: copied counters/digests, certification-only evidence, whole-object materialization, digest-only authority
- Sealed constructors on proof-bearing types
- Lower-crate production law ownership (isolation, recovery-physics, scheduler, backend, integrity)
- Counter semantics on denial paths

---

## Phase 1 closeout evidence

- [x] This cleanup map artifact (`plans/worth-store/s7-1-cleanup-map.md`)
- [x] Findings classified by T1–T10 failure modes
- [x] Per-finding evidence posture (`structural-only` | `structural + compile-fail` | `structural + runtime test` | `out-of-scope`)
- [x] Explicit out-of-scope register
- [x] Phase routing for every critical target
- [x] God-function decomposition specs (dedupe through recovery-physics)
- [x] `worth-store-recovery-physics` root-file, copied-constructor, and partial-publication inventory
- [x] Concept freeze statement
- [x] Target skeletons per crate
- [x] No production code changes (baseline unchanged; later phases implement)

## Repair log (review turn `7d7965024b45`)

| Review finding | Repair |
|----------------|--------|
| Missing evidence posture per finding | Added legend, posture column on T2/T3/T6 tables, out-of-scope register |
| `recovery-physics` under-inventoried | Added root-file routing, partial-publication inventory, state graph, checkpoint/WAL table |
| God-function gaps for Phases 7–11 | Extended T3 table: lifecycle, streaming resume, placement, compaction, corruption, recovery entry |
