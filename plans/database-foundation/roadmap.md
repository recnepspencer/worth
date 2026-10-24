# Database Foundation Roadmap

Make Store the durable foundation for Relational and the production application
host. Preserve the [Store vision](../worth-store/worth_store_vision.md):
canonical commits, structural deltas, branch history, lineage, schema evolution
and retention remain meaningful across crashes and restarts.

Milestones are verification boundaries, not sequential build phases.
Implementation may proceed across several milestones at once. A milestone
closes when the crash-consistency oracle and its completion requirements cover
its behavior on the final code. Milestone 3 delivers the first durable
application journey. Milestone 4 makes the database suitable for sustained use
with data larger than memory. Milestone 5 completes recovery of the supported
application runtime.

The shared requirements apply to every milestone. The crash-consistency oracle
comes first, because it decides whether the implementation it judges is
correct.

## Shared requirements

**Canonical artifact ownership**

- Relational's canonical commit envelope is the durable semantic authority for
  graph commits. Store persists the envelope Relational produces. It does not
  define a parallel envelope, a second codec or its own reading of commit
  meaning.
- Relational owns the envelope's format, versioning and readmission. Bytes
  Store recovers become authority only after Relational readmits them under the
  schema boundary they record.
- Store owns what it adds: journal ordering, physical placement, the binding
  between a commit and its selected physical state, integrity and checkpoint
  coverage.
- World, Query and Signal follow the same rule for the facts they persist
  through Store. Each owner defines and readmits its own durable artifacts.
- Store tests may use a narrow payload only for purely physical claims such as
  byte integrity, allocation or journal ordering. Semantic publication and
  recovery claims use real owner artifacts.

**One durable lifecycle owner**

- One component owns the durable commit decision and recovery sequencing for
  an application home: when a commit becomes durable, the order in which owners
  recover and when service may begin. Owners never coordinate durability with
  each other directly.
- Applications open a home through one Query host call. They never construct,
  open or synchronize a Store runtime themselves.
- The Store vision's durable mode places this ownership in Store, with the
  runtime inside it. Milestone 3's host-opened composition places the Query
  host on top. Milestone 1 records which component owns open, commit and
  recovery sequencing, and how the other reaches it, before implementation
  depends on either. The first implementation must not decide this implicitly.

**Crash-consistency oracle**

- Build a differential crash harness before the durable implementation it
  judges. Each milestone extends its coverage; no milestone closes on
  hand-picked crash points alone.
- The reference is the memory-only runtime executing the same operation
  sequence, and its committed history is the expected result. The oracle
  disputes persistence and recovery, not graph or application semantics, so
  sharing the semantic engine does not make it self-certifying.
- Interrupt at every durable I/O boundary an operation crosses, including each
  write, flush, rename and checkpoint step. After recovery, observed state must
  equal the reference state at a committed prefix containing every
  acknowledged commit.
- Retries after recovery resolve to original occurrences. An unresolved
  submission stays unresolved until its owner establishes its fate. Recovery
  never manufactures a committed receipt or repeats an effect.
- Interrupt recovery itself. A second or third crash must reach the same
  conclusion as the first.
- Interrupt at a media boundary for breadth, and kill real processes on real
  filesystem media for the durability claim. A substituted media adapter
  narrows what a run can claim; it does not replace the filesystem lane.
- Generate seeded, reproducible operation sequences alongside the fixed
  journeys. A failing seed must reproduce deterministically.
- Interruption controls follow the repository's control rules: `cfg(test)`
  hooks or sealed owned handles, never Cargo features. They must not change
  production semantics or composition.
- Run cheap seeds in focused lanes and broad campaigns in scheduled lanes with
  stated resource budgets. The harness is ordinary test infrastructure, not a
  proof ledger.

## Milestone 1: Canonical durable commits

Store can durably publish and recover a complete semantic commit. This is the
durability boundary consumed by Relational, rather than an application result
or a replacement for semantic transaction validation.

### Requirements

**Canonical artifacts and their physical representation**

- Persist Relational's canonical commit envelope, its structural deltas and
  the selected successor as Relational produced them, together with their
  stable identities and schema/format versions.
