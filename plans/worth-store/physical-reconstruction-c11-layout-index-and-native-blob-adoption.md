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
consumer. All eight documents in `../../docs/coding-guidelines/` govern this design.
Historical S.7/S.8 simulations and certification harnesses establish nothing
about the production join.

## Store Format Policy: No Historical Compatibility

Store has never been deployed, and its existing development data is disposable.
C.11 therefore supports only the current Store artifact and protocol grammars;
it has no obligation to reopen data from abandoned development builds. This
decision governs Store and its direct writer, validator, recovery, rejoin and
offline-observation consumers, not unrelated platform workspaces.

For each artifact or protocol family, declare one supported current grammar
with explicit typed operation and state variants. Keep stable identity and
version tags so stale or unsupported bytes cannot be mistaken for current
data. Change the identity/version when incompatible encoding changes require
it; do not reinterpret old bytes or retain an old reader merely because a
version was bumped. Historical read/write windows, dual-read or dual-write
lanes, old-format migrations, downgrade conversions, rolling-upgrade support,
legacy aliases and compatibility-only registries or harnesses are not C.11
deliverables and must not survive solely to preserve development history.

Wire identity belongs exclusively to the Format codec, never to a mutation
instance or caller-selected operation. The recovery projection stores one
`PersistedPhysicalRecoveryOperation` sum: session and reuse operations carry
their exact binding; `RecordsDropped` alone can carry a release-head effect;
`DerivedDirectory` alone can carry a retirement. These attachments retain their
lawful present/absent states, and unavailable source facts remain explicit.
The public projection exposes the complete operation, not parallel semantic
and attachment fields. Checked construction validates identities, generations,
digests and resource limits; an operation value does not grant recovery or
publication authority. `Frames` and `SourceCopy` remain distinct payloads.

The current-only projection codec writes one fixed identity unconditionally and
admits only that identity before decoding its body. There is no per-instance
format selector or version-conditioned operation dispatch. Rust construction,
visibility and exhaustive operation matching enforce the public contract;
the existing boundary checker additionally inspects the producer/decoder AST,
the single supported-domain declaration and the projection module graph to
reject a second grammar, a version selector or a domain-selection branch.
Unsupported-version diagnostics may parse a rejected numeric identity, but
that rejection path may not select an operation or enter a historical reader.
Compiler-negative examples protect attachment/type construction; checker
negative cases protect the private wire boundary. Current publication,
checkpoint, fresh C.8 recovery, independent Store rejoin and Serving remain
the integration acceptance, alongside malformed-current-operation refusals.

This is not permission to delete live operation semantics. `Frames` and
`SourceCopy`, classified or unavailable source facts, `NoRelease`, active and
terminal release custody, session/frontier transitions, tier certificates and
head retirement retain their distinct authority and lifecycle meaning in the
current grammar. An older numeric tag still emitted by a current writer is a
coordinated cutover target, not evidence that its operation can be deleted.
Likewise, an older checkpoint or WAL member within a current-format journey is
current recovery history, not historical software compatibility.

A cutover updates the production writer, physical format, WAL/root publication,
C.9 integrity admission, C.8 recovery, independent Store rejoin and independent
offline parsers together at the affected family boundary. Remove the replaced
production encoding/reader and its compatibility-only fixtures in that slice;
do not leave parallel lanes or introduce a migration framework. Rebuild junk
fixtures through the current production format. Existing canonical current
encoding, hash and authority tests remain valuable; only promises to admit an
abandoned format are retired. Preserve accepted evidence whose seam is unchanged.

Unsupported historical identities/versions produce a typed unsupported-format
denial before mutation, recovery promotion or Serving. Opening such a namespace
must not silently migrate, erase or reinitialize it. Creating a fresh disposable
namespace is an explicit development action, not a recovery fallback. Current
format corruption, partial writes, cancellation, checkpoint replacement, lawful
WAL pruning and fresh-process recovery still require their full existing fates,
integrity checks, resource budgets and authority provenance. The decisive
pruned A/B continuation journey below is unchanged.

Before accepting a format cutover, independent review must see a genuine
current-format positive publication/checkpoint/recovery/Serving journey and
focused unsupported-version negative evidence at the affected admission
boundaries. A missing head or altered control in the current grammar must still
deny; a version rejection alone cannot substitute for current-format recovery
or structural integrity evidence. There is no requirement for positive recovery
from historical golden artifacts or coexistence with an abandoned writer.

Architecture 21 and DX 7 explicitly permit this undeployed, disposable,
current-only Store baseline while retaining identity, typed unsupported-version
rejection and current-format recoverability. All other governing constraints
remain binding; this is not a general compatibility waiver. Future deployment
or an actual retained-data commitment requires
a new explicit compatibility decision; speculative upgrade machinery is not
prebuilt here.

## Decisions Made In Place Of Open Questions

The spec-designer process asks the user where the sources leave a choice open.
No user was reachable, so each choice below is a recorded decision with its
rationale. A reviewer who disagrees changes the decision here before implementation starts.

| # | Decision | Rationale |
| --- | --- | --- |
| D1 | This document is `physical-reconstruction-c11-layout-index-and-native-blob-adoption.md`; the roadmap's spec list is corrected to this name. | The requested name states the native-blob priority; the roadmap listed a shorter placeholder name before the spec existed. |
| D2 | Blob chunk bytes, chunk-tree nodes and generation publications are stored as C.5 extent-backed physical records (`RecordArtifactFile::Extent`/`ExtentManifest`, `PhysicalExtentRecordAuthority`), not a new media file family. | Extents already run through append, WAL, group commit, root publication, `ExtentChunk` integrity, `rewrite_selected_extent_record`, retirement, recovery and offline walk. A packed blob-segment family would be exactly the parallel path C.11 exists to remove. The file-per-extent cost is not accepted; it is repaired in the extent platform itself (D16), so blobs, LSM runs and every large row share the fix. |
| D3 | `StoredChunkDigest` is SHA-256 over a versioned canonical stored-content subframe (chunk rule and bytes), excluding session, object, ordinal, placement and RecordId. Every newly written prepublication chunk or tree-node C.5 record also carries a separately authenticated occurrence claim: declared session, kind, ordinal or level/index, length and canonical digest; the selected C.5 route binds it to record identity. The current 64-bit FNV-1a `ChunkTreeRoot` fold in `worth-store-blob-chunks::chunk_integrity` is comparison evidence only. | Content identity remains stable across occurrences for Phase 4 dedupe, while claims make both chunk and partial-tree custody decidable after WAL truncation. A 64-bit non-cryptographic fold cannot be native content authority. |
| D4 | Chunking is fixed-size in C.11 with an admitted `BlobChunkSize` of 64 KiB to 256 KiB (default 256 KiB); content-defined chunking is a Part II customization. | S.7 leaves the rule open. Fixed size gives bounded frames and exact counters. A 256 KiB chunk plus its frame overhead exceeds the ordinary 256 KiB record-preserving rewrite ceiling, so movement uses the bounded C.10 SourceCopy lane, not `rewrite_selected_extent_record`. |
| D5 | The chunk tree is a real durable tree: leaf nodes list ordered chunk digests plus record identities, interior nodes list child-node digests plus record identities, and the generation root is the SHA-256 of the root node frame. `ChunkTreeRoot` is Store-local physical-layout identity; `LogicalContentDigest` is portable plaintext identity. | A flat manifest for a blob above memory does not fit one record and cannot be verified with a bounded window. Record identities legitimately change after cross-Store import, so its physical tree root is not a portable equality claim. |
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
| D18 | Every C.11 Store artifact/protocol family is current-format-only under the policy above. Abandoned development formats and compatibility-only production, API and test paths are removed; current writers and all affected consumers cut over together. | Store is undeployed and existing data is disposable. A version tag prevents reinterpretation; it does not create a historical-data support promise. |

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
| C.10 scheduler vocabulary has `BackgroundPressureKind::{BlobIngestPressure, BlobMigrationPressure, CompactionRewrite, RepairScan}` and `BackgroundDebtKind::BlobContention` with no Store producer; `RetainedBackgroundHeads` retains only checkpoint and reclamation heads. | Add ingest, blob reclaim, index rebuild and LSM compaction producers to `instance/scheduler_admission/`. Extend the retained-head set for retry-owned compaction, while synchronous per-frame ingest and reclaim release their dispatch attempts on denial. |
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
| `src/placement/movement/types/read_hold.rs` | `StablePhysicalReadReceipt` | D in Phase 2 with the obsolete mechanism-owned execution/read-hold lane. Phase 6 introduces the real Store-owned hold under `blob/placement/` when movement executes through the runtime. |
| `src/streaming/ingest/orchestration/bounded_ingest.rs` | `BlobPhysicalAllocation` | M to `blob/ingest/session.rs`; the pure frame-sequence and frontier law it calls stays in the mechanism crate |
| `src/streaming/read/orchestration/verify_bounded.rs` | `BlobPhysicalAllocation` | M to `blob/read/verify.rs`; digest-comparison law stays |
| `src/streaming/read/{tests,test_support,pressure_tests}.rs`, `src/streaming/ingest/ingest_tests.rs`, `src/placement/movement/test_support.rs`, `src/test_support/allocation.rs` | certification-only receipts and allocation scopes | Cases that exercise the Store join move to `worth-store/tests/physical_blob_journeys/`; cases that test pure mechanism law are rewritten against mechanism-owned inputs; none keeps a Store-minted certification receipt |
| `src/compile_fail/construction_boundaries.rs` (Store ingest-type cases only) | `RecoveryPhysicalAllocation`, `BlobPhysicalAllocation`, `ServingPhysicalRuntime` | M to `worth-store/tests/physical_runtime_authority/` under the existing `physical_runtime_authority_ui` target, with a valid ingest counterpart and intended allocation/lifetime failures. |
| `src/compile_fail/placement_movement.rs` (Store-type cases only) | `StablePhysicalReadReceipt` as a substitute for movement execution or a movement read hold | D in Phase 2 with the obsolete target types and execution adapters; retain pure planning-law tests. Phase 6 must prove these non-substitution boundaries against its actual Store-owned movement and hold types in `physical_runtime_authority_ui`. |

