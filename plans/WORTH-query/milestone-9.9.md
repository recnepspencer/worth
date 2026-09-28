# Milestone 9.9 Engineering Spec: Graph Touch Obligation Authority

> **Status:** Completed.
>
> **Closeout:** [milestone-9.9-closeout.md](./milestone-9.9-closeout.md)
>
> **Roadmap parent:** [worth_query_roadmap.md](./worth_query_roadmap.md)
>
> **Primary predecessors:** [milestone-9.8.md](./milestone-9.8.md), [milestone-9.7.md](./milestone-9.7.md), [milestone-9.6.md](./milestone-9.6.md)
>
> **Purpose:** establish graph touch obligation authority as a complete Query
> boundary — pure index-backed obligation selection, explicit dispatch plans,
> executor contracts, typed verdicts, canonical envelopes on receipts and
> decision traces, relational execution bridge, duplicate-rule elimination, and
> mechanical consumer anti-folklore — certified architecturally and proven by
> reference-consumer deletion of parallel legality.

## Goal

Make touch shape and operating world mechanically determine which obligations
execute, with typed verdicts and canonical dispatch artifacts on every covered
Query path that already carries equivalent read/mutation meaning. Host-local
legality graphs, manual invariant-pack closures, and caller-side "remember to
run this check" code must become unrepresentable on covered surfaces.

Investigation shows the real seams are not where the first draft assumed:

- **`write_graph_batch` already flows through authoritative mutation batch intent
  admission** with graph program and breadth in
  `WORTHQueryAuthoritativeMutationBatchIntentSeed` — obligation dispatch belongs
  there, not as a side callback before intent review.
- **WORTH Query already freezes its covered intent-admission floor in tests and
  docs** — authoritative mutation includes scalar write fronts,
  authoritative command-batch fronts, graph-composition fronts, and
  effect-triggered write-intent execution; read execution includes family,
  basis-context, and live-read paths. This spec must follow that shipped
  surface inventory rather than a narrower mental model.
- **`ComposeGraph` is declared for primitive construction but never executed**
  — kernel phase-chain legality is offline-only; worth-topo construction builds
  handoffs without `compose_graph`.
- **worth-topo operators already enter through configured domain handles and
  contribution orchestration** before graph composition or command submission
  executes — kernel construction does not.

This milestone builds the complete authority pipeline on those real seams.

## Why This Milestone Exists

Query exposes graph composition, relational invariant registration, policy
narrowing on read, configured domain handles, contribution orchestration,
intent admission for authoritative mutations, and commit-boundary execution —
as separate folklore lanes downstream code re-stitches by hand.

`worth-topo` calls graph composition and command-submission helpers yet
enforces loop wiring in three places. `worth-kernel` runs a full offline
phase-chain legality pipeline while declaring `ComposeGraph`, without
operating context and with motion sequencing guarded by `unreachable!`.

Per `MENTALITY.md` §2 and §14: build the foundation that survives the
adversarial constraint now; expand scope across crates until the blocker is
gone.

## Governing Summaries

- `MENTALITY.md`: enforce mechanically; scope expands until blockers are gone;
  debt is not a substitute for building the real path.
- `arch_laws.md`: Law 4, 5, 12, 17, 33 — shape determines checks; three-state
  verdicts; applicability pre-solved at entry; one truth, derived views.
- `composition_laws.md`: registration, indexing, dispatch, envelope, and denial
  projection remain distinct named surfaces.
- `domain_structure_laws.md`: relational executes invariant semantics; Query
  owns routing, touch vocabulary, and dispatch artifacts.
- `perf_laws.md`: dispatch selection is index lookup with named complexity
  contracts and proof tests.
- `worth_query_roadmap.md`: graph authoring through Query is dishonest while
  legality lives in consumer callbacks.

## Adversarial Constraint

For any registered obligation set, any admitted graph composition or
touch-bearing read/mutation artifact, and any operating world on the handle,
obligation **selection and planning** are pure functions of
`(touch_descriptor, operating_world_descriptor, obligation_index)` on every
covered lane that already exists in WORTH Query: authoritative scalar write,
authoritative command-batch write, graph-composition entry, effect-triggered
write-intent execution, declaration-entry orchestration where it gates
mutation, read-family execution, live-read execution, preview-local direct
mutation, and preview/branch intent execution where applicable.

Obligation **execution** is not pure in that same sense. Execution may require
state, basis, support posture, current policy, relational snapshots,
construction context, preview state, branch state, or commit-boundary truth.
Every executor must therefore declare its state/basis access contract before it
can produce a verdict. A final obligation envelope is replay-stable only when
the plan, execution inputs, execution statuses, verdict aggregation, and denial
projection are all canonical.

Certification proves exact-zero tolerance on covered surfaces for: false
negatives, false positives, duplicate rule implementations, manual pre-check
residue, dispatch-plan drift, obligation kinds without executable paths, and
mutation lanes that bypass dispatch.

## Product Decision Lock

- Every obligation kind has at least one real executor and every covered lane
  participates in obligation selection. Kind x lane support must be represented
  explicitly as `Supported`, `Unsupported`, `NotApplicable`,
  `DiagnosticOnly`, or `DeferredToBackstop`; fake no-op executors are
  forbidden.
- Dispatch integrates with the **authoritative mutation intent-admission
  family**. Graph composition anchors on authoritative mutation batch review
  with graph artifacts, while scalar-write, command-batch, explicit
  submission, and effect-triggered fronts must reuse the same touch vocabulary
  instead of becoming parallel obligation paths.
- Relational gains explicit graph-composition execution point; existing
  milestone-one registrations migrate from commit-only posture.
- Policy-aware graph mutation execution built so operating-context gates run
  on write authority.
- Primitive construction birth **executes** `compose_graph` from admitted
  handoffs — handoff-only receipts are not closure.
- Every dispatch materializes a canonical envelope; receipts and intent
  decision traces carry obligation evidence.
- Closure = architectural certification + full reference adoption in
  `worth-topo` and `worth-kernel` — not operator samples without birth
  execution.

## Terminology Lock

The word `batch` is overloaded across Query and Worth. This milestone must not
use it as an authority word. Every phase and test must name the exact surface
or semantic boundary it means.

- **Refactor batch**: the large implementation pause boundaries in this spec.
  These are project-management boundaries only. They are not runtime surfaces.
- **Authoritative mutation batch seam**: Query's canonical multi-command write
  authority path:
  `review_authoritative_runtime_write_batch_with_graph_artifacts(...)` →
  admitted handoff → execution binding →
  `execute_authoritative_mutation_batch_execution_binding(...)`.
  This is the central mutation seam for graph-composition write execution.
- **Graph composition entry**: `workspace.compose_graph(...)` and
  `workspace.compose_graph_with_invariant_pack(...)` as public authoring
  fronts. Graph composition lowers to `write_graph_batch(...)`, which already
  enters the authoritative mutation batch seam with graph program and breadth
  artifacts.
- **Command batch surface**: public or internal APIs that collect multiple
  `WORTHQueryWriteCommand` values, such as `runtime.write_batch(...)`,
  `runtime.write_batch_intent(...)`, `workspace.write_batch_intent(...)`,
  `workspace.submissions()?.submit_batch(...)`, and `workspace.batch(...)`.
  These are covered only when the command set yields a graph touch descriptor
  on a covered lane.
- **Scalar mutation surface**: public scalar mutation fronts such as
  `runtime.write(...)`, `runtime.write_intent(...)`, `workspace.insert(...)`,
  `workspace.update(...)`, `workspace.delete(...)`, and their existing-truth
  variants. These are not "batch" surfaces, but they must still participate in
  obligation authority when they carry covered graph touch meaning.
- **Effect-triggered write-intent surface**:
  `runtime.next_effect_write_intent(...)` and
  `admit_next_effect_write_intent_for_execution(...)`. This is not a parallel
  obligation authority; it is a front door that must lower into the same
  canonical touch vocabulary when it carries covered graph mutation meaning.
- **Preview-local direct mutation surface**:
  `WORTHQueryPreviewSession::write(...)` and
  `WORTHQueryPreviewSession::batch(...)`. These are non-authoritative fronts
  with preview operating-world descriptors. They must not be confused with the
  authoritative mutation batch seam.
- **Branch / preview intent execution surface**:
  `WORTHQueryBranchSession::execute_intent(...)` and
  `WORTHQueryPreviewSession::execute_intent(...)`. This milestone certifies
  these intent-first fronts where they carry covered graph touch meaning; it
  does not invent a branch-local batch-write API.
- **Worth topology operator batch**: a consumer/operator implementation shape
  in `worth-topo`, often meaning several topology changes are submitted
  together. It has no authority by itself. Covered topology operators must be
  described by their graph touch meaning and Query lowering path, not by the
  fact that their implementation currently uses a batch-like helper.
- **Construction birth family**: a `worth-kernel` primitive construction family
  that currently prepares admitted handoffs. This milestone requires covered
  birth families to execute `compose_graph` and consume Query obligation
  denials, rather than treating handoff preparation as closure.

Law for all later sections:

```text
Batching is an execution or transport shape. It is not the obligation
authority. Covered graph touch meaning is the authority trigger.
```

## Artifact Ladder

`9.9` must be implemented as a proof-widening artifact ladder. A later artifact
may consume earlier artifacts; it may not re-open raw graph facts, local
legality tables, or string-based rule selection when a proof-bearing artifact
already exists.

Input surfaces:

- `WORTHQueryGraphCompositionProgram`
- `WORTHQueryGraphCompositionBreadth`
- `WORTHQueryWriteCommand`
- read-family / live-read shape descriptors
- preview / branch intent payloads
- declaration-entry and contribution orchestration payloads
- configured-domain handle operating context
- relational schema contract and invariant registration descriptors

Touch and world:

- `WORTHQueryGraphTouchDescriptor`
- `WORTHQueryGraphTouchDescriptorRow`
- `WORTHQueryGraphTouchSelector`
- `WORTHQueryGraphObligationOperatingWorldDescriptor`
- `WORTHQueryGraphObligationOperatingWorldSelector`

Registration and assembly:

- `WORTHQueryGraphObligationKind`
- `WORTHQueryGraphObligationRegistration`
- `WORTHQueryGraphObligationRegistrationCatalog`
- `WORTHQueryGraphObligationIndex`
- `WORTHQueryGraphObligationIndexEntry`
- `WORTHQueryGraphObligationIndexSupportRow`
- `WORTHQueryGraphObligationIndexComplexityContract`
- `WORTHQueryGraphObligationIndexBuildCounters`

Selection and dispatch:

