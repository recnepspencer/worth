# C.11: Layout, Index, And Native Blob Adoption

## Goal And Decision

Blob bytes, chunk trees, generation publications, B-tree and LSM index nodes
live inside the one reconstructed Store physical platform as ordinary C.5
physical records. Every retained S.7 and S.8 mechanism runs through
`ServingPhysicalRuntime`: the same qualified media, `PhysicalRecordSubmission`
mutation path, WAL/root publication owner, C.9 integrity admission, C.10 root
protection and exact-effect scheduling, and C.8 fresh-process recovery. No blob
or index byte reaches disk through `std::fs`, a test-only writer, an in-memory
format model, or a certification harness.

Native blobs are the priority deliverable. The first working checkpoint is a
blob larger than the residency budget ingested through the production
`PhysicalRecordSubmission` path with a bounded window, published as one
generation, and streamed back byte-exact after a fresh-process reopen. Index
adoption then reuses the exact same record, publication, protection, rebuild
and reclaim contracts; it does not get a second platform.

The decisive proof is the test-case matrix below and its independent
observation, not the existence of family declarations, proof types, or
counters. Architecture is designed backward from the cases; the phase plan
reaches the first real blob journey before any registry, dedupe, LSM or
reclaim machinery exists.

C.11 is physical adoption, not semantics. It never decides blob meaning, record
liveness, tenant or key authorization, retention policy, Query pushdown, or
semantic graph traversal. Physical reclaim executes admitted proofs; it never
infers liveness from its own reachability graph, absence or age.

Governing sources are the
[reconstruction roadmap](physical-foundation-reconstruction-roadmap.md) (C.11
section, Product Decision Lock, Non-Fake Physical Acceptance Test Contract),
[S.7 blobs](storage-foundation-s7.md), [S.7.1](storage-foundation-s7-1.md),
[S.8 layouts](storage-foundation-s8.md),
[S.8 completion](storage-foundation-s8-domain-architecture-completion.md),
[C.9 integrity](physical-reconstruction-c9-integrity-and-offline-truth.md),
[C.10 stable reads](physical-reconstruction-c10-isolation-and-io-coordination.md),
and the runtime-integration roadmap's Milestone 12 as the first downstream
consumer. All eight documents in `../coding_guidelines/` govern this design.
Historical S.7/S.8 simulations and certification harnesses establish nothing
about the production join.

## Decisions Made In Place Of Open Questions

The spec-designer process asks the user where the sources leave a choice open.
No user was reachable, so each choice below is a recorded decision with its
rationale. A reviewer who disagrees changes the decision here before implementation starts.

