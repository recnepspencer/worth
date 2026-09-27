# Milestone B Engineering Spec: Async And Resource Node Runtime Substrate

> **Status:** Closed
>
> **Closeout:** [milestone-b-closeout.md](./milestone-b-closeout.md)
>
> **Roadmap parent:** [worth_signal_temporal_async_roadmap.md](./worth_signal_temporal_async_roadmap.md)
>
> **Vision parents:**
> - [worth_signals2.md](./worth_signals2.md)
> - [worth_signal_vision.md](./worth_signal_vision.md)
>
> **Architecture parent:** [signal_architecture2.md](./signal_architecture2.md)
>
> **Test requirements:** [test-requirements.md](./test-requirements.md)
>
> **Prerequisite milestone:**
> - [milestone-a-closeout.md](./milestone-a-closeout.md)
>
> **Primary architectural driver:** make async/resource lifecycle runtime-owned
> derived truth so pending, fulfilled, rejected, cancelled, stale, superseded,
> retried, and timed-out states stop being adapter-local conventions.

## Summary

Milestone B makes async and resource nodes first-class runtime primitives in
`worth-signal`.

This milestone is not "add futures to nodes."

It is:

- runtime-owned resource lifecycle truth
- explicit async/resource request identity
- generation-, attempt-, and epoch-safe completion admission
- in-flight registration, cancellation, supersession, retry, and timeout
  ownership
- completion handling that re-enters the runtime transactionally
- stale, contradictory, impossible, malformed, and superseded completion denial
  as typed runtime outcomes
- resource-state-aware observation and diagnostics
- branch-, restore-, rollback-, and replay-honest in-flight and completed
  resource state
- bounded in-flight tracking with named counters and proof obligations

The governing rule is:

`admit async intent once, track it once, complete it once, deny stale work explicitly`

If async/resource state remains a UI adapter state machine after this milestone,
the milestone is incomplete.

## 1. Goal

Make async and resource-backed computation a first-class capability of
`worth-signal` so that:

- resource lifecycle is canonical runtime truth
- async completion cannot commit stale or superseded work over newer intent
- retry, cancellation, timeout, and revalidation are framework-owned lifecycle
  operations
- completion-driven observation remains commit-bounded and rollback-safe
- branch, restore, replay, and diagnostics preserve one async/resource story

## 2. Why This Milestone Exists

Milestone A closed the temporal substrate. The runtime now owns clock basis,
temporal policy, scheduled wakes, previous-value access, and temporal replay
evidence.

That makes async/resource work possible without smuggling time back into host
glue.

`worth-signal` already has:

- dirty, maybe-stale, and clean node states
- transactional evaluation and rollback
- commit-bounded observation
- runtime-owned time, `StaleAfter`, and interval wake semantics
- branch and snapshot restore machinery
- diagnostics and retained causal summaries

But it does not yet own:

- pending resource state
- request identity and generation-safe in-flight state
- completion admission as a typed runtime boundary
- stale completion rejection
- cancellation, retry, supersession, timeout, and revalidation semantics
- replayable resource lifecycle history

Without this milestone:

- wasm and application layers will model resources as UI-local state
- future route-resource, form submit, and query replacement surfaces will
  invent separate lifecycle semantics
- stale completions can only be prevented by caller discipline
- async observation can punch through transaction boundaries
- branch and replay machinery will preserve values without preserving the
  lifecycle truth that produced them

## 3. Hard Part

The hard part is not spawning work.

The hard part is freezing one exact truth-preserving relationship among:

- admitted resource intent
- request identity
- generation, attempt, and branch epoch
- in-flight ownership
- timeout and freshness windows from the temporal substrate
- retry and backoff scheduling
- cancellation and supersession
- completion admission and denial
- transaction staging
- committed resource state
- observer delivery
- branch, restore, and replay reconstruction
- diagnostics and explanations of why a resource reached its current state

The design fails if:

- an older completion can overwrite newer admitted intent
- cancellation or timeout is caller-owned convention instead of runtime truth
- completion handling mutates committed derived state outside a transaction
- observers see staged async results from failed completion transactions
- replay restores a value but cannot restore the resource lifecycle story
- branch-local in-flight state leaks across branches
- broad in-flight scans hide behind cheap completion APIs
- dead, cancelled, superseded, or disposed in-flight records accumulate
  unboundedly

## 4. Explicit Assumptions

- `worth-relational` remains the owner of truth identity, mutation, history,
  diffs, and traversal.
- `worth-store` remains the owner of persistence when persistence is involved.
- `worth-signal` remains the owner of derived execution and lifecycle truth.
- async/resource nodes are derived state, not source-of-truth storage.
- hosts may perform external work and deliver completion envelopes, but they
  may not define completion legality after runtime admission.
- Milestone B is core-only; wasm, React, route-resource, form, and query
  product APIs remain out of scope.
- Milestone A temporal semantics are available and may be used for stale-after,
  timeout, retry, and backoff.
- Milestone 11 observation guarantees remain product contracts and may not be
  weakened by async work.

## 5. Governing Summaries

- `MENTALITY.md`
  The most important thing it protects here is hostile-constraint-first design.
  Async support must begin with stale-completion denial, rollback integrity,
  branch/replay parity, and bounded in-flight cost, not with ergonomic loading
  states.
- `arch_laws.md`
  The most important laws here are 16, 20, 21, 24, 27, 30, 34, 36, 37, and 41.
  Async/resource work must be declared as a managed resource, API signatures
  must reveal orchestration boundaries, lifecycle and diagnostics must stay
  separate, eligibility must precede construction, execution must consume
  lowered plans, in-flight resources must be framework-owned, checkpoint plus
  bounded journal reconstruction must hold, invalid transitions must be
  unrepresentable, and proof types must carry what has been established.
- `perf_laws.md`
  The most important thing it protects is in-flight breadth honesty.
  Completion, cancellation, retry, and cleanup may not become graph-wide or
  registry-wide scans hidden behind cheap-looking resource APIs.
- `domain_laws.md`
  The most important thing it protects is subsystem shape. Resource lifecycle,
  request identity, completion admission, retry/backoff, cancellation, and
  diagnostics need named responsibilities rather than one broad async helper.
- `worth_signals2.md`
  The most important thing it protects is the runtime thesis: observation,
  diagnostics, branch/replay, and policy-aware execution are core substrate
  responsibilities that higher-level resource abstractions must inherit.
- `worth_signal_vision.md`
  The most important thing it protects is the authority boundary:
  `worth-signal` owns derived execution semantics while remaining standalone,
  deterministic, transactional, and auditable.
- `signal_architecture2.md`
  The most important thing it protects is proof-bearing pipeline structure:
  declaration, lowering, execution, observation, diagnostics, and replay must
  remain separate enough that async lifecycle cannot collapse into ad hoc
  callback handling.
- `worth_signal_temporal_async_roadmap.md`
  The most important thing it protects is sequencing and scope. Async/resource
  follows temporal work because deadlines, freshness, retry, backoff, and
  deterministic completion admission need runtime-owned time first.
- `test-requirements.md`
  The most important thing it protects is certification. Async/resource support
  is not closed until lifecycle parity, out-of-order supersession, rollback and
  observation equivalence, branch/restore replay equivalence, and in-flight
  boundedness are all machine-checked.
- `milestone-a-closeout.md`
  The most important thing it protects is the temporal foundation now available
  to B. Async must consume the closed clock, wake, stale-after, interval,
  previous-value, and temporal diagnostic substrate rather than creating a
  second time model.
- `milestone-11-closeout.md`
  The most important thing it protects is committed observation truth.
  Completion-driven delivery must use the existing commit-bounded observation
  substrate and preserve rollback suppression, deterministic matching, and
  diagnostics-visible provenance.

## 6. Adversarial Constraint

Milestone B must survive the following hostile condition:

> A branchable, replayable runtime with deterministic execution, rollback-safe
> observation, runtime-owned time, and async/resource-backed nodes must converge
> to the same committed resource truth, deny the same stale or impossible
> completions, preserve the same in-flight lifecycle history, and emit the same
> diagnostic explanations regardless of whether work was driven by direct
> invalidation, temporal timeout, retry/backoff, cancellation, out-of-order
> completion, snapshot restore, branch replay, or ordinary synchronous
> evaluation.

Concretely, the design must remain correct when all of the following are true:

- multiple admissions happen before any completion returns
- completions arrive out of order
- retries race fresh admissions
- cancellation races completion
- timeout races success
- completion envelopes are duplicated, malformed, partial, or contradictory
- branch fork happens while work is in flight
- snapshot restore returns to a point before and after completion
- replay reconstructs from checkpoint plus bounded completion history
- diagnostics tier changes between equivalent runs

If any supported path lets stale completion commit, treats lifecycle state as
adapter-local display state, leaks observer packets from a failed completion
transaction, loses in-flight branch isolation, or widens completion handling
into graph-wide scanning under those conditions, the milestone has failed.

## 7. Product Decision Lock

- async/resource lifecycle is runtime-owned derived truth, not adapter-owned UI
  state
- hosts may execute external work, but only runtime-admitted requests can
  complete into runtime state
- request identity, generation, attempt, branch epoch, and completion ordinal
  are distinct semantic categories
- pending, fulfilled, rejected, cancelled, timed-out, stale, superseded, and
  disposed are canonical lifecycle classifications
- stale, superseded, malformed, contradictory, unknown, retired, and impossible
  completions are denied explicitly rather than ignored silently
- completion admission re-enters the runtime through a transaction boundary
- retry, cancellation, supersession, and timeout are framework-owned lifecycle
  operations
- resource-state observation uses the existing runtime observation substrate
- async/resource history must be branch-, restore-, and replay-honest
- temporal substrate ownership from Milestone A is the only accepted source for
  stale-after, timeout, retry, and backoff semantics
- async/resource support must remain core-only; higher-level resource APIs can
  consume this substrate later but may not define it

Normative consequence:

- any implementation that stores "loading" state only in wasm, UI, or adapter
  code is out of spec
- any implementation that matches completions by node id alone is out of spec
- any implementation that lets a completion update output state without
  generation and branch-epoch proof is out of spec
- any implementation that drops stale completions without retained denial
  classification is out of spec
- any implementation that makes cancellation a best-effort host callback with
  no runtime lifecycle record is out of spec
- any implementation that rebuilds in-flight state by whole-graph scan on
  restore is out of spec
- any implementation that lets diagnostics materialization decide async truth is
  out of spec

## 8. Scope

### 8.1 In Scope

- async/resource node declaration and lifecycle vocabulary
- request identity, generation, attempt, epoch, and completion ordinal types
- in-flight resource registration and ownership
- cancellation, supersession, timeout, retry, and revalidation semantics
- completion admission through a transactional runtime boundary
- stale, impossible, malformed, contradictory, retired, and superseded
  completion denial classifications
- resource-state-aware observation and diagnostics
- branch/snapshot/replay reconstruction of in-flight and completed resource
  state
- public core APIs for admitting, completing, cancelling, retrying, and
  revalidating async/resource work
- named counters and complexity contracts for async/resource work

### 8.2 Explicitly Out Of Scope

- wasm bindings
- React, Angular, or browser-store resource adapters
- route-resource APIs
- form APIs
- query replacement product facade work
- network transport implementation
- persistence beyond canonical runtime artifacts needed for replay and
  reconstruction
- domain-specific resource cache policies beyond the generic lifecycle
  substrate

## 9. Current-State Assessment

The runtime is structurally ready for this milestone in several ways:

- temporal semantics are now runtime-owned
- the transaction architecture already supports hard rollback
- the observation subsystem already stages and delivers committed change
- branch and snapshot restore already preserve runtime state categories
- diagnostics already retain bounded causal artifacts
- execution planning already distinguishes prepared work from apply/commit

The missing async/resource category is still real:

- there is no canonical resource lifecycle state machine
- no runtime-owned in-flight request registry exists
- no request identity or generation-safe completion proof exists
- retry, cancellation, timeout, and supersession are not runtime-owned
- completion handling has no typed admission/denial boundary
- replay can preserve committed node outputs without preserving the resource
  request story that produced them
- higher layers have no substrate to reduce resource/product APIs to

This means the runtime has the transaction, temporal, observation, and
diagnostic machinery async needs, but not yet the async/resource architecture
itself.

## 10. Architecture Rules For This Milestone

### 10.1 Async Is A Runtime Subsystem, Not A Callback Helper

Async/resource lifecycle must be modeled as a first-class runtime subsystem
with owned state, lifecycle, and facade access.

It must not be implemented as:

- futures stored directly on arbitrary node records
- adapter-local `loading` / `error` / `data` state
- callback-local request maps outside runtime visibility
- host-controlled completion legality

Required consequence:

- `SignalRuntime` gains an owned async/resource subsystem
- resource nodes lower into runtime-owned request descriptors
- all in-flight state is visible to branch, restore, replay, diagnostics, and
  counters

### 10.2 Async Intent And External Work Are Separate

The runtime owns admitted async intent. Hosts own external execution.

Acceptable:

- runtime admits a request and returns a host-executable request envelope
- host delivers a completion envelope with runtime-issued identity
- runtime validates completion identity, generation, branch epoch, and attempt
  before committing

Not acceptable:

- host callback decides whether a completion is still valid
- resource state is updated because a host promise resolved
- completion legality depends on current UI state, transport state, or ambient
  process memory

### 10.3 Request Identity Must Be Proof-Bearing

Completion admission requires more than node identity.

At minimum, admitted work must carry distinct proof categories for:

- resource node identity
- request identity
- resource generation
- attempt identity
- branch identity
- branch restore or lifecycle epoch
- completion ordinal
- temporal basis when timeout/backoff/stale-after is involved

Required consequence:

- a stale completion cannot present the same proof as current admitted work
- a completion from an old branch epoch cannot commit to a restored branch
- duplicate completion delivery is classified, not silently accepted

### 10.4 Lifecycle Resolution Must Precede Completion Apply

Completion handling must first resolve legality, lifecycle transition, and
observation posture, then apply through a transaction.

Required consequence:

- completion admission produces a proof-bearing result before output mutation
- execution/apply consumes admitted completion proof types
- denied completions produce typed denial artifacts without mutating committed
  resource state
- diagnostics consume retained lifecycle facts rather than re-deciding legality

### 10.5 Cancellation, Timeout, Retry, And Supersession Are Framework-Owned

These operations are not host hints. They are runtime lifecycle transitions.

Required consequence:

- cancelling a request retires or marks its completion proof so late completion
  cannot commit
- timeout uses Milestone A temporal truth, not host wall-clock callbacks
- retry creates a new attempt or generation according to explicit policy
- supersession records which admitted intent replaced which prior intent
- retry and cancellation preserve observation and diagnostic provenance

### 10.6 Completion Must Be Transactional

Async completion is an orchestration boundary. It may stage resource state,
output changes, dependency effects, observation packets, diagnostics, and retry
decisions, but it may not expose partial truth.

Required consequence:

- completion handling enters through an explicit runtime transaction or
  transaction-equivalent boundary
- rollback suppresses staged resource state and observation packets
- retry after rollback matches the no-failure control lane
- failure diagnostics record the denied or rolled-back lifecycle transition

### 10.7 Resource Observation Must Reuse The Observation Substrate

Async/resource observation must reduce to existing commit-bounded observation
semantics.

Required consequence:

- pending/fulfilled/rejected/cancelled/timed-out changes are observable
  resource-state transitions
- observer delivery remains one packet per committed boundary per observer
  where the observation policy requires that
- rollback suppresses completion-driven delivery
- easy/watch/resource layers later consume this substrate instead of defining
  their own delivery truth

### 10.8 Async State Must Be Checkpoint-Plus-Journal Reconstructable

Resource lifecycle truth must remain reconstructable from checkpoint plus
bounded async history.

Required consequence:

- in-flight state is checkpoint-visible or reconstructable from retained
  lifecycle artifacts
- completion history carries enough identity and denial evidence for replay
- branch restore does not consult host process state to recover in-flight truth
- retained history budgets produce explicit unavailable/omitted outcomes rather
  than fake reconstruction

### 10.9 Async Work Must Stay Breadth-Bounded

Completion, cancellation, retry, timeout, and cleanup must operate over the
resource-local or in-flight frontier, not total graph size.

Required consequence:

- in-flight lookup is keyed by request/generation/epoch proof, not discovered
  by scanning nodes
- completion admission cost is stated in terms of request-local lookup and
  lifecycle transition work
- cleanup and retention state their cost basis explicitly
- long-session churn exposes dead-record reclamation counters

### 10.10 Dishonest Completions Must Be First-Class Inputs

The runtime must assume completions may be hostile, malformed, partial, stale,
duplicated, or contradictory.

Required consequence:

- completion envelope validation is a named phase
- impossible status/timing claims produce typed denial outcomes
- contradictory completion reports for the same request are classified
- denied completion artifacts are retained according to diagnostics policy
- invalid completion inputs do not get normalized into generic errors

### 10.11 Performance Is A Proof Surface, Not A Diagnostics Add-On

Async/resource performance must be encoded into the same architectural objects
that carry lifecycle truth.

Required consequence:

- every public orchestration boundary returns a performance envelope
- every phase that widens work must name the semantic surface that justified the
  widening
- every broad-scan denial must be visible as a typed counter, not inferred from
  elapsed time
- every operational packet distinguishes hot-path counters from diagnostics
  richness counters
- every retained or reconstructed diagnostic read reports whether it used
  operational summaries, retained forensic detail, or cold reconstruction

The implementation is not allowed to satisfy performance requirements by adding
telemetry beside a cost-dishonest design. If completion lookup, timeout
admission, branch restore, retry scheduling, or cleanup uses broad traversal,
the API must expose that posture as debt or denial rather than report a clean
resource outcome.

### 10.12 Hot State, Cold History, And Consumer Projection Are Separate

Async/resource state has three different maintenance contracts:

- hot operational state needed to admit requests and completions
- retained lifecycle history needed for replay and diagnostics
- consumer-facing summaries needed by facade and observation surfaces

These may not share one storage shape.

Required consequence:

- hot in-flight lookup structures are optimized for request-local admission
- denial and lifecycle history retention is budgeted separately from in-flight
  matching
- facade summaries are derived from committed artifacts and do not own truth
- observer packets carry coalesced lifecycle/output deltas, not raw retained
  history
- diagnostics expansion cannot mutate hot in-flight topology

Naive trap this forbids:

- storing one `ResourceRecord` per node with current state, in-flight request,
  full history, diagnostics blobs, observer summaries, and payload together.
  That shape makes every completion pay for every view attached to the resource.

### 10.13 Bulk Completion Admission Must Be A First-Class Boundary

Real async systems receive completion floods. A scalar-only completion API
externalizes loops, fragments amortization, and hides per-completion allocation
or lookup cost.

Required consequence:

- the milestone must support a batch completion admission boundary or explicitly
  mark scalar-only completion as named debt
- batch completion admission must canonicalize completion order before apply
- batch reports must expose input width, admitted width, denied width,
  duplicate width, lifecycle transition width, observer candidate width, and
  per-boundary allocation posture
- batch completion may not reduce to N independent public scalar calls if that
  loses shared validation, sorting, deduplication, or observation coalescing
- certification must include a completion flood row that proves bulk boundary
  cardinality and counters

### 10.14 Allocation Posture Must Be Designed Before Implementation

Async pressure creates allocation traps: request handles, completion envelopes,
denial records, retry attempts, observer packets, and diagnostics records can
all allocate independently unless a lifecycle scope owns them.

Required consequence:

- in-flight records use arena/index/generation identity or an equally explicit
  lifecycle-bounded storage model
- completion staging uses transaction-local reusable buffers where practical
- denial history retention uses bounded ring, segment, or arena strategy rather
  than per-denial unbounded heap growth
- observer candidate batches reuse existing observation batching patterns
- facade report construction cannot clone full lifecycle history or payloads
- allocation counters distinguish operational allocation, retained-history
  allocation, diagnostics reconstruction allocation, and facade/report
  allocation

## 11. Required Architecture Changes

### 11.1 Add A Dedicated Async/Resource Subsystem

Add a runtime-owned async/resource subsystem under
`logic/transaction/runtime/state`.

It should own:

- resource node lifecycle state
- in-flight request registry
- request generation and attempt registries
- completion admission and denial records
- cancellation, timeout, retry, and supersession state
- branch-local async state
- async counters and diagnostics support

It must not own:

- external transport
- host data fetching implementation
- truth authority
- persistence authority
- frontend resource ergonomics

Required decomposition target:

```text
data/
  resource/
    declaration.rs
    descriptor.rs
    lifecycle.rs
    request.rs
    completion.rs
    denial.rs
    policy.rs
    proof.rs
    summary.rs
logic/
  transaction/
    runtime/
      state/
        resource/
          registry.rs
          inflight.rs
          lifecycle.rs
          branch_state.rs
          retention.rs
          counters.rs
      transaction/
        transaction_resource.rs
diagnostics/
  model/
    resource.rs
```

The exact paths may adapt to the crate's current layout, but the
responsibilities may not collapse into one `async.rs`, `resource.rs`, or
`helpers.rs` module.

Each file must have one reason to change:

- declaration and descriptor lowering change when API contracts change
- lifecycle changes when legal state transitions change
- request identity changes when admission proof changes
- completion validation changes when hostile envelope handling changes
- denial changes when rejection topology changes
- registry/in-flight storage changes when lookup topology changes
- branch state changes when restore/replay integration changes
- retention changes when lifecycle history budgeting changes
- diagnostics changes when explanation richness changes

### 11.2 Introduce Async Policy And Proof Types

Add explicit types for the async/resource pipeline, with exact names allowed to
evolve.

At minimum the architecture should preserve distinct forms for:

- async/resource node declaration
- resource lifecycle classification
- resource request intent
- admitted resource request
- in-flight resource handle
- resource generation
- resource attempt
- resource branch epoch
- completion envelope
- validated completion envelope
- admitted completion
- denied completion
- cancelled request
- timed-out request
- superseded request
- retry admission
- committed resource state
- async observation summary
- async reconstructability artifact
- async diagnostics explanation artifact

The milestone should prefer semantic newtypes wherever raw primitives would
blur meaning. This includes, at minimum:

- `ResourceNodeId`
- `ResourceRequestId`
- `ResourceGeneration`
- `ResourceAttemptId`
- `ResourceBranchEpoch`
- `ResourceCompletionOrdinal`
- `ResourceLifecycleOrdinal`
- `CancellationOrdinal`
- `RetryOrdinal`
- `AsyncCheckpointId`
- `AsyncDenialId`

### 11.2.1 Required Typestate Pipeline Shape

The implementation must make the async/resource proof chain visible in type
signatures. A compliant implementation should be recognizable even before tests
run because raw inputs cannot skip the pipeline.

The target shape is:

```rust
RawCompletionEnvelope
    -> ValidatedCompletionEnvelope
    -> CompletionIdentityMatch
    -> CompletionLifecycleEligibility
    -> AdmittedResourceCompletion | DeniedResourceCompletion
    -> StagedResourceCompletionEffect
    -> CommittedResourceCompletionArtifact
```

The exact names may evolve, but the proof progression may not collapse.

Required construction rules:

- `RawCompletionEnvelope` may be public as an input carrier, but it has no
  authority.
- `ValidatedCompletionEnvelope` is constructed only by envelope validation.
- `CompletionIdentityMatch` is constructed only by request/generation/attempt/
  epoch matching.
- `CompletionLifecycleEligibility` is constructed only after lifecycle-state
  legality is proven.
- `AdmittedResourceCompletion` and `DeniedResourceCompletion` are mutually
  exclusive terminal admission outcomes.
- `StagedResourceCompletionEffect` is constructed only by the completion
  transaction staging lane.
- `CommittedResourceCompletionArtifact` is constructed only by commit or replay
  integration.

Compile-time consequence:

- a raw completion cannot call apply
- a validated-but-unmatched completion cannot call lifecycle transition
- a denied completion cannot call committed resource mutation
- an admitted completion cannot bypass transaction staging
- a staged completion cannot be observed until commit produces the committed
  artifact

### 11.2.2 Required Lifecycle Transition Table

Milestone B must freeze a lifecycle transition table before implementation
starts. This table is not documentation only; it determines typestate surfaces,
constructor visibility, and compile-fail fixtures.

Minimum legal transitions:

| From | Event | To | Required proof |
| --- | --- | --- | --- |
| `Unrequested` | request admitted | `Pending` | lowered descriptor, request generation, branch epoch |
| `Pending` | completion admitted as success | `Fulfilled` | identity match, lifecycle eligibility, transaction commit |
| `Pending` | completion admitted as failure | `Rejected` | identity match, lifecycle eligibility, transaction commit |
| `Pending` | cancel admitted | `Cancelled` | request ownership, cancellation ordinal |
| `Pending` | timeout wake admitted | `TimedOut` | temporal wake proof, timeout policy identity |
| `Pending` | newer generation admitted | `Superseded` | supersession proof linking old and new intent |
| `Rejected` | retry admitted | `Pending` | retry policy identity, attempt lineage |
| `TimedOut` | retry admitted | `Pending` | retry policy identity, temporal basis |
| `Fulfilled` | revalidation admitted | `Pending` | revalidation policy identity, new generation |
| `Rejected` | revalidation admitted | `Pending` | revalidation policy identity, new generation |
| `Cancelled` | revalidation admitted | `Pending` | explicit new intent, new generation |
| any terminal state | disposal admitted | `Disposed` | owner lifecycle proof |

