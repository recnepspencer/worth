# WORTH Jobs Vision

## Thesis

WORTH does not need a better background queue.

It needs a first-class work runtime.

`WORTH-jobs` is the WORTH-native admitted-work semantics layer for
basis-bound, proof-bearing work. It uses WORTH's durable store, reactive signal
runtime, bridge protocol, query surface, and server delivery runtime as its
foundation. It turns asynchronous execution from "run this callback later" into
an explicit protocol:

- declare what work means
- admit it against a truth or artifact basis
- lower it into an execution packet
- lease it to a worker
- submit result candidates
- publish only through the owning authority
- emit replayable evidence for every decision

The job runner does not own truth semantics, storage semantics, signal
scheduling semantics, query semantics, bridge semantics, server delivery
semantics, cloud policy, or domain semantics. It owns the lifecycle and meaning
of admitted work: deduplication, leasing, resource reservation, retry,
cancellation, supersession, deadline handling, progress, partial results, and
publication handoff.

Most worker systems are excellent at execution mechanics and weak at semantic
meaning. They can tell you whether work ran. They struggle to tell you whether
the work was still meaningful, still authorized, still fresh, still worth doing,
and still publishable when it finished.

`WORTH-jobs` exists to make those answers structural.

The runtime is domain-agnostic. Web, supply-chain, music, CAD, AI, and cloud
examples are proof that the abstraction is broad enough; they are not domains
baked into the crate. Domain meaning arrives through registered job families,
typed resource declarations, input/output contracts, and authority witnesses.
The core runtime must remain generic enough that a workload can be a thumbnail
render, warehouse exception, nightly model training pass, database backfill,
simulation checkpoint, billing reconciliation, or human approval without the
job engine knowing the business meaning internally.

## What This Runtime Is For

`WORTH-jobs` exists for systems where asynchronous work must remain correct
after time passes, truth changes, workers crash, inputs drift, code versions
roll forward, resources become scarce, and partial results race each other.

It is meant to support:

- web applications that need emails, webhooks, imports, exports, backfills,
  thumbnail generation, search indexing, AI assists, and live user-visible work
  without rewriting idempotency, retry, and stale-result guards for every
  feature
- supply-chain, logistics, and physical operations systems that need work to
  respect deadlines, freshness windows, perishable goods, real-world resources,
  human approvals, dock appointments, inventory lots, and chain-of-custody
  constraints
- music, media, and design systems that need waveform analysis, stem
  separation, preview renders, export pipelines, plugin scans, freeze/bounce
  artifacts, and partial previews without stale work overwriting newer creative
  truth
- geometry, CAD, chip, and simulation systems that need long-running analysis,
  mesh generation, timing checks, DRC runs, export jobs, branch-local previews,
  and resumable compute over exact bases
- AI systems that need speculative, advisory, approximate, and exact work lanes
  where results can be promoted, rejected, quarantined, or left branch-local
  without hidden authority leaks
- WORTH Cloud workspaces that need tenant-safe, resource-aware, observable,
  replayable execution across worker fleets without forcing every product team
  to rebuild queue semantics

The technical thesis is the same across all of them:

- async work must bind to explicit basis
- workers may compute, but authorities publish
- stale work must classify, not blindly retry
- ordering must be declared, not inferred from queue order
- resources must be reserved and budgeted as first-class objects
- partial, degraded, speculative, and advisory outputs must never masquerade as
  exact truth
- job history must explain meaning, not only execution timing

## Why This Runtime Is Different

These are the capabilities that make `WORTH-jobs` strategically different from
ordinary queues and workflow engines:

- basis-bound job admission
- semantic idempotency identities
- normal queue primitives: enqueue, delay, schedule, lease, heartbeat, retry,
  timeout, cancel, dead-letter, pause, resume, drain, archive, and purge
- workflow/DAG primitives: tasks, dependencies, barriers, joins, fanout, mapped
  tasks, dynamic expansion, retries per task, and graph-level state
- scheduler primitives: one-shot jobs, recurring schedules, cron-like calendars,
  event-triggered jobs, asset-dataset-triggered jobs, and missed-run catchup
- backfill and catchup orchestration with explicit date/range windows
- task/operator separation between orchestration semantics and execution
  adapters
- authority-separated result publication
- typed freshness, deadline, and value-decay contracts
- business-impact classification
- typed resource calendars for digital, human, and physical resources
- job families with declared basis, resource, retry, cancellation, result, and
  publication contracts
- result candidates instead of direct worker publication
- publication witnesses and supersession records
- partial, degraded, approximate, advisory, speculative, and exact result modes
- queryable work debt
- human-in-the-loop gates as first-class lifecycle states
- external side-effect intents, receipts, and reconciliation records
- semantic dependencies over evidence, not only job ids
- branch-local and preview-scoped work with zero-authoritative-residue discard
- compatibility witnesses for long-lived queued work across code and artifact
  version changes
- simulation and dry-run of job graphs before execution
- placement-aware jobs that reason about data locality, blob tiering, recall
  cost, and worker placement
- replayable evidence bundles that explain admission, execution, publication,
  cancellation, degradation, and repair

If these are treated as optional queue features, the system collapses back into
ordinary worker glue: durable enough to run code later, but not honest enough to
explain whether later still meant the same thing.

## Mission

`WORTH-jobs` exists to make work safe, inspectable, and composable across time.

It must answer these questions as native job-runtime responsibilities:

- Why does this work exist?
- Which truth or artifact basis made it valid?
- What input packet was the worker allowed to consume?
- What resource, budget, tenant, deadline, and value-decay constraints govern
  it?
- What makes this job the same semantic operation as another job?
- What does cancellation mean if the worker is already running or already
  publishing?
- What happens when newer truth supersedes the admitted basis?
- Is a stale result publishable, historical-only, rebaseable, or rejected?
- Which authority is allowed to make the result visible?
- Which partial results are visible, and what trust class do they carry?
- What external effects were issued, acknowledged, reconciled, or compensated?
- What work debt remains if this job is delayed, degraded, or cancelled?

The core product promise is:

> Async execution without async meaning drift.

## Adversarial Constraint

`WORTH-jobs` must survive this hostile condition:

> A multi-tenant WORTH Cloud workspace with active branch edits, live
> subscriptions, large blob-backed artifacts, long-running AI/media/analysis
> jobs, external side effects, human approvals, worker crashes, deploy-time code
> version skew, retry storms, resource starvation, and out-of-order completion
> must never let stale, duplicate, partial, cancelled, incompatible, or
> authority-rejected work silently publish as current truth or exact derived
> state.

If any supported path:

- lets a worker publish directly into truth or durable artifact authority
- treats queue order as semantic order
- retries a permanent semantic rejection as though it were transient
- allows old work to overwrite newer basis-compatible work
- hides dynamic inputs behind ambient reads
- loses the distinction between exact, degraded, advisory, and speculative
  results
- lets a cancelled preview job leave authoritative residue
- lets tenant or resource pressure silently degrade unrelated work
- or requires operator folklore to explain why work ran, published, stalled, or
  became obsolete

then `WORTH-jobs` has failed.

## Architectural Model

### Runtime stack

| Layer | Responsibility | Owns |
| --- | --- | --- |
| `worth-relational` | Truth semantics | identity, transactions, branches, commits, lineage |
| `worth-store` | Durable survival | commit/artifact persistence, snapshots, blobs, compatibility, retention |
| `worth-signal` | Derived computation | dependency graph, invalidation, reactive execution |
| `worth-runtime-bridge` | Truth-to-compute protocol | basis-aware subscriptions, routing, causality, replay |
| `worth-query` | Developer-facing query intent | query expression, narrowing, result shape, live promotion |
| `worth-server` | Network delivery runtime | subscriptions, sessions, auth, sync protocol, cursor resume, delivery lanes |
| `WORTH-jobs` | Admitted-work semantics | admission, leasing, retry, resources, deadlines, publication handoff, evidence |
| WORTH Cloud | Managed platform | tenant/workspace operation, hosted workers, policy plane, observability |

