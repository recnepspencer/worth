# Storage Foundation S.10 Engineering Spec: Operational Recovery Without Trusting The Live Store

> **Status: paused and unclosed.** The
> [Physical Foundation Reconstruction Roadmap](physical-foundation-reconstruction-roadmap.md)
> reopens S.1 through S.9 and must close through C.13 before S.10 implementation
> resumes. Existing S.10 code is unadmitted substrate until it binds to the
> reconstructed sealed runtime, real media path, fresh-process recovery, and
> independent physical evidence. Historical S.10 tests and phase progress do
> not bypass that gate.

## Goal

Make Worth Store operable under real production damage by turning backup,
offline verification, restore, point-in-time recovery, repair, replica bootstrap,
disaster recovery, and forensics into typed, crash-safe authority protocols.

S.10 is complete only when an operator can begin with untrusted files and a
possibly unavailable live runtime, produce the maximally justified
classification of bytes and frontiers (including bounded unknown and
indeterminate regions), execute only a narrowly authorized owner-plan DAG, and
return a recovered store to current authority through independent
post-verification, fresh cutover authorization, durable fencing, and explicit
readmission.

## Why This Milestone Exists

S.9 checks the laws that keep physical truth coherent under crash and
concurrency. Those laws are not yet an operational recovery system. Production
damage introduces foreign media, partial copies, stale plans, human actions,
missing custody, divergent histories, and long-running work that can crash at
every boundary. If S.10 merely wraps existing recovery calls, operations will
be able to launder observations into authority or report successful byte
movement as successful recovery.

S.10 therefore builds the operational protocol before S.11 adds provider-backed
identity, encryption, key lifecycle, and tamper-evident audit, and before S.12
qualifies the complete foundation across hardware and backend profiles.

## Governing Summaries

- `MENTALITY.md` protects foundation-first correctness under adversarial
  pressure. S.10 must solve independent recovery truth and crash-safe authority
  progression before shipping convenient operator commands.
- `arch_laws.md` protects compiler-visible proof and authority transitions.
  Every operational phase consumes the sealed artifact produced by the prior
  phase; execution accepts lowered owner plans, never raw intent.
- `composition_laws.md` protects named, reviewable responsibilities. Backup,
  verification, restore, repair, promotion, forensics, and audit remain distinct
  workflows rather than branches of a generic recovery manager.
- `domain_structure_laws.md` protects ownership and dependency direction.
  Domain owners execute their own mutations and issue receipts; operations may
  orchestrate those owners but may not absorb their law.
- `perf_laws.md` protects visible and testable cost. Streaming inspection,
  reconstruction, materialization, and certification have distinct cost
  surfaces, bounded memory, and operation-bound counters.
- `dx_laws.md` protects responsible operability. Destructive work exposes
  inspect, explain, plan, authorize, execute, resume, verify, and promote
  phases through long-running sessions instead of a deceptively simple
  `repair()` call.
- `physical-database-roadmap.md` protects the physical database foundation. It
  places operational recovery after checked protocol law and before security
  hardening and cross-profile certification.
- `worth-proof/README.md` protects static progression law. S.10 reuses its
  stages, bases, freshness, witnesses, checked outcomes, and structural
  collections around Store-owned payloads without turning Proof into a runtime
  workflow or owner-receipt factory.
- `worth-foundational` orientation and family READMEs protect shared boundary
  meaning. S.10 lowers stronger Store artifacts into canonicalization,
  profiles, boundary categories/roles, diagnostics, lineage/support truth, and
  performance vocabulary only at explicit projection and certification seams.

## Adversarial Constraint

After any crash cut, corruption pattern, partial restore, operator
interference, stale plan, missing custody condition, rollback, primary-site
loss, corrupt or duplicated control record, unreachable fencing authority, or
authority change during staging, no byte set may become current authority
unless an independent verifier and the responsible owner subsystems produce
scope-bound, generation-bound, frontier-bound receipts proving the maximally
justified result: what survived, what changed, what is degraded, what remains
quarantined, and what is still unknown or indeterminate.

The system must preserve this law for stores larger than memory and while
foreground work, checkpointing, compaction, scrub, replication, blob movement,
and reclaim interfere with online operations.

## Product Decision Lock

- The live Store runtime is never an oracle for offline verification.
- Offline inspection is read-only and cannot mint repair, publication, or
  current-authority capability.
- A backup is a stable physical cut, not a directory copy or byte count.
- `MaterializedBackupBundle` is not `StructurallyVerifiedBackupBundle`;
  structural verification is not custody qualification or production restore
  admissibility; none is `RestoreDrillCertified`.
- Restore, point-in-time recovery, rollback, and replica promotion are different
  source and failure topologies. They do not share a generic `RestoreSource` or
  generic action executor. Import is outside the S.10 operational workflow; its
  existing foreign-media admission boundary remains distinct and cannot be
  treated as restore or rollback authority.
- Wall-clock time may select PITR candidates but can never authorize a recovery
  frontier. The executed target is an exact checkpoint, WAL, and acknowledged
  durability frontier.
- Restore and PITR never overwrite current authority in place. They stage under
  non-current authority and publish through copy-on-write cutover.
- Repair is split by authority topology. Current-authority-preserving derived
  maintenance may build a replacement derived generation and atomically swap it
  through that artifact owner. Any repair that changes authoritative bytes,
  quarantine reachability, lineage, root, or authority posture uses isolated
  copy-on-write staging and the same post-verification/cutover discipline as
  restore.
- `worth-store-operations` owns workflow and operator intent only. Physical,
  integrity, recovery, layout, blob, isolation, authority, backend, and
  replication owners execute their own plans and issue owner receipts.
- Destructive authorization binds the canonical fully lowered owner plan, not
  a higher-level candidate. Lowering may not add an owner, effect, footprint,
  source, target, or cost after authorization.
- Worth Proof carries static progression, basis freshness, checked outcome, and
  fixed-shape law around Store-owned payloads. It never becomes the workflow
  engine or the owner of Store recovery meaning.
- Worth Foundational is used only when a stronger Store artifact is deliberately
  lowered into shared boundary meaning. A Foundational report, receipt,
  diagnostic bundle, lineage attachment, or digest can never be raised back
  into a stronger Store execution or authority type by convention.
- Operator identity, readiness, and an external assertion are not operation
  authorization. Authorization is plan-specific, store-specific,
  tenant/key/custody-specific, expiring, revocable, and single-use or
  replay-safe.
- Staging authorization may be revoked until its declared irreversible staging
  effect. It never authorizes publication. After staging and post-verification,
  operations must resolve the candidate again against current authority,
  explain the data-loss and lineage delta, lower a cutover-specific owner DAG,
  obtain fresh cutover authorization, and establish a write fence or quiescent
  cut before publication.
- When no production authorization provider can satisfy the S.10 port, the
  destructive lane returns typed `Unsupported` or `Unavailable`; it never
  substitutes a local boolean, role string, readiness witness, or test key.
- S.10 emits canonical structured operational audit records. S.11 adds
  provider proof-of-possession, cryptographic protection, and tamper-evident
  chaining to those records rather than creating a second audit ontology.
- S.10 requires a physically independent durable operational control store and
  a real fencing authority. If either cannot establish one current record
  generation and one live serve lease/token, destructive resume, cutover, and
  promotion return typed `Unavailable`, `Unsupported`, or `Indeterminate`;
  local process memory, timestamps, and operator assertion never substitute.
- Every primary serving path is gated by a live `PrimaryServeLease` or equivalent
  storage token. Lease expiry, renewal failure, or token loss fails closed before
  the node can present itself as current, accept mutations, or acknowledge work.
- Forensic acquisition is observation-only and may preserve quarantined or
  untrusted bytes. A forensic bundle is never a backup and cannot be restored
  or admitted as a recovery source. Salvage from forensic evidence is a
  separate evidence-admission and reconstruction workflow outside S.10.
- S.12 may broaden backend, hardware, workload, and soak qualification. It may
  not be used to defer S.10 correctness, crash safety, boundedness, or hostile
  proof.

## Authority And Ownership Contract

The ordinary proof progression is:

```text
raw media or operational intent
  -> scope-admitted observation
  -> integrity and recovery classification
  -> evidence-resolved stable source
  -> canonical operation-specific owner-plan DAG
  -> staging authorization and admission
  -> execution-ready owner-plan DAG
  -> Executed<Operation>
  -> PostVerified<Operation>
  -> fresh cutover candidate resolved against current authority
  -> cutover-specific owner-plan DAG and authorization
  -> write fence or quiescence proof
  -> PublishedPendingReadmission<Operation>
  -> ReadmittedCurrent<Operation>
```

No arrow may be skipped, reversed, reconstructed from a digest, or satisfied by
copying fields into a structurally similar type. Constructors for proof-bearing
artifacts are sealed to the owner that establishes the proof.

For destructive workflows, the private common static substrate uses Worth
Proof's existing stages around Store-owned payloads:

```text
Recipe<Unresolved, StoreOperationalIntent>
  -> Recipe<Resolved, EvidenceResolvedStorePlan, CurrentObservationBasis>
  -> Recipe<Lowered, CanonicalStoreOwnerPlanDag, CurrentPlanBasis>
  -> Recipe<Admitted, AuthorizedStoreOwnerPlanDag, CurrentAuthorizationBasis>
  -> ExecutionReadyRecipe<ReadyStoreOwnerPlanDag, CurrentExecutionBasis>
  -> ExecutedRecipe<ExecutedStoreOwnerReceiptDag, ExecutionBasis>
```

The generic forms above are private implementation substrate, not public API.
Public facades are operation-named and sealed through the rest of the lifecycle:
`AuthorizedBackupRestorePlan`, `ExecutedBackupRestore`,
`PostVerifiedBackupRestore`, `PublishedBackupRestorePendingReadmission`, and
`ReadmittedBackupRestoreCurrent`, with corresponding PITR, rollback,
authority-affecting repair, bootstrap, and promotion types. A generic
`AuthorizedOperationalPlan<OperationKind>`, `PostVerifiedOperationalResult`, or
published/readmitted result must not escape a private progression module.

The ownership split is fixed:

- `worth-store-physical-format` owns bounded format decoding and structural
  media walks.
- `worth-store-physical-integrity` owns checksum evaluation, damage
  localization, integrity classification, and physical quarantine mutation.
- `worth-store-recovery-physics` owns recovery-source precedence, exact
  checkpoint/WAL admission, replay, and recovered-frontier receipts; the
  offline verifier owns candidate-evaluation projection and canonical candidate
  sets.
- `worth-store-physical-isolation` owns stable-cut leases, staging isolation,
  copy-on-write publication, source-cut reachability leases, and reachability
  barriers.
- `worth-store-physical-backend` owns actual durable media reads, writes,
  synchronization, rename/publication mechanics, backend capability facts, and
  the mechanics behind a physically independent operational-control namespace.
- `worth-store-layout-indexes` and `worth-store-blob-chunks` own rebuild,
  quarantine, and repair behavior for their artifact families.
- `worth-store-replication` owns replica bootstrap state, divergence facts,
  acknowledged replica frontiers, and promotion inputs.
- `worth-store-authority` owns fencing, authority-epoch progression, current
  authority identity, renewable primary serve leases/tokens, fencing-authority
  integration, epoch progression, and readmission.
- `worth-store-offline-verifier` owns independent read-only acquisition and the
  composition of owner observations into operational truth reports.
- `worth-store-operations` owns workflow progression, operator-facing plans,
  authorization coordination, canonical owner-plan DAG scheduling, durable
  operational-control record semantics, receipt composition, progress,
  cancellation, and recovery handles. It owns no backend durability primitive
  and no owner mutation.
- certification owns closeout verdicts and hostile oracles; it never mints
  runtime recovery capability.

## Worth Proof And Foundational Adoption Contract

The crate boundary rule is:

```text
Store owner type while semantics are Store-owned
  -> Worth Proof carrier while static progression must be enforced
  -> Worth Foundational lowering only at a shared support/export boundary
```

The reverse direction does not exist. Worth Foundational artifacts can be
revalidated as inputs to a Store owner, but they cannot promote themselves.

| S.10 surface | Strong owner meaning | Shared adoption | Forbidden weakening |
| --- | --- | --- | --- |
| operational intent and plan | operation-specific Store intent, evidence basis, owner lowering, and footprint | Worth Proof `Recipe` stages, assumption basis, freshness, witnesses, and checked outcomes | a generic `Artifact<P, ValueBag>` public workflow or authorization before lowering |
| owner plan and receipt composition | concrete Store owner-plan DAG nodes/typed edges/barriers and matching owner receipts with non-empty, canonical, acyclic, unique membership | Worth Proof `NonEmpty`, `CanonicalVec`, `UniqueVec`, and structural facts for node/edge collections where the invariant is real; Store retains DAG semantics | raw `Vec`, unordered plan sets, dynamic `ProofSet`, trait-object receipt bags, copied proof fields, or generic `TransitionOutcome` accepted as an owner receipt |
| offline truth and recovery candidates | Store-owned region classifications, source evidence, exact frontiers, and candidate semantics | Foundational `Report`/`SupportOnly`, diagnostic row families, support truth, provenance, freshness, canonical comparison, and requested/admitted/materialized profiles | a Foundational report or certified diagnostic bundle accepted by the planner as Store observation authority |
| backup, DR, and forensic bundles | Store-owned physical format, completeness, reachability, custody, and restore/forensic semantics | Foundational canonical basis, export producer shape, digest derivation, provenance, and distinct restored/replay-derived/promoted/partial/divergent lineage | digest equality as bundle authority, or one Foundational export category erasing backup versus forensic meaning |
| owner execution and closeout | Store-owned executed receipts, journals, publication receipts, and authority readmission | Foundational planned/executed/completed receipt posture and boundary-evidence attachments after owner truth exists | a Foundational receipt satisfying an owner execution, publication, or readmission API |
| denials and explanations | Store-owned denial, unavailable, stale, rebind, indeterminate, and failure variants | Foundational decision/failure/comparison/support/provenance-ready rows with explicit gaps and availability | one optional-field diagnostic row, one string error, or `Result` flattening the checked outcome lattice |
| performance evidence | owner-local measurements bound to operation and phase | Foundational claim, policy-admission, canonical bundle, counter-backed receipt, planned report materialization, and certified/readmitted performance lanes | elapsed time as proof, a policy receipt as executed work, or support expansion on an ordinary hot path |
| audit and operator support | Store-owned canonical operational record derived from actual workflow artifacts | Foundational aspect contracts/validated struct values and diagnostic masks for shared record fields, plus profile, provenance, lineage, receipt, support-truth, diagnostic, and performance attachments | generic aspect maps inside workflows, arbitrary attachment maps, a second generic evidence envelope, JSON authority, or audit projection feeding runtime authority |

Semantic precision is mandatory:

- every public `Summary`, `Report`, `Artifact`, and `Receipt` maps to the matching
  Foundational category when it crosses a shared boundary
- every boundary output declares its role as authoritative-current,
  derived-projection, support-only, planned-work, or receipt-evidence; category
  and role are never one enum
- every proof-bearing Store form names its basis and freshness posture; current,
  stale-readable, rebind-required, authority-revalidation-required, and
  boundary-bridged forms are not aliases
- every execution result preserves success, denied, deferred, stale,
  rebind-required, and failed where those outcomes are reachable; indeterminate
  long-running outcomes additionally carry a Store recovery handle
- every continuity claim says attested, restored, replay-derived,
  reconstructed-equivalent, promoted, partial, or divergent; `recovered` alone
  is not a lineage class
- `canonical` always names the canonical basis and ordering authority;
  `verified` always names the verifier, source basis, scope, algorithm/profile,
  and freshness; `current` always names the admitting authority epoch
- Foundational lowering lives in named boundary-projection modules and is never
  interleaved with owner mutation, planning, or hot-path execution
- Foundational aspects describe shared structured boundary fields only after a
  Store domain record exists. Physical pages/manifests, operational plans,
  owner receipts, and authority records remain stronger Store types; JSON stays
  a terminal projection or hostile/readmission input

The operational nouns are also locked:

- an **observation** is a sealed owner's account of facts read from one named
  basis; it contains no cross-owner policy decision
- a **classification** exhaustively assigns meaning to admitted observations;
  it neither selects a recovery source nor proposes mutation
- a **candidate** is one selectable, explained possibility with denials and
  unknowns still visible; it is not a plan
- a **resolved plan** selects source, target, policy, and intended outcome
  against a current evidence basis; it is not executable
- a **lowered owner-plan DAG** names every concrete owner, effect, footprint,
  typed prerequisite edge, durability barrier, permitted concurrency,
  irreversible point, expected nested receipt, abandonment posture, and cost;
  it is the first form eligible for authorization
