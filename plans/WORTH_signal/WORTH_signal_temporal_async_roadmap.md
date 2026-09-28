# WORTH Signal Temporal And Async Runtime Roadmap

## Purpose

This document defines the next core-runtime roadmap for `worth-signal` after
the currently closed observation substrate work.

It is intentionally core-only.

This roadmap does not plan wasm ergonomics, React helpers, route-resource
adapters, forms APIs, or query-replacement product surfaces directly. Its job
is narrower and more foundational:

- make time a first-class runtime primitive
- make async/resource lifecycle first-class and then broaden it into an
  attachable node capability

The operating rule for this roadmap is:

`admit temporal and async meaning once, lower it once, execute it against canonical runtime truth`

That rule governs every phase in this roadmap:

1. time and async semantics must be runtime-owned artifacts rather than host
   callback conventions
2. planning, legality, and lifecycle classification must happen before hot-path
   execution or completion handling
3. `worth-signal` must remain the owner of derived-computation truth, not
   truth-state authority, transport delivery, or frontend framework semantics
4. later wasm, query, route-resource, and form surfaces must inherit these
   runtime semantics rather than inventing parallel ones

## Relationship To Existing Signal Docs

This roadmap extends the direction already named in:

- [worth_signals2.md](./worth_signals2.md)
- [worth_signal_vision.md](./worth_signal_vision.md)
- [signal_architecture2.md](./signal_architecture2.md)
- [milestone-11-closeout.md](./milestone-11-closeout.md)
- [milestone-a-plan.md](./milestone-a-plan.md)
- [milestone-a-closeout.md](./milestone-a-closeout.md)
- [milestone-b-plan.md](./milestone-b-plan.md)
- [milestone-b-closeout.md](./milestone-b-closeout.md)
- [milestone-c-plan.md](./milestone-c-plan.md)
- [milestone-c-closeout.md](./milestone-c-closeout.md)
- [test-requirements.md](./test-requirements.md)

The key continuity is:

- `worth-signal` already owns conditional execution, rollback, observation,
  diagnostics, replay, and branch-aware runtime state
- the vision already names temporal and previous-value support as real runtime
  goals
- the current runtime still treats time as mostly host-resolved condition
  policy, and it does not yet own async/resource lifecycle truth

This roadmap exists to close that gap without smearing core semantics into wasm
or application-layer convenience APIs.

## Adversarial Constraint

`worth-signal` must survive the following hostile condition:

> A branchable, replayable runtime with deterministic execution, rollback-safe
> observation, time-gated nodes, previous-value-sensitive nodes, and
> async-capable nodes must converge to the same committed derived truth,
> the same lifecycle classifications, and the same diagnostic explanation
> regardless of whether work was driven by direct invalidation, logical time
> advance, async completion, retry/cancellation, snapshot restore, or branch
> replay.

If any supported path:

- lets temporal eligibility depend on opaque host callbacks instead of
  runtime-owned clock semantics
- lets async completion commit stale or superseded work over newer admitted
  intent
- changes node meaning between one-shot evaluation, replay, restore, and branch
  re-entry
- treats pending/fulfilled/rejected/cancelled as adapter-local UI states rather
  than runtime truth
- forces async capability to live in a separate conceptual node species instead
  of composing with the ordinary node model
- hides broad timer scans, broad inflight scans, or per-node allocation churn
  behind cheap-looking APIs
- forces later wasm or query layers to define their own time or async truth
  model

then `worth-signal` has failed.

## Roadmap Rules

Rules for the temporal/async roadmap phases:

- each phase must define a real runtime capability boundary, not just new
  builder methods
- each phase must preserve the authority split:
  `worth-relational` owns truth, `worth-store` owns persistence,
  `worth-signal` owns derived execution and lifecycle truth
- runtime time and async semantics must remain canonical artifacts rather than
  ambient host conventions
- no phase is complete until replay, rollback, branch, restore, and diagnostics
  parity are proven for the newly admitted semantics
- every hot-path phase must declare named complexity contracts and exact counter
  proof obligations
- any knowingly incomplete first ship must remain explicit debt rather than
  implied product completion
- later wasm, query, and form/resource layers may consume these capabilities,
  but this roadmap does not let them redefine them

## Core-Only Boundary

This roadmap is deliberately limited to `worth-signal`.

In scope:

- runtime clock and temporal eligibility semantics
- previous-value and temporal context needed by core signal evaluation
- async/resource lifecycle semantics
- retry, cancellation, supersession, and completion admission semantics
- temporal and async diagnostics, replay, history, and branch behavior
- core counters, proof artifacts, and certification requirements

