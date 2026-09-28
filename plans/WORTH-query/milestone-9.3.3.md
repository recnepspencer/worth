# Milestone 9.3.3 Engineering Spec: Authority-Scoped Effect Execution Pipeline

> **Status:** Completed.
>
> **Roadmap parent:** [worth_query_roadmap.md](./worth_query_roadmap.md)
>
> **Vision parent:** [worth_query_vision.md](./worth_query_vision.md)
>
> **Prior milestone:** [milestone-9.3.2.md](./milestone-9.3.2.md)
>
> **Foundational precedent:** [milestone-5.5.md](./milestone-5.5.md)
> established query-authored mutation, merge, and writeback declarations as
> authority-preserving workflow surfaces. Milestone 9.3.3 must turn those
> declaration families into one shared execution pipeline rather than leaving
> each executor to rediscover authority and strategy locally.
>
> **Next milestone:** [Milestone 9.3.4](./worth_query_roadmap.md#milestone-934-declared-projection-consumption-and-materialized-fact-receipts)
> continues the runtime API stabilization path by making effect-produced
> materializations expose typed fact-consumption receipts instead of forcing
> consumers to reopen source authority after execution.
>
> **Test requirements:** [test-requirements.md](./test-requirements.md)
>
> **Primary architectural driver:** make every admitted Query effect execute
> from one lowered, proof-bearing authority plan rather than from raw intents,
> ambient basis, or host-selected execution strategy.
>
> **Companion docs:**
> - [MENTALITY.md](../../docs/coding-guidelines/MENTALITY.md)
> - [arch_laws.md](../../docs/coding-guidelines/arch_laws.md)
> - [composition_laws.md](../../docs/coding-guidelines/composition_laws.md)
> - [domain_structure_laws.md](../../docs/coding-guidelines/domain_structure_laws.md)
> - [perf_laws.md](../../docs/coding-guidelines/perf_laws.md)
> - [dx_laws.md](../../docs/coding-guidelines/dx_laws.md)
> - [worth_query_vision.md](./worth_query_vision.md)
> - [worth_query_roadmap.md](./worth_query_roadmap.md)
> - [test-requirements.md](./test-requirements.md)
> - [test-requirements-milestone-9_3-and-runtime-gates.md](./test-requirements-milestone-9_3-and-runtime-gates.md)
> - [milestone-5.5.md](./milestone-5.5.md)
> - [milestone-9.3.1.md](./milestone-9.3.1.md)
> - [milestone-9.3.1-closeout.md](./milestone-9.3.1-closeout.md)
> - [milestone-9.3.2.md](./milestone-9.3.2.md)
> - [milestone-9.3.3-closeout.md](./milestone-9.3.3-closeout.md)
> - [runtime-authoritative-mutation-evidence-plan.md](./runtime-authoritative-mutation-evidence-plan.md)

## Goal

Make Query effect execution a single proof-widening pipeline, but ground that
pipeline in the APIs that already exist today:

```text
RawEffectIntent
  -> NormalizedEffectIntent
  -> BasisCapability + WorkflowContextBinding
  -> QueryWorkflowDeclaration
  -> LoweredMutationIntentDeclaration
   | LoweredMergeWorkflowDeclaration
   | QueryWritebackDeclaration
  -> WORTHQueryIntentExecution / WORTHQueryWriteReceipt / WorkflowAuthorityOutcomeArtifact
  -> SelfDescribingEffectEnvelope
  -> EffectExecutionCertificationBundle
```

The names above are intentionally concrete. Milestone 9.3.3 may still add
wrappers or sharper typestate shells, but it must first consolidate the
already-exported Query workflow/effect APIs into one execution story rather
than inventing a second speculative vocabulary beside them.

Concrete adoption rule:

- if a phase already has a real exported Query type, the milestone should reuse
  that type directly or wrap it in a thin sealed newtype
- it must not introduce a second public phase artifact that carries the same
  meaning under a different name
- if an abstract phase label remains useful in the spec, the implementation
  must map it one-to-one onto the concrete API type in the same patch

Every admitted effect family must consume basis capability proof, workflow
binding proof, declaration admission proof, and lowered execution strategy
proof before any relational or bridge executor runs. No public or internal
executor may accept raw effect intent, ambient branch/preview state, loose
strategy enums, or optional-hole artifact policy bags as permission to act.

## Why This Milestone Exists

Milestone 5.5 proved that Query may author mutation, merge, workflow, and
writeback declarations without stealing relational or bridge authority.
Milestones 9.3.1 and 9.3.2 then proved that cross-runtime explanation and
basis use already require one typed proof chain. What remains open is the
execution seam itself.

Without Milestone 9.3.3, the public runtime API risks freezing an "effect"
surface that still splits into local executors choosing their own basis
rechecks, authority target, merge strategy, preview posture, writeback family,
artifact richness, or diagnostic shape. That would recreate the same failure
mode the roadmap has been closing since Milestone 5.5: Query appears to own the
daily-driver workflow surface, but the semantically important decisions still
live in hidden adapters, controller glue, or executor-local branching.

This milestone exists to make the execution seam itself honest before
projection-fact receipts, admission lattices, lower-runtime capability routing,
the runtime API freeze, and the mutation-evidence gate build on top of it.

## Governing Summaries

- `MENTALITY.md`: the hard case is not "run a mutation." It is guaranteeing
  that equivalent effect intent always lowers to the same authority-scoped plan
  and never lets executors rediscover authority under workflow, replay, or
  preview pressure.
- `arch_laws.md`: the planner must decide, the executor must consume. Effect
  execution therefore needs proof-bearing phase outputs, self-describing
  receipts, and no executor-local strategy rediscovery.
- `composition_laws.md`: declaration normalization, authority eligibility,
  lowering, executor adapters, receipt shaping, and certification are separate
  responsibilities and must not collapse into one `effects.rs` bucket.
- `domain_structure_laws.md`: Query planning, relational execution adapters,
  bridge execution adapters, receipt materialization, and certification must
  remain structurally locatable so authority never hides inside generic helpers.
- `perf_laws.md`: plan/execute conflation is the named performance failure.
  Effect admission, lowering, execution, receipt, and envelope costs need exact
  counters so "convenient" executors cannot smuggle broad rediscovery into the
  hot path.
- `worth_query_vision.md`: Query is the ordinary platform surface for branch
  workflow and mutation orchestration, but lower crates still own truth
  mutation, merge semantics, preview lifecycle, and writeback protocol meaning.
- `worth_query_roadmap.md`: 9.3.3 belongs after basis lifecycle because effects
  need basis proof before lowering, and before 9.3.4 through 9.3.6 because
  projection receipts, decision lattices, and lower-runtime routing all depend
  on one shared execution artifact.
- `test-requirements.md`: closure requires canonical machine-checkable bundles,
  hostile denial lanes, exact counters, compile-fail boundary proofs, and
  parity across alternate effect-authoring paths.
- `milestone-5.5.md`: Query already owns mutation and writeback declaration
  surfaces, but execution authority remains lower-runtime-owned and must stay
  explicit through the lowered plan.
- `milestone-9.3.2.md`: basis capability is now the required precondition for
  execution. 9.3.3 must consume that proof rather than accepting raw branch,
  preview, historical, tenant, or policy identifiers.
- `runtime-authoritative-mutation-evidence-plan.md`: later public mutation
  evidence depends on one honest effect receipt contract. 9.3.3 must define the
  execution receipt and authority explanation shape that gate will later enrich.

## Adversarial Constraint

Under direct authoritative writes, ordered batches, branch-local preview
mutation, merge execution, conflict-class denial, query-triggered writeback,
replay, and cross-runtime inspection pressure, the same canonical effect intent
must produce the same admitted authority scope, the same lowered executor
strategy, the same authority explanation, and the same receipt/envelope meaning
regardless of how the caller authored the effect. Unsupported, stale,
authority-incompatible, preview-read-only, advisory-only, basis-mismatched,
strategy-overriding, or durable-overclaim requests must fail typed and early
before any executor, mutation batch, bridge writeback, or result artifact
exists.

## Product Decision Lock

The product surface is one Query-owned effect pipeline.

The public runtime API must let downstream domains author effect intent once,
admit it once, lower it once, execute it once, and inspect one receipt/envelope
story without:

- calling raw relational or bridge executors directly
- selecting merge or writeback strategy after lowering
- supplying ambient branch, preview, or tenant state during execution
- manually reconstructing causality, authority transfer, or artifact policy
  after execution

Crate ownership is load-bearing:

- `worth-query` owns effect intent normalization, authority eligibility,
  authority-scoped planning, lowered execution plans, public execution
  receipts, Query-facing effect envelopes, support metadata, and certification.
- `worth-relational` remains authoritative for mutation truth, invariant
  enforcement, commit strategy meaning, merge execution, branch-head
  advancement, and canonical history publication.
- `worth-runtime-bridge` remains authoritative for preview-session lifecycle,
  writeback-family protocol meaning, idempotence, replay-safe causality,
  route/execution bridging, and lower-runtime evidence carried into Query
  receipts.
- `worth-signal` remains downstream of authoritative execution for invalidation
  and derived computation; 9.3.3 may reference signal-facing consequences in
  envelopes or counters, but it must not turn signal scheduling into an effect
  executor.
- `worth-store` remains deferred for store-backed effect replay, durable
  workflow continuation, and restart-stable effect envelopes.

Forbidden in `worth-query`:

- executing truth mutation, merge application, or bridge writeback directly
- accepting raw branch IDs, preview handles, tenant/policy digests, merge-class
  strings, writeback-family strings, or strategy booleans as executable
  permission
- allowing executors to replace admitted authority scope, basis posture,
  strategy identity, invariant scope, or artifact policy after lowering
- exposing separate `execute_on_branch`, `execute_preview_mutation`,
  `execute_writeback`, or host-override APIs that are really the same lifecycle
  with different names
- encoding admitted, advisory, denied, and deferred effect posture as one
  universal token with optional fields
- claiming durable replay, store-backed execution parity, or restart-stable
  effect envelopes before Milestones 10 and 11

## Existing Effect Surfaces To Consolidate

Milestone 9.3.3 does not start from nothing. It must consolidate and
strengthen existing runtime-backed effect surfaces:

- query-authored workflow declaration APIs already exported from
  `worth-query::facade::workflow`, especially:
  - `bind_workflow_context`
  - `admit_query_workflow_declaration`
  - `lower_mutation_intent_declaration`
  - `lower_merge_workflow_declaration`
  - `lower_query_writeback_declaration`
  - `LoweredMutationIntentDeclaration`
  - `LoweredMergeWorkflowDeclaration`
  - `QueryWritebackDeclaration`
- basis-explicit branch, preview, historical, tenant, and policy capability
  proof from Milestone 9.3.2, especially the `basis_lifecycle` envelopes,
  readmission support, and scoped lane witnesses already exported through the
  Query facade
- runtime API effect/intention vocabulary already exposed through the public
  facade, especially:
  - `WORTHQueryEffectBuilder`
  - `WORTHQueryEffectDeclaration`
  - `WORTHQueryEffectAdmission`
  - `WORTHQueryIntentExecution`
  - `WORTHQueryWriteCommand`
  - `WORTHQueryWriteReceipt`
  - `WORTHQueryBatchWriteReceipt`
- causal inspection and bridge explanation binding from Milestone 9.3.1 where
  effect outcomes later need inspection or offline explanation
- preview-session, writeback, causality, idempotence, loop-prevention, replay,
  request, and receipt boundary records already owned by
  `worth-runtime-bridge`
- authoritative mutation, commit-strategy, merge, transaction, branch-head,
  and history artifacts already owned by `worth-relational`
- downstream invalidation/evaluation/forensics surfaces already owned by
  `worth-signal`; these may appear in effect aftermath or explanation, but not
  as effect execution authority

The milestone should delete duplication only when the shared effect pipeline
gives the older concept a stronger home. It must not flatten meaningful
authority or family distinctions merely because they all eventually "execute."

## Initial Family Inventory

Milestone 9.3.3 should begin from an explicit family matrix so the first
implementation cannot quietly overclaim unsupported neighbors.

Initial runtime-backed family posture:

- admitted now:
  - relational mutation lowering through
    `LoweredMutationIntentDeclaration`
  - relational merge lowering through
    `LoweredMergeWorkflowDeclaration`
  - bridge writeback lowering through `QueryWritebackDeclaration`
  - ordered batch receipt shaping through `WORTHQueryBatchWriteReceipt`
- admitted with typed denial lanes:
  - preview-derived mutation or writeback requests that reach lowering without
    authoritative rebind
  - stale or exact-basis-forbidden preview merge or writeback requests
  - host strategy or authority override attempts after lowering
- explicitly deferred:
  - store-backed execution parity
  - durable replay and restart-stable envelopes
  - temporal and async/resource effect execution neighbors
- explicitly out of scope for this milestone:
  - new lower-runtime execution families that do not already lower into public
    relational or bridge facade APIs

Phase 1 must emit this matrix as code-owned support metadata rather than
leaving it implicit in prose.

## Lower-Runtime Execution Boundary Map

Milestone 9.3.3 must reuse lower-runtime execution authorities. Query owns the
pipeline wrapper, not the underlying execution semantics.

### WORTH Relational

Use `worth_relational::facade` as the authoritative truth-mutation boundary.

Query may lower into relational commit, merge, delete, update, batch, and
history-aware execution requests only through public relational facade surfaces.
The existing leverage is already explicit:

- mutation lowering already lands on
  `worth_relational::facade::commit_strategies::RawStrategyCommitRequest`
- merge lowering already lands on
  `worth_relational::facade::merge::MergeExecutionRequest`
- the relational facade already exports the next-level authority artifacts
  Query should shape around rather than recreate:
  - `ValidatedStrategyCommitPlan`
  - `LoweredStrategyCommitPlan`
  - `PreparedMergeExecution`
  - `MergeExecutionOutcome`
  - `CommitResult`
  - `RelationalRuntime`

Query may carry relational execution identity, admission class, commit
strategy, denial family, receipt digests, and counters in Query receipts. It
must not mint relational mutation batches, merge legality classes, branch-head
updates, canonical history records, or new strategy-family semantics locally.

### Runtime Bridge

Use `worth_runtime_bridge::facade` as the preview and writeback execution
boundary.

Query may lower into bridge preview-session-aware writeback or bridge-mediated
execution families only through public bridge facade surfaces. Query may carry
bridge authority family, causality basis, idempotence posture, denial family,
receipt digests, replay posture, and counters in Query receipts. It must not
define writeback-family semantics, loop-prevention semantics, or preview
authority continuity locally.

The existing leverage is already explicit:

- Query writeback lowering already lands on
  `worth_runtime_bridge::facade::BridgeWritebackDeclaration`
- the bridge runtime already exposes the full authority chain that Query must
  reuse rather than duplicate:
  - `ValidatedBridgeWritebackDeclaration`
  - `AdmittedBridgeWritebackContract`
  - `BridgeDerivedWritebackEffect`
  - `BridgeWritebackIdempotenceBasis`
  - `BridgeWritebackLoopPreventionReport`
  - `BridgeValidatedWritebackCandidate`
  - `TruthWritebackRequest`
  - `TruthWritebackReceipt`
  - `BridgeWritebackAuthorityOutcome`
  - `BridgeWritebackReplayBundle`

Milestone 9.3.3 therefore must describe Query writeback execution as
consuming and reshaping this bridge-owned contract chain, not as inventing a
parallel Query-owned writeback protocol.

### Query-Owned Pipeline Responsibilities

Query owns only:

- `RawEffectIntent` and `NormalizedEffectIntent`
- effect-family taxonomy and canonical authoring digests
- basis-lifecycle consumption and workflow binding/admission orchestration
- typed admitted/advisory/denied/deferred effect posture
- `QueryWorkflowDeclaration` and its family-specific lowered descendants
- public execution receipt shaping around `WORTHQueryIntentExecution`,
  `WORTHQueryWriteReceipt`, `WORTHQueryBatchWriteReceipt`, and
  `WorkflowAuthorityOutcomeArtifact`
- self-describing effect envelopes
- executable support metadata and certification proving executor-local
  rediscovery is impossible

Every new effect family must first prove it can be expressed as a Query
declaration lowered into an existing lower-runtime facade. If it cannot, the
implementation must stop and either request a new lower-runtime facade API or
mark the family deferred or unsupported.

## Target Developer Experience

Common-path callers should see one obvious effect flow:

```rust
let mutation_basis = query
    .basis()
    .branch_head(branch)
    .for_mutation_preparation()?
    .admit()?;

let lowered = query
    .effect(update_customer)
    .using_basis(mutation_basis)
    .admit()?
    .lower()?;

let receipt = query.execute_effect(lowered)?;
let envelope = receipt.effect_envelope();
```

Bridge writeback should look like a sibling family in the same lifecycle, not a
second execution model:

```rust
let authoritative_basis = query
    .basis()
    .branch_head(branch)
    .for_mutation_preparation()?
    .admit()?;

let writeback_receipt = query
    .effect(projected_name_writeback)
    .using_basis(authoritative_basis)
    .admit()?
    .lower()?
    .execute()?;
```

Preview-driven writeback remains a deliberate denial or explicit rebind lane,
not a silent convenience path. The DX surface must make that visible.

Typed denial must stay explicit:

```rust
match denial.kind() {
    EffectDenialKind::PreviewReadOnly { preview, requested_family } => {}
    EffectDenialKind::AuthorityFamilyMismatch { expected, observed } => {}
    EffectDenialKind::HostStrategyOverrideForbidden { requested, admitted } => {}
    EffectDenialKind::DeferredStoreBackedReplay { family } => {}
    _ => {}
}
```

DX rules:

- ordinary caller code should author effect intent, choose basis, admit,
  lower, execute, and inspect a receipt without manually constructing proof
  internals
- the ordinary caller surface must cover both relational mutation/merge and
  bridge-backed writeback as first-class effect families in the same lifecycle,
  not as one polished path plus one lower-runtime escape hatch
- expensive or boundary-crossing work must stay visually explicit through
  `admit`, `lower`, `execute`, `inspect`, or `certify`
- result envelopes should expose authority names, decision traces, structural
  deltas, integrity markers, and counters without forcing caller-side
  relational or bridge stitching
- support discovery must tell a caller whether a family is admitted, advisory,
  denied, deferred, or unsupported before execution

## Finished Surface Contract

Milestone 9.3.3 should not merely expose "some way" to execute effects. It
should intentionally produce a finished public surface that reads like a
serious framework under `dx_laws.md`.

The final code should be recognizable in six layers:

- common-path authoring
- family-complete common-path authoring
- inspectable planning
- explicit execution boundary
- receipt/envelope inspection
- denial, support, and rebind handling

### 1. Common Path Should Read Like Intent

The ordinary caller should be able to author one effect, bind one basis,
lower once, execute once, and inspect one receipt without touching lower
runtime internals or proof construction details.

Target shape:

```rust
let basis = query
    .basis()
    .branch_head(branch)
    .for_mutation_preparation()?
    .admit()?;

let receipt = query
    .effect(rename_customer)
    .using_basis(basis)
    .admit()?
    .lower()?
    .execute()?;

let envelope = receipt.effect_envelope();
```

What this code communicates:

- `basis()` is a first-class capability authoring surface, not a raw branch id
- `effect(...)` is semantic intent authoring, not executor configuration
- `using_basis(...)` is the authority-binding boundary
- `admit()` is the eligibility and denial boundary
- `lower()` is the planning and strategy-resolution boundary
- `execute()` is the expensive authority boundary
- `effect_envelope()` is post-execution explanation materialization, not part
  of authority execution itself

The finished surface should make each of those transitions visually obvious.

That obligation applies to bridge-backed writeback too. The finished common
path is not complete if mutation reads like one intentional lifecycle while
writeback still feels like dropping into bridge-specific vocabulary.

Target sibling shape:

```rust
let basis = query
    .basis()
    .branch_head(branch)
    .for_mutation_preparation()?
    .admit()?;

let writeback_receipt = query
    .effect(projected_name_writeback)
    .using_basis(basis)
    .admit()?
    .lower()?
    .execute()?;

let writeback_envelope = writeback_receipt.effect_envelope();
```

What this code must communicate:

- writeback is still authored as Query effect intent, not bridge request
  assembly
- `using_basis(...)` is still the authority-binding seam
- `admit()` is still where preview/rebind/deferred posture becomes explicit
- `lower()` is still where Query resolves the bridge-owned declaration chain
- `execute()` still crosses a visibly expensive authority boundary
- `effect_envelope()` is still derived from a receipt rather than assembled
  from ambient bridge aftermath

### 2. Advanced Path Should Expose The Plan Before Execution

Per `dx_laws.md`, the friendly path must lower into an inspectable plan before
execution. The final code should let a serious caller stop before execution and
 inspect exactly what Query resolved.

Target shape:

```rust
let basis = query
    .basis()
    .branch_head(branch)
    .for_mutation_preparation()?
    .admit()?;

let admitted = query
    .effect(rename_customer)
    .using_basis(basis)
    .admit()?;

let lowered = admitted.lower()?;

lowered.family();
lowered.authority_lane();
lowered.basis_lane();
lowered.strategy_identity();
lowered.effect_scope();
lowered.artifact_policy();
lowered.execution_cost();
lowered.concurrency_footprint();
lowered.explain();
```

This is the "mansion crown molding" layer. The finished API should let a
developer inspect:

- effect family
- lower-runtime authority owner
- basis lane and freshness posture
- resolved strategy identity
- scope / blast radius
- artifact policy
- expected execution cost class
- concurrency or conflict footprint
- typed explanation of why this was the chosen lowering

If the lowered object cannot answer those questions without reopening runtime
state, the public surface is unfinished.

### 3. Execution Should Make Boundary Responsibility Explicit

The call that crosses the authority boundary should look meaningfully heavier
than ordinary local observation.

Target shape:

```rust
let executed = lowered.execute_with(EffectExecutionOptions {
    artifact_policy: EffectArtifactPolicy::Audit,
    diagnostics: EffectDiagnosticsPolicy::Standard,
    delivery: EffectDeliveryPolicy::DeliverDerivedEffects,
})?;
```

If 9.3.3 does not introduce explicit execution options yet, the spec should
still reserve the shape: execution is where caller responsibility changes, so
that boundary must be available as a typed surface rather than hidden forever
behind zero-argument convenience calls.

Minimum boundary controls the finished surface should eventually expose for
runtime-backed execution:

- artifact policy
- diagnostics richness
- delivery/suppression posture
- cancellation/deadline, if the runtime later admits it

Even when some of these remain fixed in the first implementation pass, the
types and naming should make room for them without needing a second execution
API later.

### 4. Denials Must Be First-Class, Not Exceptional Folklore

The finished code should make typed denial handling feel normal, not like a
rare exceptional branch.

Target shape:

```rust
match query
    .effect(projected_name_writeback)
    .using_basis(preview_basis)
    .admit()
{
    Ok(admitted) => {
        let lowered = admitted.lower()?;
        let receipt = lowered.execute()?;
        use_receipt(receipt);
    }
    Err(denial) => match denial.kind() {
        EffectDenialKind::PreviewReadOnly { .. } => {}
        EffectDenialKind::ExplicitRebindRequired { .. } => {}
        EffectDenialKind::AuthorityFamilyMismatch { .. } => {}
        EffectDenialKind::UnsupportedFamily { .. } => {}
        EffectDenialKind::DeferredStoreBackedReplay { .. } => {}
        _ => {}
    },
}
```

The important DX property is not just the enum. It is that callers can handle
denials at the right phase:

- denial before lowering
- denial during lowering
- denial before execution
- deferred/unsupported surfaced through support APIs before any real attempt

No caller should need to discover preview-rebind rules by trial and error.

The writeback family is the main place this matters. A production-honest DX
surface must let a caller see:

- authoritative-basis writeback as an admitted common path
- preview-derived writeback as typed denial or explicit rebind
- store-backed or durable writeback as typed deferred posture

If the public surface only demonstrates mutation denial while expecting users
to infer writeback denial folklore, the milestone has not finished the DX job.

### 5. Support Discovery Must Be A Real API, Not A Doc Habit

Before execution, the caller should be able to ask what is supported for a
given family/basis pairing.

Target shape:

```rust
let support = query
    .effects()
    .support()
    .for_family(EffectFamily::ProjectedStateWriteback)
    .with_basis(&preview_basis)
    .lookup()?;

support.posture();
support.authority_owner();
support.requires_rebind();
support.supported_lowering();
support.receipt_family();
support.denial_kinds();
support.deferred_neighbors();
```

What this should answer concretely:

- is this family admitted, advisory, denied, deferred, or unsupported?
- who owns execution authority?
- can this basis execute directly, or must it rebind?
- what lowered plan family would result?
- what receipt family would result?
- what typed denials should the caller expect?

This should be generated from the same family inventory/support matrix that the
spec now requires, not maintained as hand-written prose elsewhere.

### 6. Batch Code Must Look Batch-Native, Not Like A For Loop

The finished surface for ordered multi-component work must look like one batch
authority flow, not repeated scalar execution.

Target shape:

```rust
let batch_receipt = query
    .effect_batch()
    .using_basis(basis)
    .push(rename_customer)
    .push(update_address)
    .push(sync_display_projection)
    .admit()?
    .lower()?
    .execute()?;

batch_receipt.write_count();
batch_receipt.authority_lane();
batch_receipt.basis_lane();
batch_receipt.batch_mutation_evidence();
batch_receipt.graph_composition_evidence();
batch_receipt.effect_envelope();
```

The final code should make it impossible to confuse this with:

```rust
for effect in effects {
    query.effect(effect).using_basis(basis.clone()).admit()?.lower()?.execute()?;
}
```

That loop shape is exactly the kind of naive trap this milestone is supposed to
prevent.

### 7. Receipts Should Be The Operational Object; Envelopes Should Be Derived

The finished code should encourage developers to treat receipts as the
authoritative execution artifact and envelopes as explanation or transport
views over those receipts.

Target shape:

```rust
let receipt = lowered.execute()?;

receipt.authority_lane();
receipt.basis_lane();
receipt.target_evidence();
receipt.declared_effect_family();
receipt.delivery_counters();
receipt.integrity_markers();
receipt.decision_trace();

let envelope = receipt.effect_envelope();

envelope.primary_result();
envelope.warnings();
envelope.trace();
envelope.structural_deltas();
envelope.integrity();
envelope.performance();
envelope.boundaries();
```

The finished implementation should let engineers read this and understand:

- receipt = proof of what authority actually executed
- envelope = self-describing public explanation artifact derived from receipt

### 8. Inspection And Diagnostics Should Exist At The Same Semantic Level

Per `dx_laws.md`, every abstraction needs an explanation surface at its own
semantic level. The finished API should let a caller ask Query to explain the
effect they authored, not spelunk lower-runtime diagnostics.

Target shape:

```rust
let forensic = receipt.materialize_diagnostics(EffectDiagnosticsRequest {
    include_trace: true,
    include_cost: true,
    include_boundaries: true,
    include_lower_runtime_evidence: true,
})?;

forensic.trace();
forensic.cost();
forensic.boundaries();
forensic.lower_runtime_evidence();
forensic.to_diagnostic_json();
```

This is especially important because `9.3.3` sits right before more public
projection and routing work. If diagnostics stay ad hoc here, later milestones
inherit the ad hoc shape.

### 9. Final Public Naming Should Teach The Lifecycle

When we are done, the public names should teach the user the intended order of
operations:

- `basis()`
- `effect(...)` or `effect_batch()`
- `using_basis(...)`
- `admit()`
- `lower()`
- `execute()` or `execute_with(...)`
- `effect_envelope()` or `materialize_diagnostics(...)`

If the final surface instead depends on vague names like:

- `run_effect`
- `perform`
- `dispatch`
- `do_writeback`
- `execute_preview_mutation`
- `apply_effects`

then we will have left too much lifecycle meaning implicit.

### 10. What The Finished Code Must Not Look Like

The milestone should explicitly reject these caller shapes:

Raw lower-runtime leakage:

```rust
let request = RawStrategyCommitRequest::new(...);
runtime.execute_strategy(request)?;
```

Ambient branch execution:

```rust
query.execute_on_branch(branch_id, effect)?;
```

Late strategy override:

```rust
query
    .effect(effect)
    .using_basis(basis)
    .admit()?
    .lower()?
    .execute_with_strategy("some-other-strategy")?;
```

Envelope-before-receipt construction:

```rust
let envelope = EffectEnvelope::new(effect_digest, metadata, counters);
```

Scalar loop disguised as batch:

```rust
effects.into_iter().try_for_each(|effect| {
    query.effect(effect).using_basis(basis.clone()).admit()?.lower()?.execute()
})?;
```

If the implementation allows those shapes to feel natural, `9.3.3` has not
finished the public design.

## Typed Phase Progression

Milestone 9.3.3 must introduce or certify this progression. Where a phase
already has a concrete API, the milestone should adopt that API as the phase
artifact instead of layering a second near-duplicate abstraction on top:

- `RawEffectIntent`
- `NormalizedEffectIntent`
- `BasisCapability + WorkflowContextBinding`
- `QueryWorkflowDeclaration`
- `LoweredMutationIntentDeclaration | LoweredMergeWorkflowDeclaration |
  QueryWritebackDeclaration`
- `WORTHQueryIntentExecution | WORTHQueryWriteReceipt |
  WORTHQueryBatchWriteReceipt | WorkflowAuthorityOutcomeArtifact`
- `SelfDescribingEffectEnvelope`
- `EffectExecutionCertificationBundle`

Rules:

- `RawEffectIntent` is the only place compatibility or legacy inputs may enter.
  It is not executable permission.
- `NormalizedEffectIntent` canonicalizes equivalent effect-authoring paths
  without deciding authority or strategy.
- basis capability from Milestone 9.3.2 is mandatory before any workflow
  binding or declaration admission occurs
- `WorkflowContextBinding` and `QueryWorkflowDeclaration` are the existing
  authority-scoped planning seam for workflow-backed mutation, merge, and
  writeback families
- family-specific lowered declarations are the only inputs that may cross from
  Query planning into relational or bridge execution authority
- `LoweredMutationIntentDeclaration` must carry the relational
  `RawStrategyCommitRequest`; `LoweredMergeWorkflowDeclaration` must carry the
  relational `MergeExecutionRequest`; `QueryWritebackDeclaration` must carry
  the bridge `BridgeWritebackDeclaration`
- public execution receipts must prove what authority executed, what was
  denied or changed, which causality and integrity markers were emitted, and
  which next transitions remain legal
- `SelfDescribingEffectEnvelope` is the public explanation boundary.
- `EffectExecutionCertificationBundle` closes canonicalization, denial,
  executor-boundary, replay, and performance obligations.

## Typestate Proof Contract

The pipeline must be enforced by type signatures rather than comments or
executor conventions.

Required proof shape:

- each phase transition consumes the immediately prior proof type and returns
  the next proof type or a typed denial
- public callers may not construct normalized intents, admitted authority
  scopes, lowered plans, receipts, or envelopes directly
- basis capability proof from Milestone 9.3.2 is mandatory input to effect
  eligibility; raw branch, preview, snapshot, policy, or tenant inputs may not
  skip that proof
- strategy identity, authority family, and artifact policy must be sealed
  fields on lowered plans, not caller-replaceable options during execution
- a digest is evidence, not permission; executors may not accept a digest where
  they need a lowered plan
- advisory, denied, and deferred outcomes must be distinct proof families that
  cannot be promoted accidentally into execution

Required compile-time enforcement:

- public constructors for normalized intents, workflow declarations, lowered
  declarations, receipts, and envelopes must be unavailable outside their
  owning modules or facade-authorized builders
- execution entrypoints must accept only the concrete lowered forms:
  - `LoweredMutationIntentDeclaration`
  - `LoweredMergeWorkflowDeclaration`
  - `QueryWritebackDeclaration`
  or one sealed enum that transparently sums them
- lower-runtime adapters and authority bridges must be `pub(crate)` behind the
  Query facade; downstream crates may see results, not call authority adapters
  directly through Query internals
- `effect_envelope()` materialization must consume a real receipt artifact; no
  public envelope constructor may accept raw digests, ad hoc structs, or loose
  metadata bags
- compile-fail tests must prove external callers cannot:
  - call execution with `RawEffectIntent`
  - call execution with `NormalizedEffectIntent`
  - call execution with admitted-but-unlowered planning artifacts
  - WORTH a lowered plan by struct construction
  - mint an envelope without a receipt

## Implementation Sequencing Contract

The phases are mandatory sequential gates, not a buffet.

An engineer implementing `9.3.3` should complete them in order, land the
artifacts for one phase, and only then move to the next. If a later phase
discovers a missing concept from an earlier phase, the correct action is to go
back and finish the earlier phase properly, not to patch around the gap in a
later module.

Hard sequencing rules:

- no execution code before lowering types exist
- no lowering code before admission and family inventory exist
- no public DX polish before the phase-typed proof chain is real
- no certification work before the caller surface, denial surface, and receipt
  surface are executable and inspectable
- no phase may smuggle unfinished work from an earlier phase behind helper
  modules, TODO types, compatibility shims, or doc-only promises

The compiler should enforce the typestate lifecycle itself; the milestone
closeout should enforce that implementation followed this sequence in a way a
future engineer can still understand.

Linear artifact chain:

1. Phase 1 produces the vocabulary and inventory.
2. Phase 2 turns that vocabulary into admitted/denied/deferred effect
   eligibility.
3. Phase 3 turns admitted eligibility into one authority-scoped planning
   surface.
4. Phase 4 turns the planning surface into the only executable lowered plans.
5. Phase 5 turns executed lowered plans into receipts, envelopes, and
   diagnostics.
6. Phase 6 certifies that the whole stack is honest, closed, and performant.

No phase may claim completion while borrowing core artifacts from a later one.

## Phases

### Phase 1: Effect Inventory, Family Taxonomy, And Intent Normalization

Purpose

Define exactly what `9.3.3` is responsible for before building any typestate,
lowering, or execution code. This phase is where we stop the project from
quietly overclaiming effect families or caller paths that do not really exist.

Practical work

- inventory the existing concrete Query effect-adjacent surfaces and sort each
  one into:
  - reused directly
  - wrapped thinly
  - adapted
  - denied in `9.3.3`
  - deferred beyond `9.3.3`
- define `RawEffectIntent` as the only compatibility/authoring ingress point
- define `NormalizedEffectIntent` as the canonical phase-1 output
- define the canonical family taxonomy for:
  - relational mutation
  - relational merge
  - bridge writeback
  - ordered batch
  - denied/rebind/deferred neighbors
- encode canonicalization rules so equivalent authoring paths normalize to the
  same meaning and intentionally different authority or strategy meaning does
  not
- write the support matrix and public-surface inventory as code-owned artifacts,
  not prose tables in the engineer's head

Required artifacts

- effect family inventory
- `RawEffectIntent`
- `NormalizedEffectIntent`
- normalization denial types
- support matrix rows:
  - family name
  - authority owner
  - admitted basis lanes
  - concrete lowered artifact
  - concrete receipt artifact
  - denial posture
  - deferred posture
- public-surface inventory rows:
  - common-path entrypoint
  - inspectable-plan entrypoint
  - support/discovery entrypoint
  - denial/rebind entrypoint
  - batch entrypoint
  - diagnostics/envelope entrypoint
  - hidden lower-runtime types

Do not start Phase 2 until

- every family in scope has an explicit posture
- no caller story relies on unnamed future types
- normalization equivalence and divergence rules are executable, not just
  described
- support/discovery data can be generated from code-owned inventory artifacts

### Phase 2: Authority Eligibility And Lane Admission

Purpose

Turn normalized effect intent plus admitted basis capability into one typed
decision about whether Query may proceed. This is the rejection-before-
construction phase. No lowering packets or execution packets should exist yet.

Practical work

- consume only `NormalizedEffectIntent` plus admitted basis capability
- reuse or extend `bind_workflow_context` and
  `admit_query_workflow_declaration` rather than inventing a second authority
  substrate if possible
- classify every request into one of:
  - admitted and ready to lower
  - denied before lowering
  - explicit rebind required
  - deferred/unsupported before execution
- introduce typed authority-lane witnesses for the lanes that can legally
  proceed
- encode all known denial families here rather than rediscovering them in
  lowering or execution
- make the support API and common `admit()` UX surface reflect the same
  decisions

Required artifacts

- `EffectAuthorityEligibility`
- admitted/advisory/denied/deferred outcome types
- authority-lane witnesses
- typed denials for:
  - stale basis
  - preview read-only
  - advisory-only execution
  - authority mismatch
  - unsupported family
  - host override attempt
  - durable/store-backed overclaim
- finished admission UX for:
  - admitted and ready to lower
  - denied before lowering
  - explicit rebind required
  - deferred/unsupported before execution

Do not start Phase 3 until

- no lowering function accepts raw or merely normalized intent
- all denial families needed by current in-scope effects occur here or as
  explicit rebind/deferred posture
- the common `admit()` path and support/discovery path agree mechanically
- preview and rebind behavior is typed and visible at the call site

### Phase 3: Authority-Scoped Effect Plans

Purpose

Bind admitted work into one planning object that carries the exact scope of
what Query is now allowed to lower. This is where the future execution
boundary becomes inspectable and explainable, but still not executable.

Practical work

- produce `AuthorityScopedEffectPlan` or a thin equivalent over
  `QueryWorkflowDeclaration`
- ensure it carries:
  - effect family
  - admitted basis proof
  - authority lane
  - invariant scope
  - preview posture
  - policy posture
  - permitted lowering families
- expose the advanced inspectable path so a caller can stop here or after
  lowering and ask serious operational questions
- make common facade methods move callers into this phase without exposing
  proof internals
- keep family distinctions explicit for mutation, merge, writeback, batch, and
  follow-up inspection

Required artifacts

- `AuthorityScopedEffectPlan` or one-to-one equivalent over existing workflow
  planning
- common-path transition from admitted effect to scoped plan
- inspectable advanced path with answers for:
  - effect family
  - authority owner
  - basis lane / freshness posture
  - strategy identity target
  - scope / blast radius
  - artifact policy
  - execution cost class
  - concurrency / conflict footprint
  - typed explanation

Do not start Phase 4 until

- there is exactly one authority-scoped planning model for in-scope effects
- no engineer has to keep a "workflow declaration model" and an unrelated
  "effect plan model" in sync by hand
- the advanced inspection path can explain the future lowering honestly
- the common path and advanced path both terminate at the same planning truth

### Phase 4: Lowered Execution Plans And Executor Boundaries

Purpose

Turn authority-scoped plans into the only executable artifacts the system will
accept. This is the last planning phase and the first phase where lower-
runtime packets may exist.

Practical work

- lower mutation work into `LoweredMutationIntentDeclaration`
- lower merge work into `LoweredMergeWorkflowDeclaration`
- lower writeback work into `QueryWritebackDeclaration`
- if introducing `LoweredEffectExecutionPlan`, keep it as a transparent sum
  over those concrete lowered artifacts
- implement typed lowering denials for anything admitted by Phase 2 but still
  not lowerable in the current authority posture
- make execution entrypoints accept only lowered artifacts
- shape the execution API so the authority boundary looks like a real boundary,
  not a cheap helper
- reserve room for later caller-owned execution controls such as artifact
  policy and diagnostics posture even if some remain fixed in the first pass

Batch execution contract:

- ordered batch execution must lower once for the whole batch, not by looping
  over scalar execution entrypoints that each rediscover basis, authority, or
  strategy
- batch lowering must preserve one batch-scoped authority lane and one
  batch-scoped basis lane
- if the batch mixes authority lanes or basis lanes, denial must occur before
  execution begins
- the batch public path must be visibly batch-native, such as
  `effect_batch() -> using_basis(...) -> admit() -> lower() -> execute()`

Required artifacts

- only-executable lowered plan forms:
  - `LoweredMutationIntentDeclaration`
  - `LoweredMergeWorkflowDeclaration`
  - `QueryWritebackDeclaration`
  - optional sealed `LoweredEffectExecutionPlan` sum type
- lowering denial types
- compile-fail or structural tests proving executors reject:
  - raw effect intent
  - normalized effect intent
  - eligibility outcomes
  - scoped-but-not-lowered plans
- explicit execution surface
- batch-lowering path and batch-lane denial path

Do not start Phase 5 until

- no runtime-backed execution path accepts anything weaker than a lowered plan
- scalar and batch execution both flow through lowered artifacts rather than
  ad hoc rediscovery
- compile-fail boundaries prove the execution fence is real
- the advanced path can inspect the lowered result before execution

### Phase 5: Effect Execution Receipts And Self-Describing Envelopes

Purpose

Make execution outputs honest, useful, and operable. This phase is where the
public surface stops being "something that can run" and becomes "something an
engineer can rely on, inspect, and explain."

Practical work

- shape execution outputs around existing receipt vocabulary:
  - `WORTHQueryWriteReceipt`
  - `WORTHQueryBatchWriteReceipt`
  - `WORTHQueryIntentExecution`
  - `WorkflowAuthorityOutcomeArtifact`
- make those receipts the canonical operational artifact
- derive envelopes from receipts rather than constructing sibling result bags
- expose a Query-level diagnostics path so callers can explain effects at the
  same semantic level they authored them
- wire support metadata and transition rules to the real receipt families

Envelope honesty contract:

- the envelope must be a derivation of the receipt, not a sibling artifact
  assembled from ambient runtime state
- every field in the envelope must name its source receipt, lowered plan, or
  lower-runtime authority artifact
- if a field cannot be derived from admitted planning artifacts, the receipt,
  or lower-runtime returned authority artifacts, it does not belong in the
  `9.3.3` envelope
- the public split must be:
  - receipt for operational proof
  - envelope for self-describing explanation
  - diagnostics materialization for richer forensic detail under artifact
    policy

Required artifacts

- receipt families for admitted relational and bridge effects
- self-describing envelopes
- receipt-first inspection API
- diagnostics materialization API
- transition rules for inspect/materialize/project/replay/defer

Do not start Phase 6 until

- receipts are the canonical operational object
- envelopes and diagnostics are derived from receipts rather than ambient
  runtime reconstruction
- support metadata, DX transcripts, and actual receipt behavior agree
- the caller can explain an effect through Query without reopening lower-
  runtime facades

### Phase 6: Certification, Replay Honesty, And Public Boundary Closure

Purpose

Prove that the whole sequence is closed, enforced, and production-honest. This
phase is not where we invent behavior; it is where we verify the prior five
phases left no soft spots.

Practical work

- assemble `EffectExecutionCertificationBundle`
- certify proof shape, phase progression, boundary closure, replay honesty, and
  performance posture
- certify the finished public caller surface, not just internal data flow
- prove denial, rebind, batch, and diagnostics paths are real executable
  surfaces rather than docs-only aspirations

Required artifacts

- certification bundle with admitted, advisory, denied, deferred, and mismatch
  rows
- proof-shape certification
- compile-fail boundary certification
- golden DX transcript certification
- replay/parity certification
- performance certification

Phase 6 is complete only when

- a future engineer could read the finished public API and predict the
  lifecycle from names alone
- a hostile caller cannot bypass the proof chain
- an operator can inspect support posture, lowered shape, receipt meaning, and
  diagnostics without lower-runtime spelunking
- all prior phase obligations are covered by certification artifacts rather
  than trust

## Must Ship

- phase-typed effect pipeline artifacts from raw intent through certification
- authority-scoped runtime-backed effect families for admitted relational and
  bridge execution lanes
- lowered execution plans that carry basis proof, authority family, strategy
  identity, invariant scope, artifact policy, and diagnostics posture
- self-describing effect receipts and envelopes
- compile-fail proof boundaries preventing executor use of weaker plan types
- support metadata synchronized with executable eligibility and lowering facts
- exact counters and slope digests for normalization, eligibility, lowering,
  execution, receipt materialization, envelope materialization, and support
  lookup
- one code-owned family inventory/support matrix that fails closed for
  unsupported, deferred, or rebind-required neighbors
- one finished public caller surface matching the Finished Surface Contract:
  - common-path intent authoring for both relational mutation/merge and
    bridge-backed writeback
  - inspectable advanced lowering path
  - explicit denial/rebind handling path
  - support/discovery path
  - batch-native execution path
  - receipt-first explanation/diagnostics path

## Must Preserve

- Query owns declaration, admission, lowering, receipt shaping, and
  certification, not mutation truth or writeback protocol meaning
- relational remains authoritative for mutation truth, merge execution, and
  canonical history
- runtime bridge remains authoritative for preview lifecycle, writeback
  semantics, idempotence, replay-safe causality, and lower-runtime execution
  evidence
- branch mutation, preview mutation, merge, and writeback are parameters of one
  lifecycle rather than sibling ad hoc APIs
- diagnostic richness remains a cold-path envelope concern rather than an
  execution-time strategy choice
- store-backed replay, durable workflow continuation, and restart-stable effect
  envelopes remain explicitly unsupported in `9.3.3` and owned by later
  milestones rather than being treated as carry-forward implementation debt

## No Debt Escape Hatch

This milestone does not permit "temporary debt" for any runtime-backed family
or caller story that it claims to support.

Allowed:

- explicit unsupported posture for surfaces owned by later milestones
- explicit deferred posture for future store-backed, durable, temporal, or
  async/resource capabilities that are not claimed as part of `9.3.3`

Not allowed:

- shipping an in-scope runtime-backed family behind a "compatibility debt"
  label
- leaving an ordinary caller path half-designed and calling the remainder
  future cleanup
- accepting weaker proof boundaries now because later milestones might tighten
  them
- landing ad hoc helper paths, hidden alternate entrypoints, or facade escapes
  and calling them temporary

If a runtime-backed `9.3.3` surface exists in the public API, it must either:

- satisfy the full `9.3.3` proof chain and DX requirements now
- deny or reject typed and early now
- remain fully absent from the public surface until its owning milestone

## Acceptance Evidence

This milestone is complete only when `worth-query` can prove:

- the named `Authority-Scoped Effect Execution Pipeline Test` passes with
  canonical machine-checkable artifacts
- equivalent effect authoring paths normalize to the same lowered plan and
  execution receipt meaning
- intentionally different authority family, strategy identity, basis posture,
  or artifact policy changes the declared digest fields
- stale, preview-read-only, authority-mismatched, advisory-only, unsupported,
  host-override, and durable-overclaim requests fail typed and early before an
  executor runs
- ordinary public effect surfaces consume lowered plans rather than raw intent
  or scoped-but-unlowered plans
- bridge-backed writeback has a first-class public caller transcript that reads
  like Query intent authoring rather than lower-runtime bridge request
  assembly
- effect envelopes expose authority names, decision traces, structural deltas,
  integrity markers, and counters without caller-side lower-runtime stitching
- performance certification proves executor strategy was resolved upstream and
  not rediscovered during execution
- batch certification proves the implementation did not fall back to scalar
  re-admission or per-component authority rediscovery
- preview-to-authority denial or rebind lanes are explicit in DX, support
  metadata, and compile-fail coverage rather than being left as caller folklore
- golden DX certification proves the final public code actually looks like the
  intended framework surface rather than merely exposing the same capabilities
  through lower-level APIs

## Required Verification Output

The 9.3.3 certification bundle must emit:

- `query_digest`
- `raw_effect_intent_digest`
- `normalized_effect_intent_digest`
- `effect_family_digest`
- `effect_authority_digest`
- `effect_basis_digest`
- `effect_scope_digest`
- `effect_policy_digest`
- `effect_strategy_digest`
- `effect_eligibility_digest`
- `authority_scoped_effect_plan_digest`
- `lowered_effect_execution_plan_digest`
- `effect_execution_receipt_digest`
- `effect_envelope_digest`
- `relational_effect_authority_digest`
- `bridge_effect_authority_digest`
- `effect_decision_trace_digest`
- `effect_structural_delta_digest`
- `effect_integrity_marker_digest`
- `effect_target_dx_digest`
- `effect_golden_transcript_digest`
- `effect_support_matrix_digest`
- `effect_proof_shape_digest`
- `effect_phase_progression_digest`
- `effect_replay_parity_digest`
- `compile_fail_boundary_digest`
- `failure_digest`
- `counter_snapshot`
- `effect_normalization_slope_digest`
- `effect_eligibility_slope_digest`
- `effect_lowering_slope_digest`
- `effect_execution_slope_digest`
- `effect_receipt_materialization_slope_digest`
- `effect_envelope_materialization_slope_digest`
- `effect_support_lookup_slope_digest`

## Architectural Notes

- Basis proof is a precondition, not an executor option. 9.3.3 must consume the
  capability lifecycle from 9.3.2 instead of rebuilding it.
- Query already has two adjacent but not yet unified surfaces:
  - workflow declaration/lowering in `worth-query::workflow`
  - runtime mutation/effect/write receipt vocabulary in `worth-query::runtime`
  Milestone 9.3.3 should make those tell one story instead of preserving them
  as parallel concepts.
- The most likely naive implementation failure is building a fresh "effect
  pipeline" module that duplicates the already-real workflow lowering path.
  The milestone must instead converge those surfaces.
- One shared lifecycle does not mean one flattened effect enum with optional
  fields. Family differences that change authority, failure shape, or cost
  posture must remain explicit.
- A lowered plan is the execution contract. Anything less is still planning.
- Receipts and envelopes are not optional diagnostics sugar. They are the
  public evidence that execution stayed inside admitted authority.
- `worth-signal` is part of effect aftermath, invalidation, and explanation.
  It is not part of the authority boundary for mutation, merge, or writeback.
- If a caller can replace authority family, merge strategy, or writeback family
  after lowering, the milestone has failed.
- If Query needs a private lower-runtime import to execute an effect, the
  owning crate boundary is incomplete and the effect family must stop at
  deferred or unsupported posture.

## Store Dependency

Runtime-backed authority-scoped effect execution is not blocked on
`worth-store`.

The following remain deferred:

- store-backed effect execution parity
- durable effect replay
- persisted workflow continuation artifacts
- restart-stable effect envelopes
- portable effect receipt import/export

Those belong to Milestones 10 and 11 or later follow-on work.

## Sequencing Notes

Milestone 9.3.3 belongs after 9.3.2 because authority-scoped execution cannot
be honest until basis capability proof is already explicit and typed.

It belongs before 9.3.4 because projection fact consumption should bind to
effect-produced receipts and envelopes rather than reopening source authority to
discover what happened.

It belongs before 9.3.5 and 9.3.6 because the admission lattice and
lower-runtime routing need one canonical execution artifact to classify and
route instead of multiple executor-specific stories.

It belongs before the Runtime API Public Stabilization Gate because effect
execution is part of the public daily-driver facade, and freezing that facade
before this seam is closed would bake executor-local rediscovery into the API.

## Closeout Standard

This milestone may close only when:

- every ordinary admitted runtime-backed Query effect surface executes from
  `LoweredEffectExecutionPlan`
- admitted relational and bridge execution families share the same proof chain
  from raw intent through receipt
- compile-fail boundaries prove external callers cannot mint normalized
  intents, admitted scopes, lowered plans, receipts, or envelopes
- support metadata, executable behavior, and certification coverage agree for
  admitted, advisory, denied, deferred, and unsupported effect families
- the final public API visibly supports all six caller stories from the
  Finished Surface Contract without forcing lower-runtime imports or ad hoc
  helper folklore, and the common-path story is family-complete rather than
  mutation-only:
  - relational mutation/merge common path
  - bridge-backed writeback common path
- no in-scope runtime-backed surface is carried by compatibility debt, cleanup
  debt, or undocumented follow-on promises
- roadmap and test-requirement references point at this spec and named suite
  accurately

## Self-Check

- Does the milestone solve a real structural problem or just package work
  cosmetically? Yes: it closes the execution seam where authority could still
  drift out of the proof chain even after declaration and basis work exist.
- Is the adversarial constraint precise and load-bearing? Yes: it forbids
  executor-local rediscovery of authority, strategy, basis, and artifact policy
  under real workflow pressure.
- Does the milestone preserve crate authority boundaries? Yes: Query owns the
  pipeline while relational and bridge remain the only execution authorities.
- Does the milestone define proof obligations, not just implementation tasks?
  Yes: canonicalization, eligibility, lowering, executor-boundary closure,
  receipt shaping, replay parity, compile-fail boundaries, and exact counters
  are all required.
- Could a competent engineer map this spec into honest types, modules, and
  tests? Yes: the phase progression names the pipeline artifacts, denials,
  receipts, envelopes, and certification bundle required for implementation.
- Does the milestone belong in this roadmap sequence, or is it out of order?
  Yes: it extends the basis lifecycle into execution before projection
  consumption, admission unification, lower-runtime routing, and API freeze.
