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
branches.

**Store has a real copy-on-write B+tree,** in
`worth-store-physical-format/src/btree_node/` and
`worth-store/src/physical_runtime/layout/`. Its reads already go through the
buffer pool (`BoundedFrameLoader`). Its limits:

- **Only two derived families use it:** BlobCatalog and DedupeIndex.
- **Keys are fixed width.**
- **It can insert but not delete.**
- **The height limit of 5 is spelled in three places.**
- **Nodes are inline records,** one durable append each. Retirement works per
  segment, extent or arena, so a dead node cannot be freed alone.
- **Nodes carry sibling links,** which cannot be shared across forks.
- **Every node append advances the one store root generation,** so all writes
  serialize on one root line.

**Relational keeps committed truth in memory,** in persistent structures:

- a radix trie of partitions;
- structure-of-arrays record arenas;
- path-copying maps.

Its record arenas hold version history alongside the head. Root regions hold
`Arc<PartitionState>`. Nearly all application data sits in
`PartitionId::main()`, because Query creates entities there. Its local-file
durability rebuilds by replaying envelopes.

**Runtime World and the Query host keep all their state in memory.**

**C.11's release, retirement and tier-movement work assumes one root line.**
Finishing it before Store has many roots would mean building it twice.

So the order flips:

1. Applications move to the final open call now, backed by today's runtimes.
2. Store gains page-granular authoritative trees and a root for each branch.
3. Relational moves its committed truth into those trees.
4. The C.11 remainder then returns, designed against many roots.

## End State

The roadmap is done when all of these hold:

- **Each branch is a named root** in Store's root table:
  - a fork shares its parent's root and copies no pages;
  - a publish compare-and-swaps the branch's root inside WAL group commit;
  - several roots can publish atomically in one group.
- **Relational's committed truth lives in authoritative tree families:**
  records, versions, adjacency, a kind index, aspects, and the unique-field
  index.
