# Fast Track To Runtime Integration

This is the engineering spec for C.13 and for the parts of C.11 that Runtime
Integration Milestones 1 through 3 need. It replaces the planned
`physical-reconstruction-c13-platform-integration-and-s10-entry.md`.

## Goal And Decision

Part II starts as soon as Store offers one honest physical facade that
Milestones 1 to 3 can build on. Those milestones need:

- one open-or-recover entry;
- bounded record reads, generation-fenced record submission and exact fate
  after a crash;
- a bounded way to find their own durable state after a restart;
- capability rows that tell the truth;
- reopen after any crash, including a crash before the first checkpoint.

They do not need:

- blob ingest, release, reclaim or tier movement;
- maintenance;
- LSM;
- export or import.

So C.11 stops where it is, except for the items in the slices below. C.13 ships
in full, minus the joined hostile campaign, which already moved to S.12. Every
other C.11 item is deferred with an owner and a return point (see
[Deferred Work](#deferred-work)). C.11, C.12 and the rest of C.13 are finished
later, each item before the Part II milestone that first consumes it.

### What makes a deferral lawful

The facade defines what Part II can reach. A capability is `Present` only when
a facade port exposes it, and it is `Absent` when no port does. A deferral is
lawful only when all three of these hold:

- no facade port reaches it;
- its capability reports `Absent` through negotiation;
- deferring it changes no format, law or public API that Part II binds to.

The Store-internal blob, maintenance and layout facades stay inside the Store
crates. The dependency fence in slice 7 stops application composition from
reaching them. Anything that fails one of the three tests is in a slice below.

### What Milestones 1 to 3 consume

RI is `runtime-integration-roadmap.md`. Line numbers are as of 2026-10-06.

**Milestone 1** is the Query-side refactor (RI:405). It needs:

- one Store instance per composition (RI:460);
- open, which admits and recovers, and close, which drains (RI:377-383);
- owned handles that can move between threads and tasks without serializing
  the Store (RI:428, RI:499);
- no Store import of Query (RI:473).

**Milestone 2** is the first real Store consumer. It needs:

- a pre-effect capacity reservation and pressure evidence (RI:557-560);
- mutation batches with exact identity, ordering, scope and durability basis
  (RI:561);
- point, range and streaming reads (RI:563);
- Part II's semantic-family mapping registry, which maps onto the physical
  artifact-family registry C.11 built (RI:569);
- negotiation that denies before execution (RI:571, RI:598);
- a restart in a fresh process that finds the durable semantic frontier
  without a broad scan (RI:378, RI:573).

**Milestone 3** binds durability to the facade (RI:669-693). It needs:

- a typed `Indeterminate` fate with per-operation fate after reopen (RI:677);
- retry deduplication by idempotency identity;
- a crash matrix at every progression edge.

## Standing Rules

- **No historical compatibility.** Store is undeployed and development data is
  disposable (C.11 "Store Format Policy"). The slices below still make every
  format decision Part II binds to before Part II starts.
- **Pre-plan the bug classes.** Each slice lists the classes it could produce
  and the contract that kills each recurring one. Use `worth-proof` contracts
  (sealed authority, `Performed`, `LinearResource`, `TransitionOutcome`). Do not
  add a type that guards a single site.
- **One canonical spelling.** A slice that adds an entry point removes or hides
  the one it replaces in the same slice.
- **Process:**
  - one implementer and one fresh independent reviewer per slice;
  - the reviewer's APPROVE follow-ups are fixed in the same slice;
  - commit when green;
  - scope cargo to the touched crates;
  - run heavy lanes once per slice, not once per iteration.

## Slices

### 1. Recovery before the first checkpoint

Today, initializing a store, writing, and crashing before the first checkpoint
gives `BLOCKED kind=WalInventory`. Nothing caught this because every test world
writes a checkpoint first. C.7 (lines 823-826 and 857-860) and C.11 (lines
1175-1181) define generation zero as a real recovery basis, and Milestone 3's
crash matrix binds to it.

Design, decided 2026-10-04:

- **Recovery admits the generation-zero basis.** A checkpoint at initialization
  is rejected as the alternative: checkpoint publication needs a Serving runtime
  and a non-empty WAL, so it leaves the same crash window.
- **Checkpoint presence is a three-way enum,** `Present | Absent |
  Unreadable(damage)`, taken from the backend's `Absent` arm. Only `Absent`
  admits generation zero.
- **One named WAL origin constant in `worth-store-wal`.** It replaces the three
  `GENESIS.get() + 1` sites and the raw `0` frontier in
  `progression/discovered/selection.rs`.
- **Rename `WalRequiresCheckpoint`** to what it now means: the WAL does not
  retain the canonical origin.
- **At generation zero, policy identity and the retention window come from the
  recovery configuration.** Every frame binding is checked against it, as
  ordinary open does.
- **Open these gates:**
  - tail frontier;
  - physics source selection;
  - the phase-4 admitted basis;
  - binding sampling;
  - every non-optional checkpoint consumer: walk start, redo projection basis,
    source-copy evidence, cleanup basis, and Store rejoin `root_checkpoint`.
- **The ordered-history walk keeps its current shape.** Make its start accept
  the empty basis, and list anything the linear-walk rework must revisit.

Bug classes:

- **Generation zero admitted when the checkpoint is damaged.** The enum kills
  this: damage cannot become absence.
- **The origin spelled differently at different sites.** The single constant
  kills this.

Acceptance:

- **Record and blob round trips** in `production_entry`:
  1. Initialize, write a record, crash, and recover in a fresh process.
  2. Read the record back exactly.
  3. Recover again, checkpoint, and reopen.
  4. Repeat steps 1 to 3 with a two-chunk blob publish in place of the record.
- **Empty store:** initialize, crash with nothing written, and reopen clean.
- **Still denied with no checkpoint.** Each case has a test that fails if its
  denial is removed:
  - a WAL that does not start at segment 1, generation 1, at the origin;
  - a root with a tier-epoch anchor or a free-space tier-epoch start;
  - a compaction product;
  - a retained released drop;
  - a present but rejected `checkpoint.current`.
- **Green suites:** physics lib, runtime lib, Store lib, Phase 8 and the C.11
  crash suites.

### 2. The facade: one entry, owned ports

Today there are three open paths. Two are `initialize_record_store` and
`open_record_store` (`media_ownership/runtime.rs`). The third is
`WorthStoreRecovery::recover` in the recovery crate, which exports about 80
public types. Part II gets one entry and a narrow set of ports.

The slice starts with a design note of under 80 lines, reviewed before any code.
The note decides:

- **The facade's home.** By default it is a new facade crate above
  `worth-store` and `worth-store-recovery-runtime` that exports
  `WorthStorePhysicalRuntime` and the ports (RI:359). It is a dependency seam,
  so it earns its own crate.
- **The entry.** `WorthStorePhysicalRuntime::open(configuration)` either
  initializes an empty location, or recovers and rejoins an existing one. It
  returns Serving or a typed refusal. The three current paths become internal
  to the entry.
- **The ports.** Every handle is owned and `'static`, not borrowed from the
  runtime: today `PhysicalBlobFacade<'runtime>` and `PhysicalLayoutAccess<'_>`
  borrow it, which blocks handoff between tasks. The note states `Send` and
  `Sync` for each handle. The ports are:
  - **reads:** bounded point, range and streaming record reads;
  - **submission:** carries an identity, an incarnation fence, an exact scope
    and a durability request, and consumes a capacity reservation;
  - **capacity:** a pre-effect reservation and pressure evidence;
  - **fate:** the post-reopen fate of an operation, looked up by its
    idempotency identity;
  - **negotiation:** over the rows in slice 3;
  - **lifecycle:** drain, then close.
- **Fate arms.** Live fate is `NoEffect | Completed | Indeterminate`
  (PF Decision Lock 19). After reopen, fate is either completed, proven no
  effect, or outside the retention window. Today's `Unresolved` arm
  (`closeout/operation_fates/fact.rs`) must map to one of these or be shown
  unreachable, and the note says whether `Indeterminate` can survive a reopen.
- **Durable-state discovery.** Record append takes bare bytes. Reads go by
  `PhysicalRecordId` or scan from the start. The only keyed lookup is the blob
  catalog. A fresh process needs a bounded way to find Part II's durable
  semantic frontier, so the note picks one:
  - a named anchor that is updated atomically with the submission;
  - family-tagged frames plus a bounded per-family index.

  This is a format decision, so it is made here, not later. The second option
  publishes derived-index nodes above a checkpoint, so choosing it pulls the
  linear ordered-history walk into this fast track, makes the derived-index
  row `Present`, and adds that index to slice 6's tail.
- **Part II families.** The note says whether Part II families are registered
  at open. Today's artifact-family registry is closed (C.11 lines 598-600).
- **Checkpoint cadence.** Checkpoints start only when the caller asks
  (`durability/checkpoint/runtime_owner.rs`), and the WAL tail has a limit
  (`PhysicalCheckpointPolicy`). Cadence also bounds the retry-deduplication
  window (C.7 lines 823-826). By default Store owns cadence through a
  checkpoint policy in the open configuration, and drain checkpoints before
  close. If the note keeps a checkpoint port instead, it says why.
- **Diagnostics stay out of the ports.** Internal rejoin diagnostics, such as
  `PhysicalRecoverySelectedRejoinMismatch::BoundExceeded`, either do not cross
  the facade or cross as `#[non_exhaustive]` values. Then the later
  limit-and-damage separation changes no public API.
- **No Query, Relational, branch, MVCC or semantic-writer vocabulary** in any
  exported name.

Bug classes:

- **A second open path.** The old entry points become crate-private.
- **A reservation leaked or spent twice.** The reservation is a
  `LinearResource` that the submission consumes.
- **Fate lost across reopen.** Fate is a sealed `TransitionOutcome`-shaped
  value bound to the idempotency identity.
- **A submission without a fence.** The submission type cannot be built without
  an incarnation fence.

Acceptance:

- facade compile tests, both passing and failing;
- a focused integration test for each port;
- a restart test that finds the frontier with bounded operation counts;
- the facade crate's public API recorded in its `AGENT_CONTEXT.md`.

### 3. Capability rows derived from the ports

Milestone 2 negotiates against these rows, so they are a public contract.

Today:

- `InstalledCapabilityStatus` has seven coarse rows
  (`physical_runtime/availability.rs`);
- `Recovery` is hard-coded `Absent`, even on a recovered Serving runtime, and a
  test pins it.

The slice:

- **Derive each row from the port set.** A row is `Present` exactly when a
  facade port, or the entry and its configured policy, exposes it. Records,
  recovery, WAL and checkpoint, and media are `Present`.
- **These rows are `Absent`:**
  - blob ingest and read;
  - blob release, abandoned-ingest reclaim and expiry;
  - terminal-head retirement;
  - same-tier relocation and tier movement;
  - layout rebuild and derived indexes;
  - maintenance;
  - LSM;
  - export and import.
- **One typed deferral registry.** Each `Absent` row names its owner and its
  returning milestone in this registry. The `Absent` list in the handoff
  (slice 8) is generated from it, not written twice.
- **No certification features in the facade's normal build.** Certification
  crates enable `certification-test-authority` in their normal `[dependencies]`
  (for example `worth-store-physical-certification`), and Cargo feature
  unification could pull it into a product build. A boundary-check rule fails
  if the facade's normal dependency graph enables any `certification-*`
  feature.

Bug classes:

- **A row that disagrees with what a port can reach.** Rows are computed from
  the ports.
- **A deferral with no owner.** A registry entry cannot be built without an
  owner and a milestone.

### 4. Honest observer coverage

The crash-and-reopen journey (slice 6) uses the independent integrity observer,
`physical_store_integrity_observer` in `worth-store-offline-integrity-observer`.
The observer must never report a walk as complete when it skipped a root it
does not support.

- **Status and coverage are reported together.** A root the observer does not
  support makes the status `Incomplete` and names that root's family. It never
  makes the status `Complete`.
- **Out of scope:**
  - the head readers for the newest manifest schema;
  - the observer's blob-walk rules.

  Release produces those roots, and release is `Absent`.

Bug class: completeness claimed on partial coverage. A `Complete` status can be
built only from a coverage set that names every root family present.

### 5. Concurrency posture and the facade concurrency test

Today one director serializes all record preparation behind a single mutex, and
there is one root chain. Slice 2's design note records two lists:

- what may stay shared: WAL group commit and root publication;
- what must not be shared: preparation of submissions on disjoint scope.

The test proves the posture with held work, not by counting handles:

- `pause_physical_mutation_at` holds submission A mid-flight;
- submission B, on a scope disjoint from A's, reaches `Terminal` while A is
  held;
- submission C, on A's scope, waits until A finishes;
- two read handles and two submission handles are used from different threads
  at once (RI:499).

The test uses deterministic progress points, not wall-clock timing. If disjoint
submissions serialize, splitting the preparation lock is in this slice. It is
Milestone 1's requirement.

### 6. Crash-and-reopen journey through the facade

- **One journey, run only through the facade:**
  1. real writes;
  2. a checkpoint;
  3. a WAL tail;
  4. a crash;
  5. a fresh-process reopen with exact surviving reads.

  The reads are checked against an external model of the written history and
  against the integrity observer. Run the journey twice: once crashing before
  the first checkpoint, and once after one.
- **Record crash seams through the facade.** These seams from the C.11
  crash-seam matrix reopen through the facade:
  - managed arena append data settled, root not published;
  - range release in WAL, releasing root not published;
  - evacuation copies durable, root not published;
  - arena empty in every root, file deletion partial;
  - retirement intent durable, release or deletion partial, for the evacuated
    arena only.

  The blob-ingest, publication, drop, rebuild and LSM seams return with their
  capabilities.
- **The C.7 and C.8 crash matrices** run green on the current tree.
- **A Milestone-3-shaped tail.** Commit K submissions above the checkpoint,
  where K is the default retained-tail limit, using every operation kind the
  ports expose. Recover it under the default limit profile. Operation counts at
  K and at 2K must scale linearly. If they do not, or a record-only tail reaches
  the ordered-history walk, the linear-walk rework moves into this slice.

### 7. Wrong-path fence

- **Dependency checks** (boundary-check):
  - ordinary application composition reaches only the facade crate;
  - physical owners depend in the admitted direction;
  - the legacy `worth-store-branch-deltas` and `worth-store-live-query` crates
    and the Store-internal blob, maintenance and layout facades are
    unreachable from the facade's public surface.
- **Delete or quarantine the substitutes PF C.13 names.** Each one is deleted
  or moved behind `certification-test-authority` or `cfg(test)`:
  - heap runtimes;
  - replay-based reopen;
  - duplicate backends;
  - fake physical fixtures and fake harness receipts;
  - obsolete certification paths;
  - compatibility-only owners;
  - dead placement-observation sessions;
  - static inventory rows.

  This includes deciding whether `physical_store_offline_observer` in
  `worth-store-offline-verifier` duplicates the integrity observer.
- **Replace the fixture-named bounded profiles in the production recover
  binary** with one operator memory-limit argument. Tests pass the limits they
  need.

### 8. Owner regressions, sealed handoff and caller guide

- **Map the 11 C.13 owner regressions** (PF C.13) to existing tests, and add any
  that are missing:
  - acknowledgment inversion;
  - live-state reuse;
  - checksum bypass;
  - generation bypass;
  - reclaim with a live lease;
  - scheduler bypass;
  - broad scan;
  - full materialization;
  - derived-authority promotion;
  - whole-Store submission serialization;
  - branch-label admission.

  Each test must fail if its guard is removed.
- **`RuntimeIntegrationPhysicalHandoff`.** It is constructed privately, and only
  from the completed open progression. Its payload exposes the slice 2 ports
  and the `Absent` list generated from slice 3's registry. Compile-fail tests
  forbid public construction.
- **Lower-contract compilation.** An adapter-facing test crate compiles against
  the lower contracts the runtime-integration roadmap requires, with no
  compatibility report and no second readiness token.
- **Caller guide:** `plans/worth-store/physical-runtime-facade.md`. It covers
  open, the ports, fates, discovery after restart, negotiation, drain and
  close, and the `Absent` list, with compiled examples.

## Closeout

Part II Milestone 1 may start when:

- slices 1 to 8 are committed and green;
- the PD Runtime Integration Entry Gate holds as amended for this fast track;
- the Aspect-Native Workspace Gate is confirmed closed. That gate is outside
  Store's slices; if it is open, report it as the remaining entry blocker.

## Deferred Work

Each item has no facade port and reports `Absent`, or it is internal and
changes no public API or format. Each returns before its first consumer.

| Item | Owner | Returns before |
| --- | --- | --- |
| Blob ingest and read ports | Store blob facade | Milestone 12 |
| Abandoned-ingest reclaim and ingest expiry ports | Store blob custody | Milestone 12 |
| Same-tier relocation and maintenance ports | Store maintenance | Milestone 6, which owns residency, cleanup and drainage (RI:861) |
| Layout rebuild and derived-index ports, unless slice 2 picks indexed discovery | Store layout | Milestone 4 |
| Linear ordered-history walk. Today's step loop rescans every redo member at each step. Blob publishes, abandoned reclaim, relocation and segment rewrites trigger it. | Recovery runtime | The first milestone that gets a blob, maintenance, relocation or derived-index port (Milestone 4 at the latest), or sooner if slice 6's tail measurement fails |
| Physics refusals that carry no count; the directory reader's catch-all unread arm | Recovery physics and runtime | With the linear walk |
| Limit-versus-damage separation for selected-rejoin bound refusals (about 300 sites) | Recovery runtime | Milestone 8 |
| Release and pending-WAL rejoin rework, with a typed rejoined result; source copy that returns an untyped error; sharing work across release passes | Store rejoin | Milestone 15 |
| Production terminal-head retirement after release | Store blob custody | Milestone 15 |
| Remaining release custody internals; a process-level open of a store that holds an old-version certificate | Store blob custody | Milestone 15 |
| Observer head readers for the newest manifest schema, and its blob-walk rules | Integrity observer | Milestone 12 |
| Tier movement and scheduled interference (C.11 6.5) | Store blob placement | Milestone 15 |
| Release, drop and blob-ingest crash seams; release operator docs (C.11 6.6) | Store | Milestone 12 for ingest seams, Milestone 15 for release seams |
| Release-limit sweeps (`#[ignore = "release-limit-sweeps: ..."]` in `production_entry`); worlds for blob expiry, abandoned reclaim and historical rewrite | Recovery runtime | Milestone 15 |
| Pruning replaced release-control chains (C.11 lines 1338-1345; no record-chain pruner exists) | Store blob custody | Milestone 15 |
| Per-read path re-walk, about 17 metadata operations per read (a per-owner handle table) | Store media | Milestone 4, which first relies on larger-than-memory reads (RI:728, RI:758) |
| C.11 Phase 8 4 GiB heavy lane | Store | Milestone 4 |
| C.11 Phase 8 full matrix | Store | S.12, before Milestone 14 |
| C.11 Phase 7 LSM strategy and compaction. Until then Milestone 4 reports a persistent-index requirement as unavailable (RI:742). | Store LSM | Milestone 9 |
| C.11 Phase 7 export and import | Store | Milestone 11 (RI:1238, RI:1261) |
| C.12 formal protocol rebinding | Formal models | Milestone 19 |
| C.13 joined hostile campaign | Store | S.12, before Milestone 14 |