Out of scope:

- wasm bindings
- React hooks
- Angular adapters
- route-resource APIs
- signal-based forms APIs
- query replacement facade work
- network delivery and persistence concerns beyond the runtime artifacts those
  later layers will need

## Critical Path

There is one strict dependency order:

- `Phase 1: Temporal Runtime Substrate`
- `Phase 2: Async And Resource Node Runtime Substrate`
- `Phase 3: Async Resource Policy Families`
- `Phase 4: Async Capability On Arbitrary Nodes`

The order is intentional.

Async/resource nodes need runtime-owned time semantics for:

- debounce and throttle legality
- stale-after and freshness windows
- retry and backoff
- timeout and cancellation deadlines
- deterministic replay of temporal admission and completion order

If async lands before time, the runtime will either hardcode weak temporal
policy or push it back into host glue, which would poison the foundation.

Async resource policy families come after the async/resource substrate because
policy variation must consume canonical lifecycle truth, request identity,
temporal wake proof, transaction apply, and replay artifacts rather than define
them.

The capability-generalization phase comes after policy-family closeout because
it broadens how the substrate is expressed. It must reuse the closed lifecycle
and policy truth rather than redefining them.

## Phase 1: Temporal Runtime Substrate

See [milestone-a-plan.md](./milestone-a-plan.md) for the concrete engineering
specification for this phase.

### Goal

Make time a first-class runtime primitive so that temporal gating, previous-
value-sensitive computation, and replayable temporal eligibility are owned by
`worth-signal` instead of by host-supplied condition callbacks.

### Adversarial Constraint

The same graph, starting from the same authoritative inputs and the same clock
basis, must produce the same temporal eligibility decisions, the same wake
ordering, and the same committed outputs across ordinary execution, branch
fork/restore, snapshot restore, and replay.

### Why This Phase Exists

`worth-signal` already has conditional execution, but time is not yet a real
runtime-owned semantic axis.

Today:

- `OnDemand` is runtime-owned
- `Debounce(...)` exists as a node condition
- custom gating exists

But the runtime still depends on host `ConditionResolver` behavior for temporal
readiness rather than owning:

- clock domains
- scheduled wake eligibility
- temporal provenance
- branch/restore/replay semantics for time

Without this phase:

- debounce remains partially host-defined
- previous-value and temporal computation remain awkward or adapter-local
- later async/resource semantics will inherit a weak temporal model
- later route-resource and form surfaces will be forced to invent a second time
  semantics path

### Must Ship

- a runtime-owned clock abstraction and explicit clock-domain vocabulary
- a sealed first-class temporal policy family, including `After`,
  `AtOrAfter`, `Debounce`, `Throttle`, `StaleAfter`, and `Interval` or direct
  canonical equivalents
- explicit temporal eligibility artifacts rather than only host callback
  decisions
- scheduled wake ownership and deterministic wake ordering
- temporal invalidation causes as first-class runtime facts
- previous-value access semantics sufficient for time-sensitive derived nodes
- branch/snapshot/replay-aware temporal state
- diagnostics-visible temporal provenance
- public core runtime surfaces that let hosts advance or supply time honestly
  without redefining temporal semantics

### Must Preserve

- deterministic execution remains a product contract
- rollback and commit-bounded observation semantics remain unchanged
- time remains derived execution truth, not a truth-state authority surface
- hosts may supply clock inputs, but they may not define temporal meaning
  ad hoc after admission
- temporal state must remain reconstructable from checkpoint plus bounded
  history, not ambient process memory alone

### Complexity / Proof Obligations

- name exact contracts for:
  - clock advance
  - scheduled wake admission
  - ready-node selection
  - temporal replay/restore reconstruction
- expose exact counters for:
  - temporal wake count
  - deferred-by-time count
  - ready-queue width
  - branch-local temporal restoration count
  - temporal replay parity checks
  - temporal broad-scan denial count
- prove the runtime never widens timer handling into whole-graph scans when the
  ready temporal surface is narrower

### Allowed Debt

- richer temporal policy families may remain later work if the foundational
  clock, wake, and previous-value substrate is already canonical and parity-
  proven
- adapter ergonomics are explicitly out of scope and may not be used as a
  substitute for substrate completeness

### Acceptance Evidence

This phase is complete only when `worth-signal` can prove:

- the `Temporal Eligibility Replay Parity Test`
- the `Temporal Branch Restore Equivalence Test`
- the `Temporal Wake Boundedness Test`
- the `Previous-Value And Time-Gated Node Equivalence Test`

with canonical machine-checkable artifacts for:

- wake ordering
- temporal eligibility decisions
- node output digests
- branch/snapshot digests
- diagnostics/explanation digests

## Phase 2: Async And Resource Node Runtime Substrate

See [milestone-b-plan.md](./milestone-b-plan.md) for the concrete engineering
specification for this phase and
[milestone-b-closeout.md](./milestone-b-closeout.md) for the formal closeout
acceptance map.

### Goal

Make async/resource lifecycle first-class runtime truth so pending, fulfilled,
rejected, cancelled, stale, and superseded states are owned by `worth-signal`
itself rather than by adapters layered above it.

### Adversarial Constraint

The same admitted async/resource request, when subjected to overlapping
revalidation, retries, cancellation, branch churn, snapshot restore, replay,
and out-of-order completion, must converge to the same committed resource
truth, reject the same stale completions, and emit the same diagnostics and
observation boundaries.

### Why This Phase Exists

Once time is runtime-owned, the next missing substrate is async/resource truth.

The current runtime can express:

- dirty, maybe-stale, and clean node states
- condition-deferred evaluation
- committed observation

But it does not yet own:

- pending resource state
- generation-safe in-flight tracking
- stale completion rejection
- retry/backoff lifecycle
- cancellation and supersession semantics
- branch/replay parity for inflight work

Without this phase:

- wasm and application layers will model async as UI-local state
- route-resource and form surfaces will have no canonical runtime substrate
- a future query replacement will be forced to invent request/cache semantics
  outside the runtime

### Must Ship

- first-class async/resource node lifecycle vocabulary
- request identity and generation/epoch-safe completion semantics
- in-flight registration, cancellation, supersession, and retry ownership
- completion admission that re-enters the runtime transactionally
- resource-state-aware observation and diagnostics
- branch/snapshot/replay semantics for inflight and completed resource state
- explicit failure and stale-completion denial classifications
- public core runtime surfaces for:
  - admitting async work
  - completing work
  - cancelling work
  - retrying or revalidating work

### Must Preserve

- async/resource nodes remain derived state, not authority
- completion order may vary physically, but committed truth must remain
  generation-safe and replay-honest
- no stale or superseded completion may commit over newer admitted intent
- retry and cancellation must be framework-owned lifecycle, not caller-owned
  convention
- observation remains commit-bounded and rollback-safe even when driven by
  async completion

### Complexity / Proof Obligations

- name exact contracts for:
  - inflight registration
  - completion admission
  - stale-completion rejection
  - cancellation
  - retry/backoff scheduling
  - async replay and restore reconstruction
- expose exact counters for:
  - inflight request count
  - fulfilled count
  - rejected count
  - cancelled count
  - superseded completion denial count
  - retry admission count
  - async branch restore count
  - async replay parity checks
  - inflight broad-scan denial count
- prove completion handling scales with the inflight/request-local surface and
  not with total graph size

### Allowed Debt

- higher-level cache products, transport adapters, and framework-specific
  resource APIs may remain later work
- the runtime may defer domain-specific resource policy families so long as the
  lifecycle substrate is canonical, generation-safe, and parity-proven

### Acceptance Evidence

This phase is complete only when `worth-signal` can prove:

- the `Async Resource Lifecycle Parity Test`
- the `Out-Of-Order Completion Supersession Test`
- the `Async Rollback And Observation Equivalence Test`
- the `Async Branch Restore And Replay Equivalence Test`
- the `Async Inflight Boundedness Test`

with canonical machine-checkable artifacts for:

- resource lifecycle digests
- committed output/resource-state digests
- completion denial classifications
- observation boundary digests
- replay/restore digests
- diagnostics/explanation digests

### Closeout Status

Phase 2 is closed by
[milestone-b-closeout.md](./milestone-b-closeout.md).

The closeout gate is the sealed `ResourceMilestoneBCertificationRun`, which
requires a complete certification bundle, scenario matrix, hostile completion
evidence, and performance closeout before the phase can be treated as passed.

## Phase 3: Async Resource Policy Families

See [milestone-c-plan.md](./milestone-c-plan.md) for the concrete engineering
specification for this phase and
[milestone-c-closeout.md](./milestone-c-closeout.md) for the formal closeout
acceptance map.

### Goal

Complete the async/resource policy layer by building deterministic, descriptor-
backed policy families for retry, timeout, cancellation, supersession,
revalidation, observation, output continuity, retention, diagnostics, and
replay compatibility.

### Adversarial Constraint

The same async/resource workload, driven through different declared policy
families, must preserve the same hard lifecycle laws, expose the policy choices
that changed operational behavior, and replay deterministically or deny with a
typed compatibility artifact.

