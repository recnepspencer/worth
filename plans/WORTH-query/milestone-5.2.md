# Milestone 5.2 Engineering Spec: Preview Session Query Contexts And Branch Workflow Foundations

> **Status:** Completed. Closeout: [milestone-5.2-closeout.md](./milestone-5.2-closeout.md).
>
> **Roadmap parent:** [worth_query_roadmap.md](./WORTH_query_roadmap.md)
>
> **Vision parent:** [worth_query_vision.md](./WORTH_query_vision.md)
>
> **Prior milestone:** [milestone-5.md](./milestone-5.md)
>
> **Adjacent milestone:** [milestone-5.1.md](./milestone-5.1.md)
>
> **Next concurrent milestone:** `milestone-5.3.md` is not yet written; this spec treats Milestone 5.3 as parallel planning hardening rather than a dependency for preview semantics
>
> **Prior closeout:** [milestone-5-closeout.md](./milestone-5-closeout.md)
>
> **Test requirements:** [test-requirements.md](./test-requirements.md)
>
> **Primary architectural driver:** make preview-session basis identity, preview lifecycle identity, and preview-versus-promoted comparison first-class query artifacts so branch-native evaluation stays inside canonical query planning and result shaping instead of collapsing into ambient bridge orchestration or host-local branch aliases
>
> **Companion docs:**
> - [MENTALITY.md](../../docs/coding-guidelines/MENTALITY.md)
> - [arch_laws.md](../../docs/coding-guidelines/arch_laws.md)
> - [perf_laws.md](../../docs/coding-guidelines/perf_laws.md)
> - [domain_laws.md](../../docs/coding-guidelines/domain_structure_laws.md)
> - [worth_query_vision.md](./WORTH_query_vision.md)
> - [worth_query_roadmap.md](./WORTH_query_roadmap.md)
> - [test-requirements.md](./test-requirements.md)
> - [milestone-5.md](./milestone-5.md)
> - [milestone-5.1.md](./milestone-5.1.md)
> - [milestone-5-closeout.md](./milestone-5-closeout.md)
> - [worth-runtime-bridge milestone-10.md](../WORTH-runtime-bridge/milestone-10.md)
> - [worth-runtime-bridge milestone-12.md](../WORTH-runtime-bridge/milestone-12.md)
> - [BRANCHING_AND_SPECULATION.md](../../crates/worth-runtime-bridge/BRANCHING_AND_SPECULATION.md)

## Goal

Make preview and speculative sessions first-class query basis contexts so
admitted queries can bind to explicit bridge preview sessions, preserve
preview-lifecycle identity in plans and results, and compare preview-bound
results to promoted/authoritative outcomes through typed query-native
artifacts instead of host-local workflow glue.

## Why This Milestone Exists

Milestone 5 proved that canonical query meaning can survive time under live
maintenance. Milestone 5.1 hardens that substrate with locality-bearing live
narrowing and stream-contract honesty.

Those are necessary, but they still leave one of the product's most important
branch-native workflows outside the query framework:

- open a speculative session
- evaluate canonical queries against that session
- inspect preview lifecycle state and preview basis identity
- compare preview results against the corresponding promoted or authoritative
  outcome
- hand branch-native workflow artifacts forward to later merge/writeback work

If `worth-query` does not own that basis boundary, developers will fall out of
the query facade exactly when they need the strongest query guarantees:

- preview reads will become host-local branch aliases
- preview lifecycle state will live only in bridge diagnostics instead of in
  query-visible basis metadata
- preview-versus-promoted comparison will become ad hoc result diffing over
  ambient runtime state
- later workflow milestones will inherit a split architecture where reads are
  canonical but preview and branch workflow are orchestration folklore

Milestone 5.2 therefore exists to freeze:

- that preview session identity is a basis artifact, not a host variable
- that preview lifecycle identity remains bridge-owned but query-visible
- that read-only preview evaluation and promotable preview evaluation are
  distinct query context classes
- that preview-versus-promoted comparison is query-native and typed rather than
  inferred from unrelated result bags
- that branch workflow foundations for later compare/merge/writeback work are
  established now without making `worth-query` a second preview lifecycle
  engine

## Governing Summaries

- `MENTALITY.md`: the hard problem is not "let query hit preview branches." It
  is making speculative branch-native evaluation basis-honest, lifecycle-
  explicit, and promotion-comparison-safe under churn and replay pressure. The
  milestone must solve that structural problem first.
- `arch_laws.md`: Laws 1, 7, 8, 17, 18, 20, 21, 26, 27, 30, 33, 35, 40, and 41
  dominate this milestone. Preview lifecycle authority must stay in the bridge,
  query plans must carry explicit basis proofs, comparison artifacts must be
  lowered before execution, and proof-bearing types must encode exactly what
  preview facts have been established.
- `perf_laws.md`: preview support must not hide broad rebinding, repeated
  lifecycle rediscovery, or compare-by-reexecution folklore. Preview admission,
  basis resolution, lifecycle lookup, comparison width, and denial outcomes
  must be explicit counters and exact proof rows.
- `domain_laws.md`: preview basis modeling, lifecycle metadata, comparison
  lowering, workflow foundation artifacts, certification bundles, and
  diagnostics are separate responsibilities and must not collapse into one
  generic "workflow" or "preview" module.
- `worth_query_vision.md`: AI, workflow, geometry, and branch-scoped product
  surfaces all assume that branch-local and speculative truth can be queried
  through the same typed model as ordinary reads. Preview contexts are how that
  promise becomes structurally honest.
- `worth_query_roadmap.md`: Milestone 5.2 sits after live hardening and before
  frontier-aware planning and later mutation/merge lowering. It exists to make
  preview-session query contexts and branch workflow basis identity explicit
  before other workflow capabilities build on them.
- `test-requirements.md`: the `Preview Session Basis And Promotion Parity Test`
  is the closeout proof. It requires explicit preview identity, lifecycle-
  explicit bundles, parity-safe preview results for the same canonical query
  shape, query-native preview-versus-promoted comparison, and early typed
  denial for unsupported combinations.
- `milestone-5.md`: live promotion already established plan-derived, basis-
  explicit artifacts and replay-safe progress identity. Milestone 5.2 must
  reuse that proof discipline for preview basis classes instead of inventing a
  looser host-only workflow path.
- `milestone-5.1.md`: locality-bearing live semantics and stream-contract
  delivery remain adjacent but distinct. Milestone 5.2 must compose with those
  surfaces without redefining locality predicates, widening policy, or stream
  lowering.
