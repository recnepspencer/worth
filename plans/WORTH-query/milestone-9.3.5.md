# Milestone 9.3.5 Engineering Spec: Intent Admission Decision Lattice And Decision Trace

> **Status:** Closed on 2026-05-18 via
> [milestone-9.3.5-closeout.md](./milestone-9.3.5-closeout.md)
>
> **Roadmap parent:** [worth_query_roadmap.md](./worth_query_roadmap.md)
>
> **Vision parent:** [worth_query_vision.md](./worth_query_vision.md)
>
> **Prior milestone:** [milestone-9.3.4.md](./milestone-9.3.4.md)
>
> **Next milestone:** [Milestone 9.3.6](./worth_query_roadmap.md#milestone-936-lower-runtime-capability-routing-and-boundary-envelopes)
> will consolidate the remaining lower-runtime boundary envelopes, direct
> contact cleanup, and capability-routing debt after 9.3.5 establishes the
> canonical admitted handoff into real route execution.
>
> **Test requirements:** [test-requirements.md](./test-requirements.md)
>
> **Primary architectural driver:** make every Query-crossing intent resolve
> through one proof-bearing admission lattice with typed success, advisory, and
> violation outcomes before expensive construction, lowering, execution, or
> diagnostic materialization occurs.
>
> **Companion docs:**
> - [MENTALITY.md](../../docs/coding-guidelines/MENTALITY.md)
> - [arch_laws.md](../../docs/coding-guidelines/arch_laws.md)
> - [composition_laws.md](../../docs/coding-guidelines/composition_laws.md)
> - [domain_structure_laws.md](../../docs/coding-guidelines/domain_structure_laws.md)
> - [perf_laws.md](../../docs/coding-guidelines/perf_laws.md)
> - [worth_query_vision.md](./worth_query_vision.md)
> - [worth_query_roadmap.md](./worth_query_roadmap.md)
> - [test-requirements.md](./test-requirements.md)
> - [test-requirements-milestone-9_3-and-runtime-gates.md](./test-requirements-milestone-9_3-and-runtime-gates.md)
> - [milestone-9.3.1.md](./milestone-9.3.1.md)
> - [milestone-9.3.2.md](./milestone-9.3.2.md)
> - [milestone-9.3.2-closeout.md](./milestone-9.3.2-closeout.md)
> - [milestone-9.3.3.md](./milestone-9.3.3.md)
> - [milestone-9.3.3-closeout.md](./milestone-9.3.3-closeout.md)
> - [milestone-9.3.4.md](./milestone-9.3.4.md)

## Goal

Make Query admission one explicit public lifecycle:

```text
RawIntent
  -> IntentEligibility
  -> AdmissionDecision
  -> AdmittedIntentPlan | AdvisoryDecision | ViolationDecision
  -> AdmittedExecutionHandoff | AdvisoryStop | ViolationStop
  -> DecisionTraceEnvelope
  -> IntentAdmissionCertificationBundle
```

This milestone does not replace the domain-specific meaning already established
by basis lifecycle, effect execution, projection consumption, or inspection.
It unifies how those families report admission, warnings, failure classes,
downstream-proof shape, execution handoff shape, and decision trace evidence
so later lowering and routing milestones consume one honest admission model
instead of a pile of near-equivalent booleans, `Result`s, and ad hoc denial
payloads.

## Why This Milestone Exists

Milestones 9.3.1 through 9.3.4 already established real Query-owned
proof-bearing lifecycles:

- 9.3.1 made inspection requests and redaction posture explicit
- 9.3.2 made basis use a typed capability lifecycle
- 9.3.3 made effect execution a lowered authority-scoped pipeline
- 9.3.4 made projection consumption a declared, receipt-backed fact lifecycle

What remains open is the admission story across those lifecycles.

Today Query can still drift into a dangerous half-state where:

- one surface returns a typed denial enum, another returns a generic error, and
  a third uses advisory metadata with no shared decision model
- an operational path can explain success but not advisory adaptation or
  violation with the same structural richness
- expensive construction happens before the caller receives the most important
  negative or warning-bearing decision facts
- downstream code must pattern-match family-specific artifacts to answer the
  same admission question repeatedly
- later routing or public API work inherits several almost-compatible
  admission/failure vocabularies instead of one canonical public lattice

Milestone 9.3.5 exists to close that seam before 9.3.6, the runtime API public
stabilization gate, and the temporal/async milestones build on top of it.

## Governing Summaries

- `MENTALITY.md`: the main thing to protect is adversarial correctness before
  convenience. This spec must solve "how does Query fail or adapt honestly
  before work starts?" rather than merely prettifying errors after the fact.
- `arch_laws.md`: the strongest shaping constraints are that rejection must
  precede construction, proof-bearing types must encode phase progression, and
  authority for decision evidence must stay with the owning subsystems even when
  Query owns the public admission envelope.
- `composition_laws.md`: the spec must keep family classification, eligibility,
  decision shaping, trace assembly, facade DX, and certification as separate
  responsibilities rather than one catch-all admission helper bag.
- `domain_structure_laws.md`: the tree must make admission family, shared
  lattice mechanics, trace vocabulary, per-surface adapters, and certification
  physically locatable. Shared lifecycle is real here, but mixed failure/cost
  semantics still need named boundaries.
- `perf_laws.md`: admission cannot hide broad rediscovery, broad support scans,
  or trace assembly rescans behind "cold path" excuses. Exact counters and
  slopes must prove eligibility and trace construction scale with declared
  decision width, not unrelated runtime size.
- `worth_query_vision.md`: Query promises typed, composable, policy-aware,
  basis-aware surfaces. A canonical admission lattice is part of keeping those
  surfaces structurally query-shaped instead of host-repaired.
- `worth_query_roadmap.md`: 9.3.5 belongs after basis/effect/projection
  lifecycles exist and before lower-runtime boundary cleanup so real route
  execution can consume admitted intent plans rather than raw intents while
  9.3.6 finishes consolidating the remaining capability-routing surface.
- `test-requirements.md`: certification must prove admitted, advisory, and
  violation cases are equally inspectable, machine-checkable, and hostile-case
  covered.
- `test-requirements-milestone-9_3-and-runtime-gates.md`: 9.3.5 needs its own
  named suite with canonical digests, representative multi-family lanes,
  compile-fail boundaries, and exact slope evidence.
- `milestone-9.3.1.md`: inspection already distinguishes admitted/advisory/
  denied detail; 9.3.5 must lift that decision richness into the shared public
  admission lattice instead of flattening it.
- `milestone-9.3.2.md`: basis capability already proves rejection before use;
  9.3.5 must consume those proofs as one intent family rather than reminting
  basis-specific admission semantics.
- `milestone-9.3.3.md`: effect execution already separates normalization,
  eligibility, lowering, and execution; 9.3.5 must preserve that separation
  while standardizing the public admission decision vocabulary.
- `milestone-9.3.4.md`: projection consumption already distinguishes admitted,
  warning-bearing, denied, deferred, and source-mismatch outcomes. 9.3.5 must
  retain those distinctions structurally inside the shared lattice.

## Adversarial Constraint

Under policy masking, tenant/schema variation, basis drift, preview-local
posture, source mismatch, lower-runtime support gaps, warning-bearing
projection consumption, inspection redaction, effect rebind requirements, and
future lower-runtime routing requests, the same canonical Query-crossing intent
must always resolve through one typed admission lattice before construction,
lowering, execution, or diagnostic materialization.

Admitted, advisory, and violation outcomes must all carry enough structured
decision evidence that downstream code can proceed, adapt, or fail closed
without reconstructing the decision from raw input, ambient host context, or
lower-runtime internals.

If any surface:

- collapses advisory versus violation meaning into a binary `Result`
- hides the boundary where expensive work became admissible
- forces downstream code to rediscover policy, capability, basis, projection,
  invariant, or routing decisions from raw family-specific artifacts
- remints lower-runtime decision evidence as Query-owned authority
- permits advisory- or violation-only artifacts to masquerade as admitted plans
- allows trace richness to change operational meaning

then Milestone 9.3.5 has failed.

## Product Decision Lock

- `worth-query` owns the shared public admission lattice, family classification
  surface, decision-trace vocabulary, trace envelopes, admitted execution
  handoff surface, family adoption helpers, support metadata, DX helpers, and
  certification.
- `worth-query` does not absorb basis authority, projection visibility
  authority, effect execution authority, inspection-causality authority, or
  lower-runtime routing authority. Those remain owned by their existing
  subsystems and lifecycles.
- Domain-specific 9.3.x lifecycles remain authoritative for their internal
  proofs and downstream plans:
  - basis lifecycle owns basis normalization, admission, scoped use, and use
    receipts
  - effect lifecycle owns effect normalization, lowering, execution, and
    execution receipts
  - projection consumption owns declaration, contract binding, fact extraction,
    and projection-consumption receipts
  - inspection owns causal request admission, redaction posture, and inspection
    artifacts
- 9.3.5 unifies public admission vocabulary; it does not replace those
  lifecycles with one generic bag.
- Binary convenience APIs may exist only as derived summaries over the shared
  lattice. They are not allowed to become the canonical public admission
  artifact.
- Covered entrypoints must delegate, not emulate:
  - if a public Query API is listed as 9.3.5-covered, that API must become a
    thin authoring or display wrapper over the canonical lattice path
  - it may not keep a second admission or execution branch that merely returns
    equivalent-looking artifacts
- 9.3.5 must adopt at least these Query-crossing intent families:
  - read execution admission where applicable on existing public Query read
    entrypoints
  - basis use
  - projection consumption
  - effect execution
  - inspection and diagnostic materialization
  - lower-runtime capability-routing requests that already have real
    bridge-backed execution semantics available through current Query runtime
    seams
- The shared lattice must preserve family-specific semantics rather than
  flattening them:
  - advisory redaction is not the same as rebind-required
  - deferred support is not the same as violation
  - source mismatch is not the same as policy denial
  - unsupported future-neighbor requests are not the same as stale local input

## Concrete Coverage Inventory

9.3.5 is not allowed to hide behind a fuzzy idea of "covered surfaces." The
implementation spec and certification fixtures must name a closed inventory of
the concrete public entrypoints and runtime seams the milestone adopts.

Minimum required inventory shape for each covered item:

- public entrypoint or facade method name
- intent family classification
- canonical raw-intent authoring constructor
- eligibility authority source
- admitted downstream plan type
- admitted execution handoff type, or explicit `no-execution-handoff` reason
- execution seam owner and method family
- advisory and violation classes expected for that item
- execution receipt or result artifact that must carry the admission provenance

This inventory must be checked into the implementation tree as executable
fixture or compile-visible metadata, not merely prose in the milestone closeout.

Initial inventory floor already implied by the current Query runtime surface:

- `WORTHQueryRuntime::execute_intent`
- `WORTHQueryRuntime::execute_next_effect_write_intent`
- every currently supported public Query read path that can cross from admitted
  Query intent into bridge-backed evaluation or receipt-routed execution
- every currently supported public inspection or diagnostic-materialization path
  that claims 9.3.5 coverage

No implementation may claim family-wide closure for reads, inspection, or
lower-runtime capability-routing until the concrete covered methods are named
in the inventory and certified individually.

## Existing Surfaces To Consolidate

Milestone 9.3.5 must consolidate, not bypass, the admission-bearing surfaces
already present in Query:

- basis lifecycle proofs from 9.3.2
- effect lifecycle eligibility and non-admitted postures from 9.3.3
- projection-consumption eligibility and support posture from 9.3.4
- inspection admission, advisory redaction, and denied postures from 9.3.1
- support/discovery matrices already emitted by 9.3.2 through 9.3.4
- common-path facade helpers that currently expose family-specific operational
  admission stories

Normative consequences:

- if an intent family already has a trusted eligibility proof, 9.3.5 must
  wrap or adopt it rather than duplicating the decision logic
- if a family cannot honestly produce one of admitted, advisory, or violation,
  the spec must explain why; silence is not allowed
- if a lower-runtime route family does not yet have a real execution seam,
  9.3.5 must return typed deferred or violation posture rather than speculative
  pseudo-admission
- if a covered public API can still reach execution without first producing the
  canonical admitted handoff and execution provenance chain, the family is not
  actually covered

## Required Crate Changes

### `worth-query`: required changes

`worth-query` is the owning crate for Milestone 9.3.5. It must add:

- a new `intent_admission` subdomain under `crates/worth-query/src`
- a shared admission lattice with family-aware raw intent wrappers,
  eligibility records, admitted plans, advisory decisions, violation
  decisions, admitted execution handoffs, and decision-trace envelopes
- family adapters that consume existing basis, effect, projection, inspection,
  and read-admission artifacts without reminting their authorities
- bridge-backed handoff adapters for covered intent families whose admitted
  plans must cross into real route/evaluate/receipt-routing execution in 9.3.5
- one compile-visible coverage inventory for all adopted entrypoints and seams
- support/discovery metadata for which intent families are admitted,
  advisory-capable, deferred, or unsupported
- public facade exports and golden transcripts proving ordinary callers can use
  the shared admission surface without touching internal family adapters
- compile-fail boundaries proving external callers cannot construct admitted
  plans, advisory decisions, violation decisions, trace rows, or certification
  bundles directly
- the named 9.3.5 certification suite and all required verification outputs

`worth-query` must also tighten existing public surfaces so the shared lattice
becomes canonical:

- public APIs that currently expose family-specific binary admission summaries
  must either become derived helpers over the lattice or stay internal
- no new public API may return a raw `Result<T, E>` as the only admission
  contract for a Query-crossing intent covered by 9.3.5
- lower phases must consume `AdmittedIntentPlan` or a family-specific admitted
  wrapper derived from it, not raw intent plus repeated eligibility checks
- where a covered family already has a real bridge/runtime execution path, the
  canonical public path must consume a typed admitted execution handoff rather
  than re-deciding admission at execution time
- legacy public entrypoints that remain supported during 9.3.5 must become
  wrappers over the canonical lattice and handoff path rather than parallel
  implementations

### `worth-runtime-bridge`: required changes

`worth-runtime-bridge` remains authoritative for bridge route/evaluation,
preview, writeback, and cross-runtime evidence.

Required bridge boundary for this milestone:

- Query may reference bridge-owned evidence digests and family labels inside
  decision traces when a family decision depends on bridge support, preview
  posture, writeback posture, or inspection evidence availability
- Query may lower covered admitted intent plans into bridge-owned
  `route(...)`, `evaluate_current(...)`, `evaluate(...)`, or equivalent
  already-authoritative execution surfaces where those seams already exist
- Query must not move bridge routing or preview authority into the admission
  lattice; it may only own the typed handoff into bridge execution
- if bridge execution returns a route, evaluation, or receipt artifact for a
  9.3.5-covered path, that artifact must retain or link the originating
  admission and handoff digests strongly enough for later inspection and
  certification to prove no side-door execution occurred

Required bridge code changes:

- none unless implementation proves one already-authoritative support/evidence
  digest or one already-existing execution seam needed by the shared handoff is
  not currently reachable through the bridge facade

If such a gap appears, the only allowed bridge change is:

- one narrow facade export for an already-authoritative evidence digest,
  support posture, or existing execution seam needed by the Query handoff and
  trace

### `worth-relational`: required changes

`worth-relational` remains authoritative for truth, snapshot meaning, branch
meaning, policy-visible materialization truth, and relational decision
evidence.

Required relational boundary for this milestone:

- Query may bind relational authority/evidence digests into decision traces
  where a family decision depends on visibility, basis, or execution legality
- Query must not recreate relational authority decisions locally just to make
  the trace look uniform

Required relational code changes:

- none unless an implementation pass proves one already-authoritative decision
  digest needed by the lattice is not facade-reachable

### `worth-signal`: required changes

`worth-signal` remains authoritative for invalidation/evaluation strategy,
lineage, replay posture, and signal-backed observation support.

Required signal boundary for this milestone:

- Query may cite signal support or evaluation evidence when a decision depends
  on inspection richness, subscription support, or future-neighbor support
  posture
- Query must not absorb signal scheduling or evaluation semantics into generic
  admission logic

Required signal code changes:

- none for Milestone 9.3.5

### `worth-store`: required changes

- none in 9.3.5
- durable decision archives, persisted trace materialization, restart-stable
  trace reload, and store-backed admission replay remain later-milestone work

## Typed Phase Progression Lock

Milestone 9.3.5 must introduce this public progression:

```text
RawIntent
  -> IntentEligibility
  -> AdmissionDecision
  -> AdmittedIntentPlan | AdvisoryDecision | ViolationDecision
  -> AdmittedExecutionHandoff | AdvisoryStop | ViolationStop
  -> DecisionTraceEnvelope
  -> IntentAdmissionCertificationBundle
```

Minimum semantic meaning of each phase:

- `RawIntent`
  - one Query-crossing request before the shared lattice has proven whether it
    may proceed
  - family-typed: read, basis use, projection consumption, effect execution,
    inspection, diagnostic materialization, or lower-runtime capability-routing
    request
- `IntentEligibility`
  - the family-specific eligibility facts required to decide admission
  - may include support posture, policy posture, basis posture, invariant
    posture, routing-support posture, and source/projection posture where
    applicable
- `AdmissionDecision`
  - one family-aware classification into admitted, advisory, or violation
  - must bind the canonical intent digest plus the decision-trace index that
    explains why this classification happened
- `AdmittedIntentPlan`
  - the only type that may cross into expensive lowering, execution, or
    materialization phases
  - binds the exact admitted downstream family plan rather than a boolean
    "okay to continue"
- `AdmittedExecutionHandoff`
  - the typed handoff that carries one admitted family plan into an already
    authoritative runtime or bridge execution surface
  - may exist only for families where 9.3.5 actually binds to a real
    execution seam
  - must carry the execution-seam identity and the originating admission
    decision digest so execution cannot silently detach from its proof chain
- `AdvisoryDecision`
  - a typed non-binary decision where work may adapt or stop based on explicit
    warning, redaction, deferred-neighbor, or remediation posture
- `ViolationDecision`
  - a typed fail-closed decision where downstream construction and lowering are
  forbidden
- `AdvisoryStop`
  - the typed non-executing terminal handoff when advisory posture does not
    proceed into execution
- `ViolationStop`
  - the typed non-executing terminal handoff when violation posture forbids any
    downstream lowering or execution
- `DecisionTraceEnvelope`
  - the canonical public envelope for offline-readable decision evidence
  - binds semantic labels and authority-owned evidence digests without changing
    the operational decision
- `IntentAdmissionCertificationBundle`
  - the milestone-local certification artifact proving the lattice works across
    admitted, advisory, and violation lanes

Rules:

- no API may lower, execute, materialize, or route a covered intent family
  from `RawIntent` alone
- no API may construct `AdmittedIntentPlan`, `AdvisoryDecision`,
  `ViolationDecision`, `AdmittedExecutionHandoff`, `DecisionTraceEnvelope`, or
  `IntentAdmissionCertificationBundle` from raw strings, booleans, or ad hoc
  JSON
- no API may convert advisory or violation artifacts into admitted plans by
  convenience helper
- no execution receipt for a covered path may exist without a retrievable link
  back to the originating admitted execution handoff
- no API may attach richer diagnostics later by mutating the decision meaning;
  diagnostics may only enrich the envelope over already-canonical decisions
- no API may erase family-specific distinctions required for correctness just to
  fit one generic enum payload

## Compile-Time Enforcement Policy

Milestone 9.3.5 must make the shared lattice mechanically enforceable.

`Unrepresentable` in public types:

- publicly constructible admitted plans without the family-specific downstream
  proof they authorize
- publicly constructible advisory or violation decisions without an ordered
  decision-trace index and typed decision class
- publicly constructible decision traces that omit family label, decision
  class, phase label, and evidence digest references
- publicly constructible admission envelopes that can represent both admitted
  and violation outcomes by optional holes in one loose struct
- publicly constructible "admitted" markers that are not bound to one
  canonical raw intent and one ordered admission trace
- one mega handoff enum or mega plan struct whose correctness depends on
  optional fields, stringly seam labels, or runtime "kind" checks

`Uncompilable` through visibility and compile-fail enforcement:

- external construction of `RawIntent` family internals, `IntentEligibility`,
  `AdmissionDecision`, `AdmittedIntentPlan`, `AdvisoryDecision`,
  `ViolationDecision`, `AdmittedExecutionHandoff`, `DecisionTraceEnvelope`, or
  `IntentAdmissionCertificationBundle`
- public APIs that accept booleans such as `allowed`, `supported`, `warning`,
  `deferred`, or `retryable` instead of typed decision artifacts
- public APIs that lower raw intent directly when a shared admission surface is
  claimed
- public APIs that accept a string, enum tag, or loosely typed target in place
  of a family-specific admitted execution handoff
- public APIs that erase advisory-vs-violation distinctions under one generic
  non-admitted path when recovery behavior differs
- public APIs that patch decision traces after the admitted plan has been
  consumed operationally

`Construction-time rejection`:

- attempts to assemble an admitted plan from mismatched family-specific
  eligibility evidence
- attempts to treat deferred future-neighbor posture as admitted support
- attempts to attach lower-runtime routing support where no already-real
  bridge/runtime execution seam exists
- attempts to assemble a decision trace without the required authority-owned
  evidence digests for the chosen family
- attempts to execute a covered path without attaching admission provenance to
  the resulting route/evaluation/write receipt
- attempts to certify a family decision scope with only happy-path rows
- attempts to mix admitted/advisory/violation rows from different canonical
  intent meanings into one certification closure

## Target DX

Milestone 9.3.5 is not complete when the internals are merely correct. It is
complete when the public Query authoring and execution surface feels like one
intentional framework path.

The finished code should expose three clear layers:

- common-path authoring for ordinary callers
- explicit proof-chain authoring for advanced callers
- lower-runtime execution only through typed admitted handoffs

### Common Path

The ordinary surface should read like intent first, then admission, then
execution:

```rust
let admitted = query
    .read(UserProfileView::for_user(user_id))
    .at_basis(branch_basis)
    .admit()?;

let profile = query.execute(admitted.handoff()).await?;

profile.value();
profile.trace();
profile.performance();
```

Equivalent effect- and inspection-shaped paths should have the same rhythm:

```rust
let admitted = query
    .effect_write(effect_handle)
    .strategy_version("v3")
    .input_contract("rename-user")
    .admit()?;

let receipt = query.execute(admitted.handoff()).await?;
```

```rust
let admitted = query
    .inspect(last_receipt)
    .causal_history()
    .admit()?;

let report = query.execute(admitted.handoff()).await?;
```

### Advanced Path

The advanced path should expose the proof chain and plan inspection explicitly
rather than forcing engineers to infer it from hidden runtime behavior:

```rust
let raw_intent = QueryIntent::rename_user(user_id, "Spencer");
let eligibility = query.intent_admission().resolve_eligibility(&raw_intent)?;
let decision = query.intent_admission().decide(eligibility)?;

match decision {
    AdmissionDecision::Admitted(plan) => {
        plan.family();
        plan.policy_posture();
        plan.basis();
        plan.execution_seam();
        plan.cost();

        let handoff = plan.into_execution_handoff()?;
        let receipt = query.execute(handoff).await?;
        receipt.trace();
    }
    AdmissionDecision::Advisory(advisory) => {
        advisory.trace();
        advisory.remediation();
    }
    AdmissionDecision::Violation(violation) => {
        violation.trace();
        violation.reason();
    }
}
```

### Execution Boundary

Covered 9.3.5 execution must not accept raw requests, generic tags, or weak
plans. The execution boundary should look like this in spirit:

```rust
impl WORTHQueryRuntime {
    pub async fn execute<F, S>(
        &mut self,
        handoff: AdmittedExecutionHandoff<F, S>,
    ) -> Result<ExecutionReceipt<F, S>, WORTHQueryRuntimeError>;
}
```

The finished API must make these principles obvious:

- common-path callers author semantic intent, not bridge vocabulary
- advanced callers can inspect eligibility, decision, plan, handoff, and
  execution provenance as distinct phases
- expensive work looks expensive
- covered execution accepts only typed admitted handoffs
- receipts and reports carry trace and provenance without requiring callers to
  rediscover why execution was legal

## Phases

The phases below are a required linear build order. They are not a buffet.
Phase N+1 may not begin until the completion gate for Phase N is satisfied in
code, tests, and exported topology. If implementation discovers a missing
prerequisite, the correct move is to amend the earlier phase, not to smuggle
that work into a later one.

### Phase 1: Freeze The Coverage Map And Authoring Front Doors

This phase exists to remove ambiguity before any shared types are designed.
The engineer must start by naming exactly which public Query entrypoints are in
scope, which lower execution seams they target, and which ones remain deferred.

Work to complete in order:

1. Define the canonical intent-family vocabulary for 9.3.5-covered work.
2. Enumerate every covered public entrypoint and every execution seam it may
   reach.
3. Map each covered entrypoint to one canonical raw-intent authoring
   constructor.
4. Mark every non-covered or future-neighbor path as deferred or unsupported
   in compile-visible support metadata.
5. Establish the target DX front doors the implementation will honor on the
   common path and advanced path.

Artifacts that must exist before leaving this phase:

- family-typed `RawIntent` authoring surface
- canonical family inventory
- compile-visible covered-entrypoint and execution-seam inventory
- support metadata naming admitted, advisory-capable, deferred, and unsupported
  posture per covered path

This phase is incomplete if:

- any covered method is still described only as "read family" or "inspection
  family" without a concrete method inventory
- any path is treated as covered even though no real bridge/runtime seam exists
- the intended common-path and advanced-path DX remains implicit

Completion gate:

- every covered public entrypoint has one named authoring path
- every covered execution seam is named
- every deferred path is explicit
- no engineer needs to guess what 9.3.5 does or does not cover before starting
  the next phase

### Phase 2: Build Eligibility As A Closed Pre-Execution Phase

Once the coverage map is frozen, the engineer must build the eligibility phase
that resolves all admissibility facts before any admitted plan or handoff can
exist.

Work to complete in order:

1. Define the shared eligibility vocabulary for policy, capability, basis,
   invariant, projection/source, and routing-support posture.
2. Adapt basis, effect, projection, inspection, and read-admission authorities
   into that vocabulary without reminting their proofs.
3. Make eligibility resolution the only legal path from `RawIntent` into the
   later decision phase.
4. Add compile-time or construction-time barriers that prevent direct movement
   from raw intent to admitted plan.
5. Add adversarial tests proving unsupported, stale, redacted, mismatched, and
   deferred cases stop here instead of leaking later.

Artifacts that must exist before leaving this phase:

- `IntentEligibility`
- family-aware eligibility adapters
- non-admitted precursors where the lattice needs them

This phase is incomplete if:

- any later phase still needs to rediscover basis, support, or invariant facts
- any family-specific authority gets cloned into Query-owned pseudo-authority
- unsupported combinations are widened into "best effort" eligibility

Completion gate:

- eligibility alone can explain why a family is admitted, advisory, deferred,
  or violating
- expensive construction and execution are still unreachable
- later phases can consume eligibility facts rather than raw requests

### Phase 3: Turn Eligibility Into Proof-Bearing Decisions And Plans

Only after eligibility is closed may the engineer define the canonical
decision, plan, and stop artifacts. This phase is where the public typestate
contract becomes real.

Work to complete in order:

1. Define the admitted, advisory, and violation decision artifacts.
2. Bind each decision to one canonical raw-intent identity and one eligibility
   record.
3. Define the family-specific admitted downstream plan types.
4. Define the family-specific admitted execution handoff types for covered
   execution seams.
5. Seal those handoff types so each execution seam can accept only the proof it
   actually understands.
6. Define advisory-stop and violation-stop terminal forms where execution does
   not proceed.

Artifacts that must exist before leaving this phase:

- `AdmissionDecision`
- `AdmittedIntentPlan`
- `AdmittedExecutionHandoff`
- `AdvisoryDecision`
- `ViolationDecision`
- `AdvisoryStop`
- `ViolationStop`

This phase is incomplete if:

- admitted plans are still loose bags rather than proof-bearing forms
- handoffs can be created from strings, tags, or optional-field structs
- advisory and violation still collapse into one generic non-admitted branch

Completion gate:

- every covered family emits one canonical decision shape
- every covered execution seam accepts only its sealed handoff type
- nothing operational can proceed without a proof-bearing admitted artifact

### Phase 4: Bind Execution To The Proof Chain

After plans and handoffs exist, the engineer must connect them to real covered
execution seams and make provenance survive the crossing. This phase is where
the milestone stops being an abstract lattice and becomes a true execution
contract.

Work to complete in order:

1. Connect each covered admitted handoff to its real bridge/runtime execution
   seam.
2. Rewrite supported legacy entrypoints so they delegate to the canonical
   authoring, eligibility, decision, and handoff path.
3. Ensure execution receipts, route artifacts, and evaluation artifacts retain
   a retrievable link to the originating admission and handoff digests.
4. Add parity tests proving the legacy public path and the canonical lattice
   path produce the same decision, handoff, and provenance chain.
5. Remove or lock down any side-door path that can still reach execution
   without the canonical handoff.

Artifacts that must exist before leaving this phase:

- execution-handoff adapters for all covered execution seams
- rewritten delegating public entrypoints
- execution provenance linkage in covered result artifacts

This phase is incomplete if:

- any supported public entrypoint still has an independent execution branch
- any covered receipt can exist without a handoff provenance chain
- any direct runtime seam can bypass the canonical admitted handoff

Completion gate:

- every covered execution path is visibly handoff-driven
- every covered legacy path delegates rather than emulates
- execution provenance is inspectable and certifiable

### Phase 5: Materialize Trace And Finished DX

Only after the real execution path is wired may the engineer finalize the
developer-facing story. This phase turns the proof chain into something callers
can read, inspect, and trust without spelunking internals.

Work to complete in order:

1. Define ordered decision-trace rows and the offline-readable trace envelope.
2. Attach authority-owned evidence digests to trace rows without changing
   operational meaning.
3. Implement common-path helpers that expose the target DX promised by this
   spec.
4. Implement advanced-path helpers for explicit eligibility, decision, plan,
   handoff, and provenance inspection.
5. Add golden transcripts and usage examples proving the public surface reads
   like intent first, proof chain second, and lower-runtime execution last.

Artifacts that must exist before leaving this phase:

- `DecisionTraceEnvelope`
- common-path facade helpers
- advanced-path inspection helpers
- golden transcripts matching the target DX

This phase is incomplete if:

- trace meaning depends on live runtime callbacks or hidden logs
- the common path still exposes lower-runtime vocabulary by default
- the advanced path cannot inspect phase boundaries directly

Completion gate:

- admitted, advisory, and violation outcomes all have canonical trace envelopes
- common-path usage matches the target DX section
- advanced-path usage can inspect every proof-bearing phase directly

### Phase 5.5: Close The Remaining Covered Family Adoptions

Phase 5 finishes the shared lattice and caller story for the initially wired
surfaces. Before certification can honestly claim milestone-wide closure,
9.3.5 must also finish the broader covered-family adoption this spec names.
This phase exists to prevent "runtime-floor complete" from being mistaken for
"milestone complete."

Work to complete in order:

1. Adopt concrete read-execution entrypoints into the shared lattice on named
   public methods rather than leaving reads represented only as deferred
   neighbors.
2. Audit every existing direct write-, update-, and delete-shaped public Query
   operation and rewrite any remaining bypass so each one delegates through the
   authoritative-intent lattice path before 9.3.5 can close.
3. Adopt concrete inspection and diagnostic-materialization entrypoints into
   the shared lattice on named public methods rather than certifying only
   representative advisory-redaction posture.
4. Adopt the pre-9.3.6 lower-runtime capability-routing request family on the
   concrete supported authoring surfaces that already have real bridge-backed
   execution semantics, while leaving truly future work explicitly deferred.
5. Promote basis-use and projection-consumption from "implemented in family
   inventory, support, and DX" to fully closed adopted families by extending
   their public advanced-path examples, trace expectations, and bundle-facing
   certification inputs until they no longer depend on neighboring milestone
   certification to explain their meaning.
6. Update the compile-visible covered-entrypoint inventory, support matrix,
   target DX, and crate documentation so every newly adopted family is named
   through the same public path, support posture, and proof vocabulary.

Artifacts that must exist before leaving this phase:

- covered read-execution authoring and review/admit surfaces on named public
  methods
- a checked mutation-entrypoint audit covering existing write/update/delete
  public surfaces and proving they now delegate through authoritative intent
- covered inspection or diagnostic-materialization authoring and
  review/admit/materialize surfaces on named public methods
- covered lower-runtime capability-routing authoring and deferred/admitted
  posture surfaces for the pre-9.3.6 boundary
- basis-use and projection-consumption DX and certification rows that are
  first-class members of the intent-admission bundle rather than only
  neighboring representative-family evidence
- updated coverage inventory rows, support rows, and documentation examples
  for every adopted family

This phase is incomplete if:

- read execution is still only a deferred neighbor rather than a named adopted
  covered entrypoint
- any existing public write/update/delete operation can still reach execution
  without delegating through authoritative intent
- inspection advisory-redaction exists only as representative certification
  evidence without a concrete lattice-owned public entrypoint
- lower-runtime capability-routing remains described only as future-neighbor
  posture for cases that already have real bridge-backed semantics available
- basis-use or projection-consumption can be executed through the lattice in
  code but are still absent from the public advanced-path story or from the
  intent-admission bundle's direct certification evidence
- the inventory, support matrix, DX examples, and docs disagree about which
  families are truly adopted

Completion gate:

- every family named in the Product Decision Lock has either a concrete adopted
  covered entrypoint in the inventory or an explicit deferred owner that is
  still outside 9.3.5 scope
- every existing mutation-shaped public entrypoint has been audited and proven
  to delegate through authoritative intent
- basis-use, projection-consumption, read execution, inspection or diagnostic
  materialization, effect execution, and the pre-9.3.6 routing boundary all
  resolve through one shared lattice vocabulary on concrete public surfaces
- Phase 6 can certify milestone-wide closure without relying on representative
  stand-ins for families that 9.3.5 claims to have adopted

### Phase 6: Certify Closure, Cost, And Non-Bypass Guarantees

Certification comes last. This phase exists to prove the earlier phases were
actually completed and not merely sketched.

Work to complete in order:

1. Add the named 9.3.5 certification suite.
2. Add public-boundary, proof-shape, phase-progression, and topology audits.
3. Add compile-fail coverage proving callers cannot mint plans, handoffs,
   traces, or certification artifacts directly.
4. Add hostile parity cases for admitted, advisory, violation, deferred, and
   unsupported paths across representative covered families.
5. Add observational-parity checks for legacy entrypoints versus canonical
   lattice paths.
6. Add exact counters and slope proofs for family lookup, eligibility
   resolution, trace assembly, and certification coverage width.

Artifacts that must exist before leaving this phase:

- `IntentAdmissionCertificationBundle`
- compile-fail boundary suite
- parity and non-bypass certification rows
- exact performance counters and slope digests

This phase is incomplete if:

- only happy-path admitted flows are certified
- legacy delegation parity is asserted but not tested
- performance claims exist without named counters and slope evidence

Completion gate:

- support metadata, execution behavior, DX transcripts, compile-fail
  boundaries, and certification rows all agree
- covered legacy entrypoints and canonical lattice entrypoints prove
  observational parity
- later milestones can rely on one closed, non-bypassable public admission and
  handoff model

### Phase 7: Write And Install The Crate Documentation

The milestone is not complete when the code and certification exist only for
engineers who already know the internals. The public crate documentation must
teach the finished admission and handoff model as a real framework surface.

Phase 7 inherits the post-Phase-5.5 surface, not the earlier runtime-floor
interpretation of 9.3.5. The documentation phase is therefore not allowed to
retell only the original runtime-intent floor and call that milestone closure.
It must document the concrete adopted covered-entrypoint inventory and support
posture finalized by Phase 5.5.

This phase should be executed with the standards embodied by the
`feature-doc-writer` skill: product-facing, implementation-grounded,
developer-teaching documentation rather than milestone prose or internal notes.

Work to complete in order:

1. Import the finalized 9.3.5 covered-entrypoint inventory, support matrix,
   target DX transcripts, and mutation-entrypoint audit as predecessor inputs
   for the documentation pass.
2. Identify the public `worth-query` surfaces introduced or materially changed
   by 9.3.5, including the concrete post-5.5 adopted families rather than only
   the original runtime-intent floor:
   - authoritative mutation intent execution
   - batch mutation intent execution
   - read-family execution
   - basis-context read-family execution
   - live read execution
   - derived materialization
   - generic and derived inspection
   - existing-truth probe routing
   - basis observation as a `no-execution-handoff` admitted family
   - projection consumption as a `no-execution-handoff` admitted family
3. Identify the public convenience wrappers that Phase 5.5 requires to remain
   thin delegates over the shared lattice, and make their relationship to the
   canonical intent path explicit in the docs. This includes at minimum:
   - `workspace.verify_existing(...)`
   - `workspace.update_existing_verified(...)`
   - `workspace.delete_existing_verified(...)`
   - `workspace.read(...)`
   - `workspace.materialize(...)`
   - `workspace.inspect(...)`
   - `runtime.probe_existing(...)`
   - `workspace.probe_existing(...)`
4. Write crate-facing documentation that explains:
   - what intent admission is for
   - the common-path authoring and execution flow
   - the advanced proof-chain flow
   - admitted, advisory, deferred, and violation outcomes
   - execution handoffs and why covered execution requires them
   - how traces, provenance, inspection, and routing fit into normal usage
   - which covered families execute through a runtime handoff versus which
     admitted families terminate in scoped basis or bound-contract artifacts
5. Add practical examples that match the target DX promised by this spec and
   collectively cover the post-5.5 family set rather than only representative
   runtime-intent examples.
6. Install that documentation into the crate documentation surface so it ships
   with `worth-query`, rather than living only in milestone docs.
7. Verify that the crate documentation uses the final public names and does not
   rely on internal module topology or milestone vocabulary.
8. Verify that the crate documentation agrees with the finalized
   covered-entrypoint inventory, support matrix, DX/golden transcript set, and
   mutation-entrypoint audit instead of telling a separate "close enough"
   public-surface story.

Artifacts that must exist before leaving this phase:

- crate documentation pages or sections covering the 9.3.5 public surface
- runnable or compile-checked examples where the crate documentation policy
  expects them
- cross-links from the crate documentation to the relevant public types and
  entrypoints
- explicit documentation coverage for the post-5.5 adopted family set and the
  thin convenience wrappers that delegate into it

This phase is incomplete if:

- the only explanation of 9.3.5 still lives in milestone planning docs
- the crate documentation describes the internal implementation but not the
  caller experience
- examples drift from the target DX or use obsolete public naming
- the crate documentation still teaches only the original runtime-intent floor
  and omits concrete adopted read, inspection, routing, basis, or projection
  surfaces finalized by Phase 5.5
- convenience wrappers are documented as independent feature surfaces rather
  than thin delegates over the canonical intent lattice where that is the real
  implementation contract

Completion gate:

- `worth-query` crate documentation teaches the 9.3.5 surface directly
- the docs are concrete enough that a new engineer can use the common path and
  discover the advanced path without spelunking implementation modules
- the docs match the finalized covered-entrypoint inventory, support matrix,
  DX transcripts, and mutation-entrypoint audit from Phase 5.5
- the milestone cannot close until this documentation is present in the crate
  documentation surface

## Required Topology

Milestone 9.3.5 should map into responsibility-specific subdomains.

Required subdomains:

- `intent_admission/families`
  - owns family inventory, family labels, and family support posture
- `intent_admission/eligibility`
  - owns shared eligibility vocabulary and family adapters
- `intent_admission/decisions`
  - owns admitted/advisory/violation decision artifacts and downstream plan
    shaping
- `intent_admission/handoffs`
  - owns family-specific admitted execution handoffs and provenance binding
- `intent_admission/trace`
  - owns decision-trace rows, trace indexes, and offline-readable envelopes
- `intent_admission/dx`
  - owns facade helpers and golden transcript support
- `intent_admission/support`
  - owns executable support matrices and future-neighbor/deferred posture
- `intent_admission/certification`
  - owns audits, bundles, oracle rows, slope reports, and compile-fail
    fixtures

Forbidden topology:

- one generic `admission.rs` bag that mixes family inventory, eligibility,
  decision shaping, tracing, DX, and certification
- one generic `handoff.rs` bag that erases seam-specific proof shape behind
  string tags or optional fields
- host/test helper modules becoming the de facto source of family decision
  truth
- a shared enum so generic that family-specific correctness distinctions become
  comment-only

## Must Ship

- a shared public admission decision lattice for Query-crossing intent
  families
- family-typed `RawIntent`, `IntentEligibility`, `AdmissionDecision`,
  `AdmittedIntentPlan`, `AdmittedExecutionHandoff`, `AdvisoryDecision`,
  `ViolationDecision`, and `DecisionTraceEnvelope`
- a compile-visible inventory of covered entrypoints, family adapters,
  execution seams, and provenance expectations
- bridge-backed execution handoff coverage for every 9.3.5 family that already
  has a real execution seam in the current Query runtime boundary
- executable support matrices and family inventories covering admitted,
  advisory-capable, deferred, and unsupported family combinations
- decision traces that preserve policy, capability, invariant, basis,
  projection/source, and lower-runtime-routing posture where applicable
- common-path facade helpers and golden transcripts for admitted, advisory, and
  violation inspection
- compile-fail and construction-boundary enforcement proving non-admitted
  artifacts cannot be lowered as admitted plans
- certification bundles, proof-shape audits, public-boundary audits, and exact
  slope evidence

## Must Preserve

- eligibility precedes expensive construction, lowering, execution, and
  materialization
- diagnostics and richer traces can explain decisions without changing
  operational meaning
- basis, effect, projection, inspection, bridge, relational, and signal
  authorities retain ownership of their own decision evidence
- binary convenience results are derived summaries only
- family-specific correctness distinctions survive inside the shared lattice
- deferred temporal, async/resource, store-backed, and durable neighbors remain
  explicit later-milestone posture rather than silent partial support

## Acceptance Evidence

This milestone is complete only when a hostile certification program can:

- author equivalent raw intents for the same family through more than one
  admitted public path and prove they normalize to the same admission meaning
- prove intentionally different policy, basis, source, support, invariant, or
  routing-support choices change the declared digests
- prove advisory and violation outcomes are as inspectable and offline-readable
  as admitted outcomes
- prove downstream lowering/execution/materialization phases consume admitted
  plans rather than revalidating raw intents
- prove covered route/evaluate execution paths consume typed admitted
  execution handoffs rather than rediscovering admission from raw requests
- prove every supported legacy entrypoint and every canonical lattice entrypoint
  for the same covered path converge to the same decision, handoff, and
  execution provenance chain
- prove unsupported, stale, masked, mismatched, and deferred neighbors fail
  typed before expensive work begins
- prove decision trace richness changes only explanation detail, not the
  admission outcome

## Required Verification Output

The 9.3.5 certification bundle must emit:

- `query_digest`
- `intent_family_digest`
- `raw_intent_digest`
- `intent_eligibility_digest`
- `admission_decision_digest`
- `admitted_intent_plan_digest`
- `admitted_execution_handoff_digest`
- `covered_entrypoint_inventory_digest`
- `execution_seam_inventory_digest`
- `advisory_decision_digest`
- `violation_decision_digest`
- `decision_trace_digest`
- `decision_trace_envelope_digest`
- `policy_decision_digest`
- `capability_decision_digest`
- `invariant_decision_digest`
- `basis_decision_digest`
- `projection_decision_digest`
- `routing_posture_digest`
- `intent_family_inventory_digest`
- `intent_support_matrix_digest`
- `intent_public_surface_digest`
- `intent_target_dx_digest`
- `intent_golden_transcript_digest`
- `decision_proof_shape_digest`
- `decision_phase_progression_digest`
- `execution_provenance_chain_digest`
- `decision_oracle_digest`
- `decision_support_traceability_digest`
- `seeded_sequence_digest`
- `seed_replay_digest`
- `seed_generator_class_digest`
- `compile_fail_boundary_digest`
- `failure_digest`
- `counter_snapshot`
- `intent_family_lookup_width`
- `decision_trace_width`
- `admission_classification_slope_digest`
- `decision_trace_assembly_slope_digest`
- `decision_support_lookup_slope_digest`
- `decision_certification_coverage_slope_digest`

## Architectural Notes

- The shared admission lattice is a public Query contract, not a generic error
  facade.
- A family-specific admitted plan is still the only legal input to later
  lowering/execution/materialization phases. 9.3.5 standardizes admission; it
  does not erase downstream family shape.
- Advisory is not "soft success." It is a typed decision class with explicit
  adaptation or stop conditions.
- Violation is not a string error. It is a fail-closed proof that later phases
  may not proceed.
- Decision traces are derived from authority-owned evidence; they do not
  replace basis, projection, effect, bridge, relational, or signal ownership.
- The trace envelope is part of the product contract. Logs are not.
- Lower-runtime capability-routing intent may be admitted only to the extent
  that 9.3.5 can bind it to a real already-authoritative execution seam.
- 9.3.5 owns the shared admission lattice and the canonical typed handoff into
  that execution seam.
- 9.3.6 still owns the broader cleanup: remaining boundary envelopes,
  compatibility debt, and elimination of scattered direct lower-runtime
  contact.

## Deferred Scope

The following remain explicitly deferred:

- durable decision-log archives
- restart-stable trace reload
- store-backed admission replay
- temporal query-basis admission families owned by 9.4
- async/resource admission families owned by 9.5
- mixed truth/time/async delivery admission families owned by 9.6
- final temporal/async certification closure owned by 9.7

Any 9.3.5 surface that encounters those families must report typed deferred or
unsupported posture.

## Sequencing Notes

Milestone 9.3.5 belongs after 9.3.4 because basis use, effect execution,
inspection, and projection consumption need to exist as real family-owned
lifecycles before one shared admission lattice can unify them honestly.

It belongs before 9.3.6 because lower-runtime routing should consume admitted
intent plans, typed execution handoffs, and decision traces rather than
inventing another admission model.

It belongs before the Runtime API Public Stabilization Gate because the public
runtime facade should freeze around one shared Query admission vocabulary.

## Closeout Standard

This milestone may close only when:

- the 9.3.5 phases have been implemented in order, or any deviation was first
  approved by a spec amendment
- every family promised by this spec visibly uses the shared lattice
- advisory and violation lanes are as typed, inspectable, and certified as
  admitted lanes
- compile-fail boundaries prove external callers cannot mint admitted plans,
  advisory/violation decisions, decision traces, or certification artifacts
  directly
- support metadata, executable behavior, DX transcripts, and certification
  coverage agree across admitted, advisory, deferred, unsupported, and
  violating families
- exact counters and slope digests prove admission and trace assembly costs are
  bounded by declared decision width and family coverage width
- roadmap and test-requirement references point at this spec and its named
  certification suite accurately

## Self-Check

- Does the milestone solve a real structural problem or just package work
  cosmetically? Yes: it closes the missing public admission model across the
  already-built 9.3.x lifecycles.
- Is the adversarial constraint precise and load-bearing? Yes: it forbids the
  common failure where Query returns binary or family-specific admission
  outcomes that downstream code must reinterpret manually.
- Does the milestone preserve crate authority boundaries? Yes: Query owns the
  shared lattice and trace envelope while basis, projection, effect,
  inspection, relational, bridge, and signal authorities retain their own
  decision evidence.
- Does the milestone define proof obligations, not just implementation tasks?
  Yes: family adoption, typed advisory/violation closure, compile-fail
  boundaries, trace envelopes, support matrices, and exact slope evidence are
  all explicit requirements.
- Could a competent engineer map this spec into honest types, modules, and
  tests? Yes: the phase progression, topology, and verification outputs name
  the required artifacts directly.
- Does the milestone belong in this roadmap sequence, or is it out of order?
  Yes: it consolidates the 9.3.1 through 9.3.4 lifecycles before 9.3.6 routing
  and the runtime API freeze consume them.