Movement planning remains a downward mechanism in Phase 2, not a public
execution shell. A completed lower read plan plus a migration interlock may
support planning, but cannot execute or publish movement. Tests against removed
execution types do not migrate as missing-type or nonexistent-method failures:
those would prove no live authority boundary. The Phase 6 Store join owns their
replacement compiler evidence; this disposition does not remove that obligation.

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
  and strictly larger than the full `W + 5 MiB` ingest residency ceiling
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
record in a fresh process; finish and publish; build the blob catalog and
dedupe index from the authoritative publication and tree. Ingest the same
bytes again and prove that live dedupe lookup reuses chunk records within
scope. Add a third generation with one unique probe chunk; corrupt that
unique chunk frame and one B-tree leaf on media; scrub; rebuild only the
derived leaf from intact publication authority. A range crossing the probe
chunk gets a localized typed denial, not zeros, while both identical
deduplicated generations remain byte-exact; a raw outer-CRC failure cannot
fabricate an observed inner digest. Present a release proof for the first
generation while the reader session still holds it; prove reclaim is
deferred; release the reader; reclaim; crash between drop publication and
retirement; reopen fresh; prove dropped records are neither served nor
counted as orphans, and the surviving deduplicated generation still streams
byte-exact. Corrupting a shared chunk instead must deny both generations,
never be described as healed by a derived-index rebuild.

## Required Test-Case Matrix

| Case | Boundary | Passes only when |
| --- | --- | --- |
| Arena file count | Store placement + media | 16 384 extent records of 256 KiB and 16 384 of 20 KiB produce at most ceil(bytes / arena capacity) + 1 arena files per writer lane, and no per-extent data or manifest file exists on media. |
| Range reuse after retirement | Retirement + free map + protection | A retired extent's range is reallocated only after the releasing root is durable and no protected root routes it; while a reader holds the old root the range is not reused, and its bytes still read exactly. |
| No allocation over a protected range | Allocation owner | An adversarial allocation request that best-fits into a range routed by a held root is refused; the injected defect of ignoring protection is caught by the offline overlap check. |
| Stale bytes in a reused range | Frame validation | A reader routed to a reused range sees only the new extent's frame; a forged route to the old identity or generation is rejected by frame identity, not accepted as data. |
| Arena evacuation | Compaction producer + rewrite + retirement | A sparse arena's live extents move with stable record ids and exact bytes; the empty arena file is deleted through retirement; space amplification stays within the admitted bound. |
| Bounded ingest above window | Store blob owner + executor + media | Peak charged resident bytes stay under the single ingest ceiling defined below, including source window, pending C.5 frame/redo, node frames, publication frame and scheduler head; every newly written chunk is one durable extent record; counters equal observed backend writes. |
| Whole-object substitution | Store blob owner | A declaration or window at or above the object is denied before declaration effects. A whole-object frame or full-blob `Vec<u8>` supplied after `begin_ingest` is denied before any chunk effect; the already published declaration remains identifiable retained unfinished work, never misreported as `ProvenNoEffect`. |
| Interrupted ingest and resume | Writer process + WAL + recovery | Resume continues from the last durable frontier record; readmission re-verifies the last chunk digest; a forged token, changed rule or changed declared total is denied; no chunk is written twice. |
| Abandoned ingest residue | Recovery + reclaim | A completed durable checkpoint crossing the declaration's maximum checkpoint sequence, or explicit durable abort, establishes abandonment only when no publication or protected hold wins. Phase 3 then reclaims records proven exclusive to that failed operation without external semantic proof; shared and published records remain untouched. |
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
| Export and import | Blob owner + streaming | Export streams chunks with a manifest under the window; import re-ingests through the ordinary path and yields a new generation with byte-exact content and the same portable `LogicalContentDigest`. The importing Store publishes its own physical `ChunkTreeRoot`; equality with the export Store's root is not required. |
| Fresh-process reopen | C.8 recovery | Every case above reopens fresh with the same lookups, digests, counters and orphan sets as the observer computed offline. |

### Crash-seam matrix

Each seam is killed in a distinct process, reopened fresh, and observed
offline. The required fate is exact.

| Seam | Durable at kill | Required fate |
| --- | --- | --- |
| Managed arena append data settled, root not published | The append WAL member and barrier are durable before the arena frames are written | Kill after data settlement and before root publication. The pre-reopen offline walk may classify the unrooted arena file `Unknown`, never `Intact` or a published route. C.8 fresh-process redo publishes the exact record and route once; the range is not free or reusable, and the post-redo offline walk finds no competing route/free claim. No residue scan substitutes for the WAL authority. |
| Range release in WAL, releasing root not published | WAL retirement intent | The range stays routed-or-held until the root publishes; redo completes the release exactly once. |
| Evacuation copies durable, root not published | Copy intent and destination ranges, but no final copy publication | Source arena stays current and destination ranges are unrouted in the published root. Fresh-process recovery restores the exact private destination claim, preventing ordinary reuse until durable cancellation or publication resolves the intent. |
| Arena empty in every root, file deletion partial | Retirement intent | C.10 retirement completion; a missing arena file is a completed deletion, not corruption. |
| Session declaration published, chunk record appended, frontier record not | Declared session plus authenticated chunk occurrence claim in a C.5 record | Phase 2 retains identifiable unfinished-operation custody; Phase 3 resume verifies and reuses the exact selected occurrence or independently reclaims abandoned residue. A valid chunk digest without a matching occurrence claim is not session authority. |
| Frontier record durable, next chunk partial | WAL frontier | Resume from frontier; partial extent is failed-op residue reclaimed independently. |
| All chunks durable, tree nodes partial | Chunks + some nodes | Resume rebuilds the missing nodes from chunk records; no re-ingest. |
| Tree root durable, publication WAL not | Nodes | Session resumable; no generation visible. |
| Publication WAL durable, root not advanced | WAL payload | C.8 redo publishes the generation exactly once; catalog index entry appears after rebuild or live maintenance. |
| Drop publication durable, retirement intent not | WAL payload | Records unrouted after reopen; extents displaced; retirement runs later; nothing is served from them. |
| Retirement intent durable, release or deletion partial | WAL + checkpoint | Existing C.10 retirement completion: extent ranges are released exactly once; for an evacuated arena, a missing file is a completed deletion, not corruption. |
| Rebuild partially published | Some index pages | Rebuild candidate is discarded; the previous derived generation or `Absent` posture is reported; no mixed index. |
| Memtable WAL durable, run not sealed | WAL entries | Memtable replays from WAL; unsealed run extent is failed-op residue. |
| Compaction output durable, membership not | Run extents | Old membership stays current; output runs are failed-op residue. |

The managed append seam follows the installed WAL-first progression:
`append_managed_wal` → `synchronize_managed_wal` → `settle_managed_data`
→ root publication. Arena bytes written by that path cannot have the
"free and immediately reusable" fate after a crash without contradicting
the durable WAL. The distinct evacuation-copy seam above tests durable
destination bytes before the final copy publication: its source stays current
and the unpublished destination cannot become a routed record.

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
- **Crash law.** Publication absence alone never proves a range reusable.
  A durable append WAL member or evacuation-copy intent holds its range until
  recovery publishes or durably cancels that exact effect, even when the
  published free map has not yet incorporated it. Only unpublished bytes
  without a live WAL or recovery claim may remain free; recovery derives
  claims from durable authority, never a residue scan. Readers follow only
  published routes, and frame identity, generation and checksum reject stale
  bytes. Torn writes can affect only unpublished ranges, so no double-write
  buffer or full-page image is needed.
- **Fragmentation and evacuation.** An arena whose live ratio falls below its
  admitted evacuation threshold is evacuated by the `CompactionRewrite`
  producer through `PhysicalRecordSubmission::prepare_arena_evacuation`.
  `advance_extent_copy` copies and verifies bounded frames under a durable
  source-copy intent and source-root protection;
  `prepare_completed_extent_copy` hands the completed destination to the
  ordinary WAL/root publication owner. This preserves stable record identity
  and exact payload bytes without the non-streaming
  `rewrite_selected_extent_record` payload ceiling. Once empty in every
  protected root, the arena file is
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
format version, a length, the payload and authenticated frame integrity.
The C.5 outer frame and root route remain the physical record authority; a
blob digest alone cannot establish selected-record custody.

