# Milestone 5.5 Engineering Spec: Query-Orchestrated Mutation, Merge, And Writeback Declarations

> **Status:** Draft engineering spec
>
> **Roadmap parent:** [worth_query_roadmap.md](./worth_query_roadmap.md)
>
> **Vision parent:** [worth_query_vision.md](./worth_query_vision.md)
>
> **Prior milestone:** [milestone-5.4.md](./milestone-5.4.md)
>
> **Adjacent milestones:** [milestone-5.2.md](./milestone-5.2.md) and [milestone-5.3.md](./milestone-5.3.md) are already closed and remain authority-distinct inputs for preview/workflow basis identity and route-posture honesty.
>
> **Prior closeout:** [milestone-5.4-closeout.md](./milestone-5.4-closeout.md)
>
> **Test requirements:** [test-requirements.md](./test-requirements.md)
>
> **Primary architectural driver:** make query-authored mutation, merge, conflict-inspection, post-merge inspection, and writeback declarations first-class query artifacts that lower into `worth-relational` and bridge authorities without turning `worth-query` into a second mutation engine or hiding authority transfer behind host workflow glue
>
> **Companion docs:**
> - [MENTALITY.md](../../docs/coding-guidelines/MENTALITY.md)
> - [arch_laws.md](../../docs/coding-guidelines/arch_laws.md)
> - [perf_laws.md](../../docs/coding-guidelines/perf_laws.md)
> - [domain_laws.md](../../docs/coding-guidelines/domain_laws.md)
> - [worth_query_vision.md](./worth_query_vision.md)
> - [worth_query_roadmap.md](./worth_query_roadmap.md)
> - [test-requirements.md](./test-requirements.md)
> - [milestone-5.2.md](./milestone-5.2.md)
> - [milestone-5.2-closeout.md](./milestone-5.2-closeout.md)
> - [milestone-5.3.md](./milestone-5.3.md)
> - [milestone-5.3-closeout.md](./milestone-5.3-closeout.md)
> - [milestone-5.4.md](./milestone-5.4.md)
> - [milestone-5.4-closeout.md](./milestone-5.4-closeout.md)
> - [worth_runtime_bridge_roadmap.md](../worth-runtime-bridge/worth_runtime_bridge_roadmap.md)
> - [milestone-10.md](../worth-runtime-bridge/milestone-10.md)
> - [milestone-12.md](../worth-runtime-bridge/milestone-12.md)
> - [worth_relational_roadmap.md](../worth-relational/worth_relational_roadmap.md)
> - [milestone-7c-authoritative-merge-execution-spec.md](../worth-relational/milestone-7c-authoritative-merge-execution-spec.md)
> - [milestone-7d-deletion-and-topology-merge-execution-spec.md](../worth-relational/milestone-7d-deletion-and-topology-merge-execution-spec.md)

## Goal

Make query-authored mutation, merge, conflict-inspection, post-merge
inspection, and writeback declarations first-class query workflow surfaces so
domain developers can stay inside `worth-query` for branch-native workflow
orchestration while all authoritative mutation, merge execution, writeback
safety, idempotence, and publication remain owned by `worth-relational` and
the runtime bridge.

## Why This Milestone Exists

Milestone 5 made live query meaning survive time. Milestone 5.1 made locality-
bearing live narrowing and stream lowering explicit. Milestone 5.2 made
preview-session basis identity and preview-versus-promoted comparison
query-native. Milestone 5.3 made route posture planner-owned. Milestone 5.4
made advisory-versus-authoritative correspondence and historical-path honesty
explicit.

Those milestones solved how `worth-query` reads, compares, and explains truth.
They did not yet solve how ordinary application code moves from query-native
inspection into query-native branch workflow action.

Without Milestone 5.5, the platform still fractures at the exact point where
developers need it to feel like one coherent framework:

- query locates the branch/session/basis context
- query explains preview, correspondence, and historical state
- then the developer must fall out of `worth-query` into raw relational merge
  APIs, raw bridge writeback surfaces, or host-local glue to actually express
  "commit this", "merge this", "inspect conflicts", or "write back this
  derived outcome"

If that fracture remains:

- query becomes a read framework but not the daily-driver workflow facade the
  roadmap claims
- preview/compare basis identity from Milestone 5.2 stops before the first
  authority-boundary transition
- merge semantics from `worth-relational` and writeback semantics from the
  bridge disappear behind host glue instead of staying query-visible
- writeback and merge safety risks move into adapters, controllers, or UI code
  precisely where the platform is supposed to remove them

Milestone 5.5 therefore exists to freeze:

- that `worth-query` owns workflow declaration and lowering, not mutation truth
- that query-authored workflow artifacts must preserve the exact lower-crate
  authority seams they target
- that preview/compare/merge/writeback flows are declaration-shaped and
  proof-bearing rather than ambient host orchestration
- that conflict inspection and post-merge inspection stay query-shaped instead
  of degrading into raw lower-crate diagnostics bags
- that unsupported workflow families fail typed and early rather than
  degrading into hidden host fallback

## Governing Summaries

- `MENTALITY.md`: the hard problem is not "let query mutate data." It is
  letting query author workflow intent while preserving exact authority
  boundaries under merge pressure, replay pressure, and writeback safety
  pressure. The milestone must solve that boundary problem first.
- `arch_laws.md`: Laws 4, 6, 7, 8, 17, 20, 21, 26, 27, 30, 33, 35, 40, and 41
  dominate this milestone. Query-authored workflow intent must lower before
  execution, boundary crossings must be self-describing, execution must consume
  lowered decisions rather than rediscovering semantics, and proof-bearing
  types must encode what authority has and has not been granted.
- `perf_laws.md`: workflow orchestration must not hide broad re-discovery,
  branch rescans, or fallback mutation work behind cheap-looking facade calls.
  Mutation admission, merge inspection width, writeback admission width,
  causality width, and denial outcomes must be mechanically visible.
- `domain_laws.md`: workflow declarations, conflict inspection, merge intent
  lowering, relational-commit lowering, bridge writeback lowering,
  post-merge inspection, diagnostics, and certification are separate
  responsibilities and must not collapse into one "workflow" mega-module.
- `worth_query_vision.md`: `worth-query` is supposed to be the typed,
  composable developer surface for branch-native and workflow-aware products.
  That claim is incomplete if query stops at reads while the actual mutation,
  merge, and writeback workflows require raw lower-crate APIs.