- `WORTHQueryGraphObligationSelection`
- `WORTHQueryGraphObligationSelectionCounters`
- `WORTHQueryGraphObligationDispatchPlan`
- `WORTHQueryGraphObligationDispatchPlanRow`
- `WORTHQueryGraphObligationExecutionPosture`
- `WORTHQueryGraphObligationExecutorContract`
- `WORTHQueryGraphObligationExecutionInput`
- `WORTHQueryGraphObligationExecutionScope`
- `WORTHQueryGraphObligationStateAccessPolicy`
- `WORTHQueryGraphObligationExecutionBudget`
- `WORTHQueryGraphObligationStateLoadCounters`
- `WORTHQueryGraphObligationExecutionCostClass`
- `WORTHQueryGraphObligationBudgetExceededPolicy`
- `WORTHQueryGraphObligationExecutionStatus`
- `WORTHQueryGraphObligationDispatchEnvelope`
- `WORTHQueryGraphObligationVerdict`
- `WORTHQueryGraphObligationExecutionTrace`
- `WORTHQueryGraphObligationVerdictAggregation`
- `WORTHQueryGraphObligationDenialProjection`

Execution bridges:

- relational graph-composition invariant execution point
- schema-contract validator execution
- advisory obligation executor
- preflight-sequencing obligation executor
- capability-gap screen executor
- operating-context gate executor

Attachment and inspection:

- intent decision trace obligation attachment
- write receipt / batch receipt obligation attachment
- read result / live-read result obligation attachment
- graph composition domain-invariant denial attachment
- topology declared mutation artifact obligation summary
- downstream mutation evidence obligation summary

Consumer adoption and certification:

- consumer obligation authoring facade
- consumer selector coverage declaration
- consumer kind x lane support pin
- consumer local-ceremony audit declaration
- obligation adoption manifest
- obligation residue manifest
- in-memory obligation-capable test workspace
- obligation bypass audit registry entries
- worth-topo adoption manifest
- worth-kernel adoption manifest
- duplicate-rule residue report
- kind x lane x touch certification matrix
- milestone closeout support rows and docs agreement proof

## Purity Boundary

The obligation authority pipeline has two different purity regimes:

```text
descriptor derivation
-> selector match
-> obligation selection
-> dispatch plan
```

must be canonical and replay-stable from declared inputs. In particular:

```text
(touch_descriptor, operating_world_descriptor, obligation_index)
-> obligation selection / dispatch plan
```

is the pure dispatch-planning boundary.

The later pipeline:

```text
dispatch plan
-> executor contract resolution
-> executor invocation
-> execution status
-> verdict aggregation
-> envelope attachment
```

is deterministic only when its declared state and basis inputs are also part of
the execution input evidence. Execution may inspect authoritative state,
preview state, branch state, read basis, policy basis, support posture, or
construction context only through the executor contract for that obligation and
lane.

This milestone must not describe final `Allow` / `Advise` / `Block` verdicts
as pure products of touch, world, and index alone. Touch, world, and index
select what must be checked. Executor contracts define what may be inspected to
produce the checked verdict.

## Verdict And Execution Status Model

`WORTHQueryGraphObligationVerdict` remains the public decision surface:

- `Allow`
- `Advise`
- `Block`

Execution traces must carry a separate execution-status taxonomy so the public
verdict does not hide why an obligation did or did not produce a final rule
result.

Required execution statuses:

- `Selected`
- `Executed`
- `NotSelected`
- `NotApplicableAfterStateLoad`
- `DiagnosticOnly`
- `DeferredToBackstop`
- `Unsupported`
- `SuppressedByPolicy`
- `BlockedByPrerequisite`
- `BudgetExceeded`
- `ExecutorError`

Rules:

- `NotSelected` belongs in certification and matrix proof, not ordinary
  per-envelope noise unless artifact policy asks for full explanation.
- `NotApplicableAfterStateLoad` is allowed only when selectors honestly must
  be broader than the state-specific rule. It must be counted and tested so it
  does not become lazy selector design.
- `DiagnosticOnly` may produce advisory evidence but cannot masquerade as
  authoritative enforcement.
- `DeferredToBackstop` must cite the backstop execution point and rule identity.
- `Unsupported` is a support posture result, not a successful no-op.
- `BudgetExceeded` means the executor contract selected a real obligation but
  the declared state-load or diagnostic budget would be exceeded. It must cite
  the exceeded budget row, the state-load counters observed before denial, and
  the lane's fail-closed / diagnostic-only policy.
- `ExecutorError` is operational failure and must not collapse into `Block`
  unless the lane's fail-closed policy explicitly says so.

## Multi-Obligation Reduction Algebra

Every envelope must reduce selected obligation outcomes canonically.

Severity order:

```text
Block > Advise > Allow
```

Reduction rules:

- `Allow + Allow -> Allow`
- `Advise + Allow -> Allow` with advisory evidence retained
- `Advise + Advise -> Allow` with deterministic advisory set retained unless
  the lane declares advisory-as-blocking
- `Block + Advise -> Block` with advisory evidence retained
- `Block + Block -> Block` with deterministic ordered denial set retained
- same `obligation_rule_identity` observed at orchestration and execution
  becomes one logical rule instance with multiple observation points
- same rule identity with conflicting verdicts must retain every observation
  and reduce by severity order
- `CapabilityGapScreen` and `BlockingInvariant` disagreement must project
  denial through the rule with the stricter authority posture while retaining
  the other result as causal evidence

Canonical ordering:

- primary key: obligation kind
- secondary key: rule identity digest
- tertiary key: execution point
- quaternary key: descriptor digest
- final tie-breaker: envelope-local observation index

Digest rule:

```text
equivalent selected obligations + equivalent execution inputs + equivalent
statuses + equivalent verdicts + equivalent reduction order
=> identical envelope digest
```

Any change to reduction ordering, status, rule identity, execution point, state
input digest, or denial projection must change the envelope digest.

## Kind X Lane Support Matrix

The milestone must publish and certify an obligation kind x lane support
matrix. The matrix must not force every obligation kind to execute on every
lane.

Allowed support statuses:

- `Supported`: the lane has a real executor for the obligation kind.
- `Unsupported`: the lane could theoretically carry this obligation kind, but
  the implementation is not admitted. Unsupported must fail closed when a
  selected obligation requires execution.
- `NotApplicable`: the obligation kind has no semantic role on that lane.
- `DiagnosticOnly`: the lane may record advisory/diagnostic evidence but must
  not enforce.
- `DeferredToBackstop`: the lane records selection and delegates enforcement to
  a named later execution point.

The minimum matrix axes are:

- obligation kind:
  `BlockingInvariant`, `SchemaContractValidator`, `AdvisoryObligation`,
  `PreflightSequencingObligation`, `CapabilityGapScreen`,
  `OperatingContextGate`
- lane:
  graph composition, authoritative command batch, scalar mutation,
  effect-triggered write intent, declaration entry, contribution
  orchestration, read family, live read, preview mutation, preview intent,
  branch intent, policy-aware graph mutation, primitive construction birth,
  worth-topo operator catalog, worth-kernel phase chain

The matrix must prove:

- every obligation kind has at least one `Supported` executor
- every covered lane participates in selection
- unsupported selected obligations fail closed or are explicitly
  `DeferredToBackstop`
- `NotApplicable` rows are semantic exclusions, not implementation shortcuts

## Read Touch Vocabulary

Read descriptors must use graph-aware read verbs instead of pretending reads
mutate truth.

Required read touch verbs:

- `ObservesCollection`
- `ObservesRelationKind`
- `ObservesAspectPath`
- `ExposesDerivedTopology`
- `MaterializesDiagnostic`
- `RequiresPolicyBasis`
- `RetainsLiveSubscription`
- `CrossesOperatingWorld`
- `ReadsStaleBasisAllowed`

Mutation touch descriptors and read touch descriptors may share collection,
relation-kind, aspect, basis, and operating-world identity components, but they
must not share verbs that would imply the wrong authority. A harmless read must
not overfire mutation validators, and a derived/projection-heavy read must not
underfire policy, capability, or diagnostic obligations merely because it does
not "write" anything.

## Operating World And Execution Posture

`WORTHQueryGraphObligationOperatingWorldDescriptor` identifies where the work
runs. It must either include, or be paired with, an execution posture that says
how selected obligations apply in that world.

Required execution postures:

- `AuthoritativeEnforce`: enforce blocking obligations now.
- `PreviewCriticalEnforce`: enforce critical gates in preview.
- `PreviewDiagnosticDebt`: downgrade selected obligations to diagnostic debt
  with replayable evidence.
- `BranchPolicyEnforce`: enforce according to branch policy.
- `PromotionReconcile`: re-run or reconcile authoritative obligations during
  promotion.
- `ReadGate`: gate read exposure / policy / capability without mutating truth.
- `DiagnosticOnly`: record evidence without enforcement.

Preview parity does not mean preview is authoritative under another name.
Preview may allow exploratory invalidity only when the execution posture records
which obligations were enforced, downgraded, or deferred and promotion later
reconciles authoritative obligations before committed truth is claimed.

## Executor State And Basis Contract

Every obligation executor must declare its legal state and basis inputs before
execution. Required state access policies:

- `PreState`
- `PostStateCandidate`
- `DeltaOnly`
- `ReadBasis`
- `LiveBasis`
- `PreviewState`
- `BranchState`
- `CommitBoundaryState`
- `ConstructionContext`
- `SupportProfile`
- `PolicyBasis`

Rules:

- The executor contract must name the authority that supplies each input.
- Relational invariant executors must say whether they evaluate pre-state,
  post-state candidate, delta, or commit-boundary state.
- Policy and operating-context gates must cite policy basis and operating-world
  evidence.
- Construction executors must cite construction context instead of re-opening
  host-local phase-chain facts.
- Executors must not inspect raw graph commands after a touch descriptor exists
  unless the executor contract explicitly declares raw command inspection as a
  temporary audited residue.
- The executor contract must cite its execution budget, cost class, and
  state-load counters before invocation. A selected obligation is not
  executable merely because the rule exists; it is executable only when its
  state-load contract is admitted for the lane and operating world.

## Execution Budget And State-Load Guardrails

Selection is intentionally cheap and pure. Execution is where scale can become
dangerous if an obligation silently loads the world, expands a boolean
operation into a broad graph walk, or materializes rich diagnostics by default.
This milestone must make those costs declared, measured, and deniable.

Required artifacts:

- `WORTHQueryGraphObligationExecutionBudget`
- `WORTHQueryGraphObligationStateLoadCounters`
- `WORTHQueryGraphObligationExecutionCostClass`
- `WORTHQueryGraphObligationBudgetExceededPolicy`

Every executor contract must declare:

- maximum state scope it may inspect for each supported lane
- whether the state basis is pre-state, post-state candidate, delta-only,
  read-basis, live-basis, preview-state, branch-state, commit-boundary-state,
  construction-context, support-profile, or policy-basis