`WORTH-jobs` is not WORTH's durability layer, WORTH's dependency DAG, WORTH's
subscription protocol, WORTH's query engine, or WORTH's network server. Those
already exist as separate authorities. Jobs defines the rules for work that
survives across time, workers, retries, resource scarcity, external effects,
and publication races.

### Ownership boundary

`WORTH-jobs` owns:

- job family registration and declared work contracts
- queue lifecycle primitives
- recurring schedules, delayed work, trigger records, catchup, and backfill
  orchestration
- workflow graph, task, mapped-task, barrier, join, and graph-run lifecycles
- operator contracts and execution-adapter boundaries
- job declaration, admission, and rejection lifecycles
- semantic idempotency identities
- basis-bound work packets
- worker leases and heartbeats
- retry, backoff, dead-letter, and poison classification surfaces
- cancellation and supersession lifecycles
- deadlines, freshness windows, expiration, and value-decay policy
- resource reservation and resource calendar coordination
- progress, partial result, and degradation state
- result candidate intake and publication handoff
- side-effect intent, receipt, and reconciliation protocol
- queryable work debt and work impact reporting
- job evidence bundles, replay, and certification surfaces

`WORTH-jobs` does not own:

- truth mutation legality
- durable artifact semantic meaning
- storage layout, compaction, or blob placement authority
- signal scheduling internals
- bridge subscription meaning
- query meaning, result shape, or policy narrowing
- server sync protocol, delivery classes, auth, session, or transport behavior
- domain-specific business decisions
- cloud tenant policy itself

### Structural rule

Workers execute lowered packets. They do not discover meaning.

Authorities publish results. Workers do not.

Store persists the job world. Signal reacts inside the job world. The bridge
connects truth changes to job and runtime causality. Query exposes job intent,
inspection, and subscriptions. Server delivers job state, progress, and
results to external consumers. Cloud operates workers, queues, resources,
tenants, and policy at scale.

The job runtime coordinates admitted work. It does not become the owner of the
domain reason that work exists, the durable bytes that preserve it, the
reactive graph that observes it, or the network session that delivers it.

### Deeper WORTH runtime integration

`WORTH-jobs` should explicitly dig into the deeper WORTH layers instead of
rebuilding weaker versions of them.

Signal conditional nodes are a first-class job foundation. A job trigger,
readiness gate, retry delay, resource wait, human approval wait,
freshness-window wait, downstream dependency wait, or publishability gate can
be represented as deferred reaction over named conditions. Jobs should use
that machinery where the problem is reactive readiness:

- a scheduled job becomes ready only after its time condition and basis
  condition both hold
- a backfill lane advances only while resource and tenant-budget conditions
  admit it
- a downstream job wakes only when the required evidence, artifact, or
  publication witness exists
- a stale job can remain parked until a rebase, rebuild, cancellation, or
  historical-only retention condition resolves

Those conditions do not replace job admission, leases, attempts, or publication
rules. They give the job runtime a WORTH-native way to delay reaction until the
world is actually ready.

Relational merge and commit strategies are also directly relevant. Jobs that
produce truth mutations should not smuggle writes through worker callbacks.
They should submit result candidates that can be lowered into relational
strategy requests, merge plans, or validated commit plans. The relational
authority still owns canonicalization, strategy execution, lowering,
validation, merge readiness, and final commit. Jobs contributes the work
evidence: which admitted job produced the candidate, which basis it consumed,
which worker attempt submitted it, which dynamic reads occurred, which
side-effect receipts exist, and why publication is being requested now.

This matters for:

- branch-local AI or editor work that proposes changes but must merge through
  relational policy before becoming truth
- reconciliation jobs that produce intent-resolution, entity-replacement,
  aspect-field, or replica-convergence strategy inputs
- maintenance jobs that repair indexes, correspondence artifacts, or derived
  truth-adjacent structures without gaining truth authority
- long-running import, transform, or backfill jobs whose output must be
  committed through explicit strategy and merge witnesses

Extensible invariants are a third integration point. They are useful anywhere a
job family needs a typed, reusable admissibility or certification check that is
not merely queue mechanics. Invariants can participate in job admission,
pre-publication validation, post-publication certification, repair targeting,
and worker-fleet trust checks without becoming application folklore.

Examples include:

- a job may only publish if the target branch still satisfies the invariant
  class the job family declared at admission
- a result candidate may be quarantined when a custom invariant fails, while
  the worker attempt remains successfully executed
- a repair job can target the exact invariant witness that failed instead of
  scanning logs or retrying blindly
- a certified queue or worker fleet can prove it never admitted work that
  violates tenant, schema, branch, resource, or side-effect invariants
- domain extensions can register invariant families for CAD topology,
  supply-chain custody, media export completeness, AI policy gates, or finance
  reconciliation without adding domain meaning to the core job runtime

The rule is simple: Jobs can consume relational strategies, merge artifacts,
and invariant witnesses as authority-owned contracts. Jobs must not become the
authority that defines truth legality.

### Cross-roadmap obligations

The Store, Bridge, and Query roadmaps imply several job-runtime obligations
that must be named now so they do not become accidental gaps later.

#### Store-backed survival and artifact honesty

Job state must participate in Store's durable-artifact model instead of hiding
inside a queue table. Job lifecycle records, schedules, leases, attempts,
progress, packets, side-effect receipts, result candidates, dead letters,
backfill ranges, and evidence bundles need explicit artifact classification:
authoritative job lifecycle state, derived durable support artifact, or
ephemeral/session-local runtime state.

Derived job artifacts also need Store-style accuracy classes. A cached analysis
result, advisory AI result, partial media render, rebuilt support artifact, or
heuristic scheduling summary must never be consumed as exact truth-grade
evidence unless its family can prove that class.

Jobs should preserve Store's operating-mode honesty:

- in durable mode, acknowledged job lifecycle transitions survive crash and
  recovery through Store-backed artifacts
- in embedded mode, an application may own runtime execution while Store
  persists admitted job artifacts, checkpoints, and evidence without seizing
  lifecycle authority
- in absent mode, jobs may run as in-memory or test/runtime-only work, but no
  durable resume, recovery, or restart claim may be implied

Store recovery, compatibility, retention, replication, and repair programs must
be visible to Jobs. A job runtime should be able to say whether a job artifact,
checkpoint, result candidate, blob, or subscription-support dependency is
exactly available, degraded but recoverable, rebuild-required, quarantined, or
not resumable.

#### Maintenance, budgets, and admission control

Store's maintenance, tiering, budget, and repair roadmaps are directly relevant
to Jobs. Background compaction, rebuild, replication prep, blob recall,
subscription-support rebuild, and analysis checkpoint refresh are all work.
Jobs should provide the work protocol for those programs without making Store
give up ownership of retention, placement, compatibility, or repair meaning.

Job admission must therefore understand Store-published budget and risk
surfaces:

- branch depth and retained-history pressure
- WAL growth and snapshot density
- derived artifact footprint and rebuild debt
- subscription-support artifact footprint and resumability debt
- blob footprint by tier and cold-recall cost
- tenant-scoped blast radius and repair/quarantine scope

This turns "storage is under pressure" into typed job admission, deferral,
degradation, repair, or denial rather than hidden background slowdown.

