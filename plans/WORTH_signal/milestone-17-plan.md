# Milestone 17 Engineering Spec: Portable Execution Backends And Distributed Coordination

> **Status:** Planned
>
> **Prerequisite:** [milestone-16-plan.md](./milestone-16-plan.md)
>
> **Architecture parent:** [signal_architecture2.md](./signal_architecture2.md), `S9.17.4`

## 1. Goal And Roadmap Placement

Milestone 17 makes WORTH's prepared graph and partition work portable across
execution substrates without changing signal meaning.

The milestone establishes one versioned backend protocol and capability model,
then proves it through native serial/threaded execution, the existing
`worth-signal-wasm` worker boundary, and a real remote-process execution path.
It also establishes the certification boundary an accelerator adapter must pass
before CPU/GPU or other device support can be claimed.

Backends execute proof-bearing work. They do not own graph semantics, node
meaning, dependency admission, determinism policy, authority, or commit.
Semantic scope paths and physical shard placement cross the boundary as
separate identities: the first describes admitted meaning, while the second is
replaceable execution layout.

## 2. Current Boundary

`worth-signal` currently embeds a native Rayon implementation behind its
`parallel` feature. `worth-signal-wasm` already has a substantial worker
boundary with:

- placement declaration and lowering proof
- worker-owned versus main-thread-owned runtime posture
- worker boundary envelopes and readmission proof
- explicit worker-unavailable compatibility artifacts
- deny-by-default or product-declared fallback policy
- worker/main-thread committed-truth parity certification

That WASM boundary is existing authority and must be integrated, not replaced
by a generic thread abstraction.

The missing cross-platform contract is a backend-neutral prepared work
artifact with explicit capability, version, determinism, memory, cancellation,
integrity, and result-readmission semantics. Without it, each new substrate
would invent its own execution meaning and recovery posture.

## 3. Adversarial Courtroom

Execute the same versioned prepared workload through:

- serial native execution
- bounded native threaded execution
- a real `worth-signal-wasm` worker-host boundary
- a real child-process or network-loopback remote executor using serialized
  protocol artifacts rather than in-memory calls

The workload must combine:

- graph antichains and structured partition patterns
- deep hierarchical scope paths, cross-shard dependencies, and boundary reads
- canonical and contract-equivalent reductions
- large immutable inputs and large worker-local outputs
- dynamic graph work that remains host-owned and therefore cannot be exported
- capability mismatch, protocol version skew, integrity mismatch, and stale
  epoch submission
- duplicate, delayed, reordered, lost, and corrupted result delivery
- worker/remote crash before execution, during execution, after result
  persistence, and during result delivery
- deadline, cancellation, retry, queue exhaustion, memory exhaustion, and
  host authority loss
- branch capture, restore, replay, and deterministic rerun
- main-thread-hosted callbacks that are unavailable in a worker
- backend-specific shard layouts, including a placement different from the
  native reference plan and reordered physical result arrival

Required result:

- every backend either produces the same contract-valid canonical result or a
  typed precommit denial/failure/recovery outcome
- unsupported work is rejected before transfer or execution
- duplicate results are idempotently recognized and cannot commit twice
- stale-epoch results cannot overwrite newer authoritative intent
- retry never assumes an indeterminate remote execution did not occur
- backend crash leaves authoritative graph truth intact
- WASM worker unavailability follows the existing explicit fallback policy
- transport, device, and worker scheduling never become replay meaning
- a backend may change physical placement but cannot change the admitted scope
  path, work identity, or canonical publication

The courtroom must convict:

- serializing arbitrary closures or host pointers
- backend-side graph access or semantic re-planning
- a hidden main-thread WASM fallback
- retrying an indeterminate remote result without an idempotency identity
- accepting a result whose protocol, plan, capability, epoch, or integrity
  identity does not match the admitted work
- claiming distributed execution from an in-memory fake
- claiming accelerator support from a trait with no real conformance evidence
- conflating `ScopePath` with `ExecutionShardKey`, or accepting a result whose
  semantic locality matches but shard-plan/epoch binding does not

## 4. Product Decision Lock