- expected breadth basis, such as touched rows, relation-kind degree,
  collection breadth, affected component count, candidate topology component,
  construction family breadth, or policy scope
- counters that prove how much state was loaded and which indexes or
  adjacency structures were touched
- fail-closed, advisory, diagnostic-only, or deferred-to-backstop behavior when
  the declared budget is exceeded
- whether rich diagnostics are hot-path material or artifact-policy gated

Budget-exceeded behavior is not an escape from the obligation. It is a typed
execution result that keeps the system from doing unsafe work while preserving
enough evidence for the caller, support matrix, and certification suite to
understand what would have been required.

Large boolean and topology operations must therefore be admitted through one of
these routes:

- bounded sparse execution with exact counters
- declared dense execution with explicit cost class and lane support
- fail-closed `BudgetExceeded` before broad state load
- diagnostic-only evidence when the lane is not allowed to block
- deferred-to-backstop enforcement with named later execution point

The spec must not permit hidden "try it and see" graph walks. If an executor
cannot state the breadth it needs before loading state, the missing planning
or indexing capability belongs in Milestone `9.10`, and the 9.9 lane must deny
or defer explicitly rather than pretending the cost is acceptable.

## Consumer Obligation Toolset

Consumers must not have to build local ceremony because Query shipped only the
kernel and not the usable kit. This milestone must provide a public consumer
toolset that lets downstream crates register, inspect, pin, audit, and test
graph obligation adoption without rebuilding private pseudo-Query layers.

Required public or facade-backed surfaces:

- `WORTHQueryGraphObligationConsumerRegistrationDeclaration`
- `WORTHQueryGraphObligationSelectorCoverageDeclaration`
- `WORTHQueryGraphObligationSupportPin`
- `WORTHQueryGraphObligationAdoptionManifest`
- `WORTHQueryGraphObligationResidueManifest`
- `WORTHQueryGraphObligationLocalCeremonyAudit`
- `WORTHQueryGraphObligationInMemoryTestWorkspace`
- consumer-kit facade for graph obligation adoption

The consumer kit must allow a downstream crate to:

- declare obligation registrations through Query-owned vocabulary
- declare selector coverage for the touch shapes it claims to cover
- declare expected kind x lane support and pin the accepted posture
- inspect selected obligations for a real touch/world pair
- inspect execution statuses, verdict aggregation, and envelope attachment
- audit local duplicate legality, manual pre-checks, invariant packs, and
  private validator dispatch
- produce adoption and residue manifests with capped residue counts
- run the obligations against an in-memory obligation-capable workspace without
  hand-rolled runtime assembly

The target authoring shape is:

```rust
runtime_builder
    .graph_obligations(|obligations| {
        obligations
            .blocking_invariant("worth.topo.loop_wiring")
            .when_touch(touch::relation_kind(loop_next))
            .in_world(world::configured_domain_handle())
            .execute_with(relational::graph_composition_invariant(rule_id));
    });
```

The target adoption / no-local-ceremony shape is:

```rust
consumer_kit::graph_obligation_adoption("worth-topo")
    .require_no_local_legality("loop_wiring")
    .require_obligation("worth.topo.loop_wiring")
    .require_support_pin("BlockingInvariant", "graph_composition", "Supported")
    .require_selector_coverage(touch::relation_kind(loop_next))
    .seal();
```

These are DX targets, not final API signatures. The implemented API may differ
where existing facade naming requires it, but it must preserve the same
consumer jobs and avoid forcing downstream crates to assemble local
registration, selection inspection, support pinning, or in-memory proof
machinery by hand.

## Denial Projection Boundary

Denial projection is a view over obligation execution, not a second rule
authority.

Every denial projected from an obligation result must cite:

- `obligation_rule_identity`
- `obligation_kind`
- `execution_point`
- `execution_status`
- `touch_descriptor_digest`
- `operating_world_digest`
- `execution_input_digest`
- `dispatch_plan_digest`
- `envelope_digest`

Denial types may add domain-facing explanation fields, but they must not encode
a second copy of rule identity or rule applicability. If a denial cannot be
traced back to an obligation envelope, it is not a covered graph touch
obligation denial.

## Graph Obligation Exclusivity Law

After this milestone, every covered graph-touching surface must consume the
Query-owned obligation chain:

```text
touch descriptor
-> operating-world descriptor
-> graph obligation index
-> obligation selection
-> dispatch plan
-> executor contract
-> execution status
-> verdict aggregation
-> dispatch envelope
-> receipt / trace / denial / artifact attachment
```

Covered surfaces must not:

- choose validators from host-local tables
- pre-run legality graphs beside Query as the ordinary path
- use invariant packs as the ordinary covered-dispatch substitute
- dispatch by matching rendered error text, rule strings, collection labels, or
  topology helper names
- re-open raw graph commands after `WORTHQueryGraphTouchDescriptor` exists
- execute one-off operator validators without a registered obligation identity
- treat `compose_graph` success or handoff preparation as proof that covered
  obligations executed
- let `worth-topo` or `worth-kernel` carry a second implementation of a
  covered Query-owned legality rule

Allowed compatibility residue must be explicit, named, owner-bound, and
audited by Phase 14 and Phase 20. Unnamed residue is a milestone failure.

## Existing Surface Inventory

This milestone widens real surfaces. It must not invent easier surfaces just
because the real ones are awkward.

**WORTH Query mutation and intent surfaces**

- `WORTHQueryRuntime::write(...)`
- `WORTHQueryRuntime::write_intent(...)`
- `WORTHQueryRuntime::write_batch(...)`
- `WORTHQueryRuntime::write_batch_intent(...)`
- `WORTHQueryRuntime::next_effect_write_intent(...)`
- `review_authoritative_runtime_write_batch(...)`
- `review_authoritative_runtime_write_batch_with_graph_artifacts(...)`
- `resolve_reviewed_admitted_authoritative_write_batch_handoff(...)`
- `prepare_authoritative_mutation_batch_execution_binding(...)`
- `execute_authoritative_mutation_batch_execution_binding(...)`
- `write_graph_batch(...)`
- `WORTHQueryAuthoritativeMutationIntentSeed`
- `WORTHQueryAuthoritativeMutationBatchIntentSeed`

**WORTH Query workspace and graph authoring surfaces**

- `workspace.insert(...)`
- `workspace.update(...)`
- `workspace.update_existing(...)`
- `workspace.assert_existing(...)`
- `workspace.verify_existing(...)`
- `workspace.update_existing_verified(...)`
- `workspace.delete(...)`
- `workspace.delete_with(...)`
- `workspace.delete_existing(...)`
- `workspace.delete_existing_with(...)`
- `workspace.delete_existing_verified(...)`
- `workspace.compose_graph(...)`
- `workspace.compose_graph_with_invariant_pack(...)`
- `workspace.batch(...)`
- `workspace.write_intent(...)`
- `workspace.write_batch_intent(...)`
- `workspace.submissions()?.submit(...)`
- `workspace.submissions()?.submit_batch(...)`

**WORTH Query read and live surfaces**

- `compose_read(...)`
- `compose_read_with_invariant_pack(...)`
- `define_read_family(...)`
- `define_read_family_with_invariant_pack(...)`
- `execute_read_family(...)`
- `execute_read_family_in_basis_context(...)`
- `workspace.read_family_intent(...)`
- `workspace.read_family_in_basis_context_intent(...)`
- `workspace.read(...)`
- `workspace.read_live_intent(...)`
- `workspace.read_live_by_name(...)`

**Preview and branch surfaces**

- `WORTHQueryPreviewSession::write(...)`
- `WORTHQueryPreviewSession::batch(...)`
- `WORTHQueryPreviewSession::execute_intent(...)`
- `WORTHQueryPreviewSession::promote(...)`
- `WORTHQueryBranchSession::execute_intent(...)`

**Relational execution surfaces**

- relational invariant registration catalog
- relational schema contract validation descriptors
- `InvariantExecutionPoint` / execution-point classification
- commit-boundary invariant execution
- new graph-composition invariant execution point

**Worth topology surfaces**

- `milestone_one_invariant_registrations()`
- topology operator declaration entry / orchestration boundary
- contribution-composed orchestration
- `TopologyConstructionQueryMutationSurface::ComposeGraph`
- `TopologyDeclaredMutationArtifact`
- topology operator catalog local rewrites
- reference-integrity validation reports and derived read diagnostics

**Worth kernel construction surfaces**

- construction platform entry / configured-domain handle binding
- `construction/authoring.rs`
- admitted scaffold phase-chain handoffs
- seven primitive birth families under
  `construction/phase_chain/admitted_scaffold/family_birth_input/families/`
- `construction/runtime_proof/motion/branch_runtime.rs`
- construction result surface and motion proof outputs

## Covered Surface Matrix

Every covered surface must eventually have one matrix row with these columns:

```text
front door
-> descriptor source
-> operating-world source
-> selection point
-> execution point
-> envelope attachment
-> residue/deletion target
```

The minimum certification matrix for this milestone is:

| Surface family | Descriptor source | Operating world | Selection point | Execution / attachment |
| --- | --- | --- | --- | --- |
| Graph composition entry | graph program + breadth + commands from `write_graph_batch(...)` | configured domain handle or authoritative workspace default | authoritative mutation review/admit | write receipt, batch receipt, graph denial, intent trace |
| Authoritative command batch | command set lowered to graph touch descriptor when covered graph meaning exists | authoritative workspace default or configured handle | authoritative mutation batch review/admit | batch receipt and intent trace |
| Scalar mutation | scalar command lowered to graph touch descriptor when covered graph meaning exists | authoritative workspace default or configured handle | scalar mutation intent/admission bridge into shared obligation selection | write receipt and mutation evidence |
| Effect-triggered write intent | pending write-intent payload | effect execution basis and authoritative target world | effect write-intent admission | effect-intent receipt and decision trace |
| Declaration-entry orchestration | declaration payload / topology operator intent | configured domain handle operating context | declaration-entry review before workspace mutation | orchestration denial or declared mutation artifact |
| Contribution orchestration | contribution-composed mutation payload | contribution / domain handle operating context | contribution admission before runner mutation | contribution denial or declared mutation artifact |
| Read family execution | read family / query shape descriptor | read basis / policy / operating context | read-family admission | read result and decision trace |
| Live read execution | live query declaration / retained live artifact | live basis / policy / operating context | live-read admission | live result, delivery batch, decision trace |
| Preview-local mutation | preview write or preview batch payload | preview session descriptor | preview mutation admission | preview receipt / preview denial |
| Preview intent execution | preview intent payload | preview session descriptor | preview intent admission | preview intent receipt / trace |
| Branch intent execution | branch intent payload | branch session descriptor | branch intent admission | branch intent receipt / trace |
| Policy-aware graph mutation | covered graph touch + policy basis | operating-context gate descriptor | policy gate before execution | gate verdict, denial, trace |
| Primitive construction birth | birth family admitted synopsis lowered to compose program | kernel configured domain handle | compose execution through Query | typed obligation denial or topology commit receipt |
| Worth-topo operator catalog | operator touch declaration / compose program / command payload | operator configured domain handle | Query obligation selection before covered mutation | operator artifact evidence + deletion of local guard |
| Worth-kernel phase chain | construction phase touch declaration / compose program | construction configured domain handle | Query obligation selection before covered mutation or preflight | result surface evidence + deletion of offline legality duplicate |