- Preserve explicit parent ordering, lineage and schema-boundary references.
  Durable representation must retain the meaning needed for later history,
  reconstruction and replication.
- Bind the selected physical state to the canonical commit that produced it.
  A root address or a successful physical write cannot stand in for that
  semantic association.
- Keep indexes, materialized snapshots and physical read layouts
  reconstructible from surviving canonical coverage. A layout optimization
  must not silently become the only representation of truth.
- Validate integrity and format at the appropriate physical and semantic
  boundaries. Checksums establish byte integrity; they do not establish
  semantic authority, tenant admission or application success.

**Durable publication and outcome resolution**

- Support one commit's complete batch through the existing physical media,
  allocation, WAL, integrity, checkpoint and publication owners.
- Establish one durable decision for that commit. A crash cannot select some
  changes from the predecessor and others from the successor.
- Associate every potentially effectful submission with a stable attempt and
  resolvable fate before its effects can become orphaned.
- Distinguish completed, proven-no-effect, in-progress and indeterminate
  outcomes. Losing observation of a submission must not make it safe to repeat.
- Acknowledgment must meet the qualified backend's durability contract. An
  interrupted or failed durability barrier cannot be reported as success.
- Allocation, cancellation and deadline failures must retain the actual effect
  posture. A refusal before effects must leave no semantic commit; a failure
  after possible effects must preserve recovery.

**Recovery and checkpoint coverage**

- Recover a store killed before its first checkpoint as well as one killed
  above a completed checkpoint.
- Distinguish an absent checkpoint from an unreadable, corrupted or
  incompatible checkpoint. Damaged existing data must not be treated as an
  empty home.
- Select a coherent checkpoint and its admitted journal tail. Recovery must
  preserve the relationship among canonical artifacts, selected state and
  submission fates.
- Bound ordinary recovery by checkpoint coverage and the admitted subsequent
  journal, rather than replaying the complete application history.
- Preserve unresolved submissions across a second crash during recovery.
  Reopening repeatedly must not change the conclusion or repeat an effect.
- Retain canonical coverage required for recovery before allowing covered
  journal or storage to be reclaimed.

**Access and storage growth**

- Support point and bounded range access needed to retrieve canonical
  artifacts and selected committed state.
- Persist changes at their honest physical granule. A local update must not
  require rewriting the complete database or acknowledging every changed node
  as a separate semantic commit.
- Support immutable data sharing and values larger than one physical page.
  Large values must have bounded read/write access rather than requiring one
  contiguous allocation.
- Preserve existing protection of selected roots and reader-visible bytes
  while checkpoints or physical maintenance proceed.
- Settle the reference and allocation identities required for later branch
  sharing and reclamation without requiring those complete user journeys here.

### Completion requirements

Real filesystem interruption tests must distinguish the prior complete commit,
the new complete commit and an unresolved submission. They must cover the
first-checkpoint boundary, durable publication before acknowledgment, and a
second interruption during recovery.

Recovered artifacts must agree with selected state through observations that
can expose an incorrect binding. Unsupported or damaged artifacts must refuse
before serving. A caller must be able to resolve a prior submission without
submitting its effect again.

Publication and recovery claims use real Relational canonical envelopes. The
crash-consistency oracle runs against single-commit publication, the
first-checkpoint boundary and interrupted recovery. The lifecycle-ownership
decision is recorded before Milestone 2 or 3 code depends on it.

## Milestone 2: Relational cutover to Store

Relational uses Store for durable committed state while retaining graph
meaning, transaction validation, identity and MVCC. Its ordinary transactions
and reads use this integration directly.

### Requirements

**Transaction semantics**

- Connect ordinary prepared Relational transactions to Store's durable commit
  boundary. Preparation continues to validate the transaction against its
  exact admitted basis and declared invariants.
- Preserve entity/relation identity, aspect semantics, relation integrity,
  unique-field enforcement, savepoints and canonical change emission.
- Preserve stale-candidate and conflicting-write refusals. Reopening or
  introducing storage must not turn a previously prepared candidate into
  authority over a different head.
