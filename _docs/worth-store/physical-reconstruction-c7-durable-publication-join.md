# C.7: WAL, Checkpoint, Root Publication, And Physical Acknowledgment Join

## Goal

Install one canonical, Store-owned physical durability progression that binds
each admitted mutation to its WAL range, required backend barriers, page or
extent effects, pageLSNs, checkpoint or current-root publication, namespace
durability, exact terminal fate, and physical acknowledgment.

C.7 is complete only when the wrong order is unavailable to ordinary code and
when every surviving acknowledgment proves the exact effects required by the
admitted backend profile for the exact physical mutation identity.

## Current Verification Policy

C.7 behavior is protected by current direct Cargo tests and repository gates.
Generated proof ledgers, source bindings, mutation catalogs, evidence bundles,
and report pipelines are retired and must not be recreated. Git preserves the
implementation history.

## Why This Milestone Exists

C.4 through C.6 established the real media boundary, file-backed artifacts,
physical work topology, scheduler/executor separation, bounded residency, dirty
typestate, and exact writeback settlement. WAL, durability-ordering,
checkpoint, pageLSN, and root-publication mechanisms also exist in lower
crates. They are not yet one database durability law.

The most dangerous implementation is not an obviously broken WAL. It is a
well-written coordinator that calls individually valid mechanisms in a
plausible order, compares their receipts afterward, and returns success. Such a
system may pass every local test while:

- combining proofs from different operations, generations, scopes, or files;
- treating a WAL-only receipt as complete physical acknowledgment;
- letting a `WalCommit` label substitute for a WAL-before-data dependency;
- collapsing several mutations into one group-commit identity;
- retrying an effect whose physical fate is unknown;
- marking a C.6 frame clean before its exact data effect settles;
- treating rename completion as durable namespace publication; or
- leaving an isolated WAL, checkpoint, or root writer beside the canonical
  Store path.

C.7 is therefore an authority and causal-identity join, not a new mechanism
demo and not an orchestration wrapper around the current islands.

## Roadmap Placement And Inherited Truth

C.7 consumes:

- C.4's admitted filesystem owner, stable Store namespace, exact media
  operation identities, backend capability evidence, file synchronization,
  directory synchronization, atomic replacement, and indeterminate media
  outcomes;
- C.5's canonical page, segment, extent, manifest, catalog, stable record
  identity, current-root generation, copy-on-write candidate, and bounded
  physical format;
- C.5.1's Store-owned physical operation identity, derived Signal readiness,
  scheduler admission, executor-only media access, exact effect settlement,
  cancellation law, runtime generation, and terminal fate; and
- C.6's admitted dirty frame, exclusive writeback claim,
  `PhysicalWritebackSettlement`, bounded allocation scopes, frame generation,
  pressure evidence, and rule that dirty order is not durability order.

C.7 may consume those truths. It may not reconstruct or replace them.

C.7 precedes C.8 because recovery requires one real durability law. C.7 must
leave C.8 enough persisted truth to determine recovery source precedence and
resolve indeterminate operations, but C.7 does not itself claim fresh-process
recovery.

## Governing Boundary

C.7 owns physical durability ordering and physical acknowledgment.

It does not own:

- semantic MVCC, transaction visibility, or conflict resolution;
- branch identity, branch-head advancement, or branch-writer release;
- Query acknowledgment or semantic commit;
- buffer-pool construction, eviction, frame cleaning, or residency policy;
- backend capability invention;
- recovery source precedence or redo;
- stable-reader retention and reclaim policy; or
- integrity admission and corruption classification.

A physical acknowledgment means only:

> The exact mutation identity reached every physical durability edge required
> by its admitted request and backend profile, and every constituent proof is
> causally bound to that identity.

Operations that share a durability barrier do not thereby acquire semantic
order, shared mutation identity, or shared acknowledgment.

## Adversarial Constraint

The canonical durability path must survive this world:

> A real Store at least 32 times larger than its admitted resident-memory
> budget accepts independent physical mutations while three mutations share a
> group-commit opportunity, dirty eviction and exact writeback continue, a
> fuzzy checkpoint captures an exact source range, the WAL rotates at least
> twice, the bounded WAL-tail limit is reached, one caller cancels, one
> completion is delayed, and one process is killed at a named persistence
> seam. Workload and lawful scheduling are independently seeded. A fresh
> offline process receives only the Store root, stable format declarations,
> admitted backend profile, and expected scenario identity. It must prove that
> no persisted page outruns durable WAL, no root is promoted without its exact
> namespace barriers, no member identity is collapsed into its group, no
> ambiguous effect is labeled success or no-effect, and no bounded-memory or
> queue claim escapes into unbounded background work.

The system fails this milestone if the clean run is correct but any
individually valid receipt can be substituted, any effect can bypass the
C.5.1 topology, or any crash result can be explained only from writer-owned
memory, counters, logs, or serialized runtime state.

## Decisive Durability Scenario

### Production subject

The production subject is the ordinary `ServingPhysicalRuntime` record
submission and checkpoint facades after C.7 cutover.

The call path must include:

```text
worth-store public physical facade
  -> Store-owned durability admission and identity
  -> worth-store-wal frame/range declaration
  -> Store-owned physical Signal readiness
  -> worth-store-io-scheduler resource admission
  -> Store executor
  -> C.4 filesystem media owner
  -> worth-store-physical-backend effect receipt
  -> C.6 dirty/writeback settlement where data frames participate
  -> Store-owned root/checkpoint publication settlement
  -> physical terminal outcome
```

`worth-store-wal`, `worth-store-recovery-physics`,
`worth-store-physical-backend`, `worth-store-buffer-pool`, Signal, and the
scheduler remain specialized participants. None may expose a second ordinary
end-to-end execution path.

The harness may:

- choose a declared production yieldpoint;
- perturb simultaneously lawful harness decisions;
- terminate a child process without cleanup;
- wrap the C.4 media observation boundary;
- invoke the read-only offline inspector; and
- materialize terminal evidence after the run.

It may not mint production receipts, edit private runtime state, mutate files
after the declared crash, supply recovered state, or treat a simulated error as
process death.

### Process roles

The scenario retains the C.6 fresh-process role separation:

1. **Producer** initializes the Store and writes the deterministic baseline.
2. **Serving writer** opens the Store through the ordinary facade, executes the
   durability workload, and is the only process that may be killed.
3. **Offline durability inspector** opens no Store runtime and parses WAL,
   page, checkpoint, manifest, catalog, and root artifacts through stable
   physical declarations only.
4. **Fresh physical reopener** attempts the ordinary physical open posture
   allowed before C.8 and reports discovered publication/residue facts without
   performing C.8 recovery or receiving writer heap state.

Each role records distinct process, binary, source, protocol, Store, and
runtime identities. The runner rejects role reuse.

### Initial world

The initial world fixes:

- a real, named filesystem and backend durability profile;
- one Store root initialized by the producer process;
- a Store data set at least 32 times the resident-byte budget;
- a page size and WAL segment size that force multiple pages and at least two
  WAL rotations during the workload;
- a bounded dirty-frame budget and a bounded checkpoint WAL-tail budget;
- four distinct physical mutation requests, three of which become eligible for
  one shared barrier while one remains cancellable before group sealing;
- stable caller-supplied physical idempotency material bound to Store-issued
  retention leases, with no branch meaning;
- one current and one retainable previous physical root generation;
- separate workload and schedule seeds;
- exact media, WAL, page, checkpoint, root, allocation, queue, and
  acknowledgment expectations derived before the serving process starts; and
- absence of every staged WAL, checkpoint, root, or temporary artifact that the
  scenario has not yet lawfully created.

The expected operation identities and bytes are generated independently of the
runtime's classifiers and output projection.

### Crash-seam matrix

The serving process is killed against an independently produced fresh Store
root at each exact seam:

1. before WAL append;
2. during WAL append after a strict nonzero prefix but before the complete
   frame;
3. after complete WAL bytes are written but before the required WAL barrier;
4. after the required WAL barrier but before data dispatch;
5. during a data write after a strict nonzero prefix;
6. after data settlement but before current-root publication;
7. after atomic current-root replacement but before required parent-namespace
   durability; and
8. after complete physical durability but before acknowledgment is observed by
   the caller.

Each scenario changes only the crash seam. The producer deterministically
creates a new Store root from the same workload seed before each serving
process begins. A distinct fixture must not preconstruct its expected answer.
No scenario may copy a post-run Store root, workspace tree, archive or zip,
serialized runtime, derived cache, or writer memory into another seam.

### Group-commit identity blender

Four requests attack the boundary between candidate admission and sealed group
membership:

- one is cancelled while still proven pre-effect, before group sealing and WAL
  range reservation, and never becomes a group member;
- one sealed member completes and returns its physical acknowledgment;
- one sealed member completes physically but loses caller observation at the
  final seam; and
- one sealed member has data settlement and completion delivery delayed while
  the other sealed members settle, then returns its own acknowledgment.

The schedule may reorder member admission, WAL-range allocation, scheduler
dispatch, data settlement, and completion delivery only where prerequisites
permit.

The direct scenario requires:

- one exact shared barrier execution;
- one exact shared root publication when the sealed group plan declares it;
- four permanently distinct operation and idempotency identities;
- exactly three sealed members;
- one exact WAL subrange and data-effect set per sealed member;
- no WAL range, data effect, or group membership for the pre-seal cancelled
  request;
- no group-level terminal fate;
- no cancellation or acknowledgment propagation between members;
- no semantic order inferred from group position;
- two independently completed acknowledged members;
- one proven-no-effect pre-seal cancelled request; and
- one completed-but-unobserved sealed-member fact left for C.8 reconciliation.

### Checkpoint pressure siege

While foreground mutations continue:

- begin one fuzzy checkpoint from an exact admitted source range;
- keep dirtying pages while capture advances;
- force at least one dirty eviction and exact C.6 writeback;
- rotate the WAL at least twice;
- reach the declared retained-tail limit;
- submit one additional mutation that cannot be admitted without exceeding a
  queue, dirty, or retained-tail bound;
- publish the checkpoint candidate;
- attempt premature WAL deletion or recycling; and
- retain the current and immediately previous root bases required by the
  declared C.8 handoff.

The runtime must reject or backpressure before violating a bound. It may reduce
throughput. It may not stop mutation authority for the entire capture, allocate
the complete Store, widen a budget, discard required WAL, or acknowledge a
mutation from checkpoint progress.

### Independent observation

The offline inspector independently records:

- complete and torn WAL frames;
- segment and WAL generations;
- exact written and durable-prefix ranges under the named profile;
- page and extent identities, bytes, generations, and stored pageLSNs;
- checkpoint identity, source range, covered LSN range, and publication
  posture;
- current, previous, candidate, staged, and forbidden root artifacts;
- atomic replacement and parent-namespace durability observations available at
  the admitted media boundary;
- retained and prematurely missing WAL segments;
- per-operation idempotency identity and physical effect correlation;
- exact file paths, lengths, offsets, and digests; and
- absence of runtime, Signal, scheduler, pool, counter, or writer-memory
  authority in its process.

The independent oracle may share canonical byte grammar and stable identity
types. It may not call the runtime's durability classifier, settlement join,
recovery-source selector, or evidence projector.

### Required assertions

The direct scenario must prove:

- every acknowledged mutation has one exact complete WAL frame and admitted
  durable WAL basis;
- every persisted pageLSN equals the greatest exact redo-record LSN in its newly
  applied delta, its certified prior-page digest and resulting payload digest
  match, and independently observed durable WAL covers the complete new delta;
- no data effect dispatches before its matching WAL barrier proof;
- every WAL, barrier, checkpoint, root, and reused ExactWriteback effect joins
  its exact installed Signal basis, scheduler admission, executor effect, and
  Store settlement;
- no frame becomes clean without its matching C.6 effect and writeback
  settlement;
- every root or checkpoint publication consumes exact data and WAL bases;
- rename completion alone never satisfies a profile requiring directory or
  parent-namespace durability;
- group sharing reduces barrier executions without reducing member
  cardinality;
- physically disjoint mutation preparation and data work progress concurrently,
  coordinating only at exact WAL allocation/barrier and current-root cutover
  scopes;
- no receipt from another member, Store, runtime, generation, artifact, range,
  profile, or effect can settle the target mutation;
- a duplicate idempotency key with the same fingerprint performs no second
  effect, while the same key with a different fingerprint is denied before
  effect;
- cancellation before effect yields proven no effect;
- a complete authoritative WAL member prevents aggregate proven-no-effect even
  when data dispatch has not begun;
- cancellation, timeout, or caller loss after possible effect cannot erase
  settlement and cannot become automatic retry authority;
- no partially settled member or group becomes reachable from the current root;
- torn WAL and data writes remain distinguishable from complete effects;
- stale completions fail at their first Store consumer;
- checkpoint work remains within exact capture, memory, queue, and WAL-tail
  bounds;
- WAL retention consumes a covering checkpoint and contiguous-tail proof;
- the current and retained previous root facts are sufficient inputs for C.8
  without claiming that C.7 recovered them;
- ordinary close drains every durability, barrier, checkpoint, publication,
  and acknowledgment obligation;
- abrupt death leaves only classified persisted residue; and
- no ordinary direct WAL, durability-runtime, checkpoint, or root writer
  remains outside the Store progression.

### Schedule perturbation and exact replay

