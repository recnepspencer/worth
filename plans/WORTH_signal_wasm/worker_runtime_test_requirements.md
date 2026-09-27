# worth-signals-wasm Worker Runtime Test Requirements

> **Status:** Planned certification spec
>
> **Roadmap parent:** [wasm_product_roadmap.md](./wasm_product_roadmap.md)
>
> **Milestone parent:** [worker_runtime_placement_plan.md](./worker_runtime_placement_plan.md)
>
> **Core lineage:** [plans/worth_signal/test-requirements.md](../../../plans/worth_signal/test-requirements.md)

## Purpose

This document defines the certification bar for the `worth-signals-wasm`
worker-first runtime placement milestone.

It is not a list of example tests.
It is the proof contract that closes the worker-runtime milestone.

The milestone is not done when:

- a demo app feels smoother
- a worker can be spawned successfully
- some expensive computations happen off the UI thread
- TypeScript APIs for worker mode look polished

The milestone is done only when the product surface can prove that:

- worker-first mode and main-thread compatibility mode converge to the same
  committed runtime truth
- browser-owned host facts and host-side effects remain explicit typed
  boundaries rather than ambient side channels
- callback portability limits remain honest across forward execution, replay,
  restore, import, and export
- one worker-ineligible node or effect cannot silently collapse unrelated graph
  breadth back onto the main thread
- the worker bridge stays breadth-bounded and does not merely relocate hidden
  serialization or delivery cost onto the UI thread

## Governing Summaries

- `MENTALITY.md`
  The most important thing it protects is hostile-proof infrastructure design.
  This certification must prove the worker boundary survives churn, history,
  and cost pressure, not just happy-path offload.
- `arch_laws.md`
  The most important thing it protects is boundary honesty. Cross-thread lanes
  must emit self-describing envelopes, preserve one runtime authority, and keep
  placement legality explicit rather than conventional.
- `perf_laws.md`
  The most important thing it protects is breadth honesty. The test suite must
  prove that bridge, delivery, and serialization costs scale with semantic
  delta rather than graph size.
- `domain_laws.md`
  The most important thing it protects is proof-domain clarity. Placement,
  ingress, egress, historical capability posture, and delivery boundedness
  need separate owning suites rather than one giant worker bucket.
- `worth_signal_vision.md`
  The most important thing it protects is that `worth-signal` remains derived
  execution substrate. Worker mode must move runtime authority, not create a
  second client truth engine.
- `worker_runtime_placement_plan.md`
  The most important thing it protects is the milestone boundary itself:
  worker-first is runtime-authority placement plus typed host bridges, not
  generic offload.
- `worth_signal/test-requirements.md`
  The most important thing it protects is certification rigor. This document
  must require named adversarial suites, replay/restore parity, compile-time
  boundaries, and exact cost proof rather than anecdotal smoothness.

## Adversarial Constraint

This certification program must survive the following hostile condition:

> A long-lived web application with large callback-authored and graph-published
> derived state, async-capable nodes, route churn, form activity, resource
> refresh and delivery churn, browser-history events, visibility/online/timer/
> viewport host updates, branch restore and replay activity, mixed worker-
> executable and main-thread-hosted authored work, and high observation/effect
> churn must converge to the same committed runtime truth, lifecycle truth,
> visible output truth, and diagnostics/history explanation in both
> main-thread compatibility mode and worker-first mode, while keeping
> main-thread work bounded to typed host-boundary admission, explicit
> main-thread-only effect execution, and committed public delivery.

If semantically equivalent histories can produce:

- different committed graph truth
- different lifecycle or observation truth
- different route/resource/forms continuity truth
- different denial or fallback classifications
- different replay/restore/import/export capability stories
- hidden broad bridge work behind cheap API shape
- or a second main-thread lifecycle/cache/router/resource authority

then the milestone has failed certification.

## Certification Rules

Every required named suite in this document must:

- run with canonical artifact emission, not only assertion-style pass/fail
- define its hostile workload explicitly
- verify runtime behavior, public product behavior, and type-surface boundaries
  where relevant
- certify replay/restore/branch parity whenever the milestone claims those
  semantics exist
- certify breadth or cost honesty whenever the API looks cheap
- include denial, fallback, detach, or unavailability artifacts where the
  worker boundary cannot be crossed honestly

Where a suite names a compile-time boundary, the package must maintain explicit
compile-fail fixtures or equivalent type-surface proof artifacts that stay in
sync with the public placement contract.