- Carry storage failures through the existing typed transaction outcomes.
  Reads or validation cannot silently skip unavailable data and approve an
  incomplete transaction.
- Keep pre-effect denial distinct from a commit whose durable publication
  occurred but whose live installation or settlement is unfinished.
- Recover that latter commit through its original identity. Do not roll back
  its meaning, issue a replacement commit or re-execute a mutation handler.

**Durable graph state**

- Persist records, relations, endpoints, aspects and values needed to recover
  the supported graph exactly.
- Preserve schema images, aspect contracts, symbols and the schema boundary
  under which each retained canonical commit is interpreted.
- Preserve canonical commit history, ancestry, lineage and the selected
  branch state. Reconstructed state must remain inspectable through
  Relational's normal semantic surfaces.
- Persist the identity allocation facts needed to prevent aliasing: generation
  high-water marks, allocation frontiers and any outstanding reservations
  whose loss would permit unsafe reuse.
- Preserve the binding between committed data and indexes required for
  correctness. Rebuildable indexes retain their derived classification and
  explicit unavailable/rebuild behavior.
- Persist pending publication or settlement facts needed to finish a
  supported transaction after a crash.

**Owner reconstruction**

- Recover the original logical record and commit identities through the
  corresponding owner. Process-local capability values are not durable
  authority.
- Issue fresh live owner capabilities after open. Handles retained from the
  previous incarnation must not authorize reads or writes against the new one.
- Continue allocation and publication from safe recovered frontiers.
  Deletion, abandoned preparation and process death must not cause identity
  values to be reused incorrectly.
- Reject incompatible schema or canonical material before admitting service.
  Recovery cannot reinterpret old values under a different meaning.

**Read and working-state integration**

- Introduce the production storage access required by the supported graph,
  including fallible reads of committed data.
- Preserve immutable observations and exact-basis reads while later
  transactions commit.
- Support cold access and guarded native working state without a second
  independently authoritative heap database.
- Do not make every commit or reopen materialize all records or all history.
  Broader cold-query and working-set coverage is completed in Milestone 4.
- Preserve current issued/context partition placement and record identity.
  Residency changes must not relocate semantic identities.

**Cutover**

- Replace Relational's segmented-filesystem log, checkpoint storage and
  recovery path with Store. Keep the canonical envelope codec and readmission:
  they define the artifact Store persists.
- Carry Relational's existing durability contract tests forward as oracle
  cases where their behavior still applies. Delete tests that only exercise
  the retired storage.
- Remove obsolete filesystem implementations, configuration and examples as
  their callers cut over. There must be one supported durable publication
  and recovery path for the converted runtime.
- Preserve explicit memory-only execution and its semantic guarantees.
  Memory-only tests do not qualify filesystem durability.
- Update Relational's public durability and owner-component guidance to
  explain commitment, recovery and typed failure behavior.

### Completion requirements

An ordinary transaction must survive process death and fresh reconstruction
with exact records, identities, schema and canonical history. Cover both a
crash before commitment and one after durable publication but before live
installation or settlement.

Recovered state must support another ordinary edit and safe allocation.
Retained observations must remain stable across later commits. A stale
candidate, unavailable required read or invalid transaction must not publish.
The retired filesystem path must no longer be an alternative durable engine.

The oracle runs generated Relational transaction sequences, including the
supported branch and schema operations, against the memory-only reference.

## Milestone 3: Production host and original-result retry

A file-backed application survives lost responses and returns its original
committed result after restart through the normal Query host.

### Requirements

**Application home and lifecycle**

- Open and recover the durable application through the production host using
  one Store instance and the existing semantic owners.
- Initialize only a demonstrably new home. An existing home must be admitted
  and recovered before exposing application service.
- Prevent competing writable opens from independently acknowledging writes to
  one home. A previous owner's capabilities must remain fenced after reopen.
- Supply the installed program, application build identity, trusted runtime
  services and actual execution authority through the ordinary host path.
- Refuse an incompatible program, unsupported durable state family or
  incomplete recovery before exposing a partially resumed application.
- Make durable acknowledgment independent of orderly close. Close must stop
  admission and deal with accepted work without changing the meaning of
  previously acknowledged results.