C.7 extends the one C.6 schedule-perturbation harness. It does not create a
second scheduler, seed type, trace grammar, or replay protocol.

The replay identity contains:

```text
source revision
binary identity
workload seed
schedule seed
crash seam
backend profile
checkpoint policy identity
scale tier
scenario identity
```

The closed C.7 decision vocabulary may perturb only simultaneously lawful
choices:

- group-member admission;
- WAL-range reservation among admitted members;
- WAL append dispatch;
- barrier completion delivery;
- data-effect dispatch;
- dirty-writeback opportunity;
- checkpoint capture slice;
- root-publication opportunity;
- pre-effect cancellation delivery; and
- terminal completion delivery.

It may not reorder prerequisites, sleep to manufacture races, alter the
production scheduler, or select the crash seam implicitly. The seam is an
explicit scenario input so identical replay inputs select an identical kill
boundary.

CI runs a bounded set of deterministic direct scenarios chosen for distinct
causal coverage. Release qualification runs the canonical schedule at every
named crash seam. A failure prints the seed needed to replay that same direct
scenario; no fixed seed-count API or report matrix is required.

Every failure emits one exact rerun command.

### Regression sensitivity

The initial corpus contains a causal controlled defect for each disputed edge:

- acknowledge before the required WAL barrier;
- dispatch data before matching WAL durability;
- persist a pageLSN ahead of durable WAL;
- choose an unrelated but range-contained pageLSN or omit one applied redo
  record from the new page delta;
- substitute the certified prior-page identity, pageLSN, or digest;
- carry a page's lifetime redo history through the ordinary path;
- collapse group-member identities;
- substitute another member's otherwise valid receipt;
- execute a duplicate same-fingerprint idempotency request twice;
- accept one idempotency key with a different physical fingerprint;
- silently reuse an expired idempotency key;
- reclaim the last authoritative attempt binding or retain every historical
  terminal binding;
- include allocated operation, group, WAL-member, or WAL-range identity in the
  request fingerprint so a lawful retry conflicts with itself;
- omit effect-relevant policy, scope, payload, operation-family, or security
  basis from request-fingerprint equivalence;
- omit the required file synchronization;
- omit the required directory or parent-namespace synchronization;
- treat atomic replacement as durable root publication;
- clean a frame before exact C.6 settlement;
- automatically retry an indeterminate effect;
- make caller-handle drop cancel, abandon settlement, or evade close draining;
- accept a stale runtime, Store, WAL, checkpoint, or root generation;
- recycle WAL without a covering checkpoint and contiguous retained tail;
- bypass C.5.1 with one direct media effect;
- misbind or collapse one installed C.7 Signal work family or aspect partition;
- substitute a Foundational policy or performance receipt for scheduler
  admission or completed physical effect outside its exact declared role;
- accept a `StoreExecutedBoundaryReceiptEvidence` projection back into the
  authoritative mutation progression;
- ignore checkpoint-tail or queue admission;
- stop the schedule seed from affecting one declared decision;
- derive every schedule seed from unrelated mutable bookkeeping;
- preserve a competing ordinary WAL, checkpoint, or root execution lane; and
- serialize the complete mutation lifecycle under one whole-Store lock.

Each controlled defect is represented as a direct hostile input or test case at
its nearest real production boundary. Tests do not rewrite source, emit mutation
receipts, or require a catalog-derived campaign. Every later C.7 bug correction
adds the smallest direct regression test that would have exposed it.

### Mechanical anti-substitution gates

The milestone must mechanically reject:

- a WAL-only acknowledgment used as final C.7 acknowledgment;
- a durability request or counter used as completed-effect proof;
- a group barrier used as a member identity or member terminal outcome;
- a fixed-arity join, tuple, or optional slots used for a runtime-width group;
- a raw LSN comparison used to authorize data dispatch;
- request equivalence derived from allocated attempt or WAL identity;
- a Foundational profile, policy receipt, performance receipt, or executed
  evidence projection used as backend admission or physical authority;
- a direct `StoreDurabilityRuntime`, WAL executor, checkpoint writer, root log,
  or filesystem call reachable from the ordinary facade;
- Signal evaluation performing or authorizing durability effects;
- scheduler completion settling Store truth;
- C.7 importing Query, Relational, Runtime Bridge, branch, MVCC, or semantic
  writer authority;
- C.7 reaching into buffer-pool construction, frame-table, eviction, or clean
  transition internals;
- certification, replay, model, JSON, logs, or evidence bundles entering the
  ordinary progression;
- same-process crash inspection;
- graceful close standing in for abrupt death;
- an in-memory WAL or mock filesystem satisfying a physical claim; and
- a feature combination that reintroduces any removed ordinary execution
  route.

## Product Decision Lock

1. `worth-store` owns the one end-to-end physical durability progression.
   Lower crates own meaning or mechanics, not cross-owner orchestration.
2. `worth-store-wal` owns WAL frame grammar, LSN topology, append planning,
   bounded scan, and stable WAL identities. It performs no ordinary OS effect.
3. The C.4 filesystem owner remains the only ordinary media-effect boundary.
   Backend durability types declare and report admitted physical mechanics;
   they do not own mutation acknowledgment.
4. `worth-store-recovery-physics` retains recovery-facing WAL, checkpoint, tail,
   and crash-basis meaning. Its direct ordinary WAL execution path is removed.
5. C.6 dirty state proves a frame requires settlement. It never proves WAL
   order, durability, or acknowledgment.
6. The Store assigns each admitted mutation one physical operation identity and
   binds it to one caller idempotency key and canonical request fingerprint.
   Later operation, group, member, and WAL allocation live only in the
   persisted attempt binding. None of those identities may replace another or
   feed back into request equivalence.
7. Group commit may share the exact WAL barrier and one exact root publication
   for a sealed member set. Shared effects derive matching member proofs for
   every included mutation. Grouping never merges operation identity, WAL
   subrange, data effect, terminal fate, cancellation, recovery posture, or
   acknowledgment.
8. WAL range reservation is not WAL durability. WAL bytes written are not WAL
   durability. File synchronization is not namespace durability. Rename is not
   parent-directory durability. Each is a distinct proof state.
9. Data dispatch consumes the exact matching WAL-durable member proof. A
   `WalCommit` enum, queue class, boolean, counter, or raw LSN cannot substitute.
10. PageLSN meaning is split lawfully: WAL owns LSN identity and order,
    physical format owns encoded pageLSN bytes, Store owns their exact mutation
    join, and C.8 recovery will own comparison and redo policy.
11. Physical acknowledgment is constructed only from the strongest completed
    Store progression. Existing locally honest acknowledgment types are narrowed
    to their actual boundary or removed; aliases are forbidden.
12. A complete authoritative WAL member is already a physical effect and a
    durable continuation obligation. It prevents aggregate `ProvenNoEffect`
    even when no data page has yet changed.
13. Once an effect may have begun, cancellation and timeout cannot mean
    rollback. Settlement continues to completed or indeterminate physical fate.
14. Automatic retry requires proven no effect and the original exact
    idempotency identity. Indeterminate work opens no retry door before C.8
    reconciliation.
15. Every physical idempotency identity includes a Store-issued
    `PhysicalMutationIdempotencyLease` whose expiry is expressed in durable
    checkpoint generations. The admitted policy fixes one nonzero finite
    retention-generation count, one nonzero pending-unresolved bound, and one
    independent nonzero total-live-binding bound. Attempt deadlines and ambient
    time do not alter that lease. Unresolved bindings never expire; terminal
    bindings remain
    authoritative until lease expiry and at least one later namespace-durable
    checkpoint compacts their fate. An expired identity is denied and requires
    a newly issued key; it never silently creates another attempt.
16. Checkpoint capture is fuzzy or non-blocking, carries an exact source range,
    and has hard memory, queue, and retained-WAL bounds. Whole-Store capture and
    unbounded tail growth are forbidden.
17. Root publication retains one canonical current-root authority. Any
    publication history needed by C.8 or C.10 derives from that progression; a
    second root authority or `root-publications.log` writer is forbidden.
18. The immediately previous root and required WAL/checkpoint bases remain
    physically retained until a later owner proves reclamation eligibility.
    C.7 does not infer semantic liveness.
19. Worth Signal derives dependency readiness and generic async lifecycle only.
    The scheduler admits resources only. The Store decides order and settlement;
    the executor alone performs effects.
20. `worth-proof` supplies outcome, progression, basis, and structural
    collection mechanics. C.4 owns platform durability semantics and exact
    capability claims; Store owns their sealed admission join and physical
    progression. A copied proof, digest, generic marker, or declaration cannot
    become admission or completed-effect evidence.
21. Worth Foundational and Store aspect-native contracts retain semantic aspect
    identity, derived routing bases, scheduler policy admission, and
    descriptive evidence projection. They do not own backend profile,
    physical fate, current root, or acknowledgment. No generic `Durable`,
    `Committed`, or `Published` Foundational flag becomes physical authority.
22. JSON exists only at the terminal evidence projection or an explicitly named
    external compatibility edge. It is forbidden from mutation admission,
    progression, Signal binding, scheduling, settlement, or acknowledgment.
23. C.7 adds no branch registry, branch queue, semantic transaction id, semantic
    commit receipt, MVCC generation, or global semantic lock.
24. C.7 may serialize the exact WAL-allocation, barrier, and current-root
    cutover scopes. It may not hold a whole-Store mutation lock across framing,
    data work, checkpoint capture, or caller observation; physically disjoint
    work remains concurrently admissible.
25. The product is unreleased. Cutover deletes obsolete paths, aliases,
    compatibility adapters, fallback executors, and legacy features in the same
    phase that replaces their last consumer.

## Semantic Vocabulary Lock

The following distinctions are normative:

- **physical mutation**: one Store-owned operation with exact scope and
  idempotency identity; never a semantic transaction;
- **idempotency lease**: Store-issued validity and retention authority expressed
  in durable checkpoint generations; never an execution deadline, wall-clock
  expiry, or completed fate;
- **request fingerprint**: versioned canonical equivalence of effect-relevant
  request inputs; never an attempt, operation, group, member, LSN, or WAL-range
  identity;
- **attempt binding**: the persisted relationship between one idempotency
  key/lease/fingerprint and its allocated physical operation, group/member, and
  WAL facts; never request equivalence or completed fate;
- **binding compaction**: namespace-durable checkpoint-side authority preserving
  unexpired terminal and every unresolved attempt binding after its original
  WAL segment becomes reclaimable; never a derived in-memory index or an
  all-history registry;
- **WAL range reserved**: stable LSN identity exists; no durability claim;
- **WAL appended**: complete bytes were observed written; required barriers may
  still be absent;
- **WAL durable**: the exact member range is covered by the admitted backend
  barrier proof;
- **data settled**: exact page/extent effects reached completed, proven-no-
  effect, or indeterminate Store fate;
- **root replaced**: atomic replacement was observed; namespace durability may
  still be absent;
- **root namespace durable**: the admitted profile's exact root-publication
  barriers completed;
- **physical acknowledgment**: the completed mutation crossed every declared
  edge and may be reported to its caller;
- **completed but unobserved**: physical completion exists but the caller did
  not receive acknowledgment;
- **indeterminate**: an effect may have occurred and neither success nor
  no-effect is lawfully proven; and
- **semantic commit**: a future Part II decision that C.7 neither represents
  nor performs.

Production names must include the physical subject where omission would invite
semantic interpretation. Generic `CommitReceipt`, `Transaction`, `Durable`,
`Published`, `Success`, `Manager`, `Coordinator`, and `Handler` names are
forbidden when the stronger physical meaning is required.

## Normative Public API

### Durability configuration admission

The serving runtime consumes one admitted physical durability configuration:

```rust
let durability_basis: PhysicalDurabilityAdmissionBasis =
    media.physical_durability_admission_basis()?;

let durability = match PhysicalDurabilityDeclaration::builder()
    .group_commit(GroupCommitLimit::new(32)?, GroupCommitDelay::new(...)?)
    .wal(PhysicalWalPolicy::segmented(
        WalSegmentByteLimit::new(...)?,
        WalSegmentInventoryLimit::new(...)?,
    ))
    .idempotency(PhysicalIdempotencyPolicy::new(
        IdempotencyRetentionGenerations::new(...)?,
        PendingUnresolvedMutationLimit::new(...)?,
        LiveIdempotencyBindingLimit::new(...)?,
    ))
    .checkpoint(PhysicalCheckpointPolicy::fuzzy(
        CheckpointMemoryLimit::new(...)?,
        RetainedWalTailLimit::new(...)?,
    ))
    .admit(durability_basis)
    .into_raw()
{
    TransitionOutcome::Success(policy) => policy,
    TransitionOutcome::Denied(denial) => return handle_policy_denial(denial),
    TransitionOutcome::Deferred(deferred) => return handle_policy_deferral(deferred),
    TransitionOutcome::Stale(stale) => return handle_stale_policy_basis(stale),
    TransitionOutcome::RebindRequired(rebind) => return rebind_policy(rebind),
    TransitionOutcome::Failed(failure) => return inspect_policy_failure(failure),
};

let serving = match media
    .initialize_record_store(PhysicalRecordInitialization::new(
        format,
        placement,
        access,
        durability,
    ))
    .into_raw()
{
    TransitionOutcome::Success(serving) => serving,
    TransitionOutcome::Denied(denial) => return handle_store_denial(denial),
    TransitionOutcome::Deferred(deferred) => return handle_store_deferral(deferred),
    TransitionOutcome::Stale(stale) => return handle_stale_store(stale),
    TransitionOutcome::RebindRequired(rebind) => return rebind_store(rebind),
    TransitionOutcome::Failed(failure) => return inspect_store_failure(failure),
};
```

