# Storage Foundation S.6: Hardware-Aware I/O, QoS, And Background Pacing

## Goal

Make Store I/O decisions explicit, backend-aware, security-preserving, and
observable. Foreground durability and read work must retain bounded service
under background flush, compaction, repair, reclaim, and migration pressure.

## Product Decisions

- Backend capability admission uses observed or certified capabilities, not a
  configuration claim alone.
- Foreground and background work use typed lanes with explicit reservation,
  grouping, pacing, and backpressure rules.
- A queue admission decision binds the backend, security scope, durability
  requirement, work class, and resource envelope used by execution.
- Flush, sync, rename, and namespace durability ordering are modeled directly.
- Buffered, mmap, direct-I/O, trim, punch-hole, and cold-tier modes are
  capability-gated and have explicit unsupported outcomes.
- Background work yields before it violates foreground reservations.
- Latency and interference counters come from executed work. Exact structural
  counts stay exact; latency and capacity contracts use honest bounds.
- Simulation and injected faults qualify the exercised boundary only. They do
  not claim untested hardware guarantees.
- Security scope and secure-I/O posture survive every queue, batching, retry,
  fallback, and background-work transition.

## Ownership And Boundaries

- `worth-store-physical-backend` owns backend capability vocabulary and
  admitted backend profiles.
- `worth-store-io-scheduler` owns lanes, reservations, queue admission,
  grouping, pacing, and execution outcomes.
- Durability owners define flush/sync/rename ordering and acknowledgments.
- Physical-isolation owners define reclaim reachability and object lifetime.
- `worth-store-physical-certification` owns direct I/O-pressure scenarios,
  replay, observations, and physical fault classes.
- `worth-store-certification` may expose small adapters around real execution
  outcomes for courtroom assertions. It must not assemble a second readiness
  or certification protocol.

Runtime authority remains concrete and proof-carrying. Diagnostic records,
digests, transcripts, counters, and successful tests never authorize I/O.

## Core Contracts

### Backend capability admission

Admission binds a backend target profile to the evidence class that justified
each capability. Unsupported, unverifiable, or mismatched capabilities produce
typed denials before work enters a queue.

### Foreground reservations

Commit-critical WAL writes, point reads, recovery-critical work, and other
foreground classes receive explicit reservation semantics. Background work
cannot silently borrow a reservation whose deadline or scope it cannot honor.

### Background pacing

Flush, compaction, repair, reclaim, and migration have distinct pressure
classes. Pacing decisions consume current queue and service observations and
emit direct execution outcomes. The ordinary lane performs bounded work; large
reconstruction or certification work stays off that path.

### Durability ordering

The implementation distinguishes data persistence, file metadata persistence,
rename visibility, and namespace durability. A weaker barrier cannot satisfy a
stronger acknowledgment contract.

### Access and reclaim policy

Access mode is admitted against backend capability, alignment, lifetime,
security, and workload facts. Trim and punch-hole occur only after S.5
reachability allows reclaim and the backend contract makes the operation safe.

### Observation

Tests inspect queue depth, service and wait behavior, interference events,
allocation, durability operations, and policy violations directly. Counters
remain attached to the execution that produced them.

## Phase Plan

1. Admit backend and media capabilities.
2. Define foreground lane contracts and reservation admission.
3. Define background work classes and pacing policy.
4. Execute queue admission, grouping, and backpressure.
5. Enforce flush, sync, rename, and durability ordering.
6. Admit buffered, mmap, and direct-I/O access policies.
7. Admit trim, punch-hole, and cold-tier postures.
8. Preserve security scope and secure-I/O requirements.
9. Implement latency envelopes and interference counters.
10. Exercise I/O pressure through the S.4.5 harness.
11. Qualify supported backend profiles and state non-claims explicitly.
12. Publish typed handoffs for later storage milestones.
13. Verify public API adoption through direct callers and tests.
14. Run focused and cross-backend completion gates.

The phase list defines build order. It is not a mutable ledger. The reviewed
Git revision is historical evidence; current validity is the result of current
compilation, tests, and boundary checks.

## Required Direct Tests

- unsupported capability and weak-evidence profiles are denied;
- foreground reservations survive background flush, compaction, repair, and
  reclaim pressure;
- queue grouping never crosses durability or security boundaries;
- post-admission policy violations surface as typed failures;
- delayed sync, queue saturation, bandwidth throttling, page-cache pressure,
  and late-yield faults are delivered at real boundaries;
- missing pressure observations prevent a pressure oracle from passing;
- repeated deterministic scenarios preserve plan, schedule, transcript,
  oracle, and counter identity;
- different executed samples produce different counter observations;
- flush/sync/rename tests exercise actual ordering rather than copied flags;
- backend qualification distinguishes simulated, injected, emulated,
  host-observed, backend-certified, and externally guaranteed claims;
- secure-I/O and tenant scope survive fallback and retry paths;
- local, CI, and expensive profiles remain semantically equivalent while
  using different bounded resource envelopes.

## Prohibited Machinery

Do not add generated coverage rows, evidence registries, certification
materializers, readiness adoption receipts, residual-debt matrices, maturity
reports, source inventories, reviewer protocols, or tests that validate those
artifacts. Do not duplicate execution evidence merely to prove that the
duplicate cannot become runtime authority.

Protect behavior with direct tests at the owning boundary. Protect dependency,
authority, and visibility rules with compiler-visible structure and the
workspace boundary checker.

## Completion Rule

At the reviewed revision, focused scheduler/backend/durability/access/reclaim
tests, I/O-pressure scenarios, affected integration tests, cross-backend
qualification, boundary checks, formatting, and line-cap checks must pass.
Current changes are judged by those current results, never by a manually
maintained historical status.