- an **admitted plan** is that exact lowered DAG with operation-specific
  authorization; it has not yet revalidated runtime resources or freshness
- an **execution-ready plan** has current basis, unconsumed authorization,
  owner availability, resource reservations, and durable journal admission
- an **executed owner receipt** attests one owner's durable effect; it does not
  attest whole-store coherence or current authority
- **post-verified** means a fresh independent verifier checked closed/frozen
  resulting media against exact plan and receipt basis; it still carries no
  cutover or publication authority
- **cutover-resolved** means operations freshly compared a post-verified
  candidate with current authority, explained the data-loss/lineage delta, and
  lowered a distinct cutover DAG
- **published pending readmission** means isolation durably exposed one coherent
  non-current root after cutover authorization and fencing. It must remain typed
  as pending, rejected by authority, abandoned, or retained for forensics until
  authority accepts or reclaims it
- **readmitted current** means the authority owner accepted that exact published
  root as current under a fresh/current epoch
- **quarantine classification** is owned by Integrity and names affected ranges;
  artifact owners decide artifact consequences, Isolation changes reachability,
  and Authority establishes degraded/quarantined posture. Each is a separate
  DAG node and emits a separate receipt

## DX Target

The intended operator surface makes authority and cost changes visible:

```rust
let inspection = OfflineStoreInspection::open(media, inspection_scope)?;
let observed = inspection.inspect(inspection_budget).await?;
let truth = observed.classify()?;

let candidate = RestoreIntent::from_verified_backup(verified_backup)
    .for_store(target_store)
    .resolve(&truth)?;

let lowered = candidate.lower(owner_capabilities)?;
let explained = lowered.explain();
let admitted = authorization_port.authorize(lowered, operator_assertion)?;
let ready = admitted.ready(execution_resources, current_basis)?;
let session = operations.execute(ready, execution_policy).await?;

for event in session.progress() {
    report(event?);
}

let executed = session.resume_after_crash()?.finish().await?;
let verified = offline_verifier.verify_staged(executed.staged_media()).await?;
let published = operations.publish(executed, verified).await?;
let current = authority.readmit(published)?;
```

`inspect`, `plan`, `authorize`, `execute`, `resume`, `verify`, `publish`, and
`readmit` are separate capabilities. Expensive work returns a session or stream
with progress, cancellation, persisted checkpoints, structured warnings,
indeterminate recovery, and finalization.

## Phase Plan

### Phase 1: Current Boundary Ledger And Readiness-Surface Hard Cutover

Freeze the real S.9 exit surface, identify every S.10-shaped placeholder, and
remove false proof before new recovery behavior is allowed to land.

**Relevant subsystems**

- `worth-store-operations`
- `worth-store-operations-vocabulary`
- `worth-store-offline-verifier`
- `worth-store-physical-format`
- `worth-store-physical-integrity`
- `worth-store-recovery-physics`
- `worth-store-authority`
- `worth-store-replication`
- `worth-store-certification`

**Relevant APIs and current surfaces**

- current backup/export custody emissions and readiness witnesses
- current repair blast-radius handoffs
- current offline physical and recovery verifiers
- `OperationalRecoveryPosture`
- Worth Proof and Worth Foundational imports, aliases, facade exports, and
  construction authorities on every S.10-shaped surface
- compile-fail cases that claim to deny unauthorized repair
- S.9 quarantine, recovery, import, replication, and current-authority receipts

**Warnings**

- S.5.1 readiness proves that security-scope metadata can be carried; it does
  not prove that backup, repair, restore, or authorization exists.
- Types named after `S10`, a milestone, a handoff destination, or an
  implementation phase are provenance placeholders. Any such production type
  touched by S.10 must be replaced by the domain capability it actually proves.
- A compile-fail test that fails at an unresolved import, private module, or
  unrelated type error is not evidence for the intended authority boundary.
- Existing in-memory verifier declarations and string-backed recovery artifacts
  are inventory facts, not accepted implementation foundations.

**Test requirements**

- Build a machine-readable boundary ledger proving that every S.10 input comes
  from an S.9 owner receipt or an explicitly untrusted external observation,
  and that every output has one declared owner and consumer.
- Mutate each compile-fail fixture so its unrelated import/type failures are
  repaired; the fixture must still fail only at the forbidden authorization or
  publication call and must compile when that single forbidden call is removed.
- Prove that readiness, identity, custody metadata, copied digests, model
  verdicts, and certification fixtures cannot satisfy destructive-operation
  authorization.
- Scan production exports and generated contexts for milestone-shaped names,
  generic recovery actions, parallel authority lanes, and behavioral exports
  from a vocabulary crate; every exception must be removed or narrowly justified
  as inert compatibility data.
- Classify every existing Worth Proof/Foundational use as owner progression,
  shared lowering, support materialization, certification strengthening, or
  misuse; controlled scans must fail on a raw generic proof carrier exported as
  a domain facade or on Foundational support evidence imported by execution.

**Engineering decisions**

- The phase produces an `OperationalRecoveryBoundaryLedger` and a
  `CurrentRecoverySurfaceGapReport`; neither artifact carries runtime authority.
- The boundary ledger includes a `SharedVocabularyAdoptionLedger` naming the
  stronger Store source type, Proof carrier or Foundational family, lowering
  direction, strength loss, construction authority, reverse-flow denial, and
  compile/runtime proof for every adoption point.
- All false-positive compile-fail evidence is a closeout blocker and is fixed
  before functional implementation begins.
- Readiness placeholders are hard-cut over when their owner boundary is
  implemented. No alias preserves the old name as a second public lane.
- The ledger records authority owner, observation owner, mutation owner,
  dependency direction, construction authority, cost class, failure topology,
  and proof lane for every public operational artifact.

**Open questions**

- None. The current surface must be measured before it is extended.

### Phase 2: Durable Operational Control Plane And Dependency-Direction Cutover

Establish the physically independent durable control plane that makes every
long-running protocol recoverable, and make the import graph prove that
operations consumes owner contracts while lower owners never depend on an
orchestration vocabulary to obtain authority.

**Relevant subsystems**

- `worth-store-operations`
- `worth-store-operations-vocabulary`
- `worth-store-physical-backend`
- `worth-store-authority`
- all lower owner crates named in the authority contract
- `tools/boundary-check`
- `tools/agent-context`

**Relevant APIs**

- owner-specific observation, plan, execution, and receipt facades
- operational intent and report value types
- `OperationalControlStore`
- `ControlStoreGeneration` and `ControlStoreTrustPosture`
- operation journal, checkpoint, authorization-consumption, source-lease,
  fence/epoch, owner-receipt, audit, and recovery-handle records
- crate manifests and Road 1 boundary configuration
- generated `AGENT_CONTEXT.md` files

**Warnings**

- Moving behavioral types into a vocabulary crate does not make the dependency
  neutral. Constructors that admit, authorize, execute, or issue proof are
  behavior and belong to the owner.
- A generic `RepairAction`, `RecoveryAction`, `RestoreSource`, or universal
  executor erases correctness, cost, and failure-mode boundaries.
- Operations may compose receipts but cannot reconstruct an owner receipt from
  public fields or a shared trait object.
- A journal on the same media, filesystem failure domain, or publication root as
  the source or target is not a recovery control plane. Source loss must not
  erase the record needed to decide whether recovery may continue.
- Checksums, generation chains, and atomic records detect accidental damage and
  torn writes; they do not establish malicious-tamper resistance. That stronger
  claim belongs to S.11.
- Multiple readable control-store copies are not automatically replicas. Unless
  one generation is selected by the fencing/epoch authority, duplicated,
  divergent, or stale control state is `Indeterminate` and destructive work
  fails closed.

**Test requirements**

- Compile-fail tests prove that operations cannot construct owner execution
  receipts, owners cannot construct orchestration authorization, and lower
  crates cannot import behavioral operations vocabulary.
- Boundary-check mutation tests add a forbidden lower-to-operations dependency
  and prove the constitution rejects it at the exact manifest edge.
- Import-graph tests prove production Foundational imports occur only in named
  boundary-projection/certification modules and those modules cannot be reached
  from owner resolution, lowering, execution, publication, or readmission.
- For every owner facade, prove that the only ordinary execution input is that
  owner's sealed lowered plan and the only successful output is its sealed
  executed receipt.
- Delete or replace one owner implementation in a fixture graph and prove that
  unrelated owner contracts and report vocabulary remain independently
  buildable.
- Crash before and after every control-store append, sync, generation advance,
  authorization consumption, owner-receipt join, audit derivation, and terminal
  disposition; reopen from a fresh process and prove one atomic prefix and no
  lost irreversible effect.
- Destroy source media, target media, or control media independently. Source or
  target loss follows the operation-specific recovery law; control-state loss,
  corruption, divergence, or unavailability denies destructive resume and
  cutover until an explicit reconstruction-and-fresh-authorization path succeeds.
- Duplicate a control store, advance both copies, restore an older copy, and
  replay valid records out of order. Only the generation selected by the real
  fencing/epoch authority may be current; otherwise return typed
  `ControlStateIndeterminate` without owner mutation.

**Engineering decisions**

- The default cutover deletes `worth-store-operations-vocabulary`. Retention is
  allowed only if the Phase 1 ledger finds a concrete neutral value consumed by
  at least two owners for the same semantic reason; even then, the crate may
  expose no admission, readiness, handoff, emission, execution, or receipt
  constructor.
- Each lower owner exposes declarative observation and execution contracts
  through its own facade. `worth-store-operations` depends downward on those
  facades and performs orchestration only.
- No cross-owner trait erases typed receipt identity. Receipt composition uses
  a Store-owned exhaustive owner-receipt sum joined against canonical DAG node
  identities. Worth Proof may enforce non-empty/canonical/unique node and edge
  collections, but Store owns acyclicity, prerequisites, barriers, and receipt
  correspondence; none is a raw `Vec`, dynamic proof set, or optional-field bag.
- Any crate/config change updates the canonical boundary configuration first;
  generated agent contexts are regenerated and never hand-edited.
- A crate that ships a Foundational boundary projection carries Foundational as
  an ordinary production dependency or moves that projection to an existing
  legal higher boundary crate. Cargo features may gate certification/test
  authority, but may not make shared semantic meaning exist only in tests.
  Diagnostic richness and support expansion are runtime materialization policy,
  not compile-time disappearance of the contract.
- A crate whose production facade promises Proof-backed stage, basis,
  freshness, or structural-collection law carries Worth Proof as an ordinary
  dependency and uses the real substrate. A dev-dependency, copied stage enum,
  or private lookalike wrapper cannot satisfy the contract.
- `worth-store-operations` owns the schema and lifecycle of operational control
  records; `worth-store-physical-backend` owns atomic append/CAS, synchronization,
  media identity, and reopen mechanics. The configured control location must be
  physically independent of every protected source and staging/target location.
- `OperationalControlStore` guarantees atomic record visibility, monotonic
  generation under the selected fencing authority, durable prefix recovery,
  idempotent append by operation/transition identity, and explicit detection of
  torn, corrupt, stale, missing, and divergent state. It stores authorization
  consumption, owner-plan DAG checkpoints and receipts, source leases, recovery
  handles, publication/readmission disposition, serve leases/fences/epochs, and
  the canonical audit source records.
- Record construction authority is explicit: Operations writes workflow/DAG
  progression, cancellation, recovery-handle, and expected-audit-transition
  records; the authorization adapter writes provider decisions while Operations
  atomically records consumption; Isolation writes source/staging leases and
  publication disposition; Authority writes serve-lease, fence, epoch, and
  readmission records; each artifact/recovery/replication owner writes its own
  sealed effect receipt before Operations joins that receipt to a DAG node.
  Backend persists bytes for all of them but constructs none of their meaning.
- The S.10 trust model is fail-closed accidental-damage integrity, not hostile
  administrator or malicious-media authenticity. S.11 may authenticate and
  chain the same records without changing their lifecycle semantics.
- Certification receives a conforming in-process or file-backed adapter through
  the ordinary public `OperationalControlStore` port. There is no test-only
  bypass constructor, alternate journal, or authority shortcut.

#### DX target: control-state recovery is explicit

```rust
let reopened = OperationalControlStore::open(control_location)?;
let posture = reopened.inspect_generations(fencing_authority).await?;

match posture {
    ControlStoreTrustPosture::Selected(current) => {
        let operations = current.list_recoverable_operations()?;
        let handle = operations.select(operation_id)?;
        handle.explain_durable_effects().explain_safe_next_actions();
        handle.resume_with_fresh_authorization(authorization).await?
    }
    ControlStoreTrustPosture::Damaged(report) => reconstruct_or_abandon(report)?,
    ControlStoreTrustPosture::Divergent(report) => return Err(report.into()),
    ControlStoreTrustPosture::Unavailable(reason) => return Err(reason.into()),
}
```

There is no `resume(target_path)` convenience that guesses from staged bytes.
Operators first see which generation the fencing authority selected, which
effects are durable, and whether resume, abandonment, reconstruction, or
forensic retention is legal.

#### Opinionated directory target

This tree is a rough target, not a frozen file manifest. It is deliberately
strong: departures require a named ownership, lifecycle, failure, cost, or
replacement reason. File-count preference alone is not a reason. Every
`lib.rs` and `mod.rs` shown here aggregates only.

```text
worth-store-operations/src/
  lib.rs
  facade.rs
  authorization/
    admitted_external_assertion.rs
    authorization_port.rs
    lowered_plan_binding.rs
    authorization_consumption.rs
    authorization_denial.rs
    revocation_boundary.rs
    cutover_authorization.rs
  control_store/
    control_record.rs
    control_store_port.rs
    control_generation.rs
    trust_posture.rs
    reopen_recovery.rs
    divergence.rs
  owner_plan_dag/
    owner_plan_node.rs
    prerequisite_edge.rs
    durability_barrier.rs
    irreversible_point.rs
    abandonment_posture.rs
    expected_receipt.rs
    canonical_dag.rs
  workflow/
    backup/
      intent.rs
      resolution.rs
      lowering.rs
      execution_session.rs
      completion.rs
    restore/
      intent.rs
      resolution.rs
      lowering.rs
      staging_session.rs
      cutover_resolution.rs
      completion.rs
    point_in_time_recovery/
      intent.rs
      candidate_resolution.rs
      lowering.rs
      staging_session.rs
      cutover_resolution.rs
      completion.rs
    rollback/
      intent.rs
      candidate_resolution.rs
      lowering.rs
      staging_session.rs
      cutover_resolution.rs
      completion.rs
    repair/
      intent.rs
      candidate_resolution.rs
      owner_lowering.rs
      topology.rs
      execution_journal.rs
      execution_session.rs
      completion.rs
    replica_promotion/
      intent.rs
      candidate_resolution.rs
      fence_lowering.rs
      promotion_session.rs
      completion.rs
    replica_bootstrap/
      intent.rs
      source_resolution.rs
      lowering.rs
      bootstrap_session.rs
      completion.rs
    forensic_acquisition/
      intent.rs
      resolution.rs
      lowering.rs
      acquisition_session.rs
      completion.rs
  receipt_composition/
    owner_receipt_kind.rs
    non_empty_owner_receipts.rs
    canonical_owner_receipts.rs
    completed_operation_receipt.rs
    dag_receipt_join.rs
  progress/
    operation_progress_event.rs
    operation_progress_cursor.rs
    indeterminate_recovery_handle.rs
  operational_audit/
    operational_audit_record.rs
    operation_sequence.rs
    causal_parent.rs
    durable_derivation.rs
    duplicate_resolution.rs
    decision_trace.rs
    record_completeness.rs
  boundary_projection/
    foundational_profiles.rs
    foundational_diagnostics.rs
    foundational_boundary_evidence.rs
    foundational_performance.rs
    operational_audit_projection.rs

worth-store-offline-verifier/src/
  lib.rs
  facade.rs
  media_acquisition/
    untrusted_media_set.rs
    read_only_media_capability.rs
    media_identity.rs
    acquisition_denial.rs
  inspection/
    inspection_scope.rs
    inspection_budget.rs
    inspection_cursor.rs
    inspection_session.rs
    resume_checkpoint.rs
  owner_observation/
    format_walk.rs
    integrity_classification.rs
    recovery_candidate_view.rs
    layout_observation.rs
    blob_observation.rs
    replication_observation.rs
  truth_composition/
    truth_region.rs
    non_empty_evidence_set.rs
    canonical_truth_report.rs
    candidate_projection.rs
  boundary_projection/
    foundational_support_report.rs
    foundational_diagnostics.rs
    foundational_lineage.rs
    foundational_performance.rs

worth-store-physical-format/src/offline_walk/
  walk_plan.rs
  walk_cursor.rs
  bounded_decode.rs
  structural_observation.rs

worth-store-physical-integrity/src/
  offline_classification/
    classification_request.rs
    damage_localization.rs
    integrity_observation.rs
  quarantine_execution/
    range_classification.rs
    artifact_consequence_request.rs
    integrity_receipt.rs
  post_recovery_verification/
    verification_request.rs
    verification_result.rs

worth-store-offline-verifier/src/
  truth_composition/candidate_evaluation/
    mod.rs
    candidate_set.rs

worth-store-recovery-physics/src/
  backup_restore/
    replay_plan.rs
    replay_execution.rs
    recovered_frontier_receipt.rs
  point_in_time_recovery/
    time_selection.rs
    exact_frontier_resolution.rs
    replay_plan.rs
    replay_execution.rs
  rollback_recovery/
    retained_authority_resolution.rs
    rollback_plan.rs
    rollback_execution.rs

worth-store-physical-isolation/src/
  backup_cut/
    cut_admission.rs
    reachability_lease.rs
    lease_recovery.rs
  recovery_source_lease/
    source_cut.rs
    reachability_lease.rs
    lease_recovery.rs
  recovery_staging/
    staging_identity.rs
    staging_publication.rs
    staged_root_receipt.rs
  recovery_publication/
    publication_plan.rs
    atomic_publication.rs
    publication_receipt.rs
    pending_readmission.rs
    rejected_by_authority.rs
    abandoned_publication.rs
    forensic_retention.rs
    reclaim_plan.rs

worth-store-physical-backend/src/operational_control/
  control_media_identity.rs
  atomic_record_append.rs
  generation_compare_exchange.rs
  durable_prefix_recovery.rs
  control_media_fault.rs

worth-store-authority/src/
  primary_serving/
    serve_lease.rs
    lease_renewal.rs
    lease_expiry.rs
    fail_closed_gate.rs
  fencing_authority/
    fencing_port.rs
    selected_control_generation.rs
    fencing_capability.rs
    fencing_unavailable.rs
  recovery_fencing/
    fence_plan.rs
    fence_execution.rs
    fence_receipt.rs
  epoch_progression/
    next_epoch_plan.rs
    progressed_epoch.rs
  recovery_readmission/
    readmission_request.rs
    readmission_verification.rs
    current_authority_receipt.rs
    publication_rejection.rs

worth-store-replication/src/
  bootstrap/
    bootstrap_plan.rs
    bootstrap_session.rs
    bootstrap_receipt.rs
  divergence/
    history_classification.rs
    acknowledged_frontier.rs
  promotion/
    promotion_plan.rs
    promotion_execution.rs
    promotion_receipt.rs
  rejoin/
    rejoin_plan.rs
    rejoin_denial.rs

worth-store-physical-certification/src/
  drivers/operational_recovery/
  faults/operational_recovery/
  schedules/operational_recovery/
  scenarios/operational_recovery/
  oracles/operational_recovery/
  transcript/operational_recovery/

worth-store-certification/src/
  courtroom/operational_recovery/
    burning_primary_poisoned_backup.rs
    split_brain_datacenter_reversal.rs
    byzantine_maintenance_repair_marathon.rs
  evidence/operational_recovery/
    phase_coverage.rs
    proof_progression.rs
    foundational_lowering.rs
    controlled_defect_localization.rs
```