### 4.1 The Core Owns A Port, Adapters Own Mechanisms

`worth-signal` owns the semantic backend port, capability requirements, and
prepared work/result envelopes. Native pools, browser workers, device runtimes,
process transports, and network transports live in adapter boundaries.

No adapter may import graph internals or construct core execution proof.

### 4.2 Only Portable Prepared Work May Cross

Backend-crossing work contains canonical data, stable computation identity,
versioned implementation identity, declared input/output schema, required
capabilities, determinism contract, resource envelope, epoch, and integrity.

Opaque Rust closures, JavaScript callbacks, raw pointers, graph handles, ambient
context, credentials, and authority-bearing runtime references cannot cross.
Main-thread-hosted WASM callbacks therefore remain host work unless a portable
definition exists.

### 4.3 Capability Negotiation Precedes Transfer

The planner resolves backend eligibility before allocating large transfer
buffers or submitting work. Capability comparison includes:

- protocol and computation version
- supported structured patterns
- determinism level
- numerical and data-layout requirements
- memory space and maximum resident/transient bytes
- cancellation/deadline support
- trust, privacy, and data-classification scope
- result durability and recovery posture

Capability descriptors report what a backend can execute. They do not grant
data-disclosure or graph-mutation authority.

### 4.4 Results Re-enter Through Readmission

A backend result is derived, untrusted input until WORTH validates:

- request, plan, computation, version, and epoch identity
- integrity and schema
- deterministic/equivalence contract
- cancellation and deadline disposition
- output footprint and resource report
- duplicate/idempotency status

Only a sealed `BackendResultReadmission` may enter canonical publication.

### 4.5 Distributed Execution Uses Epochs And Idempotency

Every submitted batch has a stable idempotency identity and input epoch. The
remote side records enough durable execution disposition to distinguish not
started, running, completed with retained result, failed before result, and
indeterminate.

Retries reuse the same idempotency identity. An indeterminate result is never
described as rolled back. Recovery may inspect, retrieve, abandon, or
reconcile according to typed policy.

### 4.6 Distributed Commit Remains Host-Authoritative

Remote executors compute derived packets; they do not participate as peers in
graph authority. The host validates returned packets and publishes the graph
epoch. Multi-host authoritative graph replication or consensus is outside this
milestone and cannot be inferred from remote compute.

### 4.7 WASM Preserves Existing Worker Authority

`worth-signal-wasm` maps its placement proof, worker boundary envelope,
readmission proof, and fallback policy into the portable backend contract.

Worker-first remains preferred for heavy portable work. Main-thread execution
occurs only when the product declared that fallback and the resolved execution
report records it. Worker-unavailable denial remains a valid typed outcome.

### 4.8 Accelerator Support Must Be Earned Per Adapter

The protocol may describe accelerator-compatible work and memory spaces, but
WORTH claims support for a device family only after a real adapter passes
semantic, numerical, cancellation, memory-transfer, and failure certification.
The existence of a backend trait is not device support.

### 4.9 Portable Locality And Placement Remain Separate

Portable work carries the admitted semantic `ScopePath` identity needed for
readmission and a distinct non-authoritative shard assignment used for transfer
and execution. A backend may lawfully choose a different physical assignment
when capability, memory, or topology requires it, but the host must record and
validate that derived plan against the request epoch.

Returned packets identify both the semantic work they answer and the physical
plan that produced them. Readmission rejects missing, stale, or mismatched
semantic locality, partition, shard-plan, boundary-read, computation, or epoch
identity. Neither a remote host nor a worker may reinterpret path segments or
mint new semantic work from placement.

## 5. Required Boundary Forms And Caller DX

The implementation must establish canonical equivalents of:

```rust
pub struct BackendCapabilityDescriptor { /* versioned executable capabilities */ }
pub struct BackendRequirementSet { /* lowered requirements, not preference */ }
pub struct PortableComputationIdentity { /* definition and implementation version */ }
pub struct PreparedBackendBatch { /* serializable proof-bearing work */ }
pub struct BackendSubmissionEnvelope { /* epoch, idempotency, budget, integrity */ }
pub struct BackendResultEnvelope { /* output, disposition, cost, integrity */ }
pub struct BackendResultReadmission { /* sealed validation proof */ }
pub struct RemoteExecutionRecoveryHandle { /* inspect/retrieve/reconcile */ }
pub struct BackendConformanceReport { /* semantic and operational evidence */ }
pub struct PortableSemanticLocality { /* admitted scope-path identity */ }
pub struct PortableShardPlacement { /* replaceable physical layout */ }
```