Rows may be split during implementation when a family proves to contain more
than one authority boundary. Rows may not be collapsed by saying "batch" or
"operator" when the descriptor source, operating world, or attachment point
differs.

## Implementation Refactor Batch Boundaries

This milestone is intentionally too large to run as one uninterrupted
implementation campaign. The phase list below remains the detailed acceptance
plan, but implementation should proceed through the following larger refactor
groups. Each group must end with QA, warning/dead-code cleanup, directory and
file-length review, and a targeted geometry-kernel/topology-kernel cleanup pass
where the new authority has made old local proof obsolete.

The point of these boundaries is not to weaken the milestone. The point is to
avoid asking one model session to keep every obligation lane, topology
operator, validator, kernel construction family, and documentation claim in
working memory at once. Pause at each boundary, refactor what the new authority
made safe to refactor, then continue.

### Batch 1: Authority Substrate

**Covers:** Phases 1-4.

Build the obligation authority substrate: sealed obligation kinds, verdicts,
execution statuses, executor budget descriptors, dispatch envelopes, graph
touch descriptors, selectors, operating-world binding, registration,
support-pin-ready posture rows, assembly index, and complexity counters.

**Boundary requirement:** do not begin execution-lane wiring until dispatch can
be selected from a canonical touch descriptor, operating-world descriptor, and
assembly index with replay-stable evidence, support posture, and budget
identity for the selected obligations.

**Pause/refactor target:** clean `worth-query` obligation module topology,
builder/index boundaries, digest/canonicalization naming, and any broad files
created while finding the right substrate shape.

### Batch 2: Core Execution Seams

**Covers:** Phases 5-7.

Make the authority executable at the central seams: relational graph-composition
execution point, authoritative mutation intent admission, declaration-entry
dispatch, and contribution-orchestration dispatch.

**Boundary requirement:** a graph mutation that already reaches Query's
authoritative mutation family must produce typed obligation verdicts and a
canonical envelope without relying on a manual invariant-pack pre-hook as the
ordinary path.

**Pause/refactor target:** clean `worth-relational` validation execution
shape, `worth-query` mutation admission and graph-composition entrypoints, and
`worth-topo` declaration/contribution orchestration seams.

### Batch 3: Non-Canonical Front Door Parity

**Covers:** Phases 8-10.

Extend the same obligation authority across read execution, read composition,
live reads, preview-local mutation, branch/preview intent execution, and
policy-aware graph mutation gates.

**Boundary requirement:** read, preview, branch, and policy-aware fronts may
have different postures, but none may bypass the same touch/operating-world
obligation authority when they carry covered graph meaning.

**Pause/refactor target:** clean topology read proof/read-view modules,
preview/branch motion surfaces, policy/operating-context naming, and any
duplicated parity helpers introduced during the spread across front doors.

### Batch 4: Evidence, Executors, And Anti-Folklore

**Covers:** Phases 11-14.

Wire advisory, capability-gap, preflight-sequencing, and envelope attachment;
then re-home derived read validation and ship the consumer obligation bypass
audit.

**Boundary requirement:** obligation results must be inspectable on receipts,
results, decision traces, handoffs, and mutation evidence before duplicate
validators or host-local guards are deleted as covered residue.

**Pause/refactor target:** clean receipt/trace/evidence structures, delete
covered duplicate validators, tighten bypass audit patterns, and name any
remaining manual guard as explicit residue with owner and blocker.

### Batch 5: Worth Geometry Adoption

**Covers:** Phases 15-18.

Apply the completed authority to Worth: kernel construction operating context,
primitive construction birth `compose_graph` execution, `worth-topo` operator
catalog adoption, and `worth-kernel` construction-surface adoption.

**Boundary requirement:** covered topology operators and primitive construction
birth families must consume Query obligation authority instead of maintaining
parallel local legality. Any family or operator not migrated must be explicit
certification residue, not an unnamed skip.

**Pause/refactor target:** after Phase 15, clean construction platform entry
and operating-context wiring; after Phase 16, clean admitted-scaffold and
family-birth-input modules; after Phase 17, clean topology operator catalog and
reference-integrity validation; after Phase 18, clean kernel phase-chain,
offline legality, motion proof, and result-surface duplication.

### Batch 6: Docs And Certification Close

**Covers:** Phases 19-20.

Publish the product mental model, support rows, AI_README category, hostile
certification matrix, and closeout.

**Boundary requirement:** documentation may only claim closure already proven by
the implementation and reference adoption batches. AI-facing guidance must name
the exact obligation authority boundary and must not teach manual invariant
packs, local legality graphs, or duplicate validator callbacks as ordinary
covered paths.

**Pause/refactor target:** align `AI_README.md`, product docs, roadmap,
test-requirements, support rows, and closeout evidence so no stale folklore
survives in docs after the code has moved.

## Phase Plan

### Phase 1: Obligation Authority Model And Dispatch Envelope

Freeze sealed obligation kinds, three-state verdicts (`Allow` / `Advise` /
`Block`), execution statuses, dispatch plan rows, executor contracts, verdict
aggregation, denial projection identity, and the canonical dispatch envelope —
including executor budget identity and multi-obligation recordings when
several rules fire on one touch.

**Relevant subsystems**
- `crates/worth-query/src/runtime/mutation/graph_composition/`
- `crates/worth-query/src/intent_admission/trace/`

**Relevant APIs**
- new: `WORTHQueryGraphObligationKind`, `WORTHQueryGraphObligationVerdict`,
  `WORTHQueryGraphObligationDispatchPlan`,
  `WORTHQueryGraphObligationDispatchEnvelope`
- new: `WORTHQueryGraphObligationExecutionStatus`,
  `WORTHQueryGraphObligationExecutorContract`,
  `WORTHQueryGraphObligationExecutionInput`,
  `WORTHQueryGraphObligationStateAccessPolicy`,
  `WORTHQueryGraphObligationExecutionBudget`,
  `WORTHQueryGraphObligationExecutionCostClass`,
  `WORTHQueryGraphObligationBudgetExceededPolicy`,
  `WORTHQueryGraphObligationVerdictAggregation`,
  `WORTHQueryGraphObligationDenialProjection`
- `WORTHQueryGraphCompositionDomainInvariantDenial`
- `WORTHQueryIntentDecisionTraceEnvelope`

**Warnings**
- Do not make `Allow` / `Advise` / `Block` carry execution lifecycle meaning.
  Public verdict and execution status are separate concepts.
- Do not let denial projection become a second rule authority. Denials cite
  obligation identity and envelope evidence; they do not own applicability.
- Do not define dispatch plans as executable verdicts. Plans name what must
  run; executors produce status and verdict.
- Do not model budget exceed as an executor crash or a local string denial.
  `BudgetExceeded` is a planned execution outcome with a declared policy.

**Test requirements**
- Adversarial equivalence: envelope digests stable under replay.
- Adversarial localization: host-WORTHd envelopes uncompilable or sealed.
- Adversarial completeness: multi-obligation touches record every fired rule.
- Adversarial reduction: `Block + Advise`, duplicate rule observations, and
  two-block denial sets reduce in deterministic order with stable digest.
- Adversarial status split: `Unsupported`, `DiagnosticOnly`,
  `DeferredToBackstop`, `BudgetExceeded`, and `ExecutorError` do not collapse
  into public verdict variants without explicit lane policy.
- Adversarial budget identity: equivalent executor budget descriptors produce
  stable budget digests, and changing cost class, max state scope, or
  budget-exceeded policy changes the envelope-relevant identity.

**Engineering decisions**
- Block outcomes lower into graph-composition domain-invariant denials where
  compatible; advise outcomes carry structured context.
- Envelope scheme version explicit; digests lower through Milestone `9.6`
  evidence identity where applicable.
- Final envelope identity includes plan digest, execution input digest,
  execution budget digest, execution statuses, verdicts, reduction order, and
  denial projection digest.

**Open questions**
- None.

### Phase 2: Graph Touch Descriptor Vocabulary

Ship sealed `WORTHQueryGraphTouchDescriptor` derived from graph composition
programs, authoritative mutation batch seeds, command sets, scalar write
commands, read-family shapes, live-read shapes, preview/branch intent payloads,
and touched aspect keys (including `.touches(...)` where present).

**Relevant subsystems**
- `crates/worth-query/src/runtime/mutation/graph_composition/`
- `crates/worth-query/src/runtime/surface/graph_composition_domain_invariant_summary.rs`
- `crates/worth-query/src/runtime/mutation/` (batch lowering)

**Relevant APIs**
- `WORTHQueryGraphCompositionProgram`, `WORTHQueryGraphCompositionProgramStepKind`
- `WORTHQueryAuthoritativeMutationBatchIntentSeed`
- `WORTHQueryAuthoritativeMutationIntentSeed`
- `WORTHQueryGraphCompositionBreadth`
- new: `WORTHQueryGraphTouchDescriptor`

**Warnings**
- Do not let `batch` mean "all multi-command surfaces." The descriptor source
  must say whether it came from graph composition, ordinary command batch,
  scalar command, read shape, live shape, preview mutation, or branch/preview
  intent payload.
- Do not derive graph touch identity from rendered collection names, operator
  labels, or error messages when typed command, program, aspect, relation-kind,
  lifecycle, or read-shape facts exist.
- Do not reuse mutation verbs for reads. Read descriptors need read verbs such
  as `ObservesCollection`, `ExposesDerivedTopology`,
  `MaterializesDiagnostic`, and `RetainsLiveSubscription`.

**Test requirements**
- Adversarial parity: semantic-equivalent programs → equal descriptors.
- Adversarial rejection: relation kind / lifecycle / aspect changes alter digest.
- Adversarial breadth: multi-component graph programs derive all relevant
  collection, relation-kind, aspect-operation, mutation-family, and lifecycle
  keys without requiring a caller-provided validator list.
- Adversarial read vocabulary: harmless collection observation does not select
  mutation-only validators, while derived topology exposure and diagnostic
  materialization select read-side obligations when registered.

**Engineering decisions**
- Read touch descriptors use the same vocabulary rules as mutation descriptors.
- The descriptor is a proof-carrying input to obligation selection, not a
  diagnostic summary that downstream code may ignore.