The tree carries these placement laws:

- `workflow/<operation>` owns cross-owner sequence, not owner mutation law.
- authorization binds only `workflow/<operation>/lowering` output.
- `owner_plan_dag` owns canonical cross-owner topology. Workflows instantiate
  operation-specific DAGs; owner crates neither call one another nor absorb
  orchestration authority.
- `control_store` owns durable record meaning and lifecycle; backend
  `operational_control` owns only storage mechanics. Neither may construct the
  other's authority artifacts.
- Worth Proof carriers stay inside workflow/owner progression modules; public
  facades expose domain-named wrappers rather than raw generic recipes.
- Worth Foundational imports are confined to `boundary_projection` and
  certification evidence assembly unless a lower owner genuinely publishes a
  shared boundary artifact.
- `operational_audit` owns the narrow Store record; `boundary_projection` owns
  Foundational attachments and terminal/export widening. Neither owns runtime
  authorization or owner receipts.
- support projection cannot be imported by resolution, lowering,
  authorization, execution, publication, or readmission modules.
- The offline verifier is the constructor owner of observer recovery-candidate
  projections and exact observed frontiers. Recovery Physics owns source
  precedence and physical admission; it does not construct the observer's
  candidate type or select a source for an operation.
- test scenarios, drivers, faults, schedules, and oracles are different
  responsibilities and never share one `support` or `helpers` directory.
- any S.10 work touching the current flat verifier facade or behavioral
  operations-vocabulary surface performs the corresponding cutover instead of
  adding another parallel lane.

**Open questions**

- None. Dependency direction follows authority ownership.

### Phase 3: Independent Offline Media Acquisition And Bounded Structural Walk

Establish a real verifier entry boundary that opens store media directly and
walks it without constructing the live runtime or materializing the store in
memory.

**Relevant subsystems**

- `worth-store-offline-verifier`
- `worth-store-physical-backend`
- `worth-store-physical-format`
- `worth-store-physical-integrity`
- `worth-store-buffer-pool`
- `worth-store-blob-chunks`

**Relevant APIs**

- `OfflineStoreInspection::open`
- `UntrustedOfflineMediaSet`
- `OfflineInspectionScope`
- `OfflineInspectionBudget`
- bounded page, extent, manifest, WAL, index, and chunk readers
- `StructurallyWalkedMedia`
- `OfflineInspectionSession`

**Warnings**

- Accepting a caller-built `PersistedPhysicalLayout`, a vector of pages, or a
  declaration of expected digests is not offline media inspection.
- Reusing the live runtime's caches, recovery decisions, open handles,
  in-memory manifests, or private decoder graph makes verifier independence a
  lie.
- Mmap of an entire store is not a bounded-memory proof. Address-space mapping,
  resident pages, pinned buffers, and transient decode allocations remain
  separately accountable.
- Offline verification is observation-only. It cannot quarantine, repair,
  publish, fence, or readmit authority.
- A path opened read-only is not necessarily a consistent inspection basis.
  Concurrent mutation can create a physically impossible cross-file view even
  when every individual read succeeds.

**Test requirements**

- Create a store larger than the verifier memory budget, close the producer,
  start a fresh process/session, and prove the verifier opens actual files and
  completes with exact verifier-buffer, pinned-frame, decoder-allocation,
  backend-requested-byte, and page/chunk-touch counters within contract. OS RSS
  and residency are bounded profile observations, never exact protocol facts.
- Substitute a caller-built in-memory layout with identical declarations but
  different on-disk bytes; the verifier must report the disk facts and reject
  the declaration as authority.
- Run against missing files, extra residue, truncated manifests, unknown format
  versions, bad sectors, permission failures, and concurrent media mutation;
  each failure localizes to a typed acquisition or structural-walk result.
- Instrument the live runtime entrypoints and prove an offline inspection never
  constructs or calls them, including on error and resume paths.
- Cancel and crash the inspection at each file/page/chunk boundary, reopen from
  its persisted observation checkpoint, and prove convergence with an
  uninterrupted walk without treating the checkpoint as trusted media truth.
- Mutate each media family during a scan. Inspection must use a backend snapshot,
  immutable clone, stable file-generation handles, or a content-addressed
  closure; if none is supported it returns `ConcurrentMutationIndeterminate`
  and never composes a cross-generation truth report.
- Corrupt, truncate, replay, or substitute a resume checkpoint. Checkpoints are
  optimizations only: invalid state causes safe restart or bounded rewalk of the
  affected ranges and can never skip unobserved media.

**Engineering decisions**

- `worth-store-offline-verifier` owns independent acquisition and composition;
  it delegates format decode and integrity evaluation to their owners.
- The physical backend exposes a read-only offline capability with no write,
  sync, rename, reclaim, or publication methods in its type surface.
- The walk is streaming and cursor-based. Persisted resume state records
  progress identity and observations, then revalidates the media identity and
  boundary generation before resuming.
- Every completed inspection binds one explicit consistency basis: backend
  snapshot id, immutable-clone id, stable generation-handle set, or
  content-addressed closure. A backend without one of these mechanisms may be
  inspected only after externally proven quiescence; otherwise the result is
  `ConcurrentMutationIndeterminate`.
- Unknown or unsupported format/backend/security capabilities produce typed
  unsupported results, never best-effort decoding.
- The verifier binary/library dependency graph excludes operations and the live
  Store runtime.
- Store-owned inspection phases use Worth Proof assumption bases and freshness
  states to carry media identity, cursor basis, and resume revalidation. Raw
  `Recipe`/`Artifact` carriers remain private behind verifier-domain types.
- Foundational diagnostics, support truth, profiles, and performance reports
  are materialized only from a completed Store truth report through the named
  boundary-projection modules; they cannot feed owner observation or candidate
  construction.

#### DX target: incident-safe inspection

```rust
let mut inspection = OfflineStoreInspection::open(untrusted_media)?
    .scope(InspectionScope::all_physical_families())
    .budget(InspectionBudget::bounded(memory, io, deadline))
    .start()?;

for event in inspection.progress() {
    render_bounded_progress(event?);
}

let store_truth = inspection.resume_after_crash()?.finish().await?;
let summary = store_truth.summary(); // narrow Store-owned observation
let support = store_truth
    .plan_support_materialization(requested_support_profile)?
    .materialize()?; // explicit wider Foundational boundary projection
```

Opening media, streaming inspection, resuming, reading the narrow Store truth,
and materializing rich support output are visibly different cost and trust
boundaries. There is no `verify(path)` convenience that silently performs all
of them.

**Open questions**

- None. Independent real-media acquisition is required for the milestone claim.

### Phase 4: Evidence Composition, Truth Classification, And Candidate Discovery

Compose structural, integrity, recovery, layout, blob, scope, custody, and
replication observations into an evidence-bound operational truth report without
promoting that report into mutation authority.

**Relevant subsystems**

- `worth-store-offline-verifier`
- `worth-store-physical-integrity`
- `worth-store-recovery-physics`
- `worth-store-layout-indexes`
- `worth-store-blob-chunks`
- `worth-store-replication`
- `worth-store-authority`

**Relevant APIs**

- `IntegrityClassifiedMedia`
- `RecoveryCandidateSet`
- `OperationalTruthReport`
- `TrustedAuthorityRegion`
- `DegradedDerivedRegion`
- `RebuildableRegion`
- `QuarantinedRegion`
- `UnrecoverableAuthorityRegion`
- `IndeterminateTruthRegion`

**Warnings**

- `OperationalRecoveryPosture` as a bare enum is insufficient. A posture without
  source evidence, exact affected ranges, scope, frontier, and confidence is a
  label rather than an operational fact.
- Checksum success does not prove authenticity; authenticity success does not
  prove current authority; current-authority identity does not prove every
  reachable byte is intact.
- Missing custody or unavailable authenticity produces `Indeterminate` or a
  typed denial, not a silent downgrade to integrity-only trust.
- Candidate discovery proposes possible recovery frontiers. It cannot select or
  authorize one for execution.
- Exhaustive classification is physical, not merely logical. Aliased extents,
  overlapping manifests, duplicate chunk references, and multiply-owned ranges
  must be represented explicitly rather than counted as independently covered.

**Test requirements**

- Independent full and resumed inspections over the same immutable media must
  converge on the same canonical region classifications, candidate frontiers,
  source identities, and counters regardless of scan order or batch width.
- Inject damage into authoritative pages, derived indexes, blob chunks,
  checkpoints, WAL tails, and manifests; prove each region is classified by its
  true authority status and no intact derived artifact outranks damaged
  authority.
- Remove key custody, tenant evidence, or authenticity capability while leaving
  checksums valid; the report must distinguish unavailable, unsupported,
  wrong-scope, and failed authenticity.
- Present divergent checkpoint/WAL/replica histories sharing content digests;
  the candidate set must preserve lineage and frontier divergence rather than
  collapsing by representation.
- Prove every report row carries exact media source, physical range, artifact
  family, generation, authority class, integrity/authenticity/custody posture,
  recovery relevance, and evidence references.
- Prove the canonical classification covers every admitted physical byte/range
  exactly once, except explicit `AliasGroup` and `OverlapConflict` variants that
  name all claimants. Gaps, duplicate coverage, and silent overlap fail report
  construction.

**Engineering decisions**

- Truth classification is an exhaustive evidence-bound sum type. New truth
  states require updating every projection, plan admission, and certification
  oracle at compile time.
- The report carries a canonical physical-coverage proof. Exhaustive logical
  rows without non-overlap/alias accounting are not a valid truth report.
- The Store report is the stronger source. Its shared support projection uses
  Foundational `Report` with `SupportOnly`, family-distinct diagnostic rows,
  explicit partiality/gaps, provenance/freshness, and degraded-recovery support
  truth. No Foundational projection implements the Store planner input trait.
- Canonical ordering is defined for reports and candidate sets so independent
  observers can compare results without scan-order drift. Worth Foundational
  canonical basis and comparison are used for cross-producer comparison only
  after the Store-owned ordering proof exists; digest comparison is never the
  equivalence authority.
- Reports expose both trusted facts and explicit unknowns. Absence of evidence
  never serializes as a healthy or empty region.
- Candidate discovery is owned by the offline verifier and consumes sealed
  owner observations; recovery physics owns source precedence and admission,
  but cannot construct the observer's candidate projection.

**Open questions**

- None. Evidence-bound truth reporting is the input to every later plan.

### Phase 5: Online Backup Cut Admission And Reachability Protection

Admit an online backup as a stable physical cut bound to exact current authority
and protect every required byte from movement or reclaim until independent
verification succeeds or the cut is explicitly abandoned.

**Relevant subsystems**

- `worth-store-operations`
- `worth-store-authority`
- `worth-store-recovery-physics`
- `worth-store-physical-isolation`
- `worth-store-physical-integrity`
- `worth-store-layout-indexes`
- `worth-store-blob-chunks`
- `worth-store-physical-backend`

**Relevant APIs**

- `OnlineBackupIntent`
- `BackupCutAdmission`
- `AdmittedBackupCut`
- `BackupReachabilityLease`
- `BackupCutManifest`
- `BackupCutAbandonmentReceipt`

**Warnings**

- A filesystem timestamp, root path, checkpoint label, or current manifest alone
  does not identify a restorable cut.
- A checkpoint without its required WAL tail, or a WAL tail without exact
  checkpoint and durable-ack binding, is not a backup source.
- A page lease that omits blob chunks, old generations, or secondary physical
  roots allows compaction/reclaim to invalidate the cut during export.
- Backup admission must not stop foreground mutation for the duration of media
  copy. It freezes a logical physical cut through reachability proofs, not a
  global long-held latch.

**Test requirements**

- Under continuous foreground writes, compaction, checkpoint publication, blob
  migration, and reclaim, every admitted cut must materialize the exact root,
  checkpoint, WAL tail, page/extent/index/chunk reachability set, and authority
  frontier captured at admission.
- Attempt reclaim and generation reuse for every protected artifact family;
  prove the lease blocks it and exact blocked-reclaim counters identify the
  responsible backup cut.
- Crash before and after cut admission, lease persistence, checkpoint binding,
  and abandonment; reopen and prove the cut is either resumable with the same
  identity or durably abandoned without leaked protection.
- Present stale current-authority identity, wrong tenant/key scope, missing
  custody, authenticity-unavailable media, or unsupported backend durability;
  admission must fail before any broad reachability walk or output allocation.
- Create two cuts around a checkpoint/root publication race and prove each cut
  is internally coherent and neither mixes generations.

**Engineering decisions**

- `AdmittedBackupCut` binds store lineage, current-authority epoch, root and
  manifest generations, checkpoint identity, durable checkpoint LSN, required
  WAL interval, acknowledged frontier, artifact reachability basis, format and
  backend profile, tenant/key/authenticity/custody posture, and lease identity.
- The cut manifest is canonical and self-describing. A consumer need not query
  the producing runtime to interpret its completeness requirements. Store owns
  the physical completeness basis; Worth Foundational canonicalization supplies
  the portable basis sequence, producer-shape comparison, and downstream digest
  slots after that Store basis is ready.
- Isolation owners issue and persist the reachability lease; operations only
  coordinates admission and progress.
- Cut admission returns a long-running backup session handle. Cancellation
  lowers to explicit abandonment and durable lease release; ordinary completion
  releases only after independent verification has durably recorded its result.

**Open questions**

- None. Stable-cut completeness is a correctness boundary, not a policy knob.

### Phase 6: Backup Materialization And Independent Verification

Separate successful byte emission from independent bundle verification and
prepare the verified source that the ordinary restore phases will later use for
restore-drill certification.