- `worth_query_roadmap.md`: Milestone 5.5 is the explicit roadmap home for
  query-authored mutation intents, branch-native workflow orchestration, and
  query-triggered writeback declarations. It belongs after preview and route
  posture hardening and before the unified application facade.
- `test-requirements.md`: the `Query Workflow Lowering And Writeback Boundary
  Test` is the closeout proof. It requires query-authored mutation, merge,
  conflict inspection, post-merge inspection, and writeback declarations to
  lower into lower-crate authorities without semantic drift or authority
  duplication.
- `milestone-5.2.md` and `milestone-5.2-closeout.md`: preview basis identity,
  preview-live drift handling, and preview-versus-promoted comparison are
  already query-native. Milestone 5.5 must consume those explicit basis pairs
  and workflow foundations rather than inventing new host-side branch workflow
  context.
- `milestone-5.3.md` and `milestone-5.3-closeout.md`: route posture and bundle
  basis proof are already planner-owned. Milestone 5.5 must preserve that
  route honesty when conflict inspection or post-merge inspection query lanes
  compose with merge and workflow declarations.
- `milestone-5.4.md` and `milestone-5.4-closeout.md`: advisory structural
  correspondence, lineage continuity, and historical materialization-path
  metadata are already typed. Milestone 5.5 must use those explicit comparison
  and history surfaces for conflict/post-merge inspection rather than flattening
  everything into one generic merge result story.
- `worth_runtime_bridge_roadmap.md`: the bridge remains the authority for
  preview-session lifecycle, writeback safety, idempotence, replay artifacts,
  and writeback-family protocol meaning. Query may declare and lower into those
  bridge contracts but may not redefine them.
- `worth-runtime-bridge/milestone-10.md`: preview-session lifecycle,
  promotion-boundary records, and speculative/non-authoritative identity are
  already bridge-owned concepts. Query workflow declarations must consume those
  explicit artifacts rather than branch aliases or preview booleans.
- `worth-runtime-bridge/milestone-12.md`: bridge-mediated writeback is an
  explicit declaration, effect, idempotence, and authority-boundary protocol.
  Query-triggered writeback in 5.5 must lower into that protocol, not invent a
  second writeback contract.
- `worth_relational_roadmap.md`: `worth-relational` remains authoritative for
  commit strategies, merge execution, invariant enforcement, branch-head
  advancement, and canonical history publication. Query may declare intent and
  lower context into those authorities, but it may not become a second commit
  runtime.
- `milestone-7c-authoritative-merge-execution-spec.md`: authoritative merge
  execution consumes prepared, lowered, freshness-checked merge artifacts and
  executes through the shared commit pipeline. Query merge intent must lower
  into those proof-bearing relational surfaces rather than bypassing them with
  host-side merge shortcuts.
- `milestone-7d-deletion-and-topology-merge-execution-spec.md`: merge classes
  have explicit executable versus denied ontology. Query conflict inspection and
  merge intent lowering must preserve those typed merge-class distinctions
  rather than collapsing them into generic "conflict" or "merge failed" bags.

## Adversarial Constraint

Milestone 5.5 must survive the following hostile condition:

> The same canonical query shape is used to resolve branch/workflow context,
> inspect preview or correspondence evidence, declare mutation or merge intent,
> lower that intent into relational or bridge authorities, inspect conflicts,
> inspect post-merge outcomes, and optionally declare derived writeback; every
> admitted path must preserve explicit basis identity, authority ownership,
> causality, merge-class honesty, and replay-safe diagnostics without letting
> `worth-query` become a second mutation engine or letting host code fill in
> missing workflow semantics.

Concretely, the design must remain correct when all of the following are true:

- preview/session identity from Milestone 5.2 and correspondence/history
  identity from Milestone 5.4 are both part of the workflow context
- the same user-facing workflow may need:
  - read-only conflict inspection
  - a merge declaration
  - post-merge inspection
  - a query-triggered writeback declaration
- some merge classes are executable and some are typed denials under
  `worth-relational`
- some writeback families are bridge-admitted and some are denied
- a naive host would be tempted to stitch together preview facts, merge API
  calls, and writeback retries outside the canonical query plan/lowering
  boundary
- replay, diagnostics richness, and lower-runtime capability availability may
  vary without changing the canonical meaning of the declaration or denial

If any supported path:

- lets query execute mutation truth directly instead of lowering into
  relational or bridge authorities
- hides preview basis, branch basis, merge-class, or writeback-family identity
  during workflow lowering
- lets hosts supply merge truth, commit strategy identity, or writeback safety
  semantics ad hoc
- turns conflict inspection into raw lower-crate diagnostic bags with no
  query-shaped result contract
- hides authority transfer, idempotence, or causality inside convenience
  facade calls
- silently falls back from unsupported workflow declarations to host-local
  orchestration
- makes replay or certification depend on ambient UI/controller state rather
  than canonical declaration artifacts

then Milestone 5.5 has failed.

## Product Decision Lock

- `worth-query` owns workflow declaration, workflow context binding, lowering,
  and result shaping; it does not own mutation truth, merge execution, or
  writeback authority
- query-authored mutation intent is a declaration surface that lowers into
  relational commit strategy requests or merge execution requests; it is not a
  direct mutation API
- query-authored branch workflow declarations must cover at least:
  - preview / compare continuation from Milestone 5.2
  - conflict inspection
  - merge intent
  - post-merge result inspection
- conflict inspection is a query-shaped inspection family over lower-crate
  proof artifacts; it is not raw merge planner output handed straight to hosts
- post-merge inspection is a query-shaped result family over authoritative
  outcomes; it is not a convenience alias for "fetch the latest branch head"
- query-triggered writeback is declaration-owned by query and execution-owned
  by the bridge; query may not mutate truth or bridge records directly
- preview-session lifecycle authority remains in the bridge
- merge planning and merge execution authority remain in `worth-relational`
- writeback safety, idempotence, causality, and replay authority remain in the
  bridge
- query workflow declarations must remain basis-explicit, route-explicit, and
  failure-explicit
- unsupported workflow, merge, or writeback families must fail typed and early
  rather than degrading into hidden host glue
- durable workflow continuation, persisted workflow artifacts, and restart-
  stable workflow resume remain out of scope for this milestone

Normative consequence:

- any implementation path that offers a "do the merge" convenience call without
  a query-owned declaration artifact is out of spec
- any implementation path that allows host code to choose relational strategy
  identity after query lowering is out of spec
- any implementation path that exposes bridge writeback declarations as raw
  query payload bags is out of spec
- any implementation path that infers post-merge inspection basis from ambient
  latest state is out of spec