The exact constructor shape may follow the final C.6 initialization facade, but
these laws are fixed:

- the declaration is configuration, not authority;
- `PhysicalDurabilityAdmissionBasis` is a sealed C.4-derived Store admission
  basis constructible only from `QualifiedFilesystemMedia`;
- the basis binds the exact `RootProfileQualificationBasis` and
  `AdmittedBackendCapabilityWitness` claims required by the declaration,
  including `Fsync`, `DirectorySync`, and `DurableRename` where the selected
  policy needs them;
- policy admission consumes that concrete basis and returns
  `ProofOutcome<AdmittedPhysicalDurabilityPolicy, ...>` with typed admitted,
  denied, deferred, stale, rebind-required, and failed branches;
- `worth-proof` supplies the outcome and progression mechanics; it does not
  define filesystem capability, durability-profile, or barrier semantics;
- unsupported, stale, mismatched, or insufficient profiles return typed
  denial before runtime construction;
- required limits are nonzero, finite, and explicit;
- no runtime opens with an optional durability owner;
- no `bool` selects weakened durability; and
- adding the C.7 owner breaks every incomplete initialize, open, close, abort,
  and observation construction site.

The admitted policy carries the fingerprint of its consumed C.4 admission
basis. Runtime initialization verifies that fingerprint against the same
`QualifiedFilesystemMedia` generation before creating any owner. Copying
capability names, profile labels, basis digests, or individual claims cannot
construct the sealed basis or satisfy this join.

### Ordinary physical mutation

The common path requests the strongest admitted ordinary physical boundary:

```rust
let idempotency = serving
    .record_submission()
    .issue_idempotency_key(caller_key)?;
let deadline = PhysicalMutationDeadline::after_milliseconds(deadline_ms)
    .ok_or(InvalidMutationDeadline)?;

let request = PhysicalMutationRequest::platform_durable(idempotency, deadline);

let prepared = match serving
    .record_submission()
    .prepare_durable_append(
        RecordAppendBatch::try_from_iter(records)?,
        placement,
        request,
    )
    .into_raw()
{
    TransitionOutcome::Success(prepared) => prepared,
    TransitionOutcome::Denied(denial) => return handle_mutation_denial(denial),
    TransitionOutcome::Deferred(deferred) => return handle_mutation_deferral(deferred),
    TransitionOutcome::Stale(stale) => return handle_stale_mutation(stale),
    TransitionOutcome::RebindRequired(rebind) => return rebind_mutation(rebind),
    TransitionOutcome::Failed(failure) => return inspect_mutation_failure(failure),
};

let mutation: PhysicalMutationHandle = prepared.start();

match mutation.wait() {
    PhysicalMutationOutcome::Completed(completed) => {
        let acknowledgment: PhysicalMutationAcknowledgment =
            completed.into_acknowledgment();
        use_physical_fact(acknowledgment);
    }
    PhysicalMutationOutcome::ProvenNoEffect(no_effect) => {
        handle_retryable_physical_fate(no_effect);
    }
    PhysicalMutationOutcome::Indeterminate(indeterminate) => {
        preserve_for_recovery(indeterminate);
    }
}
```

This example fixes the public semantics, not every private method name.

- Preparation validates scope, idempotency, limits, health, Store generation,
  profile, and resource shape before effects.
- Preparation returns
  `ProofOutcome<PreparedPhysicalMutation, ...>` and preserves distinct denied,
  deferred, stale, rebind-required, and failed branches. It does not compress
  them into one error or convert admission failure into physical effect fate.
- `PreparedPhysicalMutation` is consuming and non-`Clone`.
- `PreparedPhysicalMutation::start(self)` returns one
  `PhysicalMutationHandle`; an optional blocking `execute(self)` is only a
  transparent `start().wait()` convenience.
- `PhysicalMutationHandle` exposes immutable mutation/idempotency identity,
  typed `poll`, consuming `wait`, and `request_cancellation` operations.
- Dropping the handle abandons caller observation only. It never requests
  cancellation, changes effect fate, releases Store settlement ownership, or
  permits a second attempt.
- `request_cancellation` returns a typed outcome distinguishing pre-effect
  cancellation accepted, settlement already effectful, already terminal, stale
  handle, and runtime closing. It cannot report aggregate proven-no-effect once
  a complete WAL member exists.
- The Store runtime owns every started mutation until terminal settlement.
  Managed close stops new admission and drains started work even when every
  caller handle has been dropped.
- Deadlines are evaluated through the admitted C.5.1/Signal monotonic clock
  basis; wall clock, sleep duration, and ambient process time are not
  correctness inputs.
- Execution returns a typed physical fate rather than `Result<Success, Error>`
  for effectful uncertainty.
- Only `CompletedPhysicalMutation` exposes
  `into_acknowledgment`.
- `ProvenNoEffectPhysicalMutation` carries the exact denial, cancellation,
  timeout, or safe-retry basis and original idempotency identity.
- `IndeterminatePhysicalMutation` carries the exact persisted-effect and
  inspection basis required by C.8 and exposes no acknowledgment or automatic
  retry method.
- Opaque upstream correlation may be retained as metadata but cannot affect
  physical scope, grouping, scheduling, ordering, or fate.

The existing `append_batch(...)->PublishedRecordBatch` convenience is replaced
or narrowed so it cannot hide durability request, idempotency, cancellation,
indeterminate fate, or contractual cost. No compatibility wrapper retains the
old stronger-looking behavior.

### Idempotency

The first admitted use of a `PhysicalMutationIdempotencyKey` binds it to one
`PhysicalMutationRequestFingerprint`. That fingerprint is canonical input
equivalence and contains:

- Store identity and admitted durability-policy basis identity;
- exact physical scope;
- immutable payload digest;
- durability request;
- operation-family identity; and
- every security or authority basis that can change the lawful physical effect.

It excludes attempt-local facts: idempotency lease and its issuance/expiry
frontiers, runtime generation, operation identity, group identity, WAL member
or range, queue placement, schedule choice, deadline, cancellation state,
completion-delivery correlation, and observation metadata. Those facts can
differ across separately keyed lawful requests without changing the requested
physical mutation; exact retry identity still requires the original complete
idempotency key.

The fingerprint uses the Store aspect-native
`StoreDigestEquivalenceBasis` and a new admitted
`StoreCanonicalBasisFamily::PhysicalMutationRequestFingerprint`. Version 1
freezes field order, canonical encoding, domain separator
`store.physical.mutation.request-fingerprint.v1`, and SHA-256. A future
algorithm or encoding change creates a new version; it never reinterprets an
existing fingerprint. Terminal JSON, debug formatting, WAL bytes, an allocated
LSN, or a caller-supplied digest is not a lawful fingerprint source.

The closed canonical registry is extended honestly rather than reusing
`WalRecord` or an evidence family:

- source kind: `StoreCanonicalBasisSourceKind::StorePhysicalMutationRequest`;
- field role:
  `StoreCanonicalBasisFieldRole::NativePhysicalMutationRequest`;
- lane: `StoreCanonicalBasisLane::PhysicalMutation`; and
- family:
  `StoreCanonicalBasisFamily::PhysicalMutationRequestFingerprint`.

The source is a validated native Store request-basis record, not a terminal
digest string, compatibility text, debug object, or raw JSON payload.

Public key issuance binds validated caller material to the current
`PhysicalMutationIdempotencyLease`. The resulting
`PhysicalMutationIdempotencyKey` carries Store identity, issuance checkpoint
generation, expiry checkpoint generation, and opaque caller material. Callers
must preserve that complete key for retry; copied raw caller material cannot
reconstruct an expired identity.

Namespace-durable Store initialization defines idempotency-retention generation
zero. Each later namespace-durable checkpoint advances that generation exactly
once; a failed, staged, renamed-but-not-namespace-durable, or copied checkpoint
cannot advance lease issuance or expiry.

Allocation creates a distinct `PhysicalMutationAttemptBinding` containing:

- the idempotency key, lease, and request fingerprint;
- the Store/runtime and physical operation identities;
- any sealed group-member identity; and
- the later exact WAL member identity and range.

The binding, not the request fingerprint, gains allocation facts. Its canonical
representation is encoded in the WAL member so fresh-process inspection can
recover the same retry relationship. An in-memory map may accelerate lookup
during one runtime generation, but it is derived, bounded, and disposable. It
is never the only idempotency authority.

The admitted policy fixes a nonzero finite
`idempotency_retention_generations`. A terminal binding remains live until both
its lease expiry frontier is namespace durable and its terminal fate has
appeared in at least one later namespace-durable binding compaction. An
unresolved binding remains live regardless of nominal expiry and counts against
the admitted pending-unresolved bound. Reaching that bound backpressures or
denies new mutation admission; it cannot grow retained authority without limit.

Every namespace-durable checkpoint publishes a
`PhysicalMutationBindingCompaction` containing exactly:

- all live unresolved bindings and their inspection bases;
- all unexpired terminal bindings and exact terminal fates; and
- any expired terminal binding not yet protected by the required later
  namespace-durable checkpoint.

The compaction is authoritative persisted input to duplicate admission and C.8.
The in-memory idempotency registry is rebuilt from the latest admitted
compaction, or the empty generation-zero basis before the first checkpoint,
plus the retained WAL tail. It remains a bounded derived index.

A duplicate request with the same key and fingerprint:

- joins or observes the existing in-flight attempt without a second effect;
- returns the already completed physical fact when lawfully available; or
- returns an exact inspection/reconciliation posture when fate is unresolved.

A duplicate key with a different fingerprint is denied before effects as an
idempotency conflict. It cannot silently select the first payload, replace it,
or create another WAL member.

Presenting an expired key returns the exact pre-effect
`PhysicalMutationIdempotencyExpired` admission denial. The caller must obtain a
new idempotency key for any newly intended mutation. The Store never treats
expired key material as an implicit retry or silently rebinds it to a new
attempt.

A request fingerprint that includes its allocated attempt, operation, group,
or WAL identity is invalid by construction: it would make the same retry
unequal to itself. Conversely, a fingerprint that omits effect-relevant scope,
durability policy, payload, operation family, or security basis is incomplete
and cannot enter admission.

After a complete WAL member exists, cancellation or failure cannot produce the
aggregate `ProvenNoEffect` variant. The Store must finish the same in-process
obligation where possible or preserve one `IndeterminatePhysicalMutation` for
C.8. Resubmission must never append a second member merely because data or
acknowledgment was incomplete.

### Checkpoint lifecycle

Checkpoint work is a managed resource:

```rust
let checkpoint = match serving
    .checkpoints()
    .start(PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new(key)?,
        PhysicalCheckpointDeadline::after_milliseconds(deadline_ms)
            .ok_or(InvalidCheckpointDeadline)?,
    ))
    .into_raw()
{
    TransitionOutcome::Success(handle) => handle,
    TransitionOutcome::Denied(denial) => return handle_checkpoint_denial(denial),
    TransitionOutcome::Deferred(deferred) => return handle_checkpoint_deferral(deferred),
    TransitionOutcome::Stale(stale) => return handle_stale_checkpoint(stale),
    TransitionOutcome::RebindRequired(rebind) => return rebind_checkpoint(rebind),
    TransitionOutcome::Failed(failure) => return inspect_checkpoint_failure(failure),
};

observe(checkpoint.progress());
let outcome = checkpoint.wait();
```

The handle exposes:

- exact checkpoint identity and admitted source range;
- capture and retained-WAL progress;
- current bounded resource use;
- cancellation safe only before publication effects begin;
- completed, proven-no-effect, and indeterminate terminal outcomes;
- finalization and disposal; and
- no raw frame, pool, WAL writer, root writer, Signal, scheduler, or media
  authority.

Checkpoint start returns `ProofOutcome<PhysicalCheckpointHandle, ...>` with the
same admitted, denied, deferred, stale, rebind-required, and failed admission
distinctions. The handle's later terminal physical fate remains Store-owned
`Completed`, `ProvenNoEffect`, or `Indeterminate`; admission outcomes and
post-effect fate are not one enum.

The framework can enumerate and terminate every live checkpoint handle during
close. Fire-and-forget checkpoint work is forbidden.

### Observation

```rust
let observation = serving.durability_observation();

assert_eq!(observation.store_identity(), serving.store_identity());
assert_eq!(observation.runtime_identity(), serving.runtime_identity());
```

Observation exposes bounded, read-only facts:

- admitted profile and policy identity;
- active and peak mutation/group/checkpoint/acknowledgment work;
- WAL frames, bytes, ranges, rotations, and barriers;
- page/extent writes and pageLSN bindings;
- checkpoint captures, covered ranges, retained tails, and denials;
- fresh-process checkpoint artifact bytes, exact bytes read, skipped dirty-body
  bytes, binding records read, retained WAL members read, and peak WAL reopen
  buffer bytes;
- root replacements, file syncs, directory syncs, and retained generations;
- completed, proven-no-effect, indeterminate, cancelled, timed-out, stale, and
  completed-but-unobserved mutation counts;
- exact physical amplification counters; and
- classified shutdown residue.