- `BlobChunkFrame` (authoritative): one versioned canonical content subframe
  with admitted rule and stored bytes. Its SHA-256 is `StoredChunkDigest` and
  excludes Store, session, object, ordinal, placement and RecordId. A distinct
  occurrence envelope in each newly written C.5 record binds Store scope,
  declared session ID, ordinal, length and canonical digest under the outer
  frame integrity. Its selected route supplies `PersistedRecordIdentity`
  without a self-referential hash. Phase 4 reuse adds an authoritative tree or
  frontier edge to an existing selected record; it does not rewrite the
  original occurrence envelope. A missing or mismatched claim is corruption,
  even when the inner digest and outer C.5 checksum are valid.
- `BlobTreeNode` (authoritative): leaf nodes carry up to 4096 ordered
  (digest, record identity, byte length) entries; interior nodes carry up to
  4096 (child digest, record identity, covered bytes) entries. A node's
  canonical identity is the SHA-256 of its content frame. Each newly written
  node's separate occurrence envelope binds the declared session, node kind,
  level/index, length and canonical digest under C.5 outer integrity and
  selected routing. Partially built nodes therefore remain attributable to
  the unfinished session after WAL pruning; an unclaimed node is not safe
  residue. A 4 GiB blob at 256 KiB chunks is 16 384 chunks, four leaves and
  one root.
- `BlobGenerationPublication` (authoritative): `BlobObjectId`,
  `BlobGeneration`, root node record identity and digest, total bytes,
  `LogicalContentDigest` (SHA-256 of plaintext), chunking rule version,
  dedupe scope, and the declared session identity it closes.
- `BlobResumeSession` (authoritative for its own fate): Phase 2 installs a
  minimal `Declared` record, root-published before the first chunk effect.
  It binds a Store-issued fresh 128-bit attempt ID (checked against selected
  sessions before publication, never caller- or S.7-deterministically
  minted), object ID, Store/key scope, admitted rule, declared total,
  memory/resource limit, declaration digest and maximum durable checkpoint
  sequence. The Store admits that limit from its selected checkpoint sequence
  plus a bounded caller-requested horizon; the caller cannot assert an
  absolute sequence or a wall-clock deadline. Phase 3 adds versioned frontier
  ordinal, last durable chunk identity/digest, explicit abort and terminal
  `Published`, `Abandoned`, `Reclaimed` transitions. A durable checkpoint
  crossing the declared sequence, or a durable explicit abort, may establish
  abandonment only if no publication and no protected hold wins under the
  same root owner; process-local time and wall-clock jumps never do. With no
  advancing checkpoint or abort, unfinished bytes remain retained.
  For this append-only terminal transition, a protected hold means the
  Store-owned same-session ingest/resume/publication claim: an inspecting or
  live claimant defeats a competing abandonment attempt. Terminal admission
  captures its claim and selected-root protection under the root owner,
  authenticates the declaration and absence of a selected publication or
  terminal, and retains exclusivity through C.5 root resolution. Uncertain
  effects require C.8 reconciliation before another session operation may
  proceed. A generic C.10 snapshot lease protects the selected bytes, not a
  right to resume or publish the unfinished session; it need not veto the
  append-only `Abandoned` record. That record leaves all routes and bytes
  retained. Every C.10 reader/recovery pin and admitted semantic hold remains
  binding at subsequent drop/reclaim and physical retirement; terminal status
  alone never authorizes deletion.
- `BlobExportManifest` (authoritative for one export): Store-local physical
  root digest, portable `LogicalContentDigest`, chunk count and export
  custody identity. Import verifies the portable digest and bytes, then
  constructs its own physical tree. Not a backup artifact.

Phase 2 extends the current `PersistedPhysicalRecoveryProjection` grammar with
one bounded typed blob semantic member: `None`, `SessionDeclared` or
`GenerationPublished`, with non-`None` members bound to exact C.5 record
identities, payload digest and root candidate. All newly encoded ordinary
recovery projections use that current grammar, with `None` for non-blob
operations; historical projection formats are rejected, not translated.
The current physical-target variants retain both `Frames` and `SourceCopy`
semantics. The producer, C.8 replay, C.9
validator and independent offline observer recognize the coordinated format
cutover. The semantic descriptor is a nested member of the existing C.8
canonical redo, not a second WAL lane, and remains within 1 KiB; the entire
redo may carry separately bounded physical frame bytes and is not claimed to
fit 1 KiB. C.8 replays declaration before chunk admission and publication
only after all claimed records. Phase 3 adds versioned
`store.physical.blob-resume.v1` frontier/terminal semantics to the same
envelope, preserving current SourceCopy semantics without historical readers.
After checkpoint WAL
pruning, selected declaration, claims and publication records still suffice
for independent custody classification of chunks and partial tree nodes;
unselected arena bytes are not authority.

Derived blob structures: blob catalog (`BlobObjectId` to publication record
identity), dedupe index (`StoredChunkDigest` and scope to chunk record
identity), reachability edge set (generation to chunk and node record
identities, plus resume, export, read-plan and quarantine holds), and orphan
classification. Each is a B-tree family over inline pages with a rebuild
basis over the full authoritative closure: selected generation publications,
session declarations/frontiers and occurrence claims, their tree nodes and
chunk edges, selected export manifests, and C.10 read/recovery protection
plus quarantine hold authorities. The catalog needs publications; dedupe and
reachability additionally traverse verified trees and selected claims. No
report, JSON row or derived index supplies missing authority. Corruption of
any derived family is `DerivedProjectionCorruption` and rebuilds from this
closure; corruption of an authoritative family is localized and reported.

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
  routed records of the indexed family, or for the blob families the selected
  publication/session records, occurrence claims, verified tree edges and
  admitted hold authorities named above. `DerivedIndexRebuildSourceInput::{CertificationRows,
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
`BlobIngestPressure`. `begin_ingest` first durably publishes its minimal
`BlobResumeSession::Declared` record through the C.5 WAL/root owner; it
cannot return an effect-bearing session or append the first chunk before that
publication. Phase 2 installs the actual bounded ingest-pressure producer,
retained head and foreground-preservation admission, not just its vocabulary.
Its admitted I/O shape is the Store's real buffered-file write, fsync,
directory-sync and root-publication sequence; it does not label synchronous
file effects as an imaginary async-I/O worker.
Each source frame is at most the window and may cross fixed chunk boundaries;
the session incrementally hashes and splits it into canonical chunks. For
each new chunk it appends a C.5 record with an authenticated occurrence claim;
Phase 4 may instead bind a previously selected, byte-compared record through
the tree edge in the same scope. Phase 3 periodically publishes frontier
records. Finishing seals leaves and interior nodes as records, then publishes
the single blob generation through the sole publication owner. Memory
accounting includes the source frame, pending frame/redo bytes, node frames,
writeback and scheduler head; no full-object batch or copied blob-sized WAL
payload is admitted.

Phase 2 can classify an interrupted prepublication session as retained
unfinished work but does not offer a resume or reclaim API. Phase 3 first
reconciles pending C.8 WAL effects, scans selected occurrence claims beyond
the frontier, re-reads and re-digests the last durable chunk, then reuses the
exact selected record or denies a conflict; it never appends one ordinal
twice. A durable checkpoint crossing the declaration's limit or explicit
durable abort establishes `Abandoned` under the same publication owner;
only then may the Store independently reclaim records proven exclusive to
that failed operation. Shared deduped records and published or protected
generations are never eligible through this rule.

### Read and verification

`BlobReadSession` (new, Store-owned) resolves a generation through the catalog
index, walks the tree nodes with protected reads, and yields chunks in order
for the requested byte range. It holds a `BlobPhysicalAllocation` of one
window and the root protection of its `RecordReadSession`. Verification is
per chunk against the leaf digest; a readable inner mismatch is
`BlobChunkCorruption` with ordinal, record identity, expected and observed
digests, reported to C.9 disposition. If the outer C.5 frame fails integrity,
the inner observed digest is unavailable rather than fabricated; data-record
damage localizes to its dependent ranges, while root/routing/manifest damage
retains global serving revocation.
Whole-object verification is a streaming pass over the same session with the
same window.

### Dedupe, reachability, orphans, reclaim

Dedupe is a derived index consulted during ingest; the first reuse of a digest
performs a byte comparison under a bounded window. Reachability traversal is
a bounded walk of publication and session records, authoritative tree nodes
and holds (never the derived dedupe index as liveness truth) that yields
per-record `Reachable`, `HeldOnly`, `FailedOperationResidue`,
`DerivedResidue`, or `Unreferenced`. Only the last three classes are orphan
candidates, and only the first two of those are Store-reclaimable without
proof. `Unreferenced` records of a published generation are never reclaimed
without an `AdmittedBlobReleaseProof`.

Reclaim executes as: eligibility (reader and recovery pins through C.10
retention; proof admission), a sorted extent-backed `DropSetManifest`
containing at most 1024 dropped record identities with count and digest,
then one `drop_reclaimed_records` publication whose bounded
`store.physical.blob-reclaim.v1` semantic WAL descriptor names that
manifest and proof/source basis, then retirement of the displaced extents
through the existing retirement owner. The descriptor stays within 1 KiB;
the manifest is C.5-routed, retained and charged until the drop and C.8
recovery frontier make it safe to retire. Recovery independently validates
the manifest before un-routing any record.
The manifest carries a Store-issued reclaim-attempt ID and the admitted
proof/source digest. If its drop descriptor is not durable, it remains
selected failed-operation residue with exact custody; recovery does not
infer a drop from the manifest alone. A later bounded owner publication
retires that residue after the pending WAL fate is reconciled. If the
descriptor is durable but the root is not, C.8 replays the exact drop.
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

Producers in `instance/scheduler_admission/`: `blob_ingest.rs`
(`IngestPressure`), `blob_reclaim.rs` (`BlobReclaimPressure`),
`rebuild.rs` (`RepairScan` class as the rebuild lane), and
`compaction.rs` (`CompactionRewrite`). `RetainedBackgroundHeads` gains a
compaction head. Checkpoint, reclamation, and compaction retain heads only while
a producer owns a retryable quantum. Blob ingest and reclaim are synchronous
one-frame attempts: a denied attempt releases its dispatch head because no
retry owner remains. Admission precedes each producer frame write, but a
managed mutation may already have durably attempted its C.10 WAL prelude;
scheduler denial then remains `Indeterminate` until C.8 reconciliation, not
`ProvenNoEffect`. The denied frame performs no data effect. Every producer
lowers an exact effect footprint (record ranges, extents, index pages, root)
through the C.10 algebra and is subject to
the same foreground floor and owed-background-turn bound.

### Lifecycle and outcome topology

Every session (ingest, read, rebuild, reclaim, compaction) carries the
lifecycle of the runtime part that owns it: cancellation before any admitted
effect attempt is `ProvenNoEffect`. Once a WAL write or later effect has
been attempted, even a sync failure cannot prove absence; uncertain
durability is typed `Indeterminate` until exact C.8 reconciliation, while
known durable effects follow their pending/redo fate. No failure claims
`ProvenNoEffect` merely because the root has not published. Close
revokes protection and allocation; shutdown drains through the C.3 sealed
lifecycle. `begin_ingest` returns an effect-bearing session only after the
declaration root is selected, but a mid-declaration failure returns a typed
pending/indeterminate identity that fresh-process recovery settles. Phase 2
therefore identifies retained unfinished work even before the first chunk.
Outcomes are typed: `Published`, `RetainedUnfinished` (Phase 2),
`Resumable`/`Abandoned` (Phase 3), `Denied(kind)`,
`Deferred(reason)`, `Indeterminate(stage)`.

## Public DX Target

```rust
use worth_proof::AdmittedBlobReleaseProof;
use worth_store::physical_runtime::{
    AdmittedBlobScope, BlobCheckpointLimit, BlobChunkSize,
    BlobIngestDeclaration, BlobIngestOutcome, BlobReadRange, BlobReclaimReceipt,
    BlobReclaimRequest, BlobResumeToken, BlobStreamingWindow, IndexFamily,
    PhysicalBlobDenial, PhysicalKeyRange, PhysicalLayoutDenial,
    PublishedBlobGeneration, ServingPhysicalRuntime,
};