Minimum illegal transitions:

- `Fulfilled -> Fulfilled` by duplicate completion
- `Cancelled -> Fulfilled` by late success
- `TimedOut -> Fulfilled` by late success unless an explicit policy classifies
  it as admissible revalidation input rather than original completion
- `Superseded -> Fulfilled` by old completion
- `Disposed -> Pending` without a new resource-node declaration
- any state -> committed mutation from a denied completion

The implementation may add lifecycle states only if it updates this table,
transition proof types, denial classifications, certification rows, and
compile-fail fixtures together.

### 11.3 Lower Resource Declarations Into Runtime Forms

Resource declarations must not remain open-ended host callbacks.

Required consequence:

- registration intent lowers into runtime-owned resource descriptors
- lifecycle policy, timeout policy, retry policy, cancellation policy, and
  stale-after policy are frozen before execution
- completion admission consumes lowered descriptors
- branch/replay artifacts record descriptor identity and version

### 11.3.1 Concrete Declaration Shape

Milestone B should introduce a single declaration object rather than scattered
registration calls.

The target declaration categories are:

```rust
pub struct ResourceNodeDeclaration {
    pub node: ResourceNodeId,
    pub lifecycle_policy: ResourceLifecyclePolicyDeclaration,
    pub retry_policy: ResourceRetryPolicyDeclaration,
    pub timeout_policy: ResourceTimeoutPolicyDeclaration,
    pub cancellation_policy: ResourceCancellationPolicyDeclaration,
    pub stale_after_policy: ResourceStaleAfterPolicyDeclaration,
    pub supersession_policy: ResourceSupersessionPolicyDeclaration,
    pub observation_policy: ResourceObservationPolicyDeclaration,
    pub payload_contract: ResourcePayloadContract,
}
```

The exact field names may evolve, but the architecture may not split these
categories into caller-managed side registries.

Required lowering output:

```rust
pub struct LoweredResourceDescriptor {
    pub descriptor_id: ResourceDescriptorId,
    pub descriptor_version: ResourceDescriptorVersion,
    pub lifecycle_policy: LoweredResourceLifecyclePolicy,
    pub retry_policy: LoweredResourceRetryPolicy,
    pub timeout_policy: LoweredResourceTimeoutPolicy,
    pub cancellation_policy: LoweredResourceCancellationPolicy,
    pub stale_after_policy: LoweredResourceStaleAfterPolicy,
    pub supersession_policy: LoweredResourceSupersessionPolicy,
    pub observation_policy: LoweredResourceObservationPolicy,
    pub payload_contract_digest: ResourcePayloadContractDigest,
}
```

The descriptor digest must appear in request admission, completion admission,
replay artifacts, and diagnostics so policy drift is machine-detectable.

### 11.3.2 Payload Contract Boundary

Milestone B does not interpret domain payloads, but it must still define how
payload identity and payload integrity participate in resource truth.

Required consequence:

- payload bytes or domain values are not themselves lifecycle authority
- completion envelopes carry a payload contract digest or equivalent integrity
  marker
- malformed, partial, missing, oversized, or contract-mismatched payloads deny
  completion before lifecycle apply
- payload equality or reuse cannot authorize resource lifecycle transitions
- later domain-specific cache layers may add payload semantics, but only above
  the generic runtime lifecycle substrate

### 11.3.3 Resource Lifecycle State And Node Output Are Separate

Resource lifecycle state is not the same object as the node's computed output.

Required consequence:

- a node may have resource lifecycle state even when its last successful output
  remains unchanged
- `Pending` does not erase the last fulfilled output unless an explicit policy
  says the visible derived value is unavailable
- `Rejected`, `TimedOut`, and `Cancelled` do not automatically replace the
  last successful output with an error payload
- observation can distinguish lifecycle change from meaningful output change
- comparator/output suppression cannot suppress lifecycle truth
- lifecycle transitions and output transitions have separate digests in
  certification artifacts

Naive trap this forbids:

- modeling a resource node as one enum like `Loading | Data(T) | Error(E)`.
  That shape collapses lifecycle, payload, output continuity, observation
  classification, and retry eligibility into one mutable value and is not
  accepted for the core runtime substrate.

### 11.4 Stage Async Facts In Transactions

`TransactionScratch` needs an async/resource lane rather than forcing commit,
observation, or diagnostics to rediscover what completion did.

Expected additions include:

- staged in-flight admissions
- staged completion admission or denial facts
- staged cancellation, timeout, supersession, and retry facts
- staged committed resource states
- staged observation classifications
- async counter deltas

The staged forms must be phase-distinct and proof-bearing.

The compile-time bias here should be explicit:

- raw completion envelopes do not cross phase boundaries without validation
- only admission can construct admitted completion proof
- only denial classification can construct denied completion proof
- only commit/replay integration can construct committed async explanation
  artifacts

### 11.4.1 Transaction Lane Shape

The async transaction lane must be concrete enough that completion handling
cannot quietly mutate state in helper code.

Expected transaction scratch categories:

```rust
pub struct TransactionAsyncResourceScratch {
    pub admitted_requests: AdmittedResourceRequestBatch,
    pub admitted_completions: AdmittedResourceCompletionBatch,
    pub denied_completions: DeniedResourceCompletionBatch,
    pub lifecycle_transitions: ResourceLifecycleTransitionBatch,
    pub cancellation_transitions: ResourceCancellationBatch,
    pub timeout_transitions: ResourceTimeoutBatch,
    pub supersession_transitions: ResourceSupersessionBatch,
    pub retry_admissions: ResourceRetryAdmissionBatch,
    pub revalidation_admissions: ResourceRevalidationAdmissionBatch,
    pub observation_candidates: ResourceObservationCandidateBatch,
    pub counter_delta: AsyncResourceCounterDelta,
}
```

These batches should be move-only unless a second observer is explicitly
justified. Diagnostics should consume committed summaries or retained copies,
not force operational packets to become broadly cloneable.

### 11.5 Add Branch / Replay / Diagnostics Async Artifacts

Async work must flow into the same trust surfaces as the rest of the runtime.

Required consequence:

- diagnostics can explain why a resource is pending, fulfilled, rejected,
  cancelled, timed out, stale, or superseded
- replay can compare canonical async lifecycle digests
- branch and restore paths preserve in-flight and committed resource state
- public facade exposes capability-shaped requests and summaries, not raw
  subsystem internals

### 11.6 Required Storage And Index Topology

The async/resource subsystem must choose storage around real traversal patterns.

Minimum indexed views:

- by `ResourceRequestId` for completion lookup
- by `(ResourceNodeId, ResourceGeneration)` for supersession and node-owned
  cleanup
- by `(SignalBranchId, ResourceBranchEpoch)` for branch-local restore and
  replay
- by temporal wake identity for timeout and retry/backoff admission
- by lifecycle terminality for retention and reclamation
- by denied completion id for diagnostics and replay parity

Minimum prohibited storage shapes:

- one global vector of in-flight records searched by completion
- one node-local optional request slot that cannot represent overlapping
  admissions, retries, and stale completions
- one branch-global blob that must be fully scanned on restore
- storing host future handles as runtime truth
- storing only the latest resource value without lifecycle history

Required storage posture:

- hot lookup structures should be arena-backed or index-backed with generation
  checks
- branch snapshots should share structure or carry retained async summaries
  rather than clone all in-flight records by default
- terminal lifecycle cleanup must be explicit and counted
- retaining denial history for diagnostics must be budgeted separately from
  in-flight operational lookup

### 11.7 Hot/Cold Resource Data Split

The runtime must make hot operational data physically distinct from cold or
consumer-facing data.

Minimum target forms:

```rust
pub struct HotResourceInflightState {
    pub request_index: ResourceRequestIndex,
    pub node_generation_index: ResourceNodeGenerationIndex,
    pub branch_epoch_index: ResourceBranchEpochIndex,
    pub temporal_wake_index: ResourceTemporalWakeIndex,
    pub lifecycle_terminal_index: ResourceLifecycleTerminalIndex,
}

pub struct RetainedResourceLifecycleHistory {
    pub retained_transitions: RetainedLifecycleTransitionStore,
    pub retained_denials: RetainedCompletionDenialStore,
    pub retained_replay_summaries: RetainedResourceReplaySummaryStore,
}

pub struct ResourceConsumerProjection {
    pub latest_lifecycle_summary: ResourceLifecycleSummaryStore,
    pub latest_observation_summary: ResourceObservationSummaryStore,
}
```