- `milestone-5-closeout.md`: the runtime-backed live substrate is already
  closed and certified for admitted families. Preview basis work should extend
  the query basis and certification bundle story, not bypass it.
- `worth-runtime-bridge milestone-10.md`: the bridge already owns preview
  session declaration, activation, discard, promotion, replay, and non-
  authoritative isolation. Query must consume those artifacts as authority-
  preserving inputs rather than duplicating them.
- `worth-runtime-bridge milestone-12.md`: preview and authority boundaries,
  promotion admissibility, and preview replay records already have bridge-owned
  canonical forms. Query comparison artifacts should lower from those forms
  rather than pretending a promoted preview is just another branch head.
- `BRANCHING_AND_SPECULATION.md`: speculation is session-shaped, not ambient
  mode. Main-branch churn must not silently retarget preview basis, and preview
  versus commit must remain an explicit authority boundary. Milestone 5.2 must
  carry exactly that stance into query-land.

## Adversarial Constraint

Milestone 5.2 must survive the following hostile condition:

> The same canonical query shape is executed once against an ordinary runtime
> basis and once against an admitted preview session whose lifecycle may
> progress from declared to active to discarded or promoted while main-branch
> truth continues to move, and the query layer must preserve explicit preview
> basis identity, explicit preview-lifecycle identity, and explicit preview-
> versus-promoted comparison semantics without host glue silently retargeting
> basis, flattening lifecycle state, or diffing unrelated results heuristically.

Concretely, the design must remain correct when all of the following are true:

- a query binds to one explicit `BridgePreviewSessionIdentity` or materially
  equivalent bridge-owned preview-session artifact
- the same canonical query shape is executed against:
  - an ordinary runtime basis
  - one active preview session
  - one promoted or authoritative outcome derived from that preview session
- preview lifecycle state changes without the canonical query shape changing
- preview sessions may be read-only or promotable, and those classes must not
  collapse into one ambient "preview mode"
- main-branch churn occurs while the preview session remains open
- preview replay and promotion diagnostics exist in the bridge and must be
  visible as query context metadata without becoming query-owned lifecycle
  authority
- admitted preview-live maintenance may continue only while the preview session
  remains explicitly active, and any later drift must produce one typed denial
  or one typed explicit rebind artifact rather than ambient fallback
- later branch-workflow milestones need to inherit stable compare/promotion
  basis identity from this milestone rather than reverse-engineering it

If any supported path:

- treats preview context as a host-local branch alias instead of an explicit
  preview-session basis
- silently retargets preview queries when main-branch truth changes
- loses preview lifecycle state between planning, execution, and result
  shaping
- compares preview and promoted outcomes by ambient host orchestration instead
  of query-native comparison artifacts
- lets query execution rediscover bridge lifecycle or promotion semantics after
  planning
- reuses preview artifacts as if they were already authoritative results
  without explicit promotion-boundary metadata

then Milestone 5.2 has failed.

## Product Decision Lock

- preview session identity is a first-class query basis input, not a free-form
  host parameter and not a branch-name alias
- preview-session lifecycle authority remains owned by the runtime bridge;
  `worth-query` may carry lifecycle metadata and enforce basis honesty, but it
  may not create, mutate, discard, or promote preview sessions on its own in
  this milestone
- query meaning remains canonical and unchanged apart from the explicitly
  declared preview basis and lifecycle metadata
- read-only preview evaluation and promotable preview evaluation are distinct
  query context classes with distinct admission and comparison rules
- preview-versus-promoted comparison is a query-owned comparison artifact over
  explicit basis identities, not host-side result diffing and not a generic
  branch diff shortcut
- preview-live maintenance is an admitted derived mode only when it reuses the
  existing live proof chain and carries explicit preview-session basis
  identity; it is not an ambient continuation of ordinary live truth
- preview replay and promotion explanations remain bridge-owned explanation
  surfaces; query may lower them into query-native comparison and workflow
  artifacts, but may not redefine them
- branch workflow foundations in this milestone are declaration and basis
  artifacts only; mutation intent lowering, merge execution, and writeback
  lowering remain later work
- durable preview replay reload, persisted workflow artifacts, and restart-
  stable continuation remain out of scope

Normative consequence:

- any implementation path that accepts a raw branch string in place of preview
  session identity is out of spec
- any implementation path that lets hosts decide whether a preview was
  promotable, active, or stale is out of spec
- any implementation path that compares preview and authoritative results
  without carrying both explicit basis identities is out of spec
- any implementation path that silently rebases preview-live maintenance onto
  authoritative live truth is out of spec
- any implementation path that implies query owns preview lifecycle transitions
  is out of spec
- any implementation path that treats a preview result as authoritative without
  explicit promotion-boundary evidence is out of spec

## Compile-Time Enforcement Policy

Milestone 5.2 must classify which preview-context guarantees become
unrepresentable, uncompilable, or construction-time rejection.

`Unrepresentable` in public types:

- publicly constructible preview-bound query basis artifacts that do not carry
  source query identity, preview session identity, lifecycle identity, and
  preview evaluation class
- publicly constructible preview-bound execution or result envelopes that do
  not carry explicit preview basis metadata distinct from ordinary runtime
  basis metadata
- publicly constructible preview-versus-promoted comparison artifacts that do
  not carry both preview basis identity and promoted/authoritative basis
  identity
- publicly constructible preview-live bindings, drift outcomes, or rebind
  artifacts that do not carry both explicit preview basis identity and explicit
  live-family identity
- publicly constructible branch-workflow foundation declarations as open bags
  rather than closed query-owned families
- publicly constructible promotion-comparison eligibility as a naked boolean,
  string mode, or optional digest bag

`Uncompilable` through visibility and compile-fail enforcement:

- external construction of `PreviewSessionQueryContext`,
  `PreviewSessionBasis`, `PreviewLifecycleMetadata`,
  `PromotionParityPreviewComparisonAdmission`,
  `PreviewWorkflowFoundationArtifact`, or
  materially equivalent proof-bearing types without crate-owned lowering
- public APIs that accept raw bridge diagnostics, raw branch aliases, or host-
  authored promotion-comparison bags as though they were admitted preview query
  context inputs
- public APIs that let consumers fabricate lifecycle states such as
  `Active`, `Discarded`, or `Promoted` without bridge-owned proof input
- public APIs that accept post-plan mutation of preview evaluation class from
  read-only to promotion-eligible
- public conversion paths that bypass ordinary canonical planning and mint
  preview-bound results directly from host preview metadata

`Construction-time rejection`:

- non-admitted query families requested for preview binding
- invalid preview session basis requests
- unsupported read-only versus promotable preview evaluation modes
- unsupported preview-versus-promoted comparison requests
- unsupported preview lifecycle states for an otherwise admitted query family
- invalid bridge preview artifact combinations or stale preview references
- comparison requests whose preview and promoted lanes do not share the same
  canonical query digest and result-family contract

Rules:

- the strongest available boundary must be used
- preview-context and preview-comparison proof types must use sealed
  constructors and private fields
- compile-fail coverage is required for:
  - no raw preview-session declaration as query basis
  - no external construction of preview-bound proof types
  - no external fabrication of preview promotion comparison artifacts
  - no bool-driven promotion eligibility toggles
- runtime rejection is allowed only for facts genuinely unavailable until the
  bridge supplies validated preview-session lifecycle or replay evidence

## Scope

### In Scope

- query contexts that bind canonical query meaning to admitted bridge preview
  sessions
- explicit preview-session basis metadata and preview-lifecycle metadata on
  plans, execution reports, and result envelopes
- distinction between read-only preview evaluation and promotable preview
  evaluation
- query-native comparison artifacts for preview result versus promoted or
  authoritative result where the workflow admits it
- bridge-compatible preview replay and promotion metadata lowering sufficient
  for certification bundles
- branch-workflow foundation artifacts that later compare/merge/writeback
  milestones can extend without redefining preview basis semantics
- typed diagnostics, counters, replay bundles, and rejection surfaces for
  preview context admission and comparison
- milestone-native certification for preview basis identity, lifecycle parity,
  comparison parity, and rejection behavior

### Explicitly Out Of Scope

- locality-bearing live narrowing, widening policy, or stream-contract
  lowering; those remain Milestone 5.1 responsibilities
- frontier-aware planning posture or deterministic parallel admission; those
  remain Milestone 5.3 responsibilities
- mutation intent lowering, merge execution, conflict classification, or
  writeback lowering; those remain Milestone 5.5 work
- general branch-head, historical, or diff basis expansion beyond preview-
  session-specific basis classes
- durable preview replay reload, persisted workflow artifacts, or restart-
  stable continuation
- store-backed preview parity
- host transport lifecycle, UI workflow state, or generic branch management

### Initial Admission Matrix

Milestone 5.2 must not leave preview support ambient.

Initial preview-context-admitted query families:

- detail queries already admitted for runtime-backed execution
- ordered collection queries already admitted for runtime-backed execution
- bounded materialization queries already admitted for runtime-backed execution
- preview-bound live reuse for the corresponding admitted Milestone 5 and 5.1
  live families where preview lifecycle remains explicitly active and
  preview-live admission can reuse the existing live proof chain without
  redefining locality or stream semantics

Initial preview-context-denied query families:

- any family not already admitted through the Milestone 3 to 5 substrate
- store-backed execution routes
- historical/diff/lineage/correspondence families
- policy-masked or tenant-variant preview combinations not yet admitted
- any family whose preview meaning would require host-authored branch mapping
  or bridge-internal state discovery

Preview-bound live reuse rule:

- preview-bound live reuse is admitted only through one explicit
  preview-live proof chain built on top of Milestone 5 and 5.1 live artifacts
- preview-live admission requires:
  - one admitted preview session basis
  - one active preview lifecycle witness
  - one live-admitted query family whose locality and delivery semantics are
    already defined outside Milestone 5.2
- preview-live maintenance must never silently degrade into ordinary
  authoritative live truth
- when preview lifecycle leaves `Active`, the preview-live lane must produce one
  closed drift outcome:
  - typed denial when the lane can no longer continue honestly
  - typed explicit rebind artifact when the system can prove how to continue
    without ambient fallback
- explicit rebind is an admitted branch of the milestone, but it must be
  structurally stricter than denial and may not masquerade as automatic
  fallback

Initial comparison-admitted families:

- detail preview result versus promoted/authoritative detail result
- ordered collection preview result versus promoted/authoritative ordered
  collection result where ordering basis remains identical
- bounded materialization preview result versus promoted/authoritative bounded
  materialization result where traversal boundary remains identical

Initial comparison denials:

- comparisons that would require historical reconstruction
- comparisons that require mutation/merge lowering semantics not yet shipped
- comparisons whose promoted side lacks explicit bridge-owned promotion proof
  linkage
- comparisons that attempt to compare preview basis to a host-selected "latest"
  result without explicit basis identity
- comparisons whose lanes disagree on canonical query digest, result-family
  identity, ordering basis, or traversal/materialization boundary

Initial preview lifecycle states visible at the query seam:

- `Declared`
- `Admitted`
- `Active`
- `Discarded`
- `Promoted`

Lifecycle states not admitted as successful query evaluation contexts:

- discarded sessions for ordinary execution
- promoted sessions treated as if still preview-active
- any host-invented lifecycle category

### Initial Performance Posture Matrix

Milestone 5.2 must also freeze the first preview-basis cost postures rather
than leaving preview support as repeated lifecycle rediscovery.

Initial admitted cost postures:

- preview basis resolution is one explicit pre-execution step
- preview lifecycle lookup is one explicit plan/result metadata dependency, not
  an executor-side repeated probe
- preview-versus-promoted comparison uses basis-explicit pairing and one closed
  comparison lowering family per admitted query family

Initial denied cost postures:

- repeated bridge diagnostics scans to rediscover session state during row or
  patch shaping
- preview comparison implemented as broad host-side result diffing over
  unrelated basis contexts
- executor-side speculation about whether a preview is promotable

### Performance Must Be Encoded Structurally

Milestone 5.2 must not treat performance as an implementation detail layered on
after preview semantics are correct. The architecture itself must make the
cheap path the natural path and broad fallback the explicit exceptional path.

Required structural encodings:

- preview basis binding must be represented as one proof-bearing artifact that
  carries the full preview binding tuple forward so later phases consume proof
  rather than re-querying bridge diagnostics
- preview lifecycle state used by execution must be lowered into immutable
  query-owned metadata before execution begins; executor code may consume it but
  may not poll bridge state repeatedly
- promotion-comparison eligibility must be computed once before comparison
  execution and carried as a proof-bearing eligibility artifact
- preview-live admission, drift detection, denial, and explicit rebind must be
  represented as separate proof-bearing artifacts so executor code never
  improvises lifecycle continuation
- branch-workflow foundation artifacts must carry the already-resolved basis
  pair and comparison eligibility so later workflow milestones inherit a narrow
  path instead of rediscovering preview facts