- any implementation path that treats unsupported workflow families as a best-
  effort local fallback is out of spec

## Compile-Time Enforcement Policy

Milestone 5.5 must classify which workflow-lowering guarantees become
unrepresentable, uncompilable, or construction-time rejection.

`Unrepresentable` in public types:

- publicly constructible query workflow declarations that do not carry
  canonical query identity, workflow basis identity, declaration family, and
  target lower-authority family
- publicly constructible mutation-lowering artifacts that do not carry explicit
  relational strategy or merge-admission identity
- publicly constructible writeback-lowering artifacts that do not carry bridge
  writeback family identity, causality basis identity, and admission class
- publicly constructible conflict-inspection or post-merge result envelopes
  that erase merge-class identity, advisory-versus-authoritative identity, or
  basis identity
- publicly constructible workflow admission outcomes encoded as booleans,
  strings, or open-ended bags instead of closed declaration/admission families

`Uncompilable` through visibility and compile-fail enforcement:

- external construction of `QueryWorkflowDeclaration`,
  `LoweredMutationIntentDeclaration`,
  `LoweredMergeWorkflowDeclaration`,
  `QueryConflictInspectionArtifact`,
  `QueryPostMergeInspectionArtifact`,
  `QueryWritebackDeclaration`, or materially equivalent proof-bearing types
  without crate-owned lowering
- public APIs that accept raw relational merge plans, raw relational mutation
  intents, raw bridge writeback declarations, or raw preview-session bags as
  though they were admitted query workflow inputs
- public APIs that let consumers override lower-authority family, merge-class,
  writeback-family, or admissibility outcome after declaration lowering
- public conversion paths that bypass canonical query planning and mint
  workflow-lowered artifacts directly from host-local controller data

`Construction-time rejection`:

- unsupported query families requested for mutation/merge/writeback workflows
- unsupported preview-basis or correspondence/history pairings for a workflow
  declaration
- unsupported conflict-inspection families
- unsupported post-merge inspection families
- unsupported relational strategy families
- unsupported merge classes for query-authored merge intent
- unsupported bridge writeback families or causality classes
- invalid basis-pairing between workflow declaration and lower-authority target
- invalid attempts to request authoritative merge/writeback behavior from
  advisory-only correspondence or preview-read-only contexts

Rules:

- the strongest available boundary must be used
- workflow declaration and lowering types must use sealed constructors and
  private fields
- compile-fail coverage is required for:
  - no raw lower-crate artifact as public workflow declaration input
  - no external construction of workflow-lowered proof types
  - no host override of target authority family
  - no bool-driven writeback or merge admission switches
- runtime rejection is allowed only for facts genuinely unavailable until lower
  runtimes report current merge admission, preview lifecycle, strategy
  compatibility, or writeback-family compatibility

## Scope

### In Scope

- query-owned workflow declarations for:
  - mutation intent lowering
  - conflict inspection
  - merge intent
  - post-merge inspection
  - query-triggered writeback declaration
- lowering from query workflow declarations into:
  - relational commit-strategy or merge-execution request surfaces
  - bridge writeback declaration surfaces
- explicit workflow context bundles that preserve:
  - query digest
  - basis identity
  - preview/comparison linkage where applicable
  - correspondence/history honesty where applicable
- query-shaped result families for conflict inspection and post-merge
  inspection
- typed diagnostics, counters, replay bundles, and rejection surfaces for
  workflow declaration admission and lowering
- milestone-native certification for workflow lowering and authority-boundary
  honesty

### Explicitly Out Of Scope

- new relational commit semantics, merge ontology, or invariant rules
- new bridge writeback-family semantics, idempotence rules, or loop-prevention
  semantics
- durable workflow continuation, persisted workflow artifacts, or restart-
  stable workflow resume
- store-backed workflow replay or store-backed writeback portability
- policy masking and tenant-schema composition beyond preserving clean seams for
  later milestones
- unified daily-driver facade and unified runtime configuration closure, which
  remain Milestone 5.6 work
- arbitrary host workflow UI state, transport framing, or controller lifecycle

### Initial Admission Matrix

Milestone 5.5 must not leave workflow support ambient.

Initial workflow-declaration-admitted query families:

- detail queries already admitted for runtime-backed execution
- ordered collection queries already admitted for runtime-backed execution
- bounded materialization queries already admitted for runtime-backed execution
- preview- and correspondence-aware inspection flows only where they reuse
  already-admitted Milestone 5.2 and 5.4 basis/evidence artifacts rather than
  redefining them

Initial mutation-lowering-admitted families:

- query-authored declarations that lower into already-admitted relational
  commit-strategy or merge-execution request families
- merge intent declarations whose branch/workflow basis can be tied to explicit
  preview/compare or branch-head proof artifacts

Initial writeback-lowering-admitted families:

- query-triggered writeback declarations that can lower into one explicit
  bridge-owned writeback declaration family without host-authored safety logic
- writeback declarations whose causality basis and source query basis can be
  frozen at lowering time

Initial denied families:

- host-authored "custom workflow" bags with no canonical query digest
- raw relational mutation batching through query
- raw bridge writeback execution through query
- workflows that require durable saved workflow artifacts
- workflows that require policy masking or tenant-schema variation to decide
  legality
- workflows that depend on unsupported merge classes or unsupported
  writeback-family admission

Any family not named above is out of scope for Milestone 5.5 and must fail
typed and early rather than becoming implicit beta support.

### Initial Authority Target Matrix

Milestone 5.5 must make the initial lower-authority targets tangible rather
than leaving them as generic "some relational or bridge surface."

Initial relational authority targets admitted at the query seam:

- one explicit relational commit-strategy declaration family already admitted
  by `worth-relational`
- one explicit pairwise merge declaration family that lowers into the
  authoritative merge execution path defined by
  [milestone-7c-authoritative-merge-execution-spec.md](../worth-relational/milestone-7c-authoritative-merge-execution-spec.md)
- merge-declaration families whose executable-vs-denied merge-class outcome is
  already explicit under 7C/7D

Initial relational denials:

- arbitrary raw mutation batching
- host-selected commit-strategy descriptors
- merge declarations that rely on unsupported deletion or topology classes to
  succeed
- merge declarations that erase target/source head identity or merge-base
  identity

Initial bridge authority targets admitted at the query seam:

- one explicit query-triggered writeback declaration family whose target
  writeback family is already admitted by the bridge
- one explicit preview-lifecycle-linked workflow declaration family where the
  bridge remains the owner of session lifecycle and replay artifacts

Initial bridge denials:

- direct execution of bridge writeback from query
- host-authored causality or idempotence basis
- preview-read-only contexts attempting to author writeback or merge authority
  requests
- bridge writeback declarations whose source query/evaluation context cannot
  produce one closed causality basis

## Query Workflow Architecture

### One Workflow Boundary

Milestone 5.5 extends the existing proof chain. It must not create a second
application workflow runtime beside canonical query planning and lower-crate
authorities.

The authoritative flow becomes:

`ValidatedQueryBundle`
-> `ExecutionPlanBundle` / admitted live or preview basis artifacts where
applicable
-> `QueryWorkflowDeclaration`
-> `WorkflowContextBinding`
-> `WorkflowAdmissionReport`
-> one of:
  - `LoweredMutationIntentDeclaration`
  - `LoweredMergeWorkflowDeclaration`
  - `QueryConflictInspectionArtifact`
  - `QueryPostMergeInspectionArtifact`
  - `QueryWritebackDeclaration`
-> lower-crate authority input
-> query-shaped authority outcome / inspection result

The declaration boundary therefore consumes already-proven query meaning and
already-proven basis meaning. It does not re-author:

- preview lifecycle semantics
- correspondence authority semantics
- historical materialization semantics
- merge ontology
- writeback-family protocol semantics

### Authority Boundaries

`worth-query` owns:

- workflow declaration vocabulary
- workflow context binding from canonical query and basis artifacts
- lowering into lower-authority request families
- query-shaped conflict-inspection and post-merge inspection surfaces
- workflow diagnostics, parity bundles, and closeout certification

`worth-relational` owns:

- commit strategy semantics
- merge planning/execution semantics
- merge-class ontology
- branch-head advancement
- invariant enforcement
- canonical commit publication

The runtime bridge owns:

- preview-session lifecycle authority
- writeback-family protocol meaning
- writeback causality, idempotence, and loop-prevention semantics
- replay-safe writeback and preview records

Hosts and application glue may own:

- transport of declaration requests
- presentation of inspection or outcome artifacts
- UI sequencing around already-lowered workflow results

Hosts and application glue may not own:

- deciding merge legality
- inventing relational strategy identity
- inventing writeback family or safety semantics
- fabricating workflow basis identity
- repairing unsupported declarations through silent local fallback

### Workflow Declaration Families

Milestone 5.5 must introduce one closed vocabulary for query-authored workflow
intent.

Representative artifact families:

- `QueryWorkflowDeclaration`
- `WorkflowDeclarationFamily`
- `WorkflowContextBinding`
- `WorkflowAdmissionReport`
- `QueryWorkflowBasis`
- `LoweredMutationIntentDeclaration`
- `LoweredMergeWorkflowDeclaration`
- `QueryConflictInspectionArtifact`
- `QueryPostMergeInspectionArtifact`
- `QueryWritebackDeclaration`
- `WorkflowLoweringFailure`
- `WorkflowLoweringCounters`

Required proof-stage families:

- `QueryWorkflowDeclaration`
- `AdmittedQueryWorkflowDeclaration`
- `LoweredAuthorityWorkflowRequest`
- `WorkflowAuthorityOutcomeArtifact`

Rules:

- every workflow declaration is anchored to one canonical query digest and one
  explicit basis family
- every declaration names one lower-authority target family
- every declaration lowers into exactly one admitted authority family or one
  typed denial
- an admitted declaration is not yet a lowered authority request
- a lowered authority request is not yet an authority outcome artifact
- conflict inspection and post-merge inspection remain query result families,
  not generic diagnostics passthrough
- writeback declarations remain declarations; the bridge still owns execution
  and final authority transfer

### Workflow Basis And Context Identity

Milestone 5.5 must preserve the earlier milestone rule that query execution is
basis-honest.

Representative artifact families:

- `WorkflowSourceQueryDigest`
- `WorkflowBasisDigest`
- `WorkflowContextBinding`
- `WorkflowPreviewLinkage`
- `WorkflowComparisonLinkage`
- `WorkflowAuthorityTargetDigest`
- `WorkflowCausalityDigest`
- `WorkflowFreshnessPolicy`
- `WorkflowFreshnessBinding`
- `WorkflowStalenessOutcome`

Rules:

- a workflow declaration must carry the same query identity that produced the
  inspection or context from which the workflow was authored
- preview-bound declarations must carry explicit preview basis identity and
  preview evaluation class
- comparison-bound declarations must carry explicit basis-pair identity
- merge declarations must carry explicit target/source basis linkage, not just
  branch names in a host object
- writeback declarations must carry explicit causality linkage to the source
  query/evaluation context
- workflow lowering must carry one explicit freshness policy rather than
  assuming that declaration time and authority-admission time are equivalent
- fresh inspection parity for certification must compare against the declared
  workflow basis, not ambient latest state

### Workflow Freshness And Staleness

Milestone 5.5 must not let query-authored workflow declarations silently drift
between declaration time and lower-authority admission time.

Representative artifact families:

- `WorkflowFreshnessPolicy`
- `WorkflowFreshnessBinding`
- `WorkflowStalenessOutcome`
- `WorkflowExplicitRebindArtifact`

Rules:

- every admitted workflow declaration must carry one explicit freshness policy
- merge and mutation declarations must preserve the lower-authority freshness
  requirements of the relational target they lower into
- writeback declarations must preserve the lower-authority freshness and
  causality requirements of the bridge target they lower into
- stale workflow declarations must produce one closed outcome family:
  - `StillFresh`
  - `StaleDenied`
  - `ExplicitRebindRequired`
- no supported path may reinterpret stale workflow declarations as fresh by
  silently reading new branch heads, new preview lifecycle state, or new
  causality basis
- hosts may present rebind UX, but the rebind artifact must be query-owned and
  explicit rather than ambient retry behavior

### Conflict Inspection And Post-Merge Inspection

Milestone 5.5 must not treat workflow inspection as raw lower-crate debug
output.

Representative artifact families:

- `QueryConflictInspectionArtifact`
- `ConflictInspectionFamily`
- `ConflictInspectionRow`
- `QueryPostMergeInspectionArtifact`
- `PostMergeInspectionFamily`
- `PostMergeInspectionRow`

Minimum required `ConflictInspectionRow` fields:

- `workflow_basis_digest`
- `merge_class`
- `merge_class_admission`
- `target_basis_digest`
- `source_basis_digest`
- `conflict_scope_digest`
- `authority_target_family`

Minimum required `PostMergeInspectionRow` fields:

- `authoritative_outcome_basis_digest`
- `authority_target_family`
- `authoritative_commit_or_outcome_digest`
- `post_merge_scope_digest`
- `merge_or_writeback_origin_digest`
- `inspection_result_family`

Rules:

- conflict inspection is a query-shaped result family over admitted merge-class
  and branch-workflow evidence
- the result must preserve explicit merge-class identity and executable-versus-
  denied meaning
- post-merge inspection is a query-shaped result family over authoritative
  outcomes and explicit post-merge basis identity
- conflict inspection may not expose one naked diagnostics payload accessor
  that bypasses the row contract above
- post-merge inspection may not expose one naked "latest state" accessor that
  bypasses the authoritative outcome basis
- neither family may erase preview/linkage/correspondence/path metadata that is
  load-bearing for the workflow story

### Mutation And Merge Lowering

Milestone 5.5 must keep mutation/merge lowering declaration-owned by query and
authority-owned by `worth-relational`.

Representative artifact families:

- `LoweredMutationIntentDeclaration`
- `MutationIntentFamily`
- `RelationalStrategyTarget`
- `LoweredMergeWorkflowDeclaration`
- `MergeWorkflowIntent`
- `MergeAuthorityTarget`
- `MergeConflictInspectionAdmission`

Rules:

- query lowers context and declaration into relational request families; it does
  not synthesize mutation batches directly
- merge declarations may target only admitted merge authority families
- conflict inspection may succeed even when merge execution would later deny;
  the result must remain explicit about that distinction
- post-merge inspection may succeed only over authoritative merge outcomes, not
  over preview guesses or host-local "expected merge" state

### Writeback Lowering

Milestone 5.5 must keep writeback lowering declaration-owned by query and
protocol-owned by the bridge.

Representative artifact families:

- `QueryWritebackDeclaration`
- `WritebackDeclarationFamily`
- `WritebackCausalityBinding`
- `LoweredBridgeWritebackTarget`
- `WritebackAdmissionReport`

Rules:

- query-triggered writeback starts from canonical query/evaluation context and
  lowers into one admitted bridge writeback declaration family
- query may not execute writeback or skip bridge writeback admission
- query may not reduce writeback identity to "same query, write it back"
  folklore; causality, family identity, and authority target must remain
  explicit
- unsupported writeback-family or causality combinations must deny typed and
  early

## Performance Architecture

### Workflow Cost Must Be Admission-Owned

Milestone 5.5 must not treat performance as a telemetry concern added after
the workflow surface already exists. The workflow architecture itself must
encode which cost posture was admitted and which broadening behaviors are
forbidden.

Representative artifact families:

- `WorkflowCostClass`
- `WorkflowBudgetClass`
- `WorkflowInspectionBudget`
- `WorkflowLoweringBudget`
- `WorkflowWritebackBudget`
- `WorkflowBudgetOutcome`
- `WorkflowPredictionReport`

Rules:

- every admitted workflow declaration family must carry one explicit cost class
  and one explicit budget class
- inspection families and authority-lowering families may not share one vague
  "workflow cost" bucket if their breadth surfaces differ materially
- the planner or lowering phase must attach cost posture before execution or
  lower-authority admission begins
- execution and authority outcome bundles must report realized cost against the
  admitted budget class rather than only raw counters
- unsupported or over-budget workflow shapes must deny or require explicit
  rebind/replanning rather than silently broadening

### Required Workflow Cost Surfaces

Milestone 5.5 must encode cost in terms of the real traversal and authority
surfaces this milestone introduces, not just elapsed time.

Required architectural cost surfaces:

- declaration width:
  how many query clauses, basis links, and authority-target checks are required
  to admit the workflow declaration
- inspection width:
  how many conflict or post-merge rows, merge classes, and basis-linked
  evidence surfaces may be materialized
- lowering width:
  how many lower-authority request records, strategy descriptors, merge-class
  links, or causality bindings are emitted
- freshness width:
  how many branch-head, preview-lifecycle, promotion-linkage, or causality
  checks are required before authority admission
- denial width:
  how many denied classes or denied authority-target families may be surfaced
  before the workflow fails closed

These widths must be represented as architectural facts, not inferred later
from logs.

### Budget-Bearing Workflow Families

Milestone 5.5 should freeze the first explicit budget-bearing workflow families.

Initial budget-bearing inspection families:

- `conflict-inspection-narrow`:
  bounded by one admitted merge-family scope and one explicit query basis
- `post-merge-inspection-narrow`:
  bounded by one authoritative outcome basis and one explicit result-family
  scope

Initial budget-bearing lowering families:

- `mutation-lowering-narrow`:
  bounded by one declared relational strategy target and one explicit workflow
  basis
- `merge-lowering-narrow`:
  bounded by one admitted pairwise merge target plus explicit target/source
  basis proof
- `writeback-lowering-narrow`:
  bounded by one admitted bridge writeback family and one explicit causality
  binding

Any workflow family whose cost posture would require:

- broad branch rescans
- broad merge-class rediscovery
- broad retained preview-session searches
- broad causality reconstruction
- host-local result diff reconstruction

must fail typed and early rather than being admitted as a cheap workflow lane.

### Prediction Drift Must Be Explicit

Milestone 5.5 must not let implementations claim bounded workflow cost while
quietly exceeding that bound during lower-authority admission or inspection.

Representative artifact families:

- `WorkflowPredictionReport`
- `WorkflowPredictionDriftOutcome`

Required `WorkflowPredictionDriftOutcome` classes:

- `WithinBudget`
- `ExplicitBroadeningDenied`
- `ExplicitRebindRequired`

Rules:

- predicted workflow width belongs to the admitted declaration or inspection
  artifact
- realized workflow width belongs to the result or denial artifact
- if realized width exceeds admitted budget, the outcome must become one typed
  drift class rather than one passive counter increment
- no admitted path may continue after broadening silently changed the workflow
  cost posture

### No Cheap-Looking Authority Surfaces

Milestone 5.5 must make workflow APIs honest about the cost they can induce.

The architecture must forbid:

- one generic `execute_workflow(...)` surface that hides inspection-versus-
  lowering breadth
- one generic `merge()` or `writeback()` helper whose signature conceals
  branch-head checks, merge-class admission, or causality checks
- one generic "inspect conflicts" surface that can materialize arbitrarily wide
  diagnostics without an explicit budget-bearing family

If an API can trigger authority-target compatibility checks, merge-class
admission, freshness checks, and query-shaped row materialization, its type
surface must make that orchestration boundary explicit.

## Phases