The exact names may evolve, but the maintenance split may not collapse.

Required consequence:

- completion admission mutates hot in-flight state and committed lifecycle
  artifacts, not retained diagnostics history directly
- retention policy copies or derives from committed artifacts after the
  operational boundary is known
- facade summaries derive from committed artifacts or retained summaries
- diagnostics reconstruction cannot trigger in-flight lookup repair
- hot-state compaction and cold-history retention have separate counters

### 11.8 Boundary Performance Envelopes

Every public async/resource orchestration boundary must return a named
performance envelope, either directly or inside its report.

Minimum target form:

```rust
pub struct ResourceBoundaryPerformanceEnvelope {
    pub boundary: ResourceBoundaryKind,
    pub input_width: u32,
    pub request_lookup_count: u32,
    pub node_generation_lookup_count: u32,
    pub branch_epoch_lookup_count: u32,
    pub temporal_wake_lookup_count: u32,
    pub lifecycle_transition_count: u32,
    pub admitted_count: u32,
    pub denied_count: u32,
    pub observer_candidate_count: u32,
    pub retained_history_write_count: u32,
    pub operational_allocation_count: u32,
    pub retained_history_allocation_count: u32,
    pub diagnostics_allocation_count: u32,
    pub facade_report_allocation_count: u32,
    pub broad_scan_denial_count: u32,
    pub cost_contract: ResourceCostContractId,
    pub cost_posture: ResourceCostPosture,
}
```

Required consequence:

- reports cannot say "fulfilled" or "cancelled" without naming the work surface
  used to reach that result
- if an operation falls back to a debt path, `cost_posture` must say so
- ordinary summary reads expose zero reconstruction work when they are retained
  summaries
- cold reconstruction is never hidden inside `resource_summary`

### 11.9 Resource Cost Contract Registry

Milestone B must introduce a small registry of named async/resource cost
contracts. A counter without a named contract is not enough.

Minimum contracts:

| Contract | Expected cost basis |
| --- | --- |
| `resource_declaration_lowering` | declared policy count + descriptor width |
| `resource_request_admission` | one node/generation lookup + policy lowering summary |
| `resource_completion_validation` | completion input width + payload integrity surface |
| `resource_completion_admission` | request lookup + generation/attempt/epoch proof checks |
| `resource_cancellation` | request-local footprint + owner index update |
| `resource_timeout_admission` | due temporal wake width + affected request count |
| `resource_retry_backoff_admission` | retry decision width + temporal wake footprint |
| `resource_revalidation_admission` | resource node footprint + new generation admission |
| `resource_branch_restore` | restored branch-local async state + retained summaries |
| `resource_replay_reconstruction` | checkpoint span + retained completion history |
| `resource_retention_compaction` | terminal lifecycle records selected by policy |
| `resource_summary_read` | retained summary lookup only |
| `resource_diagnostics_expansion` | retained detail width or explicit reconstruction budget |

Each contract must carry status:

- `Verified`
- `Debt`
- `DeniedFallback`

No public report may omit the contract id and status for the boundary it
represents.

### 11.10 Density And Pressure Strategy

Async/resource traffic can be sparse, dense, bursty, or pathological. The
runtime must not pretend one strategy is optimal for all densities.

Required consequence:

- request admission and completion admission reports include current in-flight
  density posture
- batch completion admission can switch between sparse indexed lookup and
  dense sorted/deduplicated admission only through named strategy selection
- strategy selection is derived from runtime state and reported in the
  performance envelope
- dense fallback cannot change lifecycle truth or denial classification
- certification includes sparse, dense, and bursty pressure rows

## 12. Milestone Phases

### Phase 1: Async Contract Freeze

Deliver:

- the core type vocabulary for resource lifecycle truth
- the async/resource subsystem boundary
- the product-decision lock encoded in docs and public naming
- the high-level phase model for async/resource runtime progression

Must prove:

- resource lifecycle is no longer spec-shaped as adapter-local UI state
- request identity and lifecycle classification are distinct categories
- the milestone sequence is explicit enough that implementation cannot collapse
  validation, admission, execution, observation, and diagnostics into one
  callback path

### Phase 2: Resource Lifecycle Vocabulary And State Machine

Deliver:

- canonical lifecycle classifications for pending, fulfilled, rejected,
  cancelled, timed-out, stale, superseded, disposed, and retained-history-
  unavailable states
- typed transition rules between lifecycle states
- lifecycle transition ordinals and branch-local lifecycle identity

Must prove:

- impossible lifecycle transitions are unrepresentable or denied before apply
- terminal states cannot be completed again without explicit retry/revalidation
- lifecycle vocabulary distinguishes runtime truth from display status

### Phase 3: Request Identity, Generation, Attempt, And Epoch Proofs

Deliver:

- request identity and generation types
- attempt identity and retry lineage
- branch epoch and restore epoch proof
- completion ordinal and duplicate detection basis

Must prove:

- node id alone cannot admit a completion
- stale generations and stale branch epochs are rejected
- duplicate and contradictory completions are classifiable

### Phase 4: In-Flight Registry And Resource Frontier Indexing

Deliver:

- in-flight request registry
- request-local lookup structures
- owner indexes for resource nodes
- lifecycle cleanup and retention hooks
- counters for in-flight frontier width and broad-scan denial

Must prove:

- completion matching does not scan the graph
- cancellation, timeout, and retry operate over owned in-flight records
- dead or retired records have a framework-owned reclamation path
- hot in-flight lookup, retained lifecycle history, and consumer projection are
  physically distinct maintenance surfaces

### Phase 5: Resource Declaration Lowering And Policy Freezing

Deliver:

- resource node declaration forms
- lowered resource descriptors
- lifecycle, retry, timeout, cancellation, stale-after, and supersession policy
  identity
- descriptor versioning for replay and diagnostics

Must prove:

- completion legality consumes lowered runtime descriptors rather than host
  callbacks
- temporal policy is consumed from Milestone A forms
- descriptor drift is detectable during replay or restore
- descriptor lowering emits cost-contract identity for later boundaries

### Phase 6: Completion Envelope Validation And Admission

Deliver:

- raw completion envelope boundary
- validated completion envelope
- admitted completion proof
- typed denied completion classifications
- completion integrity checks for malformed, unknown, partial, contradictory,
  stale, superseded, retired, and impossible completions

Must prove:

- invalid completion inputs cannot mutate committed resource state
- stale and superseded completions are explicitly denied
- denial classification is stable across replay
- scalar and batch completion admission both report the same lifecycle truth
  while exposing different width and strategy counters

### Phase 7: Cancellation, Timeout, Supersession, Retry, And Revalidation

Deliver:

- cancellation lifecycle transitions
- timeout transitions driven by Milestone A temporal truth
- supersession records tying old intent to replacing intent
- retry and backoff scheduling
- revalidation admission semantics

Must prove:

- late completion after cancellation cannot commit
- success after timeout is denied or classified according to explicit policy
- retry lineage is preserved and replayable
- revalidation cannot silently overwrite a newer generation
- retry/backoff and timeout handling report temporal wake footprint and retry
  decision width rather than elapsed-time folklore

### Phase 7.5: Resource Policy Extensibility Freeze

Deliver:

- frozen resource policy descriptor vocabulary for retry, timeout,
  cancellation, supersession, revalidation, observation, output continuity, and
  retention
- policy ids, semantic names, versions, digests, and selection-basis records
  equivalent in rigor to merge and commit strategy descriptors
- built-in policy registrations matching the policies already used by the
  substrate, so current behavior lowers through the same registry path future
  policies will use
- declaration-time policy resolution that rejects unknown policy names before
  request admission, temporal wake allocation, completion admission, or
  transaction staging
- lowered resource descriptors that record resolved policy identity, parameter
  digest, compatibility posture, and cost-contract identity
- compile-time and runtime proof that policy execution consumes frozen,
  deterministic descriptors rather than arbitrary host callbacks

Must prove:

- lifecycle legality remains hard-coded runtime law while variable behavior is
  encoded as deterministic policy descriptors
- retry, timeout, cancellation, supersession, revalidation, observation, output
  continuity, and retention policy identity all participate in descriptor
  digests