#### Bridge causality and family-aware protocol lowering

The bridge roadmap says bridge work must be planned, lowered, replayable,
policy-explicit, and family-aware. Jobs should follow that same shape for any
truth-to-work or work-to-derived-state integration.

Jobs should explicitly integrate with:

- patch-to-job triggering through stable basis and snapshot-backed bridge
  source contracts
- aspect-, region-, partition-, and facet-aware job triggers where the bridge
  admits narrower routing than whole-entity invalidation
- lineage, structural correspondence, merge-aware history, and branch-local
  preview flows for deciding whether work continues, splits, rebases,
  supersedes, or rejects
- bridge change-stream, checkpoint, backpressure, and multi-consumer contracts
  where jobs consume or produce long-lived streams
- bridge policy propagation for deterministic-vs-optimized execution,
  tolerance, cost, priority, convergence, artifact retention, and diagnostics
- bridge writeback families when derived/job output wants to become a
  truth-mutating candidate without bypassing commit authority
- bridge-native subscription declaration, lifecycle, continuation, fanout,
  preview, and certification bundles where job progress, job dependencies, or
  job-triggering semantics depend on long-lived observation

Jobs must not treat "subscription event happened" or "change stream advanced"
as complete meaning. It should consume the admitted bridge family, basis,
checkpoint, continuation, and diagnostics artifacts that explain what that
event means.

#### Query facade, policy, and developer ergonomics

The Query roadmap makes Query the daily-driver framework surface for ordinary
application code while preserving lower-crate authority. Jobs should be shaped
so Query can expose job capabilities without becoming the job runtime.

That means Jobs should define canonical artifacts that Query can lower into:

- typed job declarations and job-family selection
- workflow-aware predicates over job state and work debt
- job result-shape and progress-shape metadata for dashboards, inspectors,
  grouped/kanban views, timelines, charts, and delivery surfaces
- branch, preview, historical, and diff contexts for job declarations and job
  evidence queries
- policy-aware and tenant-aware job admission, masking, and delivery metadata
- query-authored mutation, merge, and writeback declarations that create job
  declarations or consume job result candidates without bypassing lower
  authorities
- durable saved job queries, job cursors, delivery continuations, and
  import/exportable job artifacts once Store support is present
- blob/media-backed job result delivery through query-shaped handles rather
  than ad hoc file plumbing

Query should be able to say "enqueue this work," "show me this work debt,"
"subscribe to this job graph," or "inspect this result candidate" through one
developer-facing surface. Jobs remains the authority for work lifecycle meaning.

#### Server delivery, sessions, and integration runtime

WORTH Server is not just the HTTP layer. It is the network-facing runtime that
owns delivery semantics, sync protocol, durable server-side subscriptions,
transport sessions, typed middleware, auth/policy enforcement, file transfer,
view-shaped patch delivery, freshness negotiation, and protocol diagnostics.
Jobs must treat Server as the authority for making work visible and actionable
over the network.

Jobs should explicitly integrate with Server for:

- job progress and result delivery through typed delivery classes:
  authoritative-ordered, replaceable-latest-state, coalescible-region,
  ephemeral-presence, and advisory-hint
- invalidation-lane versus patch-lane delivery for job state, large results,
  and stale-work notifications
- freshness modes for job subscriptions, including strict live progress,
  coalesced progress, invalidate-only background work, pull-on-focus, and
  advisory-only status
- server-side outboxes and coalescing so slow, disconnected, or backgrounded
  clients do not redefine job lifecycle truth
- basis negotiation and compound catchup when a client resumes observing a job,
  work graph, backfill, or work-debt view
- anti-entropy reconciliation when durable cursors are gone but the client can
  prove a local state basis
- shared subscription bases and server-side view materialization for hot job
  dashboards, collaborative work views, and high-fanout progress surfaces
- view-shaped patch delivery for job tables, grouped/kanban work boards,
  timelines, charts, inspectors, and detail-progress surfaces
- typed middleware and policy pipeline enforcement for job enqueue, cancel,
  retry, pause, drain, inspect, approve, and publish operations
- subscription-scoped authorization so permission changes affect existing job
  observers before delivery leaks unauthorized progress or result metadata
- branch-aware and tenant-scoped job reads, subscriptions, mutations, and
  delivery
- background HTTP delivery, webhooks, scheduled pull, and serverless consumers
  for integrations that cannot hold a WebSocket
- file upload/download and streaming-response paths for job inputs, large
  exports, media artifacts, logs, evidence bundles, and partial outputs
- optimistic mutation protocol for user-facing job controls that stage locally
  and settle against canonical job lifecycle outcomes
- distributed saga and outbox integration for external APIs where a server
  request creates local truth plus asynchronous external work
- first-party and extensible integration wiring for external callbacks,
  polling, OAuth events, billing events, and webhook ingestion that become
  schema-validated truth or job declarations
- priority-based load shedding and adaptive delivery degradation so low-value
  advisory work does not starve authoritative mutations or live interactive
  delivery
- workspace-affinity routing, edge read replicas, control-plane invalidation,
  and tenant-scoped delivery when job progress and work debt are served across
  a distributed server fleet

Server owns the network contract. Jobs owns the work contract. The integration
point is that every job-facing server operation should lower into admitted job
declarations, lifecycle transitions, result candidates, effect receipts, or
evidence queries, and every server delivery should preserve the job's basis,
trust class, policy basis, tenant basis, and lifecycle meaning.

#### Certification matrix discipline

Store, Bridge, and Query all require machine-checkable certification bundles
instead of local happy-path tests. Jobs should adopt the same discipline from
the beginning. Every admitted job capability should have:

- a canonical artifact boundary
- hostile replay or restart proof
- runtime-backed versus store-backed parity where both exist
- typed unsupported/debt posture where support is incomplete
- counters for admission, execution, fallback, denial, retry, publication, and
  repair breadth
- certification coverage across generic and domain workloads

If a future Jobs roadmap names a capability without naming its artifact
boundary, authority owner, proof obligation, and store/bridge/query dependency,
the roadmap is incomplete.

## Principles

1. A job is durable intent, not a callback.
2. Every admitted job binds to an explicit basis.
3. Ordinary queue, schedule, workflow, and backfill mechanics are foundational,
   not optional integrations.
4. Workers consume lowered packets, not ambient application state.
5. Result publication belongs to the authority that owns the target surface.
6. Idempotency identities must encode semantic sameness, not request trivia.
7. Staleness is a typed topology, not a boolean.
8. Queue order is never business order unless a job family explicitly declares
   that order as its basis.
9. Cancellation is a lifecycle, not a flag.
10. Partial, degraded, advisory, approximate, speculative, and exact results are
   different authority classes.
11. Resource scheduling must understand meaning, deadlines, tenants, locality,
    and scarcity instead of only priority.
12. Human gates are part of work, not an external ticketing afterthought.
13. External side effects require intents, receipts, and reconciliation.
14. Work debt is queryable product state.
15. Repair and reconciliation use the same job protocol as primary work.
16. Every job decision must be explainable from canonical evidence.

## Foundational Decisions

These are locked architectural decisions:

- job families are declared capabilities, not string task names
- queue entries, scheduled runs, workflow runs, task attempts, backfill ranges,
  leases, and dead-letter records are first-class runtime artifacts
- scheduler triggers and workflow graph expansion produce declarations; they do
  not bypass admission
- execution adapters are replaceable workers, not owners of job semantics
- job declaration, admission, lease, execution, result submission, publication,
  cancellation, and repair are phase-typed lifecycles