Because the worker placement plan requires `worth-proof` for the Rust-side
placement/lowering/readmission proof chain, compile-time boundary suites must
prove the `worth-proof` progression cannot be skipped or WORTHd. It is not
enough to test that local structs have private fields.

## Verification Package Standard

Every broad certification family should emit a canonical verification package
containing the categories relevant to that suite.

The package vocabulary for this milestone is:

- declaration identity digest
- placement classification digest
- placement identity digest
- lowered-plan identity digest
- worker runtime identity digest
- transaction envelope digest
- host-capability envelope digest
- browser-history envelope digest
- host-effect envelope digest
- output delivery digest
- observation delivery digest
- diagnostics/history read digest
- fallback and denial digest
- capability availability and reattachment digest
- replay/restore/import/export digest
- compatibility-mode truth digest
- worker-first truth digest
- boundary performance envelope
- allocation posture digest
- main-thread broad-work denial artifact

Equivalent runs must match exactly except for fields explicitly declared
non-semantic.

## 0. The Full Worker Hostile Parity And UI-Freeze Denial Test

Purpose

Prove that the complete worker-first product surface remains one coherent
system rather than one worker runtime plus several drifting main-thread helper
engines.

Why it matters

Phase-local suites can all pass while the real product still forks into:

- one truth story for compatibility mode
- another truth story for worker-first mode
- one capability story for forward execution
- another capability story for replay and restore
- one cost story for compute
- another hidden cost story for serialization and delivery

That is exactly the failure mode this milestone exists to prevent.

What to stress

Build one medium-large application graph containing:

- callback-authored computed and output nodes
- graph-published controllers
- async-capable nodes and resource-backed surfaces
- route churn and browser-history ingress
- host capability families such as visibility, viewport, online, clock, and
  persistence
- host effects that must execute on the main thread
- mixed worker-executable and main-thread-hosted authored work
- branch, restore, replay, and diagnostics reads

Run one hostile script with:

- repeated transactions with overlapping invalidation bursts
- async completion churn interleaved with host-capability updates
- browser back/forward and direct URL edits interleaved with app-issued
  navigation
- output delivery under large structured projection values
- branch fork before and after completion and host-effect execution
- restore before and after capability loss
- replay from retained history and replay from full canonical history
- compatibility-mode execution of the same semantic workload

Execute the full scenario in at least:

- main-thread compatibility mode
- worker-first mode
- branch fork plus restore execution
- retained-history replay
- full canonical replay

What to verify

- all modes converge to identical committed truth when semantically equivalent
- all modes converge to identical lifecycle and observation truth
- worker-first historical operations preserve capability posture explicitly
- no path creates a second lifecycle, cache, router, or resource authority on
  the main thread
- main-thread work remains bounded to host ingress, host effects, and public
  delivery rather than runtime-owned broad work
- fallback occurs only where the product surface explicitly admits it, and all
  other impossible worker postures deny honestly

Pass condition

The verification package must emit placement, truth, lifecycle, delivery,
capability, replay/restore, denial, and boundary-performance artifacts.
Equivalent histories must match exactly when semantically equivalent.

## Phase Coverage Map

- Full milestone closeout additionally requires suite 0.
- Phase 1 is closed only by suites 1 through 2.
- Phase 2 is closed only by suite 3 plus the non-callback/non-host slice of
  suite 4.
- Phase 3 is closed only by suites 5 through 7.
- Phase 4 is closed only by suites 8 through 10 plus the callback and
  main-thread-hosted slices of suite 4.
- Phase 5 is closed only by suites 11 through 12.
- Phase 6 is closed only by suites 13 through 14.
- Phase 7 is closed only by suites 15 through 16.

## Phase 1: Placement Taxonomy And Bridge Artifact Lock

1. The Placement Taxonomy And Envelope Identity Test

Purpose

Prove that worker-executable, main-thread-hosted, and unavailable work are
real product categories with one stable boundary-envelope vocabulary.

What to stress

- authored declarations that are clearly worker-executable
- authored declarations that are clearly main-thread-hosted
- authored declarations that are unavailable in worker-first posture
- equivalent boundary operations through transaction, capability ingress,
  browser-history ingress, host effect, output delivery, and diagnostics reads

What to verify

- one semantic declaration lowers to one stable placement category
- every boundary family emits one stable envelope category
- no envelope family collapses distinct failure or cost semantics into one bag
- transaction and generation causality fields are stable enough to order
  equivalent boundary histories canonically

Pass condition

The verification package must emit placement classification digest, envelope
family digest, and denial/fallback digest. Equivalent declarations must match
exactly.

2. The Placement Boundary Compile-Time Separation Test

Purpose