- changing a policy parameter changes the lowered descriptor digest and replay
  compatibility evidence
- unknown, duplicate, or incompatible policy registrations are denied before
  execution work is constructed
- policy selection is visible in boundary reports and diagnostics, not inferred
  from hidden runtime branches
- later policy families can be added without changing request identity,
  completion identity, lifecycle proof types, or transaction apply semantics

### Phase 8: Transactional Completion Apply And Observation

Deliver:

- transaction staging for admitted and denied completions
- committed resource-state mutation path
- rollback handling for completion transactions
- observation classification for resource lifecycle changes
- observer delivery through existing commit-bounded observation machinery

Must prove:

- completion-driven mutation is rollback-safe
- no observer packet escapes from a failed completion transaction
- retry after rollback matches no-failure control
- resource observation does not become a second delivery engine
- operational transaction packets remain move-only unless a second observer is
  structurally justified

### Phase 9: Branch Restore, Snapshot, And Replay Integration

Deliver:

- branch-local in-flight state
- async checkpoint and restore artifacts
- completion stream replay artifacts
- lifecycle and denial history reconstruction
- retained-history unavailable/omitted outcomes

Must prove:

- branch-local in-flight state stays isolated
- restore reconstructs the same resource lifecycle story as the original
  history
- equivalent replay produces identical lifecycle, denial, observation, and
  explanation digests
- restoration does not rebuild async state through whole-graph scans
- branch restore reports restored async state width, retained-summary width,
  and denied broad-rebuild count

### Phase 10: Diagnostics, Facade, And Certification Surface

Deliver:

- diagnostics-visible async/resource provenance
- public core facade for admitting, completing, cancelling, retrying, and
  revalidating async/resource work
- counters and complexity-contract documentation for async/resource work
- cost-honest resource summaries
- certification bundle types for required async test families

Must prove:

- diagnostics richness does not alter async/resource truth
- public surfaces reveal orchestration and cost boundaries
- certification can prove lifecycle parity, supersession, rollback,
  branch/replay parity, and in-flight boundedness
- every public report contains a `ResourceBoundaryPerformanceEnvelope` or
  direct equivalent

### Phase 11: Performance Contract Certification

Deliver:

- `ResourceCostContractId` and `ResourceCostPosture`
- boundary performance envelopes wired into public reports
- exact counter assertions for scalar and batch completion admission
- exact counter assertions for timeout, retry, cancellation, branch restore,
  retained summary read, diagnostics expansion, and retention compaction
- sparse, dense, and bursty async pressure certification rows
- allocation posture counters separated by operational, retained-history,
  diagnostics, and facade/report allocation lanes

Must prove:

- completion matching scales with request-local or batch-local surface, not
  graph size
- batch completion admission amortizes validation, sorting, deduplication, and
  observation coalescing honestly
- retained summary reads perform zero cold reconstruction
- diagnostics expansion cost is explicit and budget-gated
- hot operational allocation does not grow with retained diagnostics richness
- density strategy selection is reported and truth-preserving

### Phase 12: Compile-Time Boundary And Fixture Hardening

Deliver:

- compile-fail fixtures for private proof fields
- compile-fail fixtures for out-of-order lifecycle transitions
- compile-fail fixtures proving raw completions cannot call apply
- compile-fail fixtures proving denied completions cannot mutate resource state
- compile-fail fixtures proving facade consumers cannot construct admitted
  request, admitted completion, or committed artifact proofs directly
- compile-fail fixtures proving async subsystem internals are not reachable
  outside the facade

Must prove:

- critical async/resource rules fail at compile time where Rust can enforce
  them
- public facade surfaces expose capability-shaped handles and summaries without
  exposing internal constructors
- the milestone does not rely on comments or runtime assertions for lifecycle
  ordering that type signatures can enforce

### 12.1 Phase Ordering Rationale

The ordering is intentionally strict.

- `Phase 1` freezes the boundary so implementation cannot slide into a nicer
  promise wrapper.
- `Phase 2` comes before request identity because the runtime must know which
  lifecycle states are legal before it can prove transitions between them.
- `Phase 3` defines the proof identity needed to reject stale completions.
- `Phase 4` builds the bounded in-flight substrate after identity is frozen.
- `Phase 5` lowers declarations and policies only after lifecycle and in-flight
  storage shape are clear.
- `Phase 6` admits completions only after descriptors, identity, and storage
  can carry the proof chain.
- `Phase 7` lands cancellation, timeout, retry, supersession, and revalidation
  after ordinary completion admission exists, because these are lifecycle
  extensions over the same request identity substrate.
- `Phase 7.5` freezes policy extensibility before transactional apply depends
  on any one retry, timeout, observation, or retention behavior as if it were
  universal runtime law.
- `Phase 8` integrates completion with transactions and observation only after
  lifecycle legality is proof-bearing.
- `Phase 9` integrates branch/restore/replay after the runtime has real async
  artifacts worth restoring and replaying.
- `Phase 10` exposes facade, diagnostics, and certification only after runtime
  truth is frozen, so the public surface cannot define missing semantics by
  accident.
- `Phase 11` comes after facade exposure because performance envelopes must
  certify the real public boundary shape, not an internal sketch.
- `Phase 12` closes the boundary after facade and performance exposure so
  compile-fail fixtures can target the real public surface instead of an
  internal prototype.

If any future edit tries to merge non-adjacent phases, it must prove that no
real structural dependency is being hidden by that compression.

## 12.2 Public Facade Target Shape

The public API must make orchestration boundaries physically visible. Cheap-
looking getters may inspect retained summaries; they may not drive completion,
retry, cancellation, restore, or reconstruction work.

The target public surface should resemble:

```rust
impl SignalRuntime {
    pub fn declare_resource_node(
        &mut self,
        declaration: ResourceNodeDeclaration,
    ) -> Result<ResourceNodeHandle, ResourceDeclarationError>;

    pub fn admit_resource_request(
        &mut self,
        handle: ResourceNodeHandle,
        intent: ResourceRequestIntent,
    ) -> Result<ResourceAdmissionReport, ResourceAdmissionError>;

    pub fn complete_resource_request(
        &mut self,
        completion: RawCompletionEnvelope,
    ) -> Result<ResourceCompletionReport, ResourceCompletionError>;

    pub fn complete_resource_requests(
        &mut self,
        completions: ResourceCompletionBatch,
    ) -> Result<ResourceBatchCompletionReport, ResourceCompletionError>;

    pub fn cancel_resource_request(
        &mut self,
        request: ResourceRequestHandle,
        reason: ResourceCancellationReason,
    ) -> Result<ResourceCancellationReport, ResourceCancellationError>;

    pub fn retry_resource_request(
        &mut self,
        request: RetryEligibleResourceRequest,
        reason: ResourceRetryReason,
    ) -> Result<ResourceRetryAdmissionReport, ResourceRetryError>;

    pub fn revalidate_resource_node(
        &mut self,
        handle: ResourceNodeHandle,
        reason: ResourceRevalidationReason,
    ) -> Result<ResourceAdmissionReport, ResourceRevalidationError>;

    pub fn resource_summary(
        &self,
        handle: ResourceNodeHandle,
    ) -> ResourceLifecycleSummary;
}
```

The exact names may evolve, but the public shape must preserve these facts:

- declaration is separate from request admission
- request admission is separate from external execution
- completion is an explicit boundary accepting raw untrusted input
- scalar and batch completion boundaries are distinct and both cost-honest
- cancellation, retry, and revalidation are explicit orchestration calls
- retained summaries are read-only and cannot cause hidden reconstruction
- handles are capability-shaped and cannot be WORTHd from raw ids outside the
  facade

Required report contents:

- primary lifecycle outcome
- structured warnings
- denial classification where applicable
- request/generation/attempt/branch identity digest
- observation boundary summary
- performance counters for the boundary
- cost contract id and cost posture
- density strategy where relevant
- allocation posture split by operational, retained-history, diagnostics, and
  facade/report lanes
- diagnostics availability posture

If a facade call cannot return this information without a later diagnostics
query, the boundary envelope is incomplete.

## 12.3 Practical Scenario Rows

Implementation should include small, concrete scenario rows before the full
hostile certification suite so engineers can develop against tangible behavior
instead of only abstract parity goals.

Minimum scenario rows:

- `single_request_success_commits_fulfilled_resource`
  Admit one request, complete it successfully, and prove pending then fulfilled
  lifecycle transition, one committed observation boundary, and zero denial.