**Relevant subsystems**

- `worth-store-operations`
- `worth-store-physical-backend`
- `worth-store-physical-format`
- `worth-store-physical-integrity`
- `worth-store-offline-verifier`
- `worth-store-recovery-physics`
- `worth-store-blob-chunks`
- `worth-store-certification`

**Relevant APIs**

- `BackupMaterializationSession`
- `MaterializedBackupBundle`
- `StructurallyVerifiedBackupBundle`
- `CustodyQualifiedBackupBundle`
- `ProductionRestoreAdmissibleBackupBundle`
- `BackupBundleCustodyManifest`
- `BackupVerificationReport`

**Warnings**

- Bytes emitted, files closed, digests matched, or a manifest parsed are not
  restoreability evidence.
- Verification by the same code path and in-memory expectations used to produce
  the bundle is same-run self-comparison.
- A forensic bundle may intentionally contain damaged or untrusted bytes and
  must never satisfy any backup verification or restore-admissibility state.
- `verified` without a named strength invites structural consistency to be
  mistaken for authenticity, custody, or production restore admissibility.
- Future restore-drill certification does not retroactively verify a particular
  backup. Per-backup verification and sampled operational certification remain
  distinct proof states.

**Test requirements**

- Stream multi-GB pages, WAL, indexes, and blob chunks into a bundle under a
  memory budget smaller than the bundle; assert exact source reads, output
  writes, buffered bytes, retries, and resume boundaries.
- Crash or cancel at every manifest, page range, WAL segment, chunk, sync, and
  final publication boundary; resume must converge byte-for-byte and
  receipt-for-receipt with uninterrupted materialization or durably abandon the
  incomplete bundle.
- Independently open the completed bundle in a fresh process, without producer
  state, and prove structural completeness, integrity, authenticity/custody
  posture, exact frontier, and reachability closure.
- Remove, duplicate, reorder, truncate, or cross-scope substitute every bundle
  component while preserving outer metadata; verification must localize the
  defect and refuse `StructurallyVerifiedBackupBundle` construction.
- Inject a controlled producer defect that preserves per-file digests but breaks
  cross-component reachability or frontier closure; independent verification
  must reject it before ordinary restore phases may consume the bundle.

**Engineering decisions**

- The backup progression is `AdmittedBackupCut -> MaterializedBackupBundle ->
  StructurallyVerifiedBackupBundle -> CustodyQualifiedBackupBundle ->
  ProductionRestoreAdmissibleBackupBundle`. Structural verification proves
  physical and cross-component consistency only. Custody qualification records
  the available S.10 custody/authenticity posture without claiming S.11
  cryptographic proof. Restore admissibility is a current policy/authority
  decision over those stronger inputs. The later ordinary restore and reopen
  path is the only constructor authority for `RestoreDrillCertification`.
- Bundle publication uses durable staging and atomic final-manifest publication.
  Partially materialized output is discoverable only as an incomplete session,
  never as a backup.
- Verification consumes actual bundle media and canonical manifests, not
  producer declarations. Restore-drill certification consumes the ordinary
  restore and reopen paths.
- The Store bundle format and verified/restorable states remain Store-owned.
  Foundational export vocabulary describes producer shape and boundary
  publication; provenance, restored-continuity attachments, profiles, and
  digests are derived shared surfaces and never the verification oracle.
- Requested backup support/retention/certification richness progresses through
  Foundational requested, admitted, and materialized profiles. Narrowing is
  explicit; omitted custody, reachability, or restoreability evidence cannot be
  hidden as a lower diagnostic-richness choice.
- Backup retention/release of the stable-cut lease occurs only after durable
  independent structural verification (including cross-component reachability)
  or explicit abandonment, not after materialization or the last data write.
- The lease is released only after durable independent structural verification
  or explicit abandonment; materialization success alone is insufficient.

#### DX target: backup truth ladder

```rust
let cut = OnlineBackupIntent::for_current_store(store)
    .resolve(current_authority, checkpoint_frontier)?
    .lower(backup_owners)?
    .admit_cut()?;

let materialization = cut.materialize(target, backup_budget).await?;
let materialized = materialization.resume_after_crash()?.finish().await?;
let structural = offline_verifier.verify_backup(materialized).await?;
let custody = structural.qualify_custody(custody_evidence)?;
let admissible = authority.admit_backup_for_restore(custody, current_policy)?;

// Available only after the ordinary restore, readmission, reopen, and final
// offline comparison path completes in Phase 13.
let drill = certification.restore_drill(admissible, isolated_target).await?;
```

The API does not return one `BackupResult`. Autocomplete and types make it
obvious whether the caller holds a cut, materialized bytes, independently
structurally verified bytes, custody-qualified evidence, a currently
restore-admissible backup, or restore-drill certification.

**Open questions**

- None. The explicit strength ladder deliberately prevents operational
  overclaiming.

### Phase 7: Operation-Specific Authorization And Destructive-Plan Admission

Establish the Store-local authorization protocol that later destructive plans
must satisfy without pretending S.10 is an identity provider or cryptographic
access-control system.

**Relevant subsystems**

- `worth-store-operations`
- `worth-store-authority`
- S.5.1 tenant, key, authenticity, custody, and blast-radius vocabulary
- external operator authorization adapter boundary
- `worth-store-certification`

**Relevant APIs**

- `ExternalOperatorAssertion`
- `OperationalAuthorizationPort`
- operation-specific authorizable facades such as `LoweredBackupRestorePlan`
- operation-specific authorized facades such as `AuthorizedBackupRestorePlan`
- operation-specific cutover facades such as `AuthorizedBackupRestoreCutover`
- `AuthorizationDenial`
- `AuthorizationConsumptionReceipt`
- `AuthorizationRevocationObservation`

**Warnings**

- An operator id, role, readiness witness, custody handoff, signed display token,
  or generic `AuthorityMarker` is not authorization to mutate Store authority.
- The Store must not parse provider-specific roles or tokens in domain code.
  External identity and proof-of-possession belong to the adapter and S.11.
- A reusable authorization that is not bound to an exact plan permits
  time-of-check/time-of-use plan substitution.
- Authorization success cannot waive integrity, scope, generation, source, or
  post-verification preconditions established by owner law.

**Test requirements**

- For each destructive operation kind, substitute a plan with a different
  digest, store lineage, tenant/key/custody scope, footprint, source, target
  generation, or recovery frontier; authorization consumption must fail before
  owner execution.
- Replay a consumed single-use authorization and concurrently race two
  consumers; at most one operation may cross admission, with a durable typed
  replay result after restart.
- Expire or revoke staging authorization before execution, during reversible
  staging, and after the declared irreversible staging point. Before that point
  work stops/abandons; after it, durable cleanup/resume law governs and
  revocation cannot pretend the effect never happened. In every case publication
  remains impossible without a fresh cutover authorization.
- Run with no provider, an unavailable provider, an unsupported assertion type,
  a wrong proof-of-possession binding, and a provider timeout; destructive work
  must not begin and the outcomes remain machine-distinguishable.
- Compile-fail tests prove an authorization for restore cannot authorize repair,
  PITR, rollback, or replica promotion even when the plans share the same
  footprint and representation.

**Engineering decisions**

- S.10 defines a provider-neutral authorization port and admitted external
  assertion envelope. S.11 supplies production provider and cryptographic
  implementations.
- Private generic authorization substrate is phantom-tagged by a sealed
  operation kind; public APIs expose only operation-named facades. It consumes
  only a fully lowered owner-plan DAG and binds the canonical plan identity,
  exact owner-plan membership, store/authority
  identity, security scope, footprint, source, target frontier/generation,
  issuance/expiry, revocation semantics, and replay policy.
- Authorization is the `Lowered -> Admitted` transition. Runtime freshness,
  resource reservations, unconsumed authorization, and owner availability then
  produce `ExecutionReadyRecipe`; no API executes an admitted-but-not-ready
  plan.
- Authorization is consumed durably at the execution boundary. Crash recovery
  distinguishes not-started, started/resumable, completed, abandoned, and
  indeterminate consumption.
- Revocation is legal before start and throughout explicitly reversible staging.
  After the plan's first durable irreversible effect, revocation changes the
  permitted recovery disposition but cannot erase or roll back that effect.
  Cutover has a separate freshly issued/consumed authorization after
  post-verification and current-authority comparison.
- In short: staging authorization is revocable before start and during reversible
  staging, but certification still enters through the ordinary public authorization port.
- Certification injects a conforming deterministic adapter through the ordinary
  public authorization port. It has no test-only constructor, bypass facade, or
  feature that mints admitted plans directly.

**Open questions**

- None. S.11 strengthens the provider; it does not change S.10 plan binding.

### Phase 8: Non-Current Restore Staging And Recovery

Restore a production-restore-admissible backup into isolated non-current media,
recover and verify it
there, and produce a staged result that the common post-verification and
publication phase can later consume without overwriting the current store.

**Relevant subsystems**

- `worth-store-operations`
- `worth-store-physical-backend`
- `worth-store-physical-isolation`
- `worth-store-recovery-physics`
- `worth-store-physical-integrity`
- `worth-store-offline-verifier`
- `worth-store-authority`
- layout and blob owners

**Relevant APIs**

- `BackupRestoreIntent`
- `EvidenceBoundBackupRestorePlan`
- `AuthorizedBackupRestorePlan`
- `RestoreStagingSession`
- `StagedRestoredStore`
- `RestoreExecutionReceipt`

**Warnings**

- Restore never targets the current root or mutates current media in place.
- A verified source bundle does not prove target capacity, backend capability,
  target-scope compatibility, or absence of collision with current authority.
- Treating staged recovery as published success would make the live runtime the
  first consumer of potentially bad restored media.
- Rollback is not an alias for backup restore. Import does not acquire an S.10
  workflow by implication; its foreign-media admission boundary stays outside
  this milestone.

**Test requirements**

- Restore a production-restore-admissible backup while the current store remains readable and
  writable; prove staged writes cannot alter current roots, generations,
  reachability, or authority identity before publication.
- Crash at allocation, component copy, WAL application, sync, and staged-root
  finalization; reopen from disk and prove current authority remains untouched
  while the staged operation is exactly resumable, complete, or abandoned.
- Supply insufficient capacity, unsupported format/backend profile, wrong
  tenant/key scope, missing custody, stale target epoch, and a target path
  aliasing current media; admission must deny before destructive allocation.
- Corrupt or remove staged bytes after copy but before verification; atomic
  publication must remain uncallable and current authority must stay unchanged.
- Retry/resume every interrupted phase using the same operation identity and
  prove idempotent convergence; a different plan identity cannot adopt the
  staged residue.

**Engineering decisions**

- The restore plan binds one `ProductionRestoreAdmissibleBackupBundle`, one non-current target
  identity, capacity/backend posture, exact security scope, footprint, expected
  source frontier, expected target generation, retention policy, and operation
  idempotency identity.
- Restore resolution and owner lowering use the shared Worth Proof recipe
  stages with Store-owned payloads. Staging authorization binds the lowered
  backup, backend, recovery, integrity, isolation, and blob/layout owner-plan
  DAG before staging begins; it does not authorize publication or readmission.
- Staging uses a foreign/non-current authority type with no ordinary read or
  current-publication facade.
- Backend owners perform durable writes; recovery physics applies the required
  WAL tail; artifact owners validate their families. Phase 13 later performs
  independent post-verification, fresh comparison with then-current authority,
  cutover-specific lowering/authorization, write fencing, atomic publication,
  and authority readmission. Current authority is allowed to advance during
  staging; the resulting data-loss/lineage delta must be explained at cutover.
- Old current media is retained or retired only through an explicit post-cutover
  plan. Successful promotion does not silently delete rollback evidence.
- A restore executed as a certification drill receives
  `RestoreDrillCertification` only after Phase 13 fresh-process verification and
  reopen complete through the ordinary path.

**Open questions**

- None. Non-current staging is the mandatory restore topology.

### Phase 9: Exact-Frontier Point-In-Time Recovery

Turn a human time request into a precisely explained recovery candidate, then
recover to an exact physical and acknowledged frontier through the same
non-current staging discipline.

**Relevant subsystems**

- `worth-store-operations`
- `worth-store-recovery-physics`
- `worth-store-physical-isolation`
- `worth-store-offline-verifier`
- `worth-store-authority`
- `worth-store-physical-backend`
- layout and blob owners

**Relevant APIs**

- `PointInTimeRecoveryIntent`
- `PointInTimeCandidateSet`
- `ExactRecoveryFrontier`
- `ResolvedPitrCandidate`
- `AdmittedPitrSourceCut`
- `PitrReachabilityLease`
- `EvidenceBoundPointInTimeRecoveryPlan`
- `AuthorizedPointInTimeRecoveryPlan`
- `PointInTimeRecoveryReceipt`

**Warnings**

- Wall-clock order can be ambiguous, skewed, duplicated, or absent. It is a
  candidate-selection hint, never a replay or publication authority.
- The requested instant may fall between acknowledged commits, after the last
  intact WAL record, inside a damaged interval, or across divergent lineages.
- Reporting the last replayed LSN without checkpoint, acknowledgment,
  authority-epoch, and skipped/damaged interval context is incomplete.
- PITR does not overwrite current media and does not silently round across an
  integrity, authenticity, custody, or lineage gap.

**Test requirements**

- Generate skewed, repeated, non-monotonic, and missing timestamps around exact
  checkpoint/WAL frontiers; candidate selection must be deterministic and the
  executed result must bind the selected exact frontier, not the clock value.
- Recover the same exact frontier with different replay batch sizes and crash
  schedules; fresh-process offline reports and executed receipts must converge.
- Request targets before retention, after the intact tail, inside a corrupted
  WAL interval, across a missing checkpoint, and on divergent replica history;
  return explicit unavailable, indeterminate, degraded, or denied candidates.
- Race retention/reclaim with admitted PITR and prove required checkpoint/WAL,
  pages, indexes, and chunks remain protected until completion or abandonment.
- Change or reclaim the chosen source after resolution but before source-cut
  admission. Lowering must be impossible until Isolation has issued a durable
  `PitrReachabilityLease`; crash recovery must retain or explicitly abandon it.
- Substitute a different candidate after authorization; plan fingerprint and
  exact-frontier binding must reject execution.

**Engineering decisions**

- Candidate explanation includes requested time, clock provenance/uncertainty,
  surrounding exact frontiers, selected checkpoint, required WAL interval,
  acknowledged frontier, gaps, source lineage, and expected recovery result.
- `ExactRecoveryFrontier` is constructed only by Recovery Physics from admitted
  evidence and cannot be built from a timestamp or raw LSN. It visibly carries
  checkpoint-durability frontier, WAL-structural frontier, local durable-commit
  frontier, client-ack frontier, replication-ack frontier, authority epoch, and
  source lineage. Comparison is a named partial order: incomparable lineage,
  epoch, or acknowledgment dimensions remain incomparable rather than being
  flattened to one scalar.
- The source progression is `ResolvedPitrCandidate -> AdmittedPitrSourceCut ->
  PitrReachabilityLease -> LoweredPointInTimeRecoveryPlan`. Lowering and staging
  authorization cannot occur before the source closure is durably pinned.
- PITR's boundary lineage is Foundational replay-derived continuity, not
  attested or ordinary restored continuity. The Store exact-frontier receipt is
  created first and remains the only input to later publication/readmission.
- PITR execution stages into non-current media and reuses owner-specific copy,
  replay, verification, publication, and readmission mechanisms without
  erasing PITR's distinct source and error topology.
- Any rounding policy is explicit in the intent and appears in the plan and
  audit record; it cannot cross a damaged or untrusted interval.

#### DX target: exact-frontier recovery

```rust
let candidates = PointInTimeRecoveryIntent::near(requested_time)
    .resolve(&offline_truth)?;

let selected = candidates
    .explain()?            // clock uncertainty, gaps, neighboring frontiers
    .select(exact_frontier)?;

let leased = selected.admit_source_cut(isolation)?.lease()?;
let lowered = leased.lower(recovery_owners)?;
let admitted = authorization.authorize_staging(lowered, operator_assertion)?;
let ready = admitted.ready(current_basis, resource_reservations)?;
let staged = operations.execute(ready).await?.resume_after_crash()?.finish().await?;

let verified = offline_verifier.verify_staged(staged.closed_media()).await?;
let cutover = verified.resolve_cutover(authority.current()?)?;
cutover.explain_data_loss_and_lineage_delta();
let lowered_cutover = cutover.lower_cutover()?;
let authorized_cutover = authorization.authorize_cutover(lowered_cutover, operator_assertion)?;
let fenced = authorized_cutover.establish_write_fence()?;
let published = operations.publish(fenced).await?;
let current = authority.readmit(published)?;
```