Prove that ordinary product code cannot bypass placement classification or
WORTH cross-thread envelope proofs accidentally.

What to stress

- constructing placement-bearing proofs outside the owning module
- calling worker-only or host-only surfaces without the required proof path
- forging boundary-envelope shapes directly
- widening maybe-worker declarations into definitely worker-executable use
- passing raw callbacks or raw declarations past the proving boundary
- constructing lowered worker or host execution plans without sealed proof types
- treating a `worth-proof` unresolved/raw placement payload as placement
  classified
- treating a placement-classified declaration as lowered without the worker or
  host lowering capability witness
- treating a lowered worker or host plan as execution-ready without the
  runtime-admission or host-boundary authority witness
- treating a boundary-bridged transport, restore, import/export, or host
  acknowledgement form as readmitted without explicit `worth-proof`
  readmission

What to verify

- the compiler rejects illegal placement access where possible
- runtime admission rejects any remaining WORTHd or widened paths before
  execution begins
- undeclared fallback paths cannot be reached through ordinary product APIs
- checked `worth-proof` outcomes preserve denial, fallback, unavailable, stale,
  rebind-required, and failed categories instead of flattening them into one
  ordinary error

Pass condition

No worker-only declaration helper, host-only execution surface, or boundary
envelope proof may be reachable without the corresponding declaration-bearing
proof path. Raw declarations must not be admissible to worker-first publication
or execution APIs once the proving boundary exists. Compile-fail fixtures must
cover the `worth-proof` placement progression stages directly, including
resolution, lowering, readiness, and readmission misuse.

## Phase 2: Worker-Owned Runtime Shell And Graph Lifecycle

3. The Worker Compatibility Truth Equivalence Test

Purpose

Prove that worker-first mode and main-thread compatibility mode mean the same
thing for equivalent non-host workloads.

What to stress

- identical already-lowered or placement-classified non-host graphs in both
  deployment modes
- invalidation bursts
- async lifecycle churn
- branch fork and restore
- large output graphs

What to verify

- committed graph truth is identical
- lifecycle, observation, and diagnostics truth are identical
- branch and restore semantics remain identical

Pass condition

The verification package must emit compatibility-mode truth digest,
worker-first truth digest, lifecycle digest, observation digest, and replay/
restore digest. Equivalent runs must match exactly, and the suite must not rely
on provisional raw-callback publication.

4. The Mixed Placement Graph Isolation Test

Purpose

Prove that one worker-ineligible node or effect does not silently collapse
unrelated graph breadth onto the main thread.

What to stress

- graphs containing both worker-executable and main-thread-hosted work
- invalidations that touch only worker-executable regions
- invalidations that cross the mixed boundary
- repeated recompute storms with one isolated host-only node

What to verify

- unrelated worker-executable regions remain worker-owned
- main-thread-hosted work stays isolated to its own declared boundary
- broad placement collapse never happens silently

Pass condition

The verification package must emit placement-frontier digest, worker-breadth
digest, main-thread-hosted digest, and broadening denial artifact.

Phase ownership note

This suite intentionally spans two gates:

- Phase 2 owns only the already-lowered, non-callback, non-host isolation slice
  required to prove the worker runtime shell does not collapse ordinary
  placement breadth.
- Phase 4 owns the callback-authored, main-thread-hosted, unavailable, and
  denial/fallback slices after placement classification and host-execution
  lowering exist.

Phase 2 may not satisfy this suite by inventing a provisional raw-callback
transport or temporary main-thread execution escape hatch.

## Phase 3: Main-Thread Host Capability And Host Effect Bridges

5. The Host Capability Worker Bridge Parity Test

Purpose

Prove that browser-owned host facts enter the worker runtime through one typed
ingress lane and preserve the same runtime truth as compatibility mode.

What to stress

- visibility, viewport, online/offline, clock, and persistence families
- coalesced and repeated equivalent updates
- overlapping async completions and invalidation churn
- branch restore before and after host updates

What to verify

- host updates are classified and routed identically in equivalent histories
- host capability never becomes ambient worker state
- coalescing does not change semantic truth
- worker admission preserves the declared transaction and generation order of
  host-boundary updates

Pass condition

The verification package must emit host-capability envelope digest, lifecycle
digest, truth digest, and coalescing digest.

6. The Browser History Worker Admission Parity Test

Purpose

Prove that browser-history and raw location ingress converge with app-issued
navigation on one canonical route truth in worker-first mode.

What to stress

- push, replace, popstate, and direct URL edits
- speculative navigation and redirects
- route-local resource continuity under history churn
- restore before and after browser-history events