- every admitted job carries a basis identity and publication contract
- every worker receives a bounded `WorkPacket` or equivalent lowered artifact
- dynamic reads inside jobs require declared read classes and receipts
- reactive readiness, deferred reaction, resource waits, approval waits, and
  downstream evidence waits should use Signal conditional-node semantics where
  the problem is condition-gated reaction
- job outputs begin as result candidates, never as published authority
- publication uses compare/admit/publish witnesses owned by the target authority
- truth-mutating job outputs lower through relational strategy, merge, or
  validated commit authority instead of publishing directly from worker code
- job admission and publication may consume extensible invariant witnesses, but
  invariant authority remains relational and domain-owned
- exact current publication, historical-only retention, supersession, rebase,
  degradation, and rejection are distinct outcomes
- external side effects are ledgered as effect intents and effect receipts
- resource calendars represent scarce digital, human, and physical resources
- budgets are first-class admission and scheduling inputs, not merely billing
  metadata or operational dashboards
- physical-world bindings must be first-class for work tied to locations,
  equipment, people or teams, inventory lots, time windows, custody chains,
  hazard classes, or perishable classes
- elastic compute and capacity-scaling intents must be typed scheduling
  outputs that Cloud or deployment schedulers can act on without changing job
  semantics
- job evidence is canonical enough for replay, audit, certification, and support
- job lifecycle records, work packets, attempts, leases, receipts, progress,
  result candidates, dead letters, and evidence bundles are persisted through
  `worth-store` rather than through an ad hoc queue database
- job artifacts must declare Store-style authority posture and, where derived,
  accuracy class; exact, conservative, approximate, heuristic, advisory,
  ephemeral, and rebuildable artifacts may not be confused
- durable, embedded, and absent operating modes must be explicit for job
  execution, persistence, recovery, and resume claims
- retention, compatibility, replication, capsule export/import, repair,
  quarantine, and forensic recovery must produce typed job-artifact and
  job-resume conclusions instead of forcing operators to infer them
- Store maintenance, tiering, admission-control, budget, blob, and repair
  surfaces may feed job admission and scheduling, but their domain authority
  remains Store-owned
- job triggers, readiness predicates, deferred reactions, and downstream
  invalidations may use `worth-signal`, but signal execution remains the
  derived-computation authority rather than the job authority
- truth-to-job and job-to-derived-state causality may pass through
  `worth-runtime-bridge` when basis, subscription, routing, replay, or
  continuation meaning matters
- bridge-integrated job flows must consume family-aware bridge artifacts for
  patch routing, source contracts, change streams, writeback, policy
  propagation, preview flows, and subscriptions rather than raw host events
- `worth-query` may provide the developer-facing enqueue, inspect, filter,
  subscribe, and work-debt APIs, but query meaning does not become job
  execution meaning
- Query-facing job APIs must lower into canonical job declarations, job
  predicates, work-debt result shapes, policy/tenant admission metadata,
  saved job queries, durable job cursors, and blob-backed result handles
  without inventing a second job runtime
- `worth-server` may expose job progress, result delivery, subscriptions,
  upload/download channels, and resumable client sessions, but transport
  delivery does not become job lifecycle authority
- Server-facing job operations must lower through the typed middleware,
  policy, auth, tenant, and branch-scoping pipeline before they enqueue,
  mutate, cancel, approve, retry, or publish job lifecycle state
- job delivery through Server must preserve delivery class, freshness mode,
  basis negotiation, cursor/catchup semantics, view shape, authorization
  scope, and protocol provenance instead of degrading into generic pub/sub
- large job inputs, exports, evidence bundles, media outputs, and partial
  artifacts use Server's file, streaming, upload/download, and blob-handle
  surfaces rather than polluting the sync channel
- WORTH Cloud may host and scale worker fleets, but `WORTH-jobs` remains the
  reusable runtime contract underneath

## Capability Pillars

### Queue And Scheduler Foundation

#### Ordinary queue lifecycle

Technical role:
`WORTH-jobs` must still provide the expected worker substrate: enqueue, delayed
enqueue, schedule, lease, heartbeat, timeout, retry, backoff, pause, resume,
drain, cancel, dead-letter, archive, purge, and inspect.

What this enables:

- ordinary background work is not forced into a separate queue system
- teams can migrate from Sidekiq, Celery, BullMQ, SQS workers, or custom queues
  without losing basic operational vocabulary
- WORTH-specific proof layers sit on top of a complete worker substrate rather
  than replacing missing fundamentals with theory
- Cloud can operate workers, queues, and schedules with familiar controls

#### Schedule and trigger model

Technical role:
Jobs may be submitted by direct request, delayed request, recurring schedule,
cron-like calendar, event trigger, dataset/artifact update, subscription event,
manual operator action, or dependency completion.

What this enables:

- ordinary cron-style automation
- Airflow-style dataset-triggered pipelines
- event-driven work from commits, blobs, subscriptions, or external receipts
- missed-run catchup and schedule backfill
- explicit distinction between scheduled intent and execution admission

#### Retry, timeout, and dead-letter semantics

Technical role:
Retry behavior is a declared family policy with typed failure classes. Timeouts
distinguish lease timeout, execution timeout, external-effect timeout,
freshness expiration, and publication timeout. Dead-letter records carry the
semantic reason the runtime stopped trying.

What this enables:

- transient infrastructure failures retry differently from authority rejection
- poison jobs stop consuming capacity
- retry storms can be bounded by policy and resource budgets
- operators can repair, rebase, replay, or intentionally abandon dead-lettered
  work with context

#### Queue operations and operational controls

Technical role:
Operators need safe controls: pause a family, pause a tenant, pause a resource,
drain a queue, replay a range, quarantine a family, cancel superseded work,
resume after repair, and inspect backlog by semantic class.

What this enables:

- incident response without ad hoc SQL
- tenant-local blast-radius control
- rollout and rollback safety for worker fleets
- controlled backfills and maintenance windows
- support workflows that understand work meaning

### Workflow And DAG Architecture

#### Work graphs

Technical role:
`WORTH-jobs` must support workflow graphs made of tasks, dependencies,
barriers, joins, fanout, mapped tasks, dynamic expansion, nested workflows, and
graph-level state. The graph is an orchestration artifact; each task still
adheres to job family admission and publication rules.

What this enables:

- Airflow-style pipelines without losing WORTH proof semantics
- imports, exports, backfills, ETL, media pipelines, AI pipelines, and
  simulation workflows can run as coherent graphs
- graph-level cancellation, replay, backfill, and repair remain structured
- dynamic task mapping can expand from admitted data without ambient scans

#### Operators and adapters

Technical role:
The runtime distinguishes orchestration semantics from execution adapters.
An operator describes the work contract; an adapter performs the concrete
execution against a local process, container, VM, external API, plugin sandbox,
GPU worker, or human gate.

What this enables:

- domain work can run on many execution substrates without changing meaning
- Cloud can host workers directly, through containers, or through specialized
  pools
- external systems can participate as adapters without defining job protocol
  semantics
- testing can certify operator semantics independently from adapter variation

#### Backfill and catchup

Technical role:
Backfill is first-class. A backfill declares the range, basis window, schedule
window, artifact family, tenant or branch scope, resource budget, and catchup
policy before execution expands into tasks.

What this enables:

- large historical repairs do not run as dangerous scripts
- missed scheduled runs can catch up under explicit policy
- backfills can be simulated, paused, resumed, and audited
- date-window pipelines, migration ranges, and artifact rebuild ranges share
  one bounded work model

#### Workflow state and lineage

Technical role:
Workflow runs, task attempts, mapped task expansions, barriers, joins, retries,
skips, branch decisions, and graph-level outputs are canonical state with
lineage. A downstream task depends on upstream evidence, not merely completion.