Performance-encoding consequence:

- if a later phase needs to ask "which preview session is this?", "is this
  preview still active?", or "are these two lanes comparable?" by scanning
  diagnostics, bridge records, or host workflow state, the architecture is
  incomplete

### Named Complexity Contracts

Milestone 5.2 must declare explicit complexity contracts for the new preview
 surfaces rather than relying only on counters.

Minimum required contracts:

- `preview_basis_binding_contract`
  Preview basis binding is `O(1)` in query family width with respect to bridge
  lifecycle lookup count because it consumes one admitted preview binding tuple,
  not a search over retained preview records.
- `preview_execution_metadata_contract`
  Preview-bound execution is `O(1)` in preview lifecycle resolution work per
  execution lane because lifecycle metadata is pre-lowered and immutable for
  that execution.
- `preview_comparison_eligibility_contract`
  Comparison eligibility is `O(1)` in basis-pair resolution work and
  `O(ordering_width + traversal_boundary_width)` in shape-compatibility proof,
  never `O(result_rows)` or `O(materialized_payload_size)`.
- `preview_workflow_foundation_contract`
  Workflow foundation emission is `O(1)` in preview artifact lookup count
  because it consumes previously lowered basis and eligibility artifacts rather
  than rescanning bridge-owned records.
- `preview_live_admission_contract`
  Preview-live admission is `O(1)` in preview/live basis resolution work
  because it consumes one admitted preview basis plus one admitted live family
  proof rather than searching host workflow state.
- `preview_live_drift_contract`
  Preview-live drift checking is `O(1)` in lifecycle resolution work per
  maintenance step because it consumes one explicit active-lifecycle witness and
  one closed drift-outcome family rather than broad diagnostics rescans.
- `preview_live_rebinding_contract`
  Explicit preview-live rebind is `O(1)` in basis retargeting work with respect
  to preview session selection because the rebind artifact must name both the
  original preview-live basis and the newly admitted continuation basis
  directly.

Each contract must carry:

- named hot-path boundary
- declared complexity
- exact counters that prove the claim
- explicit `Verified` or `Debt` status

### Forbidden Hidden Cost Patterns

Milestone 5.2 must reject these performance-dishonest shapes explicitly:

- scanning retained preview execution, discard, or promotion records to resolve
  the current preview binding for an already-admitted query lane
- computing preview-versus-promoted comparison by materializing both result
  payloads and diffing them without prior shape-compatibility proof
- recomputing preview eligibility on every live patch or collection row instead
  of once at basis binding time
- allowing preview lifecycle state to live only in diagnostics and then
  reloading it ad hoc during result shaping
- using broad branch-head reads as a fallback when preview basis linkage is
  incomplete

If an implementation needs one of those to make the milestone work, the right
fix is to strengthen the artifact boundary, not to excuse the cost.

## Preview Query Architecture

### One Query Meaning, One Extra Basis Class

Milestone 5.2 extends the existing proof chain. It must not create a second
preview query language.

The authoritative flow should become:

`ValidatedQueryBundle`
-> `ExecutionPlanBundle`
-> `ExecutionPreflightBundle`
-> `PreviewSessionQueryContext`
-> `PreviewSessionPlanBinding`
-> `PreviewExecutionEnvelope`
-> `PreviewLiveSessionPlanBinding` where admitted
-> `PreviewLiveExecutionEnvelope` where admitted
-> `PromotionParityPreviewComparisonAdmission` where admitted

Preview support therefore consumes already-proven query meaning. It does not
re-author:

- predicates
- projection meaning
- ordering meaning
- traversal/materialization meaning
- result-family meaning
- live patch-family meaning, which must be reused rather than reauthored when
  preview-live composition is admitted in this milestone

### Authority Boundaries

`worth-query` owns:

- preview session query context vocabulary
- basis-resolution compatibility between canonical query execution and bridge
  preview artifacts
- preview lifecycle metadata on query plans and result envelopes
- preview-versus-promoted comparison lowering for admitted query families
- branch-workflow foundation artifacts that describe preview/compare intent in
  query-native terms
- query-native certification bundles for preview parity and rejection

The runtime bridge owns:

- preview-session declaration, validation, admission, activation, discard,
  promotion, and replay lifecycle
- preview session identity and declaration identity
- promotion admissibility proof semantics
- preview execution, discard, and promotion record authority
- preview replay bundle authority and diagnostics explanations

Execution owns:

- consuming already-lowered preview basis bindings
- executing admitted queries against the declared preview basis
- emitting preview-basis-explicit result envelopes
- producing comparison bundles only from admitted preview and promoted basis
  pairs

Hosts and workflow glue may own:

- deciding when to ask for preview evaluation
- presenting preview basis and lifecycle explanations to users
- using query-native preview artifacts as input to later UI workflow steps

Hosts and workflow glue may not own:

- deciding what preview session the query targeted after the fact
- rewriting preview basis to ordinary branch basis silently
- deciding whether preview-versus-promoted results are comparable
- synthesizing workflow foundation artifacts from raw bridge records outside
  the query facade

### Preview Basis, Lifecycle, And Comparison Artifacts

Milestone 5.2 must introduce one closed vocabulary for preview-bound query
basis.

Representative artifact families:

- `PreviewSessionQueryContext`
- `PreviewSessionBasis`
- `PreviewEvaluationClass`
- `PreviewLifecycleMetadata`
- `PreviewSessionPlanBinding`
- `PreviewExecutionEnvelope`
- `PreviewLiveSessionPlanBinding`
- `PreviewLiveExecutionEnvelope`
- `PreviewLiveMaintained`
- `PreviewLiveDriftOutcome`
- `PreviewLiveRebindArtifact`
- `PromotionParityPreviewComparisonAdmission`
- `PreviewWorkflowFoundationArtifact`
- `PreviewBindingCounters`
- `PreviewExecutionCounters`
- `PreviewComparisonCounters`
- `PreviewLiveCounters`

Representative bridge-owned inputs that query should lower from rather than
redefine:

- `BridgePreviewSessionIdentity`
- `BridgePreviewSessionDeclaration`
- `BridgePreviewExecutionRecord`
- `BridgePreviewPromotionRecord`
- `BridgePreviewReplayBundle`
- `BridgePromotionAdmissibilityProof`

Rules:

- preview basis identity must include explicit preview session identity and
  explicit query basis digest
- preview basis identity must not be reconstructed from host branch aliases,
  ambient diagnostics scans, or transport-local session labels
- preview lifecycle metadata must include the bridge lifecycle state relevant to
  the query result without making query the lifecycle authority