trait BlobFrameSource {
    fn declared_bytes(&self) -> u64;
    fn next_frame(&mut self, max_bytes: u64) -> Result<Option<&[u8]>, PhysicalBlobDenial>;
    fn seek(&mut self, byte_offset: u64) -> Result<(), PhysicalBlobDenial>;
}
trait BlobByteSink {
    fn write(&mut self, bytes: &[u8]) -> Result<(), PhysicalBlobDenial>;
}

// Phase 2 API illustration only: a 4 MiB source fits this 1 MiB window,
// but certification uses >6 MiB so full materialization exceeds W + 5 MiB.
fn ingest_and_read(
    store: &ServingPhysicalRuntime,
    scope: AdmittedBlobScope,
    limit: BlobCheckpointLimit,
    source: &mut impl BlobFrameSource,
    sink: &mut impl BlobByteSink,
) -> Result<(), PhysicalBlobDenial> {
    let blobs = store.blobs()?;
    let window = BlobStreamingWindow::bounded(1 << 20)?;
    let object = blobs.issue_object_id()?; // opaque physical ID, not semantic identity
    let declaration = BlobIngestDeclaration::new(
        object, BlobChunkSize::from_bytes(256 << 10)?,
        source.declared_bytes(), scope, limit,
    )?;
    let mut ingest = blobs.begin_ingest(declaration, window)?;
    // begin_ingest has already root-published the durable session declaration.
    while let Some(frame) = source.next_frame(window.bytes())? {
        ingest.push_frame(frame)?; // arbitrary source frames split across chunk boundaries
    }
    let published = match ingest.finish()? {
        BlobIngestOutcome::Published(generation) => generation,
        BlobIngestOutcome::RetainedUnfinished(session) =>
            return Err(PhysicalBlobDenial::RetainedUnfinished(session)),
    };

    let mut read = blobs.read(
        object, published.generation(), BlobReadRange::bytes(1 << 20, 1 << 20)?, window,
    )?;
    while let Some(chunk) = read.next_chunk()? {
        sink.write(chunk.bytes())?;
    }
    let receipt = read.finish();
    assert_eq!(receipt.chunks_read(), 4);
    Ok(())
}

// Phase 3: this API does not exist in the Phase 2 MVP.
fn resume(store: &ServingPhysicalRuntime, token: BlobResumeToken, source: &mut impl BlobFrameSource)
    -> Result<PublishedBlobGeneration, PhysicalBlobDenial> {
    let blobs = store.blobs()?;
    let mut ingest = blobs.resume_ingest(token, BlobStreamingWindow::bounded(1 << 20)?)?;
    source.seek(ingest.frontier().bytes())?;
    while let Some(frame) = source.next_frame(ingest.window().bytes())? {
        ingest.push_frame(frame)?;
    }
    ingest.finish()?.published().ok_or(PhysicalBlobDenial::StillResumable)
}

// Phase 6: published-generation release still requires an admitted proof.
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
  recovery_construction/
    port.rs, handoff.rs                          E sole Store construction and one-shot custody handoff; pending claim exclusive of checkpoint NoRelease/release
    selected_rejoin/pending_wal_release.rs       N Store-owned same-media C.9 WAL/control/source/post-redo-root rejoin
    selected_rejoin/release_heads/               N independent selected-head roster and per-object predecessor rejoin (Phase 6)
  durability/publication/current_root_owner/release_capacity/
    heads.rs, checkpoint_heads.rs                N Store-owned selected per-object head ledger, pre-effect capacity and checkpoint fold (Phase 6)
  record_serving/planning/rebased_root/
    release_heads.rs                            N V3-admitted copy-on-write head-tree projection inside the one result-root publication (Phase 6)
  record_serving/publication/director/
    release_head_preparation.rs                 N pre-WAL descriptor identity and head-tree effect reservation (Phase 6)
  record_serving/admission/recovered_custody.rs  E Serving revalidates pending claim's selected-media fingerprint
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
  manifest/durable_root/release_head_reference.rs  N versioned root anchor for the authoritative head tree (Phase 6)
  manifest/release_head_routing/                 N bounded keyed head entries and copy-on-write tree blocks (Phase 6)
  recovery_projection/release_head_effect.rs   N exact C.9 metadata-effect envelope; one V3 descriptor data record remains (Phase 6)
  recovery_projection/release_head_retirement.rs N distinct owner-proof-backed terminal-head retirement, never a synthetic V3 (Phase 6)
  checkpoint/release_certificate/heads.rs       N current accumulator head-tree commitment; unsupported formats and insufficient current custody deny (Phase 6)
  btree_node/                                   N slotted.rs, separator.rs, sibling.rs
  lsm_run/                                      N header.rs, entries.rs, membership.rs
  integrity_declarations/families/              E blob_*, btree_node, lsm_run declarations
  blob_manifest/                                R placeholder rows replaced
  offline_walk/                                 E blob and index families

worth-store-physical-integrity/src/artifact/
  extent_arena/                                 N arena frame validator (Phase 1)
  free_space/                                   E free-range run validation
  blob_chunk/, blob_tree_node/, blob_generation/, btree_node/, lsm_run/  N validators and validated views
  release_custody_head/                         N head-tree block and selected commitment validation (Phase 6)

worth-store-offline-integrity-observer/src/integrity_observation/families/
  extent_arena/                                 N overlap, routed-but-free and unaccounted-byte checks
  blob/, index/                                 N independent offline families
  release_custody_heads/                        N selected roster versus rooted head observation (Phase 6)