### Why This Phase Exists

Milestone B freezes the extensibility boundary, but async is too important to
leave policy richness as a vague later adapter concern.

Without this phase:

- route-resource, query, form, and background-refresh products will each invent
  separate retry, timeout, visibility, and retention semantics
- retry storms, timeout drift, and retention truncation will be product bugs
  instead of certified runtime behavior
- replay and diagnostics will not be able to explain why one policy admitted a
  retry, cancelled host work, preserved output, or retained history

### Must Ship

- complete frozen registries for all async/resource policy families
- built-in policy families for retry, timeout, cancellation, supersession,
  revalidation, observation, output continuity, retention, diagnostics, and
  replay compatibility
- typed policy denial classifications for unknown, duplicate, incompatible,
  budget-exhausted, and semantically illegal policy decisions
- policy-specific boundary performance envelopes
- certification suites for retry budgets/backoff, deadline behavior,
  cancellation/supersession, revalidation/freshness, observation/visibility,
  retention/diagnostics, and replay compatibility

### Must Preserve

- lifecycle legality remains runtime law, not policy preference
- policy variation is resolved before execution and consumed as lowered
  descriptor truth
- host callbacks do not decide runtime legality inside the hot path
- public product layers consume policy truth instead of redefining it

### Acceptance Evidence

This phase is complete only when `worth-signal` can prove:

- the `Async Resource Policy Family Certification Test`
- the `Async Retry Budget And Backoff Certification Test`
- the `Async Timeout Deadline Certification Test`
- the `Async Cancellation Supersession Policy Certification Test`
- the `Async Revalidation Freshness Certification Test`
- the `Async Observation Output Continuity Certification Test`
- the `Async Retention Replay Policy Certification Test`

with canonical machine-checkable artifacts for policy descriptors, selection
bases, decision outcomes, replay compatibility, diagnostics, and boundary
performance envelopes.

### Closeout Status

Phase 3 is closed by
[milestone-c-closeout.md](./milestone-c-closeout.md).

The closeout gate is the sealed `ResourceMilestoneCCertificationRun`, which
requires a complete policy certification bundle, scenario matrix, performance
closeout, replay compatibility evidence, compile-fail proof boundaries, and
the crucial mixed hostile workload suites before the phase can be treated as
passed.

## Phase 4: Async Capability On Arbitrary Nodes

See [milestone-d-plan.md](./milestone-d-plan.md) for the concrete engineering
specification for this phase and
[milestone-d-closeout.md](./milestone-d-closeout.md) for the formal closeout.

### Goal

Make async a first-class capability attachable to ordinary nodes so the runtime
keeps one graph model while preserving separate graph dirtiness and async
lifecycle truth.

### Adversarial Constraint

The same node, when viewed through capability-first and legacy
resource-shaped APIs, and when combined with conditions, previous-value logic,
aspects, partitions, branch restore, and replay, must converge to the same
committed truth and the same denial/explanation artifacts.

### Why This Phase Exists

Milestones B and C correctly closed lifecycle and policy truth in
resource-shaped terms, but the long-term product vision is broader:

- async should be a capability nodes can opt into
- conditions and aspects should compose with async capability directly
- future query, route, form, and refresh APIs should not need to explain a
  second node species

### Must Ship

- explicit async-capability attachment on ordinary nodes
- capability-aware node declaration and builder vocabulary
- condition-, temporal-, and previous-value-aware async admission on ordinary
  nodes
- aspect-/partition-aware async-capable nodes
- async-capable keyed/query/computed families
- compatibility vocabulary that keeps legacy resource-shaped APIs lowering
  through the same substrate

### Must Preserve

- node dirtiness remains `Clean | MaybeStale | Dirty`
- async lifecycle remains orthogonal runtime truth
- lifecycle law and policy law remain the Milestone B/C substrate, not a new
  abstraction layer
- capability attachment may broaden expression, but may not broaden legality

### Acceptance Evidence

This phase is complete only when `worth-signal` can prove:

- the `Async Capability Attachment Equivalence Test`
- the `Condition-Gated Async Admission Parity Test`
- the `Aspect-Scoped Async Capability Test`
- the `Previous-Value And Temporal Async Capability Parity Test`
- the `Interior Async Node Gate Equivalence Test`
- the `Hierarchical Async Capability Replay And Cancellation Test`
- the `Legacy Resource Alias Compatibility Test`
- the `Async Capability Compile-Time Boundary Test`

### Closeout Status

Phase 4 is closed.

See [milestone-d-closeout.md](./milestone-d-closeout.md).

The closeout gate is the sealed `AsyncNodeMilestoneDCertificationRun`, which
now requires:

- compile-time boundary proof ownership
- exact scenario coverage
- exact performance-envelope coverage
- keyed/gate/hierarchy historical and explanation-lineage agreement
- direct strict-suite evidence for public rediscovery, branch churn, and
  combined restore pressure

## Per-Phase Format

Each phase in this roadmap is intentionally small in count but heavy in
obligation surface.

For each phase, read `Must Ship` as four separate obligations:

- surface primitives
- semantic guarantees
- proof obligations
- explicit debt boundaries

If a future revision only adds API vocabulary without also naming semantics and
proof obligations, the roadmap is incomplete.

## Completion Standard

This roadmap is complete only when all of the following are true:

- time is runtime-owned rather than host-interpreted
- previous-value-sensitive and time-gated nodes are replay- and branch-honest
- async/resource lifecycle is runtime-owned rather than adapter-local
- async/resource policy variation is descriptor-owned rather than adapter-local
- async capability is attachable to arbitrary nodes without inventing a second
  semantic species
- pending, fulfilled, rejected, cancelled, stale, and superseded states are
  canonical runtime truths
- branch, snapshot, restore, replay, rollback, diagnostics, and observation all
  preserve one semantic story for temporal and async work
- retry, timeout, cancellation, supersession, revalidation, observation,
  output-continuity, retention, diagnostics, and replay policies are fully
  certified runtime families
- later wasm, query, route-resource, and form layers can consume these
  semantics without needing a second truth model

## Successor Handoff To The Numbered Core-Runtime Roadmap

The temporal/async sequence is closed through Milestone D. The next numbered
`worth-signal` engineering sequence inherits all of those guarantees:

1. [Milestone 12 - Aspect-Causal Invalidation](./milestone-12-plan.md)
2. [Milestone 13 - Locality-First Frontier Execution](./milestone-13-plan.md)
3. [Milestone 14 - Deterministic Parallel Execution Foundation](./milestone-14-plan.md)
4. [Milestone 15 - Proof-Carrying Graph Parallelism](./milestone-15-plan.md)
5. [Milestone 16 - Structured Partitioned Parallelism](./milestone-16-plan.md)
6. [Milestone 17 - Portable Execution Backends And Distributed Coordination](./milestone-17-plan.md)

Milestones 12-13 expand the fintech financial world and repair and certify the
invalidation substrate during implementation. Milestones 14-17 then establish
bounded deterministic execution, proof-carrying graph parallelism, domain-
agnostic structured partition work, and portable backend execution. None of
them reopens temporal or async lifecycle meaning.

The strict continuation order is intentional:

- parallel execution may not amplify an invalidation frontier before causal
  correctness and locality are certified
- graph parallelism consumes the bounded resource and publication authority
  established by Milestone 14
- partitioned inner computation consumes the graph task's same hierarchical
  resource lease rather than creating a second pool
- WASM workers, native threads, accelerators, and remote processes remain
  backend mechanisms behind the portable Milestone 17 boundary

## Companion Documents

- [worth_signals2.md](./worth_signals2.md)
- [worth_signal_vision.md](./worth_signal_vision.md)
- [signal_architecture2.md](./signal_architecture2.md)
- [milestone-11-closeout.md](./milestone-11-closeout.md)
- [milestone-a-closeout.md](./milestone-a-closeout.md)
- [milestone-b-closeout.md](./milestone-b-closeout.md)
- [milestone-c-closeout.md](./milestone-c-closeout.md)
- [milestone-d-closeout.md](./milestone-d-closeout.md)
- [milestone-d-plan.md](./milestone-d-plan.md)
- [milestone-12-plan.md](./milestone-12-plan.md)
- [milestone-13-plan.md](./milestone-13-plan.md)
- [milestone-14-plan.md](./milestone-14-plan.md)
- [milestone-15-plan.md](./milestone-15-plan.md)
- [milestone-16-plan.md](./milestone-16-plan.md)
- [milestone-17-plan.md](./milestone-17-plan.md)
- [test-requirements.md](./test-requirements.md)
- [MENTALITY.md](../../docs/coding-guidelines/MENTALITY.md)
- [arch_laws.md](../../docs/coding-guidelines/arch_laws.md)
- [perf_laws.md](../../docs/coding-guidelines/perf_laws.md)
- [domain_laws.md](../../docs/coding-guidelines/domain_structure_laws.md)

The purpose of this roadmap is not to make `worth-signal` feel more modern by
name. It is to make temporal and async/resource semantics part of the same
truthful runtime substrate that already owns invalidation, recomputation,
rollback, observation, diagnostics, and replay.