Ordinary callers continue to express posture rather than mechanism:

```rust
let outcome = runtime
    .evaluate_many(targets)
    .with_backend_preference(BackendPreference::AnyCertified)
    .run()?;
```

An advanced caller may constrain locality, disclosure, transfer, deadline, or
backend class. Selecting a remote or accelerator class must make transfer,
trust, failure, and recovery responsibility visible in the API.

## 6. Architectural Destination

Milestone 17 completes the committed backend topology and establishes external
adapter boundaries:

```text
crates/worth-signal/src/
  data/proof/execution/
    backend.rs                         [capability and readmission proof]
  logic/planner/execution/
    backend/
      mod.rs                           [stable internal backend port]
      serial.rs                        [reference implementation]
      native.rs                        [bounded native adapter]
      admission.rs                     [requirement/capability resolution]
      readmission.rs                   [result validation]

crates/worth-signal-execution-protocol/ [created boundary-schema crate]
  src/
    lib.rs                             [stable protocol facade]
    capability.rs                      [versioned capability wire form]
    submission.rs                      [prepared batch envelope]
    result.rs                          [result/disposition envelope]
    locality.rs                        [semantic scope-path wire identity]
    placement.rs                       [physical shard-plan wire identity]
    recovery.rs                        [idempotency and recovery protocol]
    integrity.rs                       [digest and schema identity]

crates/worth-signal-wasm/src/runtime/
  worker_bridge/                       [existing authority boundary]
    portable_backend_admission.rs      [created adapter]
    portable_backend_submission.rs     [created adapter]
    portable_backend_readmission.rs    [created adapter]
    worker_deployment_posture.rs       [existing, preserved]
    worker_fallback_policy.rs          [existing, preserved]

crates/worth-signal-remote-executor/   [created external-effect adapter]
  src/
    lib.rs                             [adapter facade]
    client/
      mod.rs
      admission.rs
      submission.rs
      recovery.rs
    host/
      mod.rs
      registry.rs
      execution.rs
      result_retention.rs
    transport/
      mod.rs                           [transport port]
      process.rs                       [real reference boundary]

crates/worth-signal/src/tests/parallel_execution/
  backend_conformance.rs
  distributed_recovery.rs
  wasm_worker_parity.rs
  oracle/serial_execution.rs
```

The protocol crate owns stable boundary representation, not graph semantics or
execution authority. The remote adapter owns external lifecycle and transport.
The WASM adapter remains under its existing worker authority. Higher-authority
core meaning does not import either adapter.

Forbidden placements include network or browser code in `worth-signal`, graph
semantics in the protocol crate, credentials in prepared batches, transport
retries hidden under cheap-looking evaluation calls, generic backend helpers,
or adapter code re-exported as core authority.

## 7. Ordered Implementation Phases

### M17.0 - Port, Protocol, And Compatibility Freeze

- freeze backend capabilities, requirements, portable computation identity,
  protocol versioning, integrity, and readmission
- identify which existing computation forms are portable and which remain host
  only
- make unsupported crossing fail before serialization
- freeze separate semantic-locality and physical-placement identities in every
  submission/result lane

### M17.1 - Serial And Native Conformance

- execute the protocol form through reference serial and bounded native
  adapters
- prove the boundary form does not change local semantics
- install the common conformance suite
- prove alternative native shard layouts preserve the same admitted work and
  canonical result

### M17.2 - WASM Worker Integration

- adapt the existing placement/lowering and worker boundary proofs
- preserve worker-first and declared fallback postures
- certify worker/main-thread/serial truth parity and callback unavailability
- preserve semantic scope identity while allowing worker-specific placement

### M17.3 - Remote Submission And Recovery