What this enables:

- failed workflows can be replayed or repaired without losing semantic context
- skipped tasks remain explainable
- mapped tasks can be compared by input partition and basis
- workflow lineage becomes queryable alongside ordinary job evidence

### Work Declaration Architecture

#### Job families

Technical role:
Every admitted kind of work is represented by a job family. A family declares
the basis it can bind to, the input packet it lowers, the resources it may
consume, the retry and cancellation topology it supports, the result candidate
shape it returns, and the authority boundary that may publish it.

What this enables:

- feature teams register work without inventing shadow queue protocols
- domains can add media, supply-chain, analysis, export, AI, and repair work
  safely
- unsupported work fails at admission instead of becoming an opaque payload
- certification can prove family-specific behavior rather than one generic
  queue happy path

Named vocabulary:

- `JobFamily`
- `JobFamilyCapability`
- `JobFamilyRegistration`
- `BasisRule`
- `ResourceRule`
- `OutputArtifactClass`
- `SideEffectPolicy`
- `RetryPolicy`
- `CertificationLane`
- `JobFamilyBasisRules`
- `JobFamilyResourceRules`
- `JobFamilyOutputContract`
- `JobFamilySideEffectPolicy`
- `JobFamilyRetryPolicy`
- `JobFamilyCertificationLane`
- `InstallableJobFamily`

#### Durable work intent

Technical role:
A job declaration records why work should happen before it is admitted. The
declaration carries work family, desired target, declared basis request,
business impact, resource hints, deadline/freshness requirements, and desired
publication class.

What this enables:

- operators can answer why work exists without reading worker logs
- duplicated work can be compared before execution
- low-value speculative work can be discarded before consuming scarce resources
- product surfaces can expose pending work as real state

#### Semantic idempotency

Technical role:
Idempotency identity is derived from job family semantics: basis identity,
input digest, target authority, output contract, side-effect class, and declared
publication meaning.

What this enables:

- duplicate workers cannot accidentally publish duplicate effects
- retried work can converge without weak request-id heuristics
- older equivalent work can satisfy newer demand when the family admits reuse
- idempotency remains meaningful across deploys and replay

### Basis And Validity Architecture

#### Basis-bound admission

Technical role:
Admission binds a job declaration to an explicit truth, artifact, branch,
snapshot, subscription, blob, or external-effect basis. The admitted job carries
the proof of what was valid when execution became legal.

What this enables:

- workers never need to infer what "current" meant at enqueue time
- publication can classify basis drift exactly
- branch-local and preview-scoped jobs remain isolated from authoritative lanes
- repair, replay, and audit can reconstruct the same admission conclusion

#### Branch-composed work

Technical role:
Jobs should compose with WORTH branches as a native basis form. Work can run
against preview branches, speculative branches, migration branches, creative
branches, AI proposal branches, or historical snapshots. If the branch is
promoted, outputs may be revalidated and promoted through authority-owned
publication. If the branch is discarded, job outputs leave zero authoritative
residue unless an explicit historical-retention rule admits them.

Named vocabulary:

- `BranchJobBasis`
- `PreviewJobBasis`
- `SpeculativeJobBasis`
- `BranchPromotionRevalidation`
- `BranchDiscardResidueCheck`
- `HistoricalOnlyJobOutput`

What this enables:

- preview mix renders, proposed supply plans, design alternatives, schema
  migration dry runs, and AI transformations can run without touching current
  truth
- branch promotion can reuse valid work instead of recomputing blindly
- branch discard can cleanly retire speculative work and partial outputs
- job replay can explain which branch or snapshot made work meaningful

#### Freshness and value decay

Technical role:
Jobs may declare freshness windows, expiration policies, latest-useful-by
times, and value-decay curves. The runtime can decide whether work is still
worth doing before execution, during leasing, or at publication.

What this enables:

- previews and AI suggestions can be skipped when the user moved on
- supply-chain jobs can understand ordering windows and perishable urgency
- media renders can avoid wasting work on superseded edits
- resource pressure can choose high-value current work over stale low-value
  backlog without relying on crude priority alone

#### Typed staleness outcomes

Technical role:
Staleness is classified by the job family and publication contract:
`StillValid`, `HistoricalOnly`, `Superseded`, `RebaseRequired`,
`DegradedButUseful`, `RejectedBasisDrift`, or equivalent typed outcomes.

What this enables:

- stale results do not silently overwrite current artifacts
- useful historical or preview results can still be retained honestly
- rebaseable work can be reissued without full operator intervention
- product UI can explain degradation rather than showing binary failure

### Execution Packet Architecture

#### Lowered work packets

Technical role:
The executor consumes a packet whose inputs, dynamic read permissions, budgets,
resource leases, and output contract were pre-resolved at admission and
lowering time.

What this enables:

- workers do not rediscover semantic context
- dynamic reads require receipts and become visible inputs
- hot execution can stay narrow, deterministic, and bounded
- replay can compare the actual packet rather than reconstructing it from logs

#### Dynamic read receipts

Technical role:
When a job family allows dynamic reads, each read produces a receipt bound to
the admitted job, read class, basis, and observed artifact identity.

What this enables:

- hidden ambient reads become impossible to ignore
- publication can reject results that consumed undeclared or drifted inputs
- debugging can answer what the worker really depended on
- expensive read surfaces can be counted and budgeted

#### Worker leases and heartbeats

Technical role:
Leases are authority to attempt execution, not authority to publish. A worker
may hold a lease, renew it, lose it, observe cancellation, submit progress, and
submit result candidates under strict lifecycle states.

What this enables:

- lost workers can be recovered without guessing whether result publication was
  legal
- lease loss and late result submission become typed outcomes
- heartbeat failure does not imply domain failure
- duplicate execution can be contained by idempotency and publication witnesses

### Publication Architecture

#### Result candidates

Technical role:
Workers return result candidates. A result candidate is not visible truth. It
is proposed output plus evidence, basis, dynamic read receipts, resource
accounting, and publication contract.

What this enables:

- workers cannot become shadow authority
- authorities can validate candidates before committing or publishing
- candidates can be compared, superseded, degraded, or quarantined
- exact and approximate outputs remain separate by construction

#### Publication witnesses

Technical role:
The target authority constructs the witness that makes a result visible:
truth commit witness, store artifact publication witness, bridge protocol
publication witness, query artifact reload witness, external-effect receipt, or
domain-specific authority witness.

What this enables:

- partial worker side effects cannot masquerade as published results
- publication failures localize to the owning authority
- replays can prove why one candidate published and another did not
- Store, Query, Bridge, Relational, and Cloud keep their authority boundaries

#### Supersession records

Technical role:
When newer work, newer basis, better evidence, or a stronger result class makes
older work obsolete, the runtime records a supersession relationship rather
than treating the older job as ordinary failure.

What this enables:

- out-of-order completion stops being dangerous
- UIs can show that old work was replaced, not broken
- expensive duplicate work can be suppressed safely
- repair/audit can reason about lineage of work artifacts

### Result Trust Architecture

#### Exact, degraded, advisory, approximate, and speculative results

Technical role:
Job outputs carry explicit trust and authority classes. Exact outputs may
publish where the authority admits them. Degraded or approximate outputs can be
visible only through surfaces that acknowledge their class. Advisory and
speculative outputs never become authoritative without explicit promotion.

What this enables:

- low-quality previews can appear quickly without lying
- AI recommendations stay advisory unless promoted
- supply-chain forecasts can be consumed with confidence bounds
- analysis artifacts can be exact for their declared basis while non-authority
  for truth