worth-store-recovery-physics/src/redo_replay/    E blob/index payload kinds in record.rs and plan/admission.rs typed arms
worth-store-recovery-physics/src/redo_replay/release_head_effect/  N C.9 source/result/node/free-space transition admission (Phase 6)
worth-store-recovery-physics/src/source_precedence/pending_wal_release_custody.rs  N private C.8 pending-WAL release proof, distinct from tag-7 custody
worth-store-recovery-physics/src/source_precedence/release_custody/heads/  N selected head-tree membership and pending V3 metadata-transition proof (Phase 6)
worth-store-recovery-runtime/src/               E blob/index reconciliation and orphan fate
  orchestration/planning/completion/blob_reclaim/selected_release_gate/pending_wal.rs  N C.8 admitted-redo/fate join before handoff
  orchestration/{planning,publication,reopen,handoff}.rs  E carry the pending claim across completed publication and fresh reopen

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

The compatibility-only owners have no successor authority lane: remove
`worth-store-compatibility/`,
`worth-store-layout-indexes/src/evolution/migration/`, its
`src/observation/evolution.rs` leaf, and
`worth-store-test-support/src/harness/layout_evolution/`, together with their
obsolete dependencies, exports and registration. Close callers of
compatibility-only artifact-family declarations before deleting those rows;
retain the current artifact-family inventory and format-owned identity checks.
Current format mechanisms remain in the existing physical-format, integrity,
recovery and Store owners above, not in a replacement compatibility crate.

The recovery-projection cutover refines the existing Format owner: its
`recovery_projection/operation.rs` owns the operation sum and checked attachment
construction, `recovery_projection/codec/operation.rs` owns operation tags,
and `recovery_projection/codec/domain.rs` owns current identity admission.
The root projection owns bounded common state and the stable public facade
remains `worth-store-physical-format::lib`. Remove `codec/version_semantics.rs`
and all version-specific ordinary lanes. The compiler-shaped constraint belongs
at `tools/boundary-check/src/source_rules/analysis/store_current_projection.rs`,
with named wire-boundary checks beneath it; no compatibility crate, alternate
producer facade or generic migration framework replaces the removed paths.

## Cost Contracts

The single Phase 2 ingest residency ceiling is `W + 5 MiB`, where `W` is
the admitted source window (at most 64 MiB). The 5 MiB allowance includes
one encoded chunk frame (at most 1 MiB), one pending canonical-redo copy
(at most 1 MiB), one writeback buffer (at most 1 MiB), two encoded tree
nodes (at most 512 KiB each), one publication frame (at most 64 KiB), one
retained scheduler head (at most 64 KiB), and remaining bounded digest and
bookkeeping scratch. The format codecs and producer enforce each component
maximum before effect; the allocation and process-level high-water probe
charge simultaneously live buffers, including a caller-supplied source
frame. No full-blob `Vec`, batch, or uncharged frame clone is permitted.
If a later format exceeds a component maximum, its phase must revise the
contract and proof before admitting it.

| Path | Ordinary cost | Ceiling and scale axis |
| --- | --- | --- |
| Blob ingest | One durable declaration before the first chunk; one C.5 chunk-record append/root progression per newly written chunk; one leaf write per 4096 chunks; one blob-generation publication. Phase 3 adds one frontier transition per admitted interval (default every 64 chunks). | Peak charged resident bytes <= `W + 5 MiB` by the component accounting above; arena files = ceil(bytes / arena capacity) + 1, independent of chunk count. "One blob-generation publication" never means one physical-root update for the entire ingest. |
| Extent allocation | Best-fit lookup in the published free map, or append to the filling arena | O(log free runs); no media read; free runs per arena bounded by the evacuation threshold |
| Blob range read | Node path (height <= 3 for 2^36 chunks) plus chunks touched | Read amplification <= 1 chunk per side; resident <= window |
| Dedupe hit | 1 B-tree probe plus 1 bounded byte comparison on first reuse | No whole-object comparison; comparison window = chunk size |
| Publication | 1 typed semantic WAL member + root member, with physical frame bytes separately bounded | Semantic descriptor <= 1 KiB; do not apply this cap to the whole canonical redo |
| Catalog/dedupe rebuild | Bounded traversal of selected publications, sessions, occurrence claims, verified tree edges and admitted holds | Record/node visits and pages touched measured from the full authority closure; resident <= rebuild allocation |
| B-tree point | Height page touches | Height <= 4 for 2^32 entries **only when** the admitted layout achieves fanout >= 256 (for example, a 64 KiB page with the registered DedupeIndex key64/value56 shape). The default 16 KiB page has lower DedupeIndex fanout and may require height 5 at that scale; no 2^32-entry empirical run is claimed. |
| B-tree range | Height + leaves spanned | Budgeted by caller; envelope violation is denial |
| LSM point | Memtable probe + 1 probe per run in membership | Runs per level bounded by compaction policy |
| Reclaim | 1 drop publication per batch + retirement per displaced extent | Batch <= 1024 records; retained bytes fall by dropped bytes after retirement |
| Reachability | One bounded pass over selected publication/session records, occurrence claims, authoritative tree nodes and admitted holds | Resident <= verification allocation; never loads chunk bytes merely to infer liveness |

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
`prepare_arena_evacuation`, `advance_extent_copy`, and
`prepare_completed_extent_copy` with ordinary WAL/root publication and
whole-arena retirement. Evacuation must remain bounded for extents larger
than the non-streaming selected-record rewrite ceiling. The C.10 extent
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
`BlobResumeSession::Declared`/`BlobChunkFrame`/`BlobTreeNode`/
`BlobGenerationPublication` families with their C.9 declarations and
validators, and the current typed recovery transition without historical read
admission.
The Store-issued session declaration is root-published before any chunk;
each new chunk has its distinct authenticated occurrence claim. Install the
real `BlobIngestPressure` producer and bounded retained head together with
the Store `BlobIngestSession` over `PhysicalRecordSubmission`, generation
publication through the existing C.8 owner, and `BlobReadSession` over
protected reads. No registry, dedupe, resume, reclaim or index yet; the
catalog lookup for this phase is a bounded scan of publication records
admitted as the `Rebuild` lane shape, replaced in Phase 4.
`CapabilityAvailability::Present` (D12) lands here together with
`blobs()`, so status and accessor become real in the same change;
`layouts()` and Layout `Present` land together in Phase 4.

Closeout gate: a blob of at least 4 x the window and strictly larger than
`W + 5 MiB` ingests through the production path with measured resident bytes
under the ceiling; the injected whole-object materialization breaches the
same measured bound. It publishes exactly once,
survives a fresh-process reopen, and streams a range byte-exact with
counters equal to observed I/O; the whole-object substitutions are typed or
compile-time denials; the three cycle-removal proofs hold (no `cargo tree` path, no `worth_store::` import in
`worth-store-blob-chunks`, boundary-check
DAG snapshot) and every file in the cycle-removal table has its disposition;
the offline observer walks the new families without runtime APIs. Kill after
declaration publication and after a claimed chunk but before generation,
then require fresh C.8 reopen plus independent observer to identify retained
unfinished custody without an invented orphan or published blob. Kill after
generation WAL durability but before root publication and require exact
once-only redo; current Frames and SourceCopy operations remain valid, while
unsupported historical projection versions deny before promotion or effects.
Proof obligation: hiding a full materialization fails the residency
predicate, while omitting an occurrence claim despite valid C.5 checksum and
inner SHA fails offline custody validation.

### Phase 3: Interrupted ingest, resume, and independent residue reclaim

Extend Phase 2's durable `BlobResumeSession` declaration with versioned
frontier and terminal records, resume readmission, checkpoint-sequence
expiry arbitration, explicit abort and Store-independent reclaim of
failed-operation residue through `drop_reclaimed_records` plus existing
retirement. This phase installs record-dropping publication because
abandoned residue is the first legitimate consumer; published generations
and shared deduped chunks are never eligible here.

Closeout gate: kill distinct writer processes after a selected declaration
and claimed chunk before frontier, after frontier before the next complete
chunk, after all chunks with only partial tree nodes, and after the tree root
before generation publication. Each fresh reopen has the exact declared,
claimed, frontier or partial-tree fate from the blob seam rows; resumed ingest
reuses selected records and writes no ordinal twice. Abandoned exclusive
residue is reclaimed with retained bytes falling and no external proof;
forged or mismatched tokens are denied.

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

Checkpoint release custody is positive, never inferred from a missing drop
record or a digest alone. The versioned tag-7 stream carries bounded
released-drop Batch/Accumulator custody after a proven drop, or a distinct
`NoRelease` marker bound to the selected checkpoint and source root. Store may
mint the first zero-predecessor marker only from its trusted fresh-genesis
zero-release ledger; a successor names the exact prior selected marker and
its payload digest. The current C.7/C.10 namespace has one durable
`checkpoint.current` and an exact WAL suffix: publication atomically replaces
that checkpoint, and candidate files are residue, not retained predecessors.
C.8 validates the selected current marker. Its prior fields are Store-attested
custody from the exact previously selected ledger, not independently replayed
history; a future governed retained-checkpoint role would additionally require
C.8 to validate any selected predecessor it retains. Reopen without either valid custody form remains
unavailable, including old unmarked checkpoints. C.8 validates the selected
certificate, root and typed route/WAL evidence, while Store independently
rejoins the same selected media before issuing a one-shot Serving seal. Ordinary
reopen whose retained WAL suffix has no released-drop member is the clean case:
Store itself verifies the selected certificate against the loaded root and
source and installs that custody without C.8. Any retained released-drop
member keeps custody unavailable until the C.8 handoff.