Observation cannot mint a phase, settle an effect, acknowledge a mutation,
admit a retry, delete WAL, or publish a root.

## Compiler-Visible Durability Progression

The Store progression is semantically equivalent to:

```text
AdmittedPhysicalMutation
  -> WalRangeReservedPhysicalMutation
  -> WalAppendedPhysicalMutation
  -> WalDurablePhysicalMutation
  -> DataDispatchedPhysicalMutation
  -> DataSettledPhysicalMutation
  -> RootPublicationPreparedPhysicalMutation
  -> RootReplacedPhysicalMutation
  -> RootNamespaceDurablePhysicalMutation
  -> CompletedPhysicalMutation
  -> PhysicalMutationAcknowledgment
```

Operation families that do not require every node use a distinct, exhaustively
declared progression. They do not skip phases through `Option`, flags, or
runtime branching on a generic state object.

Each phase:

- consumes the exact preceding type;
- preserves Store, runtime, operation, idempotency, profile, and scope identity;
- adds one stronger proof-bearing fact;
- exposes only the next legal actions;
- carries explicit cancellation and partial-effect posture; and
- cannot be constructed by callers, lower crates, Signal, scheduler,
  certification, or raw backend receipts.

The executor consumes an already lowered effect plan. It may observe facts
unavailable before execution, but it cannot rediscover grouping, WAL
dependencies, durability policy, root policy, retry policy, or acknowledgment
eligibility.

## Group Commit Contract

Group commit has a sealed group authority and separate member authority:

```text
AdmittedPhysicalDurabilityGroup
  owns: group identity, exact sealed members, grouping scope,
        shared WAL barrier requirement, optional shared root plan

WalBarrierMember<M>
  owns: exact mutation identity, idempotency identity, WAL subrange, data scope

RootPublicationMember<M>
  owns: exact mutation identity, exact included data settlement,
        exact shared successor-root membership
```

`SealedPhysicalDurabilityGroupMembers` is a Store-specific wrapper constructed
from Worth Proof `NonEmpty<WalBarrierMember<_>>`. Its consuming constructor
validates the ordered members into `UniqueVec` projections for mutation,
member, and idempotency identities while preserving the original order. The
wrapper therefore carries nonemptiness and uniqueness without the invalid
shape `NonEmpty<UniqueVec<_>>` or duplicated member storage. It additionally
proves homogeneous policy, scope, barrier, byte-limit, and
range-disjointness laws. Runtime-width groups are never represented by a
fixed-arity proof join, tuple, or optional member slots.

One sealed group barrier proof derives one `WalDurablePhysicalMutation` per
matching member by consuming exact membership and range inclusion. When the
sealed group also shares one successor root, its namespace-durable publication
proof derives one root-completed member proof for each and only each included
data settlement. A member cannot be acknowledged until every shared effect in
its declared progression is complete.

It is mechanically impossible to:

- create a member from a group id alone;
- acknowledge a group;
- substitute one member for another;
- remove a failed or cancelled member after its WAL range may exist;
- infer semantic order from group order;
- change member durability request through grouping;
- widen security, tenant, key, artifact, or physical scope; or
- use one member's WAL, data, or root membership to complete another.

Group width, aggregate bytes, queue age, and barrier delay are admitted before
group formation. Saturation rejects or dispatches a smaller lawful group; it
never grows an unbounded queue.

## WAL-Before-Data And PageLSN Contract

WAL range reservation allocates identity only. The WAL owner emits immutable
framed bytes and an append declaration. Each member frame carries:

- Store, runtime-generation, operation, idempotency, and group/member identity;
- exact physical scope and immutable mutation fingerprint;
- one exact `PhysicalMutationAttemptBinding`;
- `CanonicalRedoRecords`, a Store wrapper constructed from Worth Proof
  `NonEmpty<RedoRecord>` and admitted only after its owner-defined order
  validates into `CanonicalVec<RedoRecord>`;
- one exact LSN per redo record;
- the target page/extent identity and payload or canonical redo basis;
- the expected resulting payload digest;
- admitted durability-profile identity; and
- framing, version, and integrity fields required for bounded C.8 inspection.

Store lowers that declaration through the C.5.1 work topology.

The matching data-effect plan is constructed with its page/extent WAL basis but
cannot be dispatched until Store consumes:

- complete WAL append evidence;
- the exact required backend barrier proof;
- matching Store/runtime/group/member identities;
- exact range inclusion; and
- current lifecycle and health authority.

Every candidate page image carries one bounded `PageWalBasis` containing:

- a `CertifiedPriorPageBasis` with exact page identity, prior encoded pageLSN,
  and prior payload digest; and
- a canonical ordered nonempty delta naming only the exact redo records newly
  applied to that prior image.

The resulting encoded pageLSN equals the greatest exact redo-record LSN in the
new delta and must advance consistently from the certified prior pageLSN. It is
not an arbitrary member-range start, end, frontier, or counter. Every delta
record must belong to the same admitted image basis, the resulting payload
digest must match, and the durable WAL frontier must cover the complete new
delta.

Physical format encodes the pageLSN supplied by the admitted Store plan. It
cannot choose, advance, or validate WAL durability. A data dispatch is lawful
only when the durable WAL frontier covers the complete newly applied delta.
C.8 later composes the checkpoint-certified prior image with the retained WAL
tail; C.7 proves the written image never outran or misidentified that bounded
causal extension. No ordinary page, frame, request, or checkpoint carries a
page's lifetime redo vector.

A raw `LogSequenceNumber`, range comparison, queue class, or barrier counter is
not dispatch authority.

## Root And Namespace Publication Contract

One Store-owned current-root progression consumes:

- the exact old current root;
- the successor candidate;
- every member data settlement required by that root;
- the exact WAL basis;
- the admitted backend publication requirement; and
- the current Store/runtime generation.

Publication progresses through candidate file durability, atomic replacement,
and required parent-namespace durability as distinct types.

The current-root authority changes only after the strongest required
publication proof exists. The old root and required supporting artifacts remain
retained. C.7 emits retention facts; C.8 and C.10 later consume them for
recovery and stable-reader policy.

The disconnected `PhysicalRootPublicationStore` and
`root-publications.log` do not survive as a second authority. Any reusable
old/new root validation or epoch meaning is moved behind the canonical Store
progression; the direct writer, file, runtime, and parallel current-root state
are deleted.

## Checkpoint And WAL-Retention Contract

A fuzzy checkpoint captures:

- one exact checkpoint identity;
- one admitted begin boundary;
- one exact covered WAL range;
- the physical root and dirty-generation bases it observed;
- bounded capture memory and I/O;
- exact concurrent-mutation posture;
- one published checkpoint artifact;
- one authoritative `PhysicalMutationBindingCompaction`; and
- one contiguous retained WAL tail beginning at the checkpoint boundary.

Checkpoint capture does not freeze all mutation authority. A short,
responsibility-named cutover fence may protect the final publication boundary;
it cannot span full capture or whole-Store traversal.

Binding compaction and reopen are incremental. Publication encodes one retained
binding record at a time from the locked registry; it does not retain a second
encoded record set. Fresh-process admission reads only the fixed checkpoint
header, compaction header, footer, and one bounded binding record at a time,
skipping the dirty-record body. Total rebuild memory is bounded by the admitted
total-live-binding count plus one checkpoint record and one WAL segment buffer,
not by historical WAL or checkpoint dirty cardinality.

The latest namespace-durable compaction and retained WAL suffix after its exact
cutoff are one `PhysicalDurabilityReopenBasis`. WAL framing and integrity
are verified once; idempotency consumes borrowed verified payload views and is
the only owner that interprets attempt-binding meaning. The pre-reopen policy
owner exposes observation only. A distinct post-reopen owner is required to
obtain idempotency, grouping, binding-compaction, WAL, or checkpoint authority,
making authority distribution before rebuild unavailable by type.

Compaction preserves `ReopenedUnresolved` obligations without granting them
fresh cancellation or group-sealing authority. Retained WAL may upgrade only
the exact matching obligation. Persisted terminal outcomes live behind the
closed `idempotency/fate/` seam; the registry stores that fate but does not own
its encoding vocabulary. Reopen rejects noncanonical order or encoding,
duplicates, foreign Store/policy, invalid leases, discontinuous WAL, incomplete
or substituted groups, and either admitted registry bound being exceeded.

WAL deletion or recycling requires:

- a namespace-durable checkpoint publication;
- exact checkpoint identity;
- a covering LSN range;
- a contiguous retained tail;
- absence of unresolved physical obligations requiring the candidate segment;
- a namespace-durable `PhysicalMutationBindingCompaction` containing every live
  attempt binding for which the candidate segment holds the last authoritative
  copy;
- proof that every binding in the candidate segment is either still present in
  the retained WAL tail, present in that compaction, or terminal, expired, and
  protected by the required later namespace-durable checkpoint; and
- later retention/recovery constraints already admitted to C.7.

WAL age, file count, disk pressure, checkpoint existence, or a copied tail
range is not eligibility.

`physical_runtime/durability/wal/reclamation/` is the sole owner of eligibility,
proof-bearing deletion authority, C.4 removal execution, and the resulting WAL
inventory transition. Inventory reports physical facts; checkpoint reports its
namespace-durable compaction and tail; neither may delete on its own. Failed or
indeterminate removal remains explicit residue and does not advance inventory
as if the segment disappeared.

`ContiguousRetainedWalTail` is constructed from Worth Proof
`NonEmpty<RetainedWalSegment>` and admitted only after the owner-defined order
validates into `CanonicalVec<RetainedWalSegment>`. The Store wrapper adds
adjacency, range-coverage, checkpoint-boundary, and no-required-gap proof.
`CanonicalVec` supplies stable ordering only; it never proves nonemptiness,
contiguity, or retention eligibility by itself.

WAL is not an all-history idempotency registry. Once the binding-compaction and
lease-expiry proofs above exist, deleting the superseded WAL copy is required.
Conversely, WAL recycling is forbidden while it would delete the last
authoritative copy of any live binding.

## Failure, Cancellation, And Acknowledgment Contract

Before any effect:

- denial, timeout, supersession, or cancellation produces proven no effect;
- all reserved resources and group membership are released or terminally
  accounted; and
- no WAL, data, checkpoint, or publication media operation occurs.

After an effect may have begun:

- cancellation stops caller waiting where lawful but not settlement;
- timeout cannot rewrite effect fate;
- short/torn/partial effects carry exact completed breadth;
- scheduler rejection after effect cannot become no-effect;
- completion delivery loss becomes completed-but-unobserved evidence where
  physical completion is proven;
- ambiguity becomes `IndeterminatePhysicalMutation`; and
- retry remains unavailable until exact no-effect proof or C.8
  reconciliation.

Physical acknowledgment is a consuming projection of
`CompletedPhysicalMutation`, not a mutable flag or independent canonical
artifact. Counters, traces, logs, reports, and Foundational projections derive
from the completed settlement and cannot manufacture it.

At an explicit support, certification, or cross-crate observation boundary,
Store may derive a one-way `StoreExecutedBoundaryReceiptEvidence` projection
from `PhysicalMutationAcknowledgment`. The projection names the physical
operation, request fingerprint, attempt binding, durability-policy basis, and
completed breadth. It cannot be accepted back into mutation admission,
progression, settlement, retry, acknowledgment construction, or current-root
authority. `ProvenNoEffect` and `Indeterminate` may produce separately named
diagnostic projections, never executed-boundary receipts.

## Signal, Proof, And Foundational Law

### Worth Signal

Signal represents derived readiness for:

- WAL append;
- required durability barrier work;
- checkpoint capture slices;
- root publication; and
- completion delivery.

C.7 installs these exact Store-owned physical work families into the existing
frozen binding and lifecycle:

- `WalAppend` for immutable member-frame append;
- `DurabilityBarrier` for the exact admitted WAL, file, or namespace barrier;
- `CheckpointCapture` for bounded capture slices; and
- `RootPublication` for candidate durability, replacement, and namespace
  effects.

Data work reuses C.6 `ExactWriteback` rather than creating a C.7 data queue.
The Store admits it only after consuming the exact
`WalDurablePhysicalMutation`. Completion delivery uses the existing generic
Signal resource-completion lifecycle; it is not a second C.7 effect family.

Store installs admitted physical aspect bindings and immutable observations.
Signal evaluators perform no media, WAL mutation, page mutation, checkpoint
publication, root replacement, settlement, or acknowledgment.

Destroying the Signal graph must not destroy any durability truth. After C.8,
it will be reconstructible from persisted physical authority and live runtime
admission.

### `worth-proof`

Worth Proof owns the reusable proof machinery C.7 consumes:

- `ProofOutcome` and `TransitionOutcome` category preservation;
- sealed progression and consumption mechanics;
- `NonEmpty`, `UniqueVec`, and `CanonicalVec` structural proof collections;
- exact basis and witness composition; and
- stale, rebind-required, assumption, and inspection posture.

C.7 uses that machinery without relocating domain meaning:

- durability-policy admission is a `ProofOutcome`;
- mutation preparation and checkpoint start preserve admitted, denied,
  deferred, stale, rebind-required, and failed categories;
- initialize, open, close, and abort retain the inherited Store
  `ProofOutcome` algebra;
- post-effect `Completed`, `ProvenNoEffect`, and `Indeterminate` remain
  Store-specific physical fate; and
- runtime-width groups and retained collections use structural collection
  proofs rather than fixed-arity joins.