- `late_success_after_cancel_is_denied`
  Admit a request, cancel it, deliver success afterward, and prove committed
  state remains cancelled with a typed late-completion denial.
- `out_of_order_r2_before_r1_commits_only_current_generation`
  Admit R1, supersede with R2, complete R2 then R1, and prove R1 denial is
  stable and observable only as denial provenance.
- `timeout_then_retry_preserves_attempt_lineage`
  Admit a request with timeout policy, advance runtime time to timeout, retry,
  complete retry, and prove attempt lineage plus temporal basis digest.
- `completion_failure_rolls_back_observation`
  Inject a failure after staging completion observation but before commit and
  prove no observer delivery escapes.
- `branch_restore_rejects_pre_restore_completion`
  Admit on a branch, restore before admission, deliver the old completion, and
  prove branch epoch denial.
- `malformed_completion_has_no_payload_side_effect`
  Submit partial or contract-mismatched payload and prove no resource state
  mutation plus typed malformed denial.
- `disposed_resource_cannot_be_reanimated_by_completion`
  Dispose resource ownership, deliver a late completion, and prove denied
  completion with no new pending or fulfilled state.
- `batch_completion_deduplicates_before_apply`
  Submit a batch containing duplicate, stale, valid, and malformed completions;
  prove canonical admission order, one lifecycle transition per admitted
  request, stable denial classifications, and exact batch width counters.
- `retained_summary_read_does_zero_reconstruction`
  Read a resource summary after lifecycle churn and prove the summary reports
  retained availability with zero cold reconstruction counters.
- `diagnostics_expansion_is_budgeted_cold_work`
  Request richer lifecycle explanation under a reconstruction budget and prove
  cold-work counters are attributed to diagnostics rather than completion
  admission.
- `dense_completion_flood_reports_strategy`
  Submit a large completion flood with sparse valid winners and prove strategy
  selection, deduplication width, observer coalescing width, allocation counts,
  and identical lifecycle truth to scalar replay.

These rows are not substitutes for certification. They are development rails
that prevent the milestone from staying too abstract during implementation.

## 12.4 Required Compile-Fail Fixtures

The implementation must add compile-fail fixtures for API and proof boundaries,
not only runtime tests.

Minimum fixtures:

- external code cannot construct `AdmittedResourceRequest`
- external code cannot construct `AdmittedResourceCompletion`
- external code cannot construct `DeniedResourceCompletion`
- external code cannot construct `CommittedResourceCompletionArtifact`
- external code cannot mutate private lifecycle state fields
- raw `ResourceRequestId` cannot be used where `ResourceRequestHandle` is
  required
- raw `RawCompletionEnvelope` cannot call completion apply
- `ValidatedCompletionEnvelope` cannot skip identity matching
- `DeniedResourceCompletion` cannot call committed mutation
- `DisposedResourceHandle` cannot admit retry or completion
- stale branch epoch capability cannot be reused after restore
- async subsystem modules are not publicly reachable except through facade

Each fixture should fail for the intended privacy or type-boundary reason, not
because of unresolved imports.

## 13. Must Ship

Milestone B is not done because a node can return an async value.

It is done only when `worth-signal` ships:

- a runtime-owned async/resource subsystem
- explicit resource lifecycle vocabulary
- proof-bearing request identity, generation, attempt, branch epoch, and
  completion ordinal types
- in-flight request ownership and bounded lookup structures
- lowered resource declarations and frozen lifecycle policy identity
- transactional completion admission
- typed denial for stale, superseded, malformed, partial, contradictory,
  unknown, retired, cancelled, timed-out, and impossible completions
- runtime-owned cancellation, timeout, supersession, retry, and revalidation
  semantics
- frozen extensibility registries for resource retry, timeout, cancellation,
  supersession, revalidation, observation, output continuity, and retention
  policies
- lowered resource descriptors that carry policy id, semantic name, version,
  digest, selection basis, compatibility posture, and cost-contract identity
- branch/restore/replay-aware in-flight and completed resource state
- diagnostics-visible async/resource provenance
- public core APIs for admitting, completing, cancelling, retrying, and
  revalidating async/resource work
- named counters and complexity contracts for async/resource work
- boundary performance envelopes on every public async/resource report
- hot/cold/projection storage separation for resource state
- scalar and batch completion admission with cost-honest reports, or explicit
  named debt if batch completion is intentionally deferred
- compile-fail fixtures for proof constructors, lifecycle ordering, facade
  boundaries, and stale branch/restore capabilities

### 13.1 Required Named Test Families

- `async_resource_lifecycle_parity`
- `out_of_order_completion_supersession`
- `async_rollback_observation_equivalence`
- `async_branch_restore_replay_equivalence`
- `async_inflight_boundedness`

These families are the owning implementation lanes for the corresponding
async/resource substrate requirements declared in
[`test-requirements.md`](./test-requirements.md), especially:

- `15. The async resource lifecycle parity test`
- `16. The out-of-order completion supersession test`
- `17. The async rollback and observation equivalence test`
- `18. The async branch restore and replay equivalence test`
- `19. The async inflight boundedness test`
- `19A. The worst async nightmare grammar`
- `19B. The regulated-system adversarial rule`

### 13.2 Hostile Conditions Required In Certification

- multiple admissions before completion
- out-of-order completion
- duplicate completion delivery
- success after timeout
- failure after supersession
- cancellation racing completion
- retry racing fresh admission
- broken or delayed completion delivery
- contradictory completion reports for the same request
- partial payload delivery
- impossible status or timing claims
- completion with missing or corrupted request identity
- completion for unknown, retired, cancelled, or superseded request
- completion that lies about generation or attempt identity
- lost completion and ghost in-flight state
- long-session acquire, supersede, cancel, retry, dispose churn
- branch fork with in-flight work
- snapshot restore before and after completion
- diagnostics-tier variation across equivalent runs

## 14. Must Preserve

- deterministic execution remains a product contract
- commit-bounded observation remains unchanged
- rollback remains hard rewind rather than best-effort cleanup
- authority stays outside `worth-signal`
- async/resource nodes remain derived state, not truth storage
- temporal meaning remains owned by the Milestone A temporal substrate
- one canonical async/resource truth artifact remains the source for
  diagnostics, replay, and observation-derived views
- hosts may execute work and submit completions but may not define completion
  legality after runtime admission
- diagnostics richness may vary by policy, but async/resource truth may not

## 15. Performance Contracts

The milestone must expose named counters for at least:

- in-flight request count
- fulfilled count
- rejected count
- cancelled count
- timed-out count
- stale completion denial count
- superseded completion denial count
- malformed completion denial count
- contradictory completion denial count
- duplicate completion denial count
- unknown request denial count
- retry admission count
- revalidation admission count
- async branch restore count
- async replay parity check count
- in-flight broad-scan denial count
- in-flight retired record count
- in-flight reclaimed record count
- completion transaction rollback count
- completion observation suppression count
- batch completion input width
- batch completion deduplicated width
- batch completion admitted width
- batch completion denied width
- observer candidate width
- observer coalesced width
- hot in-flight lookup count
- hot in-flight compaction count
- retained lifecycle history write count
- retained lifecycle history pruned count
- retained summary read count
- cold reconstruction request count
- cold reconstruction denied count
- operational allocation count
- retained-history allocation count
- diagnostics allocation count
- facade/report allocation count
- density strategy selection count

The milestone must also declare named complexity contracts for:

- resource declaration lowering
- in-flight registration
- completion envelope validation
- completion admission
- stale-completion rejection
- cancellation
- timeout admission
- retry/backoff scheduling
- revalidation admission
- in-flight cleanup/reclamation
- branch restore of async state
- async replay reconstruction
- diagnostics-time async explanation expansion
- batch completion admission
- retained summary read
- lifecycle history retention compaction
- density strategy selection

Each contract must name its real cost bases explicitly. At minimum:

- completion matching cost must be stated in terms of request-local lookup and
  generation/epoch proof checks, not total graph size
- cancellation cost must be stated in terms of the request or owner in-flight
  footprint, not total active resources
- retry/backoff scheduling cost must be stated in terms of admitted retry
  decisions and temporal wake footprint, not elapsed time span or graph size
- timeout admission cost must be stated in terms of temporal frontier and
  affected in-flight records, not all resources
- branch restore cost must be stated in terms of restored branch-local async
  state and retained summaries, not total live resource registry breadth
- replay reconstruction cost must be stated in terms of retained completion
  history and checkpoint span
- diagnostics expansion cost must be explicitly separated from operational
  completion admission cost