What to verify

- equivalent navigation histories converge to one route truth
- browser events remain typed ingress, not ambient reads
- route continuity truth stays runtime-owned

Pass condition

The verification package must emit browser-history envelope digest, route truth
digest, continuity digest, and replay/restore digest.

7. The Main-Thread Host Effect Boundary Test

Purpose

Prove that host effects remain explicit main-thread execution boundaries rather
than becoming a second hidden lifecycle engine.

What to stress

- DOM or browser-facing effects
- success, failure, detachment, and unavailability
- effect churn under branch restore and output delivery churn

What to verify

- host effects execute only through typed requests
- effect completion or failure does not mutate runtime truth through ambient
  side channels
- detached or unavailable effect paths emit explicit artifacts
- acknowledgements do not become truth until worker-side typed admission

Pass condition

The verification package must emit host-effect request digest, acknowledgement
digest, denial/unavailability artifact, and lifecycle integrity digest.

## Phase 4: Computation Placement, Callback Eligibility, And Honest Fallback

8. The Callback Placement Eligibility And Denial Test

Purpose

Prove that callback-authored work is classified honestly as worker-executable,
main-thread-hosted, or unavailable rather than by folklore.

What to stress

- signal-only callbacks
- callbacks with main-thread-only host capture
- callbacks unavailable at restore/import time
- duplicate debug names or equivalent authored shapes with different
  capability posture

What to verify

- placement eligibility is explicit and stable
- denial/fallback/unavailability artifacts are typed
- no callback is treated as worker-portable merely because it compiled once
- main-thread-hosted execution consumes a closed worker-issued request and
  returns only typed result or denial artifacts
- main-thread-hosted execution cannot perform ambient graph reads or local
  shadow lifecycle writes
- categories not admitted to the narrow main-thread-hosted lane deny rather than
  widening into arbitrary main-thread derivation

Pass condition

The verification package must emit placement digest, denial/fallback digest,
capability availability digest, replay/import compatibility digest, and
placement-identity digest.

9. The Callback Host Read Dependency Admission Test

Purpose

Prove that host capability reads inside worker-first callback capture become
typed worker-owned dependencies, not ambient closure reads or per-read
main-thread RPC.

What to stress

- callbacks that read viewport, visibility, online, clock, and persistence host
  facts through admitted host handles
- callbacks that mix signal reads and host-capability reads
- host-capability churn that invalidates callback-backed readables
- detached, stale, missing, and unsupported host capability reads during
  callback capture
- main-thread-hosted callback execution with a worker-issued closed input
  frontier that includes admitted host fact snapshots

What to verify

- captured host reads lower into proof-bearing host dependency records
- worker-owned runtime truth owns the dependency edge from host ingress to
  callback recomputation
- host churn recomputes through committed host ingress, not fresh execution-time
  host requests
- unsupported host reads deny before publication or readmission
- main-thread-hosted callback execution rejects ambient host reads outside the
  closed input frontier
- compatibility mode and worker-first mode converge on equivalent host-read
  callback truth under the same host ingress sequence

Pass condition

The verification package must emit callback host-read dependency digest,
host-capability ingress digest, callback recomputation digest, ambient host-read
denial artifact, worker-first truth digest, compatibility truth digest, and
boundary-performance envelope proving zero per-read host RPC.

10. The Worker Ineligible Node Does Not Collapse Graph Breadth Test

Purpose

Prove the worst mixed-placement trap directly: one worker-ineligible node
cannot silently drag unrelated breadth back onto the UI thread.

What to stress

- worker-ineligible interior node
- worker-ineligible output node
- worker-ineligible effect
- dense invalidation pressure elsewhere in the graph

What to verify

- unrelated breadth remains worker-owned
- placement collapse emits explicit denial or fallback artifacts if it would
  otherwise happen
- cost counters explain the isolated host-boundary footprint

Pass condition

The verification package must emit isolation digest, worker breadth digest,
main-thread breadth digest, and placement-collapse denial artifact.

## Phase 5: Observation, Output, Diagnostics, And History Boundary

11. The Observation And Output Delivery Boundary Test

Purpose

Prove that off-thread committed truth becomes on-thread public delivery through
one bounded runtime story rather than many tiny reactive side channels.

What to stress

- large structured outputs
- overlapping observation and output delivery
- rollback-producing failures
- watcher/effect churn during delivery

What to verify

- committed observation and output delivery preserve one runtime story
- rollback suppresses delivery correctly
- delivery breadth scales with changed public surface, not total graph size

Pass condition