Worth Proof does not define or own backend durability profiles, filesystem
capability claims, WAL bytes, barrier execution, pageLSN meaning, physical
effect receipts, current-root authority, acknowledgment, or retry eligibility.
Those remain with C.4, WAL, physical format, and Store according to their
existing boundaries.

Only `PhysicalDurabilityAdmissionBasis`, derived from
`QualifiedFilesystemMedia` and its exact C.4 qualification claims, admits the
durability policy. Only sealed receipts from the C.4 media owner, joined by
Store to exact identities and progression phases, establish that an effect
occurred.

Generic `AuthorityMarker` bounds are forbidden on every governed C.7 public or
cross-crate surface. Any underlying marker used to implement a sealed concrete
authority remains private and unnameable outside its owner. Copied digests,
profile labels, evidence projections, or public constructors open no C.7 door.

### Worth Foundational and Store aspect-native

Foundational owns stable aspect meaning, contract admission, validated values,
authoritative state, patches, and distinct projection/mutation/diagnostic mask
laws.

C.7 installs these exact derived work-basis contracts. The fifth mutation
contract, WAL reclamation, was populated with the Phase 6 retention boundary;
it is part of the final C.7 contract rather than a generic durability aspect.

| Store aspect-native contract key | Basis posture and role | Masks | Signal family | Exact partition |
| --- | --- | --- | --- | --- |
| `store.physical.durability.policy-binding-basis` | projection; `Dependency` | projection only | `WalAppend`, `DurabilityBarrier`, `CheckpointCapture`, `RootPublication`, and `WalReclamation` | exact admitted durability-policy identity |
| `store.physical.durability.wal-append-basis` | mutation; `DependencyAndOutput` | projection and mutation | `WalAppend` only | exact stable Store identity |
| `store.physical.durability.wal-barrier-basis` | mutation; `DependencyAndOutput` | projection and mutation | `DurabilityBarrier` only | exact stable Store identity |
| `store.physical.durability.checkpoint-capture-basis` | mutation; `DependencyAndOutput` | projection and mutation | `CheckpointCapture` only | exact stable Store identity |
| `store.physical.durability.root-publication-basis` | mutation; `DependencyAndOutput` | projection and mutation | `RootPublication` only | exact stable Store identity |
| `store.physical.durability.wal-reclamation-basis` | mutation; `DependencyAndOutput` | projection and mutation | `WalReclamation` only | exact stable Store identity |

These are derived routing and invalidation bases. Artifact-specific WAL range,
barrier, checkpoint, root-candidate, and reclaim identities remain in the
typed Store work declaration and executor command; copying them into an aspect
partition would create a second physical truth lane. The stable Store
partition prevents cross-Store invalidation. Each installed Signal graph is
already owned by one runtime incarnation; adding that ephemeral identity to
the partition would destabilize the semantic profile across lawful reopen.
Exact runtime identity remains in typed Store work and progression authority.
These contracts are not authoritative
“WAL generation,” “durability-profile generation,” “current-root generation,”
or “checkpoint generation” state. In particular,
`root-publication-basis` does not name or advance the current root.

The C.7 bases join the inherited lifecycle-health and C.6
`store.physical.record.frame-writeback-basis` contracts. Only the
responsibility-named Store `work_semantics/` owner constructs them as
`PhysicalWorkSemanticBasis`; lower physical owners remain Foundational- and
Signal-agnostic. Store maps each admitted aspect to bounded Signal routing
slots through the existing frozen binding law. A native patch invalidates
exactly its declared physical work family and partition; raw Signal bits cannot
create or widen that dependency.

Every C.7 scheduler demand for group/barrier, WAL append/tail, checkpoint
capture, or root publication consumes a matching
`FoundationalPolicyAdmissionReceipt` from the existing policy-admission path.
The receipt proves that the named demand was admitted under its exact work
class and finite budget; it does not prove execution or physical durability.

Counter-backed completion can be projected at an explicit observation or
certification boundary as
`StorePerformanceReceiptEvidence<FoundationalAuthoritativePerformanceClaim>`.
That evidence
names the admitted policy receipt, work class, scale, completed breadth, and
exact counters. It is derived evidence, never scheduler capacity, effect
settlement, or mutation authority. The ordinary hot path updates bounded
counters and does not materialize evidence objects or JSON.

`PhysicalMutationAcknowledgment` may project one-way to
`StoreExecutedBoundaryReceiptEvidence` as defined above. No Foundational or
aspect-native receipt is accepted back as C.7 authority.

Foundational does not own:

- WAL bytes or durable prefixes;
- pageLSNs;
- current-root authority;
- effect completion;
- physical mutation fate; or
- acknowledgment.

The general Foundational `FoundationalProfileSet` is not the C.4 backend
durability profile and cannot admit one.

A generic `durable=true`, `published=true`, `committed=true`, or JSON object is
forbidden as internal authority.

## Authority Ownership

| Type or responsibility | Constructed by | Proves | Authorizes | Cannot authorize | Consumed by |
| --- | --- | --- | --- | --- | --- |
| `PhysicalDurabilityAdmissionBasis` | C.4 physical-backend owner from `QualifiedFilesystemMedia`, `RootProfileQualificationBasis`, and exact `AdmittedBackendCapabilityWitness` claims | the qualified media generation satisfies the requested filesystem capability vocabulary | Store durability-policy admission only | an effect, barrier completion, current root, or acknowledgment | `PhysicalDurabilityDeclaration::admit` |
| `AdmittedPhysicalDurabilityPolicy` | Store from declaration plus consumed `PhysicalDurabilityAdmissionBasis` through `ProofOutcome` | exact supported C.4 profile, barrier posture, and finite C.7 limits | construction of the matching C.7 runtime owner | effect completion | Store instance construction |
| `PhysicalMutationIdempotencyLease` | Store from the current namespace-durable checkpoint generation and admitted retention-generation count | exact Store-scoped issuance and expiry checkpoint frontiers | issuance of a bounded physical idempotency key | execution deadline, effect, or terminal fate | public key issuance and binding retention |
| `PhysicalMutationIdempotencyKey` | Store public issuance from validated caller material plus current `PhysicalMutationIdempotencyLease` | stable bounded physical retry identity | admission lookup and duplicate correlation | operation scope, success, semantic transaction identity | Store mutation admission |
| `PhysicalMutationRequestFingerprint` | Store from the admitted canonical request basis | exact effect-relevant request equivalence independent of any attempt | same-key duplicate comparison | operation/group/WAL identity, execution, or success | idempotency admission and WAL attempt binding |
| `PhysicalMutationAttemptBinding` | Store admission and WAL reservation | exact key/lease/fingerprint relationship to one operation and later group/member/range | continuation and fresh-process reconciliation of that one attempt | equivalence of a different request or completed fate | WAL frame, progression, and C.8 |
| `PhysicalMutationBindingCompaction` | Store checkpoint owner from the prior compaction, retained WAL tail, exact lease frontiers, and terminal settlements | bounded authoritative set of every unexpired terminal and unresolved attempt binding whose original WAL may be reclaimed | duplicate admission, WAL-reclamation eligibility, and C.8 handoff | all-history retention, semantic liveness, or effect completion | checkpoint publication, Store reopen, and C.8 |
| `PhysicalMutationHandle` | Store when `PreparedPhysicalMutation::start` consumes one admitted preparation | exact started-mutation identity and caller observation/cancellation relationship while Store retains settlement ownership | typed poll, wait, and cancellation request for that mutation | effect execution, forged no-effect, drop-implies-cancellation, or settlement abandonment | caller observation and Store lifecycle drain |
| `AdmittedPhysicalMutation` | Store | current identity, scope, policy, generation, and no effect yet | WAL planning | media access or acknowledgment | WAL reservation |
| `SealedPhysicalDurabilityGroupMembers` | Store from `NonEmpty` members plus preserving-order `UniqueVec` identity validation | nonempty exact members with unique mutation, member, and idempotency identities | group-policy admission | effect, group barrier, or member fate | group admission |
| `AdmittedPhysicalDurabilityGroup` | Store from `SealedPhysicalDurabilityGroupMembers` | immutable group membership and allowed shared effects | one group barrier and declared shared root plan | member identity, fate, or acknowledgment | group execution |
| `WalRangeReservedPhysicalMutation` | Store plus WAL owner | exact immutable WAL range and bytes for one member | WAL append declaration | WAL durability or data dispatch | physical work lowering |
| `WalDurablePhysicalMutation` | Store from matching append and barrier receipts | exact member WAL basis is durable under profile | matching data dispatch | another member, root publication, or acknowledgment | executor lowering |
| `CertifiedPriorPageBasis` | Store from the currently admitted page image and its exact identity, encoded pageLSN, and payload digest | fixed-size prior image basis from which one new mutation delta begins | construction of the matching bounded `PageWalBasis` | WAL durability, a different page, or lifetime redo history | Store data-effect planning |
| `PageWalBasis` | Store from one `CertifiedPriorPageBasis` plus canonical ordered nonempty newly applied redo | exact bounded causal extension and resulting pageLSN/digest basis | matching data dispatch after the new delta is WAL durable | arbitrary pageLSN advance or whole-history carriage | physical-format encoding, data settlement, and C.8 |
| `PhysicalWritebackSettlement` | existing C.6 Store progression | exact frame effect fate and Signal settlement | C.6 clean/retry/inspection transition | WAL or root durability | C.7 data-settlement join |
| `DataSettledPhysicalMutation` | Store | every required member data effect has exact terminal fate | root publication when completed | semantic visibility | root progression |
| `RootNamespaceDurablePhysicalMutation` | Store from exact root and namespace receipts | required current-root publication barriers completed | final physical completion | branch-head or semantic commit | completion |
| `CompletedPhysicalMutation` | Store | every declared physical edge completed | creation of one physical acknowledgment | retry or semantic publication | caller projection |
| `PhysicalMutationAcknowledgment` | consuming completed outcome | caller-visible exact physical completion fact | correlation by a future adapter | branch progression, writer release, Query acknowledgment | Part II adapter later |
| `ProvenNoEffectPhysicalMutation` | Store settlement | no physical effect occurred for exact attempt | safe same-idempotency retry subject to fresh admission | success | caller or retry admission |
| `IndeterminatePhysicalMutation` | Store settlement | possible effect with exact unresolved basis | preservation and C.8 inspection | acknowledgment or automatic retry | C.8 |
| `PhysicalCheckpointHandle` | Store checkpoint owner | one managed bounded checkpoint lifecycle | progress, safe cancellation, finalization | raw WAL/root/frame mutation | caller and lifecycle drain |
| `ContiguousRetainedWalTail` | Store from checkpoint and WAL inventory | exact retained tail follows exact checkpoint | retention eligibility evaluation | recovery success | C.8 and WAL retention |
| `FoundationalPolicyAdmissionReceipt` | existing Foundational policy-admission path | one named scheduler demand fits its admitted work class and finite budget | matching scheduler demand admission | backend capability, execution, settlement, or durability | C.7 scheduler |
| `StorePerformanceReceiptEvidence<FoundationalAuthoritativePerformanceClaim>` | Store observation/certification projection from policy receipt and exact counters | described completed cost within the named boundary | nothing | capacity, execution, settlement, or acknowledgment | operators and certification |
| `StoreExecutedBoundaryReceiptEvidence` | one-way Store projection from `PhysicalMutationAcknowledgment` | descriptive evidence of the named completed physical boundary | nothing | admission, progression, retry, root authority, or acknowledgment reconstruction | support, certification, and cross-crate observation |
| observations and reports | read-only Store/certification projections | executed facts within their named boundary | nothing | any phase transition | callers, operators, certification |

## Required Destination Directory And Module Plan

Status legend:

- **retain**: existing responsibility remains in place;
- **create**: C.7 establishes the populated boundary;
- **move**: responsibility relocates without compatibility re-export;
- **replace**: successor becomes the only ordinary owner;
- **remove**: path and authority disappear; and
- **committed successor**: named future insertion point, not an empty file to
  create now.

### Store-owned cross-domain orchestration