A released-drop WAL member may become durable after a selected checkpoint and
before the next checkpoint. Its `NoRelease` marker, if present, attests only
that checkpoint's source root; a selected Batch/Accumulator instead supplies
the certified Store-wide cumulative starting point. Neither form alone
authorizes a post-checkpoint released root. Distinct released objects can
produce multiple V3 WAL batches before a successor checkpoint, so C.8's
private-field pending-WAL claim must carry the complete ordered sequence,
not only its tip. For each batch C.8 joins the exact unique C.9-admitted
`RecordsDropped` member and honestly classified C.9 operation fate
(`Indeterminate` before root materialization, or a genuinely completed fate),
V3 descriptor, reservation, manifest, authenticated source closure, and exact
published root and free-space topology. It proves the source/result chain
through every intervening canonical non-release publication; no descriptor
alone authenticates a missing historical link. V3 predecessor and cumulative
fields are per released object, whereas Batch/Accumulator cumulative evidence
and pending-batch order are Store-wide; an unrelated object's descriptor need
not name the global tip. A first drop must name its publication in the drop
set; a successor may instead drop the next payload only after its exact
predecessor is authenticated against the selected per-object chain. Store's
same-media rejoin also resolves that predecessor's exact selected descriptor
and manifest, compares their released-object source basis to the successor,
and checks nonterminal cumulative progression; a different object's valid
Batch is not a substitute. When a pending or historical V3's per-object
predecessor is a checkpoint-source head, that head's Store-attested custody
settles the object's closure records absent at the checkpoint source root;
absence at a later source root is settled only by that key's exact retained
post-checkpoint drops in the verified ordered root history, and a closure
record removed by another key's edge, or absent under neither form, denies.
Neither C.8 nor Store enumerates pre-checkpoint lifetime drops. A selected
TierEpoch+NoRelease checkpoint has two distinct certificates, not an invalid
extra tag: C.8 and Store bind the exact tier intent/completion, tier-anchored
root/free header, NoRelease source marker, and complete WAL inventory before
Serving. Store independently rereads the selected checkpoint
and its source, every required pre-redo source, the final root/free/routes,
complete retained WAL and unique member fates, and all routed controls before
folding the ordered batches into its release ledger and issuing a one-shot
Serving seal. The pending claim remains distinct from the selected checkpoint
certificate: it neither rewrites nor synthesizes tag-7. The next checkpoint
must fold genuine Batch/Accumulator custody from the selected base and every
pending batch. A stale checkpoint-only claim, missing or reordered batch,
substituted control frame, altered WAL member, mismatched fate, or wrong
post-redo topology must deny Serving. If a fresh C.8 sees a prior V3 result
already selected, it may treat the old WAL group as historically consumed only
after proving the exact old member/target/digest/coordinate and complete
source-to-result route and free-space transition; this does not change that
member's C.9 fate or pretend to execute its drop again.

#### Per-object custody across checkpoint replacement

The Store-wide accumulator tip is not a per-object predecessor. An unfinished
A release, then B's release and successor checkpoint, must still permit A's
lawful next release after A's old WAL and checkpoint are pruned. The governing
representation is a versioned, copy-on-write `ReleaseCustodyHead` tree bound by
one fixed reference in the selected root manifest. It has one keyed entry per
released object/generation that may still continue. Its canonical leaf entry
names the latest V3 descriptor,
manifest and reservation identities and frame digests, the released-object
source basis, exact predecessor, per-object cumulative progress and
terminality. Neither the descriptor nor head entry contains the digest of the
resulting head tree/root or its containing checkpoint, which would make
publication circular. The root's head-tree reference authenticates the
bounded, ordered leaves; a fixed root field alone never substitutes for the
per-object entries. Old selected V3 controls are evidence to rejoin, but a
control without current head-tree membership is not release authority.

Every completed V3 drop replaces only its own head-tree entry in the same
WAL-backed root publication as the drop result; unselected copy-on-write
blocks are residue.
The first head requires the exact first-drop publication in the drop set. A
replacement requires the authenticated prior head for the same key, a
nonterminal predecessor, identical source basis and exact next-payload
cumulative advance. A different object's head or global Batch tip cannot
authorize it. Store constructs this transition from its selected release
ledger and admitted V3 proof, never from routed descriptor bytes. The V3 WAL
member still owns exactly one descriptor data record; its versioned metadata
effect also commits the exact keyed head-tree transition, source reference,
copy-on-write node identities/addresses, resulting reference and allocation
effects. C.9 validates this metadata effect with the same member and root:
an extra, missing, substituted or wrong-generation node, wrong descriptor
identity, unrelated-key change or unexpected free-space effect denies. C.8
redo reproduces that exact transition, not an allocation chosen from the
replay-time free map. C.7 may not improvise a head mutation merely because
WAL became durable. The dependency is source root, admitted descriptor and
controls, head entry, new tree reference, result root, then checkpoint;
descriptor identity comes from the admitted WAL binding. Before any effect,
Store reserves worst-case copy-on-write path/split blocks, node storage,
replacement or retirement work, checkpoint-roster work and C.8/Store
recovery-resident charge. Exhaustion is a typed pre-effect denial; a completed
drop may not become uncheckpointable.

The next versioned tag-7 Accumulator binds, alongside the existing ordered
Batch and Store-wide cumulative ratchet, the complete **checkpoint-source**
head roster:
unique canonical object/generation order, count, and domain-separated digest
of each keyed entry and the exact root-referenced tree identity/checksum. It carries the
prior selected roster count/digest as Store-attested custody. Current Batches
fold in order per object; the final selected tree entry for each updated key
must match its latest Batch, and intermediate entries need not remain selected. Store
proves unchanged heads carry byte-exactly when it constructs the successor
checkpoint from its selected ledger. C.8 cannot replay that carryforward from
a superseded checkpoint that no longer exists. The roster scales with
selected release state rather than the
64-record, 64-KiB certificate section: the accumulator commits a bounded,
streamed rooted roster, not one tag-7 certificate per historical object.
The checkpoint-source root's head-tree reference, authenticated tree walk and
accumulator must enumerate the same heads; duplicate keys, unreferenced or
extra selected nodes, omitted entries, unknown versions and noncanonical
order deny. A block outside the selected tree is residue, not a head. An
entry without selected-checkpoint tree membership is not independently
authoritative.

C.8 validates the current checkpoint's roster commitment, exact selected
head-tree blocks and routed descriptor/manifest/reservation controls, current
Batch-to-head updates, root/free/WAL
topology and still-retained genuine C.9 fates. Store separately rereads the
same selected bytes and rederives the per-object map before its one-shot
Serving handoff. A carried head after old WAL pruning is current
Store-attested checkpoint custody, not a claim that C.8 replayed a deleted
historical Batch or C.9 result. As with the existing NoRelease prior ratchet,
a self-consistent rewrite of the whole trusted namespace is outside this
integrity boundary; a digest by itself never grants release authority. Fresh
pending V3 groups remain independently joined from exact WAL/source/result
and head-transition evidence until selected into the next checkpoint. They
advance the checkpoint-source roster to an **effective post-WAL roster**;
neither C.8 nor Store compares that effective roster with the old accumulator
as if it were still the checkpoint source. The Serving seal binds its final
root and effective roster, and the next checkpoint commits that roster. A
current-format NoRelease marker provides the zero-release source for the first
admitted release and its head-bearing successor checkpoint. NoRelease and
released custody are typed states of the current certificate grammar, not a
historical V1-to-V3 compatibility journey. A current released checkpoint missing
its required heads remains unavailable; selected controls cannot supply the
missing custody. Historical certificate formats deny without migration or
legacy fallback.

Terminal heads stay as tombstones until Store consumes owner-issued exclusion
of the exact object's publication, reader/recovery holds and retry/idempotency
claims, plus durable non-reissue of its identity. A checkpoint alone proves
none of those exclusions. Retirement is a separately typed, WAL-admitted
head-tree/root transition included in the next roster ratchet, not silent
omission or a false repeated V3 drop. Ordinary root publications preserve the
head-tree reference unless they consume an explicitly admitted head update
or retirement. A replaced head and
its older descriptor/manifest/reservation chain may be physically retired
only after the successor head is checkpoint-attested, no protected reader,
recovery pin or retry requires the older controls, and no Batch in the
**currently selected** checkpoint or pending WAL claim still needs them for
its ordered fold, per-object identity or source/result rejoin. Checkpoint
replacement or explicit discharge must remove those dependencies before
retirement. The current head and its exact control closure remain retained.
The pre-effect charge covers that
whole retained closure, WAL/root publication, checkpoint and C.8/Store
recovery work, not just head count. An explicit admitted head-population bound
`H` is derived from these charged budgets; a first release for a new distinct
key at `H + 1` defers before WAL or root effects. This bounds current selected
custody, not the number of releases over the Store's lifetime.