- batch completion admission cost must be stated in terms of input width,
  deduplicated width, admitted width, denied width, and observer candidate
  width
- retained summary read cost must be stated as retained summary lookup only,
  with zero cold reconstruction
- lifecycle retention compaction cost must be stated in terms of terminal
  lifecycle records selected by policy, not total graph size
- density strategy selection cost must be stated in terms of current in-flight
  pressure summary, not broad registry inspection

### 15.1 Named Async Performance Failure Modes

Milestone B should name the failure modes it intends to prohibit so later work
cannot reintroduce them under nicer names.

At minimum:

- `CompletionBroadScan`
  A completion path scans the graph, all resource nodes, or all in-flight
  records when the request identity should be enough.
- `InflightRetentionLeak`
  Dead, cancelled, superseded, or disposed in-flight records accumulate without
  a framework-owned reclamation path.
- `RetryStormAmplification`
  Retry/backoff scheduling creates work proportional to historical attempts or
  elapsed periods rather than admitted retry decisions.
- `BranchRestoreAsyncRebuild`
  Restoring a branch reconstructs async state by broad rediscovery instead of
  retained branch-local async artifacts.
- `AdapterTruthLeak`
  Host/UI adapter state becomes the only place that knows pending, fulfilled,
  rejected, cancelled, or stale lifecycle truth.
- `SilentCompletionDrop`
  Impossible, stale, duplicated, or malformed completions disappear without a
  typed denial artifact.
- `ProjectionCoupledCompletion`
  Completion admission updates facade summaries, observer packets, retained
  history, and diagnostics blobs as one maintenance surface.
- `ScalarCompletionLoop`
  Bulk completion traffic is forced through N public scalar calls, hiding lost
  amortization and fragmented observation coalescing.
- `HiddenColdReconstruction`
  A summary or ordinary facade read performs diagnostics reconstruction or
  retained-history expansion without an explicit budgeted API call.
- `OperationalAllocationChurn`
  Completion admission, cancellation, retry, or timeout allocates per record
  without a lifecycle-bounded buffer, arena, or counted debt posture.
- `DensityBlindStrategy`
  Sparse and dense async pressure use one unreported strategy even when the
  runtime has enough state to choose a cheaper truthful path.

## 16. Acceptance Evidence

Milestone B is complete only when `worth-signal` can certify all of the
following with canonical machine-checkable artifacts:

- the `Async Resource Lifecycle Parity Test`
- the `Out-Of-Order Completion Supersession Test`
- the `Async Rollback And Observation Equivalence Test`
- the `Async Branch Restore And Replay Equivalence Test`
- the `Async Inflight Boundedness Test`

The certification bundle must include canonical digests for:

- resource declarations
- lowered resource descriptors
- request identities
- generation and attempt lineage
- in-flight sets
- lifecycle transitions
- cancellation, timeout, retry, supersession, and revalidation decisions
- resolved resource policy descriptors and policy parameter digests
- resource policy selection bases and replay compatibility postures
- admitted completions
- denied completions and denial classifications
- committed resource states
- lifecycle-state digests separate from output-state digests
- observer delivery boundaries
- rollback suppression artifacts
- branch/restore async state
- replay lifecycle and denial history
- boundary performance envelopes
- cost contract ids and postures
- allocation posture counters
- retained summary read counters
- cold reconstruction counters
- density strategy decisions
- diagnostics/explanation artifacts

## 17. Architectural Notes

- Resource lifecycle vocabulary should be sealed enough to make product truth
  stable, while still allowing later higher-level resource products to compose
  domain-specific policy above the substrate.
- Policy extensibility is allowed only through deterministic descriptor
  registries. Arbitrary host callbacks may decide external work, but they may
  not decide retry, timeout, cancellation, supersession, revalidation,
  observation, output continuity, retention, or completion legality inside the
  hot runtime boundary.
- Built-in policies are not special execution branches. They must register and
  lower through the same policy descriptor path as later user-defined policies.
- `pending` is not a UI spinner. It is runtime-owned derived state associated
  with an admitted request and its generation/attempt identity.
- `fulfilled` and `rejected` are committed lifecycle results, not host promise
  states.
- `cancelled`, `timed-out`, `stale`, and `superseded` must remain distinct.
  Collapsing them into one error loses the actionable structure needed by
  diagnostics, retry, and replay.
- Timeout and stale-after semantics must consume Milestone A temporal
  primitives. This milestone may add async-specific timeout policy, but it may
  not invent a second clock model.
- Retry must preserve attempt lineage. A retry that looks like a brand-new
  unrelated request loses the lifecycle story and is out of spec.
- Revalidation must be distinct from retry. Retry continues an admitted intent
  after failure; revalidation admits new intent to refresh resource truth.
- Cancellation should support both best-effort host signalling and runtime-hard
  completion denial. Host signalling may fail; runtime denial may not.
- Completion payload shape remains generic. The runtime owns lifecycle and
  causality, not domain payload interpretation.
- Lifecycle state and successful output continuity must remain separate. The
  runtime may expose ergonomic summaries later, but the core substrate cannot
  collapse them into a single `Result`-like value without losing observation
  and replay honesty.

## 18. Explicit Deferrals

Milestone B intentionally does not include:

- wasm resource bindings
- React or Angular resource adapters
- route loaders
- form submit/action abstractions
- query replacement product surfaces
- network transport, fetch, websocket, or RPC implementation
- domain-specific cache eviction products
- persistence-layer resource cache storage
- speculative background refresh UX
- full resource policy family implementation beyond the descriptor registry
  skeleton and built-in policies needed by this substrate

Those remain later roadmap work. They can only be considered honest once they
reduce to this runtime-owned async/resource substrate.

## 19. Sequencing Notes

This milestone belongs immediately after Milestone A because async lifecycle
truth depends on runtime-owned time.

Milestone A closed:

- clock basis
- temporal policy
- stale-after
- interval wake generation
- previous-value access
- branch/restore/replay temporal artifacts
- temporal diagnostics and counters

Milestone B consumes that foundation to define:

- timeout semantics
- retry/backoff scheduling
- stale completion denial
- deterministic completion admission against temporal truth
- branch/replay parity for in-flight resource state

If async/resource substrate ships without this temporal dependency, it will
push time-sensitive decisions back into host glue and recreate the exact
folklore Milestone A eliminated.

## 20. Required Self-Check

- Does this milestone solve a real structural problem or just package work
  cosmetically?
  Yes. It creates the missing runtime-owned async/resource lifecycle substrate,
  not merely convenience APIs.
- Is the adversarial constraint precise and load-bearing?
  Yes. Stale/out-of-order/contradictory completion, rollback observation,
  branch restore, replay, and bounded in-flight cost all directly shape the
  architecture and acceptance tests.
- Does the milestone preserve crate authority boundaries?
  Yes. `worth-signal` owns derived lifecycle truth; hosts execute external
  work; relational/store remain truth/persistence authorities.
- Does the milestone define proof obligations, not just implementation tasks?
  Yes. Request identity, generation, branch epoch, completion admission,
  denial classification, transactional apply, replay digests, and counters are
  all required proof surfaces.
- Is performance encoded into architecture rather than left as observability?
  Yes. Hot/cold/projection storage is separated, every public boundary must
  return a performance envelope, cost contracts carry explicit status, scalar
  and batch completion admission are separate cost surfaces, and certification
  must prove sparse, dense, bursty, allocation, retention, and reconstruction
  behavior.
- Could a competent engineer map this spec into honest types, modules, and
  tests?
  Yes. The spec names subsystem boundaries, proof types, transaction lanes,
  diagnostics artifacts, performance envelopes, counters, failure modes, and
  certification families.
- Does the milestone belong in this roadmap sequence, or is it out of order?
  Yes. It directly follows Milestone A because async needs runtime-owned
  temporal semantics for timeout, stale-after, retry, and deterministic replay.

## 21. Milestone Done When

Milestone B is done only when `worth-signal` can support async/resource-backed
derived computation through a frozen, typed, replay-honest substrate that:

- preserves authority boundaries
- makes resource lifecycle runtime-owned truth
- rejects stale and impossible completions explicitly
- keeps completion apply transactional and rollback-safe
- keeps observation commit-bounded
- exposes bounded, measurable in-flight work
- integrates with temporal policy, diagnostics, branch, restore, and replay
  without inventing a second semantic story

At that point, higher-level wasm, route-resource, form, query, and app
resource surfaces can finally inherit one trustworthy async model instead of
inventing their own.