- "Same vocabulary rules" means same canonical identity discipline, not same
  verbs. Read verbs and mutation verbs remain authority-distinct.

**Open questions**
- None.

### Phase 3: Registration, Touch Selectors, And Operating World Binding

Ship touch selectors, operating-world selectors, and registration for every
obligation kind on the ordinary Query runtime builder. Auto-index native
relational schema contracts from lowered contract descriptors. Registration
also declares the support posture and executor budget identity later consumed
by support pins and execution-lane admission.

**Relevant subsystems**
- `crates/worth-query/src/runtime/builder.rs`
- `crates/worth-relational/src/validation/data/`, `schema/`

**Test requirements**
- Adversarial equivalence: selectors do not cross-match unrelated lanes.
- Adversarial rejection: conflicting registrations fail at assembly.
- Adversarial world split: preview, branch, configured-domain-handle, and
  committed-authority selectors only match their declared operating worlds plus
  explicit any-world selectors.
- Adversarial support posture: two registrations with the same rule identity
  and selector cannot declare contradictory lane support or budget identity
  without failing assembly.

**Engineering decisions**
- Custom invariant scope planners normalize into selector vocabulary.
- Registration is declaration of applicability. It must not execute relational
  invariants, inspect topology state, or allocate dispatch envelopes.
- Registration owns declared support posture and budget identity, but it does
  not own consumer adoption manifests. Phase 14 turns these rows into public
  support pins and adoption proof.

**Open questions**
- None.

### Phase 4: Assembly Index And Complexity Contract

Build `WORTHQueryGraphObligationIndex` at assembly with inspectable support rows
and named complexity contracts for index build and dispatch selection. The
index must retain selected obligation support posture and executor budget
identity so later execution can deny unsafe state load without re-opening
registration catalogs or consumer-local policy tables.

**Relevant subsystems**
- `crates/worth-query/src/runtime/mutation/graph_composition/obligation/index/`
- `crates/worth-query/src/runtime/builder.rs`
- `crates/worth-query/src/runtime/graph_obligation_registration.rs`

**Relevant APIs**
- `WORTHQueryGraphObligationRegistrationCatalog`
- `WORTHQueryGraphObligationIndex`
- `WORTHQueryGraphObligationSelection`
- `WORTHQueryGraphObligationIndexBuildCounters`
- `WORTHQueryGraphObligationSelectionCounters`
- `WORTHQueryGraphObligationIndexSupportRow`
- `WORTHQueryGraphObligationExecutionBudget`
- `WORTHQueryGraphObligationExecutionCostClass`
- `WORTHQueryGraphObligationBudgetExceededPolicy`

**Warnings**
- The index is derived state. It must be rebuildable from the registration
  catalog and must not become a second registration authority.
- Selection counters must count lookup work. A no-match selection must report
  attempted bucket lookups without inventing candidate obligations.
- The index may carry executor budget identity and support posture, but it must
  not run executor state-load counters. State-load counters belong to the
  execution phase that actually touches state.

**Test requirements**
- Adversarial parity: identical registration → identical index digests.
- Adversarial budget: dispatch selection counters prove O(matched obligations).
- Adversarial no-match: unrelated selectors produce zero matched obligations,
  zero candidates, zero matched buckets, and zero catalog scan count.
- Adversarial shape sensitivity: changing a registration selector, obligation
  kind, operating-world selector, or support/complexity surface changes the
  relevant digest.
- Adversarial support-pin basis: selected obligations expose enough support
  row identity for a later consumer support pin to detect posture drift without
  asking the consumer to inspect private index internals.
- Adversarial budget drift: changing selected obligation budget identity
  changes the selection/index digest used by later dispatch planning, while
  ordinary selection counters remain bounded and do not execute state loads.

**Engineering decisions**
- Lookup: `(touch_descriptor, operating_world_descriptor) -> obligation
  selection`. Dispatch plan materialization is a later artifact that consumes
  this selection; selection itself must stay inspectable and replay-stable.
- Support rows report Verified for every obligation kind and lane this
  milestone ships.
- Execution budget rows are index-carried planning evidence. Actual
  `WORTHQueryGraphObligationStateLoadCounters` are produced only by later
  executor invocation.

**Open questions**
- None.

### Phase 5: Graph Composition Execution Point And Relational Rule Migration

Add explicit relational graph-composition invariant execution point. Migrate
covered milestone-one custom invariant registrations from commit-only execution
to graph-composition + commit backstop with single rule identity.

**Relevant subsystems**
- `crates/worth-relational/src/validation/data/execution.rs`
- `crates/worth-relational/src/validation/engine/`
- `crates/worth-topo/src/validation/reference_integrity/`

**Relevant APIs**
- `InvariantExecutionPoint` (new graph-composition variant)
- `milestone_one_invariant_registrations()`

**Test requirements**
- Adversarial parity: `.m1.topology.loop_wiring` and siblings share one rule
  identity across graph-composition and commit-boundary execution.
- Adversarial duplication audit: exact-zero second implementations on covered
  rule identities.

**Engineering decisions**
- Document relationship to `MutationSensitive` and `CommitBoundary`; covered
  topo rules migrate to graph-composition execution point for compose-time
  dispatch while retaining commit backstop.

**Open questions**
- None.

### Phase 6: Authoritative Mutation Intent Admission Integration

Integrate obligation dispatch into the canonical authoritative mutation
intent-admission family. `compose_graph` anchors on
`review_authoritative_runtime_write_batch_with_graph_artifacts(...)` →
admitted handoff → execution binding → execution. Scalar-write,
command-batch, graph-composition, and effect-triggered write-intent fronts must
lower covered graph touch meaning into the same touch vocabulary and obligation
selection model rather than becoming separate obligation authorities.

**Relevant subsystems**
- `crates/worth-query/src/runtime/runtime_batch_write_intents.rs`
- `crates/worth-query/src/runtime/runtime_batch_write_entrypoints.rs`
- `crates/worth-query/src/runtime/runtime_intents.rs`
- `crates/worth-query/src/runtime/workspace.rs`
- `crates/worth-query/src/intent_admission/plans/mutation.rs`
- `crates/worth-query/src/runtime/workspace_graph.rs`

**Relevant APIs**
- `WORTHQueryAuthoritativeMutationBatchIntentSeed`
- `WORTHQueryAuthoritativeMutationIntentSeed`
- `review_authoritative_runtime_write_batch_with_graph_artifacts`
- `resolve_reviewed_admitted_authoritative_write_batch_handoff`
- `execute_authoritative_mutation_batch_execution_binding`
- `runtime.write`, `runtime.write_intent`, `runtime.write_batch`,
  `runtime.write_batch_intent`
- `workspace.write`, `workspace.write_intent`, `workspace.write_batch_intent`,
  `workspace.batch`
- `workspace.insert`, `workspace.update`, `workspace.update_existing`,
  `workspace.assert_existing`, `workspace.verify_existing`,
  `workspace.update_existing_verified`, `workspace.delete`,
  `workspace.delete_with`, `workspace.delete_existing`,
  `workspace.delete_existing_with`, `workspace.delete_existing_verified`
- `runtime.next_effect_write_intent`,
  `admit_next_effect_write_intent_for_execution`
- `WORTHQueryGraphObligationDispatchEnvelope`

**Target shape — before**

```rust
// workspace_graph.rs — manual pack pre-hook, then intent admission underneath
invariant_gate(&invariant_context)?;
self.runtime.write_graph_batch(commands, breadth, program)?;
```

**Target shape — after**

```rust
// obligation dispatch runs inside the authoritative mutation batch seam.
// Graph program + breadth already enter WORTHQueryAuthoritativeMutationBatchIntentSeed.
self.runtime.write_graph_batch(commands, breadth, program)?;
// Review/admit/execute materializes the obligation dispatch envelope on the
// handoff path. The authority is the lowered graph touch meaning, not the word
// "batch."
```

**Warnings**
- Do not implement a side callback before intent review. Dispatch must be part
  of the proof-bearing admission/execution chain.
- Do not say "batch path" in tests or docs without naming the exact front door:
  graph composition entry, authoritative command batch, scalar mutation,
  explicit submission batch, or effect-triggered write intent.
- Do not remove the manual invariant-pack path by deleting semantics before the
  obligation path can produce the same typed denial and evidence. Compatibility
  residue must be contained and audited, not silently erased.

**Test requirements**
- Adversarial rejection: loop-wiring violation blocks during authoritative
  mutation review/admit/execution with typed denial + envelope — not via manual
  pre-hook only.
- Adversarial equivalence: scalar mutation, authoritative command batch, and
  graph composition paths sharing the same covered touch shape produce
  compatible dispatch evidence through the authoritative mutation intent
  family.
- Adversarial delegation: `runtime.next_effect_write_intent(...)` cannot become
  an obligation bypass around the same touch descriptors the authoritative
  mutation family would enforce directly.
- Adversarial attachment: the decision trace, handoff, execution binding, and
  write/batch receipt all cite the same obligation envelope digest for the same
  admitted mutation.
- Adversarial state contract: relational graph-composition execution declares
  whether it consumes pre-state, post-state candidate, delta-only, or
  commit-boundary state, and false-positive/false-negative fixtures prove the
  declared state input matters.

**Engineering decisions**
- Remove manual `invariant_pack` as the ordinary compose path; mechanically
  contain it as thin projection helper only.
- Scalar topology closeout that currently materializes through
  `finalize_batch_write_closeout` participates only by lowering to the same
  canonical touch/selection/envelope chain; the helper name is not an authority
  boundary.
- Do not treat scalar write, authoritative command batch, graph composition,
  explicit submission batch, and effect-triggered write intent as separate
  obligation authorities. They are distinct front doors over the same covered
  graph touch authority where graph meaning is present.
- Authoritative mutation integration attaches executor contracts before
  invoking relational, schema, advisory, capability, preflight, or operating
  context executors.

**Open questions**
- None.

### Phase 7: Declaration-Entry And Contribution-Orchestration Dispatch

Wire obligation dispatch at declaration-entry and contribution-composed
orchestration boundaries — where worth-topo already stops mutations before
the runner reaches graph composition, command submission, or explicit workspace
mutation.

**Relevant subsystems**
- `crates/worth-query/src/grouped_authoring/contributions.rs`
- `crates/worth-topo/src/topology_operators/application/declaration_entry/orchestration_boundary.rs`
- `crates/worth-topo/src/topology_operators/query_workflow/`

**Relevant APIs**
- declaration-entry orchestration request / result artifacts
- contribution-composed orchestration payloads
- configured-domain handle operating context
- `WORTHQueryGraphObligationOperatingWorldDescriptor`
- `WORTHQueryGraphObligationDispatchEnvelope`

