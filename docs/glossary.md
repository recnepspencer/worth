# Glossary

> Exact meanings of the words WORTH uses. These definitions are normative. If
> a crate guide uses a term differently, the guide is wrong: report it.

Each entry has a definition. Where it helps, an entry also has **Not** (what
the term is commonly mistaken for) and **See** (where the mechanism is
explained). Rust names are given where a type embodies the term.

**Jump to:** [A](#a) · [B](#b) · [C](#c) · [D](#d) · [E](#e) · [F](#f) ·
[G](#g) · [H](#h) · [I](#i) · [L](#l) · [M](#m) · [O](#o) · [P](#p) · [R](#r) ·
[S](#s) · [T](#t) · [V](#v) · [W](#w)

---

## A

**Action**
: Something a feature owns in a program, recorded as an
  `ApplicationActionDeclaration`: a mutation binding, a declared operation, or
  a conditional operation, attached with `ApplicationFeatureSpec`
  (`mutation::<Binding>()`, `operation::<Op>()`,
  `conditional_operation::<Op>()`). The installed program lists its actions
  (`installed_program().actions()`). **Not** the same as an *Operation*: an
  action is program ownership of an operation.
  **See** *Operation* and [Build an Application §3.2](build-an-application.md#32-attach-actions-to-features).

**Admission**
: The owner's check that a request or value may proceed *now*, against
  current facts. For a Query request it runs in order: select the branch,
  resolve the principal, resolve the scope, authorize, bind the source and
  idempotency. Admission is never carried between executions.
  **Not** performance: *admitted* does not mean *performed*.

**Adoption**
: Moving one exact product branch from the program revision it runs to a
  target revision, in one product publication. You must compare first. Every
  custody item and every workflow on the branch needs a legal disposition.
  Adoption across a set of branches is not atomic: each branch publishes on
  its own, and progress can be resumed.
  **Not** an automatic upgrade when the host changes.
  **See** [Build an Application §7](build-an-application.md#7-adopt-a-new-program-on-a-branch)
  and [How WORTH Works §12](how-it-works.md#12-branches-programs-and-adoption).

**Advancement**
: One executing public entry under one bounded request: a query, mutation,
  live delivery, adoption step, or demand `advance`. Internal phases borrow
  its custody; descendant work shares its limits. The callback closes the
  request. Pump output progress with `advance`; workers compute but do not
  publish. **See** [execution and cost](how-it-works.md#99-resources-execution-and-cost).

**Aftermath**
: What can happen to a committed change afterward. Every operation declares a
  posture: *reversible*, *compensatable*, *reconcilable*, or *irreversible*.
  An escaping external effect cannot be reversible.
  **See** [How WORTH Works §14](how-it-works.md#14-aftermath-and-recovery).

**Approval**
: In a workflow, an `Approval` node (`approval::<Capability>(id)`) waits for a
  decision about a *proposal*. The decision is a control step
  (`prepare_workflow_approval` with `WorkflowApprovalDecision::Approve` or
  `Reject`), authorized by the approval's own capability and signed with a
  fresh *signature*. The recorded decision (`PerformedWorkflowApproval`, with
  `approver()`) is evidence, not permission: it never satisfies a guarded
  operation's ordinary operation authorization, and a guarded operation needs
  one only when its binding's `workflow_requirements` or an `approval_authority`
  edge asks for it.
  **Not** an *elevation approval*, a step in the emergency-elevation
  capability lifecycle (`authorize_elevation_approval`), where approving is
  also not using.
  **See** [How WORTH Works §13.1](how-it-works.md#131-who-may-run-a-guarded-operation)
  and [Workflows, Approvals](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#approvals).

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
  The word has three narrower uses; keep them apart. (1) *Workflow occurrence
  authority*: the single-use binding an advance issues for one guarded
  operation occurrence; it is not permission. (2) *Approval authority*
  (`approval_authority`): a definition edge that routes an approval node's
  decision to an Operation node as required evidence; it grants nothing.
  (3) *Execution authority*: the physical owner of computation workers and the
  charged-memory ledger, unrelated to permission.
  **See** *Workflow occurrence authority* and *Execution authority*.

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
  principals and checked at admission. For a workflow approval, the capability
  is also the type parameter that names what the approval means
  (`approval::<Capability, Operation, Input>()` when installing the vocabulary,
  `approval::<Capability>(id)` when authoring), and the *signature*'s signing
  intent includes it. Declare one capability per signature meaning.

**Commit**
: The atomic landing of a validated candidate on a branch. A commit moves the
  branch reference once and produces a receipt.
  **Not** settlement (see *Settled*), and **not** external completion.

**Clean reuse**
: Keeping a settled output without rerunning its producer or source query
  when its selected lineage is continuous, its settlement is unmarked, and
  no recorded full-verification requirement applies. This decision uses
  marking, not a producer opt-in contract. It is distinct from *cutoff* and
  *partition reuse*.

**Compare-and-publish**
: Relational's linearization point. The candidate's full expected
  observation is compared with the branch's current reference. If they match,
  the next root is installed and the generation advances exactly once.
  Otherwise the result is `Stale`.

**Condition**
: A pure predicate over declared inputs. In a workflow, a condition node
  decides a branch in the graph. It is never an effect.

**Conflict group**
: In Signal, one unit of ready graph work that may run beside the other
  units of its batch because their worker-local effects are proven not to
  overlap. The planner lowers a stage into ordered groups
  (`DisjointApplyGroup`, one per task). Each batch (`DisjointGraphBatch`)
  admits a prefix of the stage's remaining ready tasks and is created only
  by the invalidation progression owner, never from the planner's groups;
  the owner derives every member node's full set of proposal surfaces
  (state, dependencies, produced aspects, subscriptions, snapshot, lineage,
  observation, diagnostic) and refuses the batch when two members overlap.
  The plan reduces group results in stage task order.
  **Not** an execution partition: a conflict group proves that worker-local
  proposal surfaces do not collide, not that final graph writes are
  disjoint, and carries no data identity.
  **See** *Partition (execution)*.

**Contribution**
: One named slice of an application schema, plus the host setup it needs:
  handlers, invariants, and conditional nodes
  (`worth_query_application_contribution!`, `WorthQueryApplicationContribution`).
  A program lists the contributions it installs.
  **See** [Build an Application §2](build-an-application.md#2-declare-the-schema-and-its-contributions).

**Correspondence**
: The installed mapping from a portable truth dependency to exact Signal
  targets (graph, node, partition, aspect), with `Exact` or
  `DeclaredWidening` precision. A committed change reaches a Signal target
  only through a matching correspondence.

**Currentness**
: Whether a basis still matches the owner's live state. Currentness is part
  of authority: stronger operations accept only a current basis. In Proof,
  an execution-ready recipe exposes its strong basis only when that basis
  carries `CurrentValidity`.
  **Not** residency: a retained basis can be available and not current.

**Cutoff**
: Ending propagation at a proved equality. Input cutoff can skip a producer
  only under its declared canonical-bitwise input-reuse contract and context
  requirements. Stable output republication can resolve downstream pending
  edges through the exact equal, clean successor. Equality never waives the
  attempt's final consumed-fact checks. **See** *Clean reuse* and *Marking*.

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

**Decomposition**
: An execution-layer pattern that solves partition interiors, reduces their
  interface contributions, solves the interface, and substitutes its solution
  back into the interiors. Each stage has a distinct failure and cost boundary.

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

**Determinism contract**
: The declared comparison rule under which an execution result is independent
  of placement. The default is canonical bitwise equality; an installed
  equivalence contract supplies a predicate at the execution authority.
  Without an in-flight interruption and with sufficient memory, canonical
  settlement preserves the result, charged work and structural span across
  admitted widths. Physical metrics can differ. Managed partition retention
  requires canonical bitwise equality.

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

**Execution authority**
: The physical process owner of computation workers and the charged-memory
  ledger. It admits execution policy and issues request leases. Only one
  physical authority may be constructed during a process lifetime. A serial
  backing owns no physical pool; its bounded lease still charges and reports
  work. Install policy at the host, not inside a computation.

**External effect**
: A consequence outside WORTH, such as sending an email or calling a payment
  provider. A commit writes a dispatch outbox record atomically. The effect's
  posture (`NotDeclared`, `PendingDispatch`, `Acknowledged`, `Completed`,
  `Unresolved`) is tracked separately from the commit.

## F

**Feature**
: A named unit of application meaning inside a program (`ApplicationFeature`).
  A feature owns actions (mutations and operations), declared with an
  `ApplicationFeatureSpec`, and may provide or require typed ports that
  connections bind. A connection between ports whose values differ does not
  compile. On a runtime installed with a program, every mutation runs
  through a program lane (`execute_in_program` and its capability,
  retained, and performed variants), never through plain `execute()`.
  **Not** a runtime switch or flag.
  **See** [Build an Application §3](build-an-application.md#3-declare-features-and-the-program).

**Footprint**
: The recorded set of entities, field revisions, and adjacency revisions that
  an observed source read, carried inside the opaque
  `WorthQueryObservedSource`. At admission, the source is checked against
  the facts built from its footprint and no others, so an edit outside the
  footprint does not invalidate the source.

**Frontier (staged execution)**
: Two or more ready stages of one installed staged operation, ordered by
  unique stage identity.
  Preparation and application run on the owner; pure task computation can
  run on workers. Frontier entry sorts the supplied identities and refuses
  fewer than two members or duplicates. Ordinary single-stage computation
  uses the underlying checked map. This engine surface is distinct from
  the application's workflow kernel and its caller-pumped control mutations.

**Freshness**
: How current a basis is, recorded in its type: `CurrentValidity`,
  `StaleReadable`, `RebindRequired`, or `AuthorityRevalidationRequired`.
  Crossing a trust boundary weakens freshness. Only explicit readmission
  restores it.

## G

**Guarded operation**
: An installed mutation whose binding declares `workflow_authority required`
  (`REQUIRES_WORKFLOW_AUTHORITY`). It commits only inside the workflow
  occurrence that awaits it: bind the request with `for_workflow_operation`
  after an advance returns `AwaitingOperation`. Without that binding the
  request is refused before any effect (`RequiresWorkflowTransition`);
  authority already consumed is `WorkflowAuthoritySpent`. It still needs
  ordinary operation authorization,
  and needs an *approval* only as its `workflow_requirements` or the
  definition demand. Guarded bindings must explicitly declare the installed
  requirement; `ApplicationWorkflowExecutionRequirements::occurrence_only()`
  adds no approval minimum. Omission is a compile refusal.
  **See** [How WORTH Works §13.1](how-it-works.md#131-who-may-run-a-guarded-operation).

## H

**Handler**
: Application code that implements an operation (`OperationHandler`): it
  prepares a decision plan, states its resource requirements, and builds a
  candidate. The default `prepare_decision` calls `decide`; a computing handler
  overrides preparation and completes on the owner after compute. Workflow
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

**Island**
: At the execution layer, a connected component of items under a declared
  coupling relation. A component partitioner groups it into a partition and
  uses its least item identity as the component identity.

## L

**Lease**
: A runtime hold issued by an owner. An execution lease carries worker,
  charged-memory, and work ceilings from admitted policy; descendants can
  narrow them and share ancestor accounting; physical placement also shares
  the process ledger. A serial execution lease has one worker, no pool and
  the same checked execution doors. A residency lease
  instead keeps a selected basis available. Neither grants mutation authority.
  A residency lease (`RelationalBranchRetentionLease`,
  `SignalBranchRetentionLease`) is an owner-issued obligation that is
  released or dropped exactly once and exposes no read or mutation
  capability.
  A Query advancement uses the same bounded request throughout Bridge,
  Relational, Signal, and World execution. Serial placement supplies serial
  backing and one worker; nested computation still shares its parent's bounds.

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

**Marking**
: How a commit invalidates derived outputs. The commit's *touched graph* is
  intersected with the *reverse index*; each matched settlement is marked
  dirty, and every settlement that consumed its output is marked
  pending-upstream. Marks are per branch lineage. An unmarked settlement on
  a continuous basis, without a full-verification requirement, can certify
  currentness without re-running its source query.
  A settlement recorded before a delivery discontinuity, or one that carries
  a recorded verification requirement, requires full
  verification. Marking that exceeds its installed ceiling or its retained
  capacity becomes such a discontinuity instead of refusing the writer.
  **See** [How WORTH Works §10.5](how-it-works.md#105-marking-and-currentness).

## O

**Observation**
: An exact reading of a branch reference: branch id, target, and generation
  (`FoundationalBranchReferenceObservation`). An observation describes. It
  does not act.

**Observed source**
: An opaque record, taken from a published query row, of what a later
  mutation was based on (`WorthQueryObservedSource`). It is descriptive input,
  checked at admission. It is not read authority.

**Occurrence**
: One concrete instance of something otherwise named in general. The word has
  several senses; the context names which. (1) *Product occurrence*: one live
  occurrence of the application's world, selected on a product branch
  (`ProgramNotActiveOnOccurrence`). (2) *Workflow occurrence*: the exact
  operation, input, and branch a workflow instance is waiting for now; see
  *Workflow occurrence authority*. (3) *Inbound occurrence*: one authenticated
  inbound message received through an installed inbound source
  (`inbound_occurrences().receive(..)`); its decoded claims are data.
  (4) *Component occurrence*: one named expansion of a reusable workflow
  component (`expand_component(occurrence, &component)`), which prefixes the
  expanded node paths. (5) *Producer occurrence*: one producer run admitted in
  a *required wave*.

**Operation**
: (1) *Installed operation*: a business operation declared in the schema
  (`worth_query_operation!`, `ApplicationOperationMarkerIdentity`). A mutation
  binding names it (`type Operation`); a receipt reports it
  (`installed_operation()`). (2) *Operation node*: a workflow node
  (`Operation { operation, requires_workflow_authority }`) that refers to an
  installed operation. It is a proposal slot when the binding is unguarded and
  a *guarded operation* when it requires workflow authority. (3) *Control
  operation*: the schema-declared operation a workflow control capability
  gates (the `Operation` in `approval::<Capability, Operation, Input>()` and
  the `*_capability` calls); the kernel records it and no handler serves it.
  **See** *Action* and *Guarded operation*.

**Owner**
: The single runtime that holds a kind of authority and alone may mint it.
  Relational owns committed truth, Signal owns derived computation, the Bridge
  owns correspondence, Runtime World owns product branches, and Query owns
  application meaning.

## P

**Partition (execution)**
: A data-identified unit dispatched to one kernel and settled in canonical
  order. Its identity is a function of the canonical partition set, not the
  lease, worker count or machine. Define membership before choosing placement.
  **Not** a Signal *conflict group* (`DisjointApplyGroup`): a conflict group
  is ready graph work admitted because its worker-local proposal surfaces do
  not overlap another group's, and it carries no data identity.
  **Not** a *whole-partition observation scope*.
  **See** *Conflict group*, *Partitioner* and *Scope path*.

**Partition reuse**
: A producer's next managed computation carries unchanged owner calls and
  partition results from its published prior record. Installation, producer
  edition and input digest must match; recorded facts decide which calls
  change. Carried accepted charges preserve full-run-equivalent work and
  span. Memory reserves what is physically held. Missing usable prior state
  takes the full-run path. Declare stable items, partition keys and pure
  owner functions; do not select placement in them.
  **See** [the computation owner](build-an-application.md#36-declare-and-install-a-partitioned-managed-computation).

**Partitioner**
: The owner of the rule assigning stable item identities to execution
  partitions. An application computation plan offers keyed grouping. The
  execution layer also offers a component partitioner for connected items
  and a bisection partitioner (`Bisection`), which recursively bisects a
  weighted item graph under a maximum leaf weight, identifies each leaf by
  its root-to-leaf cut path, and re-cuts only an overloaded leaf or an
  ancestor outside tolerance.
  Worker placement does not define partition membership.

**Performed**
: The action ran. For a commit, the branch reference moved. Performed evidence
  (`worth_proof::Performed`) is separate from any admission.
  **Not** settled.

**Phase**
: A stage in a typed progression, carried in the type so that skipping or
  reordering steps does not compile.

**Pinned read**
: Access to one retained immutable source root selected by an exact owner
  observation. A later commit does not change that read. Its hold preserves
  availability, not live currentness or write authority. Releasing the hold
  ends access. **See** *Currentness* and *Residency (retention)*.

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
  Declared as an `ApplicationProgramDefinition` (contributions, features,
  output graph, rules) and validated into a `ValidatedApplicationProgram`.
  **See** [Build an Application §3](build-an-application.md#3-declare-features-and-the-program).

**Proof**
: Type-level evidence that a fact holds, minted only by an authority that
  proves it (`worth_proof::Proof<P, A>`). It has no runtime cost.

**Proposal**
: The record that a workflow instance has proposed input at an unguarded
  Operation node, published with `prepare_workflow_proposal`
  (`PublishedWorkflowProposalRef`). It records the operation, the input type,
  an input-identity digest, the source identity when present, and the node
  path, plus the subjects it covers. It does not retain the input value, runs
  no handler, and writes no business state. Assessments and conditions read
  the current state of a covered subject, never the proposed value. An
  *approval* decides on a proposal, and a guarded
  operation's input must hash to the proposal's digest.
  **See** [How WORTH Works §13.2](how-it-works.md#132-propose-authorize-execute).

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

**Required set**
: The outputs still owed: those with an open demand, and those a performed
  operation requires (`start_required_outputs`). An `advance` progresses the
  caller's dependency chain and queued required work within its budget.
  Membership can have several holders; closing one demand does not remove
  an output still required by another.

**Required wave**
: Required producer occurrences admitted together by the required-set owner
  against one selected basis. Each member's consumed upstream outputs are
  already committed on that basis. Independent compute can run concurrently;
  completion, commits, and publications follow canonical member keys. Every
  wave of an advancement shares its request. **See** *Advancement*.

**Residency (retention)**
: Keeping a basis available in memory under a lease. A resident basis is not
  necessarily current: the lease holds the root its observation selected,
  and a root the branch has retired is reclaimed only once no lease
  reserves it.

**Reverse index**
: Query's index from consumed facts (field revisions, index keys, selection
  and absence facts, and also entity lifecycle, relation membership, and
  adjacency facts) to the settlements that read them. It is filled when a
  settlement is recorded and consulted by *marking*, so marking selects matched
  settlements and their downstream closure without scanning every settlement.

## S

**Scope**
: The entity a request acts on. A mutation intent names it through its
  binding's scope field (`scope_field()`, read from the intent by
  `scope_binding()`); a query carries its own scope.
  It is resolved at admission and used in authorization.
  **Not** `WorthQueryRequestScope`, which carries a request's optional deadline
  and cancellation token; an absent deadline is unbounded, not elapsed.
  **See** [Build an Application §5](build-an-application.md#5-run-the-program).

**Scope path**
: A hierarchical address that refines an affected region. Signal subscriptions
  can select an exact path or its subtree. **See** *Whole-partition observation scope*.
  A path has one to eight non-empty segments (`ScopePath::MAX_DEPTH`); a
  longer path is refused. The Bridge lowers each change it delivers into
  changed regions on these paths. A partition-local dependency yields the
  change's own scope or the target partition's subtree. A record-local
  dependency yields the change's own scope or the exact path of the target
  partition and the changed record. A whole-graph dependency, or a
  correspondence with `DeclaredWidening` precision, yields no region.

**Settled**
: Performed, and also made durable and published by Query. A commit that
  moved the branch but did not finish settlement is `SettlementDeferred`. The
  fix is to recover settlement, not to redo the change. Query's in-memory
  settlement records do not themselves guarantee survival of a process
  restart; that requires Store's durable ownership
  ([How WORTH Works §15.1](how-it-works.md#151-store-durable-physical-survival)).

**Settlement**
: The word is used at several levels; the context names which. (1) *Commit
  settlement*: finishing durability and Query publication after the branch
  moved (see *Settled*; `SettlementDeferred`,
  `recover_deferred_application_settlement`). (2) *Output settlement*: the
  recorded result of a program output's producer, which *marking* marks dirty
  or pending and which a demand reports as `Settled`. (3) *Workflow node
  settlement*: the record that a node took its control outcome. A guarded
  operation's business write and its node settlement publish in the same
  candidate; navigating back publishes one Back settlement. (4) *Owner
  settlement*: an external effect's owner accepting its result, which ends
  *custody*.

**Shard**
: A placement unit for storage or execution. Its placement does not determine
  which facts changed or which consumers require recomputation: the *touched
  graph* names what changed, and the *reverse index* names the settlements
  that consumed it.

**Signal**
: The runtime for deterministic, incremental derived computation. Signal
  decides whether an output changed meaningfully. It is never truth.

**Signature**
: The signed form of a workflow *approval* decision: a fresh authentication
  event (`WorthQueryAuthenticationEvent`) that the installed
  authentication-event owner issues to the approving principal for one exact
  signing intent. Each signing consumes one use; the owner's policy bounds the
  event's age and its uses (`SingleUse` or `AtMost(n)`). The signing intent
  (`WorthQueryAuthenticationEventIntent`, purpose `workflow-approval-signature`)
  digests the branch, instance, definition and its content identity, program
  revision, approval node path, approval capability, target operation, the
  proposal (identity, operation, input type, input identity), and the
  decision. A separate subject-coverage digest covers the instance, the
  proposal, and the proposal's source when present. A replayed key returns
  the recorded decision without a new event.
  **Not** a package release signature or an inbound-message signature, which
  are verified by their own owners and authorize no approval.
  **See** *Capability* and [Workflows, Approvals](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#approvals).

**Span**
: An execution-layer cost measuring the structural length of charged dependent
  computation. Sequential stages add span; independent branches take their
  maximum; nested work adds to its calling branch. Span is not wall-clock
  duration. **See** [work and span](how-it-works.md#99-resources-execution-and-cost).

**Stale**
: The basis an operation relied on is no longer current. In Relational, any
  movement of the branch after your observation makes a candidate stale.
  Re-read and retry. The runtime never retries or rebases for you.

**State**
: A materialized value that can be computed, cached, or displayed. State may
  be derived from truth. It is never authority over truth.

## T

**Touched graph**
: The exact changes sealed by a commit: changed records, aspect field paths,
  adjacency changes, observable revision bumps, and old/new index membership.
  It supplies both performed evidence and the cause of invalidation. The
  facts each settlement consumed, recorded in the *reverse index* when the
  settlement is registered, determine which consumers intersect those
  changes. A declared dependency does not: marking matches recorded facts
  only. A commit sealed without an exact graph (`Unavailable`) cannot assert
  that nothing changed. It is delivered as a discontinuity, and a
  settlement recorded before it requires full verification.
  Coarser precision is declared on the Bridge *correspondence*, never
  inferred from the touched graph: a correspondence admitted with
  `DeclaredWidening` precision is counted as a widened match and delivers
  its changes to Signal with no narrowing region.
  **See** [How WORTH Works §10](how-it-works.md#10-the-touched-graph).

**Touched records**
: The record layer of the *touched graph* exposed on a receipt
  (`WorthQueryTouchedRecordIdentity`). `receipt.mutation_work()` returns an
  `Option`; read the records from the evidence it holds, as in
  `receipt.mutation_work().map(|work| work.touched_records())`. Sealed by the
  commit and never supplied by a caller. Undo depends on them.
  **See** [How WORTH Works §10.3](how-it-works.md#103-layer-3-commit-sealed-records).

**Truth**
: Committed, owner-held facts: in WORTH, the entities, relations, and
  aspects Relational commits. Truth
  changes only through a governed commit.
  **Not** derived state, cached projections, or reports.

## V

**Visited (Signal observation)**
: Nodes considered during committed transaction processing. The `Visited`
  observation tier includes consideration that produces no recomputation or
  value change. `Recomputed` observes evaluation; `MeaningfulChange` observes
  the committed change selected by the node's comparison contract.

## W

**Whole-partition observation scope**
: A Signal subscription selecting the entire subtree under one partition
  segment of a scope path (`PartitionSubscription::whole_partition`). It
  decides which changed regions reach an observer. It does not dispatch a
  kernel or prove worker-local disjointness. Choose it for whole-subtree
  observation, not to request an execution width. **See** *Partition (execution)*
  and *Conflict group*.

**Witness**
: A zero-sized value proving the caller is in an authorized lane
  (`AuthorityWitness`). A witness proves the lane, not which runtime
  instance.

**Work**
: The sum of charged operation units, an amount in the execution report,
  not elapsed time or memory. An execution map reports its accepted canonical
  prefix and records discarded suffix work as a physical metric. In a
  producer wave, member denials are data: every member that computed remains
  charged when owner completion keeps only a prefix. Owner work is charged
  where it runs; undispatched waves cost nothing. Interpret work together
  with *span* and the report's physical metrics.

**Workflow**
: A branch-local, versioned graph of steps, authored against an installed
  vocabulary (*spec*). Its seven node kinds are operations, assessments,
  inbound waits (`AwaitInbound`), conditions, approvals, evidence joins, and
  terminals. A *definition* is one published revision. An
  *instance* is one run of a pinned definition revision. *Control steps*
  (publish or retire a definition; start, cancel, migrate, or continue an
  instance on a fork; advance or navigate back; approve) are governed
  mutations whose bindings set `WORKFLOW_CONTROL`. Query's workflow kernel
  authorizes and records them, never a handler. Applications reach the kernel
  through the workflow runtime (`WorthQueryWorkflowApplicationRuntime`).
  Progress is caller-pumped.
  **Not** preview, promotion, writeback, branch merge, or a staged run of
  installed domain operations. Applications import only `worth-query-decl`
  and `worth-query-host`.
  **See** [Build an Application §6](build-an-application.md#6-author-publish-and-run-workflows)
  and [How WORTH Works §13](how-it-works.md#13-workflows).

**Workflow frontier**
: A declared set of independent stages in an installed domain operation's
  staged run. Preparation fixes each stage's facts on the owner; compute takes
  only its inert task; application returns to the owner in canonical stage
  order. The least failing stage bounds the applied prefix. An apply failure
  does not refund compute work already settled. This frontier is
  separate from a required wave and from application workflow control steps.
  **See** [How WORTH Works §9.9](how-it-works.md#99-resources-execution-and-cost).

**Workflow occurrence authority**
: The single-use binding an advance issues when it reaches a guarded
  Operation node, carried by `AwaitingOperation`. `for_workflow_operation`
  binds a mutation request to it; the request's operation and input must
  match the awaited occurrence, and the authority is rechecked at commit. It
  lives in memory: if it is lost, advance again. It binds one occurrence and is
  not permission: ordinary operation authorization still applies.
  **Not** an *approval* and **not** *execution authority*.
  **See** *Guarded operation*, *Authority*, and
  [How WORTH Works §13.1](how-it-works.md#131-who-may-run-a-guarded-operation).

**World (Runtime World)**
: The runtime that composes exact component bases into product branches and
  publishes across component owners in a fixed order. Its outcomes are
  `Performed`, `NoEffect`, or `ProductUnpublished`.