The wall-clock request disappears after candidate selection. Every later call
requires the exact frontier and makes staging, verification, publication, and
readmission separate actions.

**Open questions**

- None. Exact frontier identity is mandatory even when the UX begins with time.

### Phase 10: Retained-Authority Rollback Recovery

Recover from a retained prior authority generation through a source-stable,
non-current, independently verified rollback protocol. Rollback is a complete
operation parallel to backup restore and PITR, not a repair subtype or inverse
mutation.

**Relevant subsystems**

- `worth-store-operations`
- `worth-store-recovery-physics`
- `worth-store-physical-isolation`
- `worth-store-physical-backend`
- `worth-store-physical-integrity`
- `worth-store-offline-verifier`
- `worth-store-authority`
- layout and blob owners

**Relevant APIs**

- `RollbackIntent`
- `ResolvedRollbackCandidate`
- `AdmittedRollbackSourceCut`
- `RollbackReachabilityLease`
- `LoweredRollbackPlanDag`
- `AuthorizedRollbackPlan`
- `RollbackStagingSession`
- `ExecutedRollback`
- `PostVerifiedRollback`
- `AuthorizedRollbackCutover`
- `PublishedRollbackPendingReadmission`
- `ReadmittedRollbackCurrent`
- `RollbackAbandonmentReceipt`
- `IndeterminateRollbackRecoveryHandle`

**Warnings**

- A prior generation's continued existence does not prove it is complete,
  reachable, scope-compatible, or admissible under current authority policy.
- Rollback does not mutate the current root backward or apply an implicit inverse
  journal. It stages a new root whose lineage names the retained source and the
  rollback decision.
- A stale staging authorization cannot decide how much newer acknowledged work
  may be discarded. That delta is resolved against then-current authority and
  authorized separately at cutover.
- Rollback evidence is not a forensic bundle, backup bundle, or PITR candidate;
  representation similarity grants no cross-operation authority.

**Test requirements**

- Resolve retained generations with missing WAL, reclaimed blobs, overlapping
  tenant/key scope, divergent lineage, corrupt manifests, and incomplete
  reachability; candidates must be denied, degraded, or indeterminate without
  constructing a rollback plan.
- Race retention/reclaim after candidate resolution. Prove `ResolvedRollbackCandidate
  -> AdmittedRollbackSourceCut -> RollbackReachabilityLease ->
  LoweredRollbackPlanDag` is mandatory and the lease covers every source byte.
- Crash before and after source admission, staging authorization consumption,
  every owner-DAG barrier, staged-root close, post-verification, cutover
  authorization, fencing, publication, readmission, and source-lease release;
  fresh-process recovery must resume, abandon, or preserve an explicit pending
  state without changing the prior current root accidentally.
- Advance current authority while rollback stages. Fresh cutover resolution must
  report exact local-durable, client-ack, replication-ack, lineage, and epoch
  loss; stale cutover authorization or a post-authorization epoch change denies.
- Compile-fail and runtime tests prove restore, PITR, repair, and promotion
  authorizations, plans, receipts, and source leases cannot satisfy rollback
  APIs.

**Engineering decisions**

- Recovery Physics is the sole constructor of `ResolvedRollbackCandidate` from
  retained-authority evidence. Isolation admits and durably leases the exact
  source closure before owner lowering.
- The canonical rollback owner-plan DAG declares typed prerequisites,
  concurrency, durability barriers, irreversible points, expected owner
  receipts, abandonment posture, and staging/cutover separation. Owners never
  call one another.
- Rollback has operation-specific public typestates from intent through
  `ReadmittedRollbackCurrent`; private Proof carriers may implement the shared
  progression but never escape the rollback facade.
- Rollback publication follows the Phase 13 pending-readmission state machine.
  Retained source evidence is released only by explicit completion, abandonment,
  or forensic-retention policy.

**Open questions**

- None. Rollback has a distinct source and authority topology and therefore
  earns a distinct phase.

### Phase 11: Evidence-Bound Repair Resolution And Owner Lowering

Resolve an operational truth report into a closed repair candidate, then require
each affected owner to lower its concrete portion into one canonical plan DAG
whose preconditions, blast radius, source evidence, cost, and authority
consequences can be inspected before authorization.

**Relevant subsystems**

- `worth-store-operations`
- `worth-store-physical-integrity`
- `worth-store-recovery-physics`
- `worth-store-layout-indexes`
- `worth-store-blob-chunks`
- `worth-store-physical-isolation`
- `worth-store-authority`
- `worth-store-replication`

**Relevant APIs**

- `RepairIntent`
- `RepairCandidateSet`
- `EvidenceBoundRepairPlan`
- `DerivedRebuildPlan`
- `DerivedQuarantinePlan`
- `AuthorityQuarantinePlan`
- `TrustedSourceRestorePlan`
- `CurrentAuthorityPreservingMaintenancePlan`
- `AuthorityAffectingStagedRepairPlan`
- `UnrecoverableDamageReport`
- `RepairPlanExplanation`
- `LoweredRepairOwnerPlanDag`

**Warnings**

- There is no generic `RepairAction`. Derived maintenance, authority-affecting
  staged replacement, quarantine consequence, trusted-source restoration, and
  abandonment have different authority, cost, and failure topology. Rollback is
  the complete Phase 10 operation, not a repair class.
- Damaged authority cannot be repaired from an intact derived artifact or a
  lower-authority digest. It requires an admitted trusted upstream source or an
  unrecoverable result.
- A plan produced from stale observations is not safe merely because its target
  page still exists. Generations, roots, frontiers, scope, and neighboring blast
  radius may all have changed.
- Resolution and lowering cannot mutate quarantine, reserve output, consume
  authorization, or begin execution. Lowering may reject; it may not silently
  weaken or broaden the resolved candidate.
- “In-place repair” means only that current authority remains current while an
  artifact owner builds a replacement derived generation and performs its
  owner-local atomic swap. Authoritative bytes, global roots, lineage, and
  quarantine reachability are never mutated in place.
- Any quarantine that changes reachable data or current-authority posture is an
  authority-affecting staged repair. A `DerivedQuarantinePlan` is only the
  artifact owner's consequence node inside that larger DAG, never permission to
  alter reachability by itself.

**Test requirements**

- For every truth classification, prove the candidate set is exhaustive and
  rejects illegal source/target combinations: derived rebuild from authority,
  authority repair from derived state, cross-scope restore, and untrusted source
  promotion.
- Change any expected generation, root, frontier, quarantine set, security
  scope, custody posture, or source identity after planning; execution admission
  must reject the stale plan before the first mutation.
- Generate equivalent truth reports with different scan orders and prove
  canonical plans, footprints, preconditions, owner plan identities, and
  explanations converge.
- Give the planner overlapping repair regions owned by different subsystems;
  it must either establish an explicit ordered composition or deny ambiguity,
  never emit conflicting independent mutations.
- Generate quarantine findings that cross integrity, artifact, isolation, and
  authority boundaries. The plan must contain separate DAG nodes and receipts:
  Integrity classifies ranges, each artifact owner chooses consequence,
  Isolation changes reachability, and Authority establishes degraded or
  quarantined current posture.
- Permute owner-lowering order and prove the resulting non-empty owner-plan DAG
  has one canonical topology, unique owner/effect identities, identical plan
  basis, typed edges/barriers, and identical authorization fingerprint.
- Prove plan explanation accounts for every observed damaged region and every
  intentionally untouched region; omission is a plan-construction failure.

**Engineering decisions**

- Repair classification is a sealed sum type with separate plan types per
  repair class. A shared lifecycle may use phantom-tagged wrappers, but no API
  erases the operation kind.
- Repair first selects one topology: current-authority-preserving derived
  maintenance, or authority-affecting staged repair. The former builds and
  atomically swaps an owner-local derived generation without changing global
  authority. The latter always uses non-current copy-on-write staging,
  independent post-verification, fresh cutover resolution/authorization,
  fencing, publication, and readmission.
- The Worth Proof progression in this phase is `Unresolved -> Resolved ->
  Lowered`. Store-owned resolved and lowered facade types hide the raw recipe
  carrier while preserving basis, freshness, and checked denial/defer/stale/
  rebind/failure outcomes.
- Every plan binds canonical source evidence, expected owner generations and
  frontiers, exact read/write/quarantine/reachability footprint, preconditions,
  compensation/abandon posture, post-verifier requirements, idempotency identity,
  and cost estimate.
- Operations owns the canonical cross-owner DAG and explanation. Each owner
  constructs the plan node that it alone can execute. Typed edges encode
  prerequisites, concurrency, durability barriers, irreversible points,
  expected nested receipts, and abandonment posture. The DAG is non-empty,
  canonical, acyclic, complete against the resolved effects, and bound to one
  plan basis; owners never invoke one another.
- `UnrecoverableDamageReport` is a successful truthful planning outcome, not an
  internal error to be hidden or coerced into a best-effort repair.

**Open questions**

- None. Repair classes are fixed by authority and failure topology.

### Phase 12: Authorized Repair Admission And Crash-Safe Execution

Authorize the exact lowered repair owner-plan DAG, establish runtime readiness,
then execute it as a persisted, idempotent sequence and collect sealed receipts
without centralizing mutation law in operations.

**Relevant subsystems**

- `worth-store-operations`
- all mutation owners named in Phase 11
- `worth-store-physical-backend`
- `worth-store-io-scheduler`
- `worth-store-authority`

**Relevant APIs**

- `AuthorizedRepairPlan`
- `LoweredRepairOwnerPlanDag`
- owner-specific lowered plan types
- `RepairExecutionJournal`
- `RepairExecutionSession`
- `ExecutedRepairOwnerReceiptDag`
- `RepairAbandonmentReceipt`
- `IndeterminateRepairRecoveryHandle`

**Warnings**

- The scheduler may not reclassify damage, choose a source, expand footprint,
  alter artifact policy, or decide owner ordering. Those facts and all legal
  concurrency are frozen in the authorized DAG.
- A process-local state machine or log line is not a repair journal. Crash
  recovery must reopen durable phase state and owner receipts.
- Idempotency means repeating an admitted operation converges on the same
  effects and receipts; it does not mean swallowing mismatched generations or
  treating every existing output as success.
- Partial owner success is an explicit indeterminate or resumable state, never a
  generic failure that loses which effects became durable.

**Test requirements**

- Crash before and after authorization consumption, owner-plan start, every
  durable owner effect, receipt persistence, phase advance, cancellation,
  abandonment, and finalization; discard all process state and prove exact
  resume or safe stop from disk.
- Execute the same authorized plan concurrently and after restart; prove owner
  mutations occur at most once semantically and mismatched operation identities
  cannot adopt each other's journal or residue.
- Inject stale generations and changed blast radius between owner steps; the
  next owner must deny rather than expanding or recomputing the plan in the
  executor.
- Fail each owner independently after earlier owners succeed; the recovery
  handle must enumerate durable receipts, unapplied plans, safe resume
  preconditions, and whether abandonment or compensating isolation is possible.
- Instrument operations and prove it never calls backend mutation primitives or
  constructs owner receipts directly; all mutation crosses owner facades with
  concrete lowered plans.
- Attempt to wrap a generic Worth Proof `TransitionOutcome::Success`, a
  Foundational executed receipt, or a canonical digest in an owner-receipt
  variant; compile-fail and runtime boundary tests must prove only the concrete
  owner can construct the accepted receipt.
- Prove current-authority-preserving derived maintenance writes a new derived
  generation and swaps only through the artifact owner, while
  authority-affecting repair writes exclusively to non-current staging media.
  Controlled attempts to mutate authoritative current bytes must fail at the
  type/facade boundary.

**Engineering decisions**

- The control-store journal is canonical durable state with operation identity,
  authorized DAG fingerprint, node/edge progress, expected preconditions, authorization
  consumption, durable owner receipts, cancellation state, warnings, and final
  disposition.
- Owner receipts are persisted before the orchestration phase advances. On
  reopen, the owner validates receipt/effect correspondence rather than trusting
  the journal's copied summary.
- The proof progression is `LoweredRepairOwnerPlanDag -> AuthorizedRepairPlan ->
  ExecutionReadyRepair -> ExecutedRepair`.
  Authorization is admitted-plan authority, readiness additionally proves
  current basis/resource/owner availability, and execution alone produces
  Store owner receipts. A generic Proof outcome describes topology but never
  impersonates an owner outcome.
- Foundational planned/executed/completed receipt posture is derived only after
  Store owner receipts and journal closeout exist. It is for cross-crate support
  and audit attachment, never orchestration input.
- Successful derived maintenance closes with owner-local post-validation and a
  current-authority-preserving receipt. Successful authority-affecting repair
  closes only as `ExecutedAuthorityAffectingRepair` and must continue through
  Phase 13; execution alone cannot publish or readmit.

#### DX target: repair without a nightmare button

```rust
let candidates = RepairIntent::from_truth(offline_truth).resolve()?;
let selected = candidates.select(RepairClass::RebuildDerived)?;
let lowered = selected.lower_owners()?;

lowered.explain().show_sources().show_footprint().show_denials();

let admitted = authorization.authorize(lowered, operator_assertion)?;
let ready = admitted.ready(current_basis, repair_resources)?;
let session = operations.execute_repair(ready).await?;

match session.resume_after_crash()?.finish().await? {
    RepairExecution::Executed(receipts) => verify_after_repair(receipts).await?,
    RepairExecution::Indeterminate(handle) => return recover(handle).await,
    RepairExecution::Abandoned(receipt) => preserve_abandonment(receipt),
}
```

There is intentionally no `repair()`, `force()`, generic action list, or
boolean override. The normal path makes source, class, footprint, owner plans,
authorization, readiness, progress, and recovery disposition inspectable.
- Cancellation is cooperative and phase-aware. It never rolls back a durable
  authority mutation through an implicit inverse; it yields resumable,
  isolated, abandoned, or indeterminate state according to the plan.
- Backend and scheduler calls remain visible through owner plans and exact
  resource reservations; operations does not hide I/O behind a synchronous
  method.

**Open questions**

- None. Durable owner receipts are the only execution evidence.

### Phase 13: Independent Post-Verification, Cutover, Publication, And Readmission

Require a fresh independent observation of staged media, re-resolve the
post-verified candidate against then-current authority, authorize the exact
cutover and data-loss delta, establish a write fence/quiescent cut, publish one
non-current root, and separately readmit it. This is a second plan and authority
protocol, not the tail of staging execution.

**Relevant subsystems**

- `worth-store-offline-verifier`
- `worth-store-physical-integrity`
- `worth-store-recovery-physics`
- artifact owners
- `worth-store-physical-isolation`
- `worth-store-authority`
- `worth-store-operations`

**Relevant APIs**

- operation-specific post-verification requests and results, including
  `PostVerifiedBackupRestore`, `PostVerifiedPointInTimeRecovery`,
  `PostVerifiedRollback`, and `PostVerifiedAuthorityAffectingRepair`
- operation-specific `Resolved*CutoverCandidate` and `Lowered*CutoverPlanDag`
- operation-specific `Authorized*Cutover`
- `WriteFenceReceipt` or `QuiescentCutReceipt`
- `AtomicRecoveryPublicationReceipt`
- operation-specific `Published*PendingReadmission`
- `PublishedRejectedByAuthority`
- `PublishedAbandoned`
- `PublishedRetainedForForensics`
- `CurrentAuthorityReadmissionReceipt`
- `RestoreDrillCertification`
- `ReadmissionDenial`

**Warnings**

- Successful owner receipts prove that planned effects executed; they do not
  prove the complete resulting store is coherent.
- Verification performed through still-open execution handles can inherit stale
  cache, descriptor, or in-memory manifest state.
- Atomic root publication and current-authority readmission are separate
  transitions. A published non-current root does not automatically become
  current.
- Staging authorization says nothing about the current authority that exists
  after a long staging run. Reusing it for cutover can silently discard newly
  acknowledged work.
- A published root that fails readmission is a durable operational state, not an
  impossible error path or garbage that may be reclaimed implicitly.
- Post-verification cannot ignore declared degraded or quarantined regions.
  A degraded admission, if supported, must preserve those regions in the new
  authority contract rather than relabeling them healthy.

**Test requirements**

- After each restore, PITR, repair, rollback, and promotion class, close all
  execution handles and inspect/reopen the resulting media in a fresh process;
  the report must match the plan's promised frontier, scope, and residual
  posture.
- Corrupt staged media or change authority epoch between execution,
  post-verification, cutover resolution, cutover authorization, fencing,
  publication, and readmission. An epoch change before cutover resolution is a
  normal input that changes the explained delta; one after cutover authorization
  or fencing makes that authorization stale and denies publication/readmission.