The verification package must emit output-delivery digest, observation-delivery
digest, rollback-suppression artifact, and delivery breadth envelope.

12. The Diagnostics Summary Cost Honesty Test

Purpose

Prove that summary diagnostics reads stay cheap and do not trigger hidden rich
reconstruction across the worker boundary.

What to stress

- repeated summary reads under retained and rich-history conditions
- reads before and after replay/restore capability is available
- worker-first and compatibility-mode summary reads

What to verify

- summary reads preserve semantic parity
- summary reads do zero rich reconstruction
- rich diagnostics remain a separate cost boundary

Pass condition

The verification package must emit diagnostics summary digest, rich-read
availability digest, cold-reconstruction counters, and boundary performance
envelope. Cold-reconstruction counters must remain zero for summary reads.

## Phase 6: Replay, Restore, Import/Export, And Capability Parity

13. The Worker Replay Restore Capability Honesty Test

Purpose

Prove that worker-first historical operations preserve capability posture, not
just output values.

What to stress

- branch fork with mixed placement work
- restore before and after host-capability updates
- restore before and after callback availability changes
- replay from checkpoint plus retained history

What to verify

- equivalent histories preserve the same capability story
- same-runtime exact restore and portable restore remain distinct when they
  should
- branch restore does not resurrect stale capability
- replay and restore compare canonical declaration, placement, and lowered-plan
  identities rather than names or object identity

Pass condition

The verification package must emit replay/restore digest, capability
availability digest, exact-restore artifact, and incompatibility or
unavailability artifact where relevant.

14. The Import Export Callback Unavailability Test

Purpose

Prove that import/export never lies about callback or host capability
portability.

What to stress

- export from worker-first mode with callback-authored work
- import into same-runtime exact restore posture
- import into portable transport posture without the needed callback or host
  capability
- mixed worker-executable and main-thread-hosted graphs

What to verify

- same-runtime exact restore admits richer reattachment only when honest
- portable import emits explicit unavailability artifacts where required
- imports never silently reuse stale callback-derived truth as live capability

Pass condition

The verification package must emit export digest, import digest, capability
reattachment digest, and callback-unavailability artifact.

## Phase 7: Certification, Performance Closeout, And Product Guidance

15. The Worker Bridge Boundedness Test

Purpose

Prove that transaction, host-capability, history, and delivery traffic stay
breadth-bounded across the worker bridge.

What to stress

- large graphs with narrow changed host surface
- large graphs with narrow changed public-delivery surface
- repeated equivalent host updates
- dense worker-side recompute with sparse main-thread delivery

What to verify

- transaction bridging scales with batch width, not total graph size
- host-capability ingress scales with changed host frontier
- output and observation delivery scale with changed public surface
- bridge allocation posture remains attributable and bounded

Pass condition

The verification package must emit bridge breadth envelope, allocation posture
digest, coalescing digest, and main-thread broad-work denial artifact.

16. The UI Freeze Surface Denial Test

Purpose

Prove that worker-first mode does not merely relocate compute while leaving the
UI thread exposed to broad serialization or delivery work.

What to stress

- large structured outputs
- repeated high-frequency invalidations
- simultaneous host-capability churn and output delivery
- compatibility-mode control runs

What to verify

- main-thread work remains bounded to typed host ingress, host effects, and
  public delivery
- broad runtime-owned work is denied from the UI thread
- worker-first mode materially narrows main-thread operational breadth compared
  with compatibility mode

Pass condition

The verification package must emit main-thread breadth digest, compatibility
control digest, worker-first delivery digest, and UI-freeze-surface denial
artifact.

## Certification Closeout Rule

The worker-runtime milestone is not closed until:

- every named suite in this document has a real owning proof lane
- suite 0 exists as a real hostile end-to-end certification lane rather than a
  narrative aspiration
- compile-time boundary suites have maintained compile-fail artifacts
- cost-honesty suites expose named counters or equivalent mechanical proof
- replay/restore/import/export suites emit canonical capability artifacts
- parity and history suites compare canonical declaration, placement, and
  lowered-plan identities rather than friendly names or object identity
- equivalent compatibility-mode and worker-first histories converge exactly when
  they mean the same thing
- denial, fallback, detach, and unavailability states remain explicit rather
  than drifting silently
- fallback exists only on product surfaces that explicitly declare it

The milestone fails certification if any supposedly worker-friendly surface:

- requires a second main-thread truth engine
- weakens route/resource/forms continuity truth
- hides broad bridge or serialization scope behind cheap API shape
- or can be explained only by reading implementation details instead of the
  canonical emitted artifacts
