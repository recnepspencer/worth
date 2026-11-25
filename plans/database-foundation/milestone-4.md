# Milestone 4: Database Scale, Branches, and Sustained Operation

Governing roadmap: [Database Foundation Roadmap](roadmap.md#milestone-4-database-scale-branches-and-sustained-operation) (`L512–L617`).
Builds on: [Milestone 1](milestone-1.md), [Milestone 2](milestone-2.md) (Residency & Cold Guards), and [Milestone 3](milestone-3.md) (Application Home Durability, Adversarial Crash Oracle, and `StoreDurability.tla` / C6 Seeded DST).
Companion specifications:
- `worth-orchestration/milestones/semantic-git.md` — Whole-DB Semantic Git & Branching (Home PC, Phases 0–8; consumes M4's single-owned `HomeRootManifest` / `PinTable` / `BranchLease` substrate)
- `worth-orchestration/milestones/platform-dx-hardening.md` — Public WORTH Platform DX Hardening (`Home<P>`, `Request`, `ProductRef`, `HistoryRefValue`, `Disposition`, `Reportable`, `worth-dst`)
- [Deferred Work](../deferred-work.md#worth-store) — Deferred Store portfolios (C.11 tiering/LSM, S.10 backup/PITR, S.11 encryption, replication/sync)

Baseline verified against: `recnepspencer/worth` `origin/master` `0129df8c42` (`2026-11-24`).

> [!IMPORTANT]
> **How to read Rust code blocks in this spec:** Every Rust snippet labelled `// PROPOSED` specifies the **required target architecture or public DX contract** verified from external crates (`docs/dx-reference/store_m4_scale_and_branches.rs` + `trybuild` `.stderr` fences). Every `file:line` citation names verified source reality at `origin/master` `0129df8c42` that Milestone 4 fixes, extends, or retires.

---

## 1. Goal, Architectural Vision & Single-Owned Seam

Modern embedded and server databases split sustained-scale storage, copy-on-write branching, and schema-aware time travel into disconnected layers:

- **Embedded B-tree / LSM engines (SQLite, LMDB, RocksDB)** operate larger than memory and reclaim space, but have no native concept of durable semantic branching, retained-basis historical cursors that survive restarts without rebinding to `HEAD`, or transactional custody over derived application graphs.
- **Storage-branching engines (Neon, Dolt)** provide copy-on-write snapshots or row-level branches, but rely on separate garbage collectors disconnected from live reader loans, historical cursor lifetimes, and buffer-pool writeback fencing.

Milestone 3 proved that WORTH Store + Relational + World + Query survive arbitrary crashes at every I/O and recovery barrier. **Milestone 4 closes the Database Foundation Roadmap** by turning that crash-safe Home into a **sustained-operation database** that serves data and history far larger than resident memory, forks and retires branches in O(1) without copying base state, preserves exact retained-basis historical views across restarts and index discards, and reclaims unreachable physical space safely under concurrent writes, reads, and mid-reclamation crashes — all behind a **clean, ergonomic, zero-plumbing Rust DX** where chunking, eviction, cold summaries, and reclamation work invisibly underneath `Home<P>`, `Request`, and `ProductRef`.

### Core Design Principles

1. **Zero MVP shortcuts — one coherent authority across storage, residency, branching, and reclamation.**
   Every capability below shares one storage model, one reachability definition, and one refusal/disposition taxonomy:
   - Single-owned whole-Home O(1) copy-on-write root pinning (`HomeRootManifest`, `PinTable`, `BranchLease`, `NestedDropPolicy`),
   - Relational branch lifecycle (`fork` including durable historical-root reopening, `archive`, `delete`, permanent retired-name enforcement) with concurrent branch preparation and Home-frontier composition,
   - Deadlock-free real-allocation working-set eviction (`Arc<ResidentRegion>` reservations + cleared raw load caches + rolling bounded windows),
   - Single clean checkpoint/region format cutover (`CHK24`/`REG8`) with content-addressed chunked placement groups, consolidation-invariant semantic commitments, per-slot continuation pages, producer-verified cold summaries, mirror pin/lifecycle separation, and paged reopen directories (`G7`),
   - Resumable canonical-key historical cursor tokens (`RetainedBasisToken`) paired with reclaim-fenced in-process `ReaderLoan`s,
   - Writeback-fenced, cancel- and crash-monotone `ExtentAllocationGeneration` ABA protection across `ExtentArena` coordinates, and
   - Crash-safe, chained-recovery-interruptible physical space reclamation that measurably returns space while keeping foreground local edits `O(touched)`.

2. **Effortless Developer Experience (Hard DX Bar).**
   Application developers never manage `RegionChunkId`s, never invoke manual eviction hooks, never pass raw `AllocationGeneration` tags, and never decode raw `u64` cursor offsets. You open a `Home<P>` with a declarative `ResidencyBudget`, read and write larger-than-memory datasets through ordinary `ProductRef` / `Request` / `ValueReader` / `HistoricalCursor` APIs, and handle genuine exhaustion or pruned history through exhaustive typed `Reportable` / `Disposition` outcomes.

3. **Fast real-workload feedback first — M3 & `semantic-git.md` §12 cadence.**
   M4 deliberately avoids a multi-week upfront proof wall:
   - **M4.1 lands the O(1) whole-Home pin/lease substrate and Relational branch lifecycle FIRST** (touching disjoint crates from Relational chunking and immediately unblocking Home PC's Semantic Git Phase 2!).
   - **M4.2 lands real deadlock-free larger-than-memory eviction SECOND** with zero checkpoint format bump so real larger-than-memory workloads run immediately.
   - **M4.3 performs ONE clean format cutover (`CHK24`/`REG8`)** for chunking, continuation pages, cold summaries, mirror separation, and paged reopen.
   - **M4.4–M4.5** deliver historical cursors and writeback-fenced physical reclamation.
   - Stable pure storage kernels are formally verified in Verus + Lean 4 *as they are built*; full composed TLA+ + 10-scenario DST + `10×` multi-axis scaling + mutant-kill hardening runs in **M4.6** once APIs have settled, and cross-workspace certification + docs close the roadmap in **M4.7**.

4. **Assign each formal layer strictly to defects it can actually observe (zero ceremony).**
   All four formal layers (`Lean 4`, `Verus`, `TLA+`, extended `c6_durability_dst`) stay completely outside the ordinary `./loop edit` compile closure. Verus verifies **only** small, pure, executable Rust transition kernels (never `Mutex`/`Arc` glue); Lean 4 proves **only** unbounded mathematical invariants; TLA+ models **split crash-interruptible barriers**; Rust DST and courts verify real `Arc`/reservation memory custody, `try_lock` concurrency, `cold == warm` agreement, real `10×` scaling headroom, and observed production NDJSON trace replay into TLC.

5. **Headroom and equivalent-path scaling proofs, never brittle fixture-specific page totals.**
   Every scaling gate independently scales DB size, record cardinality, and retained commit history by `10×` while holding a local edit's semantic scope fixed, asserting asymptotic headroom (`O(1)` fork, `O(touched)` local edit, bounded reopen & checkpoint under the same resident allowance) rather than hardcoded page counts.

### Single-Owned Cross-Programme Seam (Zero Double Ownership)

| Programme | Owns (Single Authoritative Owner) | Explicitly Does NOT Own |
|---|---|---|
| **Store Milestone 4** *(this spec, Work PC)* | 1. **Whole-Home O(1) CoW Pin & Lease Substrate (`D3b`):** `HomeRootManifest` (Native + World + Signal + installed program digest + **persistent product-map root**), durable `PinTable`, `BranchLease` (`Active → Expiring → Expired`), `NestedDropPolicy`, Memory-backend parity, Verus `PinTable::{acquire, release, expire_lease}`, Lean `StoreGCPinSafety` & `LeaseExpiryNoReaderLoss`, and TLA+ `PublishForkPin` / `ExpireForkLease` — wired directly into Store's `reclaim_guard` and reopen.<br>2. **Relational Branch Lifecycle & Concurrent Preparation (`D3a`):** Durable historical-root fork reopening (`RetainedSourceDurabilityUnsupported`), durable `archive` + `delete` + retired-name durability, and stale-candidate Home-frontier composition at FIFO publication.<br>3. **Bounded Larger-Than-Memory Residency (`D1`, `D2`):** Real-allocation `Arc<ResidentRegion>` eviction, rolling-window scans/maintenance, `CHK24`/`REG8` chunked placement groups + per-slot continuation pages + consolidation-invariant semantic commitments + producer-verified cold summaries + mirror separation + paged reopen/checkpoint (`G7`).<br>4. **Historical Retained-Basis Cursors (`D4`)** and **Writeback-Fenced, Durable-Epoch Safe Physical Reclamation (`D5`, `D6`)**. | Higher-level 3-way semantic merge (`MonotonicCounter`, `AdditiveSet`, `MergeSession`, `ReviewPacket`, `SemanticDiff`), sandboxed `EnvironmentFork` outbound quarantine / `ImpersonateInSandbox` / `into_dry_run::<V2>`, `BranchCatalog` named-ref policy, or `worth-ui`. |
| **Semantic Git & DB Branching** (`worth-orchestration/milestones/semantic-git.md`, Home PC) | Consumes M4's `HomeRootManifest`, `PinTable`, `BranchLease`, Verus `PinTable`, Lean `StoreGCPinSafety`/`LeaseExpiryNoReaderLoss`, and TLA+ `PublishForkPin`/`ExpireForkLease` directly (re-pointing `semantic-git.md` Phase 2 item 1 and §9 proofs to M4's single owner — see §13). Owns `BranchCatalog` named-ref policy, `EnvironmentFork` sandbox/posture/dry-run UX, `BranchProtection`, `MergeSession`, and 3-way reconciliation. | Re-implementing `PinTable`, `StoreGCPinSafety`, physical extent reclamation, buffer-pool writeback fencing, or cold residency eviction. |
| **Platform DX Hardening** (`worth-orchestration/milestones/platform-dx-hardening.md`, Mac Studio) | Owns the public `Home<P>`, `Request`, `ProductRef`, `HistoryRefValue`, `Disposition`, `Reportable`, `worth-dst`, and `W1.1` `RuntimeWorldRetentionPort` public surface. *(M4 defines the narrow `RuntimeWorldRetentionPort` contract directly in `worth-runtime-world` if M4.1 lands before Mac `W1.1`, using the exact same shape so `W1.1` extends it seamlessly with zero rewrite.)* | Physical storage chunking, buffer-pool writeback fencing, or physical extent reclamation. |
| **Deferred Portfolios** ([deferred-work.md](../deferred-work.md#worth-store)) | C.11 tiering/relocation/LSM/export-import, S.10 backup/PITR/disaster-recovery, S.11 encryption/key lifecycle, and distributed replication/sync remain deferred and honestly unavailable. | M4 implements **basic safe reclamation** and **bounded residency** only — never silently claiming deferred S-qualifications or unrebound legacy formal models. |

---

## 2. Verified Pre-M4 Baseline & Concrete Gap Inventory (`origin/master` `0129df8c42`)

### 2.1 Existing M1–M3 Foundation Reused Directly (No Reinvention)

Source inspection across `recnepspencer/worth` (`origin/master` `0129df8c42`) establishes the exact existing subsystems M4 builds on:

1. **Store Physical Buffer Pool (`workspaces/worth-store/crates/worth-store-buffer-pool/src/physical_residency/`):**
   - `pool.rs:46-62`: `PhysicalFrameKey { store, coordinate: RecordFrameCoordinate }`, `PhysicalCandidateFrameKey`, `PhysicalBoundedFrameKey`.
   - `pool/writeback_claim.rs:209`: dirty frame writeback claim admission and lifecycle.
   - `pool/eviction.rs`, `pool/range_invalidation.rs` (`invalidate_clean_range`, refusing pinned or dirty overlap), `pool/pin_lifecycle.rs`, `limits.rs` (`PhysicalResidencyLimits`).
   - `integrity_validation.rs`: `PhysicalResidentFrameGeneration` *(explicitly process-local byte-image identity, NOT a durable allocation generation)*.
2. **Store Physical Retirement & Extent Release (`workspaces/worth-store/crates/worth-store/src/physical_runtime/`):**
   - `durability/retention/admission.rs:24-41`: `PhysicalPublicationAdmission` tracking `charged_bytes`, `pending`, `generations`, `garbage: BTreeMap<RetiredArtifact, RetainedGarbage>`, `wal_publications`, and displaced capacity leases.
   - `durability/retention/retirement.rs:24-150`: `RETIREMENT_DOMAIN` (`store.physical.retirement.v2`), `RETIREMENT_{INTENT,COMPLETION,EXTENT_INTENT,EXTENT_COMPLETION}`, `RetiredArtifact::{Segment, Extent, Arena}` (with durable generations on segments and `DurableExtentRecordPlacement::extent_generation()`), `RetirementRemovalPermit`, `PhysicalRetirementDenial::{Absent, Protected, Unresolved, Retained, ...}`.
   - `record_serving/publication/director/retirement_release.rs:125` & `retirement_release/resume.rs:101`: existing free-space release ordering and recovery double-release guard (`RetirementReleaseProjection`).
   - `record_serving/publication/director/extent_copy/resolution.rs:157-161`: candidate extent cancellation path releasing reservations once a checkpoint covers the cancellation WAL record (without publishing a successor root).
   - `stability/retention/registrations/{reclaim_guard.rs:5, retained_snapshot.rs, terminal_head_hold.rs}`: in-process `RootProtectionRegistry`, boundary-locked by `tools/boundary-check/config/road1.toml:87-107` to `stability`, `current_root_owner`, `durability/retention`, `instance/construction/record_serving.rs`, `record_serving/publication/director`, `serving_runtime.rs`, and `record_reader.rs`.
3. **Relational Residency, Branch Lifecycle & Current Codecs (`crates/worth-relational-*`):**
   - `worth-relational-storage/src/storage/residency/{cell.rs:55-160, lease.rs:18-93}`: `RelationalRegionResidency` (`leases: AtomicUsize`, `ResidencyState::{Cold, Head, Full, Damaged}`), `RelationalResidencyLease<Head | Full>`, per-cell `retain_members` custody (`cell.rs:103-117`, `worth-store-relational/src/cold_source/member_custody/`).
   - `worth-relational-branch/src/branch/{fork.rs, coordination.rs, historical_selection.rs, basis_retention.rs}` & `worth-relational/src/branch/{fork_publication.rs:143, fork_target_preparation.rs:37, lifecycle_archive.rs:12}`: durable branch fork & archive transitions, branch-local `RelationalBranchCoordinationCell`, unmanaged Native fork already sharing source root by `Arc`, and exact kind scans (`worth-relational-branch/src/visibility/materialization/read_records/reader/truth_kind_scan/directory.rs:19`).
   - Current durable formats: **`CHK23`** (`worth-relational-durability/src/durability/log/native_checkpoint_codec.rs:9`) and **`REG7`** (`worth-relational-storage/src/storage/residency/region_image.rs:13`), plus existing leaf consistency checks (`checkpoint_region_consistency.rs:24`).
4. **Store M3 Formal & Seeded DST Harness:**
   - `specs/tla/store_m3/StoreDurability.tla` (including multi-step interruptible recovery actions at `:291`) and `workspaces/worth-store/crates/worth-store-court-application/.../c6_durability_dst.rs`.

### 2.2 Verified Gap Inventory (`G1–G7`)

| ID | Gap | Verified Source Location (`0129df8c42`) | Why Naive Patching Fails / Concrete Consequence |
|---|---|---|---|
| **G1** | **No Relational working-set eviction, leaky `OnceLock`/raw caches & monolithic whole-root demands** | `worth-relational-storage/src/storage/residency/cell.rs:76-99`; `retention.rs:95` (`OnceLock` root-lifetime leases); `cell_load_cache.rs:8` (strong `Arc` `raw_head`/`raw_full`); `lease/cell_custody.rs` (`retire_native`/`Clone` outside lock); `milestone-2.md:335-344` (whole-root/mirror declared demand & whole-root probe escalation) | Even when `leases == 0`, root-retained `OnceLock`s and `raw_head`/`raw_full` keep `Arc<ResidentRegion>` allocations alive for the entire lifetime of a root. Worse, `Clone for RegionLeaseCustody` can bump `0 → 1` outside the cell state lock, and evicting cell B while holding cell A's state mutex causes an AB/BA deadlock. Whole-root scans/maintenance/Full retention blow the resident allowance immediately on DBs larger than memory. |
| **G2** | **Monolithic whole-partition & single-slot residency, consolidation & CoW edit** | `worth-relational-storage/src/storage/residency/cell.rs:56-67` (`ResidentRegion { partition: PartitionState }`); `region_image.rs:60` (monolithic slot history & adjacency); `milestone-2.md:325,420-427` (Full consolidation & CoW edit require full residency witness) | A single placement group larger than the resident allowance — OR a single record with a long version history, OR a single high-degree vertex adjacency list with small individual values — cannot be read, edited, or checkpoint-consolidated without exceeding the memory allowance. Large values and in-flight fills lack bounded streaming ceilings. |
| **G3** | **Unauthenticated/incomplete cold summaries & mirror duplication** | `plans/database-foundation/milestone-2.md:439-445`; `worth-relational-storage/src/runtime/state/subsystems/storage/residency_demand.rs:6` | Cold kind/adjacency summaries do not exist at chunk granularity (so first scans/traversals warm whole partitions), and branch pins/lifecycle remain coupled to the mirror so the mirror stays a separate working copy of `main`. |
| **G4** | **No durable O(1) whole-Home pin/lease substrate; missing Relational historical-fork reopening & Home-frontier concurrent-prep composition** | `worth-store/src/physical_runtime/stability/retention/registrations/reclaim_guard.rs:5`; `worth-relational-branch/src/branch/fork.rs:36` (`RetainedSourceDurabilityUnsupported`); `worth-relational/src/branch/fork_publication.rs:143` | `RootProtectionRegistry` is in-process only (`pub(in crate::physical_runtime)`), so there is no durable cross-restart `HomeRootManifest` / `PinTable` / `BranchLease` for O(1) whole-Home forks. Relational refuses durable historical-root reopening (`RetainedSourceDurabilityUnsupported`), and two branches preparing concurrently from the same Home frontier can overwrite each other's component root at publication unless revalidated and composed onto the current Home frontier at the FIFO reservation. |
| **G5** | **Process-local retained bases & no canonical-key resumable cursor token** | `worth-relational-branch/src/branch/basis_retention.rs:24` (refuses `ForeignRuntime` on `runtime_instance_id`); `historical_selection.rs:52-99` | Retained bases and leases are strictly process-local today. Resuming a historical query across a restart or after physical index discard has no durable basis+query-digest+canonical-key token and no atomic resume-vs-prune loan fence, risking either silent rebinding to `HEAD`, skipped/duplicated rows after index discard, or unbounded history leaks if cursors pinned history forever. |
| **G6** | **Semantic-to-physical reclamation wiring + ungenerationed `ExtentArena` frame coordinate + writeback/cancel/double-free fencing** | `worth-store-buffer-pool/src/physical_residency/{pool.rs:46-62, pool/writeback_claim.rs:209}`; `worth-store/src/physical_runtime/durability/retention/{admission.rs:24-41, retirement.rs:24-150}`; `retirement_release.rs:125`, `resume.rs:101`, `extent_copy/resolution.rs:157-161` | `RecordArtifactFile::ExtentArena { arena }` + offset inside `RecordFrameCoordinate` carries no durable allocation generation. A paused generation-`g` writeback can overwrite freshly reallocated `g+1` bytes *before* any read-time check runs; candidate-extent cancellation can return a range without a root publication; and a crash during recovery's own free-space publication before completion can double-free a reused range. Existing per-cell `retain_members` custody (`cell.rs:103-117`) and `RootProtectionRegistry` must be united with durable pins inside `Reach`. |
| **G7** | **Unbounded reopen & linear checkpoint-cutover metadata scaling** | `storage/residency/record_directory/portable.rs:14` (1 entry/record); `runtime/state/subsystems/record_identity/mod.rs:124` & `checkpoint_restore/record_identity.rs:14,49,127` (eager restore of every slot generation high-water mark, reusable slot, pending reservation, and append frontier); `checkpoint_restore/cold_directory.rs:94` (commit-graph & patch-position maps over all manifest commits); `publication_cutover.rs:49` -> `selected_manifest_pins.rs:104` (linear route scan at checkpoint cutover) | Even if every record payload fits in a chunk, high record cardinality, outstanding reservations, or a long commit history exhausts the resident allowance during reopen *before the first chunk read*. Meanwhile, checkpoint cutover runs `scan_rebuild` linearly over every route in the selected root. |

---

## 3. Shared Vocabulary, Authoritative Types & Refusal Taxonomy

This section defines the single authoritative Rust types, invariants, and refusal/disposition contracts shared across Store, Relational, World, Host, Semantic Git, and all 4 formal verification layers.

### 3.1 Declarative Residency & Streaming Budgets (`worth-store-application` & `worth-relational-contracts`)

```rust
// PROPOSED — Declarative user-facing residency budget (zero internal cache plumbing exposed)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResidencyBudget {
    /// Maximum resident bytes allowed for active branch head working sets.
    pub head_working_set_bytes: u64,
    /// Separate ceiling for historical reads, cursors, and time-travel windows
    /// (opening a historical view can NEVER pin or starve active head leases).
    pub historical_working_set_bytes: u64,
    /// Maximum aggregate bytes concurrently in flight across cold chunk fills and
    /// large-value streams (bounds queued I/O and prevents hidden whole-value buffers).
    pub max_in_flight_fill_bytes: u64,
    /// Fixed streaming window size for large-value readers and rolling scans.
    pub streaming_window_bytes: u32,
}

/// Exact immutable basis identity verified across every update, eviction, and rewarm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BasisIdentity {
    pub content_digest: [u8; 32],
    pub schema_carrier_digest: [u8; 32],
    pub level: ResidencyLevelKind, // Head | Full
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResidencyPressureSnapshot {
    pub head_resident_bytes: u64,
    pub head_borrowed_bytes: u64,
    pub historical_resident_bytes: u64,
    pub historical_borrowed_bytes: u64,
    pub raw_load_cache_bytes: u64,
    pub in_flight_fill_bytes: u64,
    pub evictions_completed: u64,
    pub busy_victim_skips: u64,
}
```

### 3.2 Whole-Home `HomeRootManifest`, `PinTable` & `BranchLease` (`worth-store`, `worth-store-application`, `worth-runtime-world`)

```rust
// PROPOSED — Single-owned whole-Home frontier & durable pin/lease substrate
// Owned by Store Milestone 4 (M4.1) and consumed verbatim by Semantic Git Phase 2!

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HomeIncarnation([u8; 16]); // domain-separates cursors, pins, leases, forks

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PinId(u64); // strictly monotone durable pin identifier per HomeIncarnation

/// Fixed-size, already-sealed whole-Home frontier pinned in O(1) under Store's reclaim fence.
/// Covers ALL components required by M4 AND Semantic Git §4.1 (zero missing product-map root!).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HomeRootManifest {
    pub format_version: u16,
    pub home_incarnation: HomeIncarnation,
    pub frontier_sequence: u64,
    pub native_root: PhysicalRootPointer,
    pub world_root: PhysicalRootPointer,
    pub signal_root: PhysicalRootPointer,
    pub product_map_root: PhysicalRootPointer, // persistent product-name -> ProductRef map
    pub installed_program_digest: [u8; 32],
    pub manifest_digest: [u8; 32],             // canonical SHA-256 over all fields above
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinKind {
    /// Permanent while catalog entry exists (named branches, archived branches, immutable tags).
    PermanentCatalog,
    /// Expirable lease (EnvironmentForks, MergeSessions, bounded temporary retentions).
    ExpirableLease { expires_at_maintenance_tick: u64 },
    /// In-flight durable decision fence (released atomically when decision settles or rolls back).
    InFlightDurableDecision,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NestedDropPolicy {
    RefuseIfPinnedByChild,
    DetachParentKeepChildRoots,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeaseLifecycleState {
    Active,
    /// Atomically fences NEW reader loans (`LeaseExpired`) while retaining underlying
    /// root pin & parent overlay edges until `active_reader_loans == 0`.
    Expiring,
    /// `active_reader_loans == 0` and no surviving child overlay depends on this pin;
    /// pin is durably released to Store's reclamation horizon.
    Expired,
}
```

### 3.3 `CHK24` / `REG8` Chunked Placement & Per-Slot Continuation Carriers (`worth-relational-storage` & `worth-relational-durability`)

```rust
// PROPOSED — Consolidation-invariant semantic chunk entry vs. checkpoint locator binding

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SlotRange {
    pub start_slot_inclusive: u64,
    pub end_slot_exclusive: u64, // invariant: start < end; fixed partition boundaries
}

/// Folded into the selected root's Merkle commitment.
/// Strictly INVARIANT across history flattening/consolidation (milestone-2.md:420)!
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkSemanticEntry {
    pub slot_range: SlotRange,
    /// Canonical hash of logical chunk meaning (head + history records/relations),
    /// independent of layered vs. flattened physical encoding!
    pub semantic_chunk_commitment: [u8; 32],
    /// Deterministically derived from (and verified against) canonical chunk projection BEFORE sealing.
    pub head_summary: ChunkKindAdjacencySummary,
    pub history_summary: ChunkKindAdjacencySummary,
}

/// Authenticated in the CHK24 checkpoint manifest; updated freely on history consolidation
/// without altering `ChunkSemanticEntry.semantic_chunk_commitment` or breaking cursors!
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkLocatorBinding {
    pub slot_range: SlotRange,
    pub head_locator: RelationalDurableMemberLocator,
    pub history_locator: RelationalDurableMemberLocator,
    /// Optional root of bounded intra-slot continuation tree for high-version-count records
    /// or high-degree vertex adjacency lists (`region_image.rs:60`).
    pub continuation_directory_locator: Option<RelationalDurableMemberLocator>,
}
```

### 3.4 Resumable Historical Cursor Token & Loan Guard (`worth-relational-branch` & `worth-query-host`)

```rust
// PROPOSED — Durable resumable cursor token (NOT an unbounded retention pin!)
// Extends RelationalBranchBasisDescriptor with durable incarnation, query digest, and canonical position.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetainedBasisToken {
    pub format_version: u16,
    pub home_incarnation: HomeIncarnation,
    pub branch_origin: BranchOriginIdentity,
    pub commit_id: CommitId,
    /// Selected root's consolidation-invariant semantic Merkle commitment (survives history flattening!).
    pub root_semantic_commitment: [u8; 32],
    pub schema_version: SchemaVersionIdentity,
    /// Canonical digest of the query projection/filter/ordering contract.
    pub query_digest: [u8; 32],
    /// Canonical key-order resume position (NEVER a physical index offset or tree slot —
    /// guarantees zero skipped or duplicated rows after physical index discard!).
    pub canonical_key_position: CanonicalCursorKey,
    // Private constructor / opaque wire encoding (`whc1_...`) outside owning crate.
}

/// Exhaustive typed historical read/resume refusal (implements `worth-execution::report::Reportable`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HistoricalReadDenial {
    /// Basis was legitimately pruned past the retention horizon (or unretained across restart).
    /// Disposition: `Final` (never silently rebinds to HEAD!).
    HistoryUnavailable { commit_id: CommitId },
    /// Schema/version boundary makes the requested historical meaning unsupported.
    /// Disposition: `CorrectInput` / `Final`.
    UnsupportedHistoricalMeaning { schema_version: SchemaVersionIdentity },
    /// Persisted bytes failed integrity, Merkle commitment, or allocation-generation validation.
    /// Disposition: `Escalate` (puts Store into `ServingRequiresInspection` on media corruption).
    CorruptStorage { subject: RelationalStorageSubject },
    /// Active resident borrows genuinely saturated the configured historical working-set budget.
    /// Disposition: `Retry(RetryHint::MemoryPressure)`.
    MemoryExhausted { requested_bytes: u64, allowance_bytes: u64 },
    /// Caller cancelled or exceeded execution deadline before completing read.
    /// Disposition: `Retry(RetryHint::CancelledOrDeadline)`.
    RequestStopped,
}
```

### 3.5 Durable `ExtentAllocationGeneration` & Writeback Fence (`worth-store-physical-format` & `worth-store-buffer-pool`)

```rust
// PROPOSED — Scoped strictly to ungenerationed `ExtentArena` physical coordinates
// (Zero Relational format change — Relational keeps content-addressed `RelationalDurableMemberLocator`!)

/// Crash- and cancel-monotone durable allocation identity for one `ExtentArena` byte range.
/// Established on durable WAL admission BEFORE candidate I/O and strictly advanced on every
/// extent cancel (`extent_copy/resolution.rs:157`) OR extent retirement barrier before reissue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExtentAllocationGeneration {
    pub root_publication_generation: u64,
    pub extent_fence_seq: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArenaByteRange {
    pub arena_id: ExtentArenaId,
    pub offset: u64,
    pub length: u64,
}

impl ArenaByteRange {
    /// Pure overlap check used across split and coalesced extents in Verus & production!
    pub const fn overlaps(self, other: Self) -> bool {
        self.arena_id.raw() == other.arena_id.raw()
            && self.offset < other.offset.saturating_add(other.length)
            && other.offset < self.offset.saturating_add(self.length)
    }
}

/// Carried on `worth-store-buffer-pool` arena frames AND `writeback_claim.rs:209` I/O admission.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArenaExtentFrameIdentity {
    pub range: ArenaByteRange,
    pub generation: ExtentAllocationGeneration,
}
```

---

## 4. Layer 1 — O(1) Whole-Home Pin Substrate, Relational Branch Lifecycle & Concurrent Preparation (`M4.1`)

### 4.1 Single-Owned Whole-Home `HomeRootManifest` & Durable `PinTable` (`D3b`)

#### Authoritative Design
1. **Atomic whole-Home sealing:** Every settled Home frontier seals one fixed-size `HomeRootManifest` referencing `(native_root, world_root, signal_root, product_map_root, installed_program_digest)`. Because `product_map_root` is included inside the sealed manifest, opening or forking an old frontier resolves *every* product consistently at that exact frontier — never resolving Product B through a newer product map.
2. **O(1) pin acquisition under Store's reclaim fence:**
   - Calling `pin_table.acquire(manifest, kind, parent_overlay_pin)` runs inside Store's current-root / reclamation fence (`stability/` + `durability/retention/`), writes a single constant-size durable pin record, and returns `PinId` — **zero** base-state copy and **zero** record scans.
   - `PinTable` persists inside Store's durable anchor state and is **restored on reopen BEFORE any maintenance or reclamation pass executes**.
   - **Immediate wiring in M4.1:** Store's existing `reclaim_guard.rs:5` and displaced-segment retirement consult `PinTable` immediately in M4.1 (never waiting for M4.5!). Protection is **per-reachable-root/member closure**, NOT a coarse global old-root veto: an active pin on an older frontier protects strictly its reachable roots and transitive parent-overlay deltas while unrelated displaced garbage continues to reclaim cleanly.
3. **Reader-safe lease expiry (`Active → Expiring → Expired`) & nested overlays:**
   ```text
   Active --(maintenance_tick >= expiry)--> Expiring --(active_reader_loans == 0 && live_child_pins == 0)--> Expired (pin dropped)
                                               |
                                               +-- atomically fences all NEW reader loans (`LeaseExpired`)
                                               +-- retains root pin & parent overlay edges for EVERY admitted loan & surviving child
   ```
   - `NestedDropPolicy::RefuseIfPinnedByChild` refuses explicit drop while any child overlay references this pin; `DetachParentKeepChildRoots` detaches the parent handle while preserving the parent's delta roots transitively until the last child/grandchild expires and drains its reader loans.
   - Named branches, archived branches, and immutable tags carry `PinKind::PermanentCatalog` and **never expire**.
   - **Memory-backend parity:** In-memory Homes enforce the exact same `PinTable` / `BranchLease` / `Active → Expiring → Expired` / `NestedDropPolicy` state machine within the live Home lifetime.
4. **Catalog-to-pin crash binding:** Publishing a durable branch or fork catalog entry binds atomically with `PinId` acquisition (or records a recoverable two-step intent so crashing before catalog settlement cleans up the orphan pin on reopen, and crashing after settlement preserves both catalog and pin).
5. **Crate-DAG legality (`road1.toml:392-520`):** World defines the narrow `RuntimeWorldRetentionPort` (matching Mac `W1.1`'s exact port role; extended seamlessly when `W1.1` lands). `worth-store-application` implements it over Store's physical `PinTable`; `worth-query-host` composes `worth-store-application`. World never imports Store; `worth-store/src/physical_runtime` sees only opaque `PinId` and physical root pointers (zero branch/MVCC vocabulary per `road1.toml:525-560,650-685`).

#### Why This Design (Reviewer Discoveries Prevented)
- **Eliminates double ownership with `semantic-git.md` Phase 2** *(Fable r1 #1, Astra r1 #5)*: Earlier drafts duplicated `RootPinTable` vs. `PinTable`, omitted `product_map_root`, omitted `NestedDropPolicy` & Memory parity, and cited `RuntimeWorldRetentionPort` as already existing. Making M4 the single owner (and moving `M4.1` first) eliminates all duplication and unblocks Semantic Git Phase 2 immediately.
- **Pins must protect bytes the day they ship** *(Astra r1 #6, Fable r1 #13)*: Exposing durable pins in M4.1 while deferring reclamation wiring to M4.5 would let Store's already-live displaced-segment retirement delete a pinned root's bytes in between! Wiring `PinTable` into `reclaim_guard.rs:5` and reopen right in M4.1 closes that window completely.
- **No coarse global old-root veto** *(Astra r1 #6)*: Treating any live pin as "freeze all physical reclamation" would silently break space return whenever a long-lived historical tag or audit fork exists. Reachability-scoped pin protection reclaims unrelated displaced segments while preserving pinned roots and grandchild overlays.

---

### 4.2 Relational Branch Lifecycle (`D3a`)

Relational branches inside a single Home are distinct from whole-Home `EnvironmentFork`s (`D3b`) — pinning an entire `HomeRootManifest` per Relational branch would prevent a deleted branch from releasing anything covered by a sibling branch's Home pin *(Fable r1 #2)*.

1. **Inventory of M2 baseline + exact M4 additions:**
   - M2 (`worth-relational-branch/src/branch/fork.rs`, `worth-relational/src/branch/{fork_target_preparation.rs:37, fork_publication.rs:143, lifecycle_archive.rs:12}`) already implements durable branch `fork` (sharing immutable source roots via `Arc` on unmanaged paths), durable `archive`, durable `delete`, retired-name checks, and stale-source rechecks.
   - **M4 closes the real remaining gaps:**
     1. **Retire `RelationalForkDenial::RetainedSourceDurabilityUnsupported` (`fork.rs:36`):** Forking from a historical/retained root records its exact retained source basis durably in `RelationalBranchTransitionArtifact` and reopens cleanly across crashes without copying base chunks.
     2. **Preserve divergent histories over shared immutable chunks:** Divergent branches sharing physical chunks/regions carry distinct `(branch_id, incarnation, ancestry, selected_root)` metadata across restarts; equal-looking values or shared physical bytes never collapse distinct semantic histories.
     3. **Preserve durable `archive` and crash-safe `delete`:** Archiving preserves the branch's selected root for historical reads while refusing new writes; deleting retires the branch's name permanently (crashing at any barrier never resurrects the branch or allows unsafe name/ID reuse) and drops ONLY the deleted branch's selected root reference while leaving sibling branches and active reader loans untouched.

---

### 4.3 Concurrent Independent Branch Preparation, Home-Frontier Composition & Lock Order

1. **Zero whole-Store preparation lock:** Preparation, cold chunk reads, continuation-page loads, and invariant/uniqueness validation on distinct branches run concurrently under branch-local `RelationalBranchCoordinationCell::enter` (`coordination.rs:67`) without holding any whole-Store or cross-branch preparation lock.
2. **Exact lock ordering (extending M2 `milestone-2.md:175`):**
   ```text
   1. Lifecycle admission
   2. Branch coordination cell (strictly single-branch; never acquire a 2nd branch's cell!)
   3. Branch state guard
   4. Patch-position FIFO reservation (linearization point; FIFO waiters observe cancellation/deadline)
   5. Registry membership / name locks (leaf within Relational)
   6. Store current-root / PinTable reclamation fence (strict leaf during physical cutover; NEVER calls back into Relational!)
   ```
   Memory reservation for cold demand happens **before** step 2 (or outside any cell `state` mutex per §5.2), guaranteeing zero AB/BA cycles across residency eviction and publication.
3. **Stale-candidate revalidation & Home-frontier composition at FIFO cutover *(Astra r1 #8)*:**
   - *Counterexample:* Branch A and Branch B both prepare concurrently against Home frontier `F0`. Branch A enters the FIFO reservation first and publishes frontier `F1`. If Branch B installs its pre-assembled frontier manifest built from `F0`, it **silently overwrites Branch A's component root** even though Branch B's own branch generation did not move!
   - *Fix:* Inside the FIFO patch-position reservation (`fork_publication.rs:143`):
     1. Revalidate Branch B's own `(branch_generation, lifecycle_posture)` — catching concurrent same-branch commit (`StaleCandidate`), `archive` (`BranchArchived`), or `delete` (`BranchDeleting`) cleanly before any durable I/O.
     2. Compose Branch B's prepared branch delta onto the **current** Home frontier `F1` to produce `F2` (retaining Branch A's published root untouched!) rather than installing a stale pre-assembled frontier manifest.

---

## 5. Layer 2 — Deadlock-Free Working-Set Eviction & Bounded Residency (`M4.2`)

### 5.1 Real-Allocation Memory Accounting (`D1`)

```rust
// PROPOSED — Eviction bound tracks REAL resident allocations, NOT merely `leases == 0`
pub(crate) struct WorkingSetAccountant {
    head_allowance: ByteBudget,       // separate from historical_allowance
    historical_allowance: ByteBudget, // historical reads/cursors can NEVER starve head!
    in_flight_fill_allowance: ByteBudget,
    // Every live `Arc<ResidentRegion>` and every `raw_head`/`raw_full` buffer holds an
    // `ExecutionMemoryReservation` returned strictly on `Drop` of the LAST strong `Arc`!
}
```

#### Why This Design (Reviewer Discoveries Prevented)
1. **Zero lease count did NOT return memory** *(Astra r1 #2, Fable r1 #3)*:
   - `RelationalRegionRetention` (`retention.rs:95`) stored admitted `RelationalResidencyLease`s inside `OnceLock`s for the entire lifetime of a root — scanning a long-lived root could exhaust the allowance even after every read operation finished!
   - `cell_load_cache.rs:8` retained strong `Arc`s on `raw_head`/`raw_full` independently of `ResidencyState`.
   - `RegionLeaseCustody::retire_native()` dropped the count while keeping the `Arc<ResidentRegion>` alive, `into_owned_contents` / `source_retirement_resident()` bypassed the count, and `Clone for RegionLeaseCustody` (`lease/cell_custody.rs`) could increment `0 → 1` outside the cell state lock!
   - **Fix:**
     1. Separate durable member custody (`NativeSourceMemberState`) from operation-scoped resident borrows in `RelationalRegionRetention` (`retention.rs:95`) — a long-lived root retains member custody without pinning resident `Arc<ResidentRegion>` payloads in `OnceLock`s.
     2. Clear `raw_head` and `raw_full` (`cell_load_cache.rs:8`) under the eviction protocol so transitioning to `Cold` genuinely drops the cell's strong references.
     3. Fix `RegionLeaseCustody::clone` so a retired custody can never re-mint an active count; keep `ExecutionMemoryReservation` attached directly to `ResidentRegion` (and raw cache buffers) so memory credit returns **strictly when the last strong `Arc` drops**. Treat `leases == 0` strictly as a **victim-selection heuristic**, never as proof that bytes were freed.
2. **Exact evictability rule (M2 carry law)** *(Fable r1 #3)*:
   - Cells created via `resident_with_commitment` have `backing: None`, and `checkpoint_bindings` can be shared across runtimes. Evicting an unbacked cell or a cell whose locator is absent from *this* home's verified member directory would lose committed data!
   - **Rule:** A cell/chunk is evictable **iff** (a) it has no dirty unpersisted deltas, (b) it is not sticky `Damaged`, and (c) its exact `(content_digest, locator)` is present in **this home's verified durable member directory**.

### 5.2 Deadlock-Free `try_lock()` Eviction & Basis-Preserving Rewarm

#### Why Demand-Triggered Eviction Risks AB/BA Deadlock *(Fable r1 #4)*
`cell.rs` holds the cell's `state` mutex across cold load I/O. If Thread 1 holds cell A's lock while trying to evict cell B, and Thread 2 holds cell B's lock while trying to evict cell A, both threads deadlock permanently.

#### Authoritative Lock-Free-Ordering Eviction Protocol
1. **Reserve BEFORE taking any cell `state` mutex:** Before locking target cell `X` to load a cold chunk/region/page, request `ExecutionMemoryReservation` from `WorkingSetAccountant` (head pool or historical pool according to demand class).
2. **Victim scan uses `try_lock()` only:** When the allowance needs headroom, the evictor scans candidate victims in clock/LRU order (preferring `leases == 0`, demoting `Full → Head` before evicting `Head → Cold` when a head cell is hot), acquires each candidate's `state` mutex strictly via **`try_lock()`**, and **skips any busy cell** — zero blocking lock acquisition while scanning victims!
3. **Genuine saturation fails closed:** If no evictable victim can free the required bytes because all resident allocations hold active borrows (or are unbacked dirty cells), reservation returns typed `RelationalStorageUnavailableCause::MemoryExhausted` **before** taking `X`'s lock — never panicking, never evicting an actively borrowed cell out from under memory accounting, never returning empty data, and never skipping validation.
4. **Basis-preserving rewarm:** Rewarming a previously evicted cell checks exact `BasisIdentity = (content_digest, schema_carrier_digest, level)` against the target root's leaf commitment before serving, ensuring updates + eviction + rewarm can never surface a stale warm view as current.

### 5.3 Converting Whole-Root / Whole-Mirror Demands to Rolling Bounded Windows

M2 (`milestone-2.md:335-344`) used two policies that are fundamentally incompatible with larger-than-memory serving *(Fable r1 #4)*:
- **Whole-root / whole-mirror declared demand** for scans, maintenance, inspection, and the Full retention pass, and
- **Probe-retry escalation "to the whole root"** when probe retries hit their per-round cap.

**M4 replaces both with bounded rolling windows:**
- Scans, inspection, maintenance, and Full retention passes advance partition-by-partition and chunk-by-chunk (acquiring and releasing a bounded window lease of at most `K` chunks at a time, never pinning the whole root or whole mirror simultaneously).
- Probe-retry loops switch from "escalate to whole root" to **bounded chunk-window fallback iteration** (iterating remaining unprobed chunks sequentially through the rolling window budget), returning `MemoryExhausted` only if a single bounded window itself cannot fit within the configured allowance.

---

## 6. Layer 3 — Single-Cutover Chunked Placement Groups (`CHK24`/`REG8`), Per-Slot Continuations, Cold Summaries & Paged Reopen (`M4.3`)

### 6.1 Single Clean Format Cutover (`CHK23`/`REG7` → `CHK24`/`REG8`)

Current `origin/master` (`0129df8c42`) writes **`CHK23`** (`worth-relational-durability/src/durability/log/native_checkpoint_codec.rs:9`) and **`REG7`** (`worth-relational-storage/src/storage/residency/region_image.rs:13`).
- M4.2 (eviction + budgets) performs **zero** checkpoint format bump.
- M4.3 performs **one single clean cutover** to **`CHK24`** / **`REG8`** carrying chunk directories, per-slot continuation pointers, producer-verified chunk summaries, and paged reopen directories.
- **Disposition of pre-cutover `CHK23`/`REG7` homes:** Per the repository's clean-cutover discipline, `CHK24` refuses obsolete `CHK23`/`REG7` frames at reopen with a clear typed unsupported-checkpoint-version refusal (zero half-migrated dual-format shadow readers in production), while existing frozen archive protocol golden tests continue exercising versioned refusal cleanly.

### 6.2 Content-Addressed Chunked Placement Groups & Consolidation-Invariant Semantic Commitments

```text
RegionLeaf (in CHK24 manifest / paged directory)
 ├── partition_id: PartitionId          (UNCHANGED — semantic identity never splits!)
 ├── slot_directory_bounds: (entity_slots, relation_slots)
 ├── root_semantic_merkle: [u8; 32]     <-- Merkle root over ordered ChunkSemanticEntry[]
 │                                          (INVARIANT across history flattening / consolidation!)
 └── chunk_bindings_locator             <-- Checkpoint-authenticated ChunkLocatorBinding[]
       ├── Chunk 0 [0..4096)   -> (head_loc_0, hist_loc_0, cont_dir_0)
       ├── Chunk 1 [4096..8192)-> (head_loc_1, hist_loc_1, cont_dir_1)
       └── ...
```

1. **Zero semantic identity distortion:** Subdividing a placement group into fixed, non-overlapping `SlotRange` chunks preserves `PartitionId`, `RecordId`, and slot numbers identically. Small partitions emit a single chunk (`0` overhead for small DBs).
2. **Consolidation-invariant semantic commitment *(Astra r2 #2)*:**
   - *Failure prevented:* M2 checkpoint consolidation (`milestone-2.md:420`) flattens layered history images into a flat history image, changing the encoded bytes and content-addressed `history_locator` without changing the commit's logical meaning. If `history_locator` were hashed into the selected root's semantic commitment, consolidating history would invalidate pre-consolidation `RetainedBasisToken`s!
   - *Fix:* `root_semantic_merkle` commits strictly over ordered `ChunkSemanticEntry { slot_range, semantic_chunk_commitment, head_summary, history_summary }`, where `semantic_chunk_commitment` depends **only** on canonical chunk meaning (identical before and after history flattening!). `ChunkLocatorBinding { slot_range, head_locator, history_locator, continuation_directory_locator }` is authenticated separately by the `CHK24` manifest so consolidation updates locators cleanly while preserving the exact semantic root commitment.
3. **Chunk-wise capture consolidation & chunk-wise CoW edit *(Fable r1 #8)*:**
   - M2's capture consolidation demanded `Full` on the whole partition under a 1 GiB limit, and CoW edits required a full-partition residency witness — both impossible when a single placement group exceeds resident memory.
   - M4 consolidates and copy-on-write edits **per dirty `SlotRange` chunk** (and per dirty intra-slot continuation page), keeping capture consolidation and foreground CoW edits strictly bounded by the touched chunk window.

### 6.3 Bounded Per-Slot Continuation Pages & Large-Value Streaming

1. **Intra-slot continuation pages *(Astra r2 #4)*:**
   - *Failure prevented:* Fixed slot ranges cannot subdivide a single hot record whose version history grows to 100,000 versions, or a single hub vertex whose adjacency list (`region_image.rs:60`) has 100,000 incident edges — even when every individual field value is tiny!
   - *Fix:* Within a chunk, any slot's version chain or adjacency list exceeding `SLOT_INLINE_BUDGET` spills into a bounded, content-addressed **continuation page chain/tree** (`continuation_directory_locator`), admitted and evicted window-by-window under the exact same `WorkingSetAccountant` and folded into `semantic_chunk_commitment`.
2. **Bounded large-value streaming & in-flight fill cap:**
   - Large scalar/byte field payloads stream through fixed `streaming_window_bytes` chunks behind `ValueReader`, with aggregate concurrent cold fills capped by `max_in_flight_fill_bytes` — streaming never hides a whole-value `Vec<u8>` allocation or unbounded queued read work.

### 6.4 Producer-Derived Cold Kind/Adjacency Summaries & Mirror Separation

1. **Producer-derived / pre-hash verified summaries *(Astra r1 #3, Astra r2 #3, Fable r1 #8)*:**
   - *Failure prevented:* Hashing a chunk summary into the Merkle root and checking slot-range coverage still cannot catch a buggy producer that omits an edge *before* hashing!
   - *Fix:* Every `ChunkKindAdjacencySummary` is **deterministically derived from (or verified against) the canonical head/history chunk projection inside the single chunk sealing owner before computing `ChunkSemanticEntry`'s hash**. Cold readers additionally verify complete, non-overlapping `SlotRange` coverage against `(entity_slots, relation_slots)` before trusting any negative summary to skip a chunk during kind scans (reusing `truth_kind_scan/directory.rs:19`), graph traversals, or validation reads.
2. **Mirror pin & lifecycle separation (`milestone-2.md:443`):**
   - Move branch pin and lifecycle metadata out of the mirror into the branch/root metadata owner so the mirror ceases to act as a second full resident working copy of `main`.

### 6.5 Paged Bounded-Memory Reopen & `O(touched)` Checkpoint Cutover (`G7`)

#### Why Reopen & Checkpoint Currently Scale Linearly *(Astra r1 #4, Astra r2 #5, Astra r3 #2, Fable r1 #9)*
Even if every record chunk fits in memory, four eager data structures on `origin/master` `0129df8c42` still scale linearly with total DB size or commit history:
1. `PortableRecordDirectory` (`storage/residency/record_directory/portable.rs:14`) stores one entry per entity and relation eagerly in the manifest.
2. `RecordIdentitySubsystem` restore (`runtime/state/subsystems/record_identity/mod.rs:124`, `checkpoint_restore/record_identity.rs:14,49,127`) eagerly loads **all four per-record/per-partition collections**: generation high-water marks, reusable slots, pending reservations, and per-partition append frontiers.
3. Cold recovery (`checkpoint_restore/cold_directory.rs:94`) builds in-memory commit-graph and patch-position maps over every manifest commit.
4. Checkpoint cutover (`durability/checkpoint/capture/publication_cutover.rs:49` -> `selected_manifest_pins.rs:104`) runs `scan_rebuild` linearly over every route in the selected root.

#### Authoritative Bounded `G7` Cutover
1. **Paged directories in `CHK24`:** Store `PortableRecordDirectory`, all four `RecordIdentitySubsystem` tables (generation high-water marks, reusable slots, pending reservations, per-partition append frontiers), and the commit-index / patch-position directory as **immutable content-addressed B-tree/paged directory members** keyed by `(partition_id, slot_range)` / `commit_id` / `patch_position`. Reopen loads only the fixed-size root directory descriptors and faults in individual directory pages on demand through `WorkingSetAccountant` — guaranteeing bounded reopen memory under the **same** allowance as record cardinality, outstanding reservations, and commit count each grow `10×`!
2. **Incremental / carried checkpoint pin sets:** Replace `selected_blob_manifest_pins` full-root `scan_rebuild` at `publication_cutover.rs:49` with incremental delta updates carried forward from the predecessor checkpoint's verified pin set (retaining exact checkpoint protection while reducing cutover work from `O(total_routes)` to `O(touched)`).

---

## 7. Layer 4 — Historical Retained-Basis Access, Resumable Cursor Tokens & Writeback-Fenced Reclamation (`M4.4–M4.5`)

### 7.1 Resumable `RetainedBasisToken` & Resume-vs-Prune `ReaderLoan` Fence (`M4.4`, `D4`)

#### Why Existing Retained Bases Cannot Survive Restart or Index Discard *(Astra r1 #7, Fable r1 #6, #10)*
1. `worth-relational-branch/src/branch/basis_retention.rs:24` explicitly refuses `ForeignRuntime` on `runtime_instance_id`, and residency leases are strictly in-process — so retained bases cannot survive restart by design today.
2. Persisting a durable retention pin for every emitted cursor token without an expiry/release rule would leak history indefinitely whenever a client abandons a paginated UI or HTTP request.
3. Conversely, validating a cursor token and then taking a read without holding a loan under Store's reclaim fence creates a TOCTOU race where pruning deletes the basis between validation and the first chunk read!
4. Finally, resuming by physical index offset or tree slot breaks after a physical index is discarded and rebuilt or skipped — silently skipping or duplicating rows!

#### Authoritative Token-vs-Loan Contract
1. **Resumable token, NOT an unbounded durable pin:**
   - Extend `RelationalBranchBasisDescriptor` with durable `RetainedBasisToken` (`§3.4`) binding `(home_incarnation, branch_origin, commit_id, root_semantic_commitment, schema_version, query_digest, canonical_key_position)`.
   - `canonical_key_position` is encoded strictly in **canonical logical key order** (never a physical index page offset) and paired with `query_digest` so resuming after physical index discard produces exact, duplicate-free, gap-free continuation over canonical storage.
   - `root_semantic_commitment` is consolidation-invariant (`§6.2`), so a token emitted **before** checkpoint history consolidation resumes cleanly after consolidation + reopen!
2. **Atomic resume-vs-prune `ReaderLoan` acquisition:**
   - `resume_cursor(&req, token)` validates `home_incarnation`, `query_digest`, and `schema_version`, then **atomically resolves `commit_id` + `root_semantic_commitment` and acquires an in-process `ReaderLoan` under Store's reclaim fence**.
   - While the `ReaderLoan` is held (for the entire lifetime of the resumed cursor/iterator until dropped), Store reclamation cannot release any chunk, continuation page, or directory page reachable from that root.
   - Racing `resume_cursor` against pruning produces exclusively two lawful outcomes:
     - Loan acquired before prune → exact read through the cursor's last observable byte, OR
     - Prune won before loan acquisition → typed **`HistoricalReadDenial::HistoryUnavailable`** before admission (never a partial/damaged read, never silently rebinding to `HEAD`!).
3. **Exact restart semantics:**
   - In-process `ReaderLoan`s end at process death.
   - Reopening and presenting a saved `RetainedBasisToken` succeeds **iff** that basis is still independently retained (by a live branch, an archived branch root, an immutable tag, an active `PinTable`/`BranchLease`, or the configured retention horizon). Otherwise it returns `HistoricalReadDenial::HistoryUnavailable` — **never silently rebinding to the latest branch head**.
4. **Three-way typed historical disposition:**
   - `HistoryUnavailable` — basis legitimately pruned past retention horizon.
   - `UnsupportedHistoricalMeaning` — schema/version boundary cannot answer the requested historical meaning.
   - `CorruptStorage` — persisted manifest/chunk/continuation bytes failed checksum, Merkle commitment, or allocation-generation validation.
5. **Direct historical access & canonical truth over accelerators:**
   - Historical reads access the selected retained root's chunks and per-slot continuation pages directly without replaying all intervening application operations or promoting history into the head working set.
   - Physical indexes and derived materializations are strictly disposable accelerators: discarding them never loses canonical coverage or changes a historical query's answer.

---

### 7.2 End-to-End Semantic-to-Physical Safe Reclamation (`M4.5`, `D5`)

#### Complete Independent Reachability Law (`Reach`)
Physical reclamation may free segments, extents, or arena ranges **strictly outside**:

```text
Reach =
    live branch selected roots
  ∪ archived branch selected roots
  ∪ durable PinTable & BranchLease roots (including transitive nested-overlay edges & in-flight durable decisions)
  ∪ existing per-cell `retain_members` custody (`cell.rs:103-117`, `worth-store-relational/src/cold_source/member_custody/`)
  ∪ existing `RootProtectionRegistry` in-process registrations (`stability/retention/registrations/`)
  ∪ latest & staged checkpoint manifests and their carried member closures
  ∪ pending recovery obligations (unresolved WAL/branch-transition/reclamation intents)
```

- **Respecting `road1.toml:525-560,650-685`:** `worth-store/src/physical_runtime` remains 100% free of branch/MVCC vocabulary. Upper owners (`worth-runtime-world` + `worth-store-application` + `worth-store-relational`) project an **opaque `ProtectedPhysicalRootSet`** into Store's physical reclamation planner (`PhysicalPublicationAdmission` + `reclaim_guard.rs:5`), which unions it with Store's own physical custody registrations (`PhysicalRetirementDenial::{Protected, Unresolved, Retained}`).

#### Canonical Coverage Before Prune
- A journal segment, commit representation, or materialized base becomes eligible for pruning **only after** a verified canonical `CHK24` successor checkpoint (covering its live semantic meaning) is durably sealed. A physical index or derived materialization can **never** become the sole surviving truth.

#### Four-Barrier Durable Retirement Ordering & Dual Allocator Admission Paths
Preserving and hardening Store's existing extent release protocol (`retirement_release.rs:125`, `resume.rs:101`, `extent_copy/resolution.rs:157`):

```text
Path A (Retired Extent / Segment):
  [Barrier 1] RETIREMENT[_EXTENT]_INTENT   (requires range ∩ Reach == ∅ & canonical successor coverage)
       │
       ▼  (crash-interruptible -> RecoverResumeFreeSpaceRoot)
  [Barrier 2] PublishFreeSpaceRoot / Segment Unlink
       │
       ▼  (crash-interruptible -> RecoverResumeReclaimComplete; NEVER admit to allocator yet!)
  [Barrier 3] RETIREMENT[_EXTENT]_COMPLETION
       │
       ▼  (requires WritebackFenceDrain == 0 overlapping in-flight writes + gen' > max_seen_gen)
  [Barrier 4] AdmitToAllocator(range, gen')

Path B (Cancelled Candidate Extent — `extent_copy/resolution.rs:157`, ZERO intervening root publication):
  [Barrier 1] Durable CancelCandidateExtent WAL record (advances `extent_fence_seq`!)
       │
       ▼
  [Barrier 2] Checkpoint covers cancellation WAL record
       │
       ▼  (requires WritebackFenceDrain == 0 overlapping in-flight writes + gen' > max_seen_gen)
  [Barrier 3] AdmitToAllocator(range, gen')
```

#### Why This Design (Reviewer Discoveries Prevented)
- **Existing per-cell `retain_members` custody cannot be forgotten** *(Fable r1 #6)*: If a branch is deleted and its durable pin dropped while a live reader holds a `Cold` cell with `retain_members` custody (`cell.rs:103-117`), reclaiming its members causes the next demand to hit a missing member — which triggers sticky `Damaged` AND home-wide `ServingRequiresInspection`! Uniting `retain_members` and `RootProtectionRegistry` into `Reach` eliminates this permanently.
- **Reuse-before-completion double-free** *(Fable r1 #7, Astra r1 #1, Astra r2 #7)*: For extents, "delete" publishes a free-space authority root (`retired_artifact.rs`). If a range were reallocated and written *before* `RETIREMENT_EXTENT_COMPLETION`, or if recovery itself died after publishing the free-space root before recording completion, a naive idempotent resume on the next reopen could **re-release a live generation-`g+1` range**! Enforcing `AdmitToAllocator` strictly *after* durable `RETIREMENT_EXTENT_COMPLETION` (guarded by `RetirementReleaseProjection` in `resume.rs:101`) and splitting recovery into chained-crash-interruptible barriers eliminates double-free across arbitrarily nested crashes.
- **Bounded foreground edit cost (`O(touched)`):** Reclamation runs incrementally in bounded units (`ReclaimBudget`) off the foreground commit critical path so local foreground edits never turn into an unbounded scan of total retained state.

---

### 7.3 P0 Writeback Fencing & Cancel-Monotone `ExtentAllocationGeneration` (`M4.5`, `D6`)

#### Why Read-Time Generation Checks Alone Lose Data *(Astra r1 #1, Astra r2 #1, Astra r3 #1, Fable r1 #5)*
1. **P0 — Paused writeback overwrites freshly reallocated `g+1`:** Suppose dirty frame `F` for `(arena, offset, len)` was admitted for writeback (`writeback_claim.rs:209`) at generation `g`, then paused by OS scheduling. Meanwhile the extent is retired, freed, and reallocated (possibly split or coalesced!) to a new live extent at `g+1`, whose bytes land on disk. Then paused writeback `F` from `g` reaches the device and **clobbers `g+1`'s live bytes**. Checking generation on the next read only detects corruption *after* the new data is permanently lost!
2. **Candidate-extent cancel reuses a range without publishing a successor root:** `extent_copy/resolution.rs:157-161` can write a candidate extent, durably cancel it, and return its range once a checkpoint covers the cancellation WAL record — with **no** intervening root publication. Deriving `ExtentAllocationGeneration` solely from `root_publication_generation` would reissue the exact same `(range, gen)`!
3. **Zero Relational format pollution:** `RecordArtifactFile::Segment`, `RetiredArtifact`, and `DurableExtentRecordPlacement::extent_generation()` ALREADY carry durable generations; `PhysicalResidentFrameGeneration` is process-local. Only `RecordArtifactFile::ExtentArena { arena }` + offset inside `RecordFrameCoordinate` lacks a durable generation. Relational's content-addressed `RelationalDurableMemberLocator` needs **zero** physical generation fields.

#### Authoritative Writeback-Fence & Durable-Epoch Protocol
1. **Durable crash- and cancel-monotone generation:** Pair every `ExtentArena` allocation with `ExtentAllocationGeneration { root_publication_generation, extent_fence_seq }`, established on durable WAL admission **before** any candidate I/O and strictly advanced on every extent cancel (`extent_copy/resolution.rs:157`) OR extent retirement barrier before that range can be reissued (surviving cancel → reopen → reuse with zero intervening root publication!).
2. **Allocation identity on buffer-pool frames & writeback claims:** Carry `ArenaExtentFrameIdentity { range, generation }` on `worth-store-buffer-pool` arena frames AND through `writeback_claim.rs:209` I/O admission.
3. **Mandatory `WritebackFenceDrain` BEFORE `AdmitToAllocator`:**
   - Before admitting any freed or cancelled `ArenaByteRange` back to the allocator (via Path A or Path B):
     1. Fence new writeback claims overlapping `range` (using `ArenaByteRange::overlaps`, covering exact, split, and coalesced sub/super-ranges!).
     2. **Drain or cancel every in-flight overlapping writeback claim** until zero overlapping writes remain active.
     3. Call `invalidate_clean_range` to evict any stale clean frames covering overlapping ranges from `worth-store-buffer-pool`.
     4. Only then admit `(range, gen')` to the allocator with strictly greater `gen' > max_seen_gen(range)`.
4. **Fail-closed stale-generation checks:** Any late writeback submission, clean fault-in, persisted manifest reference, or recovery replay referencing `gen != active_gen(range)` refuses immediately with a typed generation mismatch / `CorruptStorage` error.

---

## 8. Target Rust Developer Experience (`// PROPOSED` — Stable-Rust Hard DX Reference)

Milestone 4 makes larger-than-memory scale, O(1) durable branching, resumable time-travel cursors, and safe reclamation **feel completely effortless** to application developers. Every signature below aligns verbatim with `platform-dx-hardening.md` (`Home<P>`, `Request`, `ProductRef`, `HistoryRefValue`, `Reportable`, `Disposition`) and `semantic-git.md` §8, and compiles verbatim on stable `rustc` in `docs/dx-reference/store_m4_scale_and_branches.rs`.

### Hard DX Ergonomics Rules (Non-Negotiable)
1. **Zero storage-engine plumbing in application code:** Developers configure a declarative `ResidencyBudget` once on `HomeBuilder`; chunking (`RegionChunkId`), intra-slot continuation pages, `WorkingSetAccountant` eviction, cold kind/adjacency summary pruning, and `ExtentAllocationGeneration` ABA guards operate **100% transparently**.
2. **Compile-time lifetime safety on `ReaderLoan`:** Borrowed views from a `HistoricalCursor` or `ValueReader` tie their lifetime directly to the `ReaderLoan` guard (`E0597` if a view tries to outlive its loan); leaving the scope releases the loan automatically via RAII `Drop`.
3. **One clean `Disposition` story for exhaustion, pruning & corruption:** Every M4 refusal implements `worth-execution::report::Reportable` (`MemoryExhausted` → `Disposition::Retry(RetryHint::MemoryPressure)`, `HistoryUnavailable` → `Disposition::Final`, `CorruptStorage` → `Disposition::Escalate`) and integrates with `request.explain(...)`.
4. **Pre-`W2.1` honesty (matching `semantic-git.md` §12):** If M4 lands before Mac Studio's `W2.1` (`Home<P>`/`Request`) facade, M4 exposes the exact same semantic contracts through a thin, explicitly isolated Host adapter over `WorthQueryApplicationHome` / `ProductBranchReferenceCell` that swaps 1-to-1 onto `Home<P>`/`Request` with zero semantic change.

```rust
// PROPOSED — `docs/dx-reference/store_m4_scale_and_branches.rs`
// Compiles verbatim on stable Rust (zero manual chunk/eviction/generation plumbing;
// compile-time loan safety; exhaustive typed Disposition handling).

pub fn sustained_scale_and_branching_journey(
    home_dir: &std::path::Path,
    principal: &Principal,
    scope: &AccessScope,
) -> Result<(), AppError> {
    // === 1. Declarative Larger-Than-Memory Budget (Zero Manual Cache/Chunk Plumbing) ===
    let home: Home<MyApp> = Home::directory(home_dir)
        .residency_budget(ResidencyBudget {
            head_working_set_bytes: 32 * 1024 * 1024,       // 32 MiB active head working set
            historical_working_set_bytes: 8 * 1024 * 1024,  // 8 MiB isolated historical budget
            max_in_flight_fill_bytes: 4 * 1024 * 1024,      // 4 MiB max concurrent cold I/O
            streaming_window_bytes: 256 * 1024,             // 256 KiB large-value stream window
        })
        .open()?;

    let req: Request<'_, MySchema> = home.request(principal, scope)?;
    let main: ProductRef<'_, MySchema> = home.product(&ProductName::main())?;

    // === 2. Transparent Larger-Than-Memory Queries, Traversals & Large-Value Streaming ===
    // Eviction, chunked placement groups, intra-slot continuation pages, and cold summaries
    // happen completely invisibly behind ordinary typed reads.
    let mut scan = main.scan::<LargeMesh>(&req, ScanFilter::by_kind())?;
    while let Some(row) = scan.next_row(&req)? {
        // Large field payload streams through bounded 256 KiB windows — never allocates whole Vec<u8>!
        let mut reader: ValueReader<'_> = row.stream_field(&req, LargeMesh::COL.geometry_bytes)?;
        let mut buf = [0u8; 64 * 1024];
        while reader.read_chunk(&mut buf)? > 0 { /* process bounded slice */ }
    }
    drop(scan); // releases resident chunk borrows immediately

    // === 3. O(1) Copy-on-Write Durable Branching, Archive, Delete & Concurrent Preparation ===
    let base_head: ProductHead = main.observe(&req)?;
    let branch_a = main.create_branch(&req, BranchName::new("design/option-a")?, base_head.revision)?;
    let branch_b = main.create_branch(&req, BranchName::new("design/option-b")?, base_head.revision)?;

    // Independent branches prepare concurrently without a whole-Store preparation lock;
    // FIFO publication revalidates and composes onto the current Home frontier cleanly.
    std::thread::scope(|s| -> Result<(), AppError> {
        let h_a = s.spawn(|| branch_a.mutate(&req, edit_option_a(), key_a));
        let h_b = s.spawn(|| branch_b.mutate(&req, edit_option_b(), key_b));
        h_a.join().expect("thread a")?;
        h_b.join().expect("thread b")?;
        Ok(())
    })?;

    // Archive preserves historical root while refusing new writes;
    // delete permanently retires the branch name & drops its root pin without harming siblings.
    main.archive_branch(&req, &BranchName::new("design/option-a")?)?;
    main.delete_branch(&req, &BranchName::new("design/option-b")?)?;

    // === 4. Resumable Historical Time-Travel Cursor Across Consolidation & Process Restart ===
    // Open a historical cursor bound to `base_head` (canonical key order + query digest):
    let mut hist: HistoricalCursor<'_, MySchema, AuditEntry> =
        main.at_revision(&req, base_head.revision)?.scan_history(&req, AuditQuery::ordered())?;
    let _first_page = hist.next_page(&req, 50)?;
    let saved_token: RetainedBasisToken = hist.checkpoint_token(); // opaque `whc1_...`, NOT a durable leak!
    drop(hist); // releases in-process ReaderLoan cleanly

    // Advance HEAD, consolidate checkpoint history, close and reopen in a fresh process:
    main.mutate(&req, advance_main_many_times(), key_advance)?;
    home.checkpoint(&req)?;
    home.close(&req)?;

    let reopened: Home<MyApp> = Home::directory(home_dir)
        .residency_budget(ResidencyBudget {
            head_working_set_bytes: 32 * 1024 * 1024,
            historical_working_set_bytes: 8 * 1024 * 1024,
            max_in_flight_fill_bytes: 4 * 1024 * 1024,
            streaming_window_bytes: 256 * 1024,
        })
        .open()?;
    let req2 = reopened.request(principal, scope)?;
    let main2 = reopened.product(&ProductName::main())?;

    // Resume acquires a fresh ReaderLoan atomically under Store's reclaim fence:
    // NEVER silently rebinds to HEAD! Survives history consolidation seamlessly!
    match main2.resume_cursor::<AuditEntry>(&req2, &saved_token) {
        Ok(mut resumed) => {
            let _second_page = resumed.next_page(&req2, 50)?;
            // `resumed` holds its `ReaderLoan` until dropped here.
        }
        Err(HistoricalReadDenial::HistoryUnavailable { commit_id }) => {
            // Legitimately pruned past retention horizon — exact typed disposition (`Final`)!
            let _ = commit_id;
        }
        Err(HistoricalReadDenial::UnsupportedHistoricalMeaning { .. }) => {
            // Schema boundary incompatibility — exact typed disposition!
        }
        Err(HistoricalReadDenial::MemoryExhausted { .. }) => {
            // Genuine saturation (`Disposition::Retry(RetryHint::MemoryPressure)`) — never panics!
        }
        Err(denial @ HistoricalReadDenial::CorruptStorage { .. }) => {
            return Err(denial.into()); // `Disposition::Escalate`
        }
        Err(HistoricalReadDenial::RequestStopped) => return Ok(()),
    }

    // === 5. Structured Operational Observability & Bounded Space Reclamation ===
    let _pressure: ResidencyPressureSnapshot = reopened.residency_report(&req2)?;
    let _reclaim: ReclamationReport =
        reopened.reclaim_step(&req2, ReclaimBudget::bounded_bytes(16 * 1024 * 1024))?;
    reopened.close(&req2)?;
    Ok(())
}

// PROPOSED — Owner-to-Owner Seam (`worth-runtime-world` / Semantic Git Phase 2 consumer of M4.1)
// Demonstrates O(1) whole-Home pinning (including `product_map_root`), nested grandchild survival
// across parent `Active -> Expiring -> Expired`, and zero-ceremony `Reportable`/`Disposition` mapping.
pub fn semantic_git_nested_fork_pin_and_retry_seam(
    retention: &dyn RuntimeWorldRetentionPort,
    parent_scope: PinScope,
    child_scope: PinScope,
    req: &Request<'_, MySchema>,
) -> Result<Disposition, AppError> {
    // 1. O(1) atomic manifest capture under Store's reclamation fence (zero base scans!):
    let parent: PinnedFrontierLease = retention.capture_and_pin_frontier(PinSpec {
        scope: parent_scope,
        parent_pin: None,
        nested_drop_policy: NestedDropPolicy::RetainUntilChildrenDrain,
    })?;
    assert!(parent.manifest().product_map_root.is_some());

    // 2. Nested overlay child pin links directly to `parent.pin_id()`; survives parent expiry
    //    until the child's own loans and grandchildren drain:
    let child: PinnedFrontierLease = retention.capture_and_pin_frontier(PinSpec {
        scope: child_scope,
        parent_pin: Some(parent.pin_id()),
        nested_drop_policy: NestedDropPolicy::RetainUntilChildrenDrain,
    })?;

    // 3. Expiring parent transitions `Active -> Expiring` (fencing new loans on parent),
    //    while keeping every ancestor root reachable for `child`:
    let transition = retention.expire_lease(parent.pin_id())?;
    debug_assert_eq!(transition.state(), LeaseState::Expiring);

    // 4. Child reads remain exact across displaced-segment retirement & reopen;
    //    releasing `child` allows `parent` to complete `Expiring -> Expired`:
    let read_outcome = child.read_pinned_manifest();
    drop(child);

    // 5. Uniform `worth-execution::report::Reportable` disposition mapping for servers/CLIs:
    match read_outcome {
        Ok(_manifest) => Ok(Disposition::Complete),
        Err(ref err) => {
            let report = req.explain(err);
            // `MemoryExhausted` -> `Disposition::Retry(RetryHint::MemoryPressure)` (HTTP 503)
            // `HistoryUnavailable` -> `Disposition::Final` (HTTP 410)
            // `CorruptStorage` -> `Disposition::Escalate` (alert + halt)
            Ok(report.disposition())
        }
    }
}
```

### §8.1 Operational Documentation & Continuing Audience Ownership

Per `spec-designer`, public documentation is updated **in the phase that makes each behavior true** (never in a detached post-hoc journal):

| Continuing Document | Owning Phase(s) | Exact Consumer Behavior Taught |
|---|---|---|
| `docs/application-home.md` | M4.2, M4.4, M4.7 | Declarative `ResidencyBudget` tuning (`head`, `historical`, `max_in_flight_fill_bytes`, `streaming_window_bytes`), `MemoryExhausted` retry posture, and `RetainedBasisToken` restart semantics |
| `docs/how-it-works.md` | M4.1, M4.3, M4.5, M4.7 | `HomeRootManifest`/`PinTable` O(1) whole-Home pins, `CHK24`/`REG8` chunked placement groups + per-slot continuations, and two-phase `WritebackFenceDrain` + `ExtentAllocationGeneration` safe reclamation |
| `docs/capability-index.md` | M4.1, M4.4, M4.7 | Durable Relational branch lifecycle (`fork`, `archive`, `delete`, permanent retirement), `HistoricalCursor` resumability, and `ValueReader` bounded streaming |
| `plans/database-foundation/roadmap.md` | M4.7 | Milestone 4 completion status and Database Foundation Roadmap closeout |
| `worth-orchestration/milestones/semantic-git.md` | M4.1 | 3-line Phase 2 alignment note pointing Semantic Git directly at M4.1's `HomeRootManifest`, `PinTable`, `BranchLease`, Verus kernel, Lean theorems, and TLA+ pin actions |

---

## 9. Pragmatic 4-Layer Formal Verification Contract

Every verification layer is assigned **strictly to the class of defects it can actually observe** (zero ceremonial proofs on `Mutex`/`Arc` glue) and stays completely outside ordinary `./loop edit` builds:

1. **Lean 4 Unbounded Proofs (`proofs/lean/store_m4/`, `python tools/lean/run_lean.py build --manifest`):** Queries Lean's kernel directly (`isDefEq` against theorem manifest JSON + `Lean.collectAxioms`, zero `sorryAx`, standard axioms only) to prove **6 high-leverage unbounded theorems** where finite TLC cannot generalize.
2. **Verus Verified Production Rust Kernels (`python tools/verus/run_verus.py check`):** Proves pre/postconditions directly on **3 small, pure, executable Rust transition kernels** (plain-`cargo` compilable without `vstd`, zero `admit`/`assume`/`unsafe`, audited `verus:tcb_reviewed(...)` markers only where required, plus an in-tree production-body mutant kill check).
3. **Composed Split-Barrier TLA+ / TLC (`specs/tla/store_m4/StoreM4ScaleReclaim.tla`, `python tools/tla/run_tlc.py check`):** Refines `specs/tla/store_m3/StoreDurability.tla` via `-DTLA-Library` with **separate atomic actions per durable barrier** (including chained-crash interruptible recovery!) and shared pin/expiry/retirement actions consumed directly by `SemanticGit.tla`, plus **5 mandatory non-vacuity reachability witnesses**.
4. **Extended Single-Harness Seeded DST + TLC Trace Validation (`store_m4_dst` inside `c6_durability_dst`, `python tools/tla/run_tlc.py trace`):** Exercises real `Arc`/reservation eviction, `try_lock` concurrency, chunked placement groups, per-slot continuations, paged reopen, paused writebacks, candidate cancellation, chained-crash interrupted reclamation, and `10×` multi-axis scaling headroom on Memory + Directory — emitting **observed production barrier transitions, generations, and fates** (not just Rust oracle post-state!) into NDJSON traces validated by TLC, and killing all **10 named mutants (`M1–M10`)** individually.

### 9.1 Authoritative 4-Layer Traceability Matrix

| # | Invariant / Law | Lean 4 Unbounded Theorem (`proofs/lean/store_m4/`) | Verus Pure Production Kernel (Real Crate) | TLA+ (`StoreM4ScaleReclaim.tla`) | Rust DST / Courts |
|---|---|---|---|---|---|
| **1** | `StoreGCPinSafety` & `LeaseExpiryNoReaderLoss` *(shared with Semantic Git)* | Independent reachability `Reach(live roots ∪ durable pins ∪ transitive nested overlays ∪ active loans ∪ in-flight decisions) ⊆ allocated`; `Active → Expiring → Expired` retains pins until `active_reader_loans == 0 && live_child_pins == 0` | `PinTable::{acquire, release, expire_lease}` (`worth-store`) | `PublishForkPin`, `ExpireForkLease` (Witness 1) | S4, S6, M5 |
| **2** | `AddressReuseGenerationAndWriteFenceSound` *(P0)* | Across arbitrary split/coalesced `ArenaByteRange`s, candidate cancels, and crashes, `WritebackFenceDrain` + monotone `(root_pub_gen, extent_fence_seq)` prevents any generation-`g` writeback or stale lookup from aliasing or clobbering generation `g' > g` | `AllocationGenerationGuard` (`worth-store-buffer-pool` / `worth-store-physical-format`) | `WritebackIssue`, `WritebackLand`, `CancelCandidateExtent`, `AdmitToAllocator` (Witness 2) | S8, M10 |
| **3** | `ReclaimPreservesAllReachableRoots` & `CanonicalCoverageBeforePrune` | `reclaim_set ∩ Reach == ∅` (including `retain_members` cold cells & `RootProtectionRegistry`) AND every pruned journal/base has verified canonical successor coverage | `ReclamationPlanValidator` (`worth-store`, over opaque protected set per `road1.toml`) | `ReclaimIntent`, `PublishFreeSpaceRoot` | S6, M5, M8 |
| **4** | `InterruptedReclaimIdempotent` | Crashing between any of `ReclaimIntent`, `PublishFreeSpaceRoot`, `ReclaimComplete`, and `AdmitToAllocator` — OR during recovery's own `RecoverResumeFreeSpaceRoot` before `RecoverResumeReclaimComplete` — never loses reachable data, leaks permanently, or double-frees a reused `g' > g` range | `ReclamationPlanValidator` (`worth-store`) | Split reclaim + chained recovery barriers (Witness 3) | S7, M9 |
| **5** | `ChunkedCoverageAndSummarySound` | Fixed non-overlapping `SlotRange` Merkle partition + per-slot continuation pages + producer-derived summaries guarantee cold chunk/summary pruning agrees identically with canonical full scan and stays invariant across history flattening | *(Optional pure `ChunkCoverageValidator` in M4.6)* | — | S2, S5, M3 |
| **6** | `RetainedBasisCursorImmutable` | `RetainedBasisToken` (canonical key position + query digest + consolidation-invariant semantic commitment) + reclaim-fenced `ReaderLoan` never silently rebinds to `HEAD` and never skips/duplicates rows across consolidation or index discard | — | `Crash` + `RecoverServing` cursor check (Witness 5) | S5, M7 |
| **7** | `RealAllocationEvictionAndDeadlockFreedom` | *(Not a Lean/Verus glue target — observed directly in real Rust runtime!)* | *(Rust concurrency & reservation courts)* | — | S1, S3, M1, M2 |
| **8** | `ConcurrentPrepHomeFrontierLinearizable` & `MultiAxisOTouchedScaling` | *(Observed directly via TLA+ + Rust concurrency & `10×` headroom courts!)* | — | Concurrent prep + archive race (Witness 4) | S4, S9, S10, M4, M6 |

#### Verus Pure Production Kernel Contracts (`requires` / `ensures` Sketches)

```rust
// PROPOSED — Verus pure transition kernels (plain-cargo compilable via cfg(verus_keep_ghost), zero Mutex/Arc!)

// 1. `PinTable` (worth-store, shared verbatim with Semantic Git Phase 2)
// ensures:
//   - acquire(pin, manifest, parent) inserts pin with `state == Active`, preserves all existing pins,
//     and increases parent's `live_child_pins` iff `parent.is_some()`.
//   - expire_lease(pin) transitions `Active -> Expiring` (fencing new loans) and transitions
//     `Expiring -> Expired` ONLY when `active_reader_loans == 0 && live_child_pins == 0`.
//   - reachable_roots(table) contains every Active/Expiring pin root AND every transitive ancestor overlay!

// 2. `AllocationGenerationGuard` (worth-store-buffer-pool / worth-store-physical-format)
// ensures:
//   - can_admit_for_reuse(range, in_flight_writes, barrier_state, next_gen) == true ONLY IF:
//       (barrier_state == CompletedRetirement || barrier_state == CheckpointCoveredCancel)
//       && forall w in in_flight_writes: !range.overlaps(w.range)
//       && next_gen > max_seen_gen_overlapping(range)
//   - frame_lookup_matches(frame, req_range, req_gen) == (frame.range == req_range && frame.gen == req_gen)

// 3. `ReclamationPlanValidator` (worth-store, opaque protected set obeying road1.toml!)
// ensures:
//   - validate_reclaim_intent(candidate, protected_reach, canonical_covered) == Ok ONLY IF:
//       !protected_reach.contains(candidate) && canonical_covered(candidate)
//   - resume_transition(persisted_record, already_completed_gen) is idempotent and NEVER emits
//     a second release for any range whose durable `RetirementReleaseProjection` was already settled.
```

### 9.2 Composed Split-Barrier TLA+ Shared Action Mapping (`StoreM4ScaleReclaim.tla` × `StoreDurability.tla` → `SemanticGit.tla`)

Every durable barrier is a **separate atomic TLA+ action** so `Crash` can interleave between any two barriers (including inside recovery itself, matching `StoreDurability.tla:291`):

| Atomic Action | Coupled Modules | Exact Barrier / Coupling Condition |
|---|---|---|
| **`PublishBranchCommit`** | `StoreDurability` × `StoreM4ScaleReclaim` | Advances branch head & durable `root_publication_generation`, preserves sibling roots, records canonical coverage, and composes concurrent branch deltas onto the current Home frontier at FIFO cutover. |
| **`PublishForkPin` / `ExpireForkLease`** | `StoreM4ScaleReclaim` → `SemanticGit` | Captures `HomeRootManifest` (with `product_map_root`) atomically under reclamation fence (including transitive nested-overlay edges); `ExpireForkLease` moves `Active → Expiring` (fencing new loans) and `Expiring → Expired` strictly when `activeReaderLoans == 0 && liveChildPins == 0`. |
| **`RetireBranchRef`** | `StoreDurability` × `StoreM4ScaleReclaim` → `SemanticGit` | Permanently retires branch name and drops only the deleted branch's selected root pin while retaining sibling & live-loan reachability. |
| **`WritebackIssue` / `WritebackLand` / `CancelCandidateExtent`** | `StoreM4ScaleReclaim` | Models in-flight writebacks carrying `(arena, range, gen)` and durable candidate-extent cancellation without root publication (`extent_copy/resolution.rs:157`); `WritebackFenceDrain` blocks allocator admission while any overlapping writeback is active, and cancel/reclaim barriers strictly advance `(root_publication_generation, extent_fence_seq)`. |
| **`ReclaimIntent`** → **`PublishFreeSpaceRoot`** → **`ReclaimComplete`** → **`AdmitToAllocator`** | `StoreDurability` × `StoreM4ScaleReclaim` | **Four distinct crash-interruptible barriers!** `ReclaimIntent` requires `range ∩ Reach == ∅` and canonical successor coverage; `AdmitToAllocator` admits strictly via **(a) completed retirement (`ReclaimComplete`) OR (b) checkpoint-covered candidate cancellation (`CancelCandidateExtent` covered by checkpoint WAL, `extent_copy/resolution.rs:157`)**, and in **both** paths requires zero overlapping in-flight `WritebackIssue`s (`WritebackFenceDrain`) AND strictly advances `gen' > max_seen_gen(range)`. |
| **`Crash`** → **`RecoverRestorePins`** → **`RecoverResumeFreeSpaceRoot`** → **`RecoverResumeReclaimComplete`** → **`RecoverServing`** | `StoreDurability` × `StoreM4ScaleReclaim` → `SemanticGit` | **Chained-crash interruptible recovery** matching `StoreDurability.tla:291`! `Crash` drops all volatile memory (`Arc`s, buffers, in-process loans, unlanded writebacks) and may fire again **between** `RecoverRestorePins`, `RecoverResumeFreeSpaceRoot`, `RecoverResumeReclaimComplete`, and `RecoverServing` — proving that dying after recovery itself publishes the free-space root (before recording completion) resumes idempotently without double-freeing reused ranges, restores `PinTable` before maintenance, and validates `RetainedBasisToken`s without rebinding to `HEAD`. |

**Mandatory Non-Vacuity Reachability Witnesses (in `StoreM4ScaleReclaim.cfg`):**
1. **Grandchild survival + unrelated space return:** Delete a branch while a sibling branch and an active nested grandchild reader loan share its base chunks → sibling & grandchild succeed; unrelated garbage reclaims while pin is live; shared chunks reclaim once last loan drains.
2. **Paused writeback fence + candidate cancel without root pub (`P0`):** `WritebackIssue(range, g)` pauses while `range` completes retirement → observe `AdmitToAllocator` **blocked** by `WritebackFenceDrain` → release/cancel paused claim → observe `AdmitToAllocator` **succeed** at `g' > g` (for both completed retirement and checkpoint-covered candidate cancellation across split/coalesced overlap) → verify fresh submission with stale `g` is refused.
3. **Chained crash during recovery reclaim resume (`P1`):** Crash after `ReclaimIntent`, start recovery (`RecoverRestorePins` → `RecoverResumeFreeSpaceRoot`), **crash a second time before `RecoverResumeReclaimComplete`**, reopen, finish completion + reallocate & write at `g' > g`, and crash + reopen once more proving zero double-release of `g'`.
4. **Concurrent branch preparation + archive race:** Concurrent `Branch A` and `Branch B` prepare from the same Home frontier, `A` publishes, `B` revalidates and applies onto `A`'s frontier (plus `B` racing `archive`).
5. **Retained cursor across consolidation & HEAD advance:** Restart with a `RetainedBasisToken` after `HEAD` advances past it and after history consolidation changes physical locators → reads exact retained basis (and returns `HistoryUnavailable` once pruned, never `HEAD`).

### 9.3 Authoritative 10-Scenario `store_m4_dst` Suite (`S1–S10`)

All 10 scenarios run on **both Memory and Directory** backends (cold OS/fault kills on Directory; equivalent logical faults within live Home lifetime on Memory) inside C6's existing compiled `cumulative_oracle` binary (`store_m4_dst`), emitting **observed production barrier transitions, generations, and fates** (never just Rust oracle post-state!) into NDJSON traces verified by `tools/tla/run_tlc.py trace`:

1. **S1 — Real-allocation eviction, `OnceLock`/`raw_cache` cleanup & basis-safe rewarm:** Scan a DB larger than resident allowance, keep the original root open, run a second scan + concurrent edits → resident bytes stay bounded (verifying real `Arc` drop + cleared `raw_head`/`raw_full`), zero AB/BA deadlock across concurrent threads, and cold↔warm reads match memory reference.
2. **S2 — Single placement group & single-slot fan-out larger than memory + paged reopen:** Single partition larger than resident allowance **plus** (a) a single record whose version history exceeds resident allowance and (b) a single vertex whose adjacency list exceeds resident allowance with small individual values (`CHK24`/`REG8` chunks + per-slot continuation pages, consolidation-invariant semantic commitments, producer-verified cold kind/adjacency summaries, bounded large-value streaming, `MaxInFlightFillBytes`), plus high record cardinality, outstanding reservations, & long commit history reopening under the **same** memory allowance (`G7`, verifying exact post-reopen slot reuse and reservation continuity).
3. **S3 — Genuine exhaustion & cancellation:** Active reader borrows saturate resident allowance → new demand returns typed `MemoryExhausted` without panicking, returning empty results, or skipping validation; releasing borrows restores service.
4. **S4 — O(1) Home-fork pins, Relational branch lifecycle & archive/delete crash sweep:** Fork whole-Home frontier (`HomeRootManifest` including persistent product-map root) + Relational branches (including durable historical-root fork reopening), divergent edits with equal-looking values, `archive` + `delete` with crash cuts → zero resurrection, retired name refused, pinned root survives existing displaced-segment retirement + crash/reopen while unrelated garbage still reclaims, nested grandchild survives parent `Active → Expiring → Expired`.
5. **S5 — Historical retained-basis cursors across consolidation, restart, prune race & index discard:** `RetainedBasisToken` (with query digest + canonical key position) issued *before* history consolidation resumes cleanly after consolidation + reopen, races resume vs. prune under reclaim fence (`exact read` OR `HistoryUnavailable` before admission), never silently rebinds to `HEAD`, reconstructs cleanly after physical index discard, and cleanly separates `HistoryUnavailable` / `UnsupportedHistoricalMeaning` / `CorruptStorage`.
6. **S6 — End-to-end safe reclamation & measurable space return:** Repeated edit → checkpoint → branch-delete cycles with active `retain_members` cold cells, `RootProtectionRegistry` guards, sibling branches, and reader loans → zero live/sibling/cold-cell loss, and eligible unreachable segments/extents return measurably after holders release.
7. **S7 — Chained-crash interrupted reclamation & reuse-before-completion sweep:** Media faults & process kills at `ReclaimIntent`, `PublishFreeSpaceRoot`, `ReclaimComplete`, **and during recovery's own `RecoverResumeFreeSpaceRoot` before `RecoverResumeReclaimComplete`**, followed by reallocation, write, and another reopen proving zero double-free.
8. **S8 — Paused writeback drain, split/coalesce overlap & candidate-cancel reuse ABA stress:** (a) Pause a generation-`g` writeback claim (`writeback_claim.rs:209`), observe overlapping split/coalesced reuse **blocked** by `WritebackFenceDrain`, release/cancel the paused claim, observe reuse **succeed** at `g' > g`, and verify fresh submission with stale `g` is refused; (b) write candidate extent → durably cancel (`extent_copy/resolution.rs:157`) → checkpoint covers cancellation → reopen → reuse **without any intervening root publication**, verifying strictly greater durable generation.
9. **S9 — Concurrent branch preparation, Home-frontier composition & archive race:** Pause Branch A and Branch B preparations from the same Home frontier before FIFO publication → A commits, B revalidates and applies cleanly onto the current Home frontier (both edits survive reopen); separately race preparation against `archive` and verify typed refusal/retry.
10. **S10 — `10×` multi-axis scaling headroom & Verus production-body mutant kill:** Independently scale (a) DB size `10×`, (b) record cardinality `10×`, and (c) retained commit history `10×` while holding a local edit's semantic scope fixed → verify O(1) Home fork (zero scans, constant writes), `O(touched)` local edit, bounded checkpoint cutover (`G7`), and bounded reopen under the same memory allowance; verify mutating an in-tree Verus production kernel body fails `tools/verus/run_verus.py`.

### 9.4 Mandatory Named Negative-Mutant Table (`M1–M10`)

Every mutant below MUST fail its named verification layer(s) individually (each paired with a positive progress witness):

| ID | Mandatory Mutant | Owning Phase | Named Killer Layer(s) |
|---|---|---|---|
| **M1** | Retain `Arc<ResidentRegion>` in root `OnceLock`, skip clearing `raw_head`/`raw_full` on eviction, or skip `try_lock` ordering | M4.2, M4.6 | Rust eviction & concurrency courts (`S1`, `S3`) |
| **M2** | Skip `BasisIdentity` validation on rewarm after branch update | M4.2, M4.6 | Rust cold↔warm differential court (`S1`) |
| **M3** | Omit a known adjacency/kind edge from a chunk summary **before hashing** (while covering all slots), skip slot-directory coverage check, or allocate a whole-value/whole-slot buffer past ceiling | M4.3, M4.6 | Lean `ChunkedCoverageAndSummarySound` + Rust cold-vs-canonical & chunk courts (`S2`, `S3`) |
| **M4** | Leave linear route scan on Home fork or checkpoint cutover (`G7`), omit `product_map_root` from `HomeRootManifest`, or eagerly restore all `RecordIdentitySubsystem` tables on reopen | M4.1, M4.3, M4.6 | Rust `10×` scaling & multi-product fork courts (`S2`, `S4`, `S10`) |
| **M5** | Release a pinned root or parent-overlay delta while a sibling branch, nested grandchild loan, or live `retain_members` cold cell still holds it | M4.1, M4.5, M4.6 | Lean `StoreGCPinSafety` + Verus `PinTable` + TLA+ + DST (`S4`, `S6`) |
| **M6** | Allow crash recovery to resurrect a deleted branch, reuse a retired branch name, or let concurrent Branch B overwrite Branch A's Home frontier | M4.1, M4.6 | TLA+ + Rust crash & concurrent-prep courts (`S4`, `S9`) |
| **M7** | Rebind a `RetainedBasisToken` to `HEAD` on reopen, break pre-consolidation tokens when history flattens, or omit canonical-key/query-digest binding across index discard | M4.4, M4.6 | Lean `RetainedBasisCursorImmutable` + TLA+ + DST (`S5`) |
| **M8** | Prune a journal/commit representation before canonical successor coverage exists | M4.5, M4.6 | Lean `CanonicalCoverageBeforePrune` + Verus `ReclamationPlanValidator` + TLA+ + DST (`S6`) |
| **M9** | Admit a freed extent for reuse before `ReclaimComplete` (or replay an interrupted recovery-phase release onto a live reallocated generation `g+1` range) | M4.5, M4.6 | Lean `InterruptedReclaimIdempotent` + Verus `ReclamationPlanValidator` + TLA+ (Witness 3) + DST (`S7`) |
| **M10** | Remove `WritebackFenceDrain` (allowing premature admission while a generation-`g` writeback is paused), reuse a cancelled candidate extent at the same generation without advancing epoch, or omit `ExtentArena` generation check on lookup/recovery | M4.5, M4.6 | Lean `AddressReuseGenerationAndWriteFenceSound` + Verus `AllocationGenerationGuard` + TLA+ (Witness 2) + DST (`S8`) |

---

## 10. Real-Consumer Acceptance Matrix (How M4 Serves Real WORTH Applications)

Milestone 4 is certified not only on synthetic storage benches, but against the real sustained-operation patterns of WORTH's public and flagship consumers:

| # | Consumer | Decisive Milestone 4 Acceptance Assertions |
|---|---|---|
| **1** | **Daymark** *(local desktop app)* | Sustained editing across thousands of task commits under a tight desktop `ResidencyBudget` (e.g., 16–32 MiB): reopen memory stays bounded (`G7`), local single-task edits stay `O(touched)` as history grows `10×`, historical audit cursors survive restarts without rebinding to `HEAD`, and deleted/retired history measurably returns disk space. |
| **2** | **Bank World** *(regulated multi-principal server)* | High-cardinality account/ledger tables exceeding resident memory; concurrent independent branch preparations (`S9`) without whole-Store preparation lock; resumable historical audit cursors (`RetainedBasisToken`) surviving checkpoint consolidation and index discard; genuine memory saturation surfaces as clean typed `Disposition::Retry(RetryHint::MemoryPressure)` (HTTP `503`, never panic or skipped balance invariant); measurable space return after branch retirement. |
| **3** | **WORTH CAD** *(high-fanout B-rep geometry kernel)* | Single large placement group + high-degree B-rep vertex/face adjacency continuation pages (`S2`) exceeding resident allowance; editing one local feature warms only touched chunks via producer-verified cold kind/adjacency summaries (zero whole-model warming); large tessellation/STEP blobs stream through bounded `ValueReader` windows without whole-value heap spikes. |
| **4** | **Semantic Git Consumers** *(Math Forge, BioGraph, Legal Desk, Stockroom, Purchase Desk)* | O(1) `HomeRootManifest` (with `product_map_root`) pin acquisition (`M4.1`, zero base scans); nested grandchild `BranchLease` survival across parent `Active → Expiring → Expired` and Store's live displaced-segment retirement + crash/reopen while unrelated garbage still reclaims; bounded large-value streaming for scientific datasets, proofs, and legal document attachments. |

---

## 11. External-Crate `trybuild` Compile-Fail Fences & Hard DX Bar

### Required External-Crate `trybuild` `.stderr` Fences (`compile-fail-fences.toml`)
Verified on **stable `rustc`** against the **release** feature graph (never doctest `compile_fail,E0xxx`!):

1. **Head/History type fence:** Calling historical/versioned read accessors on a `Head`-only view or bypassing `RelationalResidencyLease` outside its owning crate → `E0599` / `E0451`.
2. **`ReaderLoan` lifetime & opaque token fence:** Returning a row/chunk view borrowed from a `HistoricalCursor` or `ValueReader` past `drop(cursor)` (`E0597`), or hand-constructing/mutating private fields of `RetainedBasisToken`, `PinId`, `HomeRootManifest`, or `ExtentAllocationGeneration` from an external crate → `E0451` / `E0599`.
3. **Physical boundary & MVCC-vocabulary fence:** Accessing `RootProtectionRegistry` or `RetirementRemovalPermit` outside `road1.toml:87-107`, or referencing MVCC/branch vocabulary inside `worth-store/src/physical_runtime` (`road1.toml:525-560,650-685`) → `boundary-check` + `E0451`.
4. **Bounded streaming & summary authority fence:** Materializing an unbounded whole-value buffer from `ValueReader` without an explicit byte cap, or supplying an unauthenticated `ChunkKindAdjacencySummary` directly to chunk skip logic → `E0599` / `E0308`.

### Programme-Wide Hard DX & Quality Bar
- **Compiling reference spike:** `docs/dx-reference/store_m4_scale_and_branches.rs` (§8) compiles verbatim on stable `rustc`.
- **Boundary & release cleanliness:** `scripts/check-constitution.ps1` clean across all 4 workspace manifests; zero unruled `control-callers.txt`, `src-court-baseline.json`, or `*_for_test` release leaks; all modified/new `.rs` files `<= 400` lines; `check_lockfile_alignment.py` green.
- **4-layer formal gate green (`M4.6`):** 6-theorem Lean 4 manifest (`run_lean.py build --manifest`), 3-kernel Verus suite + production-body mutant kill (`run_verus.py`), composed split-barrier TLA+ (`StoreM4ScaleReclaim.tla` × `StoreDurability.tla` + all 5 non-vacuity witnesses, `0` errors), all 10 `store_m4_dst` scenarios (`S1–S10`) on Memory + Directory with observed production NDJSON trace validation (`run_tlc.py trace`), and all 10 named mutants (`M1–M10`) killed individually.

---

## 12. Fast-Feedback Phased Implementation Plan (`M4.0–M4.7`)

Ordered deliberately like Milestone 3 and `semantic-git.md` §12: **unblock Semantic Git Phase 2 and get real larger-than-memory workloads running immediately**, prove stable pure storage kernels as they are built, perform only **one** checkpoint/region format cutover (`CHK24`/`REG8`), and run the full composed formal hardening + `10×` multi-axis scaling + cross-workspace certification gate at closeout.

### Phase Overview

| Phase | Usable result | Requires | Establishes for later phases | Parallel constraints |
|---|---|---|---|---|
| **M4.0** | Formal runners (`lean`, `verus`, `tla`), portable `verus`, compiling DX spike (`store_m4_scale_and_branches.rs`), plain-`cargo` Verus mutant-kill seam | Store M3 (`C1–C6`) on `master` | Runnable 4-layer proof harness and locked public DX + `trybuild` fences | Sequential prerequisite for all M4 phases |
| **M4.1** | O(1) whole-Home `HomeRootManifest` / `PinTable` / `BranchLease` (`RuntimeWorldRetentionPort`) wired into `reclaim_guard` + durable Relational branch lifecycle & concurrent Home-frontier composition | M4.0 | Authoritative pin/lease reachability seam (unblocks Semantic Git Phase 2 immediately) + verified `PinTable` Verus/Lean proofs | Disjoint from M4.2 — may run before or in parallel with M4.2 |
| **M4.2** | Deadlock-free working-set eviction (`try_lock`, `Arc` drop + cleared `raw_head`/`raw_full`), rolling-window reads, typed `MemoryExhausted` — zero format bump | M4.0 | Real bounded working-set eviction and basis-safe rewarm runtime | May run in parallel with M4.1; prerequisite for M4.3 |
| **M4.3** | Single clean `CHK24`/`REG8` cutover: fixed `SlotRange` chunks, consolidation-invariant semantic commitments, per-slot continuation pages, producer-verified cold summaries, bounded `ValueReader`, paged reopen (`G7`) | M4.2 | Single placement group & single-slot fanout larger than memory; `O(touched)` checkpoint cutover & bounded reopen | Exclusive owner of checkpoint/region codec cutover (`CHK23`/`REG7` → `CHK24`/`REG8`) |
| **M4.4** | Durable `RetainedBasisToken` resumable historical cursors surviving consolidation, restart, prune race & index discard | M4.3 | Reclaim-fenced `ReaderLoan` & immutable retained-basis cursor contract | Requires M4.3 semantic commitments |
| **M4.5** | Writeback-fenced (`WritebackFenceDrain`), durable-epoch (`ExtentAllocationGeneration`) safe two-phase reclamation across `Segment`/`Extent`/`Arena` & candidate-cancel reuse | M4.1, M4.3, M4.4 | End-to-end safe reclamation, measurable space return, zero ABA/write-after-reuse, verified `AllocationGenerationGuard` & `ReclamationPlanValidator` | Requires M4.1 `PinTable`, M4.3 chunks, M4.4 `ReaderLoan` |
| **M4.6** | Composed `StoreM4ScaleReclaim.tla` × `StoreDurability.tla` + 5 witnesses, `store_m4_dst` (`S1–S10`) NDJSON trace validation, full Lean+Verus+`M1–M10`+`10×` multi-axis scaling gate | M4.5 | Complete 4-layer formal + scaling certification evidence | Sequential after M4.5 |
| **M4.7** | Operational docs, real-crate DX spike & §10 consumer acceptance (Daymark, Bank World, CAD, Semantic Git), cross-workspace certification, Roadmap closeout | M4.6 | Milestone 4 & Database Foundation Roadmap closed on `master` | Final closeout phase |

---

### Phase M4.0 — Formal Tooling Integration, DX Reference Spike & Plain-Cargo Verus Seam *(~0.5–1 day)*
**Owner:** Work PC | **Prerequisites:** Store Milestone 3 (`C1–C6`) merged on `master`

**Deliverables:**
1. Port audited `tools/lean/run_lean.py` and `tools/verus/run_verus.py` from `worth-orchestration/tools/` into `recnepspencer/worth/tools/`, and add `-DTLA-Library` cross-directory module resolution to `tools/tla/run_tlc.py`.
2. Install portable `verus` under `%USERPROFILE%\.local\tools\verus\` on Work PC (Lean 4 `elan`/`lake` and TLA+ `tla2tools.jar` are already installed).
3. Author compiling `docs/dx-reference/store_m4_scale_and_branches.rs` (§8) against stub traits + initial `trybuild` fences (§11) to lock M4's public DX before implementation.
4. Stand up `proofs/lean/store_m4/` Lake skeleton and `specs/tla/store_m4/` skeleton, and verify **one real production-included Verus transition kernel** (minimal `PinTable` skeleton) compiles cleanly under plain `cargo` without `vstd` AND kills a production-body mutant under `tools/verus/run_verus.py`.

**Ownership & Placement:** Extends `tools/tla/run_tlc.py`; adds `tools/lean/`, `tools/verus/`, `proofs/lean/store_m4/`, `specs/tla/store_m4/`, and `docs/dx-reference/store_m4_scale_and_branches.rs`. No production runtime behavior changes in M4.0.

**Exit Gate:** All 3 formal runners, `store_m4_scale_and_branches.rs`, and the plain-`cargo` Verus mutant-kill smoke check pass green with zero impact on ordinary workspace compile times.

---

### Phase M4.1 — O(1) Whole-Home `HomeRootManifest` / `PinTable` + Relational Branch Lifecycle *(Moved First — Unblocks Semantic Git Phase 2 Immediately!)*
**Owner:** Work PC | **Prerequisites:** Phase M4.0

**Why First:** Disjoint from Relational chunking, immediately unblocks Home PC's Semantic Git Phase 2, and validates pin protection against Store's already-live displaced-segment retirement from day one!

**Deliverables:**
1. Authoritative `HomeRootManifest` (Native + World + Signal + installed program digest + **`product_map_root`**) + durable `PinTable` + `BranchLease` (`Active → Expiring → Expired`, `NestedDropPolicy`, Memory-backend parity) exposed via `RuntimeWorldRetentionPort` (`D3b`).
2. Wire `PinTable` **immediately** into Store's `reclaim_guard.rs:5` and reopen (restored before any maintenance runs) so pinned roots and nested grandchild overlays survive displaced-segment retirement + crash/reopen while unrelated displaced garbage still reclaims (`D3b`).
3. Close `RelationalForkDenial::RetainedSourceDurabilityUnsupported` (`fork.rs:36`), preserve durable `archive` (`lifecycle_archive.rs:12`) and `delete` + permanent retired-name enforcement (`D3a`).
4. Implement stale-candidate revalidation and Home-frontier delta composition at the FIFO patch-position reservation (`fork_publication.rs:143`) under the exact §4.3 lock order, handling concurrent branch preparations and preparation-vs-`archive` races cleanly (`D3a`).

#### Authority & Typed Progression (M4.1)

| Transition | Exact input / proof | Granting owner & admission | Output / proof | Next permitted consumer | Refusal & effect posture |
|---|---|---|---|---|---|
| `capture_and_pin_frontier` | `PinSpec { scope, parent_pin, nested_drop_policy }` | `worth-store` `PinTable` under reclamation fence (verifies live `parent_pin` if nested) | `PinnedFrontierLease { pin_id, manifest: HomeRootManifest }` | Semantic Git `EnvironmentFork`, Relational `BranchHandle`, `reclaim_guard` | `ParentPinNotActive` / `StorageUnavailable`; zero durable pin written |
| `expire_lease` | `PinId` | `worth-store` `PinTable` (durable state transition) | `LeaseState::Expiring` (fences new loans) → `Expired` iff `active_reader_loans == 0 && live_child_pins == 0` | Reclaim reachability projection | Never drops pinned roots while any loan or child pin is live |
| `fork_branch` | `BranchName`, `BranchForkSource::ActiveBranch \| HistoricalRoot` | `worth-relational-branch` (`branch/fork.rs`) + `PinTable` source pin | `BranchHandle` bound to immutable source root | Branch read/edit/publish | `NameRetired`, `SourceArchived`, `SourceDeleting`, `SourceNotRetained`; zero partial ref |
| `publish_branch_delta` | `PreparedBranchDelta { base_frontier, patch_reservation }` | `worth-relational-durability` (`fork_publication.rs:143`) under §4.3 lock order | Committed `BranchPublicationReceipt` on current Home frontier | Readers, downstream checkpoints | Revalidates if Home frontier moved; `BranchArchived` / `Conflict` leaves branch clean |
| `archive_or_delete_branch` | `BranchName` | `worth-relational-branch` (`lifecycle_archive.rs:12`) | Durable `Archived` / permanently `Retired` tombstone | Reclaim (`M4.5`) drops deleted root pin only | Retired names can never be recreated (`NameRetired`); sibling & nested pins untouched |

**Ownership, Placement & Cutover:** Extends `worth-store` (`reclaim_guard.rs`, new `pin_table` module), `worth-runtime-world` (`RuntimeWorldRetentionPort`), `worth-relational-branch` (`branch/fork.rs`, `lifecycle_archive.rs`), and `worth-relational-durability` (`fork_publication.rs`). **Cutover:** Removes `RelationalForkDenial::RetainedSourceDurabilityUnsupported` (`fork.rs:36`) and retires any ad-hoc unpinned whole-Home snapshot hooks.

#### Phase M4.1 Test Matrix

| Requirement / law | Setup & entry | Action or failure sequence | Expected result & effects | Decisive observation / defect exposed | Test boundary & lane |
|---|---|---|---|---|---|
| O(1) `HomeRootManifest` pin with `product_map_root` (`D3b`) | Multi-product Home (Native + World + Signal + `product_map_root`) on Directory & Memory | Acquire `PinnedFrontierLease`; advance HEAD across multiple displaced-segment retirements + crash/reopen | Pinned manifest & product-map root remain readable; unrelated displaced segments reclaim | Exposes omitted `product_map_root`, linear base scans on fork, or missing reopen pin restore (`M4`, `M5`) | `worth-store` + `worth-runtime-world` integration lane (`S4`) |
| Nested grandchild lease expiry (`D3b`, `LeaseExpiryNoReaderLoss`) | Parent pin → child pin → grandchild pin + active reader loan | Expire parent lease (`Active → Expiring`), trigger maintenance, attempt new loan on parent, drain grandchild | New loan on parent refused; grandchild read succeeds; parent moves to `Expired` and roots release only after grandchild drains | Exposes premature parent eviction or forgotten transitive ancestor edges (`M5`) | `worth-store` pin court + Verus `PinTable` + Lean 4 (`S4`) |
| Durable historical-root fork & permanent name retirement (`D3a`) | Active branch `b1` with historical checkpoint `c0` | Fork `b2` from `c0`, edit `b1` and `b2` with equal-looking values, `archive` `b1`, `delete` `b1` with crash cuts, attempt recreate `b1` | `b2` retains exact `c0`+delta state; `b1` never resurrects and recreate returns `NameRetired` | Exposes branch resurrection across crash or dropped sibling sharing (`M6`) | `worth-relational-branch` crash court (`S4`) |
| Concurrent branch prep & archive race (`D3a`, `S9`) | Branches A and B prepared from same Home frontier `F0`; pause both before FIFO cutover | Commit A (`F0 → F1`), resume B; separately race Branch C prep against `archive(C)` | B revalidates and applies cleanly onto `F1` (both A & B survive reopen); C returns `BranchArchived` | Exposes lost Home-frontier update from stale candidate overwrite (`M6`) | `worth-relational-durability` concurrency court (`S9`) |

**Required Proofs (Stable Pin Kernel Built Here):**
- **Verus:** `PinTable::{acquire, release, expire_lease}` (production-included, mutant killed).
- **Lean 4:** `StoreGCPinSafety` & `LeaseExpiryNoReaderLoss` *(single shared theorem consumed by Semantic Git!)*.

**Exit Gate:** `S4`, `S9`, `M5`, `M6`, pinned-root survival across displaced-segment retirement + crash/reopen, Verus + Lean 4 green, Codex review APPROVED. *(Ping Home PC orchestrator immediately that M4.1's `HomeRootManifest` / `PinTable` seam is live on `master` for Semantic Git Phase 2!)*

---

### Phase M4.2 — Deadlock-Free Working-Set Eviction & Real Larger-Than-Memory Feedback *(Zero Format Bump!)*
**Owner:** Work PC | **Prerequisites:** Phase M4.0 *(may run in parallel with or right after M4.1)*

**Deliverables:**
1. Decouple durable member custody from operation-scoped resident borrows in `RelationalRegionRetention` (`retention.rs:95`); clear `raw_head`/`raw_full` (`cell_load_cache.rs:8`) under eviction; fix `RegionLeaseCustody::retire_native` & `Clone` (`lease/cell_custody.rs`); bind `WorkingSetAccountant` strictly to live `Arc<ResidentRegion>` reservations + raw caches (`D1`).
2. Implement `try_lock()` deadlock-free evictor (reserving memory before taking any cell `state` mutex; evicting only cells verified in *this* home's member directory; basis-preserving rewarm via `BasisIdentity`; separate head vs. historical allowances) (`D1`).
3. Convert M2 whole-root/whole-mirror declared demands and probe-retry escalation (`milestone-2.md:335-344`) to bounded rolling windows; return typed `MemoryExhausted` on genuine saturation (`D1`, `D7`).

#### Authority & Typed Progression (M4.2)

| Transition | Exact input / proof | Granting owner & admission | Output / proof | Next permitted consumer | Refusal & effect posture |
|---|---|---|---|---|---|
| `reserve_working_set` | `ResidencyPoolKind (Head \| Historical)`, `bytes` | `WorkingSetAccountant` (prior to locking any cell `state` mutex!) | `MemoryReservationGuard` (released strictly on `Arc` drop + cache clear) | Cell loader / rolling-window scanner | Triggers `try_lock` eviction; if still saturated, returns `MemoryExhausted` without deadlock or partial state |
| `acquire_resident_borrow` | `DurableMemberCustody`, `BasisIdentity`, `MemoryReservationGuard` | `worth-relational-storage` cell loader | `ResidentRegionBorrow<'op>` (operation-scoped, never stored in root `OnceLock`) | Query/Relational read/edit window | Basis mismatch on rewarm → typed `CorruptStorage` / stale-basis refusal; never serves wrong branch |
| `evict_unborrowed_cell` | Candidate `CellId` with `live_borrows == 0` | `WorkingSetAccountant` via non-blocking `try_lock()` after verifying membership in *this* home's directory | Cell set to `Cold`, `raw_head`/`raw_full` cleared, `Arc<ResidentRegion>` dropped, reservation freed | Waiting `reserve_working_set` | Skips contended or foreign-home cells cleanly; zero AB/BA lock cycle |

**Ownership, Placement & Cutover:** Extends `worth-relational-storage` (`retention.rs`, `cell_load_cache.rs`, `lease/cell_custody.rs`, `WorkingSetAccountant`). **Cutover:** Removes `OnceLock` root-lifetime resident leases in `retention.rs:95` and replaces M2's whole-root/whole-mirror declared demand & probe-retry whole-root escalation (`milestone-2.md:335-344`) with bounded rolling windows.

#### Phase M4.2 Test Matrix

| Requirement / law | Setup & entry | Action or failure sequence | Expected result & effects | Decisive observation / defect exposed | Test boundary & lane |
|---|---|---|---|---|---|
| Real `Arc` drop & `raw_head`/`raw_full` cleanup (`D1`, `S1`) | DB `4×` larger than `ResidencyBudget` with original root kept open | Full scan 1 → full scan 2 + concurrent edits while original root stays open | Resident bytes stay `<= budget` across both scans; evicted cells show zero strong `Arc`s and empty `raw_head`/`raw_full` | Exposes root `OnceLock` leak or `raw_cache` retention (`M1`) | `worth-relational-storage` residency court (`S1`) |
| Deadlock-free concurrent eviction + basis-safe rewarm (`D1`, `S1`) | 8 concurrent reader/writer threads across two branches under tight budget | Hammer reads/edits triggering eviction + rewarm after branch head updates | Zero deadlocks; every rewarmed read matches canonical memory reference for its exact `BasisIdentity` | Exposes AB/BA lock order inversion or skipped `BasisIdentity` check (`M1`, `M2`) | `worth-relational-storage` stress court (`S1`) |
| Genuine saturation returns typed `MemoryExhausted` (`D1`, `D7`, `S3`) | Active reader borrows pin 100% of `ResidencyBudget` | Issue new read/edit demand → observe error → drop borrows → retry | Returns typed `MemoryExhausted` (mapped to `RetryHint::MemoryPressure` / HTTP `503`), no panic/empty result/skipped validation; retry succeeds | Exposes silent cap bypass, panic, or poisoned reservation counter (`S3`) | `worth-relational-storage` + Bank World disposition court (`S3`) |

**Verification:** Real larger-than-memory DB courts (`S1`, `S3`, `M1`, `M2`), explicitly testing a second scan with the original root still open + multi-thread concurrent eviction stress. *(Zero ceremonial Verus/Lean on `Mutex`/`Arc` glue!)*

**Exit Gate:** `S1`, `S3`, `M1`, `M2` green, Codex review APPROVED.

---

### Phase M4.3 — Single Clean Format Cutover (`CHK24`/`REG8`): Chunked Placement Groups, Per-Slot Continuations, Cold Summaries & Paged Reopen (`G2`, `G3`, `G7`)
**Owner:** Work PC | **Prerequisites:** Phase M4.2

**Deliverables:**
1. Single clean cutover from `CHK23`/`REG7` to `CHK24`/`REG8` with fixed non-overlapping `SlotRange` chunks, **consolidation-invariant** `ChunkSemanticEntry` Merkle commitments separated from checkpoint-authenticated `ChunkLocatorBinding`s, chunk-wise capture consolidation, and chunk-wise CoW edit (`D2`).
2. Bounded **per-slot continuation pages** for high-version-count record histories and high-degree vertex adjacency lists (`region_image.rs:60`), plus bounded `ValueReader` large-value streaming capped by `max_in_flight_fill_bytes` (`D2`).
3. **Producer-derived / pre-hash verified** cold kind & adjacency summaries at chunk granularity (reusing `truth_kind_scan/directory.rs:19`) + mirror pin/lifecycle separation (`D2`, `G3`).
4. **Paged reopen & `O(touched)` checkpoint cutover (`G7`):** Page `PortableRecordDirectory` (`portable.rs:14`), **all four `RecordIdentitySubsystem` tables** (generation high-water marks, reusable slots, pending reservations, per-partition append frontiers: `record_identity/mod.rs:124`, `checkpoint_restore/record_identity.rs:14,49,127`), and commit-index/patch-position directories (`cold_directory.rs:94`); replace linear checkpoint cutover pin scans (`publication_cutover.rs:49` -> `selected_manifest_pins.rs:104`) with incremental/carried pin sets.

#### Authority & Typed Progression (M4.3)

| Transition | Exact input / proof | Granting owner & admission | Output / proof | Next permitted consumer | Refusal & effect posture |
|---|---|---|---|---|---|
| `seal_chunk_image` | Non-overlapping `SlotRange`, canonical records, per-slot continuations | `worth-relational-storage` `REG8` writer (derives & verifies kind/adjacency summary *before* hashing) | Sealed `ChunkSemanticEntry` + authenticated `ChunkLocatorBinding` | `CHK24` checkpoint manifest & cold reader | Rejects overlapping slots, unverified summary, or oversize chunk before write |
| `consolidate_history_chunk` | Layered `ChunkSemanticEntry` + new flattened physical locator | `worth-relational-storage` consolidator (verifies identical `semantic_chunk_commitment`) | Updated `ChunkLocatorBinding` with **unchanged** `root_semantic_commitment` | `CHK24` checkpoint & pre-consolidation `RetainedBasisToken`s | Refuses consolidation if derived semantic commitment diverges |
| `open_value_reader` | `ValueLocator`, `MemoryReservationGuard` (`<= max_in_flight_fill_bytes`) | `worth-relational-storage` streaming reader | `ValueReader<'loan>` yielding bounded chunks | Application / CAD / Semantic Git caller | Refuses unbounded materialization; `MemoryExhausted` if stream budget unavailable |
| `restore_paged_checkpoint` | `CHK24` manifest + `ResidencyBudget` | `worth-relational-durability` (`checkpoint_restore/record_identity.rs`) | Paged handles for `PortableRecordDirectory`, all 4 `RecordIdentitySubsystem` tables, & commit/patch directories | Live serving & allocator | Restores strictly under `ResidencyBudget`; never loads whole directory eagerly |

**Ownership, Placement & Cutover:** Extends `worth-relational-storage` (`region_image.rs`, `truth_kind_scan/directory.rs`, new chunked/continuation/streaming codecs) and `worth-relational-durability` (`native_checkpoint_codec.rs`, `portable.rs`, `record_identity/mod.rs`, `checkpoint_restore/record_identity.rs`, `cold_directory.rs`, `publication_cutover.rs`). **Cutover:** Single hard format cutover from `CHK23`/`REG7` to `CHK24`/`REG8` (no dual-codec legacy maintenance); removes linear checkpoint pin scan (`publication_cutover.rs:49` -> `selected_manifest_pins.rs:104`).

#### Phase M4.3 Test Matrix

| Requirement / law | Setup & entry | Action or failure sequence | Expected result & effects | Decisive observation / defect exposed | Test boundary & lane |
|---|---|---|---|---|---|
| Single placement group & single-slot fanout `> memory` (`D2`, `S2`) | 1 partition `> budget`; (a) 1 record with version history `> budget`; (b) 1 vertex with adjacency list `> budget` of small values | Cold point lookup, kind scan, adjacency traversal, local edit, and large-value `ValueReader` stream | Peak heap stays `<= budget`; touched chunks only warm; results identical to canonical full scan | Exposes whole-group load, per-slot OOM (`region_image.rs:60`), or whole-value spike (`M3`) | `worth-relational-storage` cold-scale court (`S2`) |
| Pre-hash summary completeness & consolidation invariance (`D2`, `M3`) | Valid chunk with known kind/adjacency edge; also flatten layered history into new locator | (a) Mutate producer to omit edge before hashing; (b) consolidate history and compare `root_semantic_commitment` | (a) Sealed writer / cold-vs-canonical verifier rejects omitted edge; (b) `root_semantic_commitment` is bit-identical across locator change | Exposes authenticated-but-incomplete summary or locator-coupled root hash (`M3`) | Lean 4 `ChunkedCoverageAndSummarySound` + `worth-relational-storage` court (`S2`) |
| Paged reopen with outstanding reservations & `O(touched)` cutover (`G7`, `S2`, `S10`) | High-cardinality DB with thousands of pending reservations, reusable slots, append frontiers, & long commit history | Reopen under same `ResidencyBudget`; allocate/cancel reservations; run checkpoint cutover at `1×` and `10×` cardinality | Reopen stays under budget; post-reopen slot reuse & reservation continuity exact; cutover work stays `O(touched)` | Exposes eager `RecordIdentitySubsystem` restore (`record_identity.rs:14,49,127`) or linear cutover scan (`M4`) | `worth-relational-durability` reopen & scaling court (`S2`, `S10`) |

**Required Proofs (Stable Commitment Math Built Here):**
- **Lean 4:** `ChunkedCoverageAndSummarySound` (non-overlapping Merkle partition + consolidation invariance + pre-hash summary completeness).

**Exit Gate:** Single placement group + single-slot long-history/high-degree-adjacency larger than memory + high-cardinality/outstanding-reservation/long-history reopen under the same memory allowance (`S2`, `M3`, `M4`), Lean 4 green, Codex review APPROVED.

---

### Phase M4.4 — Historical Retained-Basis Access & Resumable Cursor Tokens (`G5`)
**Owner:** Work PC | **Prerequisites:** Phase M4.3

**Deliverables:**
1. Durable `RetainedBasisToken` extending `RelationalBranchBasisDescriptor` with `home_incarnation`, `query_digest`, `canonical_key_position`, and consolidation-invariant `root_semantic_commitment` (`D4`).
2. Atomic resume-vs-prune `ReaderLoan` acquisition under Store's reclaim fence; exact restart behavior (resumes iff independently retained, `HistoryUnavailable` otherwise, never silently rebinding to `HEAD`); exact `HistoryUnavailable` / `UnsupportedHistoricalMeaning` / `CorruptStorage` separation; direct historical reads & canonical reconstruction after physical index discard (`D4`).

#### Authority & Typed Progression (M4.4)

| Transition | Exact input / proof | Granting owner & admission | Output / proof | Next permitted consumer | Refusal & effect posture |
|---|---|---|---|---|---|
| `open_historical_cursor` | `BranchReadBasis::Historical { commitment }`, `QueryPlan` | `worth-relational-durability` + `PinTable` reclaim fence (atomically acquires `ReaderLoan`) | `HistoricalCursor<'loan>` bound to `ReaderLoan` + `RetainedBasisToken` | Caller paged iteration (`next_page`) | `HistoryUnavailable` if basis not retained; `UnsupportedHistoricalMeaning` on unsupported schema/query |
| `resume_from_token` | Encoded `RetainedBasisToken` (possibly pre-consolidation & post-restart) | `worth-relational-durability` under reclaim fence (verifies `home_incarnation`, `query_digest`, and `root_semantic_commitment` via checkpoint locator map) | New `HistoricalCursor<'loan>` resuming strictly after `canonical_key_position` | Caller paged iteration | Never silently rebinds to `HEAD`! `HistoryUnavailable` if pruned, `CorruptStorage` on tampered commitment/incarnation |
| `reconstruct_after_index_discard` | `HistoricalCursor<'loan>` when auxiliary physical index was dropped | `worth-relational-storage` canonical chunk reader | Exact ordered rows from canonical `CHK24`/`REG8` chunks using `canonical_key_position` | Caller paged iteration | Zero skipped or duplicated rows; bounded by `Historical` residency pool |

**Ownership, Placement & Cutover:** Extends `worth-relational-durability` (`RelationalBranchBasisDescriptor` → `RetainedBasisToken`, reclaim-fenced `ReaderLoan`) and `worth-query` / `worth-relational-storage` historical cursor entry points. **Cutover:** Retires any cursor resume path keyed by physical index offsets or volatile in-memory handles.

#### Phase M4.4 Test Matrix

| Requirement / law | Setup & entry | Action or failure sequence | Expected result & effects | Decisive observation / defect exposed | Test boundary & lane |
|---|---|---|---|---|---|
| Pre-consolidation token across restart & HEAD advance (`D4`, `S5`) | Issue `RetainedBasisToken` mid-scan at commit `C1` | Advance HEAD to `C10`, consolidate history (changing physical locators), drop indexes, restart Home, resume token | Resumes exact next canonical key from `C1` without duplicates/gaps | Exposes locator-coupled token, index-offset coupling, or silent `HEAD` rebind (`M7`) | `worth-relational-durability` + `worth-query` court (`S5`) |
| Atomic resume-vs-prune race & error taxonomy (`D4`, `S5`) | Token at retained commit `C1` racing concurrent prune of `C1`; also test tampered token & unsupported historical schema | Race `resume_from_token` against `ReclaimIntent(C1)`; invoke with corrupt bytes and unsupported query | Race yields either complete exact read (loan won) OR typed `HistoryUnavailable` (prune won) — never mid-read loss; tampered → `CorruptStorage`; unsupported → `UnsupportedHistoricalMeaning` | Exposes TOCTOU gap between retention check and `ReaderLoan`, or collapsed error kinds (`M7`) | `worth-relational-durability` concurrency + taxonomy court (`S5`) |

**Required Proofs:**
- **Lean 4:** `RetainedBasisCursorImmutable`.

**Exit Gate:** `S5` and `M7` green across consolidation, restarts, prune races, and discarded indexes, Lean 4 green, Codex review APPROVED.

---

### Phase M4.5 — Writeback-Fenced, Durable-Epoch Safe Physical Reclamation (`G6`)
**Owner:** Work PC | **Prerequisites:** Phases M4.1, M4.3, M4.4

**Deliverables:**
1. Project full `Reach` (live/archived branches + durable `PinTable`/`BranchLease` + existing `retain_members` cold cells + `RootProtectionRegistry` + checkpoints + recovery obligations) as an opaque protected set into Store's two-phase `RetiredArtifact::{Segment, Extent, Arena}` reclamation, enforcing canonical-coverage-before-prune and incremental bounded `ReclaimBudget` maintenance (`D5`).
2. Scope crash- and cancel-monotone `ExtentAllocationGeneration { root_publication_generation, extent_fence_seq }` (advanced on every extent claim/cancel/reclaim WAL barrier before reissue) to `ExtentArena` frame coordinates (`worth-store-buffer-pool` & `worth-store-physical-format`; zero Relational format change), carry `ArenaExtentFrameIdentity` through `writeback_claim.rs:209`, and enforce **`WritebackFenceDrain`** across exact, split, and coalesced overlapping ranges before allocator admission (`D6`).
3. Enforce dual allocator admission paths (Path A: `Intent → PublishFreeSpaceRoot → ReclaimComplete → AdmitToAllocator`; Path B: `CancelCandidateExtent → CheckpointCovered → AdmitToAllocator`) and chained-crash interruptible recovery (`RecoverRestorePins → RecoverResumeFreeSpaceRoot → RecoverResumeReclaimComplete → RecoverServing`) with zero double-free (`D5`, `D6`).

#### Authority & Typed Progression (M4.5)

| Transition | Exact input / proof | Granting owner & admission | Output / proof | Next permitted consumer | Refusal & effect posture |
|---|---|---|---|---|---|
| `plan_reclamation` | Opaque `ProtectedReachSet` (obeying `road1.toml:525-560`) + `ReclaimBudget` | `worth-store` `ReclamationPlanValidator` (verifies `candidate ∩ Reach == ∅` & canonical successor coverage) | Sealed `ReclaimIntentRecord` | Durable WAL `ReclaimIntent` | Refuses any reachable or uncovered artifact (`M5`, `M8`); zero mutation |
| `publish_free_space_and_complete` | Durable `ReclaimIntentRecord` | `worth-store` `retirement_release.rs:125` (persists free-space root, then durable `ReclaimComplete` barrier) | `RetirementReleaseProjection::Completed` | `WritebackFenceDrain` → `AdmitToAllocator` | Crash before `ReclaimComplete` resumes idempotently (`resume.rs:101`) without double-release (`M9`) |
| `drain_and_admit_range` | `(ArenaId, ArenaByteRange)` + **either** `RetirementReleaseProjection::Completed` **or** `CheckpointCoveredCancellation` (`extent_copy/resolution.rs:157`) | `worth-store-buffer-pool` `AllocationGenerationGuard` (waits until zero overlapping exact/split/coalesced writebacks in `writeback_claim.rs:209`) | Allocator admission at strictly newer `ExtentAllocationGeneration` (`g' > max_seen(range)`) | Next extent allocator / writer | Blocks while any overlapping writeback is paused; rejects stale-`g` writeback submission (`M10`) |

**Ownership, Placement & Cutover:** Extends `worth-store` (`retirement_release.rs`, `retirement_release/resume.rs`, `extent_copy/resolution.rs`, `reclaim_guard.rs`), `worth-store-buffer-pool` (`writeback_claim.rs`, `AllocationGenerationGuard`), and `worth-store-physical-format` (`ExtentAllocationGeneration` in `ExtentArena` coordinates — zero Relational format change). **Cutover:** Removes any ungenerationed `ExtentArena` frame lookup and any direct allocator return bypassing `WritebackFenceDrain`.

#### Phase M4.5 Test Matrix

| Requirement / law | Setup & entry | Action or failure sequence | Expected result & effects | Decisive observation / defect exposed | Test boundary & lane |
|---|---|---|---|---|---|
| Paused writeback fence across exact, split & coalesced overlap (`P0`, `D6`, `S8a`) | Pause gen-`g` writeback claim (`writeback_claim.rs:209`) on `[A, B)` | Complete retirement of `[A, B)` (and split `[A, M)` / coalesced `[A-k, B+k)`); attempt allocator reuse → release/cancel paused claim → retry reuse → submit stale `g` | Reuse **blocked** while writeback paused; **succeeds** at `g' > g` only after drain; stale-`g` submission refused | Exposes silent write-after-reuse corruption or split/coalesce overlap miss (`M10`) | `worth-store-buffer-pool` + `worth-store` fence court (`S8`) |
| Candidate-cancel reuse without root publication (`P1`, `D6`, `S8b`) | Write candidate extent at `g`, durably cancel (`extent_copy/resolution.rs:157`) | Checkpoint covers cancellation WAL (no root pub), reopen Home, allocate same range | Range reissued strictly at `g' > g` via Path B; old gen-`g` frame never aliases `g'` | Exposes same-generation ABA reuse when root publication gen did not advance (`M10`) | `worth-store` copy-cancel recovery court (`S8`) |
| Chained crash during recovery reclaim resume (`P1`, `D5`, `S7`) | Durable `ReclaimIntent` in WAL | Crash → reopen → inject second crash **after** `RecoverResumeFreeSpaceRoot` before `RecoverResumeReclaimComplete` → reopen → reallocate & write at `g'` → crash + reopen again | Zero double-free of `g'`; reallocated data intact; `ReclaimComplete` settles idempotently | Exposes recovery re-applying release onto newly allocated `g'` (`M9`) | `worth-store` fault-injection crash court (`S7`) |
| End-to-end `Reach` preservation & measurable space return (`D5`, `S6`) | Repeated edit/checkpoint/delete cycles with live `retain_members` cold cells, `RootProtectionRegistry`, sibling branches, & loans | Run bounded `ReclaimBudget` maintenance while holders active, then release holders and run maintenance | Zero live/cold-cell/sibling loss; canonical coverage verified before journal prune; disk bytes drop measurably after release | Exposes missed `retain_members` reach edge, premature journal prune, or permanent space leak (`M5`, `M8`) | `worth-store` + `worth-relational-durability` reclaim court (`S6`) |

**Required Proofs (Stable Storage Safety Kernels Built Here):**
- **Verus:** `AllocationGenerationGuard` + `ReclamationPlanValidator` (both production-included, mutants killed).
- **Lean 4:** `AddressReuseGenerationAndWriteFenceSound`, `ReclaimPreservesAllReachableRoots`/`CanonicalCoverageBeforePrune`, `InterruptedReclaimIdempotent`.

**Exit Gate:** `S6`, `S7`, `S8`, `M8`, `M9`, `M10` green (including blocked→released→refused paused writeback, candidate cancel → reopen → reuse without root pub, split/coalesce overlap, and chained-crash recovery + second reopen), Verus + Lean 4 green, Codex review APPROVED.

---

### Phase M4.6 — Composed Formal Hardening & Multi-Axis Scale-Headroom Gate *(M3 C6 Equivalent)*
**Owner:** Work PC | **Prerequisites:** Phase M4.5

**Deliverables:**
1. Compose `specs/tla/store_m4/StoreM4ScaleReclaim.tla` × `StoreDurability.tla` with all split-barrier actions (§9.2) and verify all invariants + all 5 non-vacuity reachability witnesses (`0` TLC errors).
2. Extend C6's compiled `cumulative_oracle` DST harness (`store_m4_dst`) with all 10 scenarios (`S1–S10`) emitting **observed production barrier transitions, generations, and fates** into NDJSON traces validated by `tools/tla/run_tlc.py trace`. Run 64–256 seeds per slice and the scaled closing sweep (`10,000+` seeds on Linux/tmpfs or highest verified sweep on Windows NTFS).
3. Certify the complete 6-theorem Lean 4 manifest (`tools/lean/run_lean.py build --manifest`), complete 3-kernel Verus suite + in-tree production-body mutant kill (`S10`), all 10 named mutants (`M1–M10`), all 4 external-crate `trybuild` fences (§11), and `10×` multi-axis scaling headroom (DB size, record cardinality, outstanding reservations, commit history).

#### Phase M4.6 Test Matrix (Cross-Layer Certification)

| Requirement / law | Setup & entry | Action or failure sequence | Expected result & effects | Decisive observation / defect exposed | Test boundary & lane |
|---|---|---|---|---|---|
| Composed TLA+ invariants + 5 non-vacuity witnesses (§9.2) | `StoreM4ScaleReclaim.tla` × `StoreDurability.tla` via `-DTLA-Library` | Run TLC model check across all invariants + Witnesses 1–5 | `0` invariant violations; all 5 reachability witnesses strictly reached | Exposes deadlocked guards, vacuous invariants, or unhandled crash interleaving | `tools/tla/run_tlc.py check` |
| Observed production trace fidelity (`S1–S10`) | Compiled `store_m4_dst` on Memory + Directory backends | Execute all 10 scenarios across seeded fault schedules; pipe emitted production NDJSON traces to TLC | Every trace accepted by `StoreM4ScaleReclaim.tla` (`0` trace mismatches) | Exposes drift between TLA+ model and real Rust barrier ordering / generations | `store_m4_dst` + `tools/tla/run_tlc.py trace` |
| Multi-axis `10×` scaling headroom & Verus mutant kill (`S10`, `M1–M10`) | Independently scale DB size, record cardinality, pending reservations, & commit history `10×` | Measure O(1) fork, `O(touched)` edit, bounded cutover & reopen under constant `ResidencyBudget`; inject `M1–M10` & Verus body mutant | Constant/O(touched) growth holds on every axis; every mutant `M1–M10` + Verus mutant fails its named layer | Exposes hidden O(N) scans, memory creep, or hollow verification (`M1–M10`) | `store_m4_dst` + `run_verus.py` + `run_lean.py` + `trybuild` |

**Exit Gate:** Entire 4-layer formal gate + `S1–S10` + `M1–M10` + `trybuild` fences green with zero waivers, Codex review APPROVED.

---

### Phase M4.7 — Operational Guidance, Consumer Acceptance, Cross-Workspace Certification & Roadmap Closeout *(M3 C5 Equivalent)*
**Owner:** Work PC | **Prerequisites:** Phase M4.6

**Deliverables:**
1. Update operational guidance covering branch/history lifecycle, cold access, working-set pressure & exhaustion dispositions, safe reclamation, and asymptotic scaling laws (`docs/application-home.md`, `docs/how-it-works.md`, `docs/capability-index.md`, `plans/database-foundation/roadmap.md`).
2. Certify `docs/dx-reference/store_m4_scale_and_branches.rs` (§8) against the real public crates and verify the §10 consumer acceptance assertions across Bank World, Daymark, CAD, and Semantic Git's `HomeRootManifest` / `PinTable` seam.
3. Run the full cross-workspace certification matrix (Store, Relational, World, Signal, Bridge, Query, Bank World; excluding `worth-ui`), `scripts/check-constitution.ps1`, lockfile alignment, `rustfmt`, strict clippy, and 400-line caps.
4. Independent Codex Milestone 4 Certification closing the **Database Foundation Roadmap** (`roadmap.md:598-617`).

**Exit Gate:** Full certification matrix, constitution, consumer acceptance, and independent Codex sign-off green; Milestone 4 merged to `master`.

---

## 13. Cross-Spec Alignment Note (`worth-orchestration/milestones/semantic-git.md`)

When Phase M4.1 lands on `master`, update `worth-orchestration/milestones/semantic-git.md` Phase 2 item 1 (§4.1, §4.4, §9.1 rows 10–11, §12 Phase 2) with a clean 3-line alignment note so there is **zero duplicated ownership**:
1. Semantic Git Phase 2 consumes Store M4.1's already-landed `HomeRootManifest`, durable `PinTable`, `BranchLease`, and `RuntimeWorldRetentionPort` directly (adding only `EnvironmentFork` sandbox posture, `ForkOrigin::AtAttestedTime` `FrontierIndex`, `ForkGrantCeiling`, and `into_dry_run::<V2>` on top).
2. Semantic Git's `SemanticGit.tla` instantiates M4's shared `PublishForkPin` / `ExpireForkLease` / `RetireBranchRef` / `CrashAndReopen` actions and reuses M4's already-verified Verus `PinTable` and Lean 4 `StoreGCPinSafety` / `LeaseExpiryNoReaderLoss` theorems without re-proving them.