- implement a real process boundary with serialization and failure injection
- establish idempotent submission, durable disposition, result retention,
  deadline/cancellation, duplicate delivery, and recovery handles
- validate results through core readmission before publication
- report cross-shard transfer, boundary reads, migrations, and retained result
  residency separately from semantic work

### M17.4 - Accelerator Conformance Boundary

- freeze numerical, memory-space, transfer, cancellation, and determinism
  requirements for future device adapters
- add a conformance harness that a real adapter must pass
- do not mark any device family supported without scheduled real-hardware proof

### M17.5 - Cross-Backend Certification And Closeout

- run canonical workloads across serial, native, WASM worker, and remote
  process backends
- certify protocol skew, corruption, crash, retry, replay, and branch behavior
- seal the portable-backend certification run

## 8. Documentation Deliverables

Milestone 17 must create or revise documentation for three audiences:

- application callers: backend posture, deadline, cancellation, transfer,
  disclosure, fallback, and typed failure/recovery outcomes
- computation authors: portable definition requirements and reasons work may
  remain host-only
- backend implementers: protocol versions, capability negotiation, memory,
  numerical determinism, idempotency, readmission, and conformance obligations

`worth-signal-wasm` documentation must describe worker-first deployment,
main-thread fallback authority, worker-unavailable outcomes, and portable versus
host-callback work. Remote documentation must expose that execution crosses an
external effect boundary and may become indeterminate.

## 9. Must Ship And Must Preserve

Must ship:

- one versioned backend capability and protocol model
- portable prepared work and result envelopes
- sealed result readmission
- native serial/threaded conformance
- real WASM worker integration through existing authority
- real remote-process execution with idempotency and recovery
- accelerator conformance contract without an unearned support claim
- cross-backend certification artifacts and structural cost reports
- separate portable semantic-locality and replaceable shard-placement forms

Must preserve:

- Milestones 14-16 resource, graph, partition, and determinism proof
- core domain independence
- host-authoritative graph commit
- branch, replay, rollback, observation, temporal, async, and invalidation truth
- explicit WASM fallback and worker-unavailable behavior
- privacy and disclosure authority separate from execution capability

## 10. Explicit Exclusions

Milestone 17 does not:

- make remote executors authoritative graph replicas
- implement multi-host consensus or transparent distributed transactions
- serialize arbitrary closures, callbacks, pointers, or runtime authority
- put Web Worker, GPU, network, or geometry vocabulary in core computation
  meaning
- make backend placement, machine topology, or transport routing part of
  invalidation or replay meaning
- claim a device backend without real adapter certification
- hide network/device transfer or indeterminate recovery behind a synchronous
  property-shaped API

## 11. Acceptance Evidence

Milestone 17 closes only when:

- serial, native, WASM-worker, and remote-process executions agree under the
  requested determinism/equivalence contract
- unsupported capabilities and versions are rejected before expensive transfer
- corrupt, stale-epoch, mismatched, or duplicate result packets cannot publish
- remote crash points produce exact typed disposition and recovery behavior
- retry uses stable idempotency identity and cannot commit twice
- authoritative graph state survives every backend crash and lost response
- WASM worker fallback occurs only under declared product policy and is visible
- main-thread-only callbacks are never exported as portable computation
- backend-specific shard layouts preserve scope/work identity and reject every
  stale or mismatched semantic-locality/placement/epoch combination
- reports expose serialization bytes, transfer bytes, queueing, execution work,
  peak memory, retries, duplicate results, recovery actions, publication
  breadth, cross-shard traffic, migrations, and residency by named lane
- protocol and adapter mutation probes turn conformance evidence red
- focused tests, complete affected suites, real WASM/browser or worker proof,
  real process-boundary proof, boundary checks, context checks, formatting, and
  dirty Rust line-cap checks pass

## 12. Successor Handoff

After Milestone 17, domain crates may build specialized computation libraries
over structured partition declarations and certified backends. They must remain
consumers of `worth-signal` infrastructure. Any future asynchronous monotone
fixed-point, new accelerator family, or authoritative distributed graph work
requires its own specification and cannot be inferred from this closeout.