Named vocabulary:

- `ExecutionMode::Exact`
- `ExecutionMode::Approximate`
- `ExecutionMode::Advisory`
- `ExecutionMode::Speculative`
- `ExecutionMode::Preview`
- `ExecutionMode::BestEffort`

#### Partial result publication

Technical role:
Long-running work may emit partial results with segment, quality, confidence,
and lifetime metadata. Partial visibility is governed by the family result
contract.

What this enables:

- waveform previews, render tiles, stem separation chunks, import progress, and
  analysis partitions become usable before full completion
- partial results cannot be confused with final output
- cancellation can preserve useful partial artifacts where the family allows it
- user-facing progress becomes meaningful product state instead of percent bars

Named vocabulary:

- `PartialResult::Preview`
- `PartialResult::LowQuality`
- `PartialResult::SegmentComplete`
- `PartialResult::ConfidenceBounded`
- `PartialResult::UsableUntilFinal`
- `PartialResult::FinalSuperseded`

#### Result lifetimes

Technical role:
Results declare residency and lifetime: authoritative commit, durable artifact,
rebuildable cache, session ephemeral, preview-only, historical-only, or external
effect receipt.

What this enables:

- Store can retain or reclaim outputs according to real meaning
- Cloud can expose which results survive restart or branch discard
- derived artifacts remain rebuildable and non-authoritative
- product surfaces can choose exact, stale, partial, or absent states honestly

Named vocabulary:

- `ResultResidency::AuthoritativeCommit`
- `ResultResidency::DurableArtifact`
- `ResultResidency::RebuildableCache`
- `ResultResidency::SessionEphemeral`
- `ResultResidency::PreviewOnly`
- `ResultResidency::HistoricalOnly`
- `ResultResidency::ExternalEffectReceipt`

#### Degradation instead of binary failure

Technical role:
Jobs should expose degradation as a first-class outcome rather than forcing
all non-perfect states into failure. A result can be exact, stale but usable,
degraded, served from fallback, waiting for manual intervention, or currently
unavailable.

Named vocabulary:

- `AvailabilityClass::Exact`
- `AvailabilityClass::StaleButUsable`
- `AvailabilityClass::Degraded`
- `AvailabilityClass::Fallback`
- `AvailabilityClass::ManualInterventionRequired`
- `AvailabilityClass::Unavailable`

What this enables:

- web apps can show cached data with refresh pending
- supply-chain systems can use yesterday's forecast while flagging ordering
  risk
- creative tools can show a lower-quality preview while the high-quality render
  runs
- operators can distinguish "bad result" from "usable under declared limits"

### Resource And Scheduling Architecture

#### Resource calendars

Technical role:
Resources are first-class schedulable objects: CPU pools, GPU slots, worker
families, object-store bandwidth, database pressure, external API quota, plugin
licenses, warehouse docks, vehicles, devices, human reviewers, or specialized
equipment.

What this enables:

- jobs can reserve the scarce thing they actually need
- worker scheduling can model physical and human constraints as cleanly as CPU
- supply-chain and media workflows fit the same runtime as web jobs
- priority inversion can be detected as a resource conflict, not guessed from
  queue latency

Named vocabulary:

- `ResourceReservation`
- `LeaseWindow`
- `CapacityClass`
- `ConflictPolicy`
- `ResourceCalendar`
- `ResourceAvailability`
- `ResourceConflict`

#### Business impact and deadline contracts

Technical role:
Jobs declare business impact, deadline shape, freshness window, expiration
behavior, and escalation policy. Priority is only one derived scheduling input.

What this enables:

- revenue-blocking, user-visible, compliance-critical, perishable, speculative,
  and internal-optimization work can be scheduled differently
- stale low-value work can be abandoned under pressure
- high-impact work can preempt low-impact work without hard-coded queue names
- Cloud can offer product-level work guarantees instead of generic worker
  throughput

#### Budget and admission strategies

Technical role:
Jobs need explicit budget strategies for money, compute, storage, human time,
physical capacity, subscription resumability, artifact maintenance, blob recall,
repair effort, tenant blast radius, and branch speculation. A budget is not
only a spend cap. It is an admission contract that says which work may exist,
which work may wait, which work may degrade, which work may borrow capacity,
and which work must be rejected before it damages the system.

Named vocabulary:

- `JobBudget`
- `TenantWorkBudget`
- `ResourceBudget`
- `ComputeBudget`
- `ArtifactBudget`
- `BlobRecallBudget`
- `SubscriptionResumeBudget`
- `RepairBudget`
- `SpeculationBudget`
- `BudgetAdmission`
- `BudgetDenial`
- `BudgetBorrow`
- `BudgetPaydown`
- `BudgetEscalation`

What this enables:

- WORTH can refuse dangerous backfills before they create operational debt
- Cloud can distinguish tenant spend limits from infrastructure safety limits
- subscription support artifacts, derived indexes, previews, repairs, and blob
  recalls can compete under declared policy rather than hidden queue pressure
- product teams can choose exact, degraded, delayed, or speculative work based
  on business meaning instead of generic priority numbers

#### Elastic compute and dynamic capacity

Technical role:
Some admitted work should be able to request more power, different power, or a
different placement class as its real execution profile becomes known. The job
runtime should expose scaling intents and capacity feedback without becoming
WORTH Cloud's autoscaler. Cloud or a deployment-specific scheduler can then
turn those typed intents into more workers, more GPUs, warmer blob placement,
larger plugin-license pools, or alternate regional execution.

Named vocabulary:

- `ComputeScalingIntent`
- `CapacityShortfall`
- `CapacityExpansionRequest`
- `ElasticWorkerPool`
- `AutoscalePolicyHint`
- `ScaleUpAdmission`
- `ScaleDownRelease`

What this enables:

- render, AI, analysis, and import jobs can ask for more capacity without
  embedding cloud-provider logic in workers
- resource starvation becomes a typed scheduling condition instead of a queue
  mysteriously getting slow
- Cloud can dynamically scale workers based on job-family meaning, deadline,
  tenant policy, data locality, and trust class
- deployments can run static, manually approved, or fully elastic capacity
  policies without changing job semantics

#### Placement-aware execution

Technical role:
Jobs reason about where inputs live and where execution should happen. A job may
need blob recall, hot-tier artifact placement, regional data locality, GPU
placement, or branch-local working-set locality before execution is profitable.

What this enables:

- WORTH Store tiering and blob placement become scheduling inputs
- Cloud can move compute to data or data to compute intentionally
- cold artifact recall cost is visible before the job starts
- large media, CAD, and simulation jobs avoid surprise cross-region or
  cross-tier work

Named vocabulary:

- `PlacementRequirement`
- `DataLocality`
- `BlobRecallRequirement`
- `ComputeLocality`
- `RegionAffinity`
- `PlacementCost`
- `RecallBeforeExecution`

### Human And Physical Work Architecture

#### Human gates

Technical role:
Approval, review, exception handling, two-person rules, manual repair, and
timeboxed escalation are lifecycle states in the job runtime.

What this enables:

- business workflows do not need a separate ticketing shadow system
- supply-chain exceptions, billing review, migration approvals, and creative
  approvals are represented as real work states
- timeouts and escalations are typed outcomes
- human decisions can be replayed as part of job evidence

#### Physical-world bindings

Technical role:
Jobs may bind to locations, equipment, inventory lots, custody chains,
perishable classes, safety classes, and scheduled physical windows.

What this enables:

- warehouse, manufacturing, lab, logistics, and field-service workflows can use
  the same job protocol
- physical constraints become part of admission and scheduling
- missed deadlines and degraded freshness have real domain meaning
- digital and physical work can be composed in one evidence model