- Crash around verification receipt persistence, root publication, fencing,
  readmission, and old-root retention; reopen and prove exactly one current
  authority with no split-brain or unclassified published root.
- Attempt to reuse a post-verification receipt for another operation, media
  generation, root, tenant/key scope, or frontier; construction or admission
  must fail.
- Compare independent post-verification with owner receipt summaries and inject
  a controlled owner bug that writes outside its promised footprint; the
  verifier and certification oracle must detect the discrepancy.
- Keep current authority writable throughout staging, acknowledge additional
  commits, then resolve cutover. The candidate must expose the exact local
  durable/client-ack/replication-ack loss and lineage divergence; authorization
  binds that delta, and any later acknowledged commit invalidates it unless the
  write fence already covers the transition.
- Deny readmission after successful publication, crash, and reopen. The control
  store must recover exactly one of `PublishedPendingReadmission`,
  `PublishedRejectedByAuthority`, `PublishedAbandoned`, or
  `PublishedRetainedForForensics`; retry requires fresh authorization, while
  reclaim requires an explicit Isolation-owned plan and receipt.
- Compile-fail tests prove `Executed*` cannot call post-verification-free
  publication, `PostVerified*` cannot publish without its named cutover
  authorization/fence, and a pending publication cannot be treated as current.

**Engineering decisions**

- Post-verification consumes closed/frozen media identity and produces a sealed
  operation-specific result bound to exact execution receipts and resulting
  media generations. Public typestates remain operation-named through
  `Executed* -> PostVerified* -> Published*PendingReadmission ->
  Readmitted*Current`; generic carriers are private only.
- Cutover resolution freshly reads current authority after post-verification,
  computes the exact frontier/lineage/security-scope delta, and lowers a distinct
  canonical owner-plan DAG. Fresh cutover authorization binds that DAG and
  delta. Publication additionally requires a current write-fence/quiescence
  receipt and compare-and-swap against the same authority epoch.
- Physical isolation owns atomic publication; authority owns fencing and
  readmission. Neither may synthesize the other's receipt.
- Worth Proof trust-boundary/readmission law carries the Store staged basis from
  boundary-bridged or authority-revalidation-required posture back to a current
  admitted basis. Foundational current-basis boundary evidence may attach only
  after the Store authority receipt exists and cannot substitute for it.
- Readmission records trusted, degraded, rebuildable, quarantined, and
  unavailable regions as explicit current-authority posture.
- Old authority remains fenced and governed by an explicit retention or
  retirement plan; no successful operation implicitly destroys recovery
  evidence.
- Publication disposition is durable control state. A pending root is retryable
  only after authority revalidation and, where required, fresh authorization;
  rejection preserves the reason/current basis; abandonment and forensic
  retention are explicit plans; reclamation is Isolation-owned and never an
  automatic readmission failure side effect.
- Certification may construct `RestoreDrillCertification` only from the
  ordinary restore execution, fresh-process post-verification, publication,
  current-authority reopen, and final offline truth receipts.

**Open questions**

- None. Post-verification, cutover authorization, fencing, publication, and
  readmission are separate compiler-visible gates.

### Phase 14: Replica Bootstrap, Disaster Recovery, Fencing, And Promotion

Define replica bootstrap and disaster recovery as lineage- and frontier-aware
authority protocols, including explicit old-primary fencing and divergent
history handling.

**Relevant subsystems**

- `worth-store-replication`
- `worth-store-authority`
- `worth-store-recovery-physics`
- `worth-store-physical-isolation`
- `worth-store-offline-verifier`
- `worth-store-operations`
- backup and blob owners

**Relevant APIs**

- `ReplicaBootstrapIntent`
- `ResolvedBootstrapSourceCut`
- `BootstrapReachabilityLease`
- `ReplicaBootstrapPlan`
- `ReplicaBootstrapReceipt`
- `MaterializedDisasterRecoveryBundle`
- `IndependentlyVerifiedDisasterRecoveryBundle`
- `ReplicaPromotionIntent`
- `DivergentReplicaHistoryReport`
- `PromotionFencePlan`
- `PrimaryServeLease`
- `OperationalFencingAuthorityPort`
- `FenceProof`
- `PromotedAuthorityEpoch`
- `OldPrimaryRejoinPlan`

**Warnings**

- Replica bootstrap, backup restore, and replica promotion may share transfer
  mechanics but do not share source authority or failure topology.
- The most advanced observed LSN is not necessarily the most advanced
  acknowledged or admissible authority frontier.
- Promotion without a durable fence and a new authority epoch permits
  split-brain even if the old primary is believed unavailable.
- A record written only on the promoting replica cannot fence an unreachable old
  primary. Epoch monotonicity and exclusion must come from an external quorum,
  shared-storage token/CAS authority, or a STONITH-equivalent fencing provider.
- A returning old primary never rejoins by comparing timestamps or accepting
  the promoted root. Divergent history must be classified and explicitly
  reconciled, retained for forensics, or discarded under authorization.

**Test requirements**

- Bootstrap a replica from a stable cut plus streamed tail under concurrent
  writes, compaction, blob movement, interruption, and restart; final offline
  truth and acknowledged frontier must match the declared bootstrap source.
- Partition primary and replica at every acknowledgment/publication boundary,
  then promote. Every serving primary must hold and renew a
  `PrimaryServeLease`; on expiry or renewal/token failure it fails closed before
  accepting or acknowledging another mutation. Promotion is unsupported or
  unavailable unless the fencing authority proves the old lease/token revoked
  and allocates a monotonic new epoch.
- Generate replicas with higher unacknowledged LSNs, lower acknowledged
  frontiers, missing blob chunks, damaged derived indexes, and divergent
  authority history; promotion must report exact RPO/trust posture and deny
  inadmissible candidates.
- Return the old primary before, during, and after promotion; ordinary rejoin
  must remain uncallable until an explicit divergence plan completes.
- Crash around fence persistence, epoch allocation, staged publication,
  readmission, and DR bundle finalization; fresh-process inspection must find at
  most one current authority and explain every non-current lineage.
- Race bootstrap source retention/reclaim and prove the progression
  `ResolvedBootstrapSourceCut -> BootstrapReachabilityLease ->
  LoweredReplicaBootstrapPlan` is mandatory and crash recoverable.
- Materialize a DR bundle with valid per-component digests but broken
  cross-component reachability/frontier closure. It must remain materialized and
  cannot construct `IndependentlyVerifiedDisasterRecoveryBundle` or feed
  bootstrap/PITR/promotion.

**Engineering decisions**

- Replication owns observed and acknowledged replica frontiers and bootstrap
  state; recovery physics evaluates recoverability; authority owns fencing and
  epoch progression; operations coordinates the workflow.
- Bootstrap resolution yields a Recovery-Physics-owned source cut; Isolation
  durably leases its checkpoint/WAL/page/blob closure before owner-plan lowering.
  The lease remains through independent verification of the bootstrapped target
  and is released only by durable completion or explicit abandonment.
- `MaterializedDisasterRecoveryBundle` is self-describing and binds source lineages,
  authority epochs, exact frontiers, backup/checkpoint/WAL components, blob
  reachability, scope/custody/authenticity posture, format/backend assumptions,
  and expected RPO. Only a separate offline pass over independently opened media
  may construct `IndependentlyVerifiedDisasterRecoveryBundle`.
- The fencing domain is explicit: all primary serve/ack paths require a renewable
  lease or storage token issued by `OperationalFencingAuthorityPort`. Supported
  implementations are external quorum, shared-storage CAS/token authority, or
  STONITH-equivalent exclusion. If the configured backend cannot prove remote
  exclusion, promotion returns typed `FencingUnsupported`; inability to reach
  the provider returns `FencingUnavailable`.
- Promotion produces a fresh authority epoch selected by the fencing authority
  and a durable `FenceProof` before publication/current readmission. No local
  node may allocate an epoch from its own last-seen value.
- Foundational lineage projections preserve attested source continuity,
  replay-derived recovery, promoted continuity, partial continuity, and
  divergence as different variants. Store replica/frontier/fence receipts are
  created first and remain the only promotion inputs.
- Replica promotion has its own authorization tag, audit vocabulary, receipts,
  and post-verification lane. It is never represented as restore.

#### DX target: promotion makes split-brain risk visible

```rust
let candidates = DisasterRecoveryInspection::from_offline_reports(reports)
    .promotion_candidates()?;
let selected = candidates.select(replica_id)?;

selected.explain_rpo();
selected.explain_divergence();
selected.explain_required_fences();

let lowered = selected.lower(replication, recovery, isolation, authority)?;
let admitted = authorization.authorize(lowered, operator_assertion)?;
let ready = admitted.ready(fencing_authority, current_serve_lease, current_basis)?;
let promoted = operations.promote(ready).await?.resume_after_crash()?.finish().await?;
let postverified = offline_verifier.verify_promotion(promoted.closed_media()).await?;
let cutover = postverified.resolve_cutover(authority.current()?)?;
let authorized = authorization.authorize_promotion_cutover(cutover.lower()?, operator_assertion)?;
let fenced = authorized.fence_old_primary(fencing_authority)?;
let published = operations.publish(fenced).await?;
let current = authority.readmit(published)?;
```

No `promote(replica_id)` shortcut exists. Candidate trust, exact RPO,
divergence, required fences, authority epoch, and old-primary rejoin posture are
visible before authorization.

**Open questions**

- None. Fencing and divergent-history law are mandatory DR foundations.

### Phase 15: Forensic Acquisition And Structured Operational Audit

Preserve damaged, quarantined, and operator-touched evidence without granting
it backup or restore authority, and emit one canonical structured operational
record stream for every workflow decision and effect.

**Relevant subsystems**

- `worth-store-offline-verifier`
- `worth-store-operations`
- `worth-store-physical-backend`
- `worth-store-physical-integrity`
- `worth-store-authority`
- S.5.1 security-scope vocabulary
- S.11 audit/security adapter boundary

**Relevant APIs**

- `ForensicAcquisitionIntent`
- `ForensicAcquisitionPlan`
- `ForensicBundle`
- `ForensicCustodyRecord`
- `OperationalAuditRecord`
- `OperationalDecisionTrace`
- `OperationLocalSequence`
- `AuditCausalParent`
- `AuditCompletenessReceipt`
- `OperationalEvidenceExport`

**Warnings**

- A forensic bundle may intentionally preserve checksum-failed,
  authenticity-unavailable, quarantined, or unknown bytes. Calling it verified
  backup material would erase the central distinction.
- Acquiring evidence must not update access times, repair headers, normalize
  records, or otherwise modify the source media.
- S.10 audit records are structured and canonical but not yet claimed
  tamper-evident. S.11 owns signing, chaining, encryption, key custody, and
  provider-bound actor proof.
- Human-readable logs and terminal projections are derived views and cannot be
  the only record of a decision, denial, warning, effect, or indeterminate state.
- Audit delivery is not allowed to be both lossy and authoritative. Crash
  ordering between a durable workflow transition, owner receipt, and emitted
  audit record must be defined, including duplicate delivery after recovery.

**Test requirements**

- Acquire a damaged store through read-only media handles and prove byte-level
  source immutability, bounded streaming memory, deterministic range identity,
  and localization of read failures or holes.
- Include quarantined and authenticity-unavailable ranges in a forensic bundle;
  backup verification and restore admission must reject the bundle even when
  its outer digest and custody record are valid.
- Crash/resume acquisition and export at every component boundary; the final
  bundle must preserve source identity, acquisition order, holes, retries,
  observers, and custody transitions without duplicate or silently omitted
  ranges.
- Reconstruct each operational workflow solely from structured audit records
  and owner receipts; remove any phase record and prove completeness validation
  identifies the exact missing transition.
- Crash between every durable workflow record and audit delivery. Reopen must
  derive at-least-once audit output from durable control artifacts, assign the
  same canonical record identity, and deduplicate without losing causal order.
- Duplicate, reorder, and replay valid audit deliveries. Canonical assembly uses
  operation identity plus operation-local sequence and causal parentage; it
  rejects conflicting duplicate payloads and proves completeness against the
  journal's expected transition set.
- Project records to logs/JSON and attempt readmission; terminal output must be
  unable to reconstruct authorization, owner receipts, forensic custody, or
  authority capability.

**Engineering decisions**

- Forensic acquisition is observation-only and has a distinct bundle magic,
  manifest, type, and facade from backup/export.
- `ForensicBundle` is never a restore source. Any future salvage path must first
  admit individual evidence through a distinct reconstruction workflow with new
  source semantics and authorization; readmission cannot upgrade the bundle.
- The custody record captures source identity, physical ranges, acquisition
  method, observer/provider assertions as untrusted/admitted evidence,
  timestamps with clock provenance, integrity/authenticity/custody posture,
  transformations, gaps, and handoffs.
- Every S.10 workflow emits canonical `OperationalAuditRecord` values derived
  from typed intent, decision, authorization, owner receipt, warning,
  publication, denial, and recovery-handle artifacts.
- The durable workflow/control record and owner receipt are the source of audit
  truth. Audit records are derived at least once with a canonical identity,
  operation-local monotonic sequence, causal parent ids, transition kind, and
  source artifact ids. Receipt durability precedes the transition that expects
  it; audit emission may follow and is safely repeatable.
- Audit completeness is checked against the operation journal's canonical
  expected transition set and terminal disposition, not merely against contiguous
  sequence numbers. Duplicate identity with unequal content is control-state
  corruption; equal duplicates collapse deterministically.
- S.11 extends the same record schema with tamper evidence and proof-of-
  possession bindings; it does not fork a parallel security audit log.
- Boundary materialization uses Foundational categories and roles rather than
  one attachment map: operational decisions become decision rows, failures
  become failure rows, comparisons become comparison rows, support gaps become
  support rows, provenance becomes provenance-ready rows, and owner receipts
  remain distinct receipt-evidence attachments.
- Audit/support richness uses Foundational requested -> admitted -> materialized
  profiles. Redaction, elision, unavailability, reconstruction cost, and named
  gaps remain explicit; a low-richness profile cannot omit the minimal facts
  required to explain an authority transition.

#### DX target: support widening is explicit

```rust
let record = operation.completed_audit_record(); // narrow canonical Store fact
let request = record.request_support_projection(requested_profile);
let plan = request.plan_materialization()?;

plan.explain_cost();
plan.explain_redactions();
plan.explain_unavailable_attachments();

let materialized = plan.materialize()?;
let exported = materialized.prepare_foundational_boundary_bundle()?;
```

Audit truth is cheap to retain and inspect narrowly. Rich diagnostics,
provenance, lineage, performance rows, and terminal projections are planned
materialization boundaries, never ambient work inside owner execution.

**Open questions**

- None. Forensics and backup must remain structurally non-interchangeable.

### Phase 16: S.4.5 Production Drivers, Yieldpoints, And Fault Program

Extend the physical simulation harness with production-boundary control over
every new long-running S.10 protocol before certification relies on it.

**Relevant subsystems**

- S.4.5 harness and physical certification crates
- all S.10 production owners
- `worth-store-certification`
- process/crash, corruption, storage, scheduler, clock, and authorization test
  drivers

**Relevant APIs**

- production drivers for control-store recovery, backup, restore, PITR,
  rollback, repair, promotion, verifier, forensics, and staging/cutover authorization
- named S.10 yieldpoints
- typed S.10 faults and corruption operators
- deterministic schedules, transcripts, shrink traces, and evidence bundles
- `ScenarioScaleProfile`, phase/owner/fault/oracle coverage matrix, and the three
  milestone-spanning scenario programs

**Warnings**

- Test support may schedule and observe production behavior; it may not mutate
  private state, mint proof types, or decide the oracle verdict.
- Sleeping, wall-clock racing, process-local mocks, or same-run comparison do
  not prove crash or concurrency behavior.
- A synthetic in-memory store cannot close real-media, durable-journal,
  stable-cut, publication, or fresh-process verification claims.
- Adding scenario names without production driver dispatch is fixture theater.

**Test requirements**

- Expose named yieldpoints before/after every durable transition in backup cut,
  materialization, restore/PITR stage and replay, authorization consumption,
  source leases, owner-DAG journals, cutover resolution/authorization,
  publication disposition/readmission, control-store generation, serve-lease
  renewal/expiry, promotion/fencing, and forensic acquisition; prove schedules
  dispatch real owner calls.
- For each yieldpoint, crash the process, discard live state, reopen actual
  files through a new process/session, and compare with an independent oracle
  derived from fixture construction, client acknowledgment history,
  independently recorded workload semantics, and injected-fault records.
- Provide generated corruption operators for bit flips, torn sectors/frames,
  truncation, omission, duplication, reordering, stale generation, wrong scope,
  source substitution, partial copy, and post-copy corruption.
- Deterministically replay and shrink failing schedules while preserving the
  owner call sequence, injected faults, durable transcript, exact counters, and
  oracle verdict.