- preview execution envelopes must distinguish:
  - ordinary runtime basis
  - preview-read-only basis
  - preview-promotable basis
- preview-versus-promoted comparison bundles must carry:
  - preview basis digest
  - promoted/authoritative basis digest
  - comparison family identity
  - bridge promotion linkage where applicable
- comparison equivalence must be defined over canonical query shape plus
  explicit basis pairing, not over host-local timing or branch names
- preview-live composition must reuse Milestone 5 live artifacts and must not
  mint a second live proof chain beside the existing live promotion substrate
- preview-live drift handling must lower into one closed outcome family rather
  than ambient lifecycle polling or hidden fallback

### Minimum Preview Binding Tuple

Milestone 5.2 must make the preview binding tuple concrete enough that code can
map to it honestly and cannot "mostly know" what preview basis it used.

Minimum required preview binding fields:

- canonical query digest
- validated result-shape digest
- preview session identity
- preview declaration digest or materially equivalent bridge declaration
  identity
- preview lifecycle state kind
- preview execution record identity for any active preview execution
- preview replay-bundle digest when replay-backed preview explanation is part of
  the result bundle
- promotion record identity and promotion proof digest when promoted-result
  comparison is admitted

Rules:

- active preview execution may not proceed without preview execution record
  identity
- promoted-result comparison may not proceed without promotion record identity
  and promotion proof digest
- declared/admitted lifecycle states may be query-visible, but only active
  preview sessions are execution-admitted in this milestone
- query must lower this tuple once at basis binding time and carry it forward;
  it may not lazily rediscover missing fields during result shaping
- if any required field is unavailable, the operation must fail typed and early
  rather than falling back to best-effort preview interpretation

### Preview Evaluation Class Must Be Proof-Bearing

The split between read-only preview and promotable preview is load-bearing and
must not devolve into a convenience boolean.

Required shape:

- one closed `PreviewEvaluationClass` family with distinct variants or witness
  types for:
  - `ReadOnlyPreviewEvaluation`
  - `PromotionEligiblePreviewEvaluation`
- a read-only preview basis must be unrepresentable as promotion-comparable
  without passing through a query-owned eligibility proof
- a promotion-eligible preview basis must require bridge-owned promotion
  admissibility input before comparison lowering can begin

Forbidden shape:

- `is_promotable: bool`
- `allow_promotion_compare: bool`
- host-owned flags that reinterpret the same preview basis after planning

Normative consequence:

- if the implementation can toggle a preview result from read-only to
  promotion-eligible without reconstructing the proof-bearing basis artifact,
  the milestone is structurally wrong

### Branch Workflow Foundation Surface

Milestone 5.2 is not yet the mutation/merge milestone, but it must establish
the basis and artifact shape later workflow work will depend on.

Representative foundation families:

- `PreviewReadIntent`
- `PreviewCompareIntent`
- `PromotionComparisonEligibility`
- `PreviewWorkflowBasisPair`
- `PreviewWorkflowFoundationArtifact`

Rules:

- these artifacts describe preview/compare basis structure only
- they must not lower commits, merges, or writeback in this milestone
- they must be sufficient for later milestones to extend preview/compare into:
  - conflict inspection
  - merge intent
  - post-merge inspection
  - writeback declaration
- any artifact that would require commit strategy or merge semantics must fail
  typed and early in Milestone 5.2 rather than pretending the later workflow
  exists already

Minimum branch workflow foundation fields:

- canonical query digest
- preview session identity
- preview evaluation class
- preview lifecycle state kind
- preview basis digest
- optional promoted basis digest when comparison eligibility is admitted
- workflow foundation family identity
- explicit authority-boundary note stating which later workflow families remain
  out of scope

Forbidden foundation shape:

- opaque metadata bag intended for "future workflow use"
- generic `workflow_context` maps
- placeholders that imply merge/writeback support without typed denial

### Decision Topology Must Stay Three-State

Milestone 5.2 should not collapse all preview-bound decisions into binary pass
or fail outcomes.

At minimum, these decision surfaces must support success, advisory, and
violation classes with typed context:

- preview basis admission
- promotion-comparison eligibility
- workflow foundation admission

Reason:

- some preview contexts are valid but non-promotable
- some comparison requests are semantically meaningful but advisory-only for
  later workflow follow-up
- some workflow foundation requests must fail closed because they cross into
  unshipped merge/writeback authority

If implementation reduces those to bare booleans or one undifferentiated error,
it will destroy the exact context later workflow milestones need.

## Phases

### Phase 1: Freeze Preview Basis Semantics And Authority Boundaries

Phase 1 exists to prevent preview support from becoming ambient mode or branch
alias glue.

Milestone 5.2 must first define:

- one closed preview query context vocabulary
- one explicit distinction between ordinary basis, preview-read-only basis, and
  preview-promotable basis
- one explicit rule that preview lifecycle authority stays bridge-owned
- one explicit rule that preview-versus-promoted comparison is query-owned but
  proof-linked to bridge promotion artifacts
- one explicit branch-workflow foundation boundary that does not yet include
  mutation/merge/writeback lowering

This phase leaves the system in a coherent state where:

- preview support is a new basis class rather than a new query language
- lifecycle authority boundaries are explicit
- later workflow work has a fixed basis vocabulary to build on

### Phase 2: Lower Bridge Preview Artifacts Into Query Basis Bindings

Phase 2 exists to make preview-session context a real pre-execution proof
boundary instead of a host annotation.

Milestone 5.2 must then implement:

- lowering from admitted bridge preview-session artifacts into
  `PreviewSessionBasis`
- preview-session compatibility checks against canonical query execution
  preflights
- plan-bound preview lifecycle metadata attachment
- typed rejection for stale, invalid, discarded, or otherwise unsupported
  preview basis requests
- exact counters for preview admissions, preview basis resolutions, and invalid
  preview denials

This phase leaves the system in a coherent state where:

- the same canonical query shape binds deterministically to the same preview
  basis
- preview basis identity is explicit before execution
- invalid preview binding fails before semantic drift begins

### Phase 3: Execute Preview-Bound Queries Without Semantic Drift

Phase 3 exists to make preview-bound execution a true query execution mode.

Milestone 5.2 must then implement:

- preview-bound execution envelopes for admitted query families
- explicit preview lifecycle metadata on execution reports and result bundles
- parity-safe execution against ordinary runtime basis and preview basis for the
  same query shape
- exact counters for preview lifecycle lookups, preview execution runs, and
  preview basis mismatch denials