```text
workspaces/worth-store/crates/worth-store/src/physical_runtime/
├── durability/                                      # create; stable C.7 owner
│   ├── mod.rs                                       # create; facade/export only
│   ├── admission/
│   │   ├── platform_basis_join.rs                    # create; Store/C.4 identity join
│   │   ├── policy.rs                                # create
│   │   ├── mutation_preparation.rs                  # create
│   │   ├── checkpoint_start.rs                      # create
│   │   └── denial.rs                                # create
│   ├── mutation/
│   │   ├── identity.rs                              # create
│   │   ├── request_fingerprint.rs                   # create
│   │   ├── idempotency/                             # create; bounded retry-authority family
│   │   │   ├── key.rs                               # create
│   │   │   ├── lease.rs                             # create
│   │   │   ├── attempt_binding.rs                   # create
│   │   │   ├── binding_compaction.rs                # create; persisted authority
│   │   │   └── registry.rs                          # create; bounded derived index
│   │   ├── handle.rs                                # create; caller observation/cancellation
│   │   ├── outcome.rs                               # create
│   │   └── progression/
│   │       ├── admitted.rs                          # create
│   │       ├── wal_reserved.rs                      # create
│   │       ├── wal_appended.rs                      # create
│   │       ├── wal_durable.rs                       # create
│   │       ├── data_settled.rs                      # create
│   │       ├── root_replaced.rs                     # create
│   │       ├── namespace_durable.rs                 # create
│   │       └── completed.rs                         # create
│   ├── grouping/
│   │   ├── admission.rs                             # create
│   │   ├── unique_membership.rs                     # create
│   │   ├── wal_barrier.rs                           # create
│   │   ├── root_publication.rs                      # create
│   │   └── member_settlement.rs                     # create
│   ├── wal/
│   │   ├── append_declaration.rs                    # create
│   │   ├── member_basis.rs                          # create
│   │   ├── canonical_redo.rs                        # create
│   │   └── barrier_join.rs                          # create
│   ├── data/
│   │   ├── prior_page_basis.rs                      # create; fixed-size certified basis
│   │   ├── page_wal_basis.rs                        # create; bounded redo delta join
│   │   └── writeback_join.rs                        # create
│   ├── checkpoint/
│   │   ├── handle.rs                                # create
│   │   ├── capture.rs                               # create
│   │   ├── progress.rs                              # create
│   │   ├── publication.rs                           # create
│   │   └── retained_wal_tail.rs                     # create
│   ├── publication/
│   │   ├── root_transition.rs                       # create
│   │   ├── namespace_durability.rs                  # create
│   │   └── retained_root.rs                         # create
│   ├── settlement/
│   │   ├── acknowledgment.rs                        # create
│   │   ├── proven_no_effect.rs                      # create
│   │   ├── indeterminate.rs                         # create
│   │   └── completed_unobserved.rs                  # create
│   ├── lifecycle/
│   │   ├── managed_work.rs                          # create
│   │   └── drain.rs                                 # create
│   ├── evidence_projection/
│   │   ├── executed_boundary.rs                     # create; one-way only
│   │   ├── performance.rs                           # create; counter-backed
│   │   └── diagnostic_fate.rs                       # create; non-authoritative
│   └── observation/
│       ├── counters.rs                              # create
│       └── snapshot.rs                              # create
├── record_serving/
│   ├── publication/                                 # retain and narrow
│   │   ├── candidate_data.rs                        # retain candidate meaning
│   │   ├── plan.rs                                  # retain record plan
│   │   └── ...                                      # move durability orchestration out
│   ├── work_semantics/                              # retain sole semantic-basis owner
│   │   └── durability/                              # create; expected to grow by family
│   │       ├── policy_binding_basis.rs              # create
│   │       ├── wal_append_basis.rs                  # create
│   │       ├── barrier_basis.rs                     # create
│   │       ├── checkpoint_capture_basis.rs          # create
│   │       └── root_publication_basis.rs            # create
│   └── residency/dirty/                             # retain C.6 owner
│       └── outcome.rs                               # retain writeback settlement
└── work/                                            # retain C.5.1 topology
    ├── signal_binding/                              # retain/extend admitted bindings
    ├── scheduling/                                  # retain/extend resource lowering
    ├── execution/                                   # retain sole media dispatch
    └── settlement/                                  # retain exact effect truth
```

The dominant axis of `physical_runtime/durability/` is Store-owned physical
durability authority. It contains cross-owner ordering and settlement only. WAL
grammar, media mechanics, residency mechanics, recovery decisions, integrity,
stable-reader policy, semantic commit, and certification are excluded.

Phase 6 refines the required destination with these populated or immediately
required semantic owners:

```text
physical_runtime/
  durability/mutation/idempotency/
    bootstrap.rs
    binding_compaction.rs
    binding_compaction/{encoding.rs,decoding.rs}
    persisted_binding.rs
    persisted_binding/decoding.rs
    fate/persisted.rs
  durability/checkpoint/
    reopen.rs
    reopen/binding_compaction.rs
  durability/wal/
    inventory/{reopen.rs,reopened_member.rs}
    reclamation/{eligibility.rs,authority.rs,execution.rs,inventory_transition.rs}
  instance/
    durability_bootstrap.rs
    construction/{work_runtime.rs,record_serving.rs}
```

`reclamation/` may begin with one populated file if only one of those
responsibilities is implemented in the first coherent slice; the directory is
still correct because eligibility, authority, execution, and inventory
transition are committed distinct growth points. A one-file directory is not
an excuse to combine them into a WAL or checkpoint god file.

`mutation/`, `grouping/`, `checkpoint/`, `publication/`, `settlement/`, and
`observation/` are distinct because they have different identity,
cardinality, lifecycle, failure, and replacement fate. A single
`durability.rs`, `transaction.rs`, `manager.rs`, `coordinator.rs`, or
`pipeline.rs` file is forbidden.

`admission/platform_basis_join.rs` consumes and verifies the sealed C.4
admission basis against Store policy/runtime identity; it does not restate
capability semantics.
`mutation/request_fingerprint.rs` owns canonical input equivalence, while
`mutation/attempt_binding.rs` owns later operation/group/WAL allocation facts.
They may not be collapsed. The inherited
`record_serving/work_semantics/durability/` boundary is the only C.7 production
directory that constructs Foundational/Store aspect-native
`PhysicalWorkSemanticBasis` values. The directory is justified even if an
early phase temporarily populates only one contract because four more named
families are committed in this milestone. `evidence_projection/` is derived
truth and has no dependency path back into admission, progression, or
settlement.

### Store aspect-native canonical registry

```text
workspaces/worth-store/crates/worth-store-aspect-native/src/
├── canonical_basis.rs                               # retain/extend closed enums
└── canonical_basis/
    ├── canonical_basis_sources.rs                   # retain/extend admission map
    ├── canonical_basis_domains.rs                   # retain/extend exact domain
    └── canonical_basis_construction.rs              # retain/extend native source
```

This boundary adds only the named request-fingerprint family, source kind,
field role, lane, domain, and native construction route. It does not import
`worth-store`, own idempotency, allocate operations, read WAL, or decide retry.
Store supplies validated native fields; the registry supplies canonical-basis
admission and digest equivalence.

### WAL meaning

```text
workspaces/worth-store/crates/worth-store-wal/src/
├── wal_topology/                                    # retain
├── artifact_store/                                  # retain encoding/scan; remove execution
├── append/
│   ├── mod.rs                                       # retain facade
│   ├── frame_plan.rs                                # move/split current planning
│   └── durability_admission.rs                      # narrow to declaration
├── publication_declaration/                         # replace durable_publication/
│   ├── mod.rs
│   ├── wal_scope.rs
│   └── checkpoint_scope.rs
└── queue_declaration/                               # replace generic durability/
    ├── mod.rs
    └── grouping_scope.rs
```

`worth-store-wal` owns immutable WAL meaning and declarations. Direct calls to
`StoreDurabilityRuntime`, filesystem paths as execution authority, final
acknowledgment, root authority, checkpoint execution, and Store progression are
excluded.

The renamed declaration directories prevent a plan or scope from presenting
itself as completed durability.

### Recovery-facing artifacts

```text
workspaces/worth-store/crates/worth-store-recovery-physics/src/
├── wal_recovery_basis/                              # replace ordinary wal_durability/
│   ├── mod.rs
│   ├── append_receipt.rs                            # narrow to observed WAL fact
│   ├── crash_basis.rs                               # move retained crash meaning
│   └── durability_observation.rs                    # retain read-only fact
├── checkpoint_cutover/                              # retain and narrow
│   ├── checkpoint_id.rs
│   ├── checkpoint_lsn.rs
│   ├── checkpoint_manifest.rs
│   ├── checkpoint_validation.rs
│   └── wal_retention.rs
├── source_precedence/                               # committed C.8 consumer
├── redo_replay/                                     # committed C.8 consumer
└── page_redo/                                       # committed C.8 consumer

remove:
  wal_durability/executed_append.rs
  direct ordinary acknowledgment construction
  any StoreDurabilityRuntime composition used by ordinary C.7
```

Recovery-facing types describe persisted facts and future recovery decisions.
They do not execute ordinary C.7 effects or mint ordinary acknowledgment.

### Backend and media

```text
workspaces/worth-store/crates/worth-store-physical-backend/src/
├── durability_profile/                              # retain
│   └── physical_admission_basis.rs                  # create from qualified media
├── artifact_tree/ or admitted media operation home  # retain C.4 effect mechanics
└── durability_ordering/                             # narrow to declarations/receipts
    ├── admission.rs
    ├── requirement.rs
    ├── receipt/
    └── counters.rs

remove:
  durability_ordering/execution/file_runtime.rs
  StoreDurabilityRuntime as a direct filesystem owner
  public execution construction outside the C.4 media owner
```

The backend may report exact barrier mechanics and sealed physical receipts. It
does not know mutation grouping, pageLSN policy, current-root authority,
checkpoint policy, or acknowledgment.

`physical_admission_basis.rs` binds the already admitted
`RootProfileQualificationBasis` and exact
`AdmittedBackendCapabilityWitness` claims to one
`QualifiedFilesystemMedia` generation. It exposes the concrete sealed
`PhysicalDurabilityAdmissionBasis` required by Store policy admission, not raw
claims, a generic marker, or an independently constructible profile.

### Root publication

```text
workspaces/worth-store/crates/worth-store-physical-isolation/src/publication/
├── epochs.rs                                        # move reusable validation if admitted
├── intent.rs                                        # narrow or move pure root-transition meaning
└── receipt.rs                                       # narrow to stable-reader facts for C.10

remove:
  store.rs
  runtime.rs
  root-publications.log writer and parser
  parallel current-root state
```

Reusable pure validation may remain only if it consumes canonical C.5/C.7 root
identity without importing Store internals or retaining a second artifact.
C.10 later adds stable-reader and reclaim authority as siblings under its own
owner; it does not resurrect this writer.

### Direct durability scenario

```text
workspaces/worth-store/tools/store-test-runner/src/
└── durable_publication_siege/                       # focused process scenario
    ├── execution.rs
    ├── world.rs
    ├── crash_seam.rs
    └── observation.rs

workspaces/worth-store/crates/worth-store-physical-certification/src/
└── durability_inspection/                           # create
    ├── wal.rs
    ├── pages.rs
    ├── checkpoint.rs
    ├── roots.rs
    └── manifest.rs
```

Schedule selection, when needed, stays inside the focused scenario and prints a
seed on failure. The inspector is read-only test infrastructure. It imports
stable lower artifact declarations, not the `worth-store` runtime or its
durability classifier. No reporting, evidence-projection, or mutation-catalog
subsystem is required.

### Structural enforcement

Dependency, visibility, feature, and source gates must prove:

- only `worth-store` composes WAL, Signal, scheduler, C.6 residency settlement,
  and backend durability effects;
- WAL, recovery physics, backend, buffer pool, physical format, and
  certification remain Signal-agnostic;
- certification and replay remain outside ordinary features;
- no lower crate imports `worth-store`;
- no direct filesystem durability executor survives outside C.4 media;
- no semantic or branch crate enters Part I;
- no public facade exports internal progression constructors; and
- no generic `AuthorityMarker`, Foundational profile, or evidence projection
  appears in a C.7 admission or progression signature;
- only `record_serving/work_semantics/durability/` constructs C.7
  Foundational/Store aspect-native work bases;
- `evidence_projection/` has no reverse dependency into admission,
  progression, execution, or settlement;
- request-fingerprint construction has no dependency on operation, group, WAL
  allocation, idempotency lease/registry, scheduler, or observation modules;
  and
- all ordinary durability APIs are reachable from semantic physical roots
  without internal module archaeology.

## Performance And Resource Contract

C.7 names and bounds:

- mutation batch record count and bytes;
- group width, aggregate bytes, maximum queue age, and maximum barrier delay;
- WAL frame bytes, segment bytes, active segments, rotations, and retained tail;
- concurrent admitted WAL, data, checkpoint, publication, and acknowledgment
  work;
- checkpoint capture memory, capture I/O breadth, dirty-page interaction, and
  retained-tail ceiling;
- pending completed-but-unobserved and indeterminate operations;
- current and retained root generations; and
- close/drain work.

Admission rejects before expensive framing, copying, allocation, queueing, or
effects where the disqualifying fact is already available.

Every scheduler demand carries its matching
`FoundationalPolicyAdmissionReceipt`; there is no global receipt that admits
all C.7 work. The exact receipt work class and finite budget must agree with the
group/barrier, WAL append/tail, checkpoint capture, root publication, or
completion-delivery demand. A mismatched, stale, copied, or broader receipt is
denied before queue admission.

The ordinary mutation path may scale with:

```text
declared mutation bytes
+ WAL framing for that mutation
+ exact touched physical frames/extents
+ admitted group-share coordination
+ required durability barriers
+ exact root/checkpoint publication granule
```

It may not scale with total Store size, total historical WAL, consumer count,
diagnostic richness, mutation-corpus size, or offline-verifier work.

`PageWalBasis` carriage scales only with the newly applied redo delta plus one
fixed-size certified prior-page basis. Idempotency authority scales only with
the admitted unresolved-attempt bound, the admitted retention-generation
window, and mutation arrival within that window; expiry and checkpoint
compaction must prevent total historical mutations from becoming retained
ordinary state.

Group commit must expose logical-to-physical amplification:

- mutations admitted;
- groups formed;
- members per group;
- WAL frames and bytes;
- barrier executions;
- data writes and bytes;
- root publications;
- acknowledgments; and
- per-member fate.

Checkpoint work has a distinct background/maintenance lane. Deferral does not
erase cost. If retained-WAL or queue pressure reaches its bound, the runtime
backpressures or denies before unbounded growth.