- A refused close must retain a usable application or explicit recovery
  posture. It cannot silently seal a runtime that the caller was told remains
  open.

**Application occurrence and history**

- Persist World product identity, branch identity/head, canonical application
  occurrence and the association with its component commits.
- Preserve the distinction between a completed Relational commit and a
  completed application publication.
- Recover the exact original occurrence, including the identity and semantic
  result facts needed by ordinary receipt and history APIs.
- Reissue live access through fresh owner admission. Recovery must not
  pretend an old process's capability or pointer is still valid.
- Retain mandatory recovery associations independently of caller-held
  handles and response delivery.

**Idempotency and original-result recovery**

- Persist the idempotency key's intent, scope, branch affinity and original
  outcome association through the owning semantic contracts.
- Retry after response loss must resolve the original application occurrence,
  rather than return only its graph row or a new receipt for a repeated edit.
- A completed occurrence cannot depend on a receipt cache in the original
  process to answer an admitted retry.
- Reuse with changed intent or foreign affinity must be rejected under the
  existing idempotency contract.
- Concurrent or repeated retries must not create another committed occurrence
  for the same admitted intent/key.
- Define result-retention behavior. Optional expired result material may be
  unavailable, but expiry must not turn a performed operation into permission
  to execute it again.

**Interrupted application publication**

- Recover pending settlement and unpublished owner custody created by the
  supported operation, including crashes after an owner effect but before
  product movement.
- Preserve existing partial and indeterminate outcomes. Physical root
  publication does not, by itself, establish composite application success.
- Resolve interrupted associations through the existing owners; do not add a
  competing transaction coordinator, idempotency table or recovery authority.
- An attempt with proven no effect may execute once on retry. An unresolved
  effect must retain its recovery posture until its fate is established.
- Limit initial operation support honestly. Unsupported effect combinations
  must refuse before their first effect, and opening a home with unsupported
  mandatory obligations must refuse before serving.

**Public workflow**

- Provide one ordinary home-opening path through the Query host. Applications
  must not manage or synchronize a separate Store runtime themselves.
- Replace superseded open/checkpoint-transport entry paths as their callers
  cut over while preserving supported program adoption.
- Update the application guide, capability index, host documentation and
  executable examples so the durable workflow is discoverable from the
  normal entry points.
- Explain commitment, retry, close, unsupported homes and recovery at the
  caller's level without requiring physical storage knowledge.

### Completion requirements

The decisive journey starts at the real production host on filesystem media:

1. Perform an ordinary mutation with an idempotency key.
2. Let it durably complete, lose its response, then kill the process without
   closing it.
3. Reopen the same home in a fresh process and retry the same request.
4. Return the original occurrence and exact branch state without invoking
   the mutation again or producing a duplicate domain effect.

The oracle then runs generated host operation sequences with idempotency keys
and response loss, interrupting at every durable boundary. These include
before commitment, after durable publication, between owner effects and
product movement, and before response delivery. Check the distinct
no-effect, committed and unresolved outcomes rather than expecting a committed
receipt for an attempt that never committed.

The journey must work on both sides of the first checkpoint. It must reject
changed-intent retries, competing writable ownership and stale capabilities,
and recovered service must support a subsequent ordinary mutation.

## Milestone 4: Database scale, branches and sustained operation

The durable database supports branching, historical access and repeated use
with data larger than memory. This is the sustained-use gate for the owner
set supported by Milestone 3.

### Requirements

**Durable branch lifecycle**

- Persist creation, fork, advancement, archive and deletion together with
  explicit ancestry, selected roots and branch identity.
- Fork by sharing immutable base data. Persisting branch metadata is allowed;
  copying the full base state is not the branch creation strategy.
- Preserve independent changes on divergent branches through restart.
  Equal-looking values or shared physical pages must not collapse their
  distinct semantic histories.
- Retire a deleted branch's name and drop its selected root coherently.
  A crash cannot resurrect it or make its identity available for unsafe reuse.
- Preserve retained observations of an older branch state when a current
  branch advances or is deleted, subject to the existing retention contract.
- Allow independent branch preparation to make progress without a whole-Store
  preparation lock. Final publication must still preserve lawful ordering.