- **What is being worked on stays warm in native form.** Everything else is
  cold and read through bounded cursors (see
  [Warm And Cold Data](#warm-and-cold-data)).
- **A restart reads the checkpointed root table** and redoes only the WAL tail
  above the checkpoint.
- **Every item on the reopen inventory is durable** in the same store (see
  [D.2](#d2-application-home)).
- **There is one durability path.** Relational's local-file durability is
  deleted.
- **Applications open their home through one call,** and reopening resumes
  everything on the inventory (see [Public Surface](#public-surface)).

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

The database adds one concept: **where the application lives.** The call below
is a sketch; [the D.2 note](d2-application-home.md) gives the real one.

```rust
let app = application_installation::program(validated, declaration, contributions)
    .roster(successors)                  // optional
    .open(ApplicationHome::at(path))?;   // or ApplicationHome::memory(); a new home starts
                                         // empty, an existing one resumes
```

- **One open call; the home is a value.** These collapse into one builder:
  - the `in_memory*` constructors;
  - the `*_with_authorization_time_source` constructors;
  - the `*_from_checkpoint*` constructors.

  The authorization time source becomes a builder option. Program transition
  at open, today `*_from_checkpoint_with_transition` and
  `WorthQueryCheckpointMigrationWriter`, becomes the open call's adoption step.
  Restoring from a checkpoint is not an application concept.
- **`ApplicationHome::memory()`** runs today's in-memory runtime until D.10, then
  the same stack on RAM-backed media built in D.7. Memory homes never certify
  persistence; durable certification runs on file media.
- **`ApplicationHome::at(path)`** is refused, with its capability row `Absent`,
  until D.9 puts Relational on Store. No replay-based bridge is wired in
  between.
- **Commits gain no new outcomes, and lose one.** With a durable home,
  `Committed` means durable. `WorthQueryApplicationCommitOutcome::Indeterminate`
  already exists. `ProductUnpublished` is deleted in D.10.
  After a reopen, a retry with the same idempotency key answers
  `AlreadyCommitted` or commits.
- **Reads gain one storage denial,** carried inside the existing Query denial.
  Read paths that are infallible today become fallible in D.8. This is a public
  enum change, so the bank-server gate runs.
- **Declarations never mention storage.** `worth-query-decl` has no index,
  table, blob or cache vocabulary. Layout follows from meaning:
  - entity kinds give the kind index;
  - relations give adjacency;
  - versions give history;
  - field types decide chunking;
  - placement is decided at creation (D.8).

  If a declared query cannot meet its work ceiling, the host says so in a
  diagnostic.
- **Operators are a separate audience.** Location, memory budget, verify,
  backup and restore, and repair live on the home handle and an operator
  command, never in application code. The memory budget joins the `runtime`
  resource profiles. S.10 and S.11 surface here.
- **Everything below is internal:** D.6 and D.8 add the Store facade and
  Relational's storage port to `docs/api.md` §7 as internal crates.

## Warm And Cold Data

Some data must live in memory in native form; a CAD model's vertices being
edited cannot sit behind a cursor. Old history must not take memory. Residency
and representation are separate questions, and the design answers both.

- **Durable truth always lives in Store's trees.**
- **The warm working set is a head-only native copy** of what is being worked
  on:
  - It uses structure-of-arrays columns, like today's record arenas, but holds
    no version history.
  - It is derived, never truth, and always rebuildable from Store.
  - Today's arenas mix history into the same structure. D.8 splits them.
- **Cold data is read through cursors and never materialized.** That covers
  history, and data nobody is attending to.
- **Warm updates come from the committed tree batch,** not from a second
  encoding of the journal:
  - the warm copy is stamped with the root generation it reflects;
  - a commit applies to it only when the stamps match, and otherwise the copy
    is dropped and rewarmed.
- **Warmth follows attention,** not cache hints:
  - output demand and live reads keep their data warm;
  - the current head of open branches is warm, and history is cold;
  - reading an old version streams it and does not promote it;
  - under the memory budget, the least recently demanded data is evicted first.

  Output demand names Query sources, not Relational partitions. D.10 defines
  the mapping.
- **The unit of warmth and placement is decided in D.8,** before any layout is
  written:
  - Partition identity is part of every record id, and today almost everything
    lands in `main()`, so "whole partition" alone cannot be the unit.
  - The design note decides how placement is chosen at creation (for example,
    by the containing owner), and the warm unit (a placement group, column
    pages within it, or both).
  - It also decides what happens when one group exceeds the memory budget.
- **Bulk values are stored as column chunks,** not one key per item, so warming
  is close to a copy. History keys are stored apart from the current head, so
  warming a head never reads its history.

## Standing Rules

- **Ownership:**
  - Store owns bytes, pages, trees and roots, and knows nothing of their
    meaning.
  - Relational owns meaning and defines its own fallible storage port in its
    own vocabulary.
  - A binding crate implements that port over the Store facade.
  - Store never imports Relational or Query. Relational and Query core never
    import Store.
  - D.7 adds boundary-check rules that enforce all of this.
- **No temporary backend.** A deferral is lawful only when all three of these
  hold:
  - no facade port reaches it;
  - its capability reports `Absent`;
  - deferring it changes no public API a later milestone binds to.
- **Formats can change later,** because the store is undeployed and its data is
  disposable. Format versions bump and older stores are refused. D.3 and D.4
  still reserve the hooks known to be coming, so the change is a version bump
  rather than a redesign:
  - a layout-class tag in every root descriptor (LSM);
  - a key-identity slot in the page envelope (S.11 encryption).
- **Pre-plan the bug classes.** Each milestone lists the classes it could
  produce and the contract that kills each recurring one. Use `worth-proof`
  contracts (sealed authority, `Performed`, `LinearResource`,
  `TransitionOutcome`). Do not add a type that guards a single site.
- **One canonical spelling.** A milestone that adds a path removes or hides the
  one it replaces in the same milestone.
- **Larger-than-memory is proven by exact counts:** pages faulted, the resident
  bound and work units. Never by wall-clock time.
- **Process:**
  - work in numbered slices (D.3.1, D.3.2, and so on);
  - one implementer and one fresh independent reviewer per slice;
  - the reviewer's APPROVE follow-ups are fixed in the same slice;
  - commit when green;
  - scope cargo to the touched crates;
  - keep every iteration test step under 5 minutes, and run heavy lanes once
    per slice. `production_entry` (325 s) and Phase 8 (about 350 s) are over
    that limit today; D.3's first slice splits them into focused groups.

## Milestones

```text
D.1  Recovery before the first checkpoint          done
D.2  Application home
D.3  Page-granular authoritative trees
D.4  Branch roots
D.5  Bounded cold reads
D.6  Physical runtime facade
D.7  Facade hardening and memory media
D.8  Two-tier Relational storage port
D.9  Relational truth on Store
D.10 Durable runtime state
D.11 Space reclamation across branches
```

**Order:**

- **D.2 comes first, so integration starts now.** Applications move to the final
  open call immediately, and later milestones change only what backs it.
- **D.3 to D.7 are Store work.**
- **D.8 has no Store dependency.** It can run before or between the Store
  milestones, and must land before D.9.
- **D.9 needs D.6 and D.8.**
- **D.10 closeout is the working database:** every branch is tracked durably,
  the data can be larger than memory, and reopen resumes the whole inventory.
- **D.11 must close before any long-running use,** because until then the
  pages of authoritative families are never reclaimed.

### D.1 Recovery before the first checkpoint

Done in commit `316870af71`. Recovery admits the generation-zero basis:

- **Checkpoint presence is three-way:** `Present | Absent | Unreadable(damage)`.
  Only `Absent` admits generation zero.
- **There is one WAL origin constant,** `WAL_ORIGIN` in `worth-store-wal`.
- **At generation zero,** policy identity and the retention window come from
  the recovery configuration.
- **Still denied, each with a killing test:**
  - reclaim and maintenance frames;
  - tier-epoch state;
  - compaction products;
  - retained released drops;
  - a rejected `checkpoint.current`.
- **Open follow-ups** are in [Deferred Work](#deferred-work).

### D.2 Application home

The final public open call, backed by today's runtimes. It needs no Store work.

**Scope:**

- **The reopen inventory.** List every piece of Query host, World and
  Relational state an application needs after a reopen, with its owner and
  where it becomes durable. It is the checklist for D.9 and D.10. At minimum it
  covers:
  - installed program revisions per branch;
  - workflow definitions and running instances;
  - idempotency records;
  - pending aftermath;
  - World's ProductUnpublished recovery catalog, deleted in D.10 rather than
    made durable;
  - inbound-occurrence receipts;
  - capability delegation, elevation and mandatory review;
  - query continuations;
  - output-demand interests;
  - the external-effect outbox;
  - branch registry, composite history and custody.

  Each item is either resumed or reported `Absent` by the home's capability
  rows. "Reopen resumes everything" means everything on this list.
- **The open call** from [Public Surface](#public-surface):
  - one builder replaces every installation constructor, and the old ones are
    deleted;
  - `ApplicationHome::memory()` runs today's in-memory runtime;
  - `ApplicationHome::at(path)` returns a typed refusal, and its capability row
    reports `Absent` until D.9. Relational's replay-based local-file mode is not
    wired through the host, because D.9 deletes it.
- **The examples, bank-server and docs** move to the builder in this
  milestone.
- **Reconcile the Runtime Integration Roadmap** with this one: mark which of its
  milestones this roadmap replaces, and renumber its return points.

**Bug classes:**

- *A second installation path.* The old constructors are deleted.
- *A reopen that silently drops state.* Every inventory item is resumed or
  reports `Absent`. The capability rows are derived from the inventory, and a
  test checks every row; D.9 and D.10 flip rows as items become durable.

**Acceptance:**

- Every example and bank-server installs through the builder.
- `ApplicationHome::at(path)` is refused with a typed reason.
- The home's capability rows match the inventory.

### D.3 Page-granular authoritative trees

Turn the derived-only B+tree into Store's general tree for authoritative data.

**Scope:**

- **First slice: fast iteration.** Split `production_entry` and Phase 8 into
  focused groups that each run in under 5 minutes.
- **Nodes are pages.** Each node owns a page that can be freed by itself. This
  is the format D.11's reclamation depends on.
- **Sibling links are dropped,** or kept only as hints that are never followed
  across roots.
- **Family classes.** The family registry distinguishes `Authoritative` from
  `Derived` families. Families are declared at open with a bounded count and a
  layout class:
  - `BTree` now;
  - an LSM class later, behind the reserved tag.
- **Keys are byte strings** up to a declared cap and compare bytewise. The
  caller owns the order-preserving encoding.
- **Values have variable length.** The design note decides how values above
  the inline limit are stored, including column chunks large enough for bulk
  geometry.
- **Delete, with underflow handling.**
- **Node splits by byte occupancy.**
- **One derived height bound,** computed from page size, key cap and minimum
  fanout. It replaces the three height-5 constants.
- **Batch mutation:**
  - one sorted batch of puts and deletes on a root is one multi-page durable
    mutation in one WAL group;
  - it produces one new root and path-copies each shared node once;
  - it publishes that root through the existing family root directory with
    one generation advance per batch, not one per node. D.4 brings
    per-root generations.

  This kills the C.13 "whole-Store submission serialization" regression at its
  source.
- **The page envelope reserves a key-identity slot** for S.11.
- **BlobCatalog and DedupeIndex move onto the general tree** with no behavior
  change.

**Bug classes:**

- *Oversized key admitted.* The bounded `TreeKey` constructor is the only way to
  make a key.
- *Height bound drift.* There is one derivation.
- *A derived family written as authority.* The family class is in the type, and
  authoritative writes need `worth-proof` authority.
- *An unsorted or duplicate batch.* The sorted batch type cannot hold one.
- *A batch half applied after a crash.* The batch is one WAL group.

**Acceptance:**

- **A seeded differential oracle** against an in-memory ordered map, over random
  batches with deletes, splits and merges.
- **Exact page-write counts** per batch.
- **A crash at every batch edge** recovers either the old root or the new one,
  never a mix.
- **The height bound holds** at the maximum key.
- **Derived-family suites stay green.**

### D.4 Branch roots

**Scope:**

- **A root table.** The design note picks its shape. The default is a tree
  keyed by root name, reached from one manifest slot. Each entry holds:
  - the root page;
  - the generation;
  - the family set;
  - the reserved layout-class tag.
- **Operations:**
  - create an empty root;
  - fork: a new name points at an existing root;
  - publish: compare-and-swap on the expected generation;
  - drop.
- **Atomic multi-root publish.** Several roots advance together in one WAL
  group. World's composite commits use it, which removes the ProductUnpublished
  partial-commit class at its source.
- **WAL and recovery:**
  - root advances are WAL frames inside group commit, and redo rebuilds the
    table;
  - checkpoints capture the table;
  - recovery discovers roots through it.
- **Retirement is fenced by root multiplicity.** Until D.11, only families that
  are single-line and store-global (today's derived families) retire replaced
  nodes. Any family reachable from more than one root never retires.

**Bug classes:**

- *A lost update.* Publish needs a sealed expected-generation token.
- *A root advance outside the WAL.* An advance can only be built inside group
  commit.
- *A fork that copies.* A test counts zero page writes per fork.
- *A shared node reclaimed.* The root-multiplicity fence kills this.
- *A composite commit half published.* The multi-root publish is one group.

**Acceptance:**

- **A many-branch world:** fork, publish on both sides, publish several roots
  atomically, crash at every progression edge, and restart to the exact root
  table.
- **Root lookup** costs at most one page per tree level.

### D.5 Bounded cold reads

Tree reads already go through the buffer pool. This milestone makes them
bounded and cheap.

**Scope:**

- **Remove the per-read routing re-walk.** Measure today's metadata operations
  per read in a deterministic test, then replace the walk with a per-owner
  handle table.
- **Bounded cursors for point, range and resumable scans:**
  - a cursor pins at most one page per tree level;
  - every step is charged against a work budget;
  - a cursor resumes from a key, not a position.
- **Cursors are scoped and cannot cross a pause point.** A long-lived reader
  holds a root lease and a resume key, never a pin.

**Bug classes:**

- *A leaked pin.* A pin is a guard.
- *A pin held across I/O or a pause.* The cursor type is not `Send` across a
  pause; lasting readers keep a lease and a key.
- *An unbounded scan.* Every cursor step is charged.

**Acceptance:**

- A tree ten times the pool's resident limit answers point and range reads
  exactly.
- Fault counts are exact, and the resident bound is never exceeded.
- A cold reopen faults exactly one page per level for a point read.
- Metadata operations per read are counted and bounded.

### D.6 Physical runtime facade

One entry and owned ports for everything above Store. The reviewed design in
[d6-physical-runtime-facade.md](d6-physical-runtime-facade.md) still governs:

- the entry;
- the `facade-owner` gate;
- the fence;
- fate and token identity;
- capacity;
- the checkpoint cadence and retry window.

Its named-anchor discovery is replaced by D.4's root table, and its record ports
by a tree port. The first slice revises the note against D.2's reopen inventory.

**Ports:**

- **Tree reads:** point, range and cursor reads on a root lease.
- **Tree writes:**
  - a fenced batch on a named root with an expected generation and a durability
    request;
  - atomic multi-root batches;
  - fork;
  - drop.
- **Capacity:** a pre-effect reservation and pressure evidence.
- **Fate:** exact fate after a reopen, looked up by a token the caller
  persists.
- **Capabilities and lifecycle.** Each capability row is derived from the port
  set. Every `Absent` row names its owner and return point in one typed
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

### D.7 Facade hardening and memory media

**Concurrency.** Shared owners are:

- WAL group commit;
- root publication;
- the checkpoint worker;
- short allocation and registry locks, never held across a pause point or I/O.

Batches on different roots prepare concurrently, and batches on one root
serialize. A deterministic test holds one batch at a progress point while a
batch on another root reaches `Terminal`.

**Crash journey through the facade only:**

1. many branches;
2. a checkpoint;
3. a WAL tail;
4. a crash;
5. a fresh-process reopen with exact reads.

The reads are checked against an external model and the integrity observer.
Run the journey crashing before and after the first checkpoint. The C.7 and C.8
crash matrices stay green.

**Memory media.** A RAM-backed media backend implements the same media contract
as the filesystem backend: a real backend for `ApplicationHome::memory()`, not
a test substitute. It is not used for durability certification.

**Wrong-path fence:**

- Boundary-check rules enforce the ownership rules in
  [Standing Rules](#standing-rules), and that application composition reaches
  only the facade crate.
- These substitutes, named in PF C.13, are deleted or moved behind
  `certification-test-authority`:
  - heap runtimes;
  - replay-based reopen;
  - duplicate filesystem backends;
  - fake fixtures;
  - obsolete certification paths.
- No certification feature reaches the facade's normal build.

**Handoff:**

- Map the 11 PF C.13 owner regressions to tests that fail when their guard is
  removed.
- Add a sealed `RuntimeIntegrationPhysicalHandoff`.
- Write a caller guide with compiled examples.

### D.8 Two-tier Relational storage port

Relational's read side assumes resident memory. This milestone puts every read
behind a port in Relational's own vocabulary, with the two tiers from
[Warm And Cold Data](#warm-and-cold-data). It has no Store dependency.

**Design note decisions,** made before the port shape is fixed:

- **Placement.** How an entity's placement group is chosen at creation (ids
  stay stable after that), and the warm unit.
- **What happens when a group exceeds the memory budget.**
- **Root regions hold a port-level root handle,** not an `Arc<PartitionState>`.
  D.9 binds that handle to a tree root.
- **The incremental content commitment is persisted** (in today's checkpoint
  image, beside `partition_image_digest`), so publishing never needs the
  previous partition resident.
- **A head-only warm arena.** Version history moves to the cold tier.
- **Prepare's read set.** Classify every read prepare performs:
  - `AllObserved` and `FullObservedScan` scopes;
  - unique-field checks;
  - kind scans;
  - invariant reads.

  Each class gets a work ceiling. A full-scan invariant is denied or backed by
  an index.
- **The unique-field index is authoritative,** written in the same batch as its
  records. Rebuilding it on open would be replay.

**Port:**

- residency guards over warm units;
- cursor reads over records, versions, adjacency and kinds;
- views that are owned or guarded, never borrowed past their guard;
- a typed storage denial that callers carry, not panic on.

**First backend: today's in-memory representation,** restructured as above,
where everything is resident.

**Convert every `get_partition` site** to a guard or a cursor. `get_partition`
becomes private to the backend. History and wide scans use cursors.

**Bug classes:**

- *A read that bypasses the port.* `get_partition` is not visible outside the
  backend.
- *An infallible assumption.* Guard acquisition and cursor reads return
  `Result`.
- *A cold read that materializes.* The cursor API has no path to a warm unit.
- *A guard held across a pause.* Guards are scoped, as D.5's cursors are.
- *A group that outgrows the budget.* The rule from the design note, with a
  test at the boundary.

**Acceptance:**

- The Relational suites are green.
- Certification cost tests keep their counts, except where a count is
  re-declared in the design note (publication and region capture).
- A fault-injecting backend proves that every caller carries a storage denial.

### D.9 Relational truth on Store

One milestone switches Relational's durability, so two paths never coexist.

**Scope:**

- **The key layout.** The design note decides:
  - the current head as column chunks per placement group;
  - versions stored apart from the head;
  - adjacency in both directions, keyed by endpoint, kind and other endpoint;
  - a real kind index;
  - the unique-field index;
  - where aspects live;
  - whether the rest of `IndexingState` stays derived or becomes authoritative.
- **Everything else `DurableCheckpoint` carries:**
  - the symbol table;
  - the record-identity allocator, which holds slot frontiers, reusable slots
    and pending reservations;
  - lineage;
  - aspect contracts;
  - root schema images;
  - index definitions and generations.

  Commit envelopes are replaced by the batch itself.
- **A binding crate** implements the D.8 port over the D.6 tree port. Every
  family has one encoder.
- **Publication:**
  - prepare turns the journal into a tree batch on the parent root;
  - publish is a root compare-and-swap;
  - settlement uses the fate port;
  - branch reference cells are named roots.
- **The pending-settlement record** is written in the same batch as the head
  move.
- **Warm copies** are stamped with a generation and updated from the committed
  batch.
- **`PersistedSegmentedLocalFs` and envelope replay are deleted.**
  `ApplicationHome::at(path)` opens on Store. Inventory items not yet durable
  keep reporting `Absent` until D.10.
- **Kind scans use the index.** The linear kind walk is deleted.

**Bug classes:**

- *A key encoding that does not preserve order.* A property test compares
  encoded order with decoded order for each encoder.
- *An index that drifts from its records.* Index entries are derived in the
  same batch.
- *The warm copy drifts from durable truth.* The generation stamp, plus a
  seeded test comparing warm and cold reads after random commits, evictions and
  rewarms.
- *A head moved without a settlement record.* One constructor builds both.
- *Recovered pending settlements* break the settlement registry's "bounded by
  construction" proof. The proof is restated to cover entries rebuilt at open.

**Acceptance:**

- The Relational suites run against both backends, with identical results.
- Work counts stay within declared bounds.
- A crash at every progression edge (prepare, publish, settle) recovers the
  exact head and settlement state.
- `SettlementDeferred` survives a restart.
- A durable home reopens in a fresh process with its Relational truth exact.
- Warming costs reads proportional to the head, never to its history.

### D.10 Durable runtime state

**Scope:**

- **Every remaining item on D.2's reopen inventory becomes durable,** behind
  storage ports in the same pattern as D.8. That covers World's registry,
  history, custody and recovery catalog, and the Query host's state.
- **World composite commits** use D.4's atomic multi-root publish. Once every
  component owner is a Store root, delete
  `WorthQueryApplicationCommitOutcome::ProductUnpublished` and World's
  ProductUnpublished recovery catalog. This is a public enum change, so the
  bank-server gate runs.
- **Residency follows demand.** Define the mapping from output-demand sources
  to warm units. The memory budget joins the `runtime` resource profiles.
- **`ApplicationHome::memory()` moves to the memory media from D.7.** The old
  in-memory runtime path is deleted.

**Acceptance:**

- **A larger-than-memory world:**
  - the data set is several times the resident limit;
  - fault counts are exact;
  - the restart is bounded.
- **The working set stays warm.** Demanded data is served without faults while
  cold history is read alongside it.
- **Reopen resumes the whole inventory,** including:
  - a workflow mid-run;
  - a pending aftermath;
  - an indeterminate commit resolved by retry.
- **A crash matrix** at every World, Query host and Relational progression edge.
- **Parity** with the D.2 in-memory runtime on the World and Query host suites.

Closing D.10 delivers the working database.

### D.11 Space reclamation across branches

**Scope:**

- **Reachability marking** runs from:
  - every live root;
  - the roots retained checkpoints need;
  - every reader lease: retained reads, published snapshot handles and open
    cursor leases.
- **Unreachable node pages are freed one by one,** which D.3's page-granular
  nodes make possible.
- **C.11's arena retirement is redesigned** for many roots, and it reclaims
  arenas the marking leaves empty.

**Bug class:** *a reachable page reclaimed.* Reclamation consumes a mark proof
taken at a published generation set, and a lease taken after the mark blocks
the free.

**Acceptance:**

- A seeded differential test runs fork, publish and drop with reclamation on and
  off. Reads are identical, and space is returned.
- The crash matrix covers each reclamation edge.

## Relationship To Other Roadmaps

- **Physical Foundation Reconstruction Roadmap:**
  - C.1 to C.10 are closed;
  - C.11, C.12 and C.13 are superseded here;
  - their specs stay as design reference, not as plans.
- **Physical Database Roadmap:**
  - S.10, S.11 and S.12 keep their positions before their consumers;
  - its Runtime Integration Entry Gate is replaced by D.10 closeout.
- **[Runtime Integration Roadmap](../worth-store/runtime-integration-roadmap.md):**
  - it predates this data model;
  - D.2 reconciles it;
  - until then, "Milestone N" in the table below refers to its current
    numbering.

## Deferred Work

Each item has no facade port and reports `Absent`, or it is internal and
changes no public API.

| Item | Owner | Returns |
| --- | --- | --- |
| Blob ingest and read ports | Store blob facade | Before the first consumer of large opaque values (Milestone 12) |
| Abandoned-ingest reclaim and ingest expiry ports | Store blob custody | With the blob ports |
| Release, terminal-head retirement and tier movement (C.11 Phase 6 remainder), redesigned for many roots | Store blob custody | After D.11, before Milestone 15 |
| Release and pending-WAL rejoin rework; release-limit sweeps; pruning replaced release-control chains | Store rejoin | With the release work |
| Same-tier relocation and maintenance ports | Store maintenance | Milestone 6 |
| Linear ordered-history walk; physics refusals with no count | Recovery runtime | Before the first blob, maintenance or relocation port |
| Limit-versus-damage separation for selected-rejoin bound refusals | Recovery runtime | S.10 |
| D.1 follow-ups: cleanup-Intent replay above the checkpoint (ignored repro in `physical_blob_journeys`), covered-frame guard tests, checkpoint-sequence-0 identities | Recovery runtime | With the linear walk |
| Observer head readers for the newest manifest schema, and blob-walk rules | Integrity observer | With the blob ports |
| LSM layout class and compaction (C.11 Phase 7) | Store LSM | Before the first write-heavy family needs it (Milestone 9) |
| Export and import (C.11 Phase 7) | Store | Milestone 11 |
| C.11 Phase 8 heavy lane and full matrix | Store | S.12 |
| C.12 formal protocol rebinding | Formal models | Milestone 19 |
| C.13 joined hostile campaign | Store | S.12 |
| S.10 bootstrap, recovery, PITR, backup and repair | Store | Milestone 8 |
| S.11 security scope and key lifecycle (uses the reserved key-identity slot) | Store | Milestone 10 |
| S.12 bulk ingest and joined certification | Store | Milestone 14 |