Elapsed-time gates are secondary to structural counters. Performance evidence
must name hardware, filesystem, profile, scale, cold/warm posture, arrival and
burst model, queue utilization, repetitions, and percentiles.

Where a support or cross-crate boundary needs typed performance evidence, it uses
`StorePerformanceReceiptEvidence<FoundationalAuthoritativePerformanceClaim>`
projected from the matching policy receipt and counter-backed completion facts.
Support or cross-crate completed-operation reports use the one-way
`StoreExecutedBoundaryReceiptEvidence` projection. Neither projection is
created merely to run the ordinary path, and neither is accepted as an input
to it.

## Cleanup And Cutover Contract

Cutover is reviewed from the current Git diff, Cargo graph, public facades, and
direct callers. Replaced paths are deleted with their final caller migration;
no removal ledger or generated absence proof is maintained.

Implement the cutover in the owning source:

- retain WAL grammar, bounded scanning, checkpoint meaning, backend mechanism
  evidence, and physical publication meaning only where the final Store path
  consumes them;
- route ordinary WAL and data effects through their media owners, with Store
  owning group fate, current-root authority, and physical acknowledgment;
- migrate current callers in the same change, then delete parallel durability
  runtimes, root-publication authority, reverse evidence lanes, generic
  authority aspects, compatibility aliases, stale fixtures, and feature edges;
- keep request equivalence separate from persisted attempt identity, and never
  include allocated operation, group, member, LSN, or WAL-range facts in a
  request fingerprint; and
- retain certification behavior only as focused tests of the canonical direct
  scenario.

A replacement is incomplete while its predecessor compiles in an ordinary
feature graph.

## Verification

C.7 closeout is historical Git evidence. Current validity comes from current
focused durability tests, compile-time authority denials, process scenarios,
and workspace boundary checks. No living ledger or source-identity service is
maintained.

At closeout, checks must prove:

- one ordinary WAL effect route;
- one ordinary data effect route;
- one canonical current-root authority;
- one physical acknowledgment type;
- zero direct `StoreDurabilityRuntime` product callsites;
- zero ordinary direct `execute_wal_durability` callsites;
- zero `root-publications.log` writer or parallel root runtime;
- zero public progression constructors;
- zero allocated attempt or WAL facts in request-fingerprint construction;
- zero idempotency-lease or registry dependency in request-fingerprint
  construction;
- zero all-history idempotency registry or last-authority WAL reclamation;
- zero lifetime-redo-vector carriage in ordinary page work;
- zero caller-handle drop path that requests cancellation or abandons
  settlement;
- zero generic durability/profile aspects or generic authority-marker bounds;
- zero evidence-projection imports in admission, progression, execution, or
  settlement;
- zero semantic transaction/branch/Query vocabulary below the future adapter;
- zero certification feature leakage;
- zero C.7-named production identifiers; and
- zero temporary scenario hooks outside sealed C.4 yieldpoints.

## Documentation Deliverables

### Physical durability guide

Create `_docs/worth-store/physical-durability-and-checkpoints.md` for Store
callers and operators.

It must explain:

- physical durability versus semantic commit;
- the public mutation request and typed outcome topology;
- request-fingerprint equivalence versus attempt/WAL binding;
- idempotency issuance, durable-generation retention, expiry, compaction,
  cancellation, timeout, completed-but-unobserved, and indeterminate behavior;
- mutation-handle ownership, drop-as-observation-abandonment, cancellation
  outcomes, and close-time draining;
- backend-profile-relative durability;
- C.4 capability admission versus Worth Proof progression mechanics;
- group commit without identity merging;
- checkpoint lifecycle, progress, bounds, and WAL retention;
- operator handling of inspection-required outcomes;
- exact cost and amplification surfaces; and
- Foundational policy-admission and one-way evidence projections, including
  their explicit non-authoritative status; and
- what C.8 recovery will and will not later add.

All examples compile against the ordinary facade.

### Owner documentation

Revise:

- `worth-store-wal/README.md` to state that it owns WAL meaning and
  declarations, not ordinary execution or final acknowledgment;
- the C.6 bounded access guide to link dirty/writeback settlement to the C.7
  durability guide without claiming dirty implies durable;
- `storage-foundation-aspect-native-gate.md` to name the six exact C.7 derived
  work-basis contracts, roles, masks, families, and partitions, and to state
  that none owns physical truth;
- the reconstruction roadmap's C.7 current-contract and C.8 handoff links.

Add responsibility-named README material only where a continuing crate audience
needs it. Do not create phase summaries, closeout narratives, or duplicate
architecture explanations.

Remove or correct every document that teaches:

- `DurableAckReceipt` as complete C.7 acknowledgment;
- direct `StoreDurabilityRuntime` product use;
- direct recovery-physics WAL execution as the ordinary lane;
- `root-publications.log` as canonical current-root authority;
- rename without namespace durability;
- checkpoint existence as WAL-deletion eligibility; or
- physical acknowledgment as semantic commit.

Documentation source paths, examples, and public names are checked against the
final implementation in CI.

## Phase Plan

### Phase 1: Freeze Authority, Vocabulary, And Removal Truth

**Becomes true:** current WAL, barrier, checkpoint, pageLSN, root-publication,
acknowledgment, and ordinary caller paths have one named owner and use the
intended production route.

**Consumes:** C.4-C.6 contracts, current Cargo graph, and phase-local review of
current production call paths.

**Establishes:** the semantic vocabulary lock, final API decision, destination
topology, and deletion of obsolete paths discovered from current callers and
the Cargo graph.

**Mechanically forbids:** new C.7 production methods, crates, aliases, or
compatibility paths before their owner and proof boundary are named.

**Evidence:** current source and caller review, Cargo dependency and feature
graphs, direct public API compilation, and focused mechanism tests.

**Next may trust:** no current mechanism is being promoted by name alone and no
duplicate path is accidentally omitted from cutover.

**Cleanup:** delete already-unreachable demonstrations and stale feature edges
that need no replacement.

### Phase 2: Admit Physical Durability Policy And Mutation Identity

**Becomes true:** one Store runtime consumes a sealed
`PhysicalDurabilityAdmissionBasis` from its `QualifiedFilesystemMedia`, and
every mutation has exact canonical request equivalence, operation,
idempotency key and durable-generation lease, scope, profile, lifecycle, and
bounded-resource identity before effects.

**Consumes:** C.4 `QualifiedFilesystemMedia`,
`RootProfileQualificationBasis`, exact `AdmittedBackendCapabilityWitness`
claims, C.5.1 physical work identity and `ProofOutcome` topology, C.6 policy
construction pattern, and the Store aspect-native canonical basis registry.

**Establishes:** sealed physical durability admission basis, admitted
durability policy, `PhysicalMutationRequestFingerprint` canonical
family/source/role/lane, mutation request, the attempt-binding contract that
Phase 3 must complete after allocation, internal admission types, exact
`ProofOutcome` denial topology, durability-policy binding work basis,
group/checkpoint limits, nonzero finite idempotency-retention generations, exact
expired-key denial, and exhaustive runtime lifecycle propagation.

**Mechanically forbids:** optional durability owners, booleans, generic
authority markers, Foundational profiles as backend authority, raw profile
labels, ambient idempotency, allocation facts in request fingerprints, omitted
effect-relevant fingerprint fields, wall-clock or deadline-derived retention,
silent expired-key reuse, and incomplete construction.

**Evidence:** builder/admission tests, profile-denial tests, construction-total
compiler failures, fingerprint version/golden vectors, lease issuance/expiry
tests, retry-equivalence and conflict regressions, basis-substitution attacks,
expired-key-reuse checks, and zero-effect rejection proofs.

**Next may trust:** every later phase starts from one exact admitted physical
mutation under one real profile.

**Cleanup:** remove obsolete configuration and public constructors that can
express unadmitted or unlimited durability.

### Phase 3: Bind WAL Meaning To The Canonical Work Topology

**Becomes true:** ordinary mutations receive immutable WAL frames and member
ranges from `worth-store-wal`, then append only through Store Signal,
scheduler, executor, and C.4 media.

**Consumes:** admitted mutation, WAL grammar/topology, C.5.1 work progression,
C.4 media.

**Establishes:** reserved and appended phase types, append declarations,
completion of `PhysicalMutationAttemptBinding` with exact member/range
identity and lease frontier, canonical ordered nonempty redo, WAL-append work
basis, matching Foundational policy admission, exact append settlement, and WAL
observation.

**Mechanically forbids:** direct WAL filesystem execution, final
acknowledgment, data dispatch, recovery-physics ordinary execution, and any WAL
allocation field feeding back into request equivalence.

**Evidence:** real WAL append journey, bounded scan by independent inspector,
torn-frame seam, persisted attempt-binding inspection, direct-media source
gate, and receipt-substitution attack.

**Next may trust:** WAL bytes and ranges are real, exact, and on the one
production path, but not yet necessarily durable.

**Cleanup:** delete StoreDurabilityRuntime-based WAL execution and migrate or
delete its last ordinary callers.

### Phase 4: Make WAL-Before-Data And PageLSN Unskippable

**Becomes true:** only the matching admitted WAL barrier proof can construct a
data-dispatchable mutation, and every persisted pageLSN is bound to that proof.

**Consumes:** appended WAL mutation, backend profile/barrier receipts, C.6
writeback claim and settlement, physical-format pageLSN encoding.

**Establishes:** WAL-durable, data-dispatched, and data-settled types, exact
bounded page/extent WAL bases composed from one certified prior basis plus the
new nonempty redo delta, and the exact durability-barrier work basis and policy
receipt join.

**Mechanically forbids:** queue labels, counters, raw LSN comparisons, foreign
receipts, early frame cleaning, and scheduler-derived settlement.

**Evidence:** compile-fail progression cases, focused WAL-before-data inversion,
pageLSN-ahead, lifetime-redo-vector, and prior-basis/delta substitution tests,
stale/foreign receipt coverage, and the real C.6 writeback join.

**Next may trust:** completed data never outran its exact durable WAL basis.

**Cleanup:** remove legacy pageLSN or writeback paths that perform the join by
convention or duplicate Store settlement.

### Phase 5: Share Physical Cost Without Sharing Mutation Identity

**Becomes true:** lawful group commit amortizes WAL barriers and one explicitly
planned group root publication while preserving exact member identity, range,
effect, cancellation, fate, and acknowledgment.

**Consumes:** admitted members, WAL ranges, grouping policy, scheduler
admission, barrier proofs.

**Establishes:** group admission through
`SealedPhysicalDurabilityGroupMembers`, constructed from nonempty members and
validated unique identity projections, shared barrier, optional shared root
publication, and exact member-derivation types with hard queue and delay
bounds.

**Mechanically forbids:** group acknowledgment, identity collapse, cross-member
receipt substitution, fixed-arity runtime groups, semantic ordering, and
unbounded batching.

**Evidence:** four-request/three-sealed-member identity blender, pre-seal
cancellation plus sealed-member delay/reorder schedule, barrier/root
amplification counters, duplicate/conflicting idempotency cases, and focused
member-collapse/substitution regressions.

**Next may trust:** shared physical effects reduce cost without changing
individual truth or admitting partial group visibility.

**Cleanup:** delete grouping surfaces that copy member fields into one aggregate
outcome or expose group membership as transaction identity.

### Phase 6: Install Bounded Fuzzy Checkpoint And WAL Retention

**Becomes true:** checkpoint capture proceeds under concurrent mutation with an
exact source range, bounded memory, bounded retained WAL, managed lifecycle,
and proof-gated retention.

**Consumes:** WAL/data progression, C.6 scoped allocations and pressure,
checkpoint artifact semantics, scheduler resources.

**Establishes:** admitted nonzero WAL segment-byte and segment-inventory limits,
bounded rotation and reopen, checkpoint `ProofOutcome`, handle, exact
checkpoint-capture work basis and policy receipt, capture progress,
publication candidate, covered range, canonical nonempty
`ContiguousRetainedWalTail`, authoritative
`PhysicalMutationBindingCompaction`, and retention eligibility proving that no
WAL deletion removes the last live attempt binding. The exact checkpoint,
compaction cutoff, retained suffix, and live inventory mint a private
per-segment no-last-copy proof; no individual input can mint it alone. Eligible
deletion uses the dedicated `WalReclamation` Signal family, Store semantic
basis, Foundational background policy, scheduled C4 removal, and exact recovery
locator. Only the completed removal receipt advances the oldest inventory
entry. WAL segment sizing and inventory breadth are WAL policy; neither is
inferred from checkpoint memory or retained-tail limits. Binding compaction and
reopen stream one bounded record at a time, and the idempotency policy
independently bounds pending unresolved bindings and total live bindings. One
checkpoint-plus-retained-WAL reopen basis must install the post-reopen
durability owner before mutation authority is available.

**Mechanically forbids:** stop-the-world capture, whole-Store materialization,
fire-and-forget checkpoint work, tail-budget escape, and checkpoint-existence
deletion, all-history idempotency retention, and reclamation before binding
compaction. It also forbids an in-memory encoded compaction copy, authority
distribution from the pre-reopen owner, terminal-fate encoding embedded in the
registry, and WAL deletion outside `wal/reclamation/`.