**Warnings**
- Do not wait for `compose_graph` when the declaration or contribution payload
  already carries enough graph touch meaning to deny before mutation.
- Do not duplicate the later execution obligation. Orchestration dispatch may
  advise, preflight, or block entry; execution dispatch still owns mutation
  authority when the mutation runs.
- Do not represent orchestration denial as a local topology error if an
  obligation envelope can carry the typed verdict.

**Test requirements**
- Adversarial rejection: contribution-denied orchestration produces obligation
  envelope evidence without reaching workspace mutation.
- Adversarial equivalence: operating context on declaration entry matches
  dispatch operating-world descriptor on subsequent mutation.
- Adversarial non-duplication: an orchestration-fired obligation and an
  execution-fired obligation with the same rule identity collapse into one
  traceable rule identity rather than two local validator implementations.

**Engineering decisions**
- Preflight and advisory obligations may fire at orchestration boundary when
  touch shape is known from declaration payload; sequencing obligations order
  orchestration before workspace execution.
- Declaration-entry and contribution-orchestration dispatch produce envelopes
  that are consumed by later declared mutation artifacts. They must not become
  a second standalone validator report family.

**Open questions**
- None.

### Phase 8: Read Execution, Read Composition, And Live Read Obligation Dispatch

Extend obligation dispatch to all read-composition product entry points:
`compose_read`, `define_read_family`, `execute_read_family`, live-read
execution, and the intent-admission front doors those surfaces already expose.

**Relevant subsystems**
- `crates/worth-query/src/runtime/workspace_queries.rs`
- `crates/worth-query/src/runtime/runtime_read_intents.rs`
- `crates/worth-query/docs/authoring/read-composition.md`
- `crates/worth-query/docs/execution/intent-admission.md`

**Relevant APIs**
- `compose_read`, `compose_read_with_invariant_pack`
- `define_read_family`, `define_read_family_with_invariant_pack`
- `execute_read_family`, `execute_read_family_in_basis_context`
- `read_family_intent`, `read_family_in_basis_context_intent`
- `read`, `read_live_intent`, `read_live_by_name`

**Warnings**
- Read obligation dispatch is not mutation validation. It gates and explains
  read access, policy, capability, and diagnostic obligations for graph-shaped
  reads without pretending reads mutate truth.
- Do not use read obligation dispatch to auto-provision graph indexes; that is
  Milestone `9.10`. This milestone may require declared read touch descriptors
  and bounded dispatch selection, but it must not hide broad read-index
  creation inside obligation execution.

**Test requirements**
- Adversarial parity: read-family obligation registered for a collection kind
  fires on matching read touch without manual pack closure.
- Adversarial rejection: unrelated read bundles do not invoke non-matching
  obligations.
- Adversarial equivalence: `execute_read_family`, `read_family_intent(...).execute()`,
  `read_live_intent(...).execute()`, and helper fronts such as `read(&view)`
  retain the same obligation evidence when they lower the same admitted read
  meaning.

**Engineering decisions**
- Read touch descriptors from Phase 2 drive read-side index lookup.
- `compose_read`, `execute_read_family`, `read_family_intent`, and
  `read_live_intent` are one public family story for this milestone; do not
  certify only the helper with the shortest name.
- Read result attachment must preserve the dispatch envelope digest without
  forcing the read hot path to materialize rich diagnostics unless the artifact
  policy requires it.

**Open questions**
- None.

### Phase 9: Preview Direct Mutation And Branch/Preview Intent Parity

Extend obligation dispatch to the real non-authoritative surfaces that exist
today: preview-local direct writes/batches and preview/branch intent execution.
Do not pretend WORTH Query already has a branch batch-write API when the branch
surface is intent-first.

**Relevant subsystems**
- `crates/worth-query/src/runtime/preview/mutation_ops.rs`
- `crates/worth-query/src/runtime/preview/workflow_ops.rs`
- `crates/worth-query/src/runtime/branch.rs`
- `crates/worth-query/src/runtime/workspace.rs` (preview/branch entry)
- `crates/worth-kernel/src/construction/runtime_proof/motion/branch_runtime.rs`

**Relevant APIs**
- `WORTHQueryPreviewSession::write`, `WORTHQueryPreviewSession::batch`
- `WORTHQueryPreviewSession::execute_intent`, `WORTHQueryPreviewSession::promote`
- `WORTHQueryBranchSession::execute_intent`

**Test requirements**
- Adversarial parity: preview-local direct writes and batches carrying graph
  touch shape receive obligation dispatch evidence equivalent to authoritative
  touch selection where policy allows execution.
- Adversarial parity: preview/branch intent execution cannot bypass obligation
  dispatch for touch-bearing intent payloads routed through those lanes.
- Adversarial localization: preview lane cannot bypass obligations that
  authoritative lane would enforce for the same touch descriptor.
- Adversarial posture: preview critical gates enforce, preview diagnostic debt
  downgrades selected obligations with evidence, branch policy enforces by
  branch posture, and promotion reconciles authoritative obligations before
  committed truth is claimed.

**Engineering decisions**
- Kernel branch-preview motion surfaces certify under this phase.
- `preview.promote()` re-enters the authoritative `runtime.write(...)` path, so
  preview-local evidence and promoted authoritative evidence must remain
  auditable as one continuous story.
- Branch parity in this milestone is about `execute_intent`, not an invented
  branch-local batch mutation API.
- Preview and branch descriptors must include or reference
  `WORTHQueryGraphObligationExecutionPosture`; parity means comparable evidence
  and reconcile rules, not identical enforcement behavior.

**Open questions**
- None.

### Phase 10: Policy-Aware Graph Mutation And Operating Context Gate Execution

Build policy-aware graph mutation execution and operating-context gate
dispatch on write authority — extending policy narrowing beyond read plans.

**Relevant subsystems**
- `crates/worth-query/src/policy_narrowing/`
- `crates/worth-query/src/policy_basis/`, `tenant_basis/`
- configured domain handle operating context surfaces

**Test requirements**
- Adversarial rejection: collaborative vs restricted operating contexts produce
  different gate verdicts on the same touch when rules differ.
- Adversarial parity: read-side policy basis and write-side gate evidence cite
  compatible basis identity.

**Engineering decisions**
- `PolicyAwareExecution` support row moves to Verified for graph mutation
  obligation dispatch in this milestone.

**Open questions**
- None.

### Phase 11: Advisory, Capability Gap, And Preflight Sequencing Executors

Wire remaining obligation kinds to Query authority surfaces: admission
contributions / intent admission lattice, capability gap and support matrix
truth, and ordered prerequisite obligations (motion witness before finish).
This phase also freezes the executor budget contract for these executor
families so expensive state access is explicit before the first real
invocation.

**Relevant subsystems**
- `crates/worth-query/docs/domain-capabilities/admission/advisory-and-violation-contributions.md`
- `crates/worth-query/docs/domain-capabilities/invariants/capability-gaps-and-invariant-denials.md`
- `crates/worth-kernel/src/construction/runtime_proof/motion/`

**Test requirements**
- Adversarial rejection: finish-before-witness sequencing blocks with typed
  preflight denial — not `unreachable!`.
- Adversarial equivalence: advisory obligations produce `Advise` verdicts in
  envelope.
- Adversarial budget denial: a capability-gap or sequencing executor selected
  for a broad construction/topology touch denies with `BudgetExceeded` before
  unbounded state load when its declared state-load budget is insufficient.
- Adversarial diagnostic policy: rich capability-gap diagnostics materialize
  only when artifact policy requests them; ordinary enforcement keeps hot-path
  counters bounded and still attaches enough evidence to explain the verdict.

**Engineering decisions**
- All executor families introduced or wired in this phase must expose
  `WORTHQueryGraphObligationExecutionBudget`,
  `WORTHQueryGraphObligationExecutionCostClass`, and
  `WORTHQueryGraphObligationStateLoadCounters`.
- Budget exceed is an execution status and support-matrix proof event, not a
  string error and not a silent fallback to local legality.

**Open questions**
- None.

### Phase 12: Envelope Attachment To Receipts, Results, Decision Traces, And Mutation Evidence

Wire obligation dispatch envelopes onto write receipts, read results, live-read
results, intent decision traces, execution handoffs, and downstream mutation
evidence surfaces.

**Relevant subsystems**
- `crates/worth-query/src/intent_admission/trace/envelope.rs`
- `crates/worth-query/src/runtime/surface/` (batch write receipt)
- `crates/worth-query/src/runtime/runtime_read_intents.rs`
- `crates/worth-topo/src/topology_operators/application/declared_mutation_artifact/`

**Test requirements**
- Adversarial equivalence: receipt inspection and intent trace reproduce the
  same dispatch envelope digest for one mutation.
- Adversarial equivalence: read-result and live-read-result evidence retain the
  same obligation envelope digest emitted at the covered read execution seam.
- Adversarial completeness: topo `TopologyDeclaredMutationArtifact` exposes
  obligation evidence without host-local recomputation.
- Adversarial denial projection: every projected denial cites obligation rule
  identity, execution point, status, descriptor digest, world digest, execution
  input digest, dispatch plan digest, and envelope digest.
- Adversarial non-authority: changing denial wording does not change rule
  applicability or obligation identity; changing obligation identity changes
  denial projection digest.

**Engineering decisions**
- `TopologyMutationApplicationEvidence` extends to carry obligation dispatch
  summary alongside existing verified-operation counts.
- Denial projection is an attachment view over envelope evidence, not a second
  execution engine or rule identity namespace.

**Open questions**
- None.

### Phase 13: Derived Read Validation Re-Homed

Re-home materialized-view and interpreted-topology validators as derived read
diagnostics from registered rule identity on covered mutation paths.

**Relevant subsystems**
- `crates/worth-topo/src/validation/`
- `crates/worth-topo/src/projection/runtime_boundary/read_stage.rs`

**Test requirements**
- Adversarial parity: derived read reports agree with compose-time envelope on
  same state.
- Adversarial duplication audit: exact-zero second implementations on covered paths.

**Open questions**
- None.

### Phase 14: Consumer Obligation Toolset And Bypass Audit

Ship the consumer obligation kit and mechanical enforcement against obligation
folklore using Milestone `9.8` audit machinery extended for legality
duplication patterns. This phase exists so downstream crates can adopt the
authority without writing local ceremony for registration, support pins,
selector coverage, in-memory proof, adoption manifests, or residue accounting.

**Relevant subsystems**
- Milestone `9.8` prohibition registry and audit artifact
- `crates/worth-query/src/facade/` or existing public consumer-kit facade home
- `crates/worth-query/src/runtime/assembly/` obligation-capable in-memory setup
- `crates/worth-query/src/support/` and support matrix surfaces
- `crates/worth-topo/src/topology_operators/`
- `crates/worth-kernel/src/construction/phase_chain/`