- zero executor rediscovery of lifecycle meaning on admitted paths

This phase leaves the system in a coherent state where:

- preview query execution stays canonical apart from explicit basis change
- query results can explain which preview session and lifecycle state they came
  from
- execution is not relying on ambient branch/workflow glue

### Phase 4: Admit Preview-Bound Live Maintenance With Explicit Drift Outcomes

Phase 4 exists to solve the hard continuation problem rather than leaving
preview-live as ambient host glue.

Milestone 5.2 must then implement:

- one proof-bearing `PreviewLiveSessionPlanBinding` family that composes:
  - one admitted preview session plan binding
  - one live-admitted Milestone 5 or 5.1 family
  - one explicit active preview lifecycle witness
- one explicit rule that preview-live maintenance reuses existing live
  patch-family, locality, and stream-contract semantics rather than redefining
  them inside the preview module
- one closed `PreviewLiveDriftOutcome` family with at minimum:
  - `DriftDenied`
  - `ExplicitRebindAvailable`
- typed denial when preview lifecycle leaves `Active` and no honest
  continuation exists
- typed explicit rebind artifact when the system can prove how to continue
  without silently retargeting to authoritative live truth
- exact counters for preview-live admission, preview-live maintenance, preview-
  live drift checks, denied drift, explicit rebind availability, and forbidden
  fallback

This phase leaves the system in a coherent state where:

- preview-live is one real proof chain rather than an extra host mode
- lifecycle drift is explicit and typed
- denial and rebind are both first-class outcomes, not ambient host policy
- ordinary live truth and preview-live truth cannot be confused

### Phase 5: Lower Preview-Versus-Promoted Comparison Into Typed Query Artifacts

Phase 5 exists to prevent preview comparison from degrading into host-side
diff folklore.

Milestone 5.2 must then implement:

- one closed comparison request family for preview result versus promoted or
  authoritative result
- one proof-bearing comparison eligibility step that verifies the preview lane
  and promoted lane share the same:
  - canonical query digest
  - result-family identity
  - ordering basis where applicable
  - traversal/materialization boundary where applicable
- comparison lowering that requires explicit preview basis plus explicit
  promoted/authoritative basis identity
- bridge promotion proof linkage where the workflow admits promoted-result
  comparison
- typed rejection for unsupported comparison families or missing promotion
  linkage
- exact counters for preview/promotion comparison runs and comparison denials

This phase leaves the system in a coherent state where:

- preview comparison is query-native and typed
- promoted results are not confused with still-preview results
- shape incompatibility is rejected before execution or diff construction begins
- later workflow milestones can inherit a stable compare surface

### Phase 6: Emit Branch Workflow Foundation Artifacts

Phase 6 exists to make later branch-native workflow work additive instead of
architectural surgery.

Milestone 5.2 must then implement:

- preview/compare workflow basis-pair artifacts
- foundation declarations for later compare/merge/writeback extension
- diagnostics and counters explaining why a workflow foundation artifact was
  admitted or denied
- exact denial paths for workflow requests that exceed this milestone's scope

This phase leaves the system in a coherent state where:

- later workflow milestones do not need to redefine preview semantics
- query can remain the daily-driver branch workflow facade without stealing
  lower-crate authority early

### Phase 7: Certification, Counter Proof, And Boundary Hardening

Phase 7 exists to close the milestone through named proof rather than
"query can hit preview now" demos.

Milestone 5.2 must finally ship:

- the `Preview Session Basis And Promotion Parity Test`
- canonical rows proving:
  - preview-basis execution parity
  - preview-lifecycle-explicit execution
  - preview-live admission parity
  - preview-live drift explicitness
  - preview-versus-promoted comparison parity
  - branch-workflow foundation admission
- rejection rows proving:
  - unsupported-preview-family
  - invalid-preview-basis
  - stale-preview-lifecycle-denied
  - preview-live-drift-denied
  - preview-live-broad-fallback-forbidden
  - unsupported-preview-promotion-comparison
  - raw-branch-alias-preview-forbidden
  - fabricated-preview-lifecycle-forbidden
- compile-fail or privacy hardening proving preview-bound proof types cannot be
  WORTHd externally

This phase leaves the system in a coherent state where:

- preview-session query contexts are certifiable rather than aspirational
- Milestone 5.3 can harden planning posture in parallel without redefining
  preview basis meaning
- Milestone 5.5 can extend branch workflow on top of real preview foundations

### Representative Scenario Matrix

Milestone 5.2 certification should exercise at minimum:

- `detail-preview-basis-parity`:
  one detail query executed against ordinary runtime basis and one active
  preview session with identical query meaning apart from basis identity
- `collection-preview-basis-parity`:
  one ordered collection query executed against ordinary runtime basis and one
  active preview session with explicit lifecycle metadata
- `bounded-materialization-preview-basis-parity`:
  one bounded materialization query bound to one active preview session
- `preview-live-admission-parity`:
  one admitted preview basis lowered into one preview-live lane that reuses the
  corresponding live family without changing canonical query meaning
- `preview-live-drift-denied`:
  one preview-live lane whose preview lifecycle leaves `Active` and must deny
  continued maintenance typed and early
- `preview-live-explicit-rebind`:
  one preview-live lane whose lifecycle drift admits one explicit rebind
  artifact rather than silent fallback
- `preview-promotion-comparison-parity`:
  one admitted preview result compared against one explicit promoted result
  with bridge promotion proof linkage
- `read-only-preview-denies-promotion-comparison`:
  one preview-read-only lane that must deny promoted-result comparison
- `discarded-preview-execution-denied`:
  one discarded preview session requested as though still executable
- `raw-branch-alias-preview-forbidden`:
  one hostile lane trying to bind preview query context from a host branch alias
  without preview session identity

If the harness cannot name concrete lanes at this granularity, the milestone is
still too abstract to close honestly.

## Must Ship

- proof-bearing `PreviewSessionQueryContext`, `PreviewSessionBasis`,
  `PreviewLifecycleMetadata`, `PromotionParityPreviewComparisonAdmission`, and
  `PreviewWorkflowFoundationArtifact` families or materially equivalent types
- preview-session basis lowering from admitted bridge preview artifacts into
  query execution context
- explicit preview lifecycle metadata on plans, execution reports, and result
  envelopes
- distinction between read-only preview evaluation and promotable preview
  evaluation
- preview-live admission, maintenance, drift, and explicit rebind artifacts for
  the admitted corresponding live families
- query-native comparison surfaces for preview result versus promoted or
  authoritative result where admitted
