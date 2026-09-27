# Milestone 9.3.2 Engineering Spec: Query Basis Capability Lifecycle

> **Status:** Closed as of 2026-05-13; see
> [milestone-9.3.2-closeout.md](./milestone-9.3.2-closeout.md)
>
> **Roadmap parent:** [worth_query_roadmap.md](./worth_query_roadmap.md)
>
> **Vision parent:** [worth_query_vision.md](./worth_query_vision.md)
>
> **Prior milestone:** [milestone-9.3.1.md](./milestone-9.3.1.md)
>
> **Next milestone:** [Milestone 9.3.3](./worth_query_roadmap.md#milestone-933-authority-scoped-effect-execution-pipeline)
> continues the runtime API stabilization path by making Query effects consume
> lowered authority-scoped plans rather than re-deciding basis, authority,
> strategy, or artifact policy during execution.
>
> **Test requirements:** [test-requirements.md](./test-requirements.md)
>
> **Primary architectural driver:** make every Query basis a phase-typed
> capability lifecycle instead of a raw branch, preview, tenant, policy,
> snapshot, or historical identifier.
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
> - [milestone-9.3.1.md](./milestone-9.3.1.md)
> - [milestone-9.3.1-closeout.md](./milestone-9.3.1-closeout.md)

## Goal

Make Query basis selection, admission, use, and explanation a proof-bearing
capability lifecycle. Observation, mutation, replay, inspection, and
materialization must consume basis capabilities that prove their authority,
scope, visibility, lifecycle posture, and permitted next transitions. No public
or internal Query surface may treat a raw branch, preview, tenant, policy,
snapshot, historical, or runtime snapshot identifier as permission to act.

## Why This Milestone Exists

Milestones 5.2, 6, 9, 9.1, 9.2, 9.3, and 9.3.1 already introduced real basis
meaning: branch and preview contexts, historical materialization posture,
tenant and policy narrowing, subscription basis posture, bridge parity basis
digests, and causal inspection basis evidence. Those surfaces work only if the
basis is more than a string. It is a capability that says which authority named
the truth, which operation may consume it, what lifecycle phase it is in, and
which downstream transitions are still honest.

Without Milestone 9.3.2, the runtime API stabilization path risks freezing a
public facade where basis meaning is scattered across branch admission,
preview admission, read-composition preflight, subscription lowering,
inspection anchors, and bridge evidence digests. That would invite domains to
pass raw branch or snapshot identifiers around as implicit permission tokens.
The result would be stale mutations, replay against the wrong authority,
inspection materialized from an incompatible basis, and later temporal/async
milestones forced to create parallel basis APIs.

This milestone consolidates existing basis vocabulary into one phase-typed
lifecycle before effect execution, projection consumption, admission lattices,
lower-runtime capability routing, temporal basis, async resources, store
parity, and durable reload build on top of it.

## Governing Summaries

- `MENTALITY.md`: the adversarial case is a stale or incompatible basis being
  accepted because it looks like a normal identifier. The spec must make
  basis permission mechanically proven before any read, write, replay,
  inspection, or materialization can proceed.
- `arch_laws.md`: phase outputs must become phase inputs. Basis must progress
  through raw intent, eligibility, admitted capability, scoped use, use receipt,
  and self-describing envelope without later phases defensively rediscovering
  what earlier phases should have proven.
- `composition_laws.md`: basis normalization, eligibility, scoped use,
  receipt construction, and certification are separate responsibilities. This
  milestone must not hide them inside a broad `basis.rs` or a god admission
  function.
- `domain_structure_laws.md`: authoritative truth basis, derived Query
  capability, runtime-bridge authority basis, tenant/policy narrowing, and
  diagnostic envelopes have different structural fates and must remain
  physically locatable.
- `perf_laws.md`: basis proof is on a control path that many surfaces consume.
  Eligibility, normalization, operation-use, receipt, and envelope
  materialization need exact counters so capability richness does not become a
  hidden broad scan.
- `worth_query_vision.md`: branch, time-travel, tenant, policy, live
  promotion, replay, and inspection are ordinary Query responsibilities, but
  Query must not own relational truth, storage recovery, signal scheduling, or
  runtime-bridge authority.
- `worth_query_roadmap.md`: 9.3.2 sits between cross-runtime causal inspection
  and authority-scoped effect execution because effects, projections,
  admissions, lower-runtime routing, temporal basis, async resources, and the
  public API gate all need one common basis capability model.
- `test-requirements.md`: closure requires canonical machine-checkable
  artifacts, hostile lanes, exact counters, typed failures, and parity across
  alternate construction paths rather than builder-path happy tests.
- `milestone-9.3.1-closeout.md`: causal inspection is closed with basis
  evidence present in anchors and bridge envelopes, but durable causal reload
  remains deferred. 9.3.2 must reuse that runtime-backed evidence posture
  without claiming store-backed or restart-stable basis envelopes.

## Adversarial Constraint

A consumer must not be able to observe, mutate, replay, inspect, or
materialize against a Query basis unless Query has proven that the requested
basis family is eligible for that exact operation and has emitted a
self-describing basis envelope that names its authority, scope, visibility,
lifecycle, policy, tenant/schema posture, lower-runtime binding, permitted next
transitions, and exact cost counters. Equivalent basis intents must normalize
to the same capability envelope. Stale, inaccessible, incompatible,
operation-ineligible, policy-masked, tenant-mismatched, preview-drifted,
historical-replay-unsupported, and durable-overclaim basis requests must fail
typed and early before operational artifacts can be constructed.

## Product Decision Lock

The product surface is a Query-owned basis capability lifecycle. The public
runtime API must let downstream domains ask for current, branch, preview,
snapshot, historical, tenant-scoped, and policy-scoped work without ever
receiving a raw lower-runtime identifier as permission to act.

Crate ownership is load-bearing:

- `worth-query` owns public basis intent normalization, basis eligibility,
  admitted basis capability types, operation-specific basis use receipts,
  Query-facing basis envelopes, facade support metadata, and basis lifecycle
  certification.
- `worth-relational` remains the authority for branch, commit, snapshot,
  head, lineage, and truth visibility. Query may bind relational evidence into
  a basis capability; it may not declare relational truth.
- `worth-runtime-bridge` remains the authority for bridge continuity,
  writeback, route, evaluation, replay, preview, and lower-runtime authority
  basis records. Query may consume bridge-returned basis bindings and receipts;
  it may not mint bridge authority bases.
- `worth-signal` remains the authority for signal observation, invalidation,
  scheduling, and live evaluation basis evidence. Query may reference signal
  basis evidence for live and inspection surfaces; it may not become the signal
  scheduler.
- `worth-store` remains deferred for durable basis reload, snapshot-plus-tail
  reconstruction, and restart-stable basis envelopes.

Forbidden in `worth-query`:

- accepting raw branch IDs, snapshot tokens, preview labels, tenant IDs, policy
  digests, runtime snapshot tokens, or bridge authority-basis strings as direct
  permission to observe, mutate, replay, inspect, or materialize
- constructing read, mutation, replay, inspection, materialization, or
  subscription artifacts before basis eligibility has produced an admitted
  operation capability or typed denial
- treating current-head, branch-head, preview, snapshot, historical,
  tenant-scoped, and policy-scoped bases as interchangeable string variants
- folding basis eligibility, operation permission, lower-runtime binding,
  receipt materialization, and certification into one optional-field envelope
- claiming durable basis reload, store-restored snapshot-plus-tail parity, or
  restart-stable basis envelopes before Milestones 10 and 11
- re-validating raw lower-runtime basis identifiers in later phases instead of
  carrying the proof-bearing basis type established earlier

Allowed:

- existing branch, preview, read-composition, subscription, inspection, and
  bridge-basis vocabulary may be wrapped, renamed, split, or adapted into the
  lifecycle if the resulting proof chain is honest
- compatibility adapters may remain temporarily if they immediately lower raw
  legacy inputs into `RawBasisIntent` and expose explicit debt or denial
  posture
- temporal and async/resource placeholders may exist only as typed deferred or
  unsupported neighbors so Milestones 9.4 through 9.7 extend this lifecycle
  instead of bypassing it

## Existing Basis Surfaces To Consolidate

The spec does not assume an empty crate. It must account for existing basis
surface area and pull it into the lifecycle:

- branch and preview admission surfaces such as
  `WORTHQueryBranchBasisAdmission` and `WORTHQueryPreviewBasisAdmission`
- read-composition basis context vocabulary such as `QueryBasisContextRequest`,
  bound/admitted query basis contexts, `ExecutionBasisIntent`, snapshot basis
  resolution, and execution-basis preflight
- subscription basis posture and bridge parity basis digests from the 9.1
  through 9.3 subscription path
- causal observation anchors, causal inspection artifacts, and bridge causal
  envelopes from 9.3.1 that already preserve basis evidence
- runtime-bridge continuity, merge, writeback, route, replay, and structural
  authority basis records
- policy, tenant/schema, relationship-proof, view-shape, and branch access
  digests that currently travel alongside query execution and inspection

The milestone should delete duplication only when the lifecycle gives the old
concept a stronger home. It should not flatten meaningful distinctions just
because they all contain the word `basis`.

## Lower-Runtime API Boundary Map

Milestone 9.3.2 must wrap existing lower-runtime authority APIs. It must not
rebuild them inside Query.

### Runtime Bridge

Use `worth_runtime_bridge::facade` as the bridge boundary.

Already-owned bridge surfaces include:

- subscription basis resolution through `BridgeSubscriptionBasisRequest`,
  `ValidatedSubscriptionBasisBinding`, `BridgeSubscriptionBasisKind`,
  `BridgeSubscriptionBasisResolutionFailure`, and
  `RuntimeBridge::admit_subscription`
- standard-path truth-view evaluation through
  `BridgeTruthViewEvaluationRequest`, `BridgeTruthViewEvaluation`,
  `BridgeTruthViewSelector`, `SnapshotReadPacket`, `RuntimeBridge::evaluate`,
  and `RuntimeBridge::evaluate_current`
- advanced truth-view planning and source-backed materialization through
  `HistoricalEvaluationDeclaration`, `BridgeTruthViewAuthorityBasis`,
  `PlannedTruthViewPacket`, `AdmittedSnapshotContext`,
  `TruthSnapshotReader`, `RuntimeBridge::plan_truth_view_packet`,
  `RuntimeBridge::plan_source_packet_set`, and
  `RuntimeBridge::materialize_source_packet`
- continuity and historical lineage proof through
  `BridgeContinuityAuthorityBasis`, `BridgeLineageContext`,
  `BridgeEligibleContinuityRequestSet`, `BridgeHistoricalLineagePacket`,
  `ResolvedLineageContinuitySet`, `BridgeContinuityArtifact`,
  `RuntimeBridge::deliver_continuity`, and canonical continuity replay
- preview-scoped bridge basis through `BridgePreviewSession`,
  `BridgePreviewExecutionRecord`, `BridgeSubscriptionPreviewBasisBinding`,
  and `RuntimeBridge::admit_subscription_preview_basis`
- writeback basis through `BridgeWritebackCausalityBasis`,
  `BridgeWritebackFamilyBasis`, `BridgeWritebackStrategyBasis`,
  `BridgeWritebackIdempotenceBasis`, bridge writeback contracts, bridge
  writeback receipts, and bridge writeback execution diagnostics
- causal explanation evidence through `BridgeCausalExplanationEnvelope`,
  `BridgeCausalEvidenceBinding`, `BridgeCausalEvidenceReference`,
  `BridgeCausalEnvelopeReceipt`, and bridge causal counters

Query should prefer the bridge standard path for ordinary truth-view basis
authoring and readmission, and use the advanced planning/materialization APIs
only when a Query lifecycle phase must make the boundary crossing explicit.
Phase 1 should name which Query families map to standard-path evaluation,
which map to advanced source planning, and which remain deferred.

Query may carry bridge-owned basis identities, digests, receipts, and denial
records as lower-runtime evidence. Query must not reconstruct, clone, or mint
bridge continuity, subscription, preview, truth-view, writeback, or causal
authority bases.

### WORTH Signal

Use `worth_signal::facade` as the signal boundary.

Already-owned signal surfaces include:

- derived-runtime timeline identity through `SignalBranchId`,
  `SignalSnapshotId`, and `SignalBranchHandle`
- signal snapshot and reconstructive authority through
  `SignalSnapshotMeta`, `SignalSnapshotV1`, and `SignalCheckpointImage`
- retained replay and lineage evidence through `ReplayCursor`,
  `ReplayEvent`, `ReplaySlice`, `LineageRecord`, `LineageRecordKind`,
  `RetainedLineageView`, and `SynthesizedLineageChain`
- diagnostic and forensic access through `GraphDiagnostics`,
  `GraphForensicDiagnostics`, `DiagnosticsAvailability`, retained
  diagnostics views, and materialized provenance artifacts
- temporal, resource, and async runtime semantics through the signal facade's
  clock, resource, effect, awaiter, promise, and async execution types

Query may reference signal timeline, snapshot, replay, lineage, and diagnostic
evidence where Query live or inspection surfaces already need it. Query must
not schedule signal work, restore signal snapshots, redefine invalidation,
reinterpret signal replay, or pre-implement signal temporal/async/resource
semantics in 9.3.2.

### WORTH Relational

Use `worth_relational::facade` as the relational boundary.

Already-owned relational surfaces include:

- truth timeline identity through `BranchId`, `BranchHead`, `CommitId`,
  `CommitReference`, `VersionId`, `SnapshotId`, and `SnapshotHandle`
- truth read and snapshot access through `RelationalRuntime`,
  `RelationalReadView`, `SnapshotGuard`, `SnapshotReadPolicy`, and
  `SnapshotInspectionSummary`
- authoritative history and replay through `HistoryAccess`,
  `CanonicalCommitEnvelope`, commit envelopes, branch heads, version graphs,
  aspect history, lineage aspect history, and replay retention artifacts
- relational inspection, lineage, merge, transaction, schema, and visibility
  facade modules
- bridge adaptation through `RuntimeBridgeRelationalSource`, which already
  implements bridge `CommittedPatchSource`, `SnapshotReadSource`,
  `TruthBranchHeadSource`, and `ContinuityLineageSource`

Query may normalize public truth-basis intent and bind relational authority
evidence into Query receipts. Query must not deep-import relational history
storage, fabricate branch heads or snapshot handles, build parallel commit
envelopes, or bypass `RuntimeBridgeRelationalSource` for bridge-bound truth
flows.

### Query-Owned Adapter Responsibilities

Query owns only the lifecycle wrapper around these authority surfaces:

- public `RawBasisIntent` and `NormalizedBasisIntent`
- Query operation-lane eligibility, admitted/denied capability shaping, and
  scoped use types
- readmission of lower-runtime evidence into Query receipts and envelopes
- typed translation of bridge, signal, and relational denials into Query basis
  denial artifacts without erasing the originating authority
- support metadata and certification proving existing lower-runtime APIs were
  reused rather than rebuilt

Every phase must include an API-reuse check before adding a new basis type. A
new Query type is allowed only when it represents Query capability lifecycle
state, not when it duplicates bridge, signal, or relational authority state.

### Adapter Shape Contract

Query lower-runtime adapters are not semantic copies of lower-runtime records.
They are lifecycle readmission wrappers. An adapter may contain only:

- the owning crate name and authority family
- the owning public facade type, function, or trait used to acquire the
  evidence
- facade-returned identity values, digests, receipts, denial classes, counters,
  support posture, and authority labels
- Query lifecycle state that was proven before or after the lower-runtime call,
  such as operation lane, eligibility decision, admitted/denied capability
  digest, scoped-use digest, receipt digest, and envelope digest

An adapter must not contain:

- reconstructive bridge, signal, or relational authority fields sufficient to
  replay, restore, re-resolve, or reissue the lower-runtime record without the
  owning crate
- private-module fields copied from bridge, signal, or relational internals
- fresh Query constructors for lower-runtime branch heads, snapshot handles,
  commit envelopes, bridge subscription bindings, bridge truth-view authority,
  bridge continuity bases, bridge writeback bases, signal checkpoints, signal
  replay cursors, or signal lineage records
- lower-runtime support claims that have not been proven through the owning
  facade or explicitly marked deferred/unsupported

Phase 1 must produce a lower-runtime API reuse matrix with one row per
basis-adjacent surface. Each row must name:

- owning crate
- owning facade module/type/function/trait
- existing lower-runtime authority artifact
- Query wrapper or adapter type, if any
- allowed carried fields
- forbidden duplicate fields
- operation lanes allowed to consume it
- denial or deferred posture when the facade cannot support the requested use
- certification test or compile-fail proof that enforces the boundary

If a proposed Query adapter cannot fill this matrix row without copying
authority-owned fields, the implementation must stop and either use a narrower
facade-returned summary, request a new owning-crate facade API, or mark the
surface unsupported/deferred.

## Target Developer Experience

The legal path must be short, readable, and intent-first. Typestate proof is
the joinery; the public API still needs a finished surface.

DX laws apply directly:

- common paths should read like caller intent, not proof-object assembly
- expensive or boundary-crossing work must look explicit through verbs such as
  `admit`, `bind_lower_runtime`, `execute`, `materialize`, or `certify`
- valid next actions should appear from the current proof type; invalid next
  actions should be unavailable or visibly impossible
- digests belong in envelopes, receipts, support rows, and certification, not
  in executable permission call sites
- advanced paths may expose raw, normalized, eligible, admitted, scoped, bound,
  receipt, and envelope phases, but ordinary domain code should not manually
  thread every phase unless it is debugging or certifying the lifecycle
- denials should be typed and domain-readable before they are forensic
- lower-runtime evidence should appear in envelopes and explanation surfaces
  without requiring the caller to stitch bridge, signal, or relational APIs

Target common-path shape:

```rust
let basis = query
    .basis()
    .branch_head(branch)
    .tenant(tenant)
    .policy(policy)
    .for_observation()?
    .admit()?;

let result = query
    .read(read_graph)
    .using_basis(basis)
    .execute()?;

let envelope = result.basis_envelope();
```

Target mutation-preparation shape:

```rust
let mutation_basis = query
    .basis()
    .branch_head(branch)
    .for_mutation_preparation()?
    .admit()?;

let prepared = query
    .prepare_mutation(intent)
    .using_basis(mutation_basis)
    .prepare()?;
```

Target inspection and lower-runtime evidence shape:

```rust
let inspected = query
    .inspect(observation)
    .using_basis(admitted_basis)
    .include_lower_runtime_evidence()
    .materialize()?;

let authority_bindings = inspected.envelope().authority_bindings();
```

Target support discovery shape:

```rust
let support = query
    .basis_support()
    .for_operation(BasisOperation::MutationPreparation)
    .explain();
```

Target denial shape:

```rust
match denial.kind() {
    BasisDenialKind::PolicyMasked { policy, scope } => {}
    BasisDenialKind::StalePreview { expected, observed } => {}
    BasisDenialKind::LowerRuntimeBindingMismatch { authority, .. } => {}
    BasisDenialKind::UnsupportedFutureNeighbor { family, owner } => {}
    _ => {}
}
```

These examples are target transcripts, not mandatory final names. Final names
must follow local WORTH naming and facade conventions, but the call-site feel
must preserve these properties:

- one narrow public authoring surface for basis intent
- operation-specific admission with typed lane witnesses
- no manual construction of proof internals on common paths
- explicit handoff from admitted basis to read, mutation preparation,
  inspection, materialization, subscription, or certification
- typed support discovery before execution
- envelopes readable by a Query consumer without spelunking bridge, signal, or
  relational internals

## Typed Phase Progression

Milestone 9.3.2 must introduce or certify this progression:

```text
RawBasisIntent
  -> NormalizedBasisIntent
  -> BasisEligibility
  -> AdmittedBasisCapability | DeniedBasisCapability
  -> ScopedExecutionOrObservationBasis
  -> BasisUseReceipt
  -> SelfDescribingBasisEnvelope
  -> BasisLifecycleCertificationBundle
```

Rules:

- `RawBasisIntent` is the only place raw public or compatibility identifiers
  may enter; it is not a capability.
- `NormalizedBasisIntent` canonicalizes equivalent builder paths, branch/head
  aliases, preview labels, tenant/policy context, and snapshot/historical
  forms without deciding operation permission.
- `BasisEligibility` decides whether the normalized intent is meaningful for
  the requested family and emits success, advisory, or violation posture.
- `AdmittedBasisCapability` is sealed, operation-aware, and phase-typed. It
  names the basis family, authority source, scope, visibility, lifecycle
  posture, policy/tenant/schema posture, lower-runtime evidence references, and
  permitted operation lanes.
- `DeniedBasisCapability` is not an optional hole in the admitted type. It is a
  typed diagnostic artifact with denial family, decision trace, failure digest,
  and zero operational residue.
- `ScopedExecutionOrObservationBasis` is the only input accepted by read,
  mutation, subscription, replay, inspection, and materialization surfaces.
- `BasisUseReceipt` proves which operation consumed the capability, which
  lower-runtime binding was used, which counters were spent, and which next
  transitions remain legal.
- `SelfDescribingBasisEnvelope` is the public explanation boundary. It must be
  interpretable without asking the producing subsystem for hidden state.
- `BasisLifecycleCertificationBundle` closes canonicalization, transition,
  denial, lower-runtime binding, future-neighbor denial, and performance
  obligations.

No phase may accept a weaker type than the previous phase emits.

## Typestate Proof Contract

The lifecycle must be enforced by type signatures, not convention, comments,
digests, or runtime assertions.

Required proof shape:

- each lifecycle transition must be a named function or method that consumes
  the prior proof type and returns exactly the next proof type or a typed
  denial
- proof-bearing constructors must be sealed to their proving module; public
  callers may read proof summaries but may not synthesize proof rows
- proof-bearing fields must be private or crate-private with read-only
  accessors that cannot mutate, widen, or reclassify the proof
- operation lanes must be encoded as distinct witness types or typed wrappers,
  not as WORTHable strings, loose enums, or boolean flags on a universal token
- advisory, violation, and success outcomes must have distinct result types or
  variants whose downstream inputs make accidental promotion impossible
- a digest is evidence, not permission; no phase may accept a digest where it
  needs the proof-bearing type that produced the digest
- lower-runtime evidence must be an attached authority witness, not a re-check
  invitation for later phases
- compatibility inputs are permitted only as entry adapters into
  `RawBasisIntent`; they may not produce admitted capabilities directly

The intended transition API shape is:

```text
normalize_raw_basis(RawBasisIntent) -> NormalizedBasisIntent | BasisIntentDenial
evaluate_basis_eligibility(NormalizedBasisIntent, OperationLaneRequest)
  -> BasisEligibility | DeniedBasisCapability
admit_basis_capability(BasisEligibility) -> AdmittedBasisCapability | DeniedBasisCapability
scope_basis_for_operation(AdmittedBasisCapability, LaneWitness)
  -> OperationScopedBasis<Lane> | DeniedBasisCapability
readmit_lower_runtime_evidence(OperationScopedBasis<Lane>, LowerRuntimeEvidence)
  -> LowerRuntimeBoundBasis<Lane> | DeniedBasisCapability
consume_basis(LowerRuntimeBoundBasis<Lane>, OperationUse)
  -> BasisUseReceipt<Lane> | DeniedBasisCapability
envelope_basis_use(BasisUseReceipt<Lane>) -> SelfDescribingBasisEnvelope<Lane>
certify_basis_lifecycle(BasisLifecycleCertificationInput)
  -> BasisLifecycleCertificationBundle
```

Implementations may choose different names, but they must preserve the same
proof flow and type-strengthening direction. A later phase that accepts raw
ids, plain digests, generic maps, or optional fields in place of the previous
phase's proof type fails this milestone.

## Implementation Sequencing Contract

The phases are not a buffet, but phase ordering is an implementation and
review discipline rather than a separate runtime or compile-time gate. The
compiler should enforce the typestate lifecycle itself; the milestone closeout
should enforce that implementation proceeded in an understandable order.

Sequencing rules:

- Each phase should land with its proof types, compile-fail boundaries,
  hostile denial tests, counter snapshots, and migration notes for touched
  existing surfaces before the next phase depends on it.
- A phase may define placeholder enum variants for later phases only when they
  return typed deferred or unsupported posture and have zero operational
  residue.
- A phase may add test fixtures for later phases only when those fixtures are
  isolated from production APIs and do not become backdoor production support.
- Any compatibility debt discovered in a phase must be recorded with owner,
  public entrypoint, current behavior, target lifecycle phase, blocking reason,
  and denial or adapter posture. An unowned "compatibility debt" bucket is not
  allowed.
- A closeout note may explain sequencing deviations, but it cannot waive the
  typestate contract, adapter shape contract, lower-runtime boundary contract,
  or public proof-boundary requirements.

Phase artifact manifest:

- Phase 1 emits `BasisInventory`, `LowerRuntimeApiReuseMatrix`,
  `RawBasisIntent`, `NormalizedBasisIntent`, normalization denials, and
  normalization counters.
- Phase 2 consumes only `NormalizedBasisIntent` and emits `BasisEligibility`,
  operation-lane witnesses, eligibility denials, advisory proofs, and
  eligibility counters.
- Phase 3 consumes only `BasisEligibility` and emits sealed admitted/denied
  capabilities plus operation-scoped basis wrappers. Lower-runtime evidence
  slots remain unbound placeholders here.
- Phase 4 consumes only operation-scoped basis wrappers and facade-returned
  lower-runtime evidence, then emits lower-runtime-bound scoped basis proofs or
  lower-runtime mismatch denials.
- Phase 5 consumes only lower-runtime-bound scoped basis proofs and emits
  operation-specific use receipts, envelopes, support rows, and transition
  rules.
- Phase 6 consumes only the prior phase artifacts and emits certification
  bundles, proof-shape audits, boundary audits, performance certification, and
  migration closeout.

## Phases

Phases are ordered gates. Later phases may sketch fixture-only test helpers, but
production code may not consume a later-phase surface until the prior phase has
closed its proof, compile-fail, and counter obligations.

### Phase 1: Basis Inventory, Family Taxonomy, And Raw Intent Normalization

Create the lifecycle home for basis meaning without changing operational
behavior yet.

Must ship:

- a complete inventory of existing Query, runtime-bridge, signal, and
  relational basis surfaces that 9.3.2 must consolidate, wrap, reuse, defer, or
  explicitly leave as compatibility debt
- a public-facade API reuse matrix for bridge, signal, and relational surfaces
  classifying each basis-adjacent API as `reused authority`, `Query adapter`,
  `deferred neighbor`, or `forbidden duplicate`
- `RawBasisIntent` variants for current head, branch head, explicit branch
  snapshot, preview, preview-derived branch/historical, runtime snapshot,
  historical snapshot, tenant-scoped, policy-scoped, and unsupported future
  temporal/async/store/durable neighbors
- a target DX transcript inventory covering current-head observation,
  branch-head mutation preparation, preview denial, causal inspection,
  lower-runtime evidence materialization, and support discovery
- `NormalizedBasisIntent` with canonical digest, family, authority posture,
  requested operation lane, policy/tenant/schema posture, and explicit source
  path
- canonicalization rules proving equivalent public construction paths produce
  the same normalized digest while intentionally different basis meaning changes
  the digest
- typed denials for malformed, ambiguous, unsupported, and future-neighbor raw
  basis intents

Must preserve:

- existing admitted branch, preview, read-composition, subscription, and
  causal-inspection behavior
- relational and runtime-bridge authority ownership
- raw compatibility inputs as non-capability values

Exit evidence:

- normalized current-head, branch-head, snapshot, preview, tenant, and policy
  lanes produce stable canonical digests
- alternate builder paths normalize to the same digest for the same meaning
- incompatible or unsupported future temporal/async/store/durable intents fail
  typed before eligibility
- compile-fail coverage proves external callers cannot construct normalized
  proof rows directly
- counters report raw intent width, normalized family count, source path count,
  and rejection width

### Phase 2: Basis Eligibility And Operation Admission

Add the eligibility boundary that decides whether a normalized basis may be
used for a requested operation family.

Must ship:

- `BasisEligibility` as a sealed proof type consuming only
  `NormalizedBasisIntent`
- operation lane witness types for observation, mutation preparation, replay,
  inspection, materialization, subscription declaration, subscription
  activation, preview closeout, and certification
- facade admission methods whose common-path names make the operation lane
  obvious at the call site
- success, advisory, and violation decisions with structured traces
- success, advisory, and violation result shapes that cannot be substituted
  for one another by downstream phase signatures
- typed denial families for stale, inaccessible, policy-masked,
  tenant-mismatched, schema-incompatible, operation-ineligible,
  preview-drifted, historical-replay-unsupported, lower-runtime-binding-missing,
  and durable-overclaim requests
- exact counters for eligibility rows consulted, policy/tenant/schema checks,
  lower-runtime evidence checks, and denied residue

Must preserve:

- eligibility precedes read, mutation, replay, inspection, materialization, and
  subscription artifact construction
- diagnostic richness may enrich eligibility traces only on the cold path
- advisory eligibility cannot be silently promoted to fully admitted operation
  capability

Exit evidence:

- each supported basis family has at least one admitted operation lane and one
  hostile denial lane
- denied operation attempts produce typed denial artifacts and zero operational
  residue
- stale preview, tenant mismatch, policy mask, schema mismatch, missing
  lower-runtime binding, and durable overclaim all stop at eligibility
- compile-fail coverage proves operation lanes cannot be WORTHd outside the
  proving module

### Phase 3: Admitted Basis Capabilities And Scoped Use Types

Create the sealed capability types that downstream Query surfaces must consume.

Must ship:

- `AdmittedBasisCapability` carrying family, authority source, scope,
  visibility, lifecycle posture, policy/tenant/schema posture,
  lower-runtime evidence references, permitted lanes, and canonical digest
- typed capability wrappers for observation, mutation, replay, inspection,
  materialization, subscription declaration/activation, preview closeout, and
  certification scope
- operation-specific scoped basis wrappers as the only input shapes accepted by
  runtime-backed read, live, mutation-preparation, replay, inspection, and
  materialization entrypoints. A broad `ScopedExecutionOrObservationBasis`
  facade may aggregate them only if each operation still receives its lane
  witness at the type level.
- common-path read, mutation-preparation, inspection, materialization,
  subscription, and certification entrypoints that accept the scoped basis
  wrappers without requiring callers to manually assemble proof internals
- unbound lower-runtime evidence placeholders that cannot be used to emit
  receipts until Phase 4 readmission succeeds
- read-only public accessors that expose evidence without exposing
  constructors or mutable lower-runtime handles
- compatibility adapters that immediately lower existing branch/preview/read
  basis admissions into admitted capability wrappers

Must preserve:

- branch and preview mutations remain basis-explicit
- Query-owned inspection and causal materialization consume capability proof,
  not raw basis digests
- runtime bridge authority bases remain bridge-owned and are referenced only as
  sealed lower-runtime evidence

Exit evidence:

- read, mutation, replay, inspection, materialization, and subscription tests
  can consume scoped capability types for admitted lanes
- advisory capabilities cannot enter mutation, effect, writeback, or closeout
  lanes unless a later proving function explicitly converts them into a
  success capability
- direct raw branch/snapshot/preview inputs are either unavailable or lower
  immediately into `RawBasisIntent`
- compile-fail fixtures prove external code cannot instantiate admitted
  capabilities or scoped use types
- counter snapshots prove scoped capability construction cost depends on
  admitted basis evidence width, not unrelated runtime graph size

### Phase 4: Lower-Runtime Binding And Trust-Boundary Readmission

Make lower-runtime contact explicit without moving lower-runtime authority into
Query.

Must ship:

- bridge-facing adapters that consume existing bridge subscription,
  truth-view, continuity, preview, writeback, and causal-envelope authority
  artifacts through `worth_runtime_bridge::facade`
- explicit bridge reuse rows for `RuntimeBridge::evaluate`,
  `RuntimeBridge::evaluate_current`, `RuntimeBridge::plan_truth_view_packet`,
  `RuntimeBridge::plan_source_packet_set`,
  `RuntimeBridge::materialize_source_packet`,
  `RuntimeBridge::admit_subscription`,
  `RuntimeBridge::admit_subscription_preview_basis`, and
  `RuntimeBridge::deliver_continuity` wherever Query claims basis meaning
- relational-facing authority references for branch, commit, snapshot, head,
  lineage, and truth visibility evidence through `worth_relational::facade`
  and `RuntimeBridgeRelationalSource` where the flow is bridge-bound
- signal-facing evidence references for live observation, invalidation,
  snapshot, replay, lineage, and forensic diagnostic basis through
  `worth_signal::facade` where existing Query surfaces already carry them
- a common-path lower-runtime evidence inclusion surface that attaches existing
  authority evidence to Query inspection/envelope output without caller-side
  bridge/signal/relational stitching
- trust-boundary readmission checks proving lower-runtime returned evidence
  agrees with the admitted Query basis capability
- typed denials for mismatched relational snapshot, mismatched bridge authority
  basis, missing signal observation basis, stale runtime snapshot, and lower
  runtime capability unsupported

Must preserve:

- Query does not mint bridge continuity, writeback, route, replay, preview, or
  causal-envelope authority bases
- Query does not duplicate bridge subscription basis binding, bridge truth-view
  authority, bridge continuity authority, bridge preview basis, bridge
  writeback bases, or bridge causal envelope evidence
- relational and signal evidence remains authority-owned and facade-mediated
- bridge-bound relational truth flows use the existing relational bridge
  adapter rather than fresh Query-side commit/snapshot loaders
- lower-runtime basis evidence is readmitted into Query capability receipts,
  not expanded into Query-owned truth facts

Exit evidence:

- admitted basis capabilities bind to lower-runtime evidence by digest and
  authority name
- mismatched bridge, relational, or signal basis evidence fails before a use
  receipt exists
- bridge, relational, and signal authority basis fields remain facade-mediated
  or sealed; Query does not export their constructors
- API reuse matrix rows prove bridge subscription/truth-view/continuity/
  preview/writeback/causal evidence, relational truth/history/snapshot
  evidence, and signal snapshot/replay/lineage/diagnostic evidence are reused
  or explicitly deferred, not rebuilt
- exact counters report lower-runtime binding attempts, readmission checks,
  mismatch denials, and retained evidence lookup width

### Phase 5: Basis Use Receipts And Self-Describing Basis Envelopes

Materialize the public explanation boundary for basis consumption.

Must ship:

- `BasisUseReceipt` variants for observation, mutation preparation, replay,
  inspection, materialization, subscription declaration, subscription
  activation, preview closeout, and certification
- `SelfDescribingBasisEnvelope` with primary basis identity, authority
  references, structured warnings, decision trace, lifecycle/transition
  posture, lower-runtime bindings, integrity markers, and performance
  accounting
- operation-specific next-transition rules, including which receipts can
  produce a later inspection, materialization, effect plan, projection
  consumption, temporal extension, async/resource extension, store-backed
  replay, or durable reload request
- public support metadata describing admitted, advisory, denied, deferred, and
  unsupported basis families
- support metadata derived from the same eligibility/admission registry used by
  executable behavior, not a hand-maintained parallel table
- public support discovery methods that let callers inspect admitted,
  advisory, denied, deferred, and unsupported basis posture before execution
- cold-path rendering/materialization policies that do not change basis
  capability meaning

Must preserve:

- receipts are derived from admitted capabilities and lower-runtime bindings,
  not from raw public identifiers
- envelopes are self-describing and offline-readable without re-running Query
- deferred temporal/async/store/durable claims remain explicit support posture,
  not hidden TODOs

Exit evidence:

- every admitted operation produces a receipt whose digest binds the capability,
  operation lane, lower-runtime evidence, and counters
- envelopes distinguish current-head, branch, preview, snapshot, historical,
  tenant, policy, and denied/future-neighbor basis posture mechanically
- support metadata agrees with executable admission behavior
- changing an admission row without updating support metadata, receipts, and
  certification creates a compile failure or named certification failure
- compile-fail coverage proves external callers cannot mint receipts,
  envelopes, or support rows

### Phase 6: Basis Lifecycle Certification And Public Boundary Closure

Close 9.3.2 with hostile certification across canonicalization, eligibility,
operation use, lower-runtime binding, future-neighbor denial, and performance.

Must ship:

- `BasisLifecycleCertificationBundle` with representative rows for admitted,
  advisory, denied, lower-runtime mismatch, future-neighbor denial, and
  performance lanes
- proof-shape certification that rejects phase skipping, raw identifier
  substitution, stale proof reuse, operation-lane WORTHry, and WORTHd
  lower-runtime authority witnesses
- golden DX certification compiling the target common-path transcripts for
  observation, mutation preparation, inspection/evidence materialization,
  support discovery, and typed denial handling
- public boundary audit proving raw branch/snapshot/preview/tenant/policy
  identifiers cannot reach ordinary read, mutation, replay, inspection, or
  materialization surfaces as capability tokens
- performance certification for normalization, eligibility, lower-runtime
  binding, scoped-use construction, receipt emission, envelope materialization,
  and support lookup
- migration audit showing existing branch, preview, read-composition,
  subscription, and causal inspection basis consumers have been moved to
  capability consumption or explicitly marked as compatibility debt

Must preserve:

- 9.3.1 causal inspection remains basis-explicit and does not regress into raw
  basis digests
- 9.3.3 effect execution can consume basis capabilities without re-deciding
  basis authority
- Runtime API Public Stabilization Gate can expose basis vocabulary without
  later temporal/async milestones needing a parallel model

Exit evidence:

- the named 9.3.2 certification suite passes with canonical machine-checkable
  artifacts
- all touched Rust production and test files remain within the 400-line rule or
  have explicit spec-listed exemptions
- full relevant `worth-query` and `worth-runtime-bridge` focused suites pass
- compile-fail boundaries prove proof-bearing lifecycle artifacts are not
  externally constructible

## Must Ship

- phase-typed basis lifecycle artifacts:
  - `RawBasisIntent`
  - `NormalizedBasisIntent`
  - `BasisEligibility`
  - `AdmittedBasisCapability`
  - `DeniedBasisCapability`
  - `ScopedExecutionOrObservationBasis`
  - `BasisUseReceipt`
  - `SelfDescribingBasisEnvelope`
  - `BasisLifecycleCertificationBundle`
- admitted basis capability families for runtime-backed current head, branch
  head, branch snapshot, preview, preview-derived, runtime snapshot,
  historical snapshot where already admitted, tenant-scoped, and policy-scoped
  usage
- typed unsupported/deferred neighbors for temporal, async/resource,
  store-backed parity, durable reload, and restart-stable basis envelopes
- operation-specific capability lanes for observation, mutation preparation,
  replay, inspection, materialization, subscription declaration, subscription
  activation, preview closeout, and certification
- polished common-path basis APIs matching the target DX transcripts closely
  enough that domain callers can express intent without manually building proof
  internals
- lower-runtime binding/readmission against existing relational,
  runtime-bridge, and signal evidence where the current runtime-backed surface
  already claims basis meaning
- self-describing basis envelopes carrying primary result, structured warnings,
  decision trace, lifecycle posture, lower-runtime authority bindings,
  integrity markers, and performance accounting
- support metadata that synchronizes advertised basis families with executable
  eligibility/admission behavior
- compile-fail proof boundaries for normalized intents, admitted capabilities,
  scoped use types, use receipts, envelopes, certification bundles, and
  lower-runtime authority witness fields
- exact performance counters and slope digests for normalization, eligibility,
  lower-runtime binding, scoped-use construction, receipt emission, envelope
  materialization, support lookup, and certification bundle assembly
- golden DX transcripts that compile and remain synchronized with executable
  admission, support metadata, receipts, and envelopes

## Must Preserve

- relational remains authority for branch, commit, snapshot, head, lineage, and
  truth visibility
- runtime bridge remains authority for bridge continuity, writeback, route,
  evaluation, replay, preview, structural, causal-envelope, and lower-runtime
  authority basis records
- signal remains authority for observation, invalidation, scheduling, and live
  evaluation basis evidence
- Query owns public basis intent normalization, admission, capability shaping,
  use receipts, envelopes, support metadata, and certification
- raw lower-runtime basis identifiers never become public capability tokens
- branch, preview, tenant, policy, subscription, and causal inspection basis
  distinctions remain mechanically visible
- temporal and async/resource milestones extend this lifecycle rather than
  adding parallel basis APIs
- store-backed and durable claims remain explicit deferred debt
- diagnostic richness remains a cold-path envelope/materialization concern
  rather than part of hot-path query execution or signal invalidation

## Acceptance Evidence

This milestone is complete only when `worth-query` can prove:

- the `Query Basis Capability Lifecycle Test` passes with canonical
  machine-checkable artifacts
- equivalent current-head, branch-head, snapshot, preview, tenant, and policy
  basis intents normalize to the same envelope across alternate construction
  paths
- intentionally different basis meaning changes the declared digest fields
- stale, inaccessible, incompatible, operation-ineligible, policy-masked,
  tenant-mismatched, preview-drifted, historical-replay-unsupported,
  missing-lower-runtime-binding, durable-overclaim, temporal-deferred, and
  async/resource-deferred requests fail typed and early
- read, mutation-preparation, replay, inspection, materialization, and
  subscription surfaces consume scoped basis capability types rather than raw
  basis identifiers
- common-path APIs let domain callers express basis intent, operation lane,
  execution, support discovery, and typed denial handling without manual proof
  construction or lower-runtime stitching
- basis use receipts bind operation lane, admitted capability,
  lower-runtime evidence, lifecycle posture, permitted next transitions, and
  exact counters
- bridge, relational, and signal evidence references are readmitted through
  authority-preserving boundaries instead of being re-minted as Query facts
- bridge subscription basis, bridge truth-view basis, bridge continuity basis,
  bridge preview basis, bridge writeback basis, relational branch/head/snapshot
  authority, relational bridge adapters, and signal snapshot/replay/lineage
  evidence are consumed from their existing facades or explicitly marked
  deferred
- support metadata and executable admission agree for admitted, advisory,
  denied, deferred, and unsupported basis families
- compile-fail boundaries prevent external construction of proof-bearing basis
  lifecycle artifacts
- performance certification proves basis cost slopes are bounded by basis
  evidence width, operation-lane width, and lower-runtime binding width rather
  than unrelated runtime graph size, subscription retention width, or bridge
  diagnostics retention width

## Required Verification Output

The 9.3.2 certification bundle must emit:

- `query_digest`
- `raw_basis_intent_digest`
- `normalized_basis_intent_digest`
- `basis_family_digest`
- `basis_authority_digest`
- `basis_scope_digest`
- `basis_visibility_digest`
- `basis_lifecycle_digest`
- `basis_policy_digest`
- `basis_tenant_schema_digest`
- `basis_operation_lane_digest`
- `basis_eligibility_digest`
- `admitted_basis_capability_digest`
- `denied_basis_capability_digest`
- `scoped_basis_digest`
- `basis_use_receipt_digest`
- `basis_envelope_digest`
- `relational_basis_authority_digest`
- `bridge_basis_authority_digest`
- `signal_basis_authority_digest`
- `lower_runtime_basis_binding_digest`
- `basis_readmission_proof_digest`
- `basis_target_dx_digest`
- `basis_golden_transcript_digest`
- `typestate_transition_digest`
- `lane_witness_digest`
- `phase_artifact_manifest_digest`
- `compatibility_debt_registry_digest`
- `basis_transition_digest`
- `basis_support_matrix_digest`
- `basis_future_neighbor_denial_digest`
- `basis_proof_shape_digest`
- `basis_phase_progression_digest`
- `failure_digest`
- `counter_snapshot`
- `basis_normalization_slope_digest`
- `basis_eligibility_slope_digest`
- `basis_lower_runtime_binding_slope_digest`
- `basis_scoped_use_slope_digest`
- `basis_receipt_slope_digest`
- `basis_envelope_materialization_slope_digest`
- `basis_support_lookup_slope_digest`
- `compile_fail_boundary_digest`

## Architectural Notes

- Basis is not truth. It is a Query capability that proves a caller may use a
  named truth posture for a named operation.
- Basis capability identity must include the authority source, family, scope,
  visibility, lifecycle, policy, tenant/schema, operation lane, and
  lower-runtime binding posture. A digest that omits any of these cannot be the
  certification digest.
- Operation lanes are not booleans. A basis admitted for observation is not
  automatically admitted for mutation, replay, inspection, materialization, or
  preview closeout.
- A universal basis enum with optional lane flags is not a typestate proof. It
  is a runtime permission table. Operation-specific wrappers or witness types
  are required wherever the operation boundary depends on the proof.
- Digests are not proof substitutes. They bind explanations and certification,
  but executable APIs must consume proof-bearing types.
- Advisory basis eligibility is first-class. It may permit narrowed
  observation or inspection while denying mutation, replay, or materialization.
- Denied basis capability artifacts must be different proof types from
  admitted capabilities.
- Lower-runtime basis evidence must keep its authority name. Query envelopes
  may summarize and bind it, but not flatten it into Query-owned truth.
- Query lower-runtime adapters must import through public facades. Any need for
  private bridge, signal, or relational module access is a spec violation until
  the owning crate exposes an intentional API.
- Do not create Query twins of bridge `ValidatedSubscriptionBasisBinding`,
  bridge truth-view authority bases, bridge continuity bases, bridge writeback
  bases, relational `BranchHead`/`SnapshotHandle`/`CanonicalCommitEnvelope`,
  or signal `SignalSnapshotV1`/`SignalCheckpointImage`/`LineageRecord`/
  `ReplayCursor`. Query evidence structs may reference those authorities by
  digest, identity, receipt, or facade-returned summary only.
- Support metadata must be executable. A family cannot be advertised as
  admitted unless executable admission and certification agree.
- Existing branch/preview/read-composition basis APIs should become adapters
  into the lifecycle rather than permanent sibling lifecycles.
- Support matrices must be generated or derived from executable admission
  facts. A manually synchronized matrix is allowed only as a rendered artifact,
  never as the source of truth.
- Golden DX transcripts are executable design constraints. If the proof model
  is correct but the common path requires callers to assemble internal proof
  rows, the implementation is unfinished.

## Store Dependency

Runtime-backed basis capability lifecycles are not blocked on `worth-store`.

The following remain deferred:

- durable basis reload
- restart-stable basis envelopes
- store-restored snapshot-plus-tail reconstruction
- store-backed historical replay parity
- portable basis capability import/export
- persisted basis use receipt archives

Those are Milestone 10, Milestone 11, or later certification scope. Any 9.3.2
surface that encounters those families must report typed deferred or
unsupported posture.

## Sequencing Notes

Milestone 9.3.2 belongs after 9.3.1 because causal inspection already proved
that query observations need explicit basis evidence, lower-runtime authority
bindings, and public inspection artifacts. Basis lifecycle turns that evidence
from repeated local fields into a reusable proof chain.

It belongs before 9.3.3 because authority-scoped effect execution cannot
honestly lower effects if it receives raw branch/snapshot/preview identifiers
and then re-decides basis authority inside the executor.

It belongs before the Runtime API Public Stabilization Gate because the public
runtime API must expose one basis model that temporal, async/resource,
store-backed, and durable milestones can extend without breaking daily-driver
domain code.

## Closeout Standard

This milestone may close only when:

- the 9.3.2 spec phases have been implemented in order. Any production
  deviation must have been approved by a spec amendment before the deviating
  implementation landed.
- every ordinary Query surface that consumes basis meaning uses scoped
  capability proof or is explicitly listed as compatibility debt
- every admitted basis family has canonicalization, eligibility, use receipt,
  envelope, support metadata, and certification coverage
- every denied/deferred neighbor has typed denial and zero operational residue
  coverage
- compile-fail boundaries prove public callers cannot construct admitted
  capability, scoped basis, receipt, envelope, or certification artifacts
- performance counters and slope digests are enforced for every claimed bounded
  basis operation
- roadmap and test-requirement references point at this spec and named suite
  accurately

## Self-Check

- Does the milestone solve a real structural problem or just package work
  cosmetically? Yes: it prevents raw basis identifiers from acting as hidden
  authority tokens across reads, writes, replay, inspection, materialization,
  subscriptions, and future temporal/async surfaces.
- Is the adversarial constraint precise and load-bearing? Yes: stale,
  inaccessible, incompatible, operation-ineligible, policy/tenant mismatched,
  lower-runtime mismatched, and deferred future basis requests must fail before
  operational artifacts exist.
- Does the milestone preserve crate authority boundaries? Yes: Query owns the
  public capability lifecycle, while relational, runtime bridge, signal, and
  store retain their authority surfaces.
- Does the milestone define proof obligations, not just implementation tasks?
  Yes: canonicalization, phase progression, operation-lane permission,
  lower-runtime readmission, support synchronization, compile-fail boundaries,
  and exact slope counters are required.
- Could a competent engineer map this spec into honest types, modules, and
  tests? Yes: the phase progression names the required proof types, denials,
  receipts, envelopes, certification bundles, and test lanes.
- Does the milestone belong in this roadmap sequence, or is it out of order?
  Yes: it follows causal inspection and precedes effect execution and public
  API stabilization because those surfaces need basis proof before they can
  freeze or execute honestly.