### Phase 1: Freeze Workflow Declaration, Basis, And Authority Taxonomy

Phase 1 exists to stop workflow orchestration from becoming host-local glue.

Milestone 5.5 must first define:

- the closed vocabulary for query-authored workflow declaration families
- the closed vocabulary for workflow basis and authority-target families
- the distinction between:
  - inspection declarations
  - mutation declarations
  - merge declarations
  - writeback declarations
- proof-bearing context binding between canonical query artifacts and workflow
  declaration artifacts
- typed denial classes for unsupported workflow family, unsupported authority
  target, invalid basis pairing, and invalid preview/comparison/writeback use
  of a given query context

This phase leaves the system in a coherent state where:

- query workflow intent is a first-class artifact family
- workflow declarations carry basis identity and target-authority identity
- illegal host-local shortcuts are blocked before any lower-authority lowering
  begins

### Phase 2: Lower Query Workflow Declarations Into Lower-Authority Requests

Phase 2 exists to keep workflow lowering honest and one-way.

Milestone 5.5 must then implement:

- lowering from `QueryWorkflowDeclaration` plus `WorkflowContextBinding` into:
  - `LoweredMutationIntentDeclaration`
  - `LoweredMergeWorkflowDeclaration`
  - `QueryWritebackDeclaration`
- explicit authority-target compatibility checks
- explicit denial for unsupported relational strategy, merge class, or
  writeback family
- explicit cost-class and budget-class attachment to every lowered authority
  request family
- exact basis/causality preservation from source query context into lowered
  authority request families
- explicit freshness binding and staleness classification for every lowered
  authority request family
- exact counters for declaration admission, lowering width, and denial classes

This phase leaves the system in a coherent state where:

- query owns declaration and lowering
- lower crates still own actual authority semantics
- unsupported workflow combinations fail before authority execution begins

### Phase 3: Query-Shaped Conflict Inspection And Post-Merge Inspection

Phase 3 exists to keep workflow inspection in the query domain rather than
dropping developers into raw lower-crate artifact bags.

Milestone 5.5 must then implement:

- query-shaped conflict-inspection result families
- query-shaped post-merge inspection result families
- explicit preservation of merge-class identity, basis identity, and
  advisory-versus-authoritative distinctions where applicable
- explicit inspection-budget classes and drift outcomes for both inspection
  families
- typed denial for unsupported inspection families
- exact counters for inspection row width, merge-class width, and denial width

This phase leaves the system in a coherent state where:

- conflict inspection is query-native instead of raw diagnostics passthrough
- post-merge inspection is query-native instead of ambient branch rereads
- workflow UI/app code can stay inside query-shaped artifacts

### Phase 4: Authority Outcome Shaping And Replay-Safe Workflow Bundles

Phase 4 exists to make authority transfer visible and certifiable.

Milestone 5.5 must then implement:

- query-shaped outcome bundles for:
  - admitted mutation lowering
  - admitted merge lowering
  - admitted writeback declaration lowering
  - conflict inspection
  - post-merge inspection
- explicit reporting of admitted cost class, admitted budget class, realized
  width, and prediction-drift outcome
- explicit reporting of freshness success, staleness denial, or explicit rebind
  requirement
- replay-safe workflow bundles carrying canonical digests for query, plan,
  result, delivery/failure where applicable, and counters
- deterministic reporting of authority-target admission, denial, and outcome
  families

This phase leaves the system in a coherent state where:

- workflow lowering and inspection can be certified from canonical artifacts
- authority transfer is explicit rather than reconstructed from host code
- later unified-facade work has honest capability metadata to surface

### Phase 5: Certification, Counter Proof, And Boundary Hardening

Phase 5 exists to close the milestone through proof rather than "query can now
do workflows" demos.

Milestone 5.5 must finally ship:

- the `Query Workflow Lowering And Writeback Boundary Test`
- canonical rows proving:
  - query-authored mutation lowering parity
  - query-authored merge lowering parity
  - conflict inspection explicitness
  - post-merge inspection explicitness
  - query-triggered writeback lowering parity
- workflow authority-boundary explicitness
- workflow freshness explicitness
- rejection rows proving:
  - unsupported workflow family
  - unsupported merge family
  - unsupported writeback family
  - raw lower-crate artifact workflow input forbidden
  - host authority override forbidden
  - ambient basis fallback forbidden
  - stale-workflow-denied
  - explicit-rebind-required
  - unsupported-deletion-topology-merge-class
  - preview-read-only-authority-request-forbidden
- compile-fail or privacy hardening proving workflow-lowered proof types cannot
  be WORTHd externally

This phase leaves the system in a coherent state where:

- query workflow lowering is certifiable rather than aspirational
- Milestone 5.6 can surface one coherent facade over already-honest workflow
  surfaces
- later policy/tenant/store milestones can compose with workflow declarations
  instead of redefining them

## Must Ship

- proof-bearing `QueryWorkflowDeclaration`, `WorkflowContextBinding`,
  `LoweredMutationIntentDeclaration`, `LoweredMergeWorkflowDeclaration`,
  `QueryConflictInspectionArtifact`, `QueryPostMergeInspectionArtifact`, and
  `QueryWritebackDeclaration` families or materially equivalent types
- query-owned workflow declaration vocabulary for mutation, merge, conflict
  inspection, post-merge inspection, and writeback
- lowering from admitted query workflow declarations into relational and bridge
  authority request families
- query-shaped conflict-inspection and post-merge inspection result families
- explicit workflow basis, authority-target, and causality identity on all
  admitted workflow paths
- one dedicated workflow-lowering performance subdomain owning counters and
  contract status rather than generic telemetry-only logging
- typed workflow diagnostics, replay bundles, and exact counters
- milestone-native certification proving workflow-lowering parity,
  authority-boundary honesty, and rejection behavior
- one representative scenario matrix binding admitted declaration families to
  concrete lower-authority targets and denial classes

## Must Preserve

- canonical query meaning from Milestone 1 remains authoritative
- validation legality from Milestone 2 remains authoritative
- proof-bearing planning and basis identity from Milestone 3 remain
  authoritative
- collection/result-family semantics from Milestone 4 remain authoritative
- live/locality semantics from Milestones 5 and 5.1 remain authoritative where
  workflow inspection composes with those lanes
- preview-session basis identity and preview comparison surfaces from Milestone
  5.2 remain authoritative where workflow declarations originate from preview
  or compare contexts
- frontier posture from Milestone 5.3 remains authoritative where workflow
  inspection composes with planned route posture