**Evidence:** 32-times-resident checkpoint pressure siege, exact allocation and
I/O counters, continued foreground progress, tail-bound denial, fresh-process
reopen from compaction plus the post-reclamation retained WAL, deterministic
fail-before and indeterminate-after-effect delete seams, unresolved-binding
retention, expired terminal binding reclamation, and focused partial-authority,
premature-retention, last-binding-deletion, receipt-substitution, and unsafe-
reopen regressions.

**Next may trust:** a published checkpoint can later bound recovery without
having claimed recovery.

**Cleanup:** remove sharp/demo checkpoint execution, duplicate retention
decisions, unbounded capture helpers, generation-zero production shortcuts,
direct WAL initialization that bypasses durability reopen, and every deletion
decision outside the canonical reclamation owner. Preserve no copied deletion
inventory, direct filesystem helper, compatibility recovery record, or
duplicate reclamation queue.

### Phase 7: Join Root Replacement To Namespace Durability

**Becomes true:** one canonical current-root transition consumes exact
WAL/data/checkpoint bases and distinguishes candidate durability, replacement,
namespace durability, and retained old-root truth.

**Consumes:** completed data/checkpoint facts, C.5 root candidate, C.4 atomic
replacement and directory sync, admitted profile.

**Establishes:** root-prepared, root-replaced, namespace-durable, and retained-
root types plus the exact root-publication work basis, partition, and matching
policy receipt.

**Mechanically forbids:** parallel root authority, rename-as-durable,
current-root advance before exact barriers, and early old-root deletion.

**Evidence:** root crash seams, exact artifact manifests, a focused missing-
directory-sync regression, foreign-root/generation tests, and offline root
inspection.

**Next may trust:** current-root authority changes once and only from exact
profile-complete physical effects.

**Cleanup:** delete `PhysicalRootPublicationStore`, `root-publications.log`,
parallel root runtime, and any duplicate current-root cache with authority.

### Phase 8: Publish Typed Outcomes And Cut Over The Ordinary Facade

**Becomes true:** the ordinary facade exposes explicit durability request,
idempotency, deadline/cancellation, completed, proven-no-effect, and
indeterminate outcomes; only completion yields physical acknowledgment.

**Consumes:** full WAL/data/root progression and C.5.1 cancellation/settlement.

**Establishes:** final public mutation and checkpoint APIs, inherited
`ProofOutcome` admission topology, Store-owned `PhysicalMutationHandle`,
typed polling/wait/cancellation, drop-as-observation-abandonment, Store-owned
settlement after caller loss, exact observation, completed-but-unobserved
evidence, one-way
`StoreExecutedBoundaryReceiptEvidence` and diagnostic projections, and
lifecycle drain.

**Mechanically forbids:** generic success/error acknowledgment, hidden weak
durability, automatic indeterminate retry, semantic commit interpretation, and
accepting any Foundational evidence projection back as authority, drop-implies-
cancellation, and caller-handle ownership of settlement.

**Evidence:** compiled caller examples, handle-drop before and after every
effect boundary, typed cancellation at every effect boundary,
acknowledgment-loss seam, stale-generation delivery, public UI compiler tests,
reverse-evidence-projection compile failures, and close with work in every
lifecycle phase and no surviving caller handles.

**Next may trust:** every ordinary caller uses one honest physical outcome
surface.

**Cleanup:** remove or narrow `append_batch(...)->PublishedRecordBatch`, old
acknowledgment types, compatibility wrappers, and alternate ordinary callers.

### Phase 9: Complete Cutover, Delete Islands, And Publish Documentation

**Becomes true:** every retained mechanism participates through the canonical
Store path, every displaced executor or authority is gone, and callers/operators
have one current contract.

**Consumes:** final API, actual Cargo graph, current direct callers, and current
documentation.

**Establishes:** final dependency direction, feature graph, owner READMEs,
physical durability/checkpoint guide, exact aspect-native gate documentation,
audit truth, and roadmap handoffs.

**Mechanically forbids:** legacy features, deprecated aliases, direct executor
callers, certification leakage, stale examples, and undocumented operational
outcomes.

**Evidence:** dependency and feature gates, direct facade compilation, compiled
docs, and focused behavior tests.

**Next may trust:** C.7 has one production authority and one discoverable
caller/operator contract.

**Cleanup:** remove obsolete tests, fixtures, snapshots, closeout artifacts,
temporary adapters, and dependencies exposed by deletion.

### Phase 10: Hostile Direct Tests And C.8 Handoff

**Becomes true:** the joined system survives every named crash seam, bounded
seeded CI schedules, the canonical release schedule, the checkpoint pressure
scenario, and named hardware qualification.

**Consumes:** final source, complete progression, independent inspector,
schedule harness, direct tests, and documentation.

**Establishes:** the one bounded C.8 physical durability handoff, plus counter-backed
`StorePerformanceReceiptEvidence<FoundationalAuthoritativePerformanceClaim>`
for each governed performance claim.

**Mechanically forbids:** same-process inspection, wrong-reason green,
temporary test hooks, and successor access to live runtime state.

**Evidence:** owner, smoke, CI, release, and hardware products; exact replay;
focused crash/recovery behavior; constitution and line-cap gates; and
independent code review.

**Next may trust:** C.8 receives real persisted durability facts rather than
mechanism vocabulary.

**Cleanup:** delete temporary test-only production hooks. Retain only
sealed C.4 yieldpoints/interposers and shared schedule infrastructure with a
continuing responsibility.

## C.8 Successor Handoff

C.7 establishes one bounded, branch-agnostic crash-recovery contract containing
persisted physical facts plus the configuration and resource requirements that
a fresh process must readmit:

- stable Store identity and final C.7 source/profile identity;
- current and immediately previous physical root bases;
- namespace-durable root-publication evidence;
- latest namespace-durable checkpoint identity and exact covered LSN range;
- contiguous retained WAL tail inventory;
- the bounded set of unexpired terminal and every unresolved physical operation
  and idempotency identity, each with its lease frontier, canonical request
  fingerprint, persisted attempt/WAL binding, and compacted fate when present;
- corresponding live completed, proven-no-effect, completed-but-unobserved, and
  indeterminate physical mutation facts;
- exact backend durability profile identity and persisted barrier facts, to be
  matched by fresh backend qualification;
- bounded recovery allocation requirements from C.6, not a live or reusable
  allocation grant; and
- classified staged or partial artifact residue.

C.8 independently reopens the persisted artifacts that carry those facts and
uses them to decide physical source precedence, redo, root selection, and
operation-fate reconciliation in a fresh process. The in-memory
`PhysicalDurabilityRecoveryHandoff` produced during orderly C.7 closeout is
closeout observation and certification evidence; it is not a crash-surviving
C.8 input, must not be serialized as one, and cannot substitute for artifact
reopen.

C.8 does not receive every historical physical mutation. Expired terminal
bindings protected by the required later namespace-durable checkpoint are
absent by construction; unresolved bindings remain bounded by C.7 admission
and cannot be reclaimed until C.8 produces a lawful fate.

C.8 does not receive:

- live `ServingPhysicalRuntime`;
- buffer-pool contents, frames, leases, or clean authority;
- Signal graph, scheduler state, queues, callbacks, or counters;
- a writer-owned decoded WAL or page model;
- C.7 acknowledgment construction;
- Foundational policy, performance, or executed-boundary evidence as
  replacement authority for persisted physical facts;
- semantic mutation, branch, MVCC, Query, or writer authority; or
- a claim that indeterminate work has already been resolved.

The orderly closeout handoff remains constructible only from the final C.7
closeout progression. The C.8 recovery entry does not accept that value. An
evidence bundle, report, digest, copied field set, serialized handoff, or public
constructor cannot mint the persisted authority C.8 must rediscover.

## Milestone Must Ship

C.7 is incomplete without:

- one admitted physical durability policy under a sealed C.4-derived
  `PhysicalDurabilityAdmissionBasis`;
- one versioned canonical `PhysicalMutationRequestFingerprint` separated from
  its persisted `PhysicalMutationAttemptBinding`;
- one identity-preserving Store durability progression;
- real WAL append through C.5.1 and C.4;
- compiler-visible WAL-before-data and pageLSN binding;
- group commit with distinct member truth;
- bounded fuzzy checkpoint capture and contiguous retained WAL tail;
- exact root replacement and namespace durability;
- completed, proven-no-effect, and indeterminate public outcomes;
- physical acknowledgment available only from exact completion;
- bounded queue, memory, WAL-tail, and amplification contracts;
- bounded durable-generation idempotency leases and checkpoint-side binding
  compaction, with expired-key denial and no last-authority WAL deletion;
- current read-only observation and operator checkpoint lifecycle;
- exact Store aspect-native work bases for policy binding, WAL append,
  barriers, checkpoint capture, and root publication;
- matching `FoundationalPolicyAdmissionReceipt` scheduler admission and
  one-way Store performance/executed-boundary evidence projections;
- removal of direct WAL, durability-runtime, checkpoint, and root execution
  islands;
- updated public/operator docs, owner READMEs, audit, and roadmap handoff;
- compile-time authority and progression denial;
- actual process death at every named seam;
- independent offline artifact observation;
- bounded replayable CI scenarios and canonical release execution at all crash
  seams;
- focused regressions retained when they protect a real failure mode; and
- the sealed C.8 crash-recovery contract plus its orderly-closeout observation
  handoff.

## Must Preserve

- C.4 remains the one media authority and fault-observation boundary.
- C.5 physical format and stable Store/record identities remain canonical.
- C.5.1 remains the one work, Signal, scheduler, executor, settlement, and
  shutdown topology.
- C.6 retains pool, lease, dirty, clean, pressure, and writeback authority.
- WAL remains specialized artifact meaning, not Store or semantic authority.
- Backend profiles remain concrete physical capability facts.
- Signal remains derived and disposable.
- Test infrastructure remains read-only or limited to sealed fault injection.
- Part I remains branch-, MVCC-, Query-, and semantic-writer-agnostic.
- Stores larger than memory, bounded resources, real crashes, independent
  observation, and direct fault assertions remain non-negotiable.

## Explicit Non-Goals

C.7 does not:

- implement C.8 recovery source selection, redo, or fresh-process fate
  reconciliation;
- implement C.9 integrity admission or corruption repair;
- implement C.10 stable-reader leases, reclaim, fairness, or maintenance QoS;
- implement C.11 index/blob durability policy beyond the generic physical
  progression they will later consume;
- formalize all transitions for C.12;
- advance semantic branch heads or release semantic writers;
- define Query acknowledgment or semantic transaction commit;
- infer semantic liveness from root or WAL retention;
- expose raw WAL, page, checkpoint, root, backend, Signal, scheduler, or pool
  mutation authority; or
- retain compatibility with displaced unreleased APIs.

## Acceptance Evidence

Closeout evidence includes:

- Git revision, built target and feature set, Store, runtime, format, and
  profile identities;
- admitted durability and checkpoint policy;
- workload seed, schedule seed, crash seam, scale tier, and exact rerun command;
- complete operation/idempotency/group/member identity trace;
- request-fingerprint canonical version, input-basis identity, golden-vector
  digest, idempotency lease frontier, persisted attempt/WAL binding, and current
  binding-compaction identity;
- C.4 durability admission basis and exact backend capability-claim identities;
- C.7 Foundational work-basis contract revisions, roles, masks, families, and
  partitions;
- scheduler `FoundationalPolicyAdmissionReceipt` identities and derived
  `StorePerformanceReceiptEvidence<FoundationalAuthoritativePerformanceClaim>`
  for governed cost claims;
- one-way `StoreExecutedBoundaryReceiptEvidence` only where a support,
  certification, or cross-crate boundary consumes it;
- WAL frames, ranges, barriers, rotations, and durable-prefix evidence;
- page/extent effects and pageLSN bindings;
- C.6 dirty/writeback settlements;
- checkpoint capture, publication, covered range, and retained-tail evidence;
- binding-compaction coverage, expiry, unresolved-retention, and WAL-reclamation
  evidence;
- root candidate, replacement, namespace durability, and retained-root facts;
- completed, proven-no-effect, indeterminate, cancellation, timeout, stale,
  and completed-but-unobserved outcomes;
- exact file, directory, sync, byte, frame, queue, memory, and amplification
  counters;
- exact per-case world, producer, baseline-observer, serving-writer,
  post-interruption-observer, fresh-reopener, evidence-binding, and total timing;
- independent inspector and fresh-reopener assertions in direct tests;
- deletion, dependency, and feature checks;
- documentation compilation and link evidence;
- bounded seeded CI scenarios and the canonical all-seam direct release test;
- C.8 crash-recovery contract proof and orderly-closeout handoff construction
  proof, with compile-time denial at the fresh-process C.8 entry.

Current validity is determined by the current tree and executed commands. A
historical result from another revision, backend profile, or harness does not
substitute for rerunning the affected direct evidence.

## Closeout Gate

C.7 closes only when one real production progression owns every physical
durability edge; exact mutation identity survives WAL grouping, barriers, data
settlement, checkpointing, root publication, cancellation, crash, and
acknowledgment; no page outruns durable WAL; no root is promoted without the
exact barriers required by its admitted profile; no ambiguous effect is called
success or no-effect; checkpoint and retained-WAL work remain bounded under
continued foreground mutation; every displaced execution or authority island
has been deleted; focused fresh-process tests exercise every named crash seam
under direct replay where a seed is used; documentation teaches the real API and
operator lifecycle; and C.8 receives persisted physical truth without live
runtime, pool, Signal, scheduler, or semantic authority.
