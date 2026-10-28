# Milestone 3: Durable Application Runtime

Governing roadmap: [Database Foundation Roadmap](roadmap.md#milestone-3-durable-application-runtime).
Builds on: [Milestone 1](milestone-1.md) and [Milestone 2](milestone-2.md).

An application opens one file-backed home through one Query host call. Every
supported runtime owner publishes its own durable artifacts through Store,
recovers through its own readmission, and continues working after process
death. That covers graph state, application occurrences and their original
results, Signal and Bridge definitions, programs, workflows and their
obligations, external custody and corrections. A retry after a lost response
returns the original occurrence. Nothing an application was told is committed
depends on process memory, orderly close or application-owned byte transport.

This milestone is delivered in ten numbered phases. Each phase states its own
requirements, design, required tests, oracle extension, documentation and exit
criteria. The cross-cutting decisions below bind every phase.

## Phase map

| Phase | Name | Depends on | Lands |
|---|---|---|---|
| 1 | State survival audit | — | On its own (documentation) |
| 2 | Store capacity, single writer and incarnation | — | On its own |
| 3 | Application home and host lifecycle | 2 | Together with Phase 4 |
| 4 | Durable occurrences and original-result retry | 3 | Together with Phase 3 |
| 5 | Signal and Bridge authored state; mixed publication | 4 | On its own |
| 6 | Programs, adoption and build identity | 4 | On its own |
| 7 | Workflows, conditional and long-running work | 5, 6 | On its own |
| 8 | External dispatch and custody | 4, 7 | On its own |
| 9 | Correction and original-owner advancement | 4, 6, 7 | On its own |
| 10 | Public cutover and milestone certification | 1–9 | Closes the milestone |

**Parallel work and landing order.**

- Phases whose dependencies have landed may be built in parallel, but only
  where their write scopes do not overlap.
- They land in numerical order unless a reviewed merge proves otherwise.

**Landing rule.**

- A phase lands on master only after its exit criteria hold and an independent
  reviewer certifies it.
- Phases 3 and 4 land together. A home that recovered graph state but not
  World occurrences would silently lose application history.
- Interim refusals (D15) keep every landed state honest, and master is never in
  a half-recovered state.

**Format stability.** An artifact format defined in a phase is final for the
milestone. Later phases add new artifacts or new tagged entries in an
extensible list the earlier phase declared; they do not change an existing
format. A phase that must change a landed format states which development
homes stop opening. Under clean cutover they refuse with a typed reason and
are never migrated.

## Adversarial constraint

A home is killed at any durable I/O boundary of any supported operation. That
includes every owner's publication, product movement, program transitions,
checkpoints, close and recovery itself. The home is then reopened any number
of times, in a fresh process, under the same or a changed application build
identity.

Each reopen must:

- recover exactly the acknowledged prefix of every owner, plus at most the
  in-flight operation, decided from durable state;
- reach owner state equal to a memory-only reference host executing the same
  operation sequence at that prefix. This covers graph records, World
  occurrences and custody, product heads, idempotency outcomes, Signal and
  Bridge definitions, programs, workflow state, external custody and
  correction lineage;
- answer every retry of an admitted key with its original occurrence, its
  recovered custody posture, or its typed no-effect posture, and never with a
  second effect;
- never reissue a committed identity of any owner;
- refuse before service, with a typed reason, when it cannot do the above.

A crash between owner effects preserves which owners acted and whether the
product moved. Unresolved effects keep their recovery posture until their
owner establishes the fate. Loss of process memory, cancellation and caller
abandonment are never an implicit completion, rollback or no-effect.

## Decisions

**D1. One application home, Store-owned, host-opened.**

A new Store-workspace crate, `worth-store-application`, owns the application
home. It handles open, initialization, recovery sequencing, checkpoint,
resolve and close for every owner (Milestone 1 D1). It replaces
`RelationalStoreHome` as the only home type. `worth-store-relational` keeps the
Relational medium adapter and cold source, which become the Relational
section of the home.

The Query host is the composition root. `worth-query-host` opens the home
through one call. Applications never construct, open or synchronize a Store
runtime, and they import only `worth-query-decl` and `worth-query-host`.

This requires an explicit boundary ruling. It is recorded as
`dependency_denials` and allowlist entries in
`tools/boundary-check/config/road1.toml`; band rules alone do not cover crates
outside `worth-contracts`. The ruling has three parts:

- **`worth-query-host`** may depend on `worth-store-application`, and only its
  production composition module may name it. No other `worth-query*` crate may
  depend on any `worth-store*` crate.
- **`worth-store-application`** may depend on the owner crates whose ports it
  implements: Relational, World, Signal, Bridge, and the Query crates that
  define Query-owned artifacts. It must never depend on `worth-query-host`.
- **`worth-store-application-oracle`** (D14) may depend on `worth-query-host`
  and `worth-store-application`. No production crate may depend on it.

**Build cost.** From now on, the Store workspace builds the Query, World and
Signal stacks for its application crates, and the host builds the Store
stack. Phase 3 records the measured build-time impact on every lane in its
evidence. Store lane partitions keep the physical-only suites off the
application crates.

**D2. Owners define artifacts and ports; Store implements them.**

Each owner defines, in its own crate:

- its durable artifacts: frame, version, digest and readmission;
- a durable medium port with the Milestone 2 answers `Committed`,
  `ProvenNoEffect` and `Indeterminate`;
- a recovery source it readmits from.

Store never decodes an owner payload. `worth-store-application` implements
every port over named journals and anchored records. Owner ports are plain
traits with no filesystem or Store dependency, so Signal still compiles for
`wasm32`. An owner's readmission is its only door from bytes to authority.

**D3. Store capacity: root schema 12 and a journal registry.**

- **Anchors.** Root manifest schema 12 widens the inline anchor table to 32
  anchors. The payload is 3,760 bytes, within the smallest page.
- **Unchanged.** The WAL advance codec, recovery projection and successor-root
  carry-forward rules do not change.
- **Journals.** The home crate declares the ordered-journal registry as a
  closed, versioned set of names. Each journal carries its own unresolved
  state.
- **Clean cutover.** Schema 11 roots refuse at open with
  `UnsupportedRootSchema`, and nothing reads them. No production home exists to
  preserve.

**D4. Owners publish in order; product movement is its own durable step.**

No root decision moves more than one anchor. A composite operation publishes
each owner's effect as that owner's own durable step, in World's fixed owner
order, durable before live (Milestone 2 D2). Product movement is World's
publication of a `RuntimeWorldOccurrenceArtifact` to `world.occurrences`. The
live product compare-and-swap happens only after that publication answers
`Committed`.

- `ProvenNoEffect` on product movement leaves the owner effects as
  `ProductUnpublished` custody, with their existing next actions.
- `Indeterminate` returns `ProductUnpublished(ProductPublicationUnresolved)`.
  Store's unresolved state is per journal and `world.occurrences` is one
  journal, so this blocks product movement on every branch of the home until
  reopen. Owner effects that do not move a product remain admissible.
  Recovery decides the fate.

No competing transaction coordinator, idempotency table or recovery authority
is added. World stays the composite coordinator, and its existing catalogs gain
durable backing.

**D5. Owner effects carry their attempt association; custody dispositions are
durable.**

Every owner effect published inside a composite attempt records the attempt's
World-defined `CompositeAttemptAssociation`: the home-scoped attempt identity,
product branch identity and incarnation. It sits in the owner's own artifact:

- a Relational commit carries it in a Relational-owned, opaque, size-bounded
  caller-association field of the canonical envelope (a new artifact version);
- a Signal advance carries it in its advance artifact (Phase 5);
- later owners follow the same pattern.

Recovery joins owner effects to occurrences by association. An owner effect
whose association has neither an occurrence nor a disposition is recovered as
durable `ProductUnpublished` custody with cause
`RecoveredWithoutProductMovement`.

Continuing or releasing custody publishes a `RuntimeWorldCustodyDisposition`
to `world.custody` before its live effect. Released custody therefore never
reappears on reopen. Open custody is carried in World's checkpoint section
(D12). Nothing is folded silently into a new root or re-executed.

**D6. Durable, home-scoped identities.**

Every identity that a durable artifact or durable row records becomes
home-scoped and runtime-neutral. For World these are owner, branch,
incarnation, reference generation, composite commit, publication attempt,
bootstrap attempt and `ProductUnpublishedOwnerEffects`. The equivalent Signal,
Bridge and Query identities follow in their phases.

- **Recording.** The home descriptor records a home identity at
  initialization.
- **Frontiers.** Issuers resume one past the largest durable value (Milestone 2
  D7) and issue deterministically in order.
- **Process-local identities.** `runtime_instance_id`, session ids and
  graph-instance ids remain only for values no durable artifact or row
  records. The Phase 1 audit lists every durable row that records one today.
  The phase that makes the row's family durable replaces that field with
  home-scoped identity.

**D7. Single writer and incarnation fencing.**

- **Writable opens.** Every open is writable; this milestone provides no
  read-only open.
- **Lease.** Store's OS mutation lease (`MutationOwnershipLease`) is held
  continuously from the start of recovery through serving. The handoff
  transfers the held lease; it never drops and reacquires it.
- **Incarnation.** Each open durably advances the home incarnation in the home
  descriptor before service. Values that carry authority are bound to that
  incarnation: capabilities, handles, sessions, leases and recovery claims. A
  value from an earlier incarnation refuses with `StaleIncarnation` before any
  effect.
- **Durable data is not fenced.** Receipts, occurrence identities, commit
  identities and history cursors are resolved by home identity and affinity,
  and remain valid across reopen.
- **New homes.** Initialization happens under the lease. A location is new
  only when the lease holder observes no Store at all. A Store left partly
  initialized is completed or refused (Milestone 2 D5), never treated as an
  empty home.

**D8. Recovery order is Store-sequenced.**

Open runs these steps in order:

1. Store physical recovery.
2. The home descriptor, then the incarnation advance.
3. Relational readmission.
4. Signal and Bridge readmission.
5. World readmission and reconciliation (D5).
6. Query readmission: idempotency, programs, workflows and custody.
7. Dispatch and correction custody.
8. Completion of a pending program transition (D16), or the transition the
   open request carries.
9. Service.

Each owner readmits only its own artifacts, after the owners it depends on.
Any refusal is typed and happens before service. Steps 1–7 are recovery: they
never republish, re-execute a handler, or issue a replacement commit or
occurrence. Step 8 is not recovery. It is the transition's own forward
progress, run against fully readmitted owners before service, under D16's
plan-digest check.

**D9. Idempotency and receipts resolve from durable state.**

Query's application idempotency binding is recorded in the same Relational
commit as the effect, as it is today. It now also records product branch
affinity (identity and incarnation) and the attempt association. A retry is
answered by reading the durable binding, the World occurrence or custody, and
the owner artifacts. The original receipt is rebuilt from those, field by
field, as Phase 4's receipt table defines.

- **Evidence store.** `WorthQueryCompletedCommitEvidenceStore` stops being an
  authority. It becomes a derived in-process index:
  - it is filled at commit and rebuilt on demand from durable state after
    reopen;
  - it is never the only source of an answer;
  - its session index stays process-local and incarnation-fenced;
  - its dispatch-correlation index is rebuilt from durable outbox rows.
- **Deleted.** `CommittedReceiptNotRetained` is deleted, and the
  retained-receipt-basis cap no longer bounds answers. `worth-server`'s
  in-memory retry caches are deleted; Server retries go through Query's
  contract.
- **Affinity check.** The `runtime_instance_id` equality in resolution is
  replaced by home identity plus affinity.
- **Concurrency.** Same-key concurrent retries are serialized by the existing
  branch commit lane under the single writer (D7).

**D10. Retention classes.**

Every durable artifact belongs to exactly one class:

- **Authoritative.** Never expires while its home exists.
- **Mandatory obligation.** Retained until its owner discharges it.
- **Optional retained material.** May expire under a declared policy. Expiry
  answers a typed `RetainedMaterialExpired` and never authorizes re-execution.
- **Derived.** Rebuildable, and never authoritative.

Idempotency bindings, occurrences and custody dispositions are authoritative.
Reclamation is Milestone 4; this milestone declares the classes and never
reclaims authoritative or obligation data.

**D11. Close.**

`close` stops admission, then settles or durably retains accepted work within
its request deadline. It answers `Closed` or `CloseRefused { posture }`.

- On refusal, the home stays open and serving, or it enters an explicit
  recovery posture the caller can inspect. It never silently seals a runtime
  the caller was told remains open.
- Acknowledged results never depend on close; durability is reached at
  publication.
- Close attempts a home checkpoint before releasing the lease (D12). A failed
  checkpoint never refuses close. It is reported in `Closed`, and the next open
  replays the longer tail.

**D12. One home checkpoint, taken automatically.**

A home checkpoint is one manifest under the `home.checkpoint` anchor,
published as one root decision.

- **Sections.** The manifest has one section per owner. Each section uses the
  owner's own checkpoint format, with content-addressed members as in
  Milestone 2 D4. The Relational manifest becomes the Relational section
  without changing its format.
- **Coverage and crashes.** Each section covers a position of its owner's
  journals, and recovery replays only the tails. A crash at any point of a
  checkpoint leaves either the previous or the new checkpoint selected.
- **Triggers.** The host checkpoints automatically:
  - when any journal's tail since the last checkpoint passes a threshold
    (count and bytes, with defaults in the host limits);
  - at close.

  Applications may also checkpoint explicitly. Reopen work is proportional to
  the manifest plus the bounded tails, not to history.

**D13. One durable path; the memory-only mode stays explicit.**

The host has one open call, with two locations:

- **A directory.** Durable.
- **Memory-only.** Explicitly non-durable, with unchanged semantics.
  Memory-only evidence never qualifies durability.

Application-owned checkpoint transport is retired by clean cutover in Phase
10. That covers:

- `capture_application_checkpoint*`;
- `WorthQueryApplicationCheckpoint` byte transport;
- the `*_from_checkpoint*` constructors;
- `repair_to_checkpoint*`;
- Relational's embedded-checkpoint placement.

Until then it is a named interim path for callers whose state families are
not yet durable.

- **One door.** Phase 3 collapses it into a single door,
  `WorthQueryInterimCheckpointRestore`, taking the new home limits. Nothing in
  the interim path keeps an `in_memory` name or `InMemory` type.
- **Frozen callers.** Its callers are frozen in
  `tools/boundary-check/config/interim-checkpoint-callers.txt`. A new caller
  fails boundary-check, and callers are only ever removed.
Checkpoint-transition and program-adoption contracts move onto the open home
(D16, Phase 6). The roadmap's former "explicit embedded-save" mode is retired
with this path.

**D14. Interruption controls and the host oracle.**

- **Store faults.** Store faults and pause gates use Store's
  `certification-test-authority` feature. `worth-store-application` forwards
  it, as `worth-store-relational` does, inside Store's one-build exemption.
  No feature is declared outside the Store workspace.
- **Owner boundaries.** Every owner effect, custody disposition, transition
  step and product movement is a Store publication, so Store's mutation seams
  interrupt every owner boundary.
- **Oracle location.** The host oracle lives in
  `worth-store-application-oracle`. Its children re-execute the test binary,
  as Milestone 2's relational_home children do, and run the real production
  host.
- **Derived-state door.** Discarding derived state uses a sealed certification
  door on the home (`discard_derived_for_test`). Its callers are recorded in
  `control-callers.txt`.
- **Owner unit courts.** Owners outside Store use `cfg(test)` in-crate media.
  Those are not a second durable path.
- **The oracle's identity rule.** The memory-only reference host and the home
  issue home-scoped identities from the same initial frontiers in the same
  order (D6). Comparison covers only durable identities. Identities consumed
  by attempts that never became durable are not part of the prefix. After each
  reopen, the reference's issuers are reset to the recovered frontiers before
  the script continues, so later identities compare exactly. Process-local
  identities are excluded from comparison.
- **Seeds and lanes.** A failing seed reproduces deterministically. Focused
  lanes run a small seed range. A scheduled lane runs the broad campaign under
  a stated time and memory budget.

**D15. Typed unsupported families, for directory homes.**

A `DurableStateFamily` enumeration names every durable family from the Phase 1
audit. Each family maps to the operation kinds and declared capabilities that
can create it. The mapping includes capabilities that create the family only
on failure, such as managed resources, which can leave managed cleanup.

Until the phase that supports a family lands, on a directory home:

- an operation whose kind or declared capabilities can create the family
  refuses at operation admission with `UnsupportedDurableFamily { family }`;
- opening a directory home that holds the family refuses before service.

Memory-only homes are never refused. Each phase's "Interim refusals" lists
exactly which operation kinds refuse until it lands. It also states, for each
existing caller (bank world, Server, examples, fixtures), which
location that caller uses meanwhile. Phase 10 deletes every variant.

**D16. Program transitions are durable and resumable.**

Opening a home with a program or schema revision different from the one
recorded requires a transition. The open request carries it as an optional
part: predecessor, transition author, resources and dispositions. Opening
without a required transition refuses with `TransitionRequired`, naming the
recorded and requested revisions.

A transition proceeds in this order:

1. **Plan.** The transition author computes its complete step plan against
   the readmitted predecessor state.
2. **Intent.** Publish a `HomeProgramTransitionIntent` to `home.transitions`.
   It names the predecessor, the successor, the transition identity and the
   digest of the complete step plan.
3. **Steps.** Publish the migration steps: Relational schema transitions,
   commits and dispositions. Each step is positioned, and each carries the
   transition association.
4. **Binding.** Publish the descriptor's program binding.

At open step 8 (D8), an intent without a binding is pending. Open recomputes
the plan from the same predecessor state and checks it against the recorded
digest:

- **The plan matches.** The transition resumes from the first step that is not
  durable. Durable steps are never re-run.
- **The plan differs.** Open refuses with `TransitionPlanDrift`. A developer
  who edited the migration after a crash cannot mix two migrations.
- **The successor differs.** Opening with a different successor refuses with
  `TransitionPending`.

A pending transition can only be finished with its original plan, or
discarded with the home.

The supported development workflow is:

- **Edit and reopen.** After editing a program or schema, reopen with the
  transition.
- **Discard.** Discard the home through the host's `discard_home(location)`.
  - It does not open the Store. It takes the filesystem mutation lease, so an
    open home cannot be discarded.
  - It first writes a discard tombstone at the location, then removes the
    Store.
  - It is idempotent. It completes a partly discarded or damaged location.
  - Opening a tombstoned location refuses with `DiscardPending` until
    `discard_home` completes.

## Anchors and journals

These are the closed registry of D3. A name is final when the phase that
creates it lands.

| Name | Kind | Owner (format) | Class | Phase |
|---|---|---|---|---|
| `home` | Anchored record | Store home: identity, incarnation, program/schema/build binding | Authoritative | 2–3 |
| `home.checkpoint` | Anchored record | Store home manifest; owner sections | Derived | 3 |
| `home.transitions` | Journal | Store home: transition intents (D16) | Authoritative | 3 |
| `relational.canonical-commits` | Journal | Relational (with caller association) | Authoritative | M2, 4 |
| `relational.branch-transitions` | Journal | Relational | Authoritative | M2 |
| `world.occurrences` | Journal | World occurrence | Authoritative | 4 |
| `world.branches` | Journal | World product branch lifecycle | Authoritative | 4 |
| `world.custody` | Journal | World custody dispositions | Authoritative | 4 |
| `signal.definitions` | Journal | Signal definition transitions (with Bridge section) | Authoritative | 5 |
| `signal.advances` | Journal | Signal branch advances | Authoritative | 5 |
| `query.obligations` | Journal | Query: support lifecycle, adoption, required-output custody (R8), managed, conditional, owned-async custody, original-owner custody | Obligation | 6–7, 9 |
| `query.dispatch` | Journal | Query: dispatch attempts, inbound custody, recovery claims | Obligation | 8 |
| `query.corrections` | Journal | Query: retained correction material and custody | Per contract | 9 |

The M2 `relational.home` and `relational.checkpoint` anchors are replaced.
`relational.home` is replaced by `home`, and `relational.checkpoint` by the
Relational section of `home.checkpoint` (D12).

Query's idempotency binding is not a journal. It travels inside the
Relational commit (D9).

---

## Phase 1: State survival audit

**Goal.** Classify every production-owned state family before any of it is
made durable. This gives every later phase an exact scope and every refusal an
exact name.

**Requirements**

- **Inventory.** Cover every state family held by Relational, World, Signal,
  Bridge, Query (installation, execution, publication), Server and the bank
  world. For each one, record:
  - its owner and owning type, with path;
  - its survival class (D10), or that it is process-local;
  - its current persistence: none, a Relational row, or in-memory;
  - the process-local identities it records (D6);
  - the phase that makes it durable. If it stays process-local, record how it
    is reacquired through fresh authority.
- **Unsupported families.** Name the families that become
  `DurableStateFamily` variants. Map each to the operation kinds and
  capabilities that can create it (D15).
- **Durable rows with process-local identities.** List every durable row
  recording `runtime_instance_id` or any other process-local identity, with
  the phase that replaces it. This includes digests derived from such
  identities, such as transport causal digests, workflow receipt digests and
  runtime source commitments. The causality rows' `world-owner` is an
  example.
- **Known defects.** Record each with its target phase:
  - the temporal idempotency key is derived from per-process runtime
    identities;
  - `PayloadRejected` is classified as unresolved;
  - support lifecycle resets to Active at install;
  - only the main branch's program activation is verified on restore.
- **Interim-path inventory.** List every public entry path retired by D9 and
  D13, with its callers. For each caller, give the location it uses in each
  interim phase.

**Deliverable.**

- The index is `plans/database-foundation/milestone-3-survival-audit.md`. It
  holds:
  - the `DurableStateFamily` list and its capability mapping;
  - the table of durable rows that record process-local identities;
  - the defect table;
  - the interim-path table.
- Per-owner inventory parts live in
  `plans/database-foundation/milestone-3-survival-audit/`.

**Required tests.** None; this phase writes no code.

**Exit criteria**

- An independent reviewer samples at least twenty audit rows against the code
  and finds them accurate.
- Every family and every row with a process-local identity that a later phase
  names appears in the audit, and no family is unclassified.

### Audit resolutions

The Phase 1 audit raised eighteen gaps in the specification. Each one is
resolved below and tagged with the phases it binds. A phase's requirements
include every resolution tagged with it.

- **R1 (Phase 1, index).** Part A audits Relational as the carrier of rows.
  The rows themselves are written by callers (idempotency, outbox, workflow,
  causality and others), and Parts B and C audit those. The index joins
  carrier and caller into one family per row kind, named after the caller.
  There is no generic "caller rows" family.
- **R2 (Phase 5).** Signal's authored topology is the declared node
  definitions plus the static dependency edges those definitions declare.
  Everything else is derived and rediscovered by evaluation:
  - dependency snapshots;
  - subscriber and evaluation edges;
  - dynamically discovered dependencies;
  - cause sets;
  - diagnostics.

  The Phase 5 authored-image format states this split before it is final.
- **R3 (Phase 5).** The `worth-signal-wasm` facade's `RuntimeStore` and
  portable envelope are a separate product surface. Neither is opened through
  the Query host, and both are outside this milestone. JS callbacks are never
  serialized. D15 does not apply to the facade.
- **R4 (Phase 5).** Some records are process-local diagnostics, not durable
  state: Bridge's writeback execution, replay and comparison records, and
  Relational's diagnostic artifact vector.
  - Their accessors gain a typed answer that distinguishes three cases: never
    retained, expired under count limits, and not retained across reopen.
  - Any evidence that recovery needs must be derivable from durable commits.
    For writeback, Phase 5 proves this.
  - `SignalSnapshotV1` stops being a restore authority, because the Phase 5
    artifacts replace it. Its restore door is deleted, or stays memory-only
    if the audit shows a memory-only consumer. It is never a second durable
    path.
- **R5 (Phases 5 and 7).** Two Signal structures hold real request custody and
  are not derived: `ResourceRuntimeState.in_flight_by_request` and native
  managed admission.
  - Until Phase 7, operations that admit Signal resource requests refuse on
    directory homes under D15.
  - Phase 7 extracts that custody into a durable form before any remaining
    resource state is treated as derived.
- **R6 (Phases 1 and 8).** The outbox wording is corrected above. Phase 8
  removes the process-local identity from every row and digest that the audit
  lists for it.
- **R7 (Phases 7 and 9).** Some run kinds retain opaque process objects:
  provider checkpoints, memory arenas, artifact contexts and `Arc<dyn Any>`
  commit authority. Serializing such an object is never allowed. Each of
  these run kinds takes one of two routes:
  - an owner-defined, versioned data contract and readmission;
  - a terminal posture that its existing lifecycle contract already defines.

  The phase's first deliverable is a per-kind table naming the route each run
  kind takes.
- **R8 (Phase 6).** General required-output custody is a durable obligation in
  `query.obligations`. That covers `DemandRecord` performed obligations and
  prerequisites, and `SourceCustody`. Ready outputs remain derived.
- **R9 (Phase 7).** Every variant of the conditional retained decision is
  classified:
  - **Durable obligations**, because they carry an effect or an obligation:
    CleanupRequired, Retryable, CommitRetryable, ExecutionControlRetryable,
    ProductStale, ProductUnpublished, Indeterminate and
    InterruptedDuringReentry.
  - **Process-local**, because they can be re-derived from durable state and
    host time: Eligible, Deferred, Suppressed and Backpressured.
  - **Terminal**: ControlStopped, TerminalFailure, Failed and NoEffect. These
    are recorded durably exactly when a caller can still ask for them.

  Phase 7 publishes this table with its evidence.
- **R10 (Phase 7).** Workflow proposal coverage is a set of graph records: it
  is authored, and it recovers with Relational. Installed vocabulary coverage
  is derived and is recreated from the supplied program. An installed
  interpreter is never persisted as authored coverage.
- **R11 (Phase 9).** Each retained use of correction material declares its
  D10 class.
  - While a prepared correction or a recovery binding pins the material, it is
    a mandatory obligation.
  - Unpinned material is optional.

  Pinning, and transfers of a pin, are recorded together with the class.
- **R12 (Phases 4, 5 and 7).** D15's capability mapping splits branch cleanup
  by component. World native cleanup is supported from Phase 4, Signal cleanup
  from Phase 5, and conditional-assembly cleanup from Phase 7. On a directory
  home, a branch operation refuses if its cleanup includes a component that
  is not yet supported.
- **R13 (Phase 8).** Revocation and retirement of an inbound source are
  home-level security facts, and they are durable. Reopening never resets a
  revoked source to Active. Verifier installation is fresh host configuration
  applied on top of that durable posture.
- **R14 (Phase 8).** The bank external rail is the external party. Its own
  survival is outside WORTH, and Phase 8 does not make it durable.
  - Phase 8's rail courts kill the application, not the rail.
  - One court restarts the rail with its state lost. The application must not
    claim completion: the effect stays indeterminate and is reconciled under
    the rail's admitted contract.
- **R15 (Phases 3, 4 and 9).** `BankHttpMutationContinuation` is transport
  custody. It is re-created fresh at open.
  - The durable original owner is the per-operation record in its worker
    records and in `mutation_custody`. Phase 9 makes that record durable.
  - Until then, actions that retain original-owner custody refuse on
    directory homes under D15.
- **R16 (Phases 3 and 9).** Recovery of HTTP state:
  - HTTP recovery and continuation tokens carry authority, so they are fenced
    by incarnation (D7). After reopen they answer `StaleIncarnation`. A fresh
    authentication then reaches the recovered durable custody by its
    operation identity.
- **R17 (Phase 6).** Installation splits its canonical identities into two
  kinds:
  - **Semantic identity** is derived from content and may be durable.
  - **Authority identity** (installed-runtime ordinals and participation
    pointers) is process-local and fenced.

  Durable artifacts record only semantic identity. Equivalence checks compare
  semantic identity, plus the authority re-established at open. They are
  never dropped.
- **R18 (Phase 3).** Server upload staging is process-local. Interrupted
  upload sessions end at reopen with a typed `UploadSessionExpired`. Durable
  ingress is outside this roadmap.

---

## Phase 2: Store capacity, single writer and incarnation

**Goal.** Give Store what an application home needs, before any owner uses it.

**Requirements**

- **Schema 12.** Root schema 12 with 32 anchors (D3):
  - successor-root builders, maintenance and residue publications carry the
    table forward unchanged;
  - schema 11 is refused with `UnsupportedRootSchema`;
  - the offline integrity observer reads schema 12.
- **Journal registry.** The journal registry is declared by the home crate. A
  publication to an unregistered name refuses before any effect.
- **Lease handoff.** The lease is handed off continuously from recovery to
  serving (D7). The drop-and-reacquire window at `physical_store.rs` recovery
  is deleted.
- **Incarnation.** The home incarnation is recorded in the descriptor and
  advanced durably on every open, before service.
- **New-home detection.** Initialization happens under the lease (D7).
  `location_is_unused` stops deciding newness without the lease.
- **Milestone 2's home.** It runs on schema 12, with no other behavioral
  change.

**Required tests**

- **Anchor table:**
  - 32 anchors publish, recover and carry forward through maintenance;
  - a 33rd anchor refuses with `AnchorTableFull`;
  - the WAL recovery projection replays advances of high-index anchors.
- **Refusals before any effect:**
  - a schema 11 root refuses at open;
  - a publication to an unregistered journal refuses.
- **Two processes open the same location concurrently.** Exactly one serves;
  the other refuses with `MutationOwnerContended`. Covered for an empty
  location, an initialized home and a home that needs recovery.
- **Kill and reopen.** A process holding a home is killed. The next open
  recovers, and the incarnation increases by exactly one per open.
- **Lease held through recovery.** An observer process fails to acquire the
  lease at every recovery yieldpoint and at the handoff.
- **Offline integrity observer.** It reads a schema 12 root.

**Oracle extension**

- Milestone 2's relational_home oracle runs once on schema 12: media sweep,
  process kills and recovery interruption.
- New media and process cases cover the incarnation advance at open.

**Docs.** The Store README and the `worth-store-relational` guidance cover the
schema, the lease and the incarnation.

**Exit criteria**

- One full lane run on the landing head, with every lane green.
- The relational_home oracle passes once.
- Independent certification.

---

## Phase 3: Application home and host lifecycle

**Goal.** The production Query host opens, recovers, serves, checkpoints and
closes a file-backed home through one call, with every owner composed by the
home.

**Requirements**

*Home and boundary*

- Create `worth-store-application` and apply the D1 boundary ruling.
- Delete `RelationalStoreHome`. Its tests and the Milestone 2 relational_home
  oracle move to the application home's Relational section in
  `worth-store-application-oracle`.
- Record the measured build-time impact on every lane in the phase evidence
  (D1).

*The open call*

- `worth-query-host` exposes one open call. Its home request carries:
  - the location, either a directory or memory-only;
  - the validated program and support roster;
  - the application build identity;
  - trusted runtime services (time source and World resources);
  - the execution authority;
  - limits, including the checkpoint thresholds (D12);
  - `initial_state`;
  - an optional program transition (D16).
- The `in_memory*` constructors are deleted. Their memory-only callers move to
  the open call, and "InMemory" disappears from the limits, profile and denial
  names.
- The `*_from_checkpoint*` constructors collapse into the single
  `WorthQueryInterimCheckpointRestore` door (D13). Its callers are frozen in
  boundary config.
- A new home is distinguished from an existing one only through Store (D7).
  `initial_state` runs only for a new home.
- `discard_home(location)` (D16).

*Program and build binding*

- **Descriptor.** The home descriptor records the program identity and
  revision, the rostered revisions, and the schema revision.
- **Program transitions** follow D16. An incompatible program refuses before
  service.
- **Changed build identity.** Opening under a changed build identity is
  admitted. Authored truth is unchanged. Recorded derived outputs whose
  producing implementation differs are stale under the existing lineage
  contract, and nothing regenerates eagerly. Every rebuild mints a new build
  identity, so this is the ordinary local-iteration path.

*Close, fencing and checkpoints*

- Close follows D11; incarnation fencing follows D7; automatic checkpoints
  follow D12.

**Interim refusals (Phases 3–4).** On a directory home, these refuse per D15:

- operations that touch Signal or Bridge state;
- conditional, temporal and managed operations;
- workflow runs;
- dispatch and inbound completion;
- corrections;
- program adoption.

Meanwhile, the bank world, Server and Signal-dependent examples
use memory-only homes or the interim checkpoint path, as the audit lists.

**Required tests**

- **Open and reopen:**
  - opening an empty directory initializes the home and serves;
  - reopening recovers;
  - a damaged descriptor, a foreign home, an unknown root schema and an
    incomplete recovery each refuse before service, with typed reasons.
- **Competing opens at host level.** Two processes open one directory through
  the host. One serves; the other gets a typed contention refusal.
- **Rebuild.** Same program and schema, new build UUID: the home opens with no
  refusal, and stale outputs are reported without being regenerated.
- **Program transitions:**
  - a schema edit without a transition refuses with `TransitionRequired`;
  - with a transition, it migrates and serves;
  - a process killed at each seam of the transition (intent, each step,
    binding) reopens and resumes from the first undurable step, without
    re-running a durable one;
  - a resume whose recomputed plan differs from the recorded digest refuses
    with `TransitionPlanDrift`;
  - opening with a different successor while a transition is pending refuses
    with `TransitionPending`;
  - an unrostered or incompatible program refuses.
- **Discard:**
  - `discard_home` on an open home refuses;
  - on a closed home it removes the home, and the next open initializes;
  - a process killed during discard leaves a tombstoned location;
  - opening that location refuses with `DiscardPending`;
  - running `discard_home` again completes, including on a damaged Store.
- **Frozen interim callers.** Adding a caller of the interim checkpoint door
  fails boundary-check.
- **Fencing and durable data.** A capability, handle and session from
  incarnation N refuse with `StaleIncarnation` at N+1. A receipt and an
  occurrence identity from N still resolve at N+1.
- **Close:**
  - with accepted work in flight, close either settles it or refuses with an
    inspectable posture;
  - a refused close leaves a serving home;
  - acknowledged commits survive a kill without close.
- **Checkpoints.** After more commits than the threshold, a checkpoint exists.
  The reopen journal tail stays at or below the threshold, measured through
  the home's read counters with headroom.
- **Memory-only.** Memory-only mode passes its existing semantic suites through
  the new call.

**Oracle extension.** `worth-store-application-oracle` covers home open,
initialization, reopen, program transition, automatic checkpoint and close.
That includes the media sweep and process kills at every Store seam of each,
and recovery interruption at every stage.

**Docs**

- The host README sections on open, close, checkpoints and transitions.
- `build-an-application.md` §4 "Install or reopen", including the development
  workflow (D16).

**Exit criteria.** This phase lands with Phase 4, under Phase 4's exit
criteria.

---

## Phase 4: Durable occurrences, interrupted publication and original-result retry

**Goal.** The decisive journey: a lost response after a durable commit, a
kill, a reopen and a retry return the original occurrence, without
re-executing anything.

**Requirements**

*World becomes durable*

- World defines and readmits these artifacts:
  - `RuntimeWorldOccurrenceArtifact`. It records:
    - the composite commit identity and parent;
    - the home identity and the publishing home incarnation;
    - the product branch identity, incarnation and generation;
    - the attempt identity and provenance;
    - the program revision, populated from this phase on;
    - caller correlation;
    - an extensible, owner-tagged list of component evidence.

    Phase 4 defines the Relational entry: commit id, position and digest.
    Later phases add tags, never fields.
  - `RuntimeWorldProductBranchArtifact`: create, retire and incarnation.
  - `RuntimeWorldCustodyDisposition`: continue or release (D5).
- World's identities become home-scoped (D6). The process-static owner
  ordinal no longer reaches any artifact or row.
- Product movement is durable before live (D4).
- On reopen, World rebuilds its history catalog, product heads, ancestry and
  open custody from its checkpoint section plus its journal tails.
  Pre-crash history pages, ancestry tracing and `recover_performed` work
  after reopen.
- Reconciliation follows D5. Pending settlement is covered: a Relational
  commit that is durable but unsettled is recovered by Relational, and World
  holds its custody. `continue_effects`, `release_effects` and
  `prepare_settled_relational_adoption` operate on recovered custody.
- Milestone 2's unresolved forks retained as `ProductUnpublished` survive
  reopen as the same custody, with their native fate decided by recovery.

*Relational association*

- The canonical envelope gains the caller-association field (D5) under a new
  artifact version. Older artifact versions are refused (clean cutover).

*Idempotency and receipts (D9)*

- The binding records affinity and association. Resolution reads only durable
  state.
- Each original receipt field has a durable source:

| Receipt field | Durable source |
|---|---|
| commit id, position, digest, `CommitResult`, `changed_record_count` | Relational journal artifact, readmitted |
| commit evidence: idempotency binding | Query idempotency binding row |
| commit evidence: outbox seal | durable outbox rows in the same commit |
| commit evidence: provider session binding | Not recovered: sessions are process-local and incarnation-fenced (D7). A recovered receipt carries the publishing home incarnation in its place, and nothing resolves through the old session. |
| `basis_descriptor` | Relational artifact (runtime-neutral, Milestone 2 D11) |
| occurrence, old and new product head, attempt | World occurrence |
| outcome identity, emitted effect count | Query idempotency binding row |
| exact output settlement | Query outcome row in the same commit (new field) |
| aftermath causality | Relational causality rows |
| publishing runtime | home identity and incarnation, recorded in the occurrence |

- These are deleted: the receipt authority of the evidence store,
  `CommittedReceiptNotRetained`, and the Server retry caches.
- **Changed intent and foreign affinity.** A retry with changed intent refuses
  with `IdempotencyIntentDrift`. A foreign product branch or incarnation
  refuses with the existing affinity refusal, now evaluated against durable
  affinity.
- **Custody and fates.** A retry whose commit is in recovered custody answers
  `ProductUnpublished` with its next actions. An attempt with proven no
  effect executes once on retry. An unresolved attempt answers its unresolved
  posture until recovery decides it.

**Interim refusals.** These are the same as Phases 3–4 above.

**Required tests**

- **The decisive journey**, on the production host and filesystem media:
  1. Perform an ordinary mutation with an idempotency key.
  2. Let it durably complete, then drop its response.
  3. Kill the process without closing it.
  4. Reopen in a fresh process and retry the same request.

  The retry returns the original occurrence and exact branch state. The
  mutation handler's invocation count is unchanged, and there is no duplicate
  domain effect. The journey runs on both sides of the first home checkpoint.
- **Retry refusals:**
  - a changed-intent retry after reopen refuses;
  - a retry on a sibling product branch refuses;
  - a stale capability from before the reopen refuses.
- **Durable data across reopen:**
  - a pre-crash receipt used for history and ancestry lookups succeeds after
    reopen;
  - recovered service then performs another ordinary mutation.
- **Every receipt field** in the table equals the reference after reopen.
- **Crash between the Relational commit and product movement.** Recovery shows
  custody: the Relational effect retained, the product unmoved, and a retry
  answering `ProductUnpublished`. Continuing publishes the original commit
  without re-execution.
- **Release, kill, reopen.** The custody stays released.
- **Caller abandonment and cancellation.** These after an owner effect leave
  custody that survives reopen.
- **Proven no effect.** An attempt with proven no effect executes exactly once
  on retry after reopen.
- **Indeterminate product movement.** It blocks product movement on every
  branch until reopen (D4), and recovery decides it.
- **Concurrent same-key retries** produce exactly one occurrence, and every
  caller receives it.
- **History and identities.** World history pages and ancestry after reopen
  equal the reference. Identities issued after reopen never collide with
  recovered ones.
- **Server.** The retry route survives a restart of the Server process.

**Oracle extension**

- **Reference and script.** A memory-only host running the same program
  executes a seeded host script. The script covers keyed and unkeyed
  mutations, response loss, branch creation, product movement on several
  branches, settlement failure, custody continue and release, caller
  abandonment, and checkpoints. Every seed in the tested range generates.
- **Interruption points.** The oracle interrupts at every durable boundary:
  - before commitment;
  - after durable Relational publication;
  - between the owner effect and product movement;
  - after product movement, before the response;
  - during custody dispositions;
  - during checkpoints.
- **Comparison after each reopen.** Graph state, occurrences, custody, product
  heads and idempotency outcomes are compared with the reference prefix. The
  oracle checks the distinct no-effect, committed and unresolved outcomes. It
  never expects a committed receipt for an attempt that never committed.
- **Continuation.** The remaining script then runs to the reference's final
  state.
- **Fault coverage.** A media sweep of every Store operation of a mutation, a
  product movement, a disposition and a checkpoint; real process kills at
  every Store seam; and recovery interruption at every stage, including World
  reconciliation.

**Docs**

- `capability-index.md`, rows "Retry a mutation safely" and "Survive a
  restart".
- `build-an-application.md`: commitment, retry, close, unsupported homes and
  recovery, at the caller's level.
- The host README.
- The bank docs' process-death disclaimers, for Relational-only flows.

**Exit criteria (Phases 3 and 4 together)**

- **One full lane run on the combined head, every lane green.** That covers
  Relational, World, Signal (including wasm), Bridge, Server, Query (library,
  integration, examples, doc and fixture gates), the bank world, and Store
  (owner, scenario, process-scenario, formal, structural). It also covers
  boundary-check, agent-context, clippy, formatting and line caps.
- **Oracles.** The cumulative oracle passes once.
- **Certification.** An independent reviewer certifies.

---

## Phase 5: Signal and Bridge authored state; mixed publication

**Goal.** Signal and Bridge recover their authored state through their own
readmission. Mixed Relational/Signal operations recover at the true owner
boundary.

**Requirements**

*Authored and derived state*

- Signal separates authored state from derived and scheduling state:
  - **Authored:** installed definitions, runtime-published definitions,
    topology and branch structure.
  - **Derived:** values, dirty and aspect versions, pending causes, traces,
    telemetry, and the resource and temporal runtime.
- The authored image is a Signal-owned artifact with a versioned, checksummed
  byte codec, not serde snapshots.

*Signal port and artifacts*

- Signal defines `SignalDurableMedium`, a wasm-safe trait with the D2
  answers.
- `SignalDefinitionTransitionArtifact` records a change to authored
  definitions. It includes Bridge's conditional definitions and lowering
  claims as a Bridge-defined section.
- `SignalBranchAdvanceArtifact` records the branch, generation, attempt
  association (D5) and definition-transition reference. It never carries
  derived values.

*Identities*

- Signal's durable identities become home-scoped (D6): owner, branch, cell
  incarnation, generation, snapshot, admission and conditional contract
  occurrence.
- Descriptor readmission accepts a recovered owner and refuses a foreign one.

*Readmission*

- Multi-branch owner readmission installs program definitions and applies
  durable definition transitions in order. It restores branch structure and
  generation frontiers for every World branch, and leaves derived state cold,
  recomputed on demand.
- A derived value never stands in for a lost definition.

*Mixed publication*

- World occurrences carry a Signal component-evidence tag.
- A crash between the Relational and Signal effects recovers custody naming
  exactly the acting owners, with the product unmoved.

*Bridge*

- Conditional definitions recover through the Signal transition section.
- Bridge's lowering registry and retention ledger are derived, rebuilt at
  installation.
- Writeback replay records follow their audit classification.
- Owned-async custody belongs to Phase 7.

*Build identity*

- Signal derived state recorded under one build is stale under another. It is
  recomputed, never reused unconditionally.

**Interim refusals lifted.** Signal-touching and mixed operations, and
conditional-definition publication.

**Still refused until later phases.** Conditional and temporal operations,
workflow runs, managed and owned-async work, dispatch, corrections and
program adoption.

**Required tests**

- **Signal readmission refuses** a damaged frame, a foreign owner, an unknown
  version and a definition-basis mismatch.
- **Reopen with several branches.** A home with several branches, published
  definitions and conditional definitions reopens to authored state equal to
  the reference.
- **Derived values.** They recompute to the reference values on first demand,
  and none is read from durable bytes.
- **Mixed operation killed between owner effects.** It recovers custody with
  exactly the acting owners. Continuing completes it without re-executing
  either effect.
- **wasm.** `worth-signal-wasm` checks for wasm32 with the port present.

**Oracle extension.** The host script adds Signal-only and mixed operations,
conditional-definition publication, and Signal forks with branch creation. It
interrupts at every Store seam of each Signal publication, and between owner
effects.

**Docs**

- The Signal README durability section and the Bridge README.
- The capability index.
- The derived-recovery section of `build-an-application.md`.

**Exit criteria**

- One full lane run on the landing head, every lane green.
- The cumulative oracle passes once.
- Independent certification.

---

## Phase 6: Programs, adoption and build identity

**Goal.** Program lifecycle and adoption obligations survive restart, and a
changed build identity is an oracle dimension rather than a single case.

**Requirements**

- **Support lifecycle.** Program support status (Active, Retiring, Retired)
  and its custody counts become durable obligations in `query.obligations`.
  Counts derive from durable obligations and are never reset to Active at
  open.
- **Program activation.** Activation is verified on every branch at open, not
  only on main. Each occurrence's program revision (recorded since Phase 4) is
  checked against the branch's activation history.
- **Adoption custody.** These become durable custody:
  - `WorthQueryBranchAdoptionRecovery`, with its support custody and
    unpublished recovery;
  - `WorthQueryStoppedBranchSetAdoption`.

  Both resume through `recover_branch_adoption` and the branch-set resume
  after reopen. They are no longer caller-held values whose loss forgets the
  obligation.
- **Build identity.**
  - Authored source and references survive a changed build.
  - Incompatible derived outputs are marked stale under their reuse
    contracts. They are recovered or regenerated through the normal demand,
    dependency, invalidation and settlement owners, never eagerly at open.
  - Same-build reuse keeps its equivalence and ownership checks.
- **Derived caches.** Discarding derived caches or materializations loses no
  authored truth, history or custody.

**Interim refusals lifted.** Program adoption and support retirement.

**Required tests**

- **Retirement survives restart.** Retire support, kill, reopen: the status is
  still Retired, and interpretation refuses where the retirement requires it.
- **Adoption resumes.** Adoption interrupted at each stage across a kill
  resumes to the reference result, with no repeated effect.
- **Unrostered activation.** A non-main branch whose activation names an
  unrostered revision refuses at open.
- **Changed build.**
  - Stale outputs are reported stale.
  - Outputs regenerate only on demand, and keep their original entities.
  - Required-output obligations keep their execution and failure semantics.
- **Discarded derived state.** Discard derived state (D14 door), reopen:
  authored truth and custody are exact.

**Oracle extension.**

- From this phase on, two dimensions apply to every generated sequence:
  - build identity changes between reopens;
  - derived material is discarded between reopens.
- Adoption and retirement operations join the host script.

**Docs.**

- The program adoption guide.
- The "Build identity" sections of `build-an-application.md`.
- The capability index.

**Exit criteria.**

- One full lane run on the landing head, with every lane green.
- The cumulative oracle passes once.
- An independent reviewer certifies.

---

## Phase 7: Workflows, conditional operations and long-running work

**Goal.** Workflows, conditional and temporal operations, managed recovery
and owned-async work resume after process death, or terminate exactly where
their existing lifecycle contracts define a terminal posture.

**Requirements**

- **Workflows.**
  - Workflow definitions, instances, transitions, approvals and coverage are
    graph records, and recover with Relational.
  - Progress and replay retention are derived, rebuilt by reconstruction.
  - A workflow interrupted mid-run resumes from durable state under its
    original keys. Its required approvals remain required.
- **Workflow migration on transition.** A home holding workflow records
  transitions (D16) under the same workflow dispositions that live program
  adoption applies (`program_adoption/preparation/dispositions.rs`).
  `checkpoint_transition/selection.rs` refuses workflow records today; this
  phase deletes that refusal.
- **Managed cleanup.** Cleanup required after a failed commit becomes a
  durable obligation, recording the run identity and failure kind. Its
  process resources are reacquired through fresh authority. It is persisted
  even when clean close would have refused.
- **Conditional decisions.**
  - These retained decisions become durable obligations: CleanupRequired,
    Retryable, CommitRetryable and ProductStale.
  - Eligible, Deferred and Suppressed wakes are process-local. They are
    re-derived from durable state and host-fed time after reopen.
  - Clock leases are reacquired.
- **Temporal operations.** The temporal idempotency key derives only from
  durable, home-scoped identities, never from per-process runtime binding
  identity. A temporal intent re-executed after restart resolves to its
  original occurrence.
- **Managed runs and owned-async custody (Bridge and Query).**
  - Each yield, suspension and mandatory in-flight request is resumed from a
    durable record.
  - Termination at reopen is allowed only where the existing lifecycle
    contract defines a terminal posture for that kind. The termination is
    then typed and observable.
  - Loss of memory is never success.
- **Resources.** Resources are process-local and are reacquired through fresh
  authority.

**Interim refusals lifted.** Workflow runs, conditional and temporal
operations, and managed and owned-async work.

**Required tests**

- **Workflow killed mid-run.** It reopens, resumes under its original keys and
  reaches the reference terminal state. Its approvals survive.
- **Workflow migration.** A transition of a home holding workflows applies the
  migration dispositions. A kill at each transition seam resumes correctly.
- **Managed cleanup.** Kill with cleanup outstanding. The reopened home shows
  the obligation. Discharge it, and close then succeeds.
- **Temporal operation.** Kill it between its commit and its response.
  Re-execution after reopen yields one occurrence.
- **Managed-run suspensions.** Each kind either resumes, or terminates with
  the typed posture its contract defines.
- **Owned-async requests.** Request custody survives a kill and resumes or
  terminates per its contract.

**Oracle extension.** The host script adds:

- workflow runs stopped mid-progress;
- conditional and temporal operations;
- managed cleanup;
- owned-async requests;
- workflow-bearing transitions.

It interrupts at every durable boundary, including between workflow steps.

**Docs.**

- The workflow and conditional-operation guides.
- The capability index rows for managed and long-running work.

**Exit criteria.**

- One full lane run on the landing head, with every lane green.
- The cumulative oracle passes once.
- An independent reviewer certifies.

---

## Phase 8: External dispatch and custody

**Goal.** External work keeps its custody across process death. Unknown
external effects stay unknown until their owner resolves them, and completed
external effects are never repeated.

**Requirements**

- **Outbox contract on reopen.** The outbox's inbound contract is rebound from
  the installed program on reopen. It is no longer `None` after restore.
  Outbox rows, transport causal digests and runtime source commitments carry
  no process-local identity. The audit's identity table for Phase 8 lists
  them.
- **Dispatch custody.** A dispatch attempt is recorded durably in
  `query.dispatch` before the external call. Its outstanding custody,
  correlation and idempotency association survive.
- **Crash after dispatch.** If the process dies after dispatch and before the
  response is recorded, the external posture is indeterminate. That posture
  survives reopen until the owner resolves it.
- **Proven no effect.** Proven no effect (for example `PayloadRejected`) is a
  first-class durable outcome. It is distinct from unresolved, compensation
  and completed.
- **Inbound custody and deduplication** are durable: message identity,
  terminal, pending and expiry.
  - A duplicate delivery after reopen is recognized as a duplicate.
  - Terminal acknowledgement settlement is durable.
  - An authentic callback for a pre-crash dispatch completes it, rather than
    answering "unknown correlation".
- **Recovery handles.** Their claim status is a durable obligation. Recovery
  progression (compensate, reconcile, redispatch, safe retry, resolve, expiry)
  operates on recovered custody.
- **Rebuilt indexes.** The inbound terminal index and the evidence store's
  correlation index are rebuilt from durable World history and Relational
  rows.
- **Deleted postures.** The `StoreCapabilityRequired` postures are deleted
  from `WorthQueryDispatchOutboxDurabilityPosture` and
  `WorthQueryRecoveryDurabilityPosture`.
- **No exactly-once claim.** There is no claim of remote exactly-once beyond
  the external owner's admitted contract.

**Interim refusals lifted.** Dispatch and inbound completion.

**Required tests**

- **Bank external rail under application kills.** With the bank external rail
  as the external party, these cases pass while the application process is
  killed:
  - exactly-once completion;
  - lost response;
  - duplicate acknowledgement;
  - completion after timeout;
  - disappearance mid-dispatch;
  - semantic rejection, as proven no effect.
- **Crash between dispatch and response recording.** Reopen shows the
  indeterminate posture. Reconcile resolves it without redispatching a
  completed effect.
- **Duplicate inbound delivery across a reopen** is deduplicated.
- **Authentic callback after reopen** completes the original dispatch.

**Oracle extension.** The host script adds dispatching operations and inbound
completions against a deterministic in-test external party. It interrupts at
every durable boundary, both before and after the external call, and before
inbound results are recorded.

**Docs.**

- The external work and recovery guides.
- The process-death disclaimers in `http-owning-money.md` and
  `public-consumer-contract.md`, rewritten to describe the supported behavior.
- The capability index.

**Exit criteria.**

- One full lane run on the landing head, with every lane green, including the
  bank courtroom and rail suites.
- The cumulative oracle passes once.
- Independent certification.

---

## Phase 9: Correction and original-owner advancement

**Goal.** 9.18 corrections and their retained material survive restart, and
the original owner keeps editing after reopen.

**Requirements**

- **Lineage readmission.** Correction occurrences, alternatives, causal links
  and lineage are Relational causality rows. They readmit against durable
  World identities, which they record as home-scoped identity since Phase 4.
- **Retained material.** Retained governed input, pre-images and original
  authority become durable in `query.corrections`, under their retention and
  disclosure contracts. Each is optional or an obligation, as its contract
  says. Expired optional material answers typed unavailability (D10). Recovery
  never guesses an inverse or reinterprets retained bytes.
- **Custody.**
  - Correction idempotency resolves through Phase 4's durable idempotency.
  - Required-output custody for corrections, meaning the pending publication
    registry's correction entries, is a durable obligation.
  - Authored correction and required derived settlement stay distinct,
    including recovery when only one has completed.
- **Divergent branches.** Correction meaning and the selected lineage are
  preserved through reopen. Recovery never consumes or substitutes an
  alternative from another branch.
- **Original-owner advancement contract.**
  - **The original owner** is the retained native operation owner, in the
    platform's existing sense: the continuation that fresh authenticated
    commands advance and close. Examples are the bank's
    `BankHttpMutationContinuation`, `capability-index.md` "advance and close
    each original owner", and `build-an-application.md`'s original-owner
    guidance. Today it is a caller-held process value.
  - **Durable custody.** The owner's continuation becomes durable custody in
    `query.obligations`. It records:
    - its original operation identity and its occurrence;
    - its native ownership material, meaning the aggregate identities and
      incident relation facts it governs;
    - its lifecycle position.

    Phase 7 supplies the managed and owned-async custody it builds on.
  - **After reopen.** The custody readmits, and the host issues a fresh owner
    value, bound to the new incarnation (D7), for the same original identity.
    Fresh authenticated commands advance and close it exactly as before the
    crash. The old process's value refuses with `StaleIncarnation`.
  - **A dependency edit followed by advancement.** Advancement continues the
    same original owner. Aggregate identities stay continuous under the
    existing keeper, split and merge laws (`aggregate_identity/transition.rs`):
    membership may change exactly as those laws allow, and no identity is
    re-minted outside them.
  - **What reconstruction must not do.** It never replaces the owner, and it
    never creates a new authoritative object to make the edit succeed. No new
    lineage kind is added.
  - **Missing material.** If the required ownership material is unavailable,
    advancement refuses with `OwnershipMaterialUnavailable`.

**Interim refusals lifted.** Corrections.

**Required tests**

- **Correct, kill, reopen.** Correction history, alternatives and lineage equal
  the reference.
- **Reapply after reopen** selects the recovered alternative.
- **Expired optional pre-image** answers typed unavailability.
- **Divergent branch.** A correction on a divergent branch, killed and
  reopened, keeps its branch's lineage.
- **Split completion.** A correction killed with the authored correction
  complete but derived settlement pending recovers that exact split.
- **Advancement after reopen.** An original owner is held across a kill and
  reopen. A fresh authenticated command, then a dependency edit, then
  advancement and close of the original owner, all match the reference:
  aggregate identities, membership and incident relations are equal, and no
  identity is re-minted outside the keeper, split and merge laws.
- **Stale owner value.** The pre-crash owner value refuses with
  `StaleIncarnation`.
- **Missing ownership material** refuses with
  `OwnershipMaterialUnavailable`.

**Oracle extension.** The host script adds corrections, reapplies and
divergent-branch corrections, and a dependency edit plus original-owner
advancement after a reopen.

**Docs.**

- The correction guide.
- The capability index rows.

**Exit criteria.**

- One full lane run on the landing head, with every lane green.
- The cumulative oracle passes once.
- An independent reviewer certifies.

---

## Phase 10: Public cutover and milestone certification

**Goal.** One supported durable path, honest public guidance, and
certification of the whole milestone.

**Requirements**

- **Retire the interim checkpoint path (D13).** Delete it entirely:
  - capture;
  - byte transport;
  - the `*_from_checkpoint*` constructors;
  - `repair_to_checkpoint*`;
  - Relational's embedded-checkpoint placement.

  Its callers move to the open call.
- **Delete `UnsupportedDurableFamily`.** Every family in the roadmap's
  Milestone 3 requirements is supported. No family within that set remains
  refused.
- **Re-audit every Phase 1 family against the code.** Confirm three things:
  - no competing durable publication or recovery path remains;
  - no durable row records a process-local identity;
  - no capability claim overstates what reopening resumes.
- **Update every ordinary guide** with supported restart behavior, failures
  and valid next actions:
  - the application, workflow, external recovery and correction guides;
  - the host README;
  - the capability index, including "Known gaps".
- **Executable examples** use the public host and its real owners:
  `observable_workflow`, `authored_workflow`, `shared_execution` and the bank
  world.

**Required tests**

- **The full journey,** on the production host and filesystem media, in one
  run:
  1. mixed-owner publication;
  2. a deferred settlement;
  3. a workflow mid-run;
  4. an ambiguous external completion;
  5. a correction on a divergent branch;
  6. kill and reopen;
  7. recovery of original occurrences and custody, without repeated effects;
  8. a dependency edit, then advancing the original owner.
- **No caller of a deleted path remains.** Boundary-check enforces this.

**Oracle.** The cumulative oracle runs once, exhaustively.

- Generated sequences mix every family, interrupt at every durable boundary,
  and reopen in a fresh process.
- Build-identity change and discarded derived state are dimensions across the
  whole run.
- Recovery interruption runs at every stage.

**Completion.** Milestone 3 is complete when:

1. **One durable home.** The host opens one durable home. No
   application-owned checkpoint transport, in-memory receipt authority or
   parallel retry cache remains.
2. **Every audited family is accounted for.** Each is either durable, or
   process-local with reacquisition through fresh authority that is
   documented and tested.
3. **Journeys pass.** The decisive journey (Phase 4) and the full journey
   (Phase 10) pass on both sides of the first home checkpoint.
4. **Oracle passes.** The cumulative oracle's generated sequences pass across
   build-identity change and discarded derived state.
5. **Every lane is green.**
   - Not merely free of new failures. The lanes are Relational, World, Signal
     (including wasm), Bridge, Server, Query (library, integration, examples,
     doc and fixture gates), bank world and Store, plus boundary-check,
     agent-context, clippy, formatting and line caps.
   - Failures that existed before the milestone are fixed honestly.
6. **Certified.** An independent reviewer certifies the milestone against this
   specification and the roadmap.

## Out of scope: worth-ui

`workspaces/worth-ui` is not ready, so it is left out of this milestone
entirely. No phase migrates `worth-ui-query-binding`, and no landing gate
includes the UI lanes: the `ui`, `check-ui-workspace` and `clippy-ui-binding`
steps of `query_gate.py`. The binding stops compiling against the host once
Phase 3 replaces the `in_memory*` constructors. When worth-ui is picked up,
its own work adopts the open call; this milestone adds no shim for it.

## Verification discipline (every phase)

- **Within a phase:** run only the lanes the diff can affect, once, at the end
  of each slice. Run only the oracle cases that the diff adds or changes,
  selected by name.
- **At each landing:**
  - exactly one full lane run and one cumulative oracle run, on the final
    head;
  - evidence logs record the command, HEAD, a clean-tree flag, times and the
    exit code;
  - reviewers verify the evidence and probe; they do not rerun whole suites.
- **Merging to master:** each landing merges the current master in as a
  reviewed step, then fast-forwards master. Nothing is ever force-pushed.