- Prove test-only authority cannot enter production facades and that every
  controlled shortcut—logs, fixture labels, caller declarations, private state
  mutation, same-run self-comparison—is rejected by an oracle or compile gate.

#### Milestone-spanning acceptance scenarios

These are not soak labels or umbrella tests that call smaller tests. Each is one
deterministic, shrinkable scenario program that enters through ordinary public
facades, persists real media, crashes real processes/sessions, and requires one
joined evidence bundle. Every run begins with the Phase 1/2 structural preflight
and ends with the Phase 17/18/19 model, performance, controlled-defect, and
closeout joins. The scenario invocation transcript must expose every one of the
19 phases; static preflight and closeout joins are invocation evidence, not
fictional runtime steps.

The oracle contract is strict: expected truth may come only from deterministic
fixture-construction records, independently captured client acknowledgment
history, independently recorded workload semantics, the injected-fault program,
and an independently implemented narrow media parser where a byte-level oracle
is necessary. Production receipts, manifests, reports, digests, audit records,
and summaries are observations under judgment; they may never confirm
themselves or define expected truth.

##### Scenario A: Burning Primary, Poisoned Backup, Exact-Frontier Escape

**World**

- a store at least eight times the verifier resident-memory budget in the
  release profile, with authoritative pages, two layout families, multi-GB blob
  trees, several checkpoints, a long segmented WAL tail, four tenant/key scopes,
  and an independently recorded semantic workload oracle
- foreground writers and readers run while checkpoint, compaction, reclaim,
  blob migration, scrub, backup materialization, and replica streaming contend
- two backup cuts straddle a root/checkpoint publication; the newer cut is
  materialized under interruption while the older cut remains independently
  verified

**Hostile program**

1. Crash after the new cut's reachability lease becomes durable but before its
   manifest binding is durable; resume from the independent control store and
   prove the same cut identity. Duplicate then corrupt control records and prove
   destructive resume fails closed until one generation is selected or safely
   reconstructed and freshly authorized.
2. During materialization, tear one WAL segment, omit one protected blob chunk,
   substitute a checksum-valid derived index from another generation, and
   recompute every non-authoritative outer digest available to the attacker.
3. Independently verify the poisoned bundle and require rejection based on
   Store completeness, generation, reachability, and frontier law rather than
   digest mismatch alone. Prove the cut lease is not released before this
   independent verdict is durable.
4. Destroy part of the primary: corrupt one authority page, several derived
   pages, a blob chunk, and the newest checkpoint while leaving misleading
   runtime logs and an intact in-memory cache behind.
5. Start a fresh verifier process from real media, classify truth, choose the
   older production-restore-admissible backup plus the admissible WAL interval, and resolve an exact
   PITR frontier near an ambiguous wall-clock request.
6. Admit a PITR source cut and durable reachability lease, lower the restore/PITR
   owner DAG, then race one wrong-scope authorization,
   one replay of a consumed authorization, and the valid assertion. Only the
   valid lowered-plan binding may become execution-ready.
7. Keep current authority writable and acknowledge new commits while staging.
   Crash once during staging copy, once during WAL replay, once after an owner
   receipt but before journal phase advance, and once after post-verification
   but before cutover resolution. Resume each time from control state, resolve
   the exact data-loss/lineage delta against the now-current authority, and
   require a fresh cutover authorization plus write fence.
8. Inject an out-of-footprint derived write into a controlled mutant; fresh-
   process post-verification must reject it. Run the clean path, atomically
   publish the staged DR replica, deny its first readmission, recover
   `PublishedRejectedByAuthority`, freshly authorize retry, fence through the
   external fencing authority, promote under its fresh epoch, readmit, reopen,
   and compare exact truth/frontiers with the independent workload oracle.
9. Preserve the destroyed primary as a forensic bundle, prove it cannot enter
   backup restore, and reconstruct the complete decision/receipt/custody story
   from structured audit records with no log interpretation.

**Required oracle verdict**

- no mixed-generation cut, reclaimed leased byte, poisoned backup admission,
  wrong/replayed authorization, in-place current mutation, skipped owner
  receipt, pre-verification publication, or forensic-to-backup promotion
- exact RPO/frontier, residual degradation/quarantine, owner effects, operation-
  local counters, Foundational lowering posture, and Proof progression for every
  transition
- controlled mutants for digest-as-authority, authorization-before-lowering,
  omitted blob/source lease, corrupt control-state resume, stale cutover
  authorization, and publication-before-post-verification all rejected and localized

##### Scenario B: Split-Brain Datacenter Reversal And Returning Old Primary

**World**

- one primary and three replicas across two failure domains, with different
  observed LSNs, acknowledged frontiers, blob completeness, checkpoint age,
  security-scope availability, and background lag
- one replica has the highest observed LSN but lower acknowledged truth, one has
  the best acknowledged frontier but a damaged blob range, and one has a clean
  older frontier plus a complete DR bundle
- deterministic network control can partition, duplicate, delay, and reorder
  acknowledgments independently of media durability
- an external fencing authority issues renewable primary serve leases and
  monotonic epochs; test control can partition the authority separately from
  replication and data media

**Hostile program**

1. Under foreground and maintenance load, admit a stable online DR cut,
   materialize and independently verify its bundle as two distinct states, then partition immediately
   around the next acknowledgment and root publication while the old primary
   continues locally and the disaster site loses power.
2. Acquire and independently verify all surviving replica/backup media without
   the live primary; classify divergent histories, exact acknowledged frontiers,
   gaps, damaged derived state, missing chunks, and custody/authenticity
   availability.
3. Require the promotion planner to reject "highest LSN wins," explain exact RPO
   for each candidate, and select a lower-observed but better-admitted source
   plus a trusted repair/bootstrap source for its damaged blob range.
4. Resolve exact-frontier PITR from the verified DR bundle and surviving WAL,
   admit durable PITR and bootstrap source leases, then lower non-current
   restore, bootstrap, authority-affecting repair, recovery, fence, epoch,
   publication, and authority plans. Revoke the first authorization after
   lowering, authorize a fresh canonical plan, and crash during restore staging,
   after fence durability, after epoch allocation, and after root publication
   but before readmission.
5. Partition the old primary from the fencing authority but leave clients
   connected. It may serve only until its existing lease expires; after expiry,
   every read-as-current, mutation, and acknowledgment path fails closed. Prove
   promotion is unavailable before external revocation/expiry proof, then reopen
   after promotion and prove exactly one current authority.
6. Duplicate and independently advance the operational control store at the two
   sites. Neither copy may authorize resume or epoch allocation until the
   fencing authority selects one generation; stale-copy replay remains denied.
7. Produce an explicit old-primary rejoin plan that retains divergent media for
   forensics, bootstraps from the promoted authority, and cannot collapse
   promoted, replay-derived, partial, or divergent lineage.
8. Verify the rebuilt replica, complete rejoin, and reconstruct the entire
   promotion/fence/RPO/lineage/audit story from canonical records.

**Required oracle verdict**

- at most one lease-holding current authority in every crash state, externally
  monotonic epochs, exact multi-dimensional frontier/RPO accounting, durable
  pre-publication fence proof, fail-closed lease expiry, and no implicit
  divergent-history discard
- Foundational lineage variants remain semantically distinct and none can feed
  Store promotion authority; Store fence, promotion, and readmission receipts
  are the only runtime inputs
- controlled mutants for highest-observed-frontier selection, fence-after-
  publication, locally allocated epoch, serve-after-lease-expiry, stale control-
  generation selection, and automatic old-primary rejoin all rejected and localized

##### Scenario C: Byzantine Maintenance And Multi-Owner Repair Marathon

**World**

- a greater-than-memory store with hundreds of independently localizable damage
  regions spanning authority pages, WAL, manifests, two index layouts, blob
  chunks, quarantine metadata, and stale replica state
- foreground load, backup leases, PITR retention, compaction, reclaim, scrub,
  and replica bootstrap all overlap the repair window
- the independent fixture oracle knows source authority, derived rebuild bases,
  tenant/key/custody scopes, expected generations, and every injected byte

**Hostile program**

1. Run a bounded offline scan with restarts and adversarial batch widths; require
   an immutable consistency basis, canonical non-overlapping physical coverage,
   explicit aliases/overlaps, and explicit unknown/unavailable rows. Corrupt a
   checkpoint and prove bounded rewalk rather than skipped ranges.
2. Resolve current-authority-preserving derived maintenance separately from an
   authority-affecting staged repair containing authority quarantine,
   trusted-source restore, and unrecoverable regions. Attempt to use
   a derived index as authority source and a cross-tenant blob as repair source;
   both must be structurally denied.
3. Lower every owner node in permuted order and require one canonical acyclic
   owner-plan DAG with typed prerequisites, barriers, irreversible points,
   expected receipts, and a stable authorization fingerprint. Split quarantine
   across Integrity classification, artifact consequence, Isolation
   reachability, and Authority posture nodes. Mutate one generation
   after lowering; the stale plan may not be re-lowered inside execution.
4. Authorize the exact lowered DAG, then crash before/after authorization
   consumption and every owner effect/receipt/journal transition. Concurrently
   attempt a generic Proof success outcome and a Foundational executed receipt
   as forged owner receipts.
5. Revoke authorization at its declared boundary, cancel and resume the
   operation, inject scheduler denial and backend indeterminacy, and require the
   recovery handle to preserve every durable effect and safe next action.
6. Prove derived maintenance publishes only a new owner-local derived generation
   while authoritative repair writes only to non-current staging. Let a
   controlled owner mutant write one extra page outside its footprint and
   another mutant omit a completed receipt. Fresh-process post-verification and
   receipt/effect correspondence must reject both before publication.
7. Complete the clean repair, freshly resolve/authorize cutover, publish/readmit,
   then execute a retained-authority rollback through its own admitted source
   cut, lease, DAG, authorization, staging, post-verification, and cutover. Finish and independently verify
   the concurrent backup, execute a restore/reopen drill plus exact-frontier PITR
   drill into isolated targets, verify all backup/PITR leases survived, and
   materialize narrow plus rich support/audit/performance projections.

**Required oracle verdict**

- no proof bag, raw vector, optional-field report, generic transition outcome,
  Foundational support artifact, stale plan, or executor re-decision can cross
  an owner or authority seam
- exact owner-DAG effects/edges/barriers, journal states, recovery dispositions,
  quarantine/degradation truth, resource interference, and operation-local
  counters remain reconstructable after every crash
- controlled mutants for proof-field copying, receipt omission,
  out-of-footprint current-authority mutation, stale-plan re-lowering,
  repair-as-rollback, and same-run post-verification all rejected and localized

#### Harness scaling contract

- `ScenarioScaleProfile` changes cardinality and exploration budget, never
  actor topology, phase topology, fault classes, owner calls, oracle families,
  denial lanes, or required evidence. Smoke/CI is the same scenario with fewer
  pages/chunks/regions and schedules, not a simplified fake path.
- `smoke` runs every semantic phase and one crash cut per durable transition;
  `ci` adds pairwise actor/fault coverage, every authorization denial, every
  controlled mutant, and store bytes at least twice the resident budget;
  `release` uses at least eight-times-memory media, exhaustive durable cutpoints,
  three-way high-risk interaction coverage, multi-GB blobs, long WAL tails, and
  seeded soak schedules.
- The scheduler uses dependency-aware partial-order reduction from declared
  owner read/write/reachability footprints. Independent commutations collapse;
  conflicting, authority-changing, durability-changing, and fault-adjacent
  orderings never collapse.
- Scenario generation is coverage-driven, not a Cartesian product. The required
  matrix is phase transition x owner x fault family x oracle family x profile,
  with targeted pairwise/three-way interaction rows for named high-risk seams.
- Shrinking may reduce cardinality and unrelated schedules but may not remove
  the phase transition, owner boundary, fault, basis change, or independent
  oracle needed to explain the failure. Shrink output remains executable from a
  clean checkout.
- Every phase must appear in at least two of the three scenario transcripts,
  contribute an ordinary production artifact to the joined evidence bundle,
  and have at least one controlled defect that the scenario localizes to that
  phase. Phase 1/2 static preflight and Phase 17-19 evidence joins are part of
  each scenario invocation, not separate paperwork.

The minimum phase coverage is explicit:

| Phase | Scenario coverage | Required joined evidence |
| --- | --- | --- |
| 1 | A, B, C | boundary/adoption ledger, honest compile-fail localization, shortcut scan |
| 2 | A, B, C | independent control-store trust/generation/recovery plus import graph and directory topology |
| 3 | A, B, C | consistent fresh-process real-media acquisition, bounded cursor/restart-or-rewalk receipt |
| 4 | A, B, C | canonical non-overlapping Store truth, aliases/gaps, Recovery-owned candidate view, support lowering |
| 5 | A, B, C | stable cut, complete reachability lease retained through verification, crash-safe release/abandonment |
| 6 | A, B, C | materialized/structurally-verified/custody-qualified/restore-admissible bundle ladder |
| 7 | A, B, C | named staging/cutover authorization, provider outcomes, consumption/replay/revocation boundary |
| 8 | A, B, C | non-current restore staging, resume/abandon posture, current-root non-mutation |
| 9 | A, B, C | admitted PITR source lease, exact multi-dimensional frontier, replay-derived continuity |
| 10 | A, C | retained-authority rollback source lease, named DAG/session/receipts, fresh cutover |
| 11 | B, C | split repair topology, canonical owner-plan DAG, quarantine owner seams, stale-plan denial |
| 12 | B, C | execution-ready repair, durable DAG journal, concrete owner receipts, forged-receipt denial |
| 13 | A, B, C | post-verification, fresh cutover delta/auth/fence, pending-publication/readmission disposition |
| 14 | A, B | bootstrap source lease, verified DR bundle, serve-lease fencing, external epoch, promotion/rejoin |
| 15 | A, B, C | read-only forensic custody and complete causal at-least-once audit projection |
| 16 | A, B, C | production scenario invocation transcript, deterministic replay, shrink, independent oracle coverage |
| 17 | A, B, C | forward-simulation mapping, reached transitions, controlled mutant, localized counterexample |
| 18 | A, B, C | exact subsystem counters, bounded RSS/profile evidence, slope/QoS/session recovery |
| 19 | A, B, C | reproducible closeout join and typed S.11/S.12 handoff denial/readiness evidence |

**Engineering decisions**

- S.10 extends the existing S.4.5 authoring, scheduling, driver, transcript,
  oracle, counter, and evidence vocabulary; it does not create a recovery-local
  harness.
- Driver methods correspond to production facade transitions and return the
  actual production artifacts. Test support never fabricates successful owner
  receipts.
- Schedules cover meaningful dependent actor orderings and interference with
  foreground writes, compaction, checkpoint, scrub, replication, blob movement,
  reclaim, cancellation, and provider availability through the scaling contract
  rather than enumerating an unbounded full permutation set.
- Heavy media fixtures are generated deterministically and may exceed memory;
  their expected truth comes from independent construction records, not the
  runtime under test.
- Production receipts, manifests, summaries, audit records, and control-store
  projections are observations only. The oracle uses independent construction,
  client acknowledgments, workload semantics, injected faults, and, where
  required, a separately implemented narrow parser.

**Open questions**

- None. S.10 correctness requires harness expansion now, not in S.12.

### Phase 17: Formal Protocol Extensions And Controlled-Defect Sensitivity

Extend S.9's checked law for the new destructive protocols and prove the models
remain diagnostic refinements of ordinary owner behavior rather than runtime
authorization.

**Relevant subsystems**

- `worth-store-formal-models`
- S.10 production owner outcome bindings
- S.4.5 counterexample round trips
- `worth-store-certification`

**Relevant APIs and model families**

- stable online backup cut and lease lifecycle
- backup materialization/final-manifest publication
- PITR/bootstrap/rollback source-cut lease lifecycle
- restore/PITR/rollback/authority-repair staging, replay, post-verification,
  fresh cutover authorization, publication disposition, and readmission
- staging/cutover authorization consumption and owner-plan DAG progression
- control-store atomic prefix, generation selection, divergence, and recovery
- replica promotion, serve-lease expiry, external fencing/epoch, and old-primary rejoin
- audit derivation, duplicate collapse, causality, and completeness
- shared durability, reachability, quarantine, acknowledged, and authority
  frontiers

**Warnings**

- Model actions, checked verdicts, trace labels, and case ids are observations,
  not operational authority.
- The model cannot invent fictional production outcomes to make a state machine
  complete. Missing owner capability is implementation work.
- Bound exhaustion, deadlock in the runner, unavailable toolchain, and a model
  that never reaches a transition are not passes.
- A mutation program that changes only model text without demonstrating
  production relevance can create false sensitivity.