The decisive process courtroom starts with genuine A partial drop and
checkpoint, then B drop and checkpoint; the production retention owner prunes
A's old WAL and superseded checkpoint. A fresh process must admit A's
successor, complete C.8 and independent Store rejoin, seal Serving, checkpoint
again and reopen with exact A/B per-object and Store-wide cumulative values.
The same world must deny a substituted unchanged entry, missing or duplicate
entry, wrong key, wrong A predecessor/progress/terminality, B-tip substitution,
an extra selected tree node or roster omission before any new effect.
The selected checkpoint-source roster must stay valid while a genuine
postcheckpoint V3 changes only the effective WAL-derived roster; a stale seal
or post-seal head mutation must deny. A current release checkpoint missing its
required heads must remain unavailable even with routed controls, while current
NoRelease advances through a genuine first release into a head-bearing
checkpoint. Unsupported historical versions deny before effects. Terminal-head
retirement without the owner-issued hold/retry/non-reissue exclusion must deny.
Kill at V3 WAL durability, head-tree node materialization, result root
publication, checkpoint file
replacement and namespace sync: no candidate head may be adopted and no
completed drop may lose its continuation. Run at admitted `H` and `H + 1`,
measuring charged resident bytes and pre-effect capacity denial; mutating the
roster-membership check or C.9 head-effect validation must turn this red.

Closeout gate: reclaim of a held generation defers with a typed reason and
completes after release with exact effects; reclaim without proof is denied
regardless of references or age; a second reclaim is proven no effect; tier
movement never exposes a half-moved chunk; the crash seams for drop and
retirement reopen to their required fates. The existing
`physical_runtime_authority_ui` target must compile a valid Store movement
journey and reject `StablePhysicalReadReceipt` alone as movement-execution or
movement-read-hold authority, using the real types introduced by this phase.

#### Phase 6 execution boundaries

Phase 6 remains one required contract, but implementation proceeds through the
six dependency-ordered submilestones below. These are delivery and review
boundaries, not independently deployable weaker modes. All requirements above
remain mandatory; none moves to Phase 7, Phase 8 or a successor. A checkpoint
commit can preserve unfinished work, including failing tests, but grants no
runtime authority and is not a submilestone or Phase 6 PASS.

The integration checkpoint is one real journey: genuine A partial drop,
checkpoint, B drop, checkpoint, lawful production pruning of A's old WAL and
superseded checkpoint, fresh process, lawful A successor, C.8 recovery and
independent Store rejoin, Serving, checkpoint and reopen with exact A/B and
Store-wide progress. First make ordinary publication and the first real drop
work under their declared budgets. Expand that same journey; do not build
parallel proof machinery while its prerequisite is red. Preserve accepted
evidence unless an edited seam invalidates it.

#### Architecture-law enforcement at every submilestone

The governing [architectural laws](../../docs/coding-guidelines/arch_laws.md),
[performance laws](../../docs/coding-guidelines/perf_laws.md),
[composition laws](../../docs/coding-guidelines/composition_laws.md) and
[domain structure laws](../../docs/coding-guidelines/domain_structure_laws.md)
are implementation constraints, not a final documentation exercise. The
following obligations identify their concrete Phase 6 enforcement boundaries.

| Laws | Required owner behavior | Evidence that must distinguish a violation |
| --- | --- | --- |
| Architecture 1, 3, 4, 16 | Store consumes admitted semantic release proof and lowers exact selected-source, head, control, allocation and root effects before execution. C.7 publishes; C.9 validates exact redo; C.8 reconstructs; Store independently rejoins before its one-shot Serving transition. Observation, routed bytes, digests and generic completion grant no authority. | Public compiler boundaries reject forged/skipped progression; no-proof, wrong-object and stale-source twins deny before new effects; exact WAL/media observation distinguishes admitted from performed work. |
| Architecture 5, 7, 14, 22 | Root preparation and every failure translation preserve the typed underlying cause, responsible boundary, proven/maybe-started effect posture, retained authority and recovery disposition. Cancellation never erases durable work. | Inject preparation, resource, WAL, root and namespace failures; assert exact cause and fate, selected artifacts and recovery result, not merely `is_err()`. A published generation with pending indexing must remain a typed partial outcome, not fixture success or rollback. |
| Architecture 9, 11, 19, 22; performance allocation and memory laws | Recovery, rejoin, Serving handoff and each drop derive one worst-case memory envelope from their admitted bounds (WAL bytes, checkpoint size, head population `H`, roster size) and admit it once before any effect; exceeding the budget is a typed denial before effects. Allocations inside an admitted operation need no individual reservation, and no per-allocation reservation plumbing is added. Retained custody keeps one charge that is carried across C.8, Store rejoin, Serving, mutation and disposal. | A memory-counting test allocator over the production journeys checks that measured peak stays within the envelope; an undersized budget denies before effects and disposal releases the retained charge. Reconcile `canonical_store_metadata_envelope` against actual retained metadata rather than raising its budget to conceal overhead. |
| Architecture 8, 10, 17, 18 | Selected checkpoint custody, pending-WAL claims, checkpoint-source roster, effective post-WAL roster and per-object predecessor are distinct owner facts. Ordinary visibility still comes from selected C.5 routes, never a retained catalog cell or the global release tip. | Pruned A/B continuation, post-checkpoint updates, unchanged-entry and stale-seal twins; ordinary surviving-object reads and independent offline observation. No old checkpoint, heap ledger or writer state crosses the fresh-process boundary. |
| Architecture 21; DX 7 | Durable head, root, metadata effect and checkpoint families declare current-only supported grammars and typed states under the explicitly permitted undeployed Store policy. Keep identity/version checks, never reinterpret old bytes, and reject historical formats without migration. | Current NoRelease advances through a genuine first release; missing current heads and unsupported versions deny; C.9/C.8 and independent Store/observer consumers admit the same current grammar and reject incompatible or altered metadata before promotion. |
| Architecture 6, 13, 15, 23 | Effects remain in the existing Store work/scheduler/executor/publication path. Physical reachability cannot issue semantic liveness, holds or non-reissue proof. Admission and recovery guarantees cannot be traded for throughput or easier review. | Real held-reader, retirement, movement and interference journeys; compile-fail movement substitutes; media observations expose executor bypass or half-publication. |
| Composition 1-9, 13-15; domain structure 1-3, 7-11, 17 | Keep proof decisions, execution, replay, diagnostic translation and independent rejoin in their existing semantic owners. Facades export contracts only. Orchestration names proof-building steps; line-count extraction cannot substitute for responsibility boundaries. | Bounded independent structural review, dependency/visibility checks, scoped 400-line guard and judgment of function advisories. New growth enters the destination tree below without phase-named buckets or a second authority lane. |

Resource admission is one envelope per operation, not per-allocation plumbing.
Each submilestone sizes the envelope for the paths it introduces or changes
and proves it with the counting allocator. Existing per-allocation funding
code is frozen: no new additions, and it may be removed where the envelope
covers it. The retained closure and checkpoint/recovery envelope are
prerequisites of the first admitted drop.

#### 6.1: Bounded ordinary publication and diagnostic foundation

Consume the existing protected selected-root reader and Maintenance admission;
make strict native blob publication and derived-index completion work in the
existing 32 MiB operation profile. Retirement admission must bound the actual
closure and all coexisting storage before construction, enforce that bound
while collecting/copying it, and retain the grant for the funded lifetime.
Do not reserve the entire global maximum independently for nested tiny
closures, remove a required reservation, or enlarge the profile to pass.

First acceptance feedback comes from the strict publication setup of
`release_reopen::shared_reuse_custody::source_first_reuse_continues_only_with_fresh_c8_custody`
in the existing Recovery Runtime `production_entry` target, plus focused
retirement-admission and root-preparation failure tests. Setup must not swallow
`PublishedIndexPending`. A too-small profile must report the responsible typed
denial before that newly admitted work's effects while preserving any earlier
durable partial publication. Passing setup enables 6.2; it does not certify the
rest of the recovered journey or Phase 6.

#### 6.2: Atomic one-object release and surviving ordinary visibility

Consume admitted release proof, current per-object source/head and live reader/
recovery protection. Establish one WAL-backed drop/head/control/root transition
with pre-effect capacity for its replacement, retirement, checkpoint roster
and C.8/Store recovery closure. Checkpointability is an admission invariant,
not a check performed after completing the drop.

If a released publication invalidates a derived directory's immutable
watermark, publish a new lawful directory binding in the same atomic
transition, retaining surviving family roots and their actual truth status.
Its metadata effect must be versioned and validated/replayed by C.9/C.8;
never keep an invalid old binding, reinterpret prior metadata bytes, or make
ordinary catalog reads fall back to a broad authoritative scan. Retained
derived cells cannot resurrect an unrouted publication.