**Historical and schema-aware access**

- Read retained historical records, versions, lineage and schema boundaries
  through the ordinary semantic surfaces.
- Bind every historical access to its exact retained basis. A continuation or
  cursor must not silently rebind to the latest branch head after restart.
- Preserve the difference between unavailable retained history, unsupported
  historical meaning and corrupted storage.
- Access an old version without replaying all intervening application
  operations or promoting all history into the native working set.
- Retain enough canonical coverage to reconstruct each promised view, even
  when physical indexes or materializations are discarded.

**Cold data and the native working set**

- Complete the cold-read and guarded working-state coverage introduced by
  Milestone 2, including validation reads, graph traversal, kind/adjacency
  access and historical selection.
- Serve a database larger than the resident allowance without loading the
  database, branch or full history into memory.
- Admit active native data in bounded chunks. A placement group larger than
  memory remains usable without splitting or changing its semantic identity.
- Keep head working data separate from historical versions. Opening a branch
  or reading an old version cannot pin its entire contents.
- Preserve the exact basis of warm data. Updates, eviction and rewarming
  cannot make a stale native representation appear current.
- Carry storage, cancellation and genuine memory-exhaustion failures through
  the appropriate typed outcomes. An unavailable guard or cursor must not
  become a panic, an empty result or permission to skip validation.
- Bound large-value access and in-flight fills as well as retained residency.
  Streaming cannot hide a whole-value allocation or unbounded queued work.

**Retention and safe reclamation**

- Coordinate semantic retention with physical reclamation, including every
  live branch, retained historical root, checkpoint and recovery obligation.
- Reader and cursor lifetimes must protect the bytes they can still observe.
  Closing or deleting a branch cannot release a sibling's shared data.
- Preserve canonical coverage before pruning a journal, commit representation
  or materialized base. An accelerator cannot become the sole surviving truth.
- Reclaim data that is genuinely unreachable and no longer retained.
  Repeated editing and branch deletion must return eligible space.
- Protect allocation identity across address reuse. Resident buffers,
  persisted references and recovery must distinguish an old allocation from
  a new page at the same location.
- Survive interruption during reclamation without losing reachable truth or
  permanently treating already-released data as live.
- Keep reclamation and maintenance from turning a local foreground edit into
  an unbounded scan of total retained state.

**Operational behavior and guidance**

- Preserve current results under concurrent readers, independent branch
  preparation, checkpointing and supported maintenance.
- State the growth behavior of local edits, forks, cold reads, recovery and
  reclaim in terms of the data they actually touch.
- Prove meaningful scaling behavior with headroom or equivalent-path
  comparisons. Avoid fixture-specific page totals and arbitrary aggregate
  quotas as acceptance criteria.
- Update branch/history, cold-access and retention guidance, including
  lifecycle, pressure, unavailability and safe recovery behavior.

### Completion requirements

Use data larger than the resident working set. Increase total database size
and retained history while keeping a local edit's semantic scope fixed; its
work must not become proportional to all stored data or history.

Fork, edit and advance both sides, retain historical readers, delete branches,
checkpoint, crash and reopen. Values, ancestry, lineage and retained views must
remain exact. Eligible space must be reclaimed without invalidating a live
reader, sibling branch or recovery obligation.

Cold and warm access must agree. Independent branches must make progress, and
real exhaustion or interrupted access must preserve typed outcomes and
committed truth.

The oracle runs generated fork, edit, delete, checkpoint and reclamation
sequences, comparing cold and warm reads against the reference, and interrupts
reclamation as well as publication.

## Milestone 5: Complete runtime recovery

The supported application runtime resumes authored state, history and mandatory
obligations through the same durable path. Its recovered owners can continue
working, rather than merely display saved data.

### Requirements

**Signal and mixed-owner publication**

- Persist and reconstruct Signal authored definitions, branch state and the
  original-owner facts needed to continue editing and evaluation.
- Keep authored source separate from derived results and runtime scheduling
  state. Derived values cannot become an authoritative replacement for lost
  definitions.
- Recover mixed Relational/Signal publication at the actual owner boundaries.
  A crash between effects must preserve which owners acted and whether the
  product head moved.