Named vocabulary:

- `PhysicalWorkBinding`
- `PhysicalLocation`
- `EquipmentBinding`
- `PersonOrTeamBinding`
- `InventoryLotBinding`
- `PhysicalTimeWindow`
- `ChainOfCustody`
- `HazardClass`
- `PerishableClass`

#### Decision jobs

Technical role:
Some jobs produce decisions rather than artifacts. A decision job returns
candidates, evidence, confidence, policy basis, selected action, and optional
human override path.

What this enables:

- AI and heuristic outputs remain inspectable and advisory unless promoted
- supply-chain substitute/expedite/cancel decisions become auditable
- creative tooling can propose edits without seizing authority
- policy-driven automation can be reviewed and replayed

### Side Effect Architecture

#### Effect intents and receipts

Technical role:
External side effects are represented as intents, dispatch attempts, receipts,
acknowledgments, reconciliation records, and compensation requirements.

What this enables:

- email, webhook, payment, shipment, upload, vendor API, and plugin calls are
  not invisible worker side effects
- retries can respect real external idempotency semantics
- partial external failure becomes auditable protocol state
- repair jobs can reconcile against receipts instead of logs

Named vocabulary:

- `EffectIntent`
- `EffectDispatch`
- `EffectReceipt`
- `EffectAcknowledgment`
- `EffectReconciliation`
- `EffectCompensation`
- `ExternalIdempotencyKey`

#### Outbox and inbox equivalence

Technical role:
WORTH Jobs should support outgoing and incoming external work symmetrically:
dispatching effects out, receiving callbacks or external events in, reconciling
receipts, and binding them to the same work evidence.

What this enables:

- webhook delivery and webhook receipt share one correctness model
- payment/provider/vendor callbacks can be matched to exact effect intents
- external systems can participate without becoming authority
- incident investigation has one causal chain

### Dependency And Work Graph Architecture

#### Evidence dependencies

Technical role:
Job dependencies target evidence, not only job ids. A downstream job can depend
on a specific artifact published by a family against a basis with a required
trust class and no superseding artifact.

What this enables:

- DAGs become semantic work graphs
- downstream work does not run merely because upstream work "completed"
- stale or degraded upstream output can block, rebase, or alter downstream work
- complex pipelines remain explainable under replay

#### Work simulation

Technical role:
Before running a job graph, the runtime can simulate expected resource use,
deadline misses, basis drift risk, publication dependencies, side effects,
tenant impact, and cost posture.

What this enables:

- dangerous backfills can be inspected before launch
- supply-chain planners can compare work plans
- media/export systems can preview cost and completion risk
- Cloud can admit or deny large work graphs before they damage live service

Named vocabulary:

- `WorkSimulation`
- `SimulatedResourceUse`
- `SimulatedDeadlineMiss`
- `SimulatedTenantImpact`
- `SimulatedStalenessRisk`
- `SimulatedCost`
- `DryRunJobGraph`
- `AdmissionForecast`

#### Work debt

Technical role:
Delayed, stale, degraded, unprocessed, superseded, or repair-required work is
tracked as queryable debt with owner, basis, impact, age, and paydown options.

What this enables:

- operators see what the system owes, not only what failed
- product UI can show stale-but-usable states with refresh pending
- background maintenance becomes governable
- Cloud can enforce budgets and escalation over real debt, not generic backlog

Named vocabulary:

- `WorkDebt`
- `DebtAge`
- `DebtImpact`
- `DebtOwner`
- `DebtPaydownPlan`
- `DebtEscalation`
- `DebtClass`
- `DebtRepairOption`

#### Queryable and subscribable work

Technical role:
Jobs should be queryable and subscribable as product state, not only observable
through dashboards. A product should be able to ask for work by project, order,
artifact, tenant, branch, resource, person, approval lane, debt class, result
trust class, or physical-world binding, and then subscribe to meaningful
changes through Query and Server without rebuilding job-specific polling.

Named vocabulary:

- `JobQuery`
- `WorkDebtQuery`
- `JobSubscription`
- `JobGraphSubscription`
- `ResourceReservationQuery`
- `HumanApprovalQuery`
- `StaleArtifactQuery`
- `BlockingWorkQuery`

What this enables:

- subscribe to jobs affecting this project
- query all work blocking this order
- show stale artifacts for this album
- show pending human approvals for this warehouse
- show resource reservations for a GPU cluster
- make jobs part of real product UX rather than back-office metrics

#### Job time travel

Technical role:
WORTH should replay work decisions, not only truth decisions. Job evidence must
answer why work was scheduled, whether it would be scheduled now, why a worker
published, what changed after execution started, and what would have happened
if the job had been cancelled, rebased, degraded, or denied.

Named vocabulary:

- `JobTimeTravel`
- `WorkDecisionReplay`
- `SchedulingReplay`
- `PublicationReplay`
- `CancellationSimulation`
- `BasisDriftTimeline`
- `WorkerAttemptTimeline`

What this enables:

- incident debugging for web systems
- audit and compliance for supply chain and finance
- creative history for music/media tools
- certifiable answers to "why did this async thing happen?"

### Observability And Certification Architecture

#### Job evidence bundles

Technical role:
Each job emits canonical evidence: declaration, admission, basis, packet,
lease, progress, dynamic read receipts, result candidates, publication outcome,
side-effect receipts, cancellation, supersession, resource use, and counters.

What this enables:

- support can answer why work ran and why it published
- replay can compare original and reconstructed meaning
- certification can prove job families under hostile conditions
- Cloud observability can expose semantic state instead of log fragments

#### Failure taxonomy

Technical role:
Failures are typed by meaning: transient infrastructure, dependency outage,
poison input, basis drift, authority rejection, incompatibility, budget denial,
resource starvation, deadline expiration, cancellation race, duplicate
suppression, external-effect uncertainty, nondeterminism, or quarantine.

What this enables:

- retry policy can be correct by construction
- permanent semantic rejections stop retrying forever
- operational alerts distinguish infrastructure from domain invalidity
- repair jobs can target the actual failure class

#### Certified queues

Technical role:
A queue or worker fleet should be able to prove that admitted work stayed within
declared contracts: no direct authority publication, no stale overwrite, no
cancelled authoritative residue, no duplicate side effect, no undeclared input
read, no tenant budget breach, and no unsupported compatibility execution.

What this enables:

- WORTH Cloud can offer trust-grade worker infrastructure
- domain products can rely on shared job guarantees
- high-risk domains can audit asynchronous work mechanically
- job runtime correctness becomes certifiable, not assumed

## Domain Fit

### Web Applications

`WORTH-jobs` should support:

- imports, exports, emails, webhooks, notifications, billing work, search
  indexing, thumbnail generation, AI assists, tenant migrations, and backfills
- semantic idempotency for every externally visible operation
- basis-bound stale-result rejection for user-visible artifacts
- typed retries and dead-letter decisions
- queryable progress and work debt

Revolutionary use:
web developers stop writing bespoke queue tables, stale-write guards,
idempotency ledgers, retry policies, progress models, side-effect ledgers, and
backfill runners for every feature. They declare work meaning and let the
platform handle the execution protocol.

### Supply Chain, Logistics, And Physical Operations

`WORTH-jobs` should support:

- resource calendars for docks, vehicles, equipment, workers, inventory lots,
  cold storage capacity, and vendor API quota
- freshness windows and value decay for forecasts, replenishment, routing, and
  perishable goods
- human exception review and escalation
- decision jobs for expedite, substitute, split, defer, or cancel choices
- chain-of-custody and physical-location bindings