- correspondence and historical-path honesty from Milestone 5.4 remain
  authoritative where conflict or post-merge inspection consumes those
  artifacts
- `worth-relational` remains authoritative for commit strategy, merge
  semantics, branch-head advancement, invariant enforcement, and canonical
  publication
- the runtime bridge remains authoritative for preview lifecycle, writeback
  safety, idempotence, causality, and replay
- conflict inspection and post-merge inspection remain query-shaped rather than
  raw lower-crate diagnostics passthrough
- unsupported workflow families fail typed and early rather than degrading into
  host-local glue

## Complexity / Proof Obligations

Milestone 5.5 must name costs and proofs in terms of:

- workflow declaration admission count
- workflow basis binding width
- workflow authority-target compatibility width
- mutation-lowering count
- merge-lowering count
- conflict-inspection row width
- post-merge inspection row width
- writeback-declaration count
- writeback causality-binding width
- workflow predicted width
- workflow realized width
- workflow budget cross count
- workflow broadening denial count
- workflow denial count
- workflow_staleness_check_count
- workflow_stale_denial_count
- workflow_explicit_rebind_required_count
- work avoided by query-owned workflow lowering versus host rediscovery
- executor rediscovery avoidance on workflow-lowered paths

Minimum required counters:

- `workflow_declaration_count`
- `workflow_basis_binding_count`
- `workflow_basis_binding_width`
- `workflow_authority_target_check_count`
- `workflow_mutation_lowering_count`
- `workflow_merge_lowering_count`
- `workflow_conflict_inspection_count`
- `workflow_conflict_inspection_row_width`
- `workflow_post_merge_inspection_count`
- `workflow_post_merge_inspection_row_width`
- `workflow_writeback_declaration_count`
- `workflow_writeback_causality_binding_count`
- `workflow_predicted_width`
- `workflow_realized_width`
- `workflow_budget_cross_count`
- `workflow_broadening_denial_count`
- `workflow_denial_count`
- `workflow_merge_denial_count`
- `workflow_writeback_denial_count`
- `workflow_staleness_check_count`
- `workflow_stale_denial_count`
- `workflow_explicit_rebind_required_count`
- `workflow_authority_override_denial_count`
- `workflow_ambient_basis_fallback_denial_count`
- `workflow_replay_bundle_count`
- `workflow_work_avoided_by_query_lowering_count`
- `workflow_executor_rediscovery_count`

Rules:

- counters belong to workflow result bundles, denial bundles, and
  certification bundles
- representative certification scenarios must assert exact counts
- `workflow_executor_rediscovery_count` must be exactly zero on every admitted
  path
- every denied workflow declaration must increment `workflow_denial_count`
- every denied merge lowering must increment `workflow_merge_denial_count`
- every denied writeback lowering must increment
  `workflow_writeback_denial_count`
- every freshness check must increment `workflow_staleness_check_count`
- every stale denial must increment `workflow_stale_denial_count`
- every explicit rebind requirement must increment
  `workflow_explicit_rebind_required_count`
- every workflow width prediction must record `workflow_predicted_width`
- every realized workflow lane must record `workflow_realized_width`
- every admitted budget crossing must increment `workflow_budget_cross_count`
- every denied broadening attempt must increment
  `workflow_broadening_denial_count`
- every forbidden host override must increment
  `workflow_authority_override_denial_count`
- every forbidden ambient basis fallback must increment
  `workflow_ambient_basis_fallback_denial_count`
- no supported path may hide authority transfer or denial inside generic
  success counters
- "work avoided" counters must make explicit that query-owned lowering avoided
  host-local workflow rediscovery and glue

Minimum certification rows should include:

- `workflow-mutation-lowering-parity`
- `workflow-merge-lowering-parity`
- `workflow-conflict-inspection-explicitness`
- `workflow-post-merge-inspection-explicitness`
- `workflow-writeback-lowering-parity`
- `workflow-authority-boundary-explicitness`
- `workflow-freshness-explicitness`
- `workflow-budget-explicitness`
- `workflow-work-avoided-counter-parity`

Minimum rejection rows should include:

- `unsupported-workflow-family`
- `unsupported-merge-family`
- `unsupported-writeback-family`
- `raw-lower-crate-workflow-input-forbidden`
- `host-authority-override-forbidden`
- `ambient-basis-fallback-forbidden`
- `stale-workflow-denied`
- `explicit-rebind-required`
- `forbidden-workflow-broadening`
- `preview-read-only-authority-request-forbidden`
- `advisory-only-authority-request-forbidden`
- `unsupported-deletion-topology-merge-class`

## Allowed Debt

- some workflow families may remain unsupported as explicit `Debt` while
  admitted families are fully parity-proven
- richer conflict-inspection or post-merge inspection views may remain `Debt`
  if admitted inspection families are explicit, typed, and certified
- broader writeback-family coverage may remain `Debt` if admitted writeback
  declaration families already preserve bridge-owned authority boundaries and
  are certified
- durable workflow continuation and persisted workflow artifacts may remain
  blocked on later milestones
- host-local workflow glue for any admitted family may not exist as debt
- raw lower-crate artifact passthrough presented as query-native workflow
  support may not exist as debt
- hidden authority override or ambient basis fallback may not exist as debt

## Acceptance Evidence

Milestone 5.5 is complete only when `worth-query` can prove:

- the `Query Workflow Lowering And Writeback Boundary Test` in
  [test-requirements.md](./test-requirements.md) passes with canonical
  machine-checkable artifacts
- admitted mutation and merge workflow declarations lower into relational
  authorities without semantic drift
- admitted writeback declarations lower into bridge authorities without hiding
  causality, idempotence, or safety semantics
- conflict inspection and post-merge inspection remain query-shaped and
  basis-explicit
- stale workflow declarations deny or require explicit rebind rather than
  silently drifting
- admitted workflow lanes remain within explicit budget classes or fail through
  typed drift outcomes
- unsupported workflow families fail typed and early
- workflow artifacts remain replay-safe and authority-boundary-honest

Required verification output must include:

- `query_digest`
- `plan_digest`
- `result_digest`
- `delivery_digest`
- `failure_digest`
- `counter_snapshot`

### Representative Scenario Matrix

Milestone 5.5 certification should exercise at minimum:

- `preview-compare-to-merge-intent`:
  one preview/comparison-bound query authors one merge declaration that lowers
  into one admitted pairwise relational merge target
- `merge-conflict-inspection-with-denied-class`:
  one conflict inspection lane surfaces an unsupported deletion or topology
  merge class explicitly rather than collapsing it into generic conflict text