| # | Decision | Rationale |
| --- | --- | --- |
| D1 | This document is `physical-reconstruction-c11-layout-index-and-native-blob-adoption.md`; the roadmap's spec list is corrected to this name. | The requested name states the native-blob priority; the roadmap listed a shorter placeholder name before the spec existed. |
| D2 | Blob chunk bytes, chunk-tree nodes and generation publications are stored as C.5 extent-backed physical records (`RecordArtifactFile::Extent`/`ExtentManifest`, `PhysicalExtentRecordAuthority`), not a new media file family. | Extents already run through append, WAL, group commit, root publication, `ExtentChunk` integrity, `rewrite_selected_extent_record`, retirement, recovery and offline walk. A packed blob-segment family would be exactly the parallel path C.11 exists to remove. The file-per-extent cost is not accepted; it is repaired in the extent platform itself (D16), so blobs, LSM runs and every large row share the fix. |
| D3 | Chunk identity is SHA-256 over the stored chunk frame; the current 64-bit FNV-1a `ChunkTreeRoot` fold in `worth-store-blob-chunks::chunk_integrity` is demoted to comparison evidence and denied as native authority. | The roadmap requires content-addressed chunk trees; a 64-bit non-cryptographic fold is neither collision-resistant nor content addressing. `sha2` is already a dependency of `worth-store` and `worth-store-blob-chunks`. |
| D4 | Chunking is fixed-size in C.11 with an admitted `BlobChunkSize` of 64 KiB to 256 KiB (default 256 KiB); content-defined chunking is a Part II customization. | S.7 leaves the rule open. Fixed size makes every chunk one extent record with a bounded frame, bounded resident window, and exact counters. Larger sizes exceed the extent payload limit that the durability doc states for `rewrite_selected_extent_record`. |
| D5 | The chunk tree is a real durable tree: leaf nodes list ordered chunk digests plus record identities, interior nodes list child-node digests plus record identities, and the generation root is the SHA-256 of the root node frame. | A flat manifest for a blob above memory does not fit one record and cannot be verified with a bounded window. A tree gives bounded verification, byte-range seeking and localized corruption. |
| D6 | The blob catalog (object identity to published generation) is a derived B-tree index over the authoritative generation-publication records; it is rebuildable and never authority. | This keeps blob identity in WAL-replayable publication records, and makes the first index adoption load-bearing for the blob path instead of a separate demo. |
| D7 | Dedupe scope in C.11 is `SameStoreSameKeyScope` (new; built over `worth_store_security::StoreKeyScope`) only; cross-scope reuse stays a typed denial through the existing `ScopeMismatchCase` law, with no executable branch. | S.7 leaves the first index scope open; tenant and key policy are Part II inputs the Store may enforce but not decide. |
| D8 | `worth-store-blob-chunks` drops its `worth-store` dependency, normal and dev, and every Store-importing file is cut over as enumerated in [Cycle removal](#cycle-removal-blob-chunks-to-worth-store); `worth-store` then depends on `worth-store-blob-chunks`, `worth-store-layout-indexes` and `worth-store-lsm-authority` as downward mechanism crates. | This is the C.10 precedent for `worth-store-physical-isolation`. `cargo tree -i worth-store -e normal` shows the single edge `worth-store-blob-chunks -> worth-store`; `worth-store-layout-indexes` reaches the Store only through it and `worth-store-lsm-authority` not at all, so removing that one edge is sufficient. The dev edge must go too: a dev cycle compiles a second copy of `worth-store-blob-chunks` whose types do not unify with the Store's. Decision Lock 15 keeps the mechanism crates Signal-agnostic; live owners are Store parts. |
| D9 | The B-tree index adopted in C.11 is a new N-ary slotted node format in `worth-store-physical-format` over C.5 inline-segment pages; the `BaselineBTree*` root-plus-two-leaves format and its hard-coded counters are removed. | The baseline cannot hold indexed data larger than memory and its counters are constants, which the roadmap forbids as access receipts. |
| D10 | LSM is adopted as one strategy of the same layout owner: WAL-backed bounded memtable, sorted runs stored as extent records, membership through `worth-store-lsm-authority` records persisted by the Store WAL, compaction as the `CompactionRewrite` scheduler class. LSM ships in Phase 7 after the B-tree, blob and reclaim paths. | The roadmap must-ship names both strategies; ordering the LSM last keeps the blob priority and lets it reuse proven publication and retirement contracts. |
| D11 | Physical reclaim of a published generation consumes a typed `AdmittedBlobReleaseProof` (new, placed in `worth-proof` because it decides legality) whose production issuer is a successor (Part II retention or S.10 repair); C.11 ships the consumer, the independent reclaim of failed-ingest residue proven by resume-session records, and the destroy/rebuild of derived structures. | Roadmap C.11 Must Ship states this split verbatim. A test-only issuer under `certification-test-authority` exercises the consumer; that mirrors C.10's read-protection disposition adapters. |
| D12 | `InstalledCapabilityStatus` becomes truthful: `CapabilityAvailability` gains `Present`, and Layout/Blob report `Present` only when the owning Store parts are constructed. | Today every capability reports `Absent`, including media and page records that C.4 to C.10 installed. Decision Lock 13 forbids a public surface that reports a shell; a permanently false report is the same defect. |
| D13 | Export and import in C.11 are bounded streaming over the same read/ingest sessions with a `BlobExportManifest` record; capsules, replication, and backup holds stay S.10/Part II. | Roadmap must-ship lists export/import with streaming; S.7 says export is not backup correctness. |
| D14 | C.11 builds on the C.10 maintenance code where it actually landed and creates no `physical_runtime/maintenance/` directory. The C.10 spec's destination topology is corrected to the as-built homes in the same change as this spec. | C.10's planned `maintenance/`, `scheduler_admission/rewrite.rs`, `worth-store-physical-format::maintenance_record/` and `worth-store-recovery-physics::maintenance_recovery/` were never created; rewrite and retirement live in `record_serving/publication/director/`, `durability/retention/`, `durability/publication/current_root_owner/`, `worth-store-physical-format::{rewrite_redo, manifest::maintenance}` and `worth-store-recovery-runtime::orchestration::planning::completion::rewrite_*`. Creating the planned directory now would open a second maintenance owner beside the working one. See [C.10 as-built homes](#c10-as-built-homes-c11-extends). |
| D15 | `worth-store-layout-indexes::maintenance::operational_repair` (raw `std::fs` rename and canonicalize) is deleted with its tests, including `LayoutOperationalRepairOwner`, `DerivedIndexRepairReceipt` and `DerivedIndexRepairExecutionDenial`; `worth-store-operations::workflow::repair` keeps its pure plan, lowering and classification law, and its derived-index execution arm returns an operations-owned typed denial (new variant) naming the Store rebuild owner. | `worth-store-operations` does not depend on `worth-store`, so it cannot call `layouts().rebuild()`; keeping a filesystem executor beside the Store rebuild owner is a parallel authority lane. Operator-authorized repair of authoritative bytes is S.10; derived-index rebuild is the Store's (Phase 4). |
| D16 | Extent storage becomes packed, range-allocated **extent arenas**, and this ships as Phase 1, before the first blob. An arena is a large file (admitted `ExtentArenaCapacity`, 64 MiB to 4 GiB, default 1 GiB) holding many aligned extent data and manifest frames. An extent's private placement becomes (arena, offset, length, extent generation). Free space becomes range truth with reuse, published with the root. See [Extent arenas](#extent-arenas). | Today every record at or above the extent threshold (which must be below one page, 16 KiB by default) gets its own data file plus its own manifest file, and free space is a bump allocator (`next_extent`, `first_unallocated`) that never reuses anything. A 4 GiB blob would be about 32 000 files, and so would 16 000 ordinary 20 KiB rows. That is a platform scaling defect, not a blob detail, so it is fixed where it lives. The model is the one used by serious storage engines (copy-on-write range allocation over large files, as in shadow-paging, LMDB's free list and BlueStore's allocator): the file count scales with bytes, not objects; writes are large and aligned; and because no allocation may overwrite a range routed by any protected root, torn writes need no double-write or full-page-image scheme. C.5 already separates stable `PhysicalRecordId` from private placement, so no public identity changes. |
| D17 | The on-disk format version is bumped for arenas; a store in the per-file extent layout is refused at open with a typed format-version denial. No migration or dual-read path exists. | No production store exists in the old layout, and AGENTS.md forbids compatibility surfaces and parallel lanes. |

## Current Boundary And Required Cutovers

The code provides real mechanisms with specific integration gaps:

| Present boundary | C.11 decision |
| --- | --- |
| Every extent-backed record is one `families/records/extents/extent-{id}-{generation}.data` file plus one `extent-manifests/...manifest` file. The free-space header and membership blocks are bump frontiers (`next_segment`, `next_page`, `next_extent`, `next_block`, `first_unallocated`, `unallocated_count`), so freed space is never reused, and retirement means deleting files. | Extent arenas with range allocation, published free-range truth and reuse fenced by C.10 protection (D16, Phase 1). Retiring an extent releases its range; only an evacuated arena is deleted as a file. |
| `worth-store-blob-chunks` never persists bytes. `BlobStreamingChunkWriter` has only test implementations; `BlobBackendChunkWriteSession::store_owned()` is `pub(crate)` dead code; ingest, read, publication and reachability consume caller-supplied witnesses and in-memory `Vec` registries. Its README claims blob bytes live inside the Store. | The Store implements the production chunk writer and observation source. `worth-store/src/physical_runtime/blob/` owns ingest, tree, publication, read, dedupe, reachability and reclaim sessions; the mechanism crate keeps pure proof, sequence and denial law. The README is corrected when the join exists. |
| `ChunkTreeRoot` is an FNV-1a fold rendered as `s7:{lane}:{hash}`; there is no chunk-tree node structure, only `layout_projection::chunk_tree` reports. | New durable `BlobTreeNode` frames in `worth-store-physical-format` with SHA-256 digests. FNV roots may be compared as `s7` lane evidence; `reject_*` denials refuse them as publication authority. |
| `worth-store-physical-format::blob_manifest` has two placeholder rows (`Reachability`, `Placement`) with test-only constructors. `worth-store-physical-backend::placement_observation` has `BlobBackendChunkWriteSession`, residue-scan and manifest-traversal sessions over `Backend` generics with no Store producer. | Replace `blob_manifest` rows with the real `blob_record` family. Cut `placement_observation` sessions over to the Store executor, or remove those with no consumer after the cutover. |
| `worth-store-layout-indexes` reads B-tree pages through `InMemoryPhysicalFormatModel`; `BaselineBTreeExecutionWitness::lookup_counters()` returns constants (`page_touches: 2`, `bytes_read: 8_192`); `LayoutReadRuntime` and degraded scan take `&mut InMemoryPhysicalFormatModel`; `maintenance/operational_repair.rs` renames files with `std::fs`; `read_plan_completion()` is not a Store root lease. | Access execution takes a Store-owned protected page port derived from `records()`; counters come from executed `RecordReadSession` observations; repair goes through `PhysicalRecordSubmission`. The in-memory model stays a format-owner test model, never a runtime. |
| `worth-store-lsm-authority` opens `WalArtifactInventory` and verifies persisted ranges with `std::fs::File::open`; records carry `persisted_path: PathBuf`; nothing writes membership bytes. | Membership records are persisted through the Store WAL payload producer and re-read through the Store's WAL member port. Path fields are replaced by `PhysicalWalMemberIdentity`. |
| `ServingPhysicalRuntime::physical_allocations().admit_blob()` grants a `BlobPhysicalAllocation` memory charge (default scope 256 MiB); `PhysicalCapability::Blob` and `Layout` report `Absent`; no `blob()` or `layout()` accessor exists. | Keep the allocation as the only temporary-byte budget for blob work. Add fallible `blobs()` and `layouts()` capability accessors on `ServingPhysicalRuntime` that acquire protection and allocation through the existing owners. |
| C.10 scheduler vocabulary has `BackgroundPressureKind::{BlobIngestPressure, BlobMigrationPressure, CompactionRewrite, RepairScan}` and `BackgroundDebtKind::BlobContention` with no Store producer; `RetainedBackgroundHeads` retains only checkpoint and reclamation heads. | Add ingest, blob reclaim, index rebuild and LSM compaction producers to `instance/scheduler_admission/`. Extend the retained-head set rather than adding a second dispatcher. |
| `PhysicalRecordSubmission` supports append and record-preserving rewrite (`rewrite_selected_inline_segment`, `rewrite_selected_inline_pages`, `rewrite_selected_extent_record`, capped by `MAXIMUM_EXTENT_REWRITE_BYTES` = 256 KiB); every rewrite preserves every record. `retire_displaced_segment()` retires a generation displaced by rewrite, and `PhysicalCurrentRootOwner` holds exactly one `displaced` slot, so one displaced generation may be outstanding at a time. | Add one record-dropping publication, `drop_reclaimed_records`, whose WAL payload names each dropped record and whose published root unroutes them; dropped extents become displaced and flow into the existing retirement path. The displaced slot becomes a bounded queue (`DisplacedArtifactQueue`, new) with the same claim/complete protocol, because one reclaim batch displaces many extents. No blob-specific deleter. |
| `PhysicalIntegrityArtifactFamily` and `worth-store-physical-integrity::artifact` cover page, extent, WAL, checkpoint, root and free-space families; C.9 reserved additive index/blob siblings. | Add `BlobChunkFrame`, `BlobTreeNode`, `BlobGenerationPublication`, `BlobResumeSession`, `BTreeNode`, `LsmRun` declarations, validators, and offline observer families. |
| `worth-store-contracts::DurableArtifactFamilyId` already names `BlobChunk`, `BlobManifest`, `BlobStream`, `ChunkTreeRoot`, `DedupeIndex`, `ReachabilityEdge`, `RetentionHold`, `ReclaimReceipt`; `worth-store-layout-indexes::artifact_family` has static inventory rows and a crate-private declaration registry. | The Store-owned artifact-family registry is a live instance constructed in `instance/parts.rs`; it consumes the mechanism crate's declaration law and binds each admitted family to its real format, validator, access operations, rebuild basis and retention mechanics. Static rows without an executable owner are removed. |

The ordinary call chain stays the C.10 chain:

```text
Store facade (blobs()/layouts()) and concrete platform admission
  -> Store stable-root / exact-effect / allocation admission
  -> physical Signal dependency readiness
  -> existing I/O scheduler resource admission and selection
  -> Store executor -> qualified filesystem media
  -> exact backend completion -> owning physical settlement
```

Pure digest computation over a resident window, node decoding, key
comparison and plan selection stay direct. A chunk fault, node fault, WAL
append, publication, reclaim or rebuild effect cannot use that exception.

## Mechanism-Crate Cutover Inventory

The three structural defects that block adoption are listed here by exact
file and consumer, so a phase cannot close by moving the defect somewhere
else.

### Cycle removal: blob-chunks to worth-store

`worth-store-blob-chunks/Cargo.toml` names `worth-store` in both
`[dependencies]` and `[dev-dependencies]`; both entries are removed in
Phase 2. Fifteen files import `worth_store::`:

| Files | Store types used | Disposition |
| --- | --- | --- |
| `src/streaming/allocation.rs` | `BlobPhysicalAllocation`, allocation scope | M to `worth-store/.../blob/allocation.rs` |
| `src/streaming/read/{admission,denial,counters}.rs` | `stability::{StablePhysicalReadReceipt, PhysicalReadExecutionDenial, StablePhysicalReadExecutionCounters}` | M to `blob/read_admission.rs`, split by responsibility if it passes 400 lines |
| `src/placement/movement/types/read_hold.rs` | `StablePhysicalReadReceipt` | M to `blob/placement/read_hold.rs` |
| `src/streaming/ingest/orchestration/bounded_ingest.rs` | `BlobPhysicalAllocation` | M to `blob/ingest/session.rs`; the pure frame-sequence and frontier law it calls stays in the mechanism crate |
| `src/streaming/read/orchestration/verify_bounded.rs` | `BlobPhysicalAllocation` | M to `blob/read/verify.rs`; digest-comparison law stays |
| `src/streaming/read/{tests,test_support,pressure_tests}.rs`, `src/streaming/ingest/ingest_tests.rs`, `src/placement/movement/test_support.rs`, `src/test_support/allocation.rs` | certification-only receipts and allocation scopes | Cases that exercise the Store join move to `worth-store/tests/physical_blob_journeys/`; cases that test pure mechanism law are rewritten against mechanism-owned inputs; none keeps a Store-minted certification receipt |
| `src/compile_fail/{placement_movement,construction_boundaries}.rs` (Store-type cases only) | `StablePhysicalReadReceipt`, `RecoveryPhysicalAllocation`, `BlobPhysicalAllocation`, `ServingPhysicalRuntime` | M to `worth-store/tests/physical_runtime_authority/` under the existing `physical_runtime_authority_ui` target |

`cargo tree -i worth-store -e normal` shows that this is the only edge:
`worth-store-layout-indexes` reaches the Store only through
`worth-store-blob-chunks`, and `worth-store-lsm-authority` does not reach it.
The Phase 2 proof is threefold: `cargo tree -p worth-store-blob-chunks -e all
-i worth-store` finds no path, a search for `worth_store::` in the crate is
empty, and the boundary-check DAG snapshot records the inverted edges.

### Constant counters and the baseline B-tree

`BaselineBTreeExecutionWitness::lookup_counters()`
(`worth-store-layout-indexes/src/strategy/btree/execution/witness.rs`) returns
`page_touches: 2` and `bytes_read: 8_192` whatever executed, and
`BaselineBTreeExactCounterWitness` (`counters.rs`) derives its values from the
shape of the request, not from reads. The fix removes the source rather than
correcting the constants:

- Delete the baseline tree: `strategy/btree/execution/` (`witness`,
  `node_codec`, `physical_access`, `read_source`, `counters`, `admission`,
  `lookup/`, `replay_outcome`, `replay_runtime`) and `read/`
  (`LayoutReadRuntime`).
- Counters become Store-owned. `layout/counters.rs` builds the
  `AccessPathCounterSnapshot` only from the
  `StablePhysicalReadExecutionCounters` of the protected reads the lookup
  actually executed (pages read, bytes read, nodes decoded). The mechanism
  crate keeps the `PlannedCounterEnvelope` comparison law, which takes
  observed values and returns a verdict. It exports no type that calls itself
  an execution witness or receipt.
- Controlled defect: a test injects one extra protected page read into a
  point lookup and the reported page touches must rise by exactly one. Point
  lookups over trees of height 1, 2 and 3 must report exactly that height. The
  offline observer recomputes the expected touches from the tree on media.

Consumers are cut over in Phase 4:

| Consumer | Uses | Disposition |
| --- | --- | --- |
| `worth-store-certification` (5 files) | `BaselineBTree{ExecutionDenial, LookupBranch, Range, ReadPreflight, ReadShape, ReadSource}` | Baseline rows deleted; B-tree evidence comes from the Store layout journeys (the crate already depends on `worth-store`) |
| `worth-store-test-support` (2 files) | `BaselineBTree{CorruptionMarker, ExecutionWitness, ReadPreflight, ReadSource}` | Fixtures deleted; corruption fixtures re-authored as on-media `BTreeNode` corruption |
| `worth-store-offline-verifier` (1 file), `worth-store-layout-indexes/src/backup_verification/bounded_index_decode.rs` | `LayoutIndexBackupFormat::BaselineBTree{Leaf,Root}V1` | Variants replaced by `BTreeNodeV1`. No Store ever persisted baseline nodes, so nothing needs migrating. The decoder's read-only `std::fs` access is offline artifact tooling, off the runtime path; S.10 owns its future |
| `worth-store-operations/src/workflow/repair/` (6 files) | `LayoutOperationalRepairOwner`, `InMemoryPhysicalFormatModel` | D15 |
| `worth-store-operations/src/certification_scenario/backup_artifacts*`, `worth-store-claim-boundaries/src/{backend_family,promotion}.rs` | `InMemoryPhysicalFormatModel` | Stays a format-owner model. Phase 4 review confirms that neither cites it as Store runtime or access evidence, and removes any claim that does |

### C.10 as-built homes C.11 extends

C.10's destination topology planned `physical_runtime/maintenance/`,
`instance/scheduler_admission/rewrite.rs`,
`worth-store-physical-format::maintenance_record/` and
`worth-store-recovery-physics::maintenance_recovery/`. None was created. C.11
extends the owners that exist (D14):

| Responsibility | As-built home (`worth-store/src/physical_runtime/` unless stated) | C.11 insertion |
| --- | --- | --- |
| Record-preserving rewrite | `record_serving/publication/director/{selected_segment_rewrite, rewrite_pages, rewrite_span_selection, rewrite_anchor, rewrite_source_liveness, extent_record_rewrite}.rs` | `record_drop.rs` beside them |
| Retirement | `record_serving/publication/director/retirement.rs`, `durability/retention/{retirement, retired_artifact}.rs`, `durability/wal/runtime_owner/retirement_hold.rs` | Consumes queue entries instead of the single slot |
| Displaced slot | `durability/publication/current_root_owner.rs` (`displaced: Mutex<Option<DisplacedArtifact>>`), `current_root_owner/displaced.rs` | `DisplacedArtifactQueue` replaces the `Option` |
| Reopen recharge | `record_serving/admission/{displaced_segments, displaced_extents}.rs` | Recharges dropped-record extents too |
| Scheduling | `instance/scheduler_admission/{reclamation, root_publication, capacity}.rs` | New producers beside them |
| Payload formats | `worth-store-physical-format/src/{rewrite_redo.rs, manifest/maintenance.rs}` | `blob_record/`, drop and LSM payloads beside them |
| Recovery | `recovery_freshness/binding/retirement_obligation.rs`; `worth-store-recovery-runtime/src/orchestration/planning/completion/rewrite_*.rs` | Blob generation and drop obligations beside them |

## Non-Fake Acceptance Setup

### Production subject and roles

The subject is `ServingPhysicalRuntime` with its blob, layout, artifact-family,
publication, scheduler, protection and recovery parts constructed by
`instance/parts.rs`, opened over qualified filesystem media. Certification
composes only public entry points: `blobs()`, `layouts()`, `records()`,
`PhysicalRecordSubmission`, `retire_displaced_segment()`, checkpoints, scrub,
and the C.8 fresh-process recovery entry.

Roles: one writer process (`physical_store_c11_blob_writer` binary, new,
beside `physical_store_c8_writer`) that ingests, indexes, rewrites and is
killed at a chosen seam; one independent observer that walks media offline
through `worth-store-offline-integrity-observer` families and the input
generator's models; one fresh reopen process. The observer never links the
runtime access APIs.

### World and scale axes

- Blob axis: one blob of at least 4 x the admitted Blob allocation window
  (default window 64 MiB against the 256 MiB scope; the heavy lane uses a 4 GiB
  blob against a 64 MiB window) generated from a deterministic seeded pattern
  with repeated 256 KiB regions for dedupe. Sparse or zero-filled sources are
  denied by the generator declaration.
- Index axis: at least 8 x the foreground residency budget of key/value
  entries with a key domain admitted through `AdmittedPhysicalKeyDomain`,
  spread over at least two inline segments, with point, range and prefix
  targets chosen after generation.
- Concurrency axis: one protected reader holding `records()` and one open
  blob read session across rewrite, reclaim and index rebuild.
- Crash axis: the seam matrix below, each seam exercised in a distinct
  process.

### Decisive interleaving

Ingest the blob with a window smaller than the blob; kill the writer after
the frontier passes half the chunks; resume from the durable resume-session
record in a fresh process; finish and publish; ingest the same bytes again and
prove dedupe reuses chunk records within scope; build the blob catalog and
dedupe index; corrupt one chunk frame and one B-tree leaf on media; scrub;
rebuild the leaf from authority; stream the byte range that crosses the
corrupted chunk and receive a localized typed denial, not zeros; present a
release proof for the first generation while the reader session still holds
it; prove reclaim is deferred; release the reader; reclaim; crash between the
drop publication and retirement; reopen fresh; prove the dropped chunks are
neither served nor counted as orphans, and the deduplicated generation still
streams byte-exact.

## Required Test-Case Matrix

| Case | Boundary | Passes only when |
| --- | --- | --- |
| Arena file count | Store placement + media | 16 384 extent records of 256 KiB and 16 384 of 20 KiB produce at most ceil(bytes / arena capacity) + 1 arena files per writer lane, and no per-extent data or manifest file exists on media. |
| Range reuse after retirement | Retirement + free map + protection | A retired extent's range is reallocated only after the releasing root is durable and no protected root routes it; while a reader holds the old root the range is not reused, and its bytes still read exactly. |
| No allocation over a protected range | Allocation owner | An adversarial allocation request that best-fits into a range routed by a held root is refused; the injected defect of ignoring protection is caught by the offline overlap check. |
| Stale bytes in a reused range | Frame validation | A reader routed to a reused range sees only the new extent's frame; a forged route to the old identity or generation is rejected by frame identity, not accepted as data. |
| Arena evacuation | Compaction producer + rewrite + retirement | A sparse arena's live extents move with stable record ids and exact bytes; the empty arena file is deleted through retirement; space amplification stays within the admitted bound. |
| Bounded ingest above window | Store blob owner + executor + media | Peak resident bytes stay under window + 2 node frames + 1 publication frame across the whole ingest; every chunk is one durable extent record; counters equal observed backend writes. |
| Whole-object substitution | Store blob owner | A frame at or above the declared total, a `Vec<u8>` full blob, and a window at or above the object are typed denials before any effect. |
| Interrupted ingest and resume | Writer process + WAL + recovery | Resume continues from the last durable frontier record; readmission re-verifies the last chunk digest; a forged token, changed rule or changed declared total is denied; no chunk is written twice. |
| Abandoned ingest residue | Recovery + reclaim | An ingest with no resume within its declared session limit is classified abandoned by its own session records; its chunks are reclaimed with no external proof; published generations are untouched. |
| Generation publication | Publication owner + WAL + root | `BlobGenerationPublished` exists only after the root advances; a crash before that yields resume or abandon, never a partial generation; the catalog index entry is derived from the publication record. |
| Streaming range read | Protected read session + blob owner | Any byte range returns exact bytes reading only the chunks the range touches plus the node path; read amplification is at most one chunk per side; the session holds root protection for its life. |
| Dedupe honesty | Store dedupe owner | Same bytes in scope reuse chunk records with a byte comparison on the first hit; the same bytes in another key scope are denied for reuse; a forced digest collision with unequal bytes yields `DigestCollisionDenied` and quarantines the digest basis. |
| Derived index destroy/rebuild | Layout owner + rebuild basis | Deleting the blob catalog or dedupe index pages and rebuilding from publication records restores byte-identical lookups with exact rebuild counters; rebuilding from reports, JSON or certification rows is a compile-time or typed denial. |
| Corrupted chunk | Integrity + read | A flipped byte in one chunk frame localizes to that chunk; ranges outside it stream; scrub reports the family and identity; rebuild is denied because the chunk is authoritative. |
| Corrupted derived index | Integrity + layout | A corrupted leaf is rejected as authority; lookups through it are denied, not empty; rebuild from authority restores parity; the acceptance predicate must fail when the corrupted leaf is accepted. |
| B-tree point/range/prefix | Layout owner + protected pages | Results equal the input model; page touches equal height for point, height plus leaves spanned for range; a broad scan behind a point request fails the counter contract. |
| Broad-scan denial | Layout owner | A foreground request whose only admitted shape is a full scan is denied; the verifier and rebuild lanes admit a declared, budgeted scan. |
| LSM lookup and compaction | Layout owner + membership + rewrite | Point/range across memtable and runs equals the model; compaction publishes a new membership only after its runs are durable; stale runs retire through the same retirement path. |
| Reclaim with live reader | Reclaim + protection + retirement | Reclaim of a generation held by a reader is deferred with a typed reason; after release the drop publication is durable, retirement deletes the extents, the retained-byte charge falls, and a second reclaim proves no effect. |
| Reclaim without proof | Reclaim | A published generation with no admitted release proof cannot be reclaimed regardless of reachability, absence of references, or age. |
| Tier movement | Placement + rewrite | Moving a chunk between placements yields a stable read, typed retry or typed denial; no read observes a half-moved chunk. |
| Export and import | Blob owner + streaming | Export streams chunks with a manifest under the window; import re-ingests through the ordinary path and yields a new generation whose root equals the export root. |
| Fresh-process reopen | C.8 recovery | Every case above reopens fresh with the same lookups, digests, counters and orphan sets as the observer computed offline. |

### Crash-seam matrix

Each seam is killed in a distinct process, reopened fresh, and observed
offline. The required fate is exact.

| Seam | Durable at kill | Required fate |
| --- | --- | --- |
| Arena range written, root not published | Arena bytes | Range is free in the published free map; nothing routes it; no residue scan runs; the next allocation may reuse it. |
| Range release in WAL, releasing root not published | WAL retirement intent | The range stays routed-or-held until the root publishes; redo completes the release exactly once. |
| Evacuation copies durable, root not published | Destination ranges | Source arena stays current; destination ranges are free by construction. |
| Arena empty in every root, file deletion partial | Retirement intent | C.10 retirement completion; a missing arena file is a completed deletion, not corruption. |
| Chunk record appended, frontier record not | Extent record | Chunk is resume-verified or abandoned residue; never orphan candidate for external proof. |
| Frontier record durable, next chunk partial | WAL frontier | Resume from frontier; partial extent is failed-op residue reclaimed independently. |
| All chunks durable, tree nodes partial | Chunks + some nodes | Resume rebuilds the missing nodes from chunk records; no re-ingest. |
| Tree root durable, publication WAL not | Nodes | Session resumable; no generation visible. |
| Publication WAL durable, root not advanced | WAL payload | C.8 redo publishes the generation exactly once; catalog index entry appears after rebuild or live maintenance. |
| Drop publication durable, retirement intent not | WAL payload | Records unrouted after reopen; extents displaced; retirement runs later; nothing is served from them. |
| Retirement intent durable, release or deletion partial | WAL + checkpoint | Existing C.10 retirement completion: extent ranges are released exactly once; for an evacuated arena, a missing file is a completed deletion, not corruption. |
| Rebuild partially published | Some index pages | Rebuild candidate is discarded; the previous derived generation or `Absent` posture is reported; no mixed index. |
| Memtable WAL durable, run not sealed | WAL entries | Memtable replays from WAL; unsealed run extent is failed-op residue. |
| Compaction output durable, membership not | Run extents | Old membership stays current; output runs are failed-op residue. |

## Architecture And Authority Lock

### Installed physical authority, branch-agnostic facade, forbidden decisions

Per Decision Lock 17 to 20: the installed physical authority is the Store's
sole publication owner (`PhysicalCurrentRootOwner`) publishing roots that
route chunk, node, publication, index and run records; the branch-agnostic
facade is `ServingPhysicalRuntime::blobs()` and `::layouts()` over physical
identities, digests, keys and generations only; the forbidden semantic
decisions are blob meaning, record liveness, tenant/key authorization,
retention, visibility, and any branch or MVCC registry.

### Extent arenas

Extent arenas replace one file per extent for every extent-backed record, not
only blob records.

- **Arena file.** `families/records/arenas/arena-{id}.data` (new) grows by
  append up to its admitted `ExtentArenaCapacity`. It is placement policy, not
  format compatibility, in the C.5 sense. Frames start on the qualified
  media's alignment unit reported by C.4, so a later direct-I/O backend needs
  no format change.
- **Frames.** An extent's manifest frame and its data chunk frames live in
  the same arena. Each frame carries extent identity, extent generation, frame
  kind, length and checksum, so bytes left in a reused range can never be
  admitted as another extent's frame. Root manifest routing blocks name
  (arena, offset, length, generation); nothing else locates an extent.
- **Free-range truth.** The free-space manifest gains per-arena runs of free
  ranges, sorted and coalesced, published as part of the root. This replaces
  the `next_extent`/`first_unallocated` bump frontier for extents. Segment
  and page frontiers are unchanged.
- **Allocation law.** A new extent takes a best-fit range that is free in the
  current published root and not reserved by an in-flight submission, or else
  appends to the arena that is filling. Every allocation is copy-on-write: it
  can never overlap a range routed by the current root or by any root that
  C.10 protection still holds.
- **Reuse law.** Retiring an extent generation releases its range. The range
  enters the published free map only in a root published after the
  retirement intent is durable, and becomes allocatable only once C.10 shows
  no reader lease or recovery obligation references a root that routes it.
  This is the C.10 retirement protocol with "release range" in place of
  "delete file".
- **Crash law.** A range written but never published is free by construction
  in the published free map, so recovery needs no residue scan for it. A
  reader follows routes only from a published root, and frame identity plus
  generation plus checksum reject stale bytes. Torn writes can only affect
  unpublished ranges, so no double-write buffer or full-page image is needed.
- **Fragmentation and evacuation.** An arena whose live ratio falls below its
  admitted evacuation threshold is evacuated by the `CompactionRewrite`
  producer through `rewrite_selected_extent_record`, which moves live extents
  to other arenas. Once empty in every protected root, the arena file is
  retired as a whole through the existing retirement owner. That is the only
  case in which an extent retirement deletes a file. Space amplification is
  bounded by the evacuation threshold plus one filling arena per writer lane.
- **Integrity and offline walk.** C.9 gains the `ExtentArenaFrame` family and
  validator; the offline observer walks arenas through root routing and the
  free map without runtime APIs, and reports overlapping routes,
  routed-but-free ranges and unaccounted bytes as corruption.
- **Out of scope.** Hole punching (sparse release inside a live arena), raw
  block devices and NUMA-aware arena placement belong to S.12 qualification
  and performance, and must not force a format change.

### Blob record families

All blob families are C.5 extent-backed physical records with a
`worth-store-physical-format::blob_record` frame prefix (new): a kind byte, a
format version, a length, the payload and a SHA-256 over the frame.

- `BlobChunkFrame` (authoritative): the stored chunk bytes for one
  `BlobChunkOrdinal` under one admitted `BlobChunkSize`. Identity is its
  SHA-256 (`StoredChunkDigest`). Record identity binds the digest to one
  `PersistedRecordIdentity`.
- `BlobTreeNode` (authoritative): leaf nodes carry up to 4096 ordered
  (digest, record identity, byte length) entries; interior nodes carry up to
  4096 (child digest, record identity, covered bytes) entries. A node's
  identity is the SHA-256 of its frame. A 4 GiB blob at 256 KiB chunks is
  16 384 chunks, four leaves and one root.
- `BlobGenerationPublication` (authoritative): `BlobObjectId`,
  `BlobGeneration`, root node record identity and digest, total bytes,
  `LogicalContentDigest` (SHA-256 of plaintext), chunking rule version,
  dedupe scope, and the resume-session identity it closes. Its WAL payload is
  `store.physical.blob-generation.v1` (new).
- `BlobResumeSession` (authoritative for its own fate): declaration, admitted
  rule, declared total, frontier ordinal, last durable chunk record identity
  and digest, session limit. WAL payload `store.physical.blob-resume.v1`
  (new). Terminal states are `Published`, `Abandoned`, `Reclaimed`.
- `BlobExportManifest` (authoritative for one export): root digest, chunk
  count and export custody identity. Not a backup artifact.

Derived blob structures: blob catalog (`BlobObjectId` to publication record
identity), dedupe index (`StoredChunkDigest` and scope to chunk record
identity), reachability edge set (generation to chunk and node record
identities, plus resume, export, read-plan and quarantine holds), and orphan
classification. Each is a B-tree family over inline pages with a rebuild
basis of "bounded scan of `BlobGenerationPublication` and `BlobResumeSession`
records". Corruption of any derived family is `DerivedProjectionCorruption`
and rebuilds; corruption of an authoritative family is localized and reported.

### Index families

- `BTreeNode` (derived unless a family is classified authoritative): a slotted
  N-ary node in one C.5 inline page; a separator directory, sibling links,
  occupancy, and a per-node checksum under the C.9 page family. Keys are
  `AdmittedConcretePhysicalKey` bytes under the family's comparator law. Root
  publication is a root-manifest routed record, so the current index root is
  protected by C.10 root protection like any record.
- `LsmRun` (derived): an immutable sorted run as one or more extent records
  with a run header, key range and entry count. The memtable is a bounded
  in-memory sorted buffer whose entries are WAL-durable under
  `store.physical.lsm-memtable-append.v1` (new) before acknowledgment.
  Membership is an `LsmMembershipRecord` persisted under
  `store.physical.lsm-membership.v1` (new) through the Store WAL; replacement
  is a root-changing publication.
- Derived index families rebuild only from a declared physical authority:
  routed records of the indexed family, or for the blob families the
  publication records. `DerivedIndexRebuildSourceInput::{CertificationRows,
  DiagnosticReport, JsonProjection}` become typed denials at the Store boundary.

### Artifact-family registry

`worth-store/src/physical_runtime/artifact_family/` (new) owns one live
registry constructed in `instance/parts.rs`. Each admitted family binds:
physical layout (page or extent, frame kind), source authority
(`Authoritative` or `Derived { rebuild_basis }`), access operations (the
admitted `AccessShape` set), rebuild basis, format version, integrity class
(the C.9 family and validator), physical retention mechanics (retirement
through displaced generations, or drop publication), and recovery
participation (which WAL payloads and redo rules apply). A family absent from
the registry has no access path, no rebuild and no reclaim; an access request
against it is denied with the family identity. The registry is not a
catalog of everything named in `DurableArtifactFamilyId`; families that no
Store owner executes in C.11 are not registered.

### Ingest and publication

Ingest is a `BlobIngestSession` (new, Store-owned) holding a
`BlobPhysicalAllocation`, root protection through `records()`, a
`PhysicalRecordSubmission`, and a scheduler reservation under
`BlobIngestPressure`. Each source frame is at most the window; the session
digests the frame, consults the dedupe index within scope, appends a
`BlobChunkFrame` record (or binds a reused record after byte comparison),
appends leaf entries, and periodically publishes a frontier record. Finishing
seals leaves and interior nodes as records, then publishes the generation
through the sole publication owner as a root-changing member. Memory ceiling:
one window, the open leaf node, one interior node under construction, and one
frame buffer.

Resume opens the session from the last durable `BlobResumeSession` record,
re-reads and re-digests the last durable chunk, and continues. A session that
misses its declared limit is abandoned by the Store, which then reclaims its
records independently because they belong only to a failed physical
operation.

### Read and verification

`BlobReadSession` (new, Store-owned) resolves a generation through the catalog
index, walks the tree nodes with protected reads, and yields chunks in order
for the requested byte range. It holds a `BlobPhysicalAllocation` of one
window and the root protection of its `RecordReadSession`. Verification is
per chunk against the leaf digest; a mismatch is `BlobChunkCorruption` with
ordinal, record identity, and both digests, reported to C.9 disposition.
Whole-object verification is a streaming pass over the same session with the
same window.

### Dedupe, reachability, orphans, reclaim

Dedupe is a derived index consulted during ingest; the first reuse of a digest
performs a byte comparison under a bounded window. Reachability traversal is
a bounded walk of publication records, resume records and holds that yields
per-record `Reachable`, `HeldOnly`, `FailedOperationResidue`,
`DerivedResidue`, or `Unreferenced`. Only the last three classes are orphan
candidates, and only the first two of those are Store-reclaimable without
proof. `Unreferenced` records of a published generation are never reclaimed
without an `AdmittedBlobReleaseProof`.

Reclaim executes as: eligibility (reader and recovery pins through C.10
retention; proof admission), one `drop_reclaimed_records` publication naming
each dropped record under `store.physical.blob-reclaim.v1` (new), then
retirement of the displaced extents through the existing retirement owner.
Effects are exact: dropped record identities, displaced generations, bytes
released, and dedupe entries removed. Reclaim never deletes a file directly.

### Layout access and plans

`layouts()` returns a `PhysicalLayoutAccess` (new) whose `btree(family)` and
`lsm(family)` yield protected index sessions. Plan selection consumes the
existing `AccessPlanSelector` law with the family's admitted shapes and a
`PlannedCounterEnvelope`; execution reports an `AccessPathCounterSnapshot`
from real `RecordReadSession` observations. A shape not admitted for the
family is denied; a hidden broad scan fails the envelope. Index mutation is
copy-on-write: it appends replacement node records for the changed path
through `PhysicalRecordSubmission`, and one root-changing publication both
routes the new path and drops the replaced nodes through
`drop_reclaimed_records`. The replaced nodes then retire through the ordinary
displaced-artifact path. Record-preserving rewrite never changes node
contents; it is used only for placement movement and compaction of unchanged
nodes.

### Scheduler service and interference

New producers in `instance/scheduler_admission/`: `blob_ingest.rs`
(`BlobIngestPressure`), `blob_reclaim.rs` (existing reclamation head),
`index_rebuild.rs` (`RepairScan` class as the rebuild lane),
`lsm_compaction.rs` (`CompactionRewrite`). `RetainedBackgroundHeads` gains
ingest and compaction heads. Every producer lowers an exact effect footprint
(record ranges, extents, index pages, root) through the C.10 algebra and is
subject to the same foreground floor and owed-background-turn bound.

### Lifecycle and outcome topology

Every session (ingest, read, rebuild, reclaim, compaction) carries the
lifecycle of the runtime part that owns it: cancellation before its first
effect is `ProvenNoEffect`; after an effect it settles to an exact fate;
close revokes protection and allocation; shutdown drains through the C.3
sealed lifecycle. Outcomes are typed: `Published`, `Resumable`, `Abandoned`,
`Denied(kind)`, `Deferred(reason)`, `Indeterminate(stage)`.

## Public DX Target

```rust
use worth_store::physical_runtime::{
    BlobChunkSize, BlobIngestDeclaration, BlobIngestOutcome, BlobObjectId,
    BlobReadRange, BlobReclaimRequest, BlobStreamingWindow, IndexFamily,
    PhysicalKeyRange, ServingPhysicalRuntime,
};

fn ingest_and_read(store: &ServingPhysicalRuntime, source: impl BlobFrameSource)
    -> Result<(), PhysicalBlobDenial> {
    let blobs = store.blobs()?;                       // protection + allocation admitted here
    let window = BlobStreamingWindow::bounded(64 << 20)?;
    let declaration = BlobIngestDeclaration::new(
        BlobObjectId::fresh(), BlobChunkSize::KIB_256, source.declared_bytes())?;
    let mut ingest = blobs.begin_ingest(declaration, window)?;
    while let Some(frame) = source.next_frame(window)? {     // frame <= window, never whole object
        ingest.push_frame(frame)?;                            // one chunk record per full chunk
    }
    let published = match ingest.finish()? {
        BlobIngestOutcome::Published(generation) => generation,
        BlobIngestOutcome::Resumable(token) => return Err(PhysicalBlobDenial::Interrupted(token)),
    };

    let mut read = blobs.read(published.generation(), BlobReadRange::bytes(1 << 30, 3 << 20)?, window)?;
    while let Some(chunk) = read.next_chunk()? {              // protected, verified, bounded
        consume(chunk.bytes());
    }
    let receipt = read.finish();                              // exact chunk/byte/node counters
    assert_eq!(receipt.chunks_read(), 12);                    // 3 MiB at a chunk-aligned offset / 256 KiB
    Ok(())
}

fn resume(store: &ServingPhysicalRuntime, token: BlobResumeToken, source: impl BlobFrameSource)
    -> Result<PublishedBlobGeneration, PhysicalBlobDenial> {
    let blobs = store.blobs()?;
    let mut ingest = blobs.resume_ingest(token, BlobStreamingWindow::bounded(64 << 20)?)?;
    source.seek(ingest.frontier().bytes())?;
    while let Some(frame) = source.next_frame(ingest.window())? { ingest.push_frame(frame)?; }
    ingest.finish()?.published().ok_or(PhysicalBlobDenial::StillResumable)
}

fn reclaim(store: &ServingPhysicalRuntime, proof: AdmittedBlobReleaseProof)
    -> Result<BlobReclaimReceipt, PhysicalBlobDenial> {
    store.blobs()?.reclaim(BlobReclaimRequest::released(proof))?.wait()
}

fn lookups(store: &ServingPhysicalRuntime, family: IndexFamily, key: &[u8])
    -> Result<(), PhysicalLayoutDenial> {
    let layouts = store.layouts()?;
    let index = layouts.btree(family)?;                       // denied if family unregistered
    let hit = index.point(key)?;                              // page touches == height
    let range = index.range(PhysicalKeyRange::from(key..), index.budget().pages(64))?;
    for entry in range { let _ = entry?; }
    let rebuilt = layouts.rebuild(family)?.wait()?;           // basis is the registry's, not an argument
    assert!(rebuilt.parity().is_exact());
    Ok(())
}
```

No method takes a `Vec<u8>` for a whole blob, a path, an in-memory model, or a
rebuild source; those are compile-time impossibilities, backed by
`trybuild` cases under the existing `physical_runtime_authority_ui` target.

## Required Destination Topology

Legend: **E** existing owner extended; **N** new populated responsibility;
**M** move/cut over existing responsibility; **R** remove or narrow obsolete
surface; **F** committed future insertion, no empty file now. Paths are
relative to `workspaces/worth-store/crates/`. Every file stays under 400
lines; a listed directory splits by responsibility, never by size alone.

```text
worth-store/Cargo.toml                          E depends on blob-chunks, layout-indexes, lsm-authority
worth-store/src/bin/
  physical_store_c11_blob_writer.rs             N crash-process writer role
worth-store/src/physical_runtime/
  mod.rs                                        E facade exports only
  availability.rs                               E truthful Present/Absent from constructed parts
  instance/
    parts.rs                                    E constructs registry, blob, layout parts
    scheduler_admission/
      blob_ingest.rs, blob_reclaim.rs           N exact producers
      arena_evacuation.rs                       N CompactionRewrite producer for sparse arenas (Phase 1)
      index_rebuild.rs, lsm_compaction.rs       N exact producers
      background_head.rs                        E ingest and compaction heads
  record_serving/planning/
    placement_policy.rs                         E ExtentArenaCapacity, evacuation threshold (Phase 1)
    batch_placement.rs                          E arena range placement instead of extent files
    free_space_projection/, free_space_routing/ E per-arena free-range runs replace the extent bump frontier
  record_serving/arena/                         N arena owner (Phase 1)
    mod.rs, allocation.rs, free_ranges.rs, release.rs, evacuation.rs
  record_serving/access/locate/extent/          E resolve (arena, offset, length, generation)
  durability/retention/retirement.rs            E release range; delete only an evacuated arena
  artifact_family/                              N one live registry
    mod.rs, registry.rs, declaration.rs, admission.rs
    families/physical_record.rs, blob.rs, index.rs
  blob/                                         N Store-owned blob owner
    mod.rs                                      N blobs() facade and denial
    allocation.rs                               M from blob-chunks streaming/allocation.rs
    read_admission.rs                           M from blob-chunks streaming/read/{admission,denial,counters}.rs
    ingest/mod.rs, session.rs, chunk_writer.rs, frontier.rs, resume.rs
    tree/mod.rs, builder.rs, node_publication.rs, walk.rs
    publication/mod.rs, generation.rs, catalog_maintenance.rs
    read/mod.rs, session.rs, range.rs, verify.rs
    dedupe/mod.rs, lookup.rs, collision.rs
    reachability/mod.rs, traversal.rs, classification.rs
    reclaim/mod.rs, eligibility.rs, drop_publication.rs, execution.rs
    placement/mod.rs, movement.rs               N tier movement as rewrite
    transfer/mod.rs, export.rs, import.rs       N bounded export/import
    counters.rs, recovery.rs
  layout/                                       N Store-owned layout owner
    mod.rs                                      N layouts() facade and denial
    page_port.rs                                N protected page reads for index nodes
    btree/mod.rs, lookup.rs, range.rs, prefix.rs, mutation.rs, split.rs, root.rs
    lsm/mod.rs, memtable.rs, run.rs, lookup.rs, membership_port.rs, compaction.rs
    rebuild/mod.rs, basis.rs, execution.rs, parity.rs
    corruption.rs, plan.rs, scan_denial.rs, counters.rs, recovery.rs
  record_serving/publication/director/
    submission.rs                               E drop_reclaimed_records
    record_drop.rs                              N record-dropping root publication
  durability/wal/
    blob_payloads.rs, lsm_payloads.rs           N payload producers for the new families
  durability/publication/
    current_root_owner.rs, current_root_owner/displaced.rs  E single Option slot becomes DisplacedArtifactQueue
  recovery_freshness/binding/
    blob_generation_obligation.rs               N redo obligation for publication payloads
    blob_reclaim_obligation.rs                  N redo obligation for drop payloads
  stability/byte_guard/                         E guard from blob chunk and index page views

worth-store/tests/
  physical_blob_journeys.rs + physical_blob_journeys/   N matrix owner tests
  physical_layout_journeys.rs + physical_layout_journeys/ N matrix owner tests
  physical_runtime_authority/                   E trybuild cases (physical_runtime_authority_ui target) for blob/layout misuse, incl. cases moved from blob-chunks compile_fail

worth-store-physical-format/src/
  extent_record/, manifest/durable_extent.rs   E arena placement and frame identity; format version bump (D17)
  arena_frame/                                  N arena frame header and alignment law
  manifest/physical_free_space_membership_block/, binary_format/free_space_policy.rs  E free-range runs
  integrity_declarations/families/extent_*.rs   E extent frames declared inside arenas
  blob_record/                                  N chunk_frame.rs, tree_node.rs, generation.rs, resume_session.rs, export_manifest.rs
  btree_node/                                   N slotted.rs, separator.rs, sibling.rs
  lsm_run/                                      N header.rs, entries.rs, membership.rs
  integrity_declarations/families/              E blob_*, btree_node, lsm_run declarations
  blob_manifest/                                R placeholder rows replaced
  offline_walk/                                 E blob and index families

worth-store-physical-integrity/src/artifact/
  extent_arena/                                 N arena frame validator (Phase 1)
  free_space/                                   E free-range run validation
  blob_chunk/, blob_tree_node/, blob_generation/, btree_node/, lsm_run/  N validators and validated views

worth-store-offline-integrity-observer/src/integrity_observation/families/
  extent_arena/                                 N overlap, routed-but-free and unaccounted-byte checks
  blob/, index/                                 N independent offline families

worth-store-recovery-physics/src/redo_replay/    E blob/index payload kinds in record.rs and plan/admission.rs typed arms
worth-store-recovery-runtime/src/               E blob/index reconciliation and orphan fate

worth-store-blob-chunks/
  Cargo.toml, src/lib.rs                        E worth-store dependency removed; README corrected
  src/streaming/allocation.rs                   M to Store
  src/streaming/read/{admission,denial,counters}.rs  M to Store
  src/placement/movement/types/read_hold.rs     M to Store
  src/streaming/ingest/orchestration/bounded_ingest.rs  M to Store blob/ingest/session.rs
  src/streaming/read/orchestration/verify_bounded.rs    M to Store blob/read/verify.rs
  test and test_support files importing worth_store  M or R per the cycle-removal table
  src/compile_fail/{placement_movement,construction_boundaries}.rs  E Store-type cases moved out
  src/chunk_integrity/                          E SHA-256 basis; FNV fold as comparison lane only
  src/harness_execution/backend.rs              R fake page receipts
  src/dedupe/*reference_registry*, src/reachability/*registry*  R Vec registries, law stays
  src/backup_verification/                      E artifact format keeps SHA-256 footer

worth-store-layout-indexes/
  src/strategy/btree/execution/{witness,node_codec,physical_access,read_source}.rs  R baseline tree
  src/read/                                     R InMemory-only runtime
  src/access/execution/degraded_scan/           E page-port trait instead of InMemory model
  src/maintenance/operational_repair{,_tests}.rs  R raw fs rename with its receipt and denial types (D15)
  src/backup_verification/bounded_index_decode.rs  E BaselineBTree{Leaf,Root}V1 replaced by BTreeNodeV1
  src/artifact_family/inventory_rows/           R rows without an executable owner
  src/planning, src/keyspace, src/access/shape, src/maintenance/rebuild/parity  E pure law consumed by Store

worth-store-lsm-authority/src/membership/
  model.rs                                      E PhysicalWalMemberIdentity replaces PathBuf
  durable_artifact/record_codec.rs              R std::fs range verification
  runtime/reopen/                               E replay from Store-provided WAL members

worth-store-physical-backend/src/placement_observation/
  chunk_write.rs                                R store_owned dead code; session consumed by Store executor
  manifest_traversal.rs, residue_scan.rs        M or R after Store reachability exists

worth-store-operations/src/workflow/repair/      E derived-index execution arm becomes a typed denial (D15)
worth-store-certification/, worth-store-test-support/  R baseline B-tree rows and fixtures (cutover inventory)
worth-store-offline-verifier/                   E BTreeNodeV1 backup format

tools/boundary-check/config/road1.toml, snapshots/crate-dag.toml  E inverted edges recorded
```

Dependency direction after Phase 2: `worth-store` imports the three mechanism
crates; none of them imports `worth-store`; `worth-store-recovery-runtime`,
`worth-store-offline-verifier` and `worth-store-test-support` keep importing
`worth-store` from above. The boundary-check snapshot is the proof.

## Cost Contracts

| Path | Ordinary cost | Ceiling and scale axis |
| --- | --- | --- |
| Blob ingest | One chunk record write per chunk, one leaf write per 4096 chunks, one WAL frontier per admitted interval (default every 64 chunks), one root publication | Resident bytes <= window + 2 node frames + 1 frame buffer; arena files = ceil(bytes / arena capacity) + 1, independent of chunk count |
| Extent allocation | Best-fit lookup in the published free map, or append to the filling arena | O(log free runs); no media read; free runs per arena bounded by the evacuation threshold |
| Blob range read | Node path (height <= 3 for 2^36 chunks) plus chunks touched | Read amplification <= 1 chunk per side; resident <= window |
| Dedupe hit | 1 B-tree probe plus 1 bounded byte comparison on first reuse | No whole-object comparison; comparison window = chunk size |
| Publication | 1 WAL payload + root member | Payload bytes <= 1 KiB |
| Catalog/dedupe rebuild | Bounded scan of publication and resume records | Pages touched = records / entries per page; resident <= rebuild allocation |
| B-tree point | Height page touches | Height <= 4 for 2^32 entries at fanout >= 256 |
| B-tree range | Height + leaves spanned | Budgeted by caller; envelope violation is denial |
| LSM point | Memtable probe + 1 probe per run in membership | Runs per level bounded by compaction policy |
| Reclaim | 1 drop publication per batch + retirement per displaced extent | Batch <= 1024 records; retained bytes fall by dropped bytes after retirement |
| Reachability | One pass over publication and resume records plus holds | Resident <= verification allocation; never loads chunk bytes |

Every ceiling is an assertion in the matrix, measured from real observations,
never from planned envelopes alone.

## Phase Plan And Fast Feedback

The design is frozen here. Execution first repairs extent storage (Phase 1),
because every later phase writes extents, and then reaches a real blob
journey (Phase 2) before any registry, dedupe, reclaim or LSM work.
No phase creates public operational shells backed by flags, `Absent`,
test-only owners, or copied evidence. Each effect-bearing path ships its
cancellation, close, and recoverable/indeterminate fate with its owning
phase. Later phases strengthen combined evidence.

### Phase 1: Packed extent arenas

Replace one file per extent with extent arenas for every extent-backed record
(D16, D17): the arena frame format and its C.9 family, (arena, offset,
length, generation) placement in root routing, published free-range truth,
copy-on-write best-fit allocation fenced by C.10 protection, range release
through the existing retirement protocol, recovery redo for release, the
offline overlap and accounting walk, and arena evacuation through
`rewrite_selected_extent_record` with whole-arena retirement. The C.10 extent
rewrite, retirement, reopen-recharge and crash tests keep passing unchanged
in intent, with their file-existence assertions rewritten as range and
route assertions.

Closeout gate: the arena rows of the test matrix and the four arena crash
seams pass in distinct processes with offline observation; no per-extent
file exists anywhere on media; the existing C.5, C.8, C.9 and C.10 lanes
(store, recovery, Phase 8 process, isolation, certification) are green on the
arena layout; opening a per-file-extent store is a typed format-version
denial. Proof obligation: the injected defect of allocating over a protected
range fails the offline overlap check.

### Phase 2: Dependency inversion and the first native blob — the working MVP

Invert the crate direction (D8), install `blob_record` frames, the
`BlobChunkFrame`/`BlobTreeNode`/`BlobGenerationPublication` families with
their C.9 declarations and validators, the Store `BlobIngestSession` over
`PhysicalRecordSubmission`, the generation publication payload, and the
`BlobReadSession` over protected reads. No registry, dedupe, resume,
reclaim or index yet; the catalog lookup for this phase is a bounded scan of
publication records admitted as the `Rebuild` lane shape, and it is replaced
in Phase 4. `CapabilityAvailability::Present` (D12) lands here together with
`blobs()`, so the capability status and the accessor become real in the same
change; `layouts()` and Layout `Present` land together in Phase 4.

Closeout gate: a blob of 4 x the window ingests through the production path
with measured resident bytes under the ceiling, publishes exactly once,
survives a fresh-process reopen, and streams a range byte-exact with
counters equal to observed I/O; the whole-object substitutions are typed or
compile-time denials; the three cycle-removal proofs hold (no `cargo tree` path, no `worth_store::` import in
`worth-store-blob-chunks`, boundary-check
DAG snapshot) and every file in the cycle-removal table has its disposition;
the offline observer walks the new families without runtime APIs. Proof obligation:
the controlled defect of hiding a full materialization fails the residency
predicate.

### Phase 3: Interrupted ingest, resume, and independent residue reclaim

Add `BlobResumeSession` records and payload, frontier publication, resume
readmission, session limits, abandonment, and the Store-independent reclaim
of failed-operation residue through `drop_reclaimed_records` plus existing
retirement. This phase installs the record-dropping publication because
abandoned residue is the first legitimate consumer; published generations
are never eligible here.

Closeout gate: the writer process is killed at the first four crash seams;
each reopens to the exact required fate; resumed ingest writes no chunk
twice; abandoned residue is reclaimed with retained bytes falling and no
external proof; forged or mismatched tokens are denied.

### Phase 4: Artifact-family registry and the first derived index

Install the live registry in `instance/parts.rs`, the `BTreeNode` format and
validator, the Store `layout/btree` owner over the protected page port, and
the blob catalog and dedupe indexes as registered derived families with
rebuild basis and live maintenance on publication. Remove the baseline tree
and the in-memory runtime paths from the mechanism crate. Dedupe honesty
(scope, byte comparison, collision denial) ships here because the dedupe
index is its first consumer.

Closeout gate: indexed data above budget answers point/range/prefix equal to
the model with exact counters; unregistered families and hidden broad scans
are denied; deleting and rebuilding the catalog and dedupe indexes restores
parity; repeated content reuses chunk records within scope and is denied
across scope; accepting a corrupted leaf as authority fails the rebuild-basis
predicate; the injected-extra-read controlled defect moves the reported page
touches by exactly one; every consumer row in the constant-counters cutover
table has its disposition, D15 is executed, and a search for `BaselineBTree`,
`LayoutReadRuntime` and `LayoutOperationalRepairOwner` across the workspace is
empty.

### Phase 5: Corruption localization, scrub, and rebuild fallback

Join scrub and resident admission for the new families, localized chunk and
node corruption in reads, derived-index corruption fallback, and the offline
observer's disagreement report.

Closeout gate: one flipped chunk byte localizes to that chunk with ranges
outside it streaming; a corrupted leaf denies lookups rather than returning
empty, rebuilds from authority, and reports exact rebuild counters; scrub
and offline observation agree on family, identity and disposition.

### Phase 6: Proof-consuming reclaim, reachability, and tier movement

Add `AdmittedBlobReleaseProof` consumption, reachability traversal and
classification, reader/recovery-pin deferral, batched drop publications,
placement movement as rewrite, and the reclaim and ingest scheduler
producers' interference evidence.

Closeout gate: reclaim of a held generation defers with a typed reason and
completes after release with exact effects; reclaim without proof is denied
regardless of references or age; a second reclaim is proven no effect; tier
movement never exposes a half-moved chunk; the crash seams for drop and
retirement reopen to their required fates.

### Phase 7: LSM strategy, compaction, and export/import

Install the memtable payload, `LsmRun` records, membership persistence
through the Store WAL, the compaction producer over C.10 rewrite, and
bounded export/import.

Closeout gate: LSM point/range equal the model across memtable and runs;
compaction publishes membership only after runs are durable and retires
stale runs through retirement; the two LSM crash seams reopen correctly;
export streams under the window and import produces a generation with the
same root.

### Phase 8: Full matrix, heavy lane, cutover and successor handoff

Run the decisive interleaving and the 4 GiB heavy lane; confirm
`InstalledCapabilityStatus` reports every constructed family truthfully
(Blob became `Present` in Phase 2 and Layout in Phase 4, with the accessor
that made them real); remove dead placement-observation
sessions, fake harness receipts and static inventory rows; revise the
documentation deliverables; update the roadmap's C.11 entry with the current
contract and the C.12/C.13 handoff.

Closeout gate: every matrix row has direct evidence at its named boundary in
a fresh process; the offline observer reproduces pages, chunks, roots,
reachability and orphan sets; no `std::fs` write or in-memory format model
remains on any production blob or index path (boundary-check denial); all
docs compile their examples.

## Parallel Work And Integration Triggers

| Work | May start | Integrates when |
| --- | --- | --- |
| Arena frame format, free-range runs and offline overlap walk | Immediately | Phase 1 closeout; it gates everything after |
| `blob_record`, `btree_node`, `lsm_run` formats and validators | Immediately | Phase 2 (blob families), Phase 4 (B-tree), Phase 7 (LSM) |
| Offline observer families | After format frames are frozen | Phase 2 closeout |
| Dependency inversion of `worth-store-blob-chunks` | Immediately | Phase 2 closeout; boundary-check snapshot |
| Mechanism-crate cleanup (baseline tree, InMemory runtime, fs repair) | After Phase 2 | Phase 4 closeout |
| Recovery-physics payload kinds | After payload versions are frozen | Phase 3 (resume, reclaim), Phase 7 (LSM) |
| Writer binary and heavy generator | Immediately | First use in Phase 2 |
| Scheduler producers | After Phase 2 | Phase 3 (ingest head), Phase 6, Phase 7 |

## QA Considerations And Verification

Architecture review must confirm one owner per truth: publication owner for
roots, Store blob owner for sessions, registry for family binding, and that no
mechanism crate, fixture, harness or in-memory model can produce a blob or
index byte. Lifecycle review must cover cancellation before and after the
first effect, resume across processes, close during an open read, and
shutdown with an active ingest. Persistence review must cover every crash seam
and the version fields of each new frame and payload. Performance review must
compare measured residency and counters to the ceilings at the heavy scale.
Security review must confirm dedupe scope enforcement and that release proofs
cannot be forged from physical observations. Tests and evidence: owner-local
tests query the actual registry, session tables and obligation index; the
matrix runs in distinct processes with offline observation; the two
controlled defects must fail their predicates. Expensive lanes (heavy blob,
crash matrix) run on the release lane; focused owner tests run on every
change.

## Documentation Deliverables

Implementation revises these against real APIs, compiling every example:

- New `physical-blobs-and-chunk-trees.md` caller doc: `blobs()`, ingest,
  resume, read, verify, export/import, reclaim, denials, counters and
  limits.
- New `physical-layouts-and-indexes.md` caller doc: `layouts()`, registry,
  B-tree and LSM access, rebuild, corruption fallback, scan denial.
- `bounded-physical-record-access.md`: blob and index sessions as successor
  allocation consumers; record-read chunk versus blob chunk vocabulary.
- `physical-durability-and-checkpoints.md`: extent arenas, free-range truth,
  range release and arena evacuation; record-dropping publication, blob and
  LSM payloads, retained-storage effects of reclaim.
- The C.5 record-path spec and `bounded-physical-record-access.md`: extent
  placement is an arena range; per-extent files are gone.
- `physical-recovery-and-reopen.md`: blob generation, resume, drop and
  membership redo; residue fates.
- `physical-integrity-and-offline-verification.md`: new families, derived
  rebuild disposition, offline blob/index walk.
- READMEs of `worth-store-blob-chunks`, `worth-store-layout-indexes`,
  `worth-store-lsm-authority`: downward mechanism posture; corrected claims.
- This roadmap's C.11 entry: current contract links and the exact C.12/C.13
  handoff when implemented.

## Closure And Successor Foresight

C.11 closes when every matrix row has adequate direct evidence at its named
boundary, no known material scoped defect remains, required checks pass, the
sole production path implements the design, and the public docs agree. This
document establishes requirements; it does not assert implementation.

Explicit non-goals, each deferred with its owner:

| Successor | Adds | Must not force a redesign of |
| --- | --- | --- |
| C.12 formal rebinding | Models of ingest/publish/resume/reclaim and index publish/rebuild transitions against the executable owners | Runtime authority; modeled verdicts grant nothing |
| C.13 integration | Joined workload with blobs, indexes, rewrite and reclaim under one scheduler; sealed platform handoff to S.10 | Facade placement, registry ownership, lifecycle composition |
| S.10 backup/repair | Backup holds as reachability edges, capsule and replication artifacts, authorized repair of authoritative chunk corruption | Reclaim proof protocol, drop publication, retirement law |
| Part II semantics | Release-proof issuers, tenant/key scope policy, cross-scope dedupe, content-defined chunking, Query pushdown, semantic traversal | Physical scopes, chunk identity, family registry, branch-agnostic sessions |
| Runtime-integration Milestone 12 | Chunk-backed range and streaming providers over `blobs()` | Constant-memory read contract, counters |

Foresight is paid at the frame formats, payload versions, the registry
binding, and the drop-publication protocol. It is not permission to implement
successors now. The first shipped feedback remains a native blob above the
window ingested, published and streamed through the production path.