Revolutionary use:
operations systems can treat real-world work as proof-bearing, schedulable,
queryable platform state instead of external workflow glue around a database.

### Music, Media, And Creative Tools

`WORTH-jobs` should support:

- waveform analysis
- stem separation
- preview renders
- high-quality exports
- plugin scans and sandboxed plugin execution
- freeze/bounce artifacts
- region or track-local partial results
- branch-local mix and design previews

Revolutionary use:
creative tools can show partial and degraded previews quickly while preserving
exact publication boundaries for final renders. Older renders cannot overwrite
newer edits, plugin output can be receipt-bound, and branch-local creative work
can be discarded with zero authoritative residue.

### Geometry, CAD, Chip, And Simulation

`WORTH-jobs` should support:

- long-running mesh, solver, timing, DRC, export, and analysis jobs
- exact branch/snapshot basis binding
- resumable partial computation
- scarce resource scheduling for GPUs, solver licenses, and region-local data
- result publication through Store or domain authority witnesses
- historical-only retention for superseded analysis artifacts

Revolutionary use:
technical computing systems can run expensive analysis and derived artifact
generation asynchronously without turning the analysis cache into shadow truth.

### AI Systems

`WORTH-jobs` should support:

- speculative branch-local work
- advisory and confidence-bounded result classes
- human-gated promotion of AI decisions
- basis-bound context packets
- nondeterminism detection and quarantine
- reusable derived analysis artifacts with explicit trust classes

Revolutionary use:
AI agents can perform long-running work, propose actions, and publish derived
artifacts without smuggling mutable ambient context or silently promoting
heuristics into truth.

### WORTH Cloud

`WORTH-jobs` should support:

- hosted worker fleets
- tenant-aware budgets
- job-family capability registration
- resource and quota policy
- Cloud-visible job state, debt, and evidence
- branch-aware, subscription-aware, and artifact-aware orchestration
- certification of shared worker infrastructure

Revolutionary use:
WORTH Cloud can replace the usual stack of queue, worker platform, retry
tables, idempotency stores, cron, dashboards, backfill scripts, and operational
runbooks with one truth-aware work operating environment.

## Roadmap Direction

This file is a vision document, not the execution roadmap. But the future work
should be derivable from it.

The highest-signal job programs are:

- ordinary queue lifecycle primitives and operational controls
- schedule, trigger, catchup, and recurring-run semantics
- Signal conditional-node integration for job readiness, deferred reaction,
  resource waits, approvals, and downstream evidence gates
- workflow/DAG runs, task attempts, mapped tasks, barriers, joins, and graph
  state
- operator contracts and execution adapter boundaries
- backfill range planning, simulation, pause, resume, and repair
- job family declaration and sealed admission lifecycles
- basis-bound work declarations and lowered execution packets
- relational strategy, merge, and validated commit handoff for truth-mutating
  result candidates
- extensible invariant witness consumption for job admission, publication,
  repair, certification, and domain-specific safety gates
- Store operating-mode contracts for durable, embedded, and absent job
  execution with explicit recovery and resume claims
- Store artifact classification and accuracy taxonomy for lifecycle records,
  support artifacts, checkpoints, partial results, analysis lanes, and
  derived job outputs
- crash-recovery, compatibility, retention, replication, capsule,
  quarantine, salvage, and forensic repair participation for job artifacts
- Store maintenance, tiering, budget/admission-control, blob, and repair
  signals as job admission and scheduling inputs
- bridge family-aware patch triggers, source contracts, change streams,
  writeback declarations, policy propagation, preview flows, and subscription
  lifecycle integration
- Query-facing canonical job declarations, workflow predicates, work-debt
  query shapes, policy/tenant-aware admission, saved job queries, durable
  job cursors, and blob/media-backed result delivery
- Server delivery integration for job progress, work-debt views, result
  delivery, server-side outboxes, basis negotiation, freshness modes,
  compound catchup, typed delivery classes, view-shaped patches, and protocol
  provenance
- Server request-pipeline integration for auth, policy, tenant scoping,
  branch scoping, rate limiting, job control mutations, background HTTP
  delivery, webhooks, streaming responses, file transfer, optimistic controls,
  and external saga/outbox workflows
- semantic idempotency identities
- worker leases, heartbeat, cancellation, and late-result containment
- result candidates and authority-owned publication witnesses
- typed stale, superseded, rebase, degraded, historical-only, and rejected
  outcomes
- partial result and result trust-class surfaces
- named `PartialResult`, `ExecutionMode`, `AvailabilityClass`, and
  `ResultResidency` vocabularies
- external effect intents, receipts, reconciliation, and compensation
- resource calendars and resource reservation
- named `JobBudget`, `TenantWorkBudget`, `ResourceBudget`, `ComputeBudget`,
  `BlobRecallBudget`, `SubscriptionResumeBudget`, `RepairBudget`,
  `SpeculationBudget`, `BudgetAdmission`, and `BudgetDenial` surfaces
- physical-world bindings for location, equipment, people/teams, inventory
  lots, time windows, custody chains, hazard classes, and perishable classes
- elastic compute and capacity-scaling intents for dynamic worker, GPU,
  plugin-license, region, and placement expansion
- business impact, deadline, freshness, expiration, and value-decay contracts
- evidence-based dependency graphs and work simulation
- named `WorkSimulation`, `DryRunJobGraph`, and `AdmissionForecast` surfaces
- queryable work debt
- queryable/subscribable job, work-debt, resource-reservation,
  human-approval, stale-artifact, and blocking-work surfaces
- job time travel, scheduling replay, publication replay, cancellation
  simulation, basis-drift timelines, and worker-attempt timelines
- human-gated work states
- compatibility and code-version admission for long-lived queued work
- placement-aware execution over Store blob/tier/locality surfaces
- canonical job evidence bundles and certification suites
- WORTH Server delivery, resumable client-session, and job-progress protocol
  integration
- WORTH Cloud hosted worker integration

If a capability is named here and not yet built, it is roadmap work.

If a capability is built but not yet proven under basis drift, retry storms,
worker loss, out-of-order completion, cancellation races, tenant pressure,
external-effect uncertainty, and code-version skew, it is certification work.

## Non-Goals

- becoming a general-purpose callback queue with WORTH branding
- owning truth mutation semantics
- owning Store artifact meaning or storage layout
- owning Query expression meaning or result-shape semantics
- owning Bridge subscription protocol semantics
- owning Signal execution semantics
- owning Server sync, auth, delivery, session, or transport semantics
- replacing domain workflow decisions with generic automation
- treating external side effects as rollbackable database writes
- using queue order as business order by default
- hiding worker correctness behind logs and dashboards instead of canonical
  evidence
- forcing every deployment to use WORTH Cloud; the job runtime should remain a
  reusable crate that Cloud can host

## Companion Documents

- [worth_relational_vision.md](../WORTH-relational/WORTH_relational_vision.md)
- [worth_store_vision.md](../worth-store/worth_store_vision.md)
- [worth_runtime_bridge_vision.md](../WORTH-runtime-bridge/WORTH_runtime_bridge_vision.md)
- [worth_query_vision.md](../WORTH-query/WORTH_query_vision.md)
- [worth_server_vision.md](../WORTH-server/WORTH_server_vision.md)
- `WORTH_cloud_vision.md`

The job runtime is where asynchronous work becomes trustworthy. If it is weak,
every product built on WORTH will still need bespoke worker correctness glue:
idempotency tables, stale-write checks, retry taxonomies, progress models,
backfill scripts, side-effect ledgers, and reconciliation jobs. If it is strong,
WORTH turns background work into basis-bound, authority-preserving, replayable
platform behavior.