**Test requirements**

- Define a total deterministic abstraction from each in-scope concrete
  production observation/transition to model state and action. Prove forward
  simulation for every production transition, preservation of every
  invariant-relevant property, and at least one ordinary production scenario
  exercising each reachable model transition. A bijection is neither required
  nor claimed: multiple concrete events may refine one model action.
- Check invariants forbidding mixed-generation backup cuts, reclaimed protected
  bytes, current in-place restore mutation, publication before verification,
  stale-cutover authorization, authorization replay, execution outside
  footprint, resume from unselected control state, serve after lease expiry,
  split-brain promotion, stranded unclassified publication, incomplete audit,
  and old-primary rejoin without divergence resolution.
- Inject at least one controlled defect per model family into the production-
  relevant transition edge; TLC/equivalent and the executable harness must
  reject and localize it to the intended invariant/owner boundary.
- Round-trip minimized counterexamples into deterministic S.4.5 scenarios and
  return durable execution traces to model-state valuations without using the
  model verdict as execution permission.
- Prove clean checkout reproducibility with pinned tool identity, explicit
  finite bounds, state-space counts, reachability coverage, deadlock status,
  and bound-exhaustion reporting.

**Engineering decisions**

- New protocols extend the existing S.9 model families where lifecycle and
  shared frontiers genuinely compose; distinct failure topologies remain
  distinct submodels.
- Production owner outcomes are the sole runtime observation source for model
  bindings. Certification composes checked and executed evidence.
- Mutation identities bind the exact changed edge, expected invariant,
  model/tool version, harness scenario, and localized counterexample.
- Formal execution remains off all ordinary backup, verification, restore,
  PITR, rollback, repair, promotion, and forensic paths.

**Open questions**

- None. The required protocol families are named by new S.10 authority changes.

### Phase 18: Performance Contracts, Exact Counters, And Operational Sessions

Make the cost, interference, progress, cancellation, and recovery posture of
every long-running operation visible and mechanically bounded.

**Relevant subsystems**

- `worth-store-operations`
- `worth-store-offline-verifier`
- all executing owner crates
- `worth-store-io-scheduler`
- `worth-store-buffer-pool`
- `worth-store-certification`

**Relevant APIs**

- operation-specific session and recovery-handle types
- `OperationalProgressEvent`
- operation-bound counter receipts
- complexity contract registry entries
- execution policy, cancellation, deadline, resource budget, and artifact policy

**Warnings**

- A global counter snapshot lets unrelated work satisfy or distort an operation's
  proof. Counters must bind invocation/operation identity and phase.
- Elapsed time cannot explain page breadth, allocation, retry, interference,
  amplification, or forbidden full-store materialization.
- Ordinary verification cost and reconstructive restore/repair cost must not
  share a cheap-looking API or performance contract.
- Cancellation that drops a handle without persisting phase disposition leaks
  leases, staging roots, authorization state, or repair journals.

**Test requirements**

- For each operation, assert exact deterministic counters for forbidden work and
  event structure, including verifier-buffer bytes, pinned frames,
  decoder-owned allocations, backend-requested bytes, page/chunk touches,
  logical decodes skipped, generations checked, leases acquired/released,
  owner DAG nodes/receipts, sync/publication steps, and authorization consumption.
- Scale media beyond memory and increase damaged-region, WAL-tail, blob, and
  candidate breadth independently; prove measured slopes match declared
  complexity variables and resident/allocation bounds stay fixed or explicitly
  budgeted.
- Run foreground traffic with backup, verifier, restore staging, repair,
  bootstrap, and forensics; prove I/O reservations, background yields, queue
  depth, foreground waits, and interference remain inside the admitted profile.
- Interleave unrelated operations while measuring one session; its counters and
  evidence must exclude foreign work while global observability still accounts
  for all work exactly once.
- Cancel, timeout, disconnect, and crash every session type; recovery handles
  must expose durable progress, safe next actions, warnings, resource holds, and
  finalization without logs.

**Engineering decisions**

- Every public long-running operation declares time/space complexity in named
  workload variables and emits counters in its result/session events.
- Ordinary inspections are bounded streaming walks. Reconstructive operations
  declare output breadth and required durable writes separately from inspection
  cost.
- Exact counter equality is required for forbidden behavior and deterministic
  protocol structure; implementation-sensitive cost uses the weakest
  sufficient bound that still catches breadth regressions.
- OS RSS, page-cache residency, allocator arenas, and mapped address space are
  environment-sensitive profile measurements. S.10 requires declared upper
  bounds and qualification metadata for them, never exact equality. Exact
  memory claims are limited to subsystem-owned buffers, frames, and allocations.
- The performance strength ladder is explicit: Foundational descriptive claim
  -> policy-admission receipt -> canonical comparison bundle -> Store execution
  counters -> Foundational counter-backed receipt -> deliberately materialized
  report -> certified/readmitted performance bundle. No step is inferred from a
  later-looking name.
- Sessions are framework-owned resources with durable identity, progress,
  cancellation, resume, abandonment, warnings, recovery, and finalization.

**Open questions**

- Hardware-specific latency thresholds are S.12 qualification inputs; S.10 still
  fixes structural counters, memory bounds, QoS behavior, and declared local
  profiles.

### Phase 19: Certification Closeout And S.11/S.12 Handoffs

Close S.10 only on independent real-media evidence and publish typed handoffs
that let S.11 strengthen security and S.12 broaden qualification without
redefining operational correctness.

**Relevant subsystems**

- `worth-store-certification`
- all S.10 production owners
- S.4.5 harness
- `worth-store-formal-models`
- S.11 security/compliance consumers
- S.12 certification/performance consumers

**Relevant APIs**

- S.10 certification evidence bundle
- operational recovery capability matrix
- authorization-provider readiness contract
- structured audit hardening handoff
- physical qualification profile handoff
- closeout verdict and denial report

**Warnings**

- A green unit/integration suite, one successful restore, or a passing formal
  model is not closeout. The evidence families must bind to each other and to
  ordinary production paths.
- S.11 handoff cannot claim provider authentication, proof-of-possession,
  encryption, tamper evidence, or secure deletion before S.11 implements them.
- S.12 handoff cannot hide missing S.10 correctness behind a future hardware
  matrix, soak, or performance program.
- Logs, markdown claims, same-run comparisons, fixture labels, and caller-built
  declarations are not accepted evidence sources.

**Test requirements**

- Run the generated matrix across backup/restore, PITR, offline verification,
  bad sectors, partial restore, damaged authority, damaged derived state,
  repair classes, rollback, bootstrap, promotion, forensics, and fresh-process
  reopen; every row binds production receipts, independent oracle evidence,
  exact counters, and applicable checked-model verdicts.
- Require the three Phase 16 milestone-spanning scenarios to pass their CI and
  release profiles, prove every phase appears in at least two transcripts, and
  prove each phase-local controlled defect is rejected at the owning boundary.
- Run wrong-tenant, wrong-key, stale-key, custody-missing,
  authenticity-unavailable, unsupported-secure-posture, cross-scope,
  stale-plan, replayed-authorization, revoked-authorization, and provider-
  unavailable denial lanes; no destructive owner call may occur.
- Corrupt, lose, duplicate, fork, and restore stale operational control state;
  partition the fencing authority; expire serve leases; advance current
  authority during staging; and reject a published root at readmission. Each
  lane must recover an explicit safe disposition or deny closeout, never infer
  success from target media or logs.
- Prove at least one controlled production defect per major workflow is caught
  by an independent lane and localized to the responsible boundary.
- Rebuild all evidence from a clean checkout and fresh media fixtures; bind
  source, binary, format, backend, configuration, model/tool identity, workload,
  scenario, schedule, and evidence digests.
- Compile the S.11 and S.12 handoff consumers against only public facades and
  prove they cannot mint current authority or reinterpret S.10 receipts.

**Engineering decisions**

- Certification is a courtroom that consumes production artifacts; it owns
  verdict construction but no operational source truth.
- The S.11 handoff exposes provider-neutral authorization requirements,
  structured audit records, custody/authenticity gaps, and security-scope
  propagation surfaces.
- The S.12 handoff exposes supported local profiles, structural complexity
  contracts, counter registries, hostile scenario matrix, residual risks, and
  unqualified backend/hardware dimensions.
- Closeout is denied if any destructive workflow lacks fresh-process crash
  proof, independent post-verification, typed authorization denial, bounded
  large-store evidence, or owner-specific execution receipts.
- Closeout is denied if the configured operational control store is unavailable
  or cannot establish one selected current generation, or if promotion cannot
  prove remote exclusion through the configured fencing domain. S.10 does not
  certify a degraded no-control-plane operating mode.
- Closeout includes a machine-readable Proof/Foundational adoption matrix. It
  proves every Store-to-shared lowering has a stronger source, legal category
  and role, explicit basis/freshness loss, canonical comparison contract,
  reverse-flow denial, and no duplicate private progression or boundary
  vocabulary.

**Open questions**

- None. Later milestones strengthen or broaden evidence; they do not weaken the
  closeout gate.

## Must Ship

- independent, read-only, bounded-memory real-media offline verification
- evidence-bound truth, degradation, rebuildability, quarantine,
  unrecoverable, unavailable, and indeterminate reporting
- physically independent, corruption-detecting operational control storage with
  atomic prefix recovery, selected-generation law, and no malicious-tamper claim
- stable-cut online backup with complete page/extent/index/WAL/blob reachability
  and reclaim protection
- distinct materialized, structurally verified, custody-qualified,
  production-restore-admissible, and restore-drill-certified backup states
- non-current backup restore and exact-frontier PITR with crash-safe staging,
  stable source leases, independent verification, fresh cutover authorization,
  fencing, explicit pending-publication disposition, and authority readmission
- retained-authority rollback as a complete source-leased, staged,
  post-verified, cutover-authorized operation
- provider-neutral, operation-specific authorization with replay, expiry,
  revocation, unsupported, and unavailable behavior
- split derived-maintenance versus authority-affecting repair topology,
  canonical owner-plan DAGs, owner-specific receipts, durable execution journals,
  resume/abandon, and post-repair verification
- replica bootstrap, disaster-recovery bundles, exact RPO/frontier reporting,
  source leases, materialized/independently-verified states, renewable serve
  leases, external old-primary fencing/epochs, promotion, and rejoin law
- observation-only forensic acquisition and canonical structured operational
  audit records
- S.4.5 production drivers/yieldpoints, checked protocol extensions, exact
  counters, controlled-defect sensitivity, and reproducible closeout evidence
- the burning-primary, split-brain reversal, and multi-owner repair milestone-
  spanning scenarios with topology-preserving smoke/CI/release scale profiles
- the Worth Proof/Foundational adoption matrix, strongly opinionated directory
  target, reverse-flow compile denials, and critical operator DX surfaces

## Must Preserve

- semantic authority remains outside physical layout and operational tooling
- observations, reports, digests, logs, model verdicts, and certification
  verdicts never become runtime mutation authority
- no destructive operation mutates current authority implicitly or in place
- no destructive resume, cutover, promotion, or readmission proceeds without one
  selected durable control-store generation
- security scope, tenant, key version, authenticity, custody, lineage,
  generation, frontier, and blast radius remain typed through every phase
- domain owners execute their own mutations and issue their own receipts;
  operations remains orchestration
- ordinary lanes never import replay-only certification authority or test
  constructors
- backup, restore, PITR, rollback, promotion, repair, and forensics
  remain distinct where source, cost, authority, or failure topology differs
- all large-media and reconstructive work remains bounded, inspectable,
  cancellable, resumable, and honestly named

## Acceptance Evidence

- a generated boundary ledger and clean constitution proving owner/dependency
  direction and absence of milestone-shaped or generic authority shortcuts
- compile-fail evidence that fails at the intended forbidden call and becomes
  green when only that call is removed
- fresh-process, real-file offline verification over media larger than memory,
  including resume, corruption, format, scope, custody, and authenticity lanes
- online backup under foreground and maintenance interference, independent
  bundle verification, and ordinary restore/reopen drill certification
- crash matrices at every durable transition for control-store recovery,
  restore, PITR, rollback, repair,
  publication/readmission, bootstrap/promotion, and forensic acquisition
- exact denial evidence for stale plans, wrong scopes, missing custody,
  unsupported capability, provider unavailability, authorization replay,
  revocation, divergent lineage, and damaged recovery intervals
- controlled-defect evidence demonstrating that independent verification,
  formal invariants, and certification oracles catch real production bugs
- exact operation-bound counters and declared complexity contracts for all
  ordinary and reconstructive paths
- reproducible evidence bundles tied to source, binary, format, backend,
  configuration, toolchain, scenario, schedule, and workload identity
- joined evidence from the three milestone-spanning scenarios proving all 19
  phases through ordinary production artifacts, independent oracles, controlled
  mutants, and topology-preserving scale profiles

## False-Completion Gates

S.10 is not complete if any of the following is true:

- the offline verifier consumes caller-constructed layouts, live runtime state,
  private caches, raw digest declarations, or whole-store materialization
- offline truth can span concurrently mutable media without a snapshot/clone/
  stable-generation/content-addressed consistency basis, or a corrupt checkpoint
  can skip ranges
- the verifier constructs recovery candidates or classification lacks canonical
  complete non-overlapping physical coverage with explicit alias/overlap states
- a backup can be called verified without independently opening actual bundle
  media, or restorable without an ordinary restore/reopen drill
- PITR execution is authorized by wall-clock time or a raw LSN
- restore, PITR, rollback, or promotion can overwrite current media in place
- operator identity/readiness, a role string, generic authority marker, or test
  authority opens a production destructive lane
- repair uses a generic action/executor, re-decides the plan during execution,
  treats rollback as a repair class, mutates authoritative current bytes in
  place, or allows damaged authority to be reconstructed from derived state
- operations directly mutates owner state or constructs owner execution receipts
- any workflow loses durable phase state, authorization consumption, owner
  receipts, or recovery disposition after process crash
- operational control state shares the protected media failure domain, permits
  ambiguous duplicate generations, or is treated as malicious-tamper-resistant
  before S.11
- publication/readmission can occur without independent post-verification and a
  fresh cutover plan/authorization, write fence, and authority-owner receipt
- a published root rejected by readmission lacks a durable pending/rejected/
  abandoned/forensic-retained disposition and explicit reclaim law
- promotion can occur without old-primary fencing, a fresh authority epoch,
  renewable serve-lease enforcement, external exclusion/epoch authority, exact
  acknowledged-frontier/RPO reporting, and divergent-history handling
- a forensic bundle can satisfy backup verification or restore admission
- audit delivery has no operation-local sequence/causal parentage, cannot be
  regenerated at least once from durable artifacts, or cannot prove completeness
  against expected journal transitions
- audit truth exists only in logs or S.10 claims tamper evidence/provider proof
  that belongs to S.11
- tests rely on logs, timing luck, same-run self-comparison, fixture labels,
  synthetic in-memory stores, caller-crafted declarations, or private mutation
- production receipts, manifests, reports, summaries, digests, or audit records
  define their own expected oracle truth
- exact RSS, page-cache residency, or OS mapping equality is claimed as a
  deterministic protocol counter instead of a bounded profile measurement
- Worth Proof is used as a dynamic workflow engine, generic owner outcome, or
  runtime evidence bag; or Worth Foundational replaces a stronger Store type,
  feeds execution/readmission, or collapses categories, roles, lineage, rows, or
  performance strength into a generic bundle
- the three milestone-spanning scenarios do not collectively exercise every
  phase through ordinary public facades at greater-than-memory scale, or their
  CI profiles omit actors, faults, oracle families, denials, or evidence present
  in release
- S.12 is named as the owner of missing S.10 correctness or crash proof

## Sequencing Notes

S.10 follows S.9 because the operational workflows must refine already-checked
durability, recovery, quarantine, publication, replication, and shared-frontier
law. If implementation exposes a missing owner outcome or model transition, the
scope expands backward and closes that foundation before the workflow proceeds.

S.10 precedes S.11 because the Store must first define exactly what an
authorization binds, which structured audit facts exist, and how tenant/key/
authenticity/custody scope propagates. S.11 then supplies production identity
providers, proof-of-possession, encryption and key lifecycle, tamper-evident
audit, and secure deletion without changing the operational protocol.

S.10 precedes S.12 because correctness, independence, boundedness, crash safety,
and hostile fault sensitivity belong to the implementation milestone. S.12
expands this evidence across declared hardware, backend, workload, soak,
performance, and hazard-analysis matrices.

The roadmap may proceed to S.11 only when the offline verifier can begin from
untrusted real media and produce admissible candidate plans, while every
destructive path still requires typed authorization, concrete owner receipts,
independent post-verification, atomic publication, and current-authority
readmission.
