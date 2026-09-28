# Glossary

> Exact meanings of the words WORTH uses. These definitions are normative. If
> a crate guide uses a term differently, the guide is wrong: report it.

Each entry has a definition. Where it helps, an entry also has **Not** (what
the term is commonly mistaken for) and **See** (where the mechanism is
explained). Rust names are given where a type embodies the term.

**Jump to:** [A](#a) · [B](#b) · [C](#c) · [D](#d) · [E](#e) · [F](#f) ·
[H](#h) · [I](#i) · [L](#l) · [M](#m) · [O](#o) · [P](#p) · [R](#r) ·
[S](#s) · [T](#t) · [W](#w)

---

## A

**Adoption**
: Moving one exact product branch from the program revision it runs to a
  target revision, in one product publication. You must compare first. Every
  custody item and every workflow on the branch needs a legal disposition.
  Adoption across a set of branches is not atomic: each branch publishes on
  its own, and progress can be resumed.
  **Not** an automatic upgrade when the host changes.
  **See** [How WORTH Works §12](how-it-works.md#12-branches-programs-and-adoption).

**Admission**
: The owner's check that a request or value may proceed *now*, against
  current facts. For a Query request it runs in order: select the branch,
  resolve the principal, resolve the scope, authorize, bind the source and
  idempotency. Admission is never carried between executions.
  **Not** performance: *admitted* does not mean *performed*.

**Aftermath**
: What can happen to a committed change afterward. Every operation declares a
  posture: *reversible*, *compensatable*, *reconcilable*, or *irreversible*.
  An escaping external effect cannot be reversible.
  **See** [How WORTH Works §14](how-it-works.md#14-aftermath-and-recovery).

**Artifact**
: A payload that carries its phase, proof set, and assumption basis in its
  type (`worth_proof::Artifact<P, T, S, A>`). A function that needs a phase
  accepts only an artifact in that phase. A bare generic artifact proves
  nothing. Owners wrap artifacts in private types to make them unforgeable.

**Aspect**
: In Foundational and Relational, a stable, named surface of meaning on an
  entity or relation (for example, an order's *status* fields), with a
  portable contract (`AspectKey`, `AspectContract`). In Signal, an *aspect*
  is a runtime-local slot inside one graph node. The two are unrelated. A
  Signal aspect is never persisted in a portable package.

**Assumption basis**
: A record of what a value assumed when it was established, with its
  freshness class in the type (`AssumptionBasis`, `FreshnessScopedBasis`).

**Audience facade**
: One of the public crates that a given kind of consumer may import.
  Application code imports `worth-query-decl` and `worth-query-host`; these
  two are also called the *Application API* or the *consumer facades*.
  Certification code imports `worth-query-replay`. Everything else is
  internal. **See** [API Map](api.md).

**Authority**
: The right to take a governed action. In WORTH, authority is a *value*
  minted by its owner. Holding the value is the permission, and it cannot be
  forged, copied into another lane, or assembled from parts.
  **Not** a report, digest, descriptor, token, or receipt clone.

## B

**Basis**
: The exact state an operation is based on: an owner-admitted observation of
  one branch at one generation. An *admitted basis* is authority to read or
  begin work from that observation. A *basis descriptor* is a serializable
  copy of it: data only, and weaker than the basis.

**Binding**
: (1) The declared link between an application's intent type and an
  installed operation or query (`ApplicationMutationBinding`,
  `ApplicationQueryBinding`). (2) In Proof, the recorded facts a capability
  was issued against (`binding_axes!`). A successful match is not a token.

**Branch**
: A named line of state. There are two kinds. A *component branch* belongs to
  one runtime: Relational or Signal. A *product branch* belongs to Runtime
  World and ties together an exact basis from each component.
  **See** *Product branch*.

**Bridge (Runtime Bridge)**
: The runtime that owns correspondence and causal routing from committed
  truth changes to exact Signal targets. It translates between truth and
  computation without giving either side the other's authority.

## C

**Candidate**
: A proposed change that has been built and validated but not committed. In
  Query, a handler builds it through `CandidateWriter`. In Relational, a
  prepared candidate is opaque, single-use, and branch-bound. Preparing a
  candidate moves nothing.

**Canonical basis**
: An ordered, versioned, domain-tagged sequence of entries from which
  identity is derived (`CanonicalBasisSequence`). Two parties that build the
  same canonical basis mean the same thing.

**Capability**
: A declared, installed permission to perform a class of actions, such as
  starting a workflow or approving a step. Capabilities are granted to
  principals and checked at admission.

**Commit**
: The atomic landing of a validated candidate on a branch. A commit moves the
  branch reference once and produces a receipt.
  **Not** settlement (see *Settled*), and **not** external completion.

**Compare-and-publish**
: Relational's linearization point. The candidate's full expected
  observation is compared with the branch's current reference. If they match,
  the next root is installed and the generation advances exactly once.
  Otherwise the result is `Stale`.

**Condition**
: A pure predicate over declared inputs. In a workflow, a condition node
  decides a branch in the graph. It is never an effect.

**Correspondence**
: The installed mapping from a portable truth dependency to exact Signal
  targets (graph, node, partition, aspect), with `Exact` or
  `DeclaredWidening` precision. A committed change reaches a Signal target
  only through a matching correspondence.

**Currentness**
: Whether a basis still matches the owner's live state. Currentness is part
  of authority: stronger operations accept only a current basis.
  **Not** residency: a retained basis can be available and not current.

**Custody**
: Responsibility, held by the runtime, for work that is still owed: a pending
  effect recovery, a source reservation, or an external workflow operation
  whose settlement is outstanding. A workflow operation in owner custody must
  be settled or recovered before its instance can be canceled, migrated, or
  continued on a fork. During program adoption, the host derives a
  disposition for each in-flight custody item; you do not choose it, and an
  item it cannot carry refuses the adoption.

## D

**Declaration**
: The application's typed description of its schema, operations, queries,
  capabilities, workflows, and aftermath. A declaration does nothing on its
  own. It cannot install, authenticate, authorize, execute, or publish.

**Declared ceiling**
: The most an operation may change, as declared (`WorthQueryOperationTouchContract`).
  **Not** a record of what changed. **See** *Touched records*.

**Demand (output demand)**
: A request that a declared program output be produced. It is advanced by
  fresh requests until it settles.

**Denial**
: A typed refusal. A refusal *before any effect* is the `Err` side of
  `execute()` and does not consume the idempotency key. Once an attempt runs,
  its outcome falls into one of four families: *landed*, *domain*, *stopped*,
  or *not a clean landing*. Each calls for a different response.
  **See** [How WORTH Works §11](how-it-works.md#11-outcomes-every-way-a-request-can-end).

**Descriptor**
: A serializable, portable description of an owner value, such as a basis or
  a fork source. Copying it weakens freshness. A descriptor carries no
  authority and cannot act by itself.

**Digest**
: A SHA-256 identity derived from a canonical basis through Foundational
  (`CanonicalDigestId`). A digest is an address, never permission.

**Domain denial**
: A refusal decided by application logic, such as "the order is already
  approved". It is returned by a handler as `DomainDenied`.

## E

**Evidence**
: A value that records that something happened or was observed. Evidence
  comes in strengths, for example planning receipt, then executed receipt,
  then completed receipt. A weaker claim never passes as a stronger one.
  Evidence is not authority.

**External effect**
: A consequence outside WORTH, such as sending an email or calling a payment
  provider. A commit writes a dispatch outbox record atomically. The effect's
  posture (`NotDeclared`, `PendingDispatch`, `Acknowledged`, `Completed`,
  `Unresolved`) is tracked separately from the commit.

## F

**Footprint**
: The recorded set of entities, field revisions, and adjacency revisions that
  an observed source read. At admission, an edit outside the footprint does
  not invalidate the source.

**Freshness**
: How current a basis is, recorded in its type: `CurrentValidity`,
  `StaleReadable`, `RebindRequired`, or `AuthorityRevalidationRequired`.
  Crossing a trust boundary weakens freshness. Only explicit readmission
  restores it.

## H

**Handler**
: Application code that implements an operation (`OperationHandler`): it
  decides, states its resource requirements, and builds a candidate. Workflow
  control steps have no handler.

## I

**Idempotency key**
: A key that binds a mutation's intent (input identity, plus source and
  workflow transition when present). Reusing a key with the same intent
  replays the recorded outcome without running the handler. Reusing it with a
  different intent returns `IdempotencyIntentDrift`.

**Indeterminate**
: A commit outcome that means the landing is *unresolved*. The outcome says
  which recovery is required. **Not** failure.

**Installation**
: Compiling a declaration into an executable runtime: contracts compiled,
  graph obligations derived, the package admitted, resources bounded. A
  missing handler fails installation.

**Invariant**
: A rule that must hold over committed state. An installed invariant is
  checked on the candidate before commit.

## L

**Lineage**
: (1) A continuity claim about evidence (attested, replay-derived, restored,
  promoted, or partial). (2) For workflows, the chain of revisions of one
  definition, or of one instance and its successors.

**Linear resource**
: A resource that exists once and ends once (`LinearResource`). A second
  termination does not compile.

**Live read**
: A bounded subscription to a query's results. Each delivery re-resolves the
  principal and scope against a fresh request. An overflow reports how many
  commit batches were missed.

## M

**Marker (authority marker)**
: A type that identifies an authority lane. It is sealed when its
  constructor is private, which `authority_marker!` generates. Surrendering a
  marker value mints a *witness*.

## O

**Observation**
: An exact reading of a branch reference: branch id, target, and generation
  (`FoundationalBranchReferenceObservation`). An observation describes. It
  does not act.

**Observed source**
: An opaque record, taken from a published query row, of what a later
  mutation was based on (`WorthQueryObservedSource`). It is descriptive input,
  checked at admission. It is not read authority.

**Owner**
: The single runtime that holds a kind of authority and alone may mint it.
  Relational owns committed truth, Signal owns derived computation, the Bridge
  owns correspondence, Runtime World owns product branches, and Query owns
  application meaning.

## P

**Performed**
: The action ran. For a commit, the branch reference moved. Performed evidence
  (`worth_proof::Performed`) is separate from any admission.
  **Not** settled.

**Phase**
: A stage in a typed progression, carried in the type so that skipping or
  reordering steps does not compile.

**Placement rule**
: Where new vocabulary lives. First *yes* wins: legality goes in
  `worth-proof`; the same meaning across a boundary goes in
  `worth-foundational`; anything needing a clock, counter, live table, or
  `Drop` goes in the owning runtime.
  **See** [How WORTH Works §5](how-it-works.md#5-the-placement-rule).

**Principal**
: The authenticated actor a request acts for
  (`WorthQueryAuthenticatedExternalPrincipal`). The principal is resolved at
  admission, and again on each live delivery.

**Product branch**
: One live occurrence of an application's world, owned by Runtime World. It
  selects an exact basis from each component runtime. In Query, a
  `WorthQueryProductBranch` is a `Copy` token that names the branch and
  grants nothing.

**Program**
: A revision of an application's authored meaning
  (`ApplicationProgramRevision`). A program has three separate identities: the
  *revision* (what it means), *support* (a host can run it), and *activation*
  (a branch runs it). A mutation never names its program. The branch decides.

**Proof**
: Type-level evidence that a fact holds, minted only by an authority that
  proves it (`worth_proof::Proof<P, A>`). It has no runtime cost.

## R

**Readmission**
: Restoring a weakened basis to current by presenting the owner's authority
  again. Readmission is never ambient.

**Receipt**
: The record a commit returns (`WorthQueryApplicationCommitReceipt`): which
  branch and commit, what changed, which records were touched, what must be
  dispatched, and under what authority. A cloned receipt is descriptive
  history.

**Recovery**
: The explicit continuation of a partial or unresolved outcome, such as a
  deferred settlement, an unpublished product, or an indeterminate commit.
  Recovery continues performed effects. It never rolls them back.

**Reporting**
: Any output that describes rather than authorizes: readiness reports, basis
  tokens, discovery answers, diagnostics, receipt clones. Reporting never
  grants anything.

**Residency (retention)**
: Keeping a basis available in memory under a lease. A resident basis is not
  necessarily current.

## S

**Scope**
: The entity a request acts within (`WorthQueryRequestScope`), resolved at
  admission and used in authorization.

**Settled**
: Performed, and also made durable and published by Query. A commit that
  moved the branch but did not finish settlement is `SettlementDeferred`. The
  fix is to recover settlement, not to redo the change. Today "durable"
  means the owners' settlement records are complete in memory. Surviving a
  process restart arrives when Store is wired in
  ([How WORTH Works §15.1](how-it-works.md#151-store-durable-physical-survival)).

**Signal**
: The runtime for deterministic, incremental derived computation. Signal
  decides whether an output changed meaningfully. It is never truth.

**Stale**
: The basis an operation relied on is no longer current. In Relational, any
  movement of the branch after your observation makes a candidate stale.
  Re-read and retry. The runtime never retries or rebases for you.

**State**
: A materialized value that can be computed, cached, or displayed. State may
  be derived from truth. It is never authority over truth.

## T

**Touched records**
: The records a commit actually changed, sealed by the commit itself
  (`WorthQueryTouchedRecordIdentity`, read with
  `receipt.mutation_work().touched_records()`). They are never supplied by a
  caller. Undo depends on them.
  **See** [How WORTH Works §10](how-it-works.md#10-the-touched-graph).

**Truth**
: Committed, owner-held facts: in WORTH, the entities, relations, and
  aspects Relational commits. Truth
  changes only through a governed commit.
  **Not** derived state, cached projections, or reports.

## W

**Witness**
: A zero-sized value proving the caller is in an authorized lane
  (`AuthorityWitness`). A witness proves the lane, not which runtime
  instance.

**Workflow**
: A branch-local, versioned graph of steps (operations, assessments,
  conditions, approvals, evidence joins, terminals), authored against an
  installed vocabulary (*spec*). A *definition* is one published revision. An
  *instance* is one run of a pinned definition revision. *Control steps*
  (publish or retire a definition; start, cancel, migrate, or continue an
  instance on a fork; advance or navigate back; approve) are governed
  mutations whose bindings set `WORKFLOW_CONTROL`. Query's workflow kernel
  authorizes and records them, never a handler. Applications reach the kernel
  through the workflow runtime (`WorthQueryWorkflowApplicationRuntime`).
  Progress is caller-pumped.
  **Not** `worth_query::facade::workflow` (preview, promotion, writeback, and
  branch merge) or `worth_query::facade::installed::workflow` (staged runs of
  installed domain operations).
  **See** [How WORTH Works §13](how-it-works.md#13-workflows).

**World (Runtime World)**
: The runtime that composes exact component bases into product branches and
  publishes across component owners in a fixed order. Its outcomes are
  `Performed`, `NoEffect`, or `ProductUnpublished`.