**Relevant APIs**
- `WORTHQueryGraphObligationConsumerRegistrationDeclaration`
- `WORTHQueryGraphObligationSelectorCoverageDeclaration`
- `WORTHQueryGraphObligationSupportPin`
- `WORTHQueryGraphObligationAdoptionManifest`
- `WORTHQueryGraphObligationResidueManifest`
- `WORTHQueryGraphObligationLocalCeremonyAudit`
- `WORTHQueryGraphObligationInMemoryTestWorkspace`
- consumer-kit facade for graph obligation adoption

**Test requirements**
- Seeded manual guards, legality graphs, and ordinary-path invariant-pack usage
  fail audit with zero false positives on literals/comments.
- Adversarial no-local-ceremony: a reference consumer can register an
  obligation, declare selector coverage, pin support posture, run an in-memory
  obligation-capable workspace, inspect selected obligations and execution
  statuses, and produce an adoption manifest using only Query-shipped surfaces.
- Adversarial residue cap: a consumer attempting to add an uncapped residue row
  or grow a previously capped residue class fails certification.
- Adversarial support pin drift: a support matrix change from `Supported` to
  `DiagnosticOnly`, `Unsupported`, `NotApplicable`, or `DeferredToBackstop`
  breaks the consumer support pin until the adoption manifest is updated with
  an explicit decision.

**Engineering decisions**
- The bypass audit is not enough by itself. The same public kit that catches
  local ceremony must also provide the replacement path so consumers do not
  rebuild private wrappers around Query.
- Consumer adoption manifests are product artifacts. They must be readable by
  certification and docs, not hidden as test-only fixtures.

**Open questions**
- None.

### Phase 15: Kernel Construction Platform Operating Context Wiring

Wire worth-kernel construction through configured domain handles and
operating context — matching binding's platform entry pattern. worth-topo
operator declaration entry already carries operating context; this phase
targets kernel construction's naked `WORTHQueryWorkspace` authoring session.

**Relevant subsystems**
- `crates/worth-kernel/src/construction/authoring.rs`
- `crates/worth-kernel/src/binding/authoring/query_domain.rs` (reference)
- `crates/worth-query/docs/domain-capabilities/configured-domain-handles.md`

**Test requirements**
- Adversarial equivalence: construction operating context yields stable
  `operating_context_identity_digest` on obligation dispatch.
- Adversarial rejection: covered construction mutation without operating context
  fails platform entry contract or audit.

**Open questions**
- None.

### Phase 16: Primitive Construction Birth Compose Execution

Build and execute primitive construction birth `compose_graph` programs from
admitted handoffs — closing the gap where `ComposeGraph` is declared but never
executed.

**Relevant subsystems**
- `crates/worth-topo/src/construction/query_native_boundary/`
- `crates/worth-kernel/src/construction/phase_chain/admitted_scaffold/`
- `crates/worth-kernel/src/construction/phase_chain/admitted_scaffold/family_birth_input/families/` (7 families)

**Relevant APIs**
- `prepare_primitive_construction_query_admitted_handoff_from_synopsis`
- `TopologyConstructionQueryMutationSurface::ComposeGraph`
- birth compose program authoring per family

**Test requirements**
- Adversarial rejection: shell-with-hole layout violation blocks at compose
  with typed obligation denial — not offline string-mapped `InvalidRequest`.
- Adversarial equivalence: handoff synopsis → compose program → obligation
  dispatch envelope → committed topology state for representative families.

**Engineering decisions**
- worth-topo construction lane executes compose; kernel consumes obligation
  denials from runtime execution instead of pre-checking offline only.
- All seven birth families in `family_birth_input/families/` reach compose
  execution or explicit certification exclusion with residue audit — no silent
  skip.

**Open questions**
- None.

### Phase 17: Reference Adoption — worth-topo Operator Catalog

Migrate worth-topo topology operators: all `milestone_one_invariant_registrations()`
with touch selectors, all covered graph-composition entries, all covered
command-batch submissions, and all covered scalar topology mutation fronts.
Delete parallel enforcement.

**Relevant subsystems**
- `topology_operators/local_rewrites/`
- `validation/reference_integrity/`
- `runtime_support.rs`

**Adoption targets**
- `RewireLoopSuccessor`, wire rehome, shell membership, face inner loop, scalar
  topology mutation operators, and operator families currently implemented via
  command-batch helpers
- Delete manual `ExistingEntityIncomingRelationCountMismatch` guards
- Delete compose-bypass loop-wiring folklore and duplicate validator implementations

**Warnings**
- Do not classify adoption by "operator uses batch." Classify by covered graph
  touch meaning, descriptor source, operating world, and Query lowering path.
- Do not delete a local guard unless the migrated operator produces an
  inspectable obligation envelope or an explicit residue report explaining why
  the guard is not covered.

**Test requirements**
- Hostile operator matrices with envelope inspectability.
- Adoption manifest residue: exact-zero on listed files/patterns.
- Adversarial equivalence: operator families that lower through graph
  composition and operator families that lower through command-batch helpers
  select the same registered obligation when their touch descriptor is the
  same.
- Adversarial deletion: seeded reintroduction of covered local legality
  helpers fails the bypass audit.

**Open questions**
- None.

### Phase 18: Reference Adoption — worth-kernel Construction Surfaces

Migrate worth-kernel construction: full phase-chain legality through registered
obligations at compose boundary, motion sequencing through preflight obligations,
delete parallel offline enforcement on covered paths.

**Relevant subsystems**
- `construction/phase_chain/admitted_scaffold/`
- `construction/runtime_proof/motion/branch_runtime.rs`
- `construction/result_surface/result.rs`

**Adoption targets**
- Delete host-local layout legality on covered shell-with-hole and family paths
- Delete motion dual-pass `unreachable!` sequencing folklore
- Verify binding workflows remain clean; add operating-context gate obligations
  where binding kinds require world-sensitive checks

**Warnings**
- Handoff preparation is not execution. A construction family is not migrated
  until the covered birth path executes `compose_graph`, receives typed
  obligation evidence, and exposes that evidence through the result surface.
- Do not preserve offline phase-chain legality as a shadow authority for a
  covered family. If a local check remains, it must be named as compatibility
  residue with owner, blocker, and audit row.
- Do not let branch-preview motion sequencing rely on `unreachable!` or
  optimistic ordering. It must become a preflight sequencing obligation with a
  typed denial.

**Test requirements**
- Construction hostile matrices pass with typed obligation denials.
- Adoption manifest residue on covered kernel files.
- Adversarial family coverage: all seven primitive birth families either
  execute covered compose paths or emit explicit certification residue; silent
  skipped families fail certification.
- Adversarial result-surface proof: construction results expose obligation
  evidence sufficient for downstream certification without re-running offline
  legality.

**Open questions**
- None.

### Phase 19: Public Docs, Support Rows, And AI_README Category

Ship graph touch obligation authority as first-class product docs; re-home
graph-composition-authoring.md away from manual invariant-pack as primary path;
add AI_README category and update the existing docs that already define these
front doors so the milestone does not silently under-document real surfaces.

**Relevant subsystems**
- `crates/worth-query/docs/AI_README.md`
- `crates/worth-query/docs/authoring/graph-composition-authoring.md`
- `crates/worth-query/docs/execution/intent-admission.md`
- `crates/worth-query/docs/execution/writes-and-intents.md`
- `crates/worth-query/docs/foundations/branches-and-previews.md`
- `crates/worth-query/docs/domain-capabilities/configured-domain-handles.md`
- `crates/worth-query/docs/domain-capabilities/declaration-entry-orchestration.md`
- new: `crates/worth-query/docs/authoring/graph-obligation-consumer-kit.md`
- new: `crates/worth-query/docs/authoring/graph-touch-obligation-authority.md`

**Test requirements**
- Adversarial agreement: docs, support rows, and certification name the same
  obligation kinds and covered lanes.
- Adversarial consumer clarity: AI_README and the consumer-kit doc make it
  unambiguous that the consumer kit is the ordinary downstream adoption path
  for obligation registration, selector coverage, support pinning, in-memory
  proof, bypass audit, adoption manifests, and residue manifests.
- Adversarial budget clarity: docs name `BudgetExceeded`, state-load counters,
  cost classes, and artifact-policy-gated diagnostics so large graph and
  boolean-like operations are not described as unbounded automatic execution.

**Open questions**
- None.

### Phase 20: Architectural Certification Closeout

Property-based hostile matrices, complexity proofs, bypass audit closure,
full kind × lane × touch certification matrix, adoption residue.

**Relevant subsystems**
- `crates/worth-query/tests/`
- `plans/worth-query/test-requirements.md`
- `crates/worth-topo`, `crates/worth-kernel` certification tests

**Test requirements**
- Add `Milestone 9.9 Graph Touch Obligation Authority Hostile Certification
  Matrix` to [test-requirements.md](./test-requirements.md).
- Property tests, false-fire/false-miss matrices, replay equivalence, every
  obligation kind × representative touch × covered lane, adoption residue.
- Kind x lane matrix proves `Supported`, `Unsupported`, `NotApplicable`,
  `DiagnosticOnly`, and `DeferredToBackstop` rows without fake no-op executors.
- Residue matrix proves every residue row has `introduced_in`,
  `must_not_exceed_count`, and `removal_trigger`, and that residue count never
  grows after introduction.
- Reduction algebra certification proves canonical ordering and digest
  stability under equivalent multi-obligation observations.
- Execution budget certification proves broad state-load attempts produce
  `BudgetExceeded` or declared dense execution before unbounded graph walks,
  with exact state-load counters and artifact-policy-gated diagnostics.
- Consumer kit certification proves a reference downstream crate can adopt a
  covered obligation with no local registration, selection-inspection,
  support-pinning, in-memory-workspace, bypass-audit, or manifest ceremony.

**Open questions**
- None.

## Admitted Surface

The admitted product surface after this milestone is the Query-owned graph
touch obligation authority:

- obligation kinds, verdicts, dispatch plans, and dispatch envelopes are sealed
  Query artifacts
- touch descriptors and operating-world descriptors are the only ordinary
  inputs to obligation selection
- runtime assembly indexes registered obligations and exposes support,
  complexity, build, and selection counters
- relational graph-composition execution point exists for covered relational
  invariant semantics
- authoritative mutation, read, live-read, preview/branch, declaration-entry,
  contribution, policy-aware, and construction fronts consume the same
  selection/envelope authority when they carry covered graph meaning
- receipts, decision traces, denials, declared mutation artifacts, and
  downstream mutation evidence expose obligation evidence without caller-side
  recomputation
- execution budgets, cost classes, and state-load counters make broad
  obligation execution deniable before unbounded graph walks