Acceptance is a genuine partial drop followed by checkpoint and ordinary reads
of surviving shared content. Include source-first and destination-first reuse,
held-reader deferral followed by exact completion, no-proof denial, repeat
proven-no-effect, exact drop/head/free-space effects and this transition's
failure/crash fates. Only this accepted production boundary may support 6.3.

#### 6.3: Independent recovered custody and multi-object continuation

Consume the real checkpoint, exact retained WAL/member fates and selected
head/control closure. C.8 and Store independently derive their respective
facts from the same media; the Store-issued one-shot seal binds final root and
effective post-WAL roster before Serving. Keep selected checkpoint certificates
distinct from pending claims and checkpoint-source rosters distinct from
effective rosters. Carry continuously owned recovery/rejoin residency into
Serving and any admitted growth; handoff cannot release a grant while funded
state remains live.

Decompose `selected_release_gate/pending_wal.rs::admit` by semantic proof steps:
ordered member/fate selection, authenticated media/source/result joins,
reservation and head-transition validation, exact replay admission, live
aggregate cost admission, then private claim construction. These steps carry
named owner facts; downstream code may not reselect raw evidence or construct
the final claim without the preceding proofs. This is not arbitrary file
fragmentation or a new generic proof framework.

Acceptance completes the decisive pruned A/B process journey above and the
same-world hostile variants already required by Phase 6. Include genuine
post-checkpoint head change without invalidating the old checkpoint-source
roster, stale/post-mutation seal denial, current NoRelease progression,
missing-current-head refusal and unsupported historical-version denial.
A passing unsealed-open denial is negative evidence only;
it never replaces positive fresh-process recovered continuation.

#### 6.4: Lawful custody retirement and admitted population

Consume checkpoint-attested successor custody plus owner-issued exact
publication/hold/retry/non-reissue exclusion. Use the separately typed
WAL-admitted head retirement and roster ratchet; retain current controls and
every older control still needed by selected checkpoint or pending folds.
No checkpoint age, missing route or terminal flag alone permits deletion.

Acceptance proves denial with each material dependency still live, exact
retirement after lawful exclusion, continued reopen after production pruning,
and populations at admitted H and H+1. Measure the whole live retained closure
and transient work; H+1 denies before WAL/root effects. These measurements
complete, not introduce, the pre-effect capacity contract used by 6.2/6.3.

#### 6.5: Genuine tier movement and scheduled interference

Consume real movement execution/read-hold authority through C.10 rewrite,
protection, scheduler and executor contracts. Preserve TierEpoch+NoRelease's
distinct certificates and their recovery binding; release proof does not
implicitly authorize movement.

Acceptance observes real chunk movement without half-moved visibility,
protected old-reader survival, exact interference with reclaim and foreground
ingest, and fresh recovery of interrupted movement. The existing authority UI
target accepts the real Store movement path and rejects a
`StablePhysicalReadReceipt` as either required movement authority.

#### 6.6: Integrated Phase 6 closure

Integrate the accepted paths at V3 WAL durability, head-node materialization,
result-root publication, checkpoint replacement and namespace synchronization.
Each earlier effect-bearing submilestone already owns its relevant interrupted
fate; this campaign must not be the first consideration of crash or cleanup.
Use the independent observer for persisted identity, topology and progress,
not writer-returned truth. Complete all remaining Phase 6 requirements above,
including mutation-sensitive roster membership and C.9 head-effect checks.

Before Phase 6 PASS, run focused owners and affected production/UI journeys,
formatting, the scoped dirty Rust line-cap guard, function advisories with
causal-scope judgment, and the mandated boundary and generated-context checks:

```text
cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .
cargo run --manifest-path tools/agent-context/Cargo.toml -- check
```

Revise the existing durability, recovery and integrity guides named in this
specification against the actual facades, compatibility and operator recovery
behavior. Independent `qa-loop`, `qa-tests` and `code-quality-qa` clearance at
bounded slice gates remains mandatory; final phase certification uses the
selected independent reviewer arrangement. Missing review or required evidence
is an unmet gate, never primary-agent self-certification.

#### Execution ownership and stopping rules

Keep one integration owner and one coordinated Cargo lane. Assign narrowly
owned production changes, independent fixture/test analysis and independent
review; stabilize shared contracts before concurrent edits. Workers may not
edit shared contracts concurrently or run contending Cargo builds. Review the
bounded delta once, then only corrections and invalidated seams; do not repeat
whole-dirty-tree certification, generate evidence registries or test tests.

Before each slice, identify its first real failing boundary, verified facts,
remaining hypotheses, owner interfaces and concrete acceptance commands. A
failure that invalidates those facts triggers a boundary/plan correction before
more edits. More than a day on the same unresolved slice warrants a focused
rebaseline of that seam and dependencies, not parallel patch accumulation;
elapsed time is neither proof nor permission to weaken the contract. Preserve
settled design and accepted evidence unless concrete invalidating evidence
requires reopening them.

If three consecutive slices are the same kind of fix (for example funding,
scratch accounting or proof plumbing for one seam), stop and propose a
structural fix to that seam before landing another.

### Phase 7: LSM strategy, compaction, and export/import

Install the memtable payload, `LsmRun` records, membership persistence
through the Store WAL, the compaction producer over C.10 rewrite, and
bounded export/import.

Closeout gate: LSM point/range equal the model across memtable and runs;
compaction publishes membership only after runs are durable and retires
stale runs through retirement; the two LSM crash seams reopen correctly;
export streams under the window and import reproduces the portable logical
digest and bytes while publishing its own Store-local physical tree root.

### Phase 8: Full matrix, heavy lane, cutover and successor handoff

Run the decisive interleaving and the 4 GiB heavy lane; confirm
`InstalledCapabilityStatus` reports every constructed family truthfully
(Blob became `Present` in Phase 2 and Layout in Phase 4, with the accessor
that made them real); remove dead placement-observation
sessions, fake harness receipts, compatibility-only owners and static inventory
rows; confirm no historical read/write or migration lane remains under the
Store format policy; revise the
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
| Recovery-physics payload kinds | After current payload grammars are frozen | Phase 2 (declaration and generation), Phase 3 (frontier, terminal and reclaim), Phase 7 (LSM); each cutover replaces rather than retains an abandoned grammar |
| Writer binary and heavy generator | Immediately | First use in Phase 2 |
| Scheduler producers | Before their effects | Phase 2 (`BlobIngestPressure` and bounded ingest head), Phase 6, Phase 7 |

## QA Considerations And Verification

Architecture review must confirm one owner per truth: publication owner for
roots, Store blob owner for sessions, registry for family binding, and that no
mechanism crate, fixture, harness or in-memory model can produce a blob or
index byte. Lifecycle review must cover cancellation before and after the
first effect, resume across processes, close during an open read, and
shutdown with an active ingest. Persistence review must cover every crash seam
and the version fields of each new frame and payload. Review must distinguish
current-format recovery history from abandoned software formats, confirm all
affected producers and independent consumers use the current grammar, and reject
historical versions before effects without a migration or fallback lane.
Performance review must
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
  LSM payloads, retained-storage effects of reclaim. For the Store operator,
  explain the selected per-object head roster, checkpoint replacement and
  pruning rule, pre-effect capacity denial and terminal-head retirement;
  current-only supported formats, unsupported-version denial and explicit
  creation of fresh disposable development namespaces, never automatic reset.
- The C.5 record-path spec and `bounded-physical-record-access.md`: extent
  placement is an arena range; per-extent files are gone.
- `physical-recovery-and-reopen.md`: blob generation, resume, drop and
  membership redo; residue fates. For recovery implementers, distinguish
  current checkpoint-attested heads from independently replayed pending WAL,
  and state why selected controls or the global tip cannot replace a missing
  per-object head. Distinguish current crash/pruning recovery from unsupported
  historical-format migration; there is no migration or downgrade procedure.
- `physical-integrity-and-offline-verification.md`: new families, derived
  rebuild disposition, offline blob/index walk. For offline operators, name
  the head-tree block/roster family, unselected-node residue and mismatch
  findings, and the limit
  of whole-namespace self-consistent rewrite detection.
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
| C.13 integration | Facade integration and crash-and-reopen journey; sealed handoff to Runtime Integration Milestone 1 (the joined workload moves to S.12) | Facade placement, registry ownership, lifecycle composition |
| S.10 backup/repair | Backup holds as reachability edges, capsule and replication artifacts, authorized repair of authoritative chunk corruption | Reclaim proof protocol, drop publication, retirement law |
| Part II semantics | Release-proof issuers, tenant/key scope policy, cross-scope dedupe, content-defined chunking, Query pushdown, semantic traversal | Physical scopes, chunk identity, family registry, branch-agnostic sessions |
| Runtime-integration Milestone 12 | Chunk-backed range and streaming providers over `blobs()` | Constant-memory read contract, counters |

Foresight is paid at the frame formats, payload versions, the registry
binding, and the drop-publication protocol. It is not permission to implement
successors now. The first shipped feedback remains a native blob above the
window ingested, published and streamed through the production path.
