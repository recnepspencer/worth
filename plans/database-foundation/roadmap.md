# Database Foundation Roadmap

## Purpose

WORTH needs a working database. That means:

- every branch's committed truth is durable in Store;
- data can be larger than memory, and cold reads fault pages in on demand;
- a restart is bounded by the last checkpoint and never replays history.

This roadmap is the path to that database. It is the single plan to follow.

It supersedes three parts of the
[Physical Foundation Reconstruction Roadmap](../worth-store/physical-foundation-reconstruction-roadmap.md):

- C.11, layout, index and native blob adoption;
- C.12, formal protocol rebinding;
- C.13, the physical facade and runtime-integration entry, including the fast
  track written for it.

C.1 through C.10 stay closed. Every C.11, C.12 and C.13 item this roadmap does
not take is listed under [Deferred Work](#deferred-work) with an owner and a
return point.

Milestones are numbered D.1, D.2, and so on. Each has a plain name. A
milestone's detailed design note lives in this directory, is written when the
milestone starts, and is reviewed before any code.

## Why The Path Changed

The fast track built a facade for appending opaque records and finding them
again through named anchors. That makes a record store, not a database for
branches. What the code has today:

- **Store has a real paged copy-on-write B+tree,** in
  `worth-store-physical-format/src/btree_node/` and
  `worth-store/src/physical_runtime/layout/`. Its limits:
  - only two derived families use it, BlobCatalog and DedupeIndex;
  - keys are fixed width;
  - it can insert but not delete;
  - the height limit of 5 is spelled in three places;
  - there is one root line;
  - tree reads go straight to the record reader, not through the buffer pool.
- **Relational keeps committed truth in memory,** in persistent structures:
  - a radix trie of partitions;
  - structure-of-arrays record arenas;
  - path-copying maps.

  Its local-file durability rebuilds by replaying envelopes. About 63 files
  read through `get_partition` and assume a resident `&PartitionState`.
- **Runtime World keeps all its state in memory:** the branch registry, the
  composite history and custody.
- **C.11's release, retirement and tier-movement work assumes one root line.**
  Finishing it before Store has many roots would mean building it twice.

So the order flips. Store gains authoritative trees and a root for each branch.
Relational moves its committed truth into those trees. The C.11 remainder then
returns, designed against many roots.

## End State

The roadmap is done when all of these hold:

- **Each branch is a named root** in Store's root table:
  - a fork shares its parent's root and copies no pages;
  - a publish compare-and-swaps the branch's root inside WAL group commit.
- **Relational's committed truth lives in authoritative tree families:**
  records, versions, adjacency, a kind index and aspects.
- **What is being worked on stays warm in native form;** everything else is
  cold and read through bounded cursors that fault pages through the buffer
  pool (see [Warm And Cold Data](#warm-and-cold-data)).
- **A restart reads the checkpointed root table** and redoes only the WAL tail
  above the checkpoint.
- **Runtime World and Query host state is durable** in the same store:
  - product branches, composite history and custody;
  - installed program revisions;
  - workflow definitions and running instances;
  - idempotency records;
  - pending aftermath.
- **There is one durability path.** Relational's local-file durability is
  deleted.
- **Applications open their home through one call,** and reopening resumes
  everything (see [Public Surface](#public-surface)).

## Public Surface

The database is invisible to application authors. Branches, transactions,
reads, migration, long-running work and recovery already have graph-level
names:

- product branches;
- mutations, with their outcomes and idempotency keys;
- declared queries;
- program adoption;
- workflows;
- aftermath.

The database adds one concept: **where the application lives.**

```rust
let app = application_installation::program(validated, declaration, contributions)
    .roster(successors)                  // optional
    .home(ApplicationHome::at(path))     // or ApplicationHome::memory()
    .open()?;                            // a new home starts empty; an existing one resumes
```

- **One open call; the home is a value.** The `in_memory*`,
  `*_with_authorization_time_source` and `*_from_checkpoint` constructors
  collapse into this builder. Reopening a home resumes branches, program
  revisions, running workflows and pending aftermath. Restoring from a
  checkpoint is not an application concept.
- **`ApplicationHome::memory()` runs the same stack on RAM-backed media.**
  There is one code path, and every test exercises real durability logic.
- **No new outcomes.** With a durable home, `Committed` means durable.
  `Commit(Indeterminate)` already exists, and is resolved after a reopen by
  idempotency key through `provisional_aftermath`.
- **Declarations never mention storage.** `worth-query-decl` has no index,
  table, blob or cache vocabulary. Layout follows from meaning:
  - entity kinds give the kind index;
  - relations give adjacency;
  - versions give history;
  - field types decide chunking;
  - containment decides locality.

  If a declared query cannot meet its work ceiling, the host says so in a
  diagnostic.
- **Operators are a separate audience.** Location, memory budget, verify,
  backup and restore, and repair live on the home handle and an operator
  command, never in application code. The memory budget joins the `runtime`
  resource profiles. S.10 and S.11 surface here.
- **Everything below is internal:** the Store facade (D.5) and Relational's
  storage port (D.7) are listed as internal crates in `docs/api.md` §7.

## Warm And Cold Data

Some data must live in memory in native form; a CAD model's vertices being
edited cannot sit behind a cursor. Old history must not take memory. Residency
and representation are separate questions, and the design answers both.

- **Durable truth always lives in Store's trees.**
- **The warm working set is Relational's in-memory representation**
  (structure-of-arrays record arenas and paged columns). It is a materialized
  copy of the partitions being worked on: derived, never truth, and always
  rebuildable from Store.
- **Cold data is read through cursors and never materialized.** That covers
  history, and partitions nobody is attending to.
- **A commit writes the tree batch and applies the same journal to the warm
  copy** when it is resident. One journal drives both, so they cannot drift. A
  seeded differential test compares warm reads with cold reads.
- **Warmth follows attention,** not cache hints:
  - live reads and output demand keep their partitions warm;
  - the current head of open branches is warm, and history is cold;
  - reading an old version streams it and does not promote it;
  - under the memory budget, the least recently demanded partitions are evicted
    first.
- **The unit of warmth is a whole partition, with a size cap.** Column-page
  warmth stays possible later with no format change.
- **Locality follows meaning.** An entity lives in its owner's partition
  through a containment relation, so a part's geometry warms as one unit.
- **Bulk values are stored as column chunks per partition,** not one key per
  item. Warming is close to a copy. History keys are stored apart from the
  current head, so warming a head does not read its history.

## Standing Rules

- **Ownership:**
  - Store owns bytes, pages, trees and roots, and knows nothing of their
    meaning.
  - Relational owns meaning and defines its own fallible storage port in its
    own vocabulary.
  - A binding crate implements that port over the Store facade.
  - Store never imports Relational or Query. Relational never imports Store.
    Query core never imports Store (AGENTS.md).
- **No temporary backend.** Every deferral can be added later with no format
  or public API change. A deferral is lawful only when all three of these hold:
  - no facade port reaches it;
  - its capability reports `Absent`;
  - deferring it changes no format or law a later milestone binds to.
- **No historical compatibility.** Store is undeployed and development data is
  disposable. Format versions bump and older stores are refused.
- **Pre-plan the bug classes.** Each milestone lists the classes it could
  produce and the contract that kills each recurring one. Use `worth-proof`
  contracts (sealed authority, `Performed`, `LinearResource`,
  `TransitionOutcome`). Do not add a type that guards a single site.
- **One canonical spelling.** A milestone that adds a path removes or hides the
  one it replaces in the same milestone.
- **Larger-than-memory is proven by exact counts:** pages faulted, the resident
  bound, and work units. Never by wall-clock time.
- **Process:**
  - work in numbered slices (D.2.1, D.2.2, and so on);
  - one implementer and one fresh independent reviewer per slice;
  - the reviewer's APPROVE follow-ups are fixed in the same slice;
  - commit when green;
  - scope cargo to the touched crates;
  - keep every iteration test step under 5 minutes, and run heavy lanes once
    per slice. `production_entry` (325 s) and Phase 8 (about 350 s) are over
    that limit today, and D.2's first slice splits them into focused groups.

## Milestones

```text
D.1  Recovery before the first checkpoint
D.2  Authoritative tree families
D.3  Cold reads through the buffer pool
D.4  Branch roots
D.5  Physical runtime facade
D.6  Facade hardening
D.7  Two-tier Relational storage port
D.8  Relational records in Store trees
D.9  Branch publication on Store
D.10 Durable runtime and the application home
D.11 Space reclamation across branches
```

**Order:**

- D.1 through D.6 are Store work.
- D.7 has no Store dependency. It can move earlier if that helps; it must land
  before D.8.
- D.8 needs D.5 and D.7.
- **D.10 closeout is the working database:** every branch is tracked durably,
  and the data can be larger than memory.
- D.11 must close before any long-running use, because until then the pages of
  authoritative families are never reclaimed.

### D.1 Recovery before the first checkpoint

In progress. Today, initializing a store, writing, and crashing before the
first checkpoint gives `BLOCKED kind=WalInventory`. Recovery must admit the
generation-zero basis, the state of a store before any checkpoint.

**Design:**

- **Checkpoint presence is three-way:** `Present | Absent | Unreadable(damage)`.
  Only `Absent` admits generation zero.
- **One named WAL origin constant,** `WAL_ORIGIN` in `worth-store-wal`,
  replaces every `GENESIS.get() + 1` site and the raw `0` frontier.
- **At generation zero,** policy identity and the retention window come from
  the recovery configuration.
- **Every checkpoint consumer accepts the empty basis.**

**Bug classes:**

- *Generation zero admitted on a damaged checkpoint.* The three-way enum kills
  this.
- *The origin spelled two ways.* The one constant kills this.

**Acceptance:**

- **Round trips** in `production_entry`, for a record and for a two-chunk blob:
  1. initialize, write and crash, then recover in a fresh process;
  2. read back exactly;
  3. checkpoint and reopen.
- **An empty store** reopens clean.
- **Still denied, each case with a test that fails if its denial is removed:**
  - a WAL that does not start at the origin;
  - tier-epoch state;
  - a compaction product;
  - a retained released drop;
  - a present but rejected `checkpoint.current`;
  - extent-copy or manifest-residue cleanup frames.
- **Green suites:** physics, runtime, Store, Phase 8 and the C.11 crash suites.

### D.2 Authoritative tree families

Turn the derived-only B+tree into Store's general tree for authoritative data.

**Scope:**

- **First slice: fast iteration.** Split `production_entry` and Phase 8 into
  focused groups that each run in under 5 minutes.
- **Family classes.** The family registry distinguishes `Authoritative` from
  `Derived` families. Families are declared at open with a bounded count and a
  layout class:
  - `BTree` now;
  - an LSM class is reserved for later, invisible to callers.
- **Keys are byte strings** up to a declared cap and compare bytewise. The
  caller owns the order-preserving encoding.
- **Values have variable length.** The design note decides how values above
  the inline limit are stored, including column chunks large enough for bulk
  geometry (see [Warm And Cold Data](#warm-and-cold-data)). This is a format
  decision, so it is made here.
- **Delete, with underflow handling.**
- **Node splits by byte occupancy.**
- **One derived height bound,** computed from page size, key cap and minimum
  fanout. It replaces the three height-5 constants.
- **Sorted batch mutation.** One batch of puts and deletes on a root produces
  one new root, and path-copies each shared node once per batch, not once per
  key.
- BlobCatalog and DedupeIndex move onto the general tree with no behavior
  change.

**Bug classes:**

- *Oversized key admitted.* The bounded `TreeKey` constructor is the only way to
  make a key.
- *Height bound drift.* There is one derivation.
- *A derived family written as authority.* The family class is in the type, and
  authoritative writes need `worth-proof` authority.
- *An unsorted or duplicate batch.* The sorted batch type cannot hold one.

**Acceptance:**

- **A seeded differential oracle** against an in-memory ordered map, over
  random batches with deletes, splits and merges.
- **Exact page-write counts** per batch.
- **The height bound holds** at the maximum key.
- **Derived-family suites stay green.**

### D.3 Cold reads through the buffer pool

**Scope:**

- **Tree page reads go through the bounded fault owner,**
  `PhysicalBoundedFrameFaultOwner`: a miss faults the page in, and eviction
  holds the resident bound.
- **A per-owner handle table** replaces today's per-read path re-walk, which
  costs about 17 metadata operations per read.
- **Bounded cursors for point, range and resumable scans:**
  - a cursor pins at most one page per tree level;
  - every step is charged against a work budget;
  - a cursor resumes from a key, not a position.

**Bug classes:**

- *A tree read that bypasses the pool.* Tree code reaches pages through one
  page-access port, and the direct reader is not visible to it.
- *A leaked pin.* A pin is a guard.
- *An unbounded scan.* Every cursor step is charged.

**Acceptance:**

- A tree ten times the pool's resident limit answers point and range reads
  exactly.
- Fault counts are exact, and the resident bound is never exceeded.
- A cold reopen faults exactly one page per level for a point read.

### D.4 Branch roots

**Scope:**

- **A root table.** The design note picks its shape. The default is a tree
  keyed by root name, holding the root page, the generation and the family set,
  reached from one manifest slot.
- **Operations:**
  - create an empty root;
  - fork: a new name points at an existing root;
  - publish: compare-and-swap on the expected generation;
  - drop.
- **WAL and recovery:**
  - root advances are WAL frames inside group commit, and redo rebuilds the
    table;
  - checkpoints capture the table;
  - recovery discovers roots through it.
- **No reclamation until D.11.** Pages of authoritative families are never
  reclaimed before D.11. Today's retirement gate compares one root generation.
  It is fenced so it never touches an authoritative family.

**Bug classes:**

- *A lost update.* Publish needs a sealed expected-generation token.
- *A root advance outside the WAL.* An advance can only be built inside group
  commit.
- *A fork that copies.* A test counts zero page writes per fork.
- *A shared node reclaimed.* The family-class guard on retirement kills this.

**Acceptance:**

- **A many-branch world:** fork, publish on both sides, crash at every
  progression edge, and restart to the exact root table.
- **Root lookup** costs at most one page per tree level.

### D.5 Physical runtime facade

One entry and owned ports for everything above Store.

**The first slice designs top-down.** It writes the public `open` call from
[Public Surface](#public-surface) and lists exactly what reopening must resume.
The internal ports are then shaped by that list, not the other way around. The reviewed design in
[d5-physical-runtime-facade.md](d5-physical-runtime-facade.md) still governs:

- the entry;
- the `facade-owner` gate;
- the fence;
- fate and token identity;
- capacity;
- the checkpoint cadence and retry window.

Its named-anchor discovery is replaced by D.4's root table, and its record ports
by a tree port. The note is revised as the first slice of D.5.

**Ports:**

- **Tree reads:** point, range and cursor reads on a root snapshot lease.
- **Tree writes:**
  - a fenced batch on a named root with an expected generation and a durability
    request;
  - fork;
  - drop.
- **Capacity:** a pre-effect reservation and pressure evidence.
- **Fate:** exact fate after a reopen, looked up by a token the caller
  persists.
- **Capabilities and lifecycle.** Each capability row is derived from the port
  set, and every `Absent` row names its owner and return point in one typed
  deferral registry.

**Observer coverage.** The integrity observer reports `Incomplete` and names the
family whenever it skips a root family. It never reports `Complete` for a
partial walk.

**Bug classes:**

- a second open path;
- a reservation leaked or spent twice;
- fate lost across reopen;
- an unfenced write;
- a token identity minted twice;
- a capability row that disagrees with the ports;
- completeness claimed on partial coverage.

The note names the contract that kills each one.

**Acceptance:**

- compile tests, passing and failing;
- an integration test for each port;
- a restart that finds every branch root with bounded counts.

### D.6 Facade hardening

**Concurrency.** Shared owners are:

- WAL group commit;
- root publication;
- the checkpoint worker;
- short allocation and registry locks, never held across a pause point or I/O.

Writes to different roots prepare concurrently, and writes to one root
serialize. A deterministic test holds one write at a progress point while a
write to another root reaches `Terminal`.

**Crash journey through the facade only:**

1. many branches;
2. a checkpoint;
3. a WAL tail;
4. a crash;
5. a fresh-process reopen with exact reads.

The reads are checked against an external model and the integrity observer.
Run the journey crashing before and after the first checkpoint. The C.7 and C.8
crash matrices stay green.

**Wrong-path fence:**

- Application composition reaches only the facade crate.
- These substitutes, named in PF C.13, are deleted or moved behind
  `certification-test-authority`:
  - heap runtimes;
  - replay-based reopen;
  - duplicate backends;
  - fake fixtures;
  - obsolete certification paths.
- No certification feature reaches the facade's normal build.

**Handoff:**

- Map the 11 PF C.13 owner regressions to tests that fail when their guard is
  removed.
- Add a sealed `RuntimeIntegrationPhysicalHandoff`.
- Write a caller guide with compiled examples.

### D.7 Two-tier Relational storage port

Relational's read side assumes resident memory. This milestone puts every read
behind a port in Relational's own vocabulary, with the two tiers from
[Warm And Cold Data](#warm-and-cold-data). It has no Store dependency.

- **Warm tier.** Hot code keeps reading in-memory partitions through a
  residency guard. Taking the guard is the fallible, bounded step: it warms the
  partition, or denies.
- **Cold tier.** Point, range and history reads go through bounded cursors and
  never materialize a partition.

Most read sites change only how they obtain a partition, not how they read it.

**Scope:**

- **Amend the Physical Database Roadmap.** S.2 "Must Preserve" says Store does
  not replace Relational's in-memory arenas. It now says Store holds
  Relational's committed truth through Relational's storage port.
- **Define the port:**
  - residency guards over partitions, with a size cap per partition;
  - cursor reads over records, versions, adjacency and kinds;
  - views that are owned or guarded, never borrowed past their guard;
  - a typed storage denial that callers carry, not panic on.
- **First backend: today's in-memory representation,** where every partition
  is resident.
- **Convert every `get_partition` site** to a guard or a cursor.
  `get_partition` becomes private to the backend.
- **Decide when a site needs a guard and when it needs a cursor.** History and
  wide scans use cursors.

**Bug classes:**

- *A read that bypasses the port.* `get_partition` is not visible outside the
  backend.
- *An infallible assumption.* Guard acquisition and cursor reads return
  `Result`.
- *A cold read that materializes a partition.* The cursor API has no path to a
  partition.
- *Work-count drift.* The exact cost certification tests stay unchanged and
  green.

**Acceptance:**

- The Relational suites and certification cost tests are green, with no changed
  counts.
- A fault-injecting backend proves that every caller carries a storage denial.

### D.8 Relational records in Store trees

**Scope:**

- **The key layout.** The design note decides:
  - the current head as column chunks per partition, so warming is close to a
    copy;
  - versions stored apart from the head, so history is a range scan and
    warming never reads it;
  - adjacency in both directions, keyed by endpoint, kind and other endpoint;
  - a real kind index;
  - where aspects live;
  - whether `IndexingState` stays derived as rebuildable tree families or
    becomes authoritative.
- **Locality follows containment.** An entity is placed in its owner's
  partition. Today the caller of each mutation intent chooses the partition;
  the design note replaces that with the containment rule, and decides where
  it is declared.
- **A binding crate** implements the D.7 port over the D.5 tree port. Every
  family has one encoder.
- **Warming and eviction.** A partition warms from its column chunks and is
  evicted by dropping the warm copy. Commits apply the same journal to both
  tiers.
- **Kind scans use the index.** The linear kind walk is deleted.

**Bug classes:**

- *A key encoding that does not preserve order.* A property test compares
  encoded order with decoded order for each encoder.
- *An index that drifts from its records.* Index entries are derived in the
  same batch, from one derivation.
- *The warm copy drifts from durable truth.* One journal drives both tiers,
  and a seeded test compares warm reads with cold reads after random commits,
  evictions and rewarms.

**Acceptance:**

- The Relational suites run against both backends, with identical results.
- Work counts stay within declared bounds.
- Cold reads are served after a restart.
- Warming a partition costs reads proportional to its head, never to its
  history.

### D.9 Branch publication on Store

**Scope:**

- Prepare turns the slot-exact journal into a tree batch on the parent root.
- Publish is a root compare-and-swap through the facade.
- Settlement uses the fate port.
- Branch reference cells are named roots.
- The pending-settlement record is written in the same batch as the head move.
- `PersistedSegmentedLocalFs` and envelope replay are deleted.

**Bug classes:**

- *A head moved without a settlement record.* One constructor builds both.
- *Two durability paths.* The old path is deleted in this milestone.

**Acceptance:**

- A crash at every progression edge (prepare, publish, settle) recovers the
  exact head and settlement state.
- `SettlementDeferred` survives a restart.

### D.10 Durable runtime and the application home

**Scope:**

- **World state moves into tree families** behind a World storage port, in the
  same pattern as D.7: the branch registry, the composite history catalog and
  custody.
- **Query host state moves too:**
  - installed program revisions per branch;
  - workflow definitions and running instances;
  - idempotency records;
  - pending aftermath.
- **The application home ships** as described in
  [Public Surface](#public-surface):
  - one `open` call;
  - `ApplicationHome::at(path)` and `ApplicationHome::memory()`, which runs on
    RAM-backed media;
  - the old `in_memory*` and `*_from_checkpoint` constructors are deleted.
- **Residency follows demand.** Live reads and output demand keep partitions
  warm, and the memory budget joins the `runtime` resource profiles.

**Acceptance:**

- **A larger-than-memory world:**
  - the data set is several times the resident limit;
  - fault counts are exact;
  - the restart is bounded.
- **The working set stays warm.** A demanded partition is served without
  faults while cold history is read alongside it.
- **Reopen resumes everything:** a workflow mid-run, a pending aftermath and an
  indeterminate commit resolved by idempotency key.
- **A crash matrix** at every World, Query host and Relational progression edge.
- **Parity** with today's in-memory runtime on the World and Query host suites.

Closing D.10 delivers the working database.

### D.11 Space reclamation across branches

**Scope:**

- **Reachability marking** runs from every live root and from the roots that
  retained checkpoints still need.
- **Unreachable node pages are reclaimed** through the existing range-release
  WAL frames.
- **C.11's arena retirement is redesigned** for many roots.

The design note confirms that no format change is needed: nodes are
self-describing pages, and allocation is already tracked by the arena and
extent manifests.

**Bug class:** *a reachable page reclaimed.* Reclamation consumes a mark proof
taken at a published generation set.

**Acceptance:**

- A seeded differential test runs fork, publish and drop with reclamation on and
  off. Reads are identical, and space is returned.
- The crash matrix covers each reclamation edge.

## Relationship To Other Roadmaps

- **Physical Foundation Reconstruction Roadmap:**
  - C.1 to C.10 are closed;
  - C.11, C.12 and C.13 are superseded here;
  - their specs stay as history and design reference, not as plans.
- **Physical Database Roadmap:**
  - S.10, S.11 and S.12 keep their positions before their consumers;
  - its Runtime Integration Entry Gate is replaced by D.10 closeout.
- **Runtime Integration Roadmap** (`runtime-integration-roadmap.md`):
  - it predates this data model;
  - its milestones are reconciled with D.1 to D.11 before Part II work resumes;
  - "Milestone N" in the table below refers to its current numbering.

## Deferred Work

Each item has no facade port and reports `Absent`, or it is internal and
changes no format or public API.

| Item | Owner | Returns |
| --- | --- | --- |
| Blob ingest and read ports | Store blob facade | Before the first consumer of large values (Milestone 12) |
| Abandoned-ingest reclaim and ingest expiry ports | Store blob custody | With the blob ports |
| Release, terminal-head retirement and tier movement (C.11 Phase 6 remainder), redesigned for many roots | Store blob custody | After D.11, before Milestone 15 |
| Release and pending-WAL rejoin rework; release-limit sweeps; pruning replaced release-control chains | Store rejoin | With the release work |
| Same-tier relocation and maintenance ports | Store maintenance | Milestone 6 |
| Linear ordered-history walk; physics refusals with no count | Recovery runtime | Before the first blob, maintenance or relocation port |
| Limit-versus-damage separation for selected-rejoin bound refusals | Recovery runtime | S.10 |
| D.1 follow-ups: manifest-residue Intent replay, covered-frame guard tests, checkpoint-sequence-0 identities | Recovery runtime | With the linear walk |
| Observer head readers for the newest manifest schema, and blob-walk rules | Integrity observer | With the blob ports |
| Column-page warmth, for partitions above the size cap | Relational storage port | When a real workload needs a partition larger than the cap |
| LSM layout class and compaction (C.11 Phase 7) | Store LSM | Before the first write-heavy family needs it (Milestone 9) |
| Export and import (C.11 Phase 7) | Store | Milestone 11 |
| C.11 Phase 8 heavy lane and full matrix | Store | S.12 |
| C.12 formal protocol rebinding | Formal models | Milestone 19 |
| C.13 joined hostile campaign | Store | S.12 |
| S.10 bootstrap, recovery, PITR, backup and repair | Store | Milestone 8 |
| S.11 security scope and key lifecycle | Store | Milestone 10 |
| S.12 bulk ingest and joined certification | Store | Milestone 14 |