- Preserve pending settlement and unpublished owner recovery under their
  existing owners. Cancellation or caller abandonment cannot erase performed
  effects.
- Recover World branches, canonical composite history, retained owner
  associations and safe identity issuer frontiers across repeated restarts.

**Programs and long-running application work**

- Persist the application program revision and support lifecycle associated
  with each retained branch and occurrence.
- Resume program adoption and its recovery obligations under the existing
  admission and migration contracts.
- Preserve workflows, their progression, required approvals and the obligations
  needed to resume after process death.
- Persist mandatory in-flight work even when clean close would have refused
  it. Reacquire process resources through fresh authority instead of reviving
  expired handles.
- Resume or explicitly terminate conditional work and managed recovery
  according to their existing lifecycle contracts; loss of process memory is
  not an implicit successful completion.

**Dispatch, inbound results and external custody**

- Preserve dispatch outbox entries, correlation, idempotency associations,
  inbound receipts and terminal lookup needed by supported external work.
- Recover completion and transport custody independently of the caller that
  initiated the operation.
- A crash after dispatch but before recording the response must retain an
  indeterminate external posture until the original owner resolves it.
- Do not repeat an externally completed effect merely because its local
  response was lost. Do not claim remote exactly-once behavior beyond the
  external owner's admitted contract.
- Recover acknowledgments, duplicate deliveries and required follow-up work
  without changing the identity of the original logical effect.
- Keep unknown external effects distinct from proven no effect, compensation
  and completed work.

**Correction and continued original-owner editing**

- Preserve 9.18 correction occurrences, alternatives, causal links and
  retained original meaning under their retention and disclosure contracts.
- Recover native authored ownership material, including membership and
  incident relation facts required to continue the original owner's work.
- Preserve the distinction between authored correction and required derived
  settlement, including recovery when only one has completed.
- After reopen, support a dependency edit followed by original-owner
  advancement. Reconstruction cannot replace the owner or silently create a
  new authoritative object to make the edit succeed.
- Preserve divergent-branch correction meaning and the selected lineage.
  Recovery must not consume or substitute an alternative from another branch.
- Return typed unavailability when optional correction material has expired;
  do not guess an inverse or reinterpret retained bytes.

**Build identity and derived recovery**

- Preserve authored source and references when the application build identity
  changes. Mark incompatible derived outputs stale under their existing reuse
  contracts.
- Recover or regenerate derived material through the normal demand,
  dependency, invalidation and settlement owners.
- Destroying derived caches or durable materializations must not lose
  authored truth, canonical history or mandatory custody.
- Do not eagerly regenerate every output merely because the application
  opened. Required-output and active-demand obligations retain their actual
  execution and failure semantics.
- Same-build reuse must preserve ordinary equivalence and ownership checks;
  persistence does not grant unconditional reuse.

**Complete public cutover**

- Audit production-owned state by its survival requirement: authored truth,
  mandatory obligations, optional retained material, derived state or
  process-local resources.
- Remove remaining competing durable publication/recovery paths and correct
  capability claims that overstate what reopening resumes.
- Preserve the declared contracts of explicit embedded-save and absent modes
  while keeping their evidence distinct from per-operation durable mode.
- Update the ordinary application, workflow, external recovery and correction
  guides with supported restart behavior, failures and valid next actions.
  Executable examples must use the public host and its real owners.

### Completion requirements

Extend the production host journey to mixed-owner publication, a deferred
settlement, a workflow mid-run, ambiguous external completion and correction
on a divergent branch. The oracle runs generated sequences mixing these
operations, interrupting at every durable boundary and reopening in a fresh
process.

Recover original occurrences and custody without repeating performed effects.
Then edit a dependency and advance the original owner. Discarded derived state
and a changed application build identity are oracle dimensions, not single
cases; authored meaning, references and mandatory obligations must remain
correct under both.

This milestone closes the runtime database cutover. Advanced replication,
backup/PITR, security lifecycle, bulk ingest and broader platform qualification
remain separate work; their owners and return triggers are recorded in
[deferred work](../deferred-work.md#worth-store).