- the consumer obligation kit lets downstream crates register, inspect, pin,
  audit, test, and certify adoption without local pseudo-Query ceremony
- `worth-topo` and `worth-kernel` covered reference consumers have deleted
  duplicate local legality on migrated paths

The admitted surface is not "all graph-looking helpers." It is only the
covered front doors that can produce canonical touch descriptors and
operating-world descriptors and can attach the resulting envelope to a
runtime-owned artifact.

## Excluded Surface

This milestone does not claim:

- automatic graph read index provisioning; that belongs to Milestone `9.10`
- store-backed durable obligation replay beyond the runtime artifacts this
  milestone emits
- branch-local batch mutation APIs that do not exist today
- topology truth ownership inside Query; `worth-topo` and relational remain
  authoritative for their respective truth semantics
- final boolean operation runtime performance closure; this milestone removes
  legality duplication and obligation bypasses, but full large-op access
  planning and background index provisioning are `9.10`
- arbitrary consumer middleware hooks around obligation execution
- manual invariant-pack hooks as the ordinary covered enforcement path
- unbounded rich diagnostic materialization on hot paths
- unbounded obligation execution after `BudgetExceeded`; the typed denial is a
  stop point, not permission to keep walking state in the background

## Workflow Surface

The ordinary mutation workflow after closure is:

```text
author graph/read/mutation intent
-> derive canonical graph touch descriptor
-> bind operating-world descriptor
-> select obligations from assembled index
-> materialize dispatch plan and envelope
-> execute allowed obligations at the owning authority
-> attach envelope to receipt / trace / denial / artifact
```

The ordinary reference-consumer migration workflow is:

```text
identify covered local legality or validator dispatch
-> register equivalent obligation with selector and rule identity
-> prove matching touch descriptor selects it
-> pin expected support posture and executor budget
-> wire execution and envelope attachment
-> delete local duplicate
-> record zero-residue or explicit residue in adoption manifest
```

## Deletion Targets

The milestone is not closed until covered duplicates are deleted or named as
audited residue. The expected deletion target classes are:

- host-local topology legality graphs for ownership, containment, loop wiring,
  shell membership, face-inner-loop legality, reference integrity, and splice /
  move eligibility on covered paths
- caller-side "remember to run this invariant" closures where the touch shape
  can be represented by selectors
- ordinary-path `compose_graph_with_invariant_pack(...)` prechecks that stand
  in for registered obligation dispatch
- duplicate implementations of milestone-one topology invariant rule identities
  outside relational/Query obligation execution
- `worth-topo` operator-local guards such as incoming-relation-count mismatch
  checks when a registered obligation covers the same rule identity
- `worth-kernel` offline shell-with-hole and family-layout legality on covered
  construction birth families
- branch-preview motion sequencing guarded only by `unreachable!` or local
  optimistic ordering
- docs and AI guidance that teach local legality graphs, manual invariant
  packs, or validator-selection tables as ordinary covered paths

## Allowed Residue

Residue is allowed only when all of the following are true:

- the covered touch descriptor or operating-world descriptor cannot yet be
  produced from the real surface
- the missing descriptor or authority seam is named
- the owning crate and follow-on phase are named
- `introduced_in` names the phase that created or discovered the residue
- `must_not_exceed_count` caps the allowed number of residue rows in that class
- `removal_trigger` names the concrete artifact, executor, or surface whose
  completion requires deletion
- the local guard cannot be mistaken for ordinary supported Query authority
- a bypass audit row or adoption manifest row records the residue
- certification proves the residue does not apply to a covered lane

Residue is not allowed for:

- convenience wrappers around a covered surface
- local validator selection when selectors can represent the same touch
- handoff-only construction birth paths that never execute `compose_graph`
- branch/preview mutation paths whose current payload already exposes covered
  graph touch meaning
- documentation drift
- residue classes whose count grows after introduction

## Operator Closure

`worth-topo` operator closure requires:

- every covered operator declares or derives graph touch meaning
- every covered operator binds an operating world through configured domain
  handle context or an explicitly admitted default
- operator evidence exposes the obligation envelope digest
- migrated operators no longer run duplicate local validators on the ordinary
  path
- non-migrated operators appear in the adoption manifest with explicit residue
  and blocker

## Kernel Construction Closure

`worth-kernel` construction closure requires:

- construction authoring enters through configured domain handles with
  operating context
- each covered primitive birth family executes a Query `compose_graph` program
  rather than stopping at admitted handoff preparation
- shell-with-hole, family-layout, and motion sequencing legality are expressed
  as registered obligations or explicit audited residue
- construction result surfaces carry obligation evidence downstream
- binding workflows remain compatible with the new operating-context gate

## Complexity / Proof Closure

The complexity proof surface must include:

- index build complexity contract and exact build counters
- dispatch selection complexity contract and exact selection counters
- zero full-catalog scan count on indexed selection
- no-match selections that still report lookup effort without fake candidates
- kind x lane x touch matrix breadth counters
- execution budget counters and budget-exceeded denial proof
- state-load counters for every executor family on covered lanes
- evidence that envelope attachment does not force rich diagnostics on every
  hot path unless policy asks for them
- adoption audits proving duplicate deletion does not replace one broad scan
  with another local broad scan
- consumer-kit certification proving adoption can be performed through
  Query-shipped registration, support-pin, manifest, audit, and in-memory
  workspace surfaces

## Milestone Done When

This milestone is done only when:

- the artifact ladder is implemented through envelope attachment
- every covered surface matrix row has tests and evidence
- every obligation kind executes on at least one representative covered lane
  and has certification coverage across the matrix
- every covered reference-consumer duplicate has been deleted or explicitly
  audited as residue
- `worth-topo` and `worth-kernel` prove adoption with real runtime evidence
- docs and AI guidance teach graph touch obligation authority as the ordinary
  path and do not teach stale folklore
- docs and AI guidance make the consumer obligation kit impossible to miss as
  the replacement for downstream local ceremony
- execution budget, state-load counter, and `BudgetExceeded` semantics are
  certified for broad graph/topology execution attempts
- the closeout includes support rows, test-requirements matrix, bypass audit
  result, and adoption manifests

## Must Ship

- complete obligation authority model with multi-obligation envelopes
- graph touch descriptors for mutation and read lanes
- read-specific touch verbs for observation, exposure, diagnostics,
  subscription retention, policy basis, stale basis, and operating-world
  crossings
- registration, auto-indexing, assembly index with complexity contracts
- executor contracts with declared state/basis access policy
- execution status taxonomy separate from public verdict taxonomy
- deterministic multi-obligation reduction algebra
- kind x lane support matrix without fake no-op executors
- operating-world execution posture for authoritative, preview, branch,
  promotion, read, and diagnostic-only lanes
- execution budgets, cost classes, state-load counters, and
  budget-exceeded policy
- relational graph-composition execution point and rule migration
- authoritative mutation intent admission integration for graph composition,
  authoritative command batch, scalar mutation, explicit submission batch, and
  effect-triggered write-intent fronts when they carry covered graph touch
  meaning
- declaration-entry and contribution-orchestration dispatch
- read execution, read composition, and live-read dispatch across existing
  helper and intent front doors
- preview direct mutation and branch/preview intent parity
- policy-aware graph mutation and operating-context gate execution
- advisory, capability-gap, and preflight-sequencing executors
- envelope attachment to receipts, decision traces, and mutation evidence
- derived read validation re-homed
- consumer obligation toolset and bypass audit
- consumer support pins, adoption manifests, residue manifests, and
  in-memory obligation-capable test workspace
- kernel construction operating context wiring
- primitive construction birth compose execution (all covered families)
- full worth-topo operator catalog adoption
- full worth-kernel construction surface adoption
- public docs and AI_README category
- architectural certification matrix closure

## Must Preserve

- relational invariant execution authority
- typed graph-composition domain-invariant denials on block paths
- denial projection as a view over obligation envelope evidence, not a second
  rule authority
- distinction between pure selection/planning and stateful execution
- distinction between public verdict and execution status
- distinction between budget denial and executor failure
- consumer kit as replacement ceremony, not optional convenience around local
  pseudo-Query adoption layers
- declaration legality and support admission as upstream lanes obligations consume
- reference-consumer semantics through migration

## Acceptance Evidence

- property-test certification of pure-function dispatch on every authoritative
  mutation lane plus covered read/live/preview-intent lanes
- authoritative mutation intent admission carries obligation dispatch for
  graph composition, command-batch, scalar, explicit-submission, and
  effect-triggered fronts where they carry covered graph touch meaning —
  manual invariant-pack pre-hook eliminated on covered compose paths
- primitive construction birth executes compose_graph with obligation routing
  for covered families
- every obligation kind executes in certification matrix across representative
  touches and lanes
- kind x lane support matrix proves supported / unsupported / not-applicable /
  diagnostic-only / deferred-to-backstop posture without no-op theatre
- executor contracts prove the legal state/basis input shape for relational,
  policy, preview, branch, construction, and read obligation execution
- execution budgets prove broad obligation execution stops or takes a declared
  dense path before unbounded state load, with exact state-load counters
- reduction algebra proves deterministic envelope digests for duplicate
  observations and mixed verdict sets
- read-specific descriptor vocabulary prevents mutation validator overfire and
  derived-read underfire
- preview / branch posture proves exploratory invalidity, diagnostic debt,
  branch policy, and promotion reconciliation without pretending preview is
  authoritative
- every denial projection cites the obligation and envelope evidence chain
- consumer-kit proof shows a downstream crate can register obligations, inspect
  selection/statuses, pin support posture, audit local ceremony, run in-memory
  obligation-capable proof, and emit manifests without private wrappers
- policy-aware mutation gates and preflight sequencing certified
- exact-zero duplicate implementations and manual pre-check residue on adoption
  manifests
- full milestone-one topo registrations indexed; kernel phase-chain covered
  surfaces migrated
- envelopes inspectable on receipts and intent decision traces
- bypass audit and certification matrix pass

## Sequencing Notes

- After Milestone `9.8` (consumer backend + bypass audit machinery).
- Before Milestone `10` (store-backed execution inherits complete obligation
  authority).
- Phases 1–5: vocabulary, index, relational execution point.
- Phase 6: canonical dispatch seam (authoritative mutation intent admission)
  before surface-specific wiring (7–9). Phase 6 must use the terminology lock:
  "batch" alone is never a sufficient surface name.
- Phases 10–12: remaining executors and envelope attachment.
- Phases 13–14: re-homing, consumer obligation kit, and bypass audit.
- Phase 15: kernel operating context before birth compose (16).
- Phases 17–18: adoption after execution surfaces exist (16) and platform
  context (15).
- Phases 19–20: docs then certification close strictly last.