- branch-workflow foundation artifacts for later compare/merge/writeback
  milestones
- one dedicated preview/workflow performance subdomain owning counters and
  contract status rather than generic telemetry-only logging
- typed preview diagnostics, replay bundles, and exact counters
- milestone-native certification proving preview basis parity, lifecycle
  explicitness, comparison parity, and rejection behavior

## Must Preserve

- canonical query meaning from Milestone 1 remains authoritative
- validation legality from Milestone 2 remains authoritative
- one-shot planning and basis identity from Milestone 3 remain authoritative
- collection/result-family semantics from Milestone 4 remain authoritative
- live promotion and patch semantics from Milestones 5 and 5.1 remain
  authoritative and unchanged while preview-live composition reuses them
  through one explicit proof chain
- the runtime bridge remains authoritative for preview-session lifecycle,
  promotion admissibility, and preview replay artifacts
- preview contexts do not degrade into host-local branch aliases
- preview-versus-promoted comparison cannot proceed without explicit basis
  pairing and promotion linkage where required
- branch workflow foundations do not imply mutation/merge/writeback support
  that has not shipped
- durable preview replay reload and persisted workflow artifacts remain out of
  scope and explicitly deferred

## Complexity / Proof Obligations

Milestone 5.2 must name costs and proofs in terms of:

- preview session admission count
- preview basis resolution count
- preview lifecycle lookup count
- preview lifecycle rediscovery count
- preview execution count
- preview-live admission count
- preview-live execution count
- preview-live lifecycle check count
- preview-live drift denial count
- preview-live explicit rebind count
- preview/promotion comparison count
- preview comparison eligibility proof count
- preview comparison shape-check width
- invalid preview basis denial count
- invalid preview lifecycle denial count
- comparison denial count
- workflow foundation admission count
- workflow foundation denial count
- workflow foundation artifact lookup count
- preview broad-fallback denial count
- work avoided by explicit preview basis pairing versus host rediscovery
- executor rediscovery avoidance on preview-bound paths

Minimum required counters:

- `preview_session_admission_count`
- `preview_basis_resolution_count`
- `preview_lifecycle_lookup_count`
- `preview_lifecycle_rediscovery_count`
- `preview_execution_count`
- `preview_promotable_execution_count`
- `preview_read_only_execution_count`
- `preview_live_admission_count`
- `preview_live_execution_count`
- `preview_live_lifecycle_check_count`
- `preview_live_drift_denial_count`
- `preview_live_rebind_available_count`
- `preview_live_broad_fallback_denial_count`
- `preview_invalid_basis_denial_count`
- `preview_invalid_lifecycle_denial_count`
- `preview_promotion_comparison_count`
- `preview_comparison_eligibility_proof_count`
- `preview_comparison_shape_check_width`
- `preview_promotion_comparison_denial_count`
- `preview_workflow_foundation_admission_count`
- `preview_workflow_foundation_denial_count`
- `preview_workflow_foundation_artifact_lookup_count`
- `preview_replay_bundle_lookup_count`
- `preview_bridge_promotion_linkage_count`
- `preview_basis_pair_width`
- `preview_broad_fallback_denial_count`
- `preview_work_avoided_by_explicit_basis_count`
- `preview_executor_rediscovery_count`

Rules:

- counters belong to preview execution envelopes, comparison bundles, and
  certification bundles
- representative certification scenarios must assert exact counts
- `preview_executor_rediscovery_count` must be exactly zero on every admitted
  path
- `preview_lifecycle_rediscovery_count` must be exactly zero on every admitted
  execution and comparison path
- every invalid preview basis request must increment
  `preview_invalid_basis_denial_count`
- every invalid lifecycle request must increment
  `preview_invalid_lifecycle_denial_count`
- every denied comparison request must increment
  `preview_promotion_comparison_denial_count`
- every denied preview-live drift continuation must increment
  `preview_live_drift_denial_count`
- every admitted explicit rebind availability must increment
  `preview_live_rebind_available_count`
- every forbidden fallback from preview-live to ordinary live truth must
  increment `preview_live_broad_fallback_denial_count`
- every denied workflow foundation request must increment
  `preview_workflow_foundation_denial_count`
- every denied broad fallback must increment
  `preview_broad_fallback_denial_count`
- no supported path may hide basis reinterpretation inside generic execution or
  comparison counts
- "work avoided" counters must make explicit that query consumed stable preview
  identity instead of rediscovering workflow state from host glue
- workflow foundation emission must keep
  `preview_workflow_foundation_artifact_lookup_count` bounded to the number of
  already-lowered artifacts it consumes, not the number of retained bridge
  records

Minimum certification rows should include:

- `preview-basis-execution-parity`
- `preview-lifecycle-explicitness`
- `preview-promotion-comparison-parity`
- `preview-lifecycle-no-rediscovery`
- `preview-live-admission-parity`
- `preview-live-drift-explicitness`
- `preview-comparison-shape-proof-width`
- `preview-shape-incompatibility-denied`
- `preview-workflow-foundation-admission`
- `preview-workflow-foundation-no-rescan`
- `preview-work-avoided-counter-parity`

Minimum rejection rows should include:

- `unsupported-preview-family`
- `invalid-preview-basis`
- `stale-preview-lifecycle-denied`
- `preview-live-drift-denied`
- `preview-live-broad-fallback-forbidden`
- `unsupported-preview-promotion-comparison`
- `promotion-eligibility-bool-forbidden`
- `preview-shape-mismatch-denied`
- `preview-broad-fallback-forbidden`
- `preview-diagnostics-rescan-forbidden`
- `raw-branch-alias-preview-forbidden`
- `fabricated-preview-lifecycle-forbidden`
- `out-of-scope-workflow-foundation-request`

## Allowed Debt

- some query families may remain non-preview-admitted as explicit `Debt` while
  admitted families are fully parity-proven
- richer preview comparison families may remain `Debt` if admitted comparison
  semantics are closed, explicit, and certified
- some preview-live family combinations may remain `Debt` if the admitted
  preview-live families already have explicit drift denial, explicit rebind
  semantics, and machine-checkable proof
- broader workflow foundation declarations may remain `Debt` if the shipped
  preview/compare foundation boundary is explicit and certified
- durable preview replay reload and persisted workflow artifacts may remain
  blocked on later durable milestones
- host-local preview basis selection, branch alias substitution, or ambient
  promotion comparison may not exist as debt
- query-owned preview lifecycle mutation may not exist as debt

## Acceptance Evidence

Milestone 5.2 is complete only when `worth-query` can prove:

- the `Preview Session Basis And Promotion Parity Test` in
  [test-requirements.md](./test-requirements.md)
  passes with canonical machine-checkable artifacts
- preview-session-bound queries preserve explicit basis and lifecycle identity
- preview-bound results preserve canonical query meaning apart from the
  declared preview basis
- preview-versus-promoted comparison remains query-native, typed, and explicit
- preview-live maintenance, denial, and explicit rebind remain basis-explicit
  and never silently retarget to authoritative live truth
- unsupported preview-session query combinations fail typed and early
- branch-workflow foundation artifacts remain authority-preserving and do not
  imply unshipped merge/writeback semantics

Required verification output must include:

- `query_digest`
- `basis_digest`
- `result_digest`
- `replay_digest`
- `counter_snapshot`

## Architectural Notes

### Preview Must Stay Session-Shaped

The bridge already established that speculation is session-shaped, not ambient
mode. Milestone 5.2 must preserve that exactly.

That means:

- the query basis points at a preview session identity
- lifecycle state is explicit
- promotion is an explicit authority boundary
- discard is terminal and non-authoritative

It must not allow:

- "preview mode" booleans without session identity
- host-selected branch aliases that happen to mean preview
- comparison against whatever the host currently thinks is authoritative

### Preview Comparison Is Not Generic Diff

Preview-versus-promoted comparison is load-bearing because it is the first
branch workflow comparison surface query owns directly.

The required rule is:

- comparison lowers from one canonical query shape plus two explicit basis
  identities and, where needed, bridge promotion linkage

It must not become:

- generic branch diffing
- host-local JSON diffing
- "compare these two result bags and hope they line up"

### Milestone 5.2 Must Not Steal 5.1 Or 5.3

Because 5.1 and 5.3 are being built concurrently, this spec must hold a hard
line on boundaries:

- Milestone 5.1 owns locality-bearing live narrowing and stream-contract
  delivery semantics
- Milestone 5.2 owns preview basis identity, lifecycle metadata, and preview-
  versus-promoted comparison semantics
- Milestone 5.3 owns frontier-aware planning posture and deterministic parallel
  admission

Milestone 5.2 may compose with live and planning artifacts that already exist,
but it may not redefine locality policy, stream policy, frontier posture, or
parallel-admission logic.

### Workflow Foundations Must Stop Before Mutation Authority

The easiest way to overreach here is to let preview foundations quietly become
early merge or writeback semantics.

Milestone 5.2 must instead stop at:

- preview basis pairing
- compare intent
- workflow foundation artifacts
- typed denials for requests that require mutation or merge authority

If the spec or code starts smuggling commit strategy, merge semantics, or
writeback safety into 5.2, the milestone has crossed its authority boundary.

## Sequencing Notes

Milestone 5.2 belongs immediately after Milestone 5 and adjacent to Milestone
5.1 because preview session query contexts need the already-frozen proof-
bearing query, basis, and live artifact substrate.

It must land before Milestone 5.5 because later mutation/merge/writeback
workflow surfaces need stable preview basis identity and preview comparison
artifacts rather than retrofitting them.

It does not need to wait for Milestone 5.3 in full, because frontier-aware
planning posture is a planning hardening milestone, not the semantic authority
for preview basis identity.

## Parallelization Notes

Once the preview basis vocabulary and comparison boundary are frozen:

- Milestone 5.1 can continue locality/live hardening in parallel without
  changing preview basis meaning
- Milestone 5.3 can harden planning posture in parallel without changing
  preview lifecycle or comparison semantics
- early Milestone 5.5 workflow lowering experiments can proceed behind explicit
  debt markers without redefining preview foundations
- compile-time hardening, counter tightening, and certification row expansion
  can proceed in parallel without changing milestone semantics

## Explicit Failure Taxonomy For Milestone 5.2

- unsupported preview-bound query family
- invalid preview basis request
- stale preview lifecycle state
- preview lifecycle incompatibility
- preview/promoted basis mismatch
- missing promotion linkage for comparison
- unsupported preview comparison family
- raw branch alias masquerading as preview basis
- fabricated preview lifecycle metadata
- workflow foundation scope overreach
- preview artifact invariant break

## Anti-Patterns Explicitly Rejected

- preview support implemented as branch-name aliasing
- ambient "preview mode" without explicit preview session identity
- query execution that rediscovers preview lifecycle state repeatedly from
  bridge diagnostics
- preview-versus-promoted comparison implemented as host-local result diffing
- query-owned preview lifecycle mutation or promotion logic
- one mega-module mixing preview basis modeling, bridge lookup, comparison
  lowering, workflow foundations, replay, and diagnostics
- public construction of preview-bound proof types without the proving path
- milestone wording that implies merge/writeback support before those lowering
  milestones exist

## Self-Check

This milestone solves a real structural problem rather than packaging work
cosmetically because it introduces the first preview-session basis boundary
inside the query framework and makes branch-native speculative evaluation
structurally query-native.

The adversarial constraint is load-bearing because it forbids the naive failure
mode where preview execution becomes a host-local branch alias with ambient
lifecycle and promotion meaning.

The milestone preserves authority boundaries because the runtime bridge still
owns preview lifecycle and promotion authority, while `worth-query` owns basis
binding, preview-visible metadata, comparison lowering, and workflow foundation
artifacts.

The milestone defines proof obligations rather than implementation chores
because basis parity, lifecycle explicitness, comparison parity, typed denials,
and exact counters are required for closeout.

A competent engineer should be able to map this spec into honest preview basis,
comparison, workflow foundation, certification, and compile-fail modules
without inventing the architecture during implementation.

This milestone belongs at 5.2 because it is the decimal insertion that makes
branch-native preview workflows explicit before later planning hardening and
workflow mutation lowering compose on top.

## Closeout Standard

Milestone 5.2 is complete only when all of the following are true:

- admitted query families can bind to explicit preview session basis without a
  second query language
- preview lifecycle identity is explicit on plans and results
- preview-bound execution preserves canonical query meaning apart from the
  declared preview basis
- preview-versus-promoted comparison is query-native, typed, and basis-explicit
- branch-workflow foundation artifacts exist without implying unshipped
  mutation/merge/writeback authority
- unsupported preview families, stale lifecycle states, and raw branch-alias
  shortcuts fail typed and early
- certification bundles prove parity and denial through canonical machine-
  checkable artifacts

If code lands but preview support still depends on branch aliases, ambient host
workflow state, fabricated lifecycle metadata, host-side result diffing, or
query-owned promotion logic, Milestone 5.2 is not complete.