- `post-merge-authoritative-inspection`:
  one admitted merge outcome is inspected through one explicit authoritative
  outcome basis rather than ambient branch reread
- `detail-query-to-writeback-declaration`:
  one detail or bounded-materialization query authors one admitted bridge
  writeback declaration with explicit causality binding
- `preview-read-only-writeback-denied`:
  one preview-read-only workflow attempts to author writeback and fails typed
  and early
- `stale-workflow-rebind-required`:
  one workflow declaration becomes stale before authority admission and emits
  one explicit rebind requirement instead of best-effort continuation
- `workflow-budget-cross-denied`:
  one hostile lane exceeds the admitted inspection or lowering budget and fails
  through one explicit broadening denial instead of silently widening
- `host-override-forbidden`:
  one hostile lane tries to replace the lowered authority target after
  declaration lowering and fails typed

If the harness cannot name concrete lanes at this granularity, the milestone is
still too abstract to close honestly.

## Architectural Notes

### Query Must Orchestrate, Not Mutate

The core rule in this milestone is simple:

- query authors workflow intent
- lower crates decide mutation and authority truth

If `worth-query` starts inventing mutation batches, merge legality, or
writeback-family semantics on its own, it has crossed the line from workflow
framework into shadow authority.

### Conflict Inspection Is Not Raw Diagnostics

Conflict inspection only counts as a real query feature if it stays query-
shaped.

That means:

- rows have typed meaning
- basis identity is explicit
- merge-class identity is explicit
- executable-versus-denied meaning is explicit

It must not become:

- raw planner rows
- raw denial bags
- whatever diagnostics payload happened to be available at the lower boundary

### Post-Merge Inspection Must Stay Outcome-Shaped

Post-merge inspection is not "read latest branch head and hope it lines up."

The required rule is:

- the post-merge inspection bundle must point at one explicit authoritative
  outcome basis
- the result must preserve the authority outcome it is inspecting

Otherwise the most important workflow-facing read after merge becomes ambient
state lookup instead of a typed workflow result.

### Writeback Lowering Must Not Smuggle Bridge Authority Upward

Query-triggered writeback is where the architecture could most easily blur.

The required rule is:

- query owns declaration
- the bridge owns writeback protocol meaning and execution

If query code starts deciding idempotence, causality equivalence, or
loop-prevention semantics locally, Milestone 5.5 has stolen Milestone 12
bridge authority instead of composing with it.

### This Milestone Must Not Steal 5.6

Milestone 5.5 is about honest workflow lowering and workflow result shaping.
Milestone 5.6 is about surfacing the unified daily-driver facade.

5.5 must therefore stop at:

- declaration
- lowering
- inspection
- authority-boundary bundles
- certification

It must not drift into:

- bag-shaped convenience APIs
- flattened cross-subsystem configuration
- broad capability-advertisement design that belongs to 5.6

## Sequencing Notes

Milestone 5.5 belongs after Milestones 5.2 through 5.4 because preview basis,
route posture, and correspondence/history honesty all need to be frozen before
workflow declarations can carry them honestly across authority boundaries.

It belongs before Milestone 5.6 because the unified application facade should
surface workflow orchestration only after mutation/merge/writeback lowering is
already authority-preserving and certified.

It also belongs before Milestones 6 through 9 because those later basis,
lineage, view-shape, and policy surfaces should compose on top of an existing
workflow-declaration boundary rather than inventing their own mutation or
writeback escape hatches.

## Parallelization Notes

Once the workflow declaration and authority-target taxonomy is frozen:

- Milestone 5.6 unified-facade/config work can proceed in parallel without
  changing workflow-lowering meaning
- later basis/policy milestones can design composition points against explicit
  workflow artifacts instead of host-local glue
- compile-time tightening, counter hardening, and certification row expansion
  can proceed in parallel without changing milestone semantics

## Explicit Failure Taxonomy For Milestone 5.5

- unsupported workflow declaration family
- invalid workflow basis binding
- unsupported relational strategy target
- unsupported merge family
- unsupported conflict-inspection family
- unsupported post-merge inspection family
- unsupported writeback family
- workflow authority-target incompatibility
- host authority override attempt
- ambient basis fallback attempt
- raw lower-crate artifact passthrough attempt
- workflow replay divergence
- workflow artifact invariant break

## Anti-Patterns Explicitly Rejected

- `worth-query` directly creating mutation batches
- `worth-query` directly executing bridge writeback
- conflict inspection implemented as raw lower-crate diagnostics passthrough
- post-merge inspection implemented as ambient branch-head reread
- host-local workflow objects with no canonical query digest or basis identity
- hidden fallback from unsupported workflow declarations into controller glue
- public construction of workflow-lowered proof types without the proving path
- booleans like `can_merge` or `should_writeback` standing in for closed
  declaration/admission families

## Self-Check

This milestone solves a real structural problem rather than packaging work
cosmetically because it closes the biggest remaining platform gap between
query-native inspection and authority-bearing workflow execution.

The adversarial constraint is load-bearing because it forbids the naive failure
mode where preview/compare/query context is canonical on the read side and then
disappears into raw lower-crate calls or host glue the moment the user wants
to act.

The milestone preserves authority boundaries because `worth-query` owns
declaration, context binding, lowering, and result shaping while
`worth-relational` and the bridge remain the only authorities for merge and
writeback truth.

The milestone defines proof obligations rather than implementation chores
because lowering parity, inspection explicitness, denial behavior, replay-safe
bundles, and exact counters are required for closeout.

A competent engineer should be able to map this spec into honest workflow
declaration, lowering, inspection, certification, and compile-fail modules
without inventing the architecture during implementation.

This milestone belongs at 5.5 because it is the workflow-boundary hardening
layer that must exist before `worth-query` can honestly present itself as the
daily-driver application framework surface in Milestone 5.6.

## Closeout Standard

Milestone 5.5 is complete only when all of the following are true:

- admitted query families can author workflow declarations without leaving the
  canonical query proof chain
- mutation and merge declarations lower into relational authorities without
  semantic drift
- writeback declarations lower into bridge authorities without stealing
  writeback semantics into query
- conflict inspection and post-merge inspection are query-shaped, basis-explicit,
  and authority-boundary-honest
- unsupported workflow families fail typed and early
- certification bundles prove workflow-lowering parity and denial through
  canonical machine-checkable artifacts

If code lands but workflow support still depends on raw lower-crate artifact
bags, host authority overrides, ambient basis fallback, direct query mutation,
or direct query writeback execution, Milestone 5.5 is not complete.
