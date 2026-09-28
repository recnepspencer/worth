# WORTH Server Future Roadmap

## Purpose

This document defines the future work for `worth-server`.

It is a future-only roadmap. The server vision is already clear, but the
server does not yet have an execution sequence that says what must be built
first, what can safely wait, and which dependency contracts from Query,
Runtime Bridge, Relational, and Store are stable enough to build on now.

The server now has a dedicated certification bar in
[`test-requirements.md`](./test-requirements.md). The acceptance sections in
this roadmap still matter, but the detailed hostile-suite and narrow-artifact
requirements now live there.

This roadmap is not a compressed summary of the vision. It is the execution
outline of the vision. If a meaningful capability appears in the vision, this
roadmap must either:

- assign it to a milestone explicitly
- mark it as a cross-cutting proof obligation across multiple milestones
- or mark it as intentionally later than the current roadmap horizon

If a capability is present in the vision but not locatable in this roadmap,
that is a roadmap defect.

## Governing Source Summaries

- `MENTALITY.md`: the roadmap must solve the hostile server failure mode first,
  not optimize for the easiest demo surface.
- `arch_laws.md`: the server must expose one facade, consume upstream
  contracts, and make wrong layering mechanically difficult rather than
  socially discouraged.
- `composition_laws.md`: request APIs, WORTH-native facade surfaces, sync
  delivery, leases, files, integrations, and diagnostics must remain visibly
  separate responsibilities even when they share a runtime.
- `domain_structure_laws.md`: authoritative truth, derived delivery,
  diagnostics, cache/resume state, integration wiring, and deployment
  topology must not collapse into one "server" bucket.
- `perf_laws.md`: no cheap-looking surface may conceal broad scans, replay
  archaeology, eager materialization, or route-local rediscovery of semantic
  meaning.
- `worth_server_vision.md`: the server owns network delivery operations, not
  runtime meaning, and must be both Query-first and strong enough for
  regulated industries and WORTH-native ergonomics.
- existing `worth_server_roadmap.md`: the previous roadmap had the right
  backbone but compressed too much vision nuance into broad milestones.
- [`test-requirements.md`](./test-requirements.md): the detailed certification
  burden now lives in the dedicated server test-requirements document; this
  roadmap names capability sequencing and milestone acceptance scope.
- `milestone-17-closeout.md`: temporal/async basis, mixed-cause ordering,
  shared delivery, restart/resume basis, and bridge certification are already
  closed upstream and must be consumed rather than recreated.
- `milestone-9.4-closeout.md`: Query closes one runtime-backed downstream
  delivery contract, runtime-backed resume negotiation, remask-aware public
  delivery posture, and mixed-cause public projection, while durable resume
  remains explicit later debt.
- `milestone-9.7-closeout.md`: Query closes shared read authority,
  deterministic submission, typed journal/replay posture, published derived
  artifacts, and hostile concurrent certification. The server must consume
  those concurrency surfaces rather than preserving pre-`9.7` single-borrow
  assumptions.
- `milestone-9.8-closeout.md`: Query closes the downstream consumer kit:
  evidence reports, hard-prohibition registry/audit, support snapshots,
  support pinning, in-memory test backend, and reference-consumer adoption.
  The server and product-facing server tests must consume this kit rather than
  hand-building support or adapter folklore.

## Adversarial Constraint

`worth-server` must survive the hostile case where a naive server would:

- rebuild Query meaning as route-local endpoint folklore
- ship a WORTH-native app surface that still requires handwritten API glue,
  generated-client sprawl, client-owned cache invalidation, and bespoke sync
  logic
- leak cross-tenant, cross-branch, or remasked truth during active delivery
- confuse runtime-backed reconnect with durable restart-stable resume
- treat CDC, transport history, or socket callbacks as the ordinary semantic
  source instead of Query-owned delivery meaning
- fail compliance review because audit posture, residency posture, denial
  evidence, or recovery honesty were bolted on after the fact
- collapse presence, invalidation, patch, binary transfer, integrations, and
  truth delivery into one undifferentiated transport lane
- scale by broad fanout, broad rescans, or per-client recomputation rather
  than explicit basis, shared evaluation, and typed delivery policy

The roadmap exists to make that naive server impossible.

## Current Dependency Contract

The server roadmap now inherits a meaningful upstream baseline:

- `worth-relational` already owns authoritative truth, CDC, history, branches,
  and commit serialization
- `worth-runtime-bridge` Milestone 17 is closed for temporal/async basis,
  completion causality, mixed-cause ordering, shared delivery, restart/resume
  basis, and certification-grade bridge artifacts
- `worth-query` Milestone 9.4 is closed for runtime-backed temporal/async/
  mixed-cause delivery projection, remask-aware downstream posture, and one
  downstream delivery contract meant for `worth-server`
- `worth-query` Milestone 9.7 is closed for real shared-read concurrency,
  deterministic submission, typed journal/replay posture, and hostile
  concurrent certification
- `worth-query` Milestone 9.8 is closed for the consumer kit that downstream
  crates and server certification should use instead of local report,
  support, audit, or test-backend folklore

The important current limitation is also explicit:

- Query ships admitted runtime-backed downstream delivery and runtime-backed
  resume negotiation now
- Query does not yet claim durable downstream resume or store-backed replay as
  closed server-facing contracts

This roadmap therefore starts from a runtime-backed server, consumes the Query
concurrency and consumer-kit closures now available, and promotes durable lease
and resume semantics later instead of pretending those debts are already gone.

## Roadmap Rules

- each milestone must describe a real server capability boundary, not just
  "wire up endpoints"
- each milestone must preserve the server's role as the network delivery
  runtime rather than turning it into a second truth runtime, scheduler, or
  storage engine
- each milestone must consume Query, Bridge, Relational, and Store contracts
  through their public seams rather than rediscovering semantics at the
  network edge
- each milestone must make WORTH-native ergonomics better, not worse
- each milestone must treat regulated-industry posture as a first-class
  requirement wherever policy, delivery, evidence, topology, or recovery are
  in scope
- each milestone must keep request/response APIs, WORTH-native facade
  surfaces, subscription APIs, sync delivery, binary transfer, policy
  enforcement, and diagnostics structurally separate even when they share one
  server facade
- no milestone is complete until it has acceptance evidence proving the server
  preserved meaning under hostile client behavior, restart, pacing variation,
  permission drift, and basis mismatch where those concerns are in scope

## Vision Coverage Rule

The roadmap must cover the full vision explicitly.

The following vision categories are mandatory coverage categories:

- request and session front door
- WORTH-native app facade
- HTTP compatibility surface
- binary and asset boundary
- lease and subscription architecture
- sync protocol architecture
- authentication, authorization, and remask
- regulated deployment architecture
- branch-aware delivery architecture
- observability and provenance
- view-specific delivery architecture
- mutation architecture
- zero-trust and cryptography
- integration and extensibility architecture
- distributed scalability architecture
- multi-tenant architecture
- durable-later closure

Every category above must map to one or more milestones below.

## Sequencing Rule

This roadmap intentionally prioritizes external product usefulness first, but
"external first" now has two lanes:

1. WORTH-native direct-consumption surfaces
2. compatibility request/response APIs for non-WORTH and explicit external
   integration consumers

That means the roadmap does not treat handwritten HTTP endpoints as the center
of gravity anymore. WORTH-native facade work may come before or alongside
compatibility APIs whenever that produces the stronger long-term architecture.

## Milestone Outline

### Milestone 1: Server Runtime Front Door, Typed Middleware, And Facade Boundary

Spec: [milestone-1.md](./milestone-1.md)

### Goal

Create one typed server runtime front door so every future WORTH-native facade,
HTTP API, lease, sync, file, and integration surface enters through the same
server-owned policy, tenant, branch, workspace, diagnostics, and observability
pipeline.

### Vision Coverage

- request and session front door
- architectural model and facade rule
- multi-tenant routing foundation
- regulated proof posture foundation

### Must Ship

- an `axum`-based server facade and subsystem-shaped server configuration
- typed request context covering:
  - authentication identity
  - tenant/workspace identity
  - branch or preview targeting where admitted
  - diagnostics policy
  - transport class
- one typed middleware pipeline for:
  - transport decoding
  - authentication
  - tenant/workspace resolution
  - branch resolution
  - rate/budget posture
  - authorization
  - request validation
  - Query/runtime execution handoff
  - response transformation
  - observability
- structured response and error envelopes that preserve typed denial and
  provenance instead of flattening everything into route-local strings
- mechanical separation between:
  - WORTH-native facade surfaces
  - compatibility request/response APIs
  - future sync delivery
  - future binary transfer
  - future lease management
  - future integration surfaces
- explicit counters and diagnostics at the server facade and middleware
  boundaries

### Must Preserve

- `worth-server` remains a facade over typed subsystem boundaries rather than a
  handler bag
- no surface bypasses authentication, tenant resolution, branch resolution,
  authorization, or diagnostics policy
- Query remains the public runtime surface for query and mutation meaning
- no lower-runtime or host-local semantics become accidental server authority

### Acceptance Evidence

- equivalent requests through different surfaces lower through the same
  middleware and execution contract
- denied authentication, tenant, branch, and authorization cases fail at typed
  pipeline boundaries
- surface-local code cannot bypass the shared pipeline without a mechanically
  visible boundary violation
- facade counters and diagnostics explain request admission, denial, and
  execution handoff without consulting host logs

### Milestone 2: WORTH-Native App Facade And Direct Typed Product Surface

Spec: [milestone-2.md](./milestone-2.md)

### Goal

Ship the first-class WORTH-native server surface so applications can consume
typed Query-backed reads, mutations, state, inspection, leases, and delivery
contracts directly without rebuilding their product meaning as endpoint glue.

### Vision Coverage

- WORTH-native app facade
- Query-first server rule
- ergonomics as architecture

### Must Ship

- a typed WORTH-native server facade for ordinary Query-backed product work
- direct server-managed access patterns for:
  - reads
  - mutations
  - state and inspection
  - lease declaration
  - delivery contract negotiation
- explicit client-facing contracts for branch, basis, remask posture, and
  provenance posture on this facade
- typed client-consumable denial and support posture when a WORTH-native path
  requests an unsupported capability
- enough direct-consumption shape that a WORTH-native app can avoid large
  families of handwritten endpoints for ordinary live product work

### Must Preserve

- the WORTH-native facade does not become a second Query runtime
- the server does not hide cost or capability posture behind "magic" ergonomic
  helpers
- typed direct-consumption surfaces still respect the same middleware,
  tenancy, branch, and diagnostics rules as every other server surface

### Acceptance Evidence

- the same product meaning expressed through direct WORTH-native consumption
  does not require a parallel handwritten endpoint family
- equivalent facade operations preserve the same canonical Query meaning as the
  compatibility API surfaces where overlap exists
- unsupported runtime-backed versus durable-later distinctions remain explicit
  on the direct facade

### Milestone 3: External HTTP, Streaming, Binary, And Blob Surface

Spec: [milestone-3.md](./milestone-3.md)
Closeout: [milestone-3-closeout.md](./milestone-3-closeout.md)

### Goal

Ship the external compatibility and interop surface for request/response,
streaming HTTP, multipart upload, range transfer, exports, health, and
non-WORTH clients without creating a second meaning model beside Query or
polluting structured truth sync with blob transport.

### Vision Coverage

- HTTP surface architecture
- request/response compatibility lane
- streaming responses
- binary and asset architecture

### Must Ship

- HTTP read surfaces for:
  - one-shot reads
  - materialized reads where admitted
  - state and inspection access
  - branch-aware and historical basis targeting where admitted
- HTTP mutation surfaces for authoritative writes that consume Query mutation
  contracts rather than raw lower-runtime mechanics
- typed schema-validation and policy-denial responses suitable for external
  clients
- streaming HTTP responses for large exports, large reads, and initial
  hydration where buffering the full response would be dishonest
- multipart upload endpoints
- range-request and resumable download endpoints
- streamed upload/download handling that does not route blobs through the sync
  delivery channel
- explicit linkage between file metadata truth and structured API responses
- typed policy enforcement for file upload/download boundaries
- explicit request contracts for branch, basis, projection, and diagnostics
  posture
- response envelopes that carry enough typed basis and provenance posture for
  clients to reason about what they just received
- explicit binary diagnostics and counters distinct from structured truth
  delivery counters

### Must Preserve

- compatibility APIs do not redefine query, branch, historical, or mutation
  semantics
- streaming changes transport shape only; it does not change canonical read or
  mutation meaning
- binary bytes are not sync protocol payloads
- file metadata remains truth-linked through the normal runtime stack
- binary route throughput does not redefine structured truth delivery
  guarantees
- file routes remain within the same auth, tenant, and branch pipeline as
  other server surfaces
- no route invents a local caching or basis model outside Query's public
  contracts
- compatibility APIs stay visibly secondary to the WORTH-native facade for
  ordinary WORTH product development

### Acceptance Evidence

- equivalent branch-aware and basis-aware requests produce equivalent canonical
  response bundles independent of handler ordering or streaming mode
- invalid branch, basis, or policy combinations fail explicitly and typed
- streaming and non-streaming variants compare equal on canonical response
  meaning where they should
- mutation responses preserve typed validation, denial, and provenance posture
  instead of flattening to transport-local success strings
- file metadata updates remain structurally separate from blob transfer while
  still linking back to truth updates correctly
- large uploads and downloads do not require sync-channel participation
- range and multipart behavior preserve typed authorization and failure
  localization
- binary and structured route counters remain independently explainable

### Milestone 4: Concurrent Operation Admission And Product Surface Runtime

Spec: [milestone-4.md](./milestone-4.md)

### Goal

Update `worth-server` to consume the real Query concurrency and consumer-kit
closures, then add the server-owned operation runtime that lets Query-direct
and product-application operations enter through one typed request, planning,
scheduling, envelope, diagnostics, and route assembly boundary.

This milestone prepares the server for editor-like product servers without
making product-specific semantics part of `worth-server`.

### Vision Coverage

- WORTH-native app facade
- HTTP compatibility surface
- mutation architecture
- branch-aware delivery architecture
- observability and provenance
- multi-tenant architecture
- regulated proof posture foundation
- direct WORTH-native consumption instead of endpoint glue
- product-application operation boundary for WORTH-native apps

### Must Ship

- Query `9.7` and `9.8` dependency audit over covered server paths
- operation-family registry distinct from surface-family registry
- canonical operation request contract and operation identity
- authority footprint and concurrency classification for every operation
- lowered operation plans with support posture, execution strategy, diagnostics
  policy, and counters
- concurrent operation scheduler using Query shared-read and deterministic
  submission surfaces where applicable
- product-application adapter boundary for product-owned operations such as
  editor render, select, available actions, apply, and stricter finalization
- optimistic product session, base-digest, idempotency, stale-basis, conflict,
  and rebase posture
- operation-declared route assembly over Axum
- product-editor-shaped readiness certification proving product operations can
  plug in without route-local server semantics

### Must Preserve

- Query remains the runtime meaning owner for Query reads, state, inspection,
  projection consumption, mutation/submission, support posture, and shared-read
  concurrency
- `worth-server` owns operation admission, planning, scheduling, envelopes,
  diagnostics, route assembly, and transport projection
- product crates own product semantics
- surface families remain transport/entry topology while operation families
  remain authority/execution topology
- route handlers decode and enter the operation runtime; they do not execute
  semantics
- product-editor readiness does not add product-specific semantic branches to
  `worth-server`

### Acceptance Evidence

- covered Query-facing server paths have zero unclassified or legacy
  pre-concurrency assumptions
- equivalent WORTH-native and compatibility HTTP inputs lower to identical
  operation identities and plans where their semantic inputs match
- shared-read-safe operations execute concurrently with serialized-replay
  equivalent envelopes and exact scheduler counters
- mutation/submission/product-draft conflicts serialize or deny through typed
  scheduler posture
- product adapters cannot bypass request context, middleware, operation
  planning, scheduler, response shaping, or diagnostics
- operation-declared routes are parity-equivalent with WORTH-native direct
  operations
- product-editor-shaped render/select/action/apply/finalize operations certify
  through the product adapter without product-specific semantics inside
  `worth-server`

### Milestone 4.1: Canonical Product Results And Durable Product Mutation Join

Spec: [milestone-4.1.md](./milestone-4.1.md)

### Goal

Replace product success keys and adapter-authored digests with canonical,
schema-bearing result artifacts, then add the product-owned atomic mutation and
completion join required by durable product truth.

### Vision Coverage

- product-application operation boundary for WORTH-native apps
- mutation architecture foundation
- self-describing compatibility responses
- durable product authority without server-owned product truth

### Must Ship

- mandatory output schema, encoding, canonicalization, and size contracts
- canonical result bodies and server-computed result artifact digests
- durable product mutation authority distinct from product draft mutation
- product-owned atomic basis, idempotency, mutation, next-basis, result, and
  completion persistence
- typed retry and indeterminate recovery conclusions
- WORTH-native and compatibility HTTP result-artifact parity
- product-shaped host-connection, manifest-admission, and deployment pressure

### Must Preserve

- product persistence remains authoritative for current basis and durable truth
- Worth Server remains the admission, planning, scheduling, envelope,
  diagnostics, and transport owner
- product sessions do not become durable storage claims
- ordinary retry does not become cert replay

### Acceptance Evidence

- adapter-supplied result digests and durable use of process-local retry state
  are absent
- fresh-runtime retry resolves one original canonical completion without a
  second product mutation
- stale basis and conflicting idempotency bindings perform zero mutation
- direct and HTTP surfaces expose identical schema, body, and artifact digests
- independent durable product scopes do not inherit a global mutation lock

### Milestone 5: Runtime-Backed Lease Registry And Server-Managed Subscription Foundation

### Goal

Create the first server-owned subscription substrate as durable-in-design but
runtime-backed in current dependency truth: lease identity, lease admission,
subscription registry, and outbox ownership survive connection loss inside the
running server process without yet claiming restart-stable durability.

### Vision Coverage

- lease and subscription architecture
- server-managed subscriptions
- runtime-backed now versus durable later boundary

### Must Ship

- server-owned lease identity and lease CRUD
- typed lease declarations covering:
  - target query/view identity
  - branch/workspace targeting
  - aspect mask and delivery posture
  - freshness mode
  - runtime-backed resume basis where admitted
- a subscription registry independent from live socket objects
- explicit outbox ownership and delivery staging per lease
- typed denial for unsupported runtime-backed delivery or resume posture
- explicit surfaced debt for durable lease persistence and restart-stable
  continuation

### Must Preserve

- subscriptions are server state, not connection-local state
- the server does not invent query meaning; it binds leases to admitted Query
  and bridge-facing delivery surfaces
- runtime-backed resume is not mislabeled as durable restart-stable resume
- lease identity remains distinct from transport session identity and raw CDC
  cursor identity

### Acceptance Evidence

- a lease survives connection drop and reconnection while the server process
  remains live
- equivalent lease declarations compare equal on canonical lease identity
- unsupported basis or resume combinations fail before activation or delivery
- runtime-backed resume debt is explicit in machine-checkable lease/support
  artifacts

### Milestone 6: Sync Transport, Delivery Classes, Basis Negotiation, And Runtime-Backed Resume

### Goal

Turn the server from facade plus leases into a real sync runtime by projecting
Query-owned downstream delivery into transport-safe protocol messages, typed
delivery classes, explicit lanes, freshness modes, and runtime-backed basis
negotiation.

### Vision Coverage

- sync protocol architecture
- typed delivery classes
- basis negotiation
- freshness modes
- multiplexed framing
- adaptive delivery degradation

### Must Ship

- WebSocket sync transport with explicit protocol framing
- postcard-first message encoding with JSON/debug fallback
- typed server protocol messages for:
  - authoritative truth-patch delivery
  - time-only delivery
  - async-backed delivery
  - mixed-cause delivery
  - invalidation-only delivery
  - presence delivery
  - advisory or diagnostic delivery where admitted
- explicit invalidation lane, patch lane, and presence lane separation
- runtime-backed basis negotiation built on Query's public downstream delivery
  contract
- freshness-mode contracts for:
  - `live_strict`
  - `live_coalesced`
  - `background_coalesced`
  - `invalidate_only`
  - `pull_on_focus`
  - `presence_only`
- multiplexed protocol framing and pacing-aware outbox behavior
- typed denial for stale, missing, or incompatible runtime-backed resume basis
- adaptive degradation contracts that change freshness or pacing posture
  without corrupting canonical meaning

### Must Preserve

- transport framing does not become the authority for delivery meaning
- server protocol messages are projections of Query/bridge delivery artifacts,
  not fresh semantics
- no raw retained-batch archaeology at the transport edge
- durable resume debt remains explicit and typed

### Acceptance Evidence

- equivalent Query downstream deliveries project to equivalent protocol bundles
- time-only and async-backed deliveries remain distinguishable without faking
  relational patches
- stale or mismatched runtime-backed basis fails explicitly and typed
- invalidation-only, patch-bearing, and presence lanes differ mechanically
  without changing canonical delivery meaning
- multiplexing prevents large payload delivery from starving higher-priority
  small frames
- adaptive degradation changes pacing policy without changing semantic parity

### Milestone 7: Policy-Scoped Delivery, Tenant Isolation, Branch Semantics, And Remask Closure

### Goal

Make active delivery safe under tenant, branch, authorization, and remask
pressure so the server never leaks aspects or cross-tenant or cross-branch
truth while still supporting shared infrastructure.

### Vision Coverage

- authentication, authorization, and remask
- branch-aware delivery architecture
- multi-tenant architecture

### Must Ship

- tenant workspace routing for every request, lease, and sync delivery path
- branch-aware reads and branch-scoped subscriptions
- policy- and aspect-level authorization on active delivery, not just on route
  entry
- remask-aware delivery projection that preserves Query denial and drift
  posture
- permission-change handling for active leases
- typed denial and diagnostics for cross-tenant, cross-branch, and remask
  failures

### Must Preserve

- aspect authorization happens before delivery leaves the server boundary
- tenant routing is structural, not post-filtering
- branch-local truth does not leak into authoritative or other-tenant delivery
- shared infrastructure may not weaken per-client permission truth

### Acceptance Evidence

- permission and tenant changes on equivalent leases produce canonical remask
  or denial posture instead of silent drift
- cross-tenant and cross-branch misroutes fail explicitly before payload
  delivery
- per-client authorization remains correct even when shared infrastructure is
  reused underneath

### Milestone 8: Regulated Deployment Evidence, Audit Posture, And Recovery Honesty

### Goal

Make regulated-industry posture first-class by introducing explicit operator
evidence, compliance-friendly topology controls, and machine-checkable honesty
about restart, retention, and recovery capability boundaries.

### Vision Coverage

- regulated deployment architecture
- regulated industries domain fit
- observability and provenance for operators

### Must Ship

- audit-grade provenance and denial evidence suitable for operator inspection
- typed operator-facing evidence surfaces for:
  - what was exposed
  - why it was exposed
  - which basis or policy decision authorized it
- compliance-friendly topology and residency controls at the server boundary
- explicit surfaced distinction between runtime-backed recovery and durable
  restart-stable closure
- typed retention and recovery posture surfaces that avoid marketing-style
  ambiguity

### Must Preserve

- compliance posture is derived from canonical server/runtime artifacts rather
  than ad hoc operator notes
- audit evidence does not redefine domain truth
- regulated proof surfaces do not force always-on rich diagnostics in the hot
  path when policy says otherwise

### Acceptance Evidence

- operators can reconstruct exposure and denial posture from server evidence
  artifacts without consulting ad hoc host logs
- residency and topology policy decisions are explicit and machine-checkable
- runtime-backed versus durable-later recovery distinctions remain visible to
  clients and operators under restart-hostile scenarios

### Milestone 9: Shared Subscription Bases, View-Shaped Delivery, And Server-Side Materialization

### Goal

Make high-fanout collaborative delivery efficient without weakening per-client
correctness by introducing shared subscription bases, view-shaped patch
projection, and server-side materialization where the contracts honestly allow
it.

### Vision Coverage

- shared subscription bases
- view-specific delivery architecture
- server-side materialization

### Must Ship

- shared subscription base contracts for equivalent or overlapping client views
- per-client masking layered over shared base evaluation
- view-shaped patch families for table, detail, grouped, timeline, chart, and
  other admitted view shapes
- server-side maintained materialization for hot shared collaborative views
- freshness-mode and coalescing policies over shared subscription delivery
- counters for fanout, shared-base reuse, coalescing, and materialization cost

### Must Preserve

- shared evaluation never changes per-client visible truth
- server-side materialization is an optimization, not new authority
- view-shaped delivery remains derived from canonical Query meaning
- coalescing changes pacing and patch packaging only through admitted policy

### Acceptance Evidence

- shared-base and non-shared equivalent leases compare equal on visible client
  truth
- different view shapes over the same truth produce distinct but canonical
  patch families
- materialized and non-materialized delivery paths remain parity-safe on
  canonical visible meaning
- fanout and coalescing counters explain the server's scale behavior honestly

### Milestone 10: Mutation Protocol, Optimistic Branch Flow, And Provenance-Bearing Results

### Goal

Close the server mutation architecture as a real network-facing mutation
runtime rather than a thin transport shell around lower writes.

### Vision Coverage

- mutation architecture
- optimistic mutation protocol
- protocol-level provenance

### Must Ship

- schema-validated mutation admission at the server boundary
- provenance-bearing mutation result envelopes derived from Milestone 4.1
  canonical product result artifacts
- branch-aware optimistic mutation flow over Query and Relational semantics
- confirmation and rejection surfaces that preserve rollback and explanation
  truth
- explicit denial and failure localization for invalid, drifted, or forbidden
  mutation attempts

### Must Preserve

- Milestone 4.1 remains the durable product mutation and canonical result
  foundation; this milestone does not create a parallel transaction lane
- optimistic mutation does not invent transport-local conflict folklore
- mutation results preserve typed validation, denial, and provenance posture
- branch-aware optimism remains a projection of runtime semantics, not a second
  authority path

### Acceptance Evidence

- equivalent admitted mutations produce canonical result envelopes across
  facade and compatibility surfaces
- rejected optimistic flows localize exactly why confirmation failed
- rollback and confirmation semantics remain branch-aware and provenance-safe

### Milestone 11: Background Delivery, Integration-Facing CDC, And Extensible Integration Wiring

### Goal

Broaden the server from interactive clients into external integrations and
background consumers without weakening the core delivery contract.

### Vision Coverage

- background HTTP delivery
- integration and extensibility architecture
- integration-facing CDC surface

### Must Ship

- long-poll, scheduled-pull, or webhook-style delivery modes where admitted
- integration-facing CDC and change-feed API surfaces
- structured integration wiring for external webhooks, polling, OAuth
  callbacks, and callback ingestion
- typed external HTTP source adapters where admitted, including:
  - request declaration
  - typed response contracts
  - source identity and freshness posture
  - explicit authority mode
- optional typed-response projection into server-managed writeback and live-view
  flows when the external source contract is strong enough to lower into
  canonical server truth or canonical derived observation artifacts
- typed mutation and outbox boundaries for cross-system workflows
- first-party integration packages only where the server can keep the boundary
  honest and audited
- extensible integration wiring for non-first-party ecosystems

### Must Preserve

- integration surfaces consume the same canonical delivery and mutation truth
- external callbacks do not become server authority without validation and
  typed mutation admission
- external HTTP sources do not become automatic truth merely because they are
  typed; admitted typed responses must still lower through explicit authority
  mode and canonical server artifacts
- optional writeback/live-view support for typed external sources must remain
  opt-in and contract-driven rather than silently attached to every HTTP call
- outbox and integration orchestration remains distinct from core delivery
  contracts
- CDC remains the explicit integration lane rather than silently becoming the
  default ordinary app contract

### Acceptance Evidence

- background delivery modes preserve canonical delivery meaning across pacing
  and transport variation
- integration ingestion fails explicitly on schema, auth, or policy mismatch
- equivalent admitted typed external responses produce equivalent canonical
  writeback or derived-observation artifacts when optional automatic
  projection is claimed
- typed external sources that are observe-only, propose-only, or
  authority-bearing compare distinctly on canonical artifacts and cannot drift
  silently between those modes
- outbox-backed cross-system flows cannot partially commit authoritative truth
  silently
- equivalent Query-shaped app delivery and CDC-shaped integration delivery
  remain intentionally distinct where they should

### Milestone 12: Durable Lease Persistence, Restart-Stable Resume, And Anti-Entropy Recovery

### Goal

Promote the runtime-backed lease system into a durable server contract once
Store and Query expose the necessary persistence and restart-grade delivery
surfaces.

### Vision Coverage

- durable-later closure
- semantic cursor-based resume
- anti-entropy state reconciliation

### Must Ship

- durable lease persistence
- restart-stable delivery and resume basis
- explicit checkpoint contracts that survive process restart
- compound catchup from retained canonical artifacts rather than replaying raw
  transport windows heuristically
- anti-entropy fallback for cursor loss or retention truncation
- typed incompatibility and truncation failure surfaces

### Must Preserve

- durable resume must consume real Store, Query, and Bridge persistence
  contracts, not server-local folklore
- anti-entropy fallback remains explicit recovery, not silent downgrade
- restart does not redefine lease identity, branch identity, or delivery class
  meaning

### Acceptance Evidence

- restarted and uninterrupted leases compare equal where retained basis is
  sufficient
- stale, truncated, or incompatible retained basis fails explicitly and typed
- compound catchup preserves canonical visible truth without replaying every
  transport event
- anti-entropy recovery localizes exactly why cursor resume was impossible

### Milestone 13: Zero-Trust Transport, Blind-Server Modes, And WebTransport Upgrade

### Goal

Close the remaining transport and cryptography-specific parts of the vision
without smearing them backward into the earlier server foundation.

### Vision Coverage

- zero-trust and cryptography
- WebTransport later-upgrade path

### Must Ship

- optional end-to-end encrypted or blind-server deployment contracts
- transport and routing posture that can operate on unclassified metadata where
  admitted
- WebTransport upgrade path on top of the same canonical sync semantics
- typed capability and incompatibility posture when blind-server or upgraded
  transport modes cannot support a requested surface

### Must Preserve

- cryptographic deployment modes do not redefine ordinary delivery semantics
- WebTransport changes transport mechanics only; it does not create a second
  sync meaning model
- unsupported cryptographic or transport combinations fail explicitly instead
  of silently degrading semantics

### Acceptance Evidence

- ordinary and blind-server compatible lanes compare equal on canonical
  visible semantics where they should
- transport upgrade preserves protocol parity across WebSocket and
  WebTransport where admitted
- unsupported combinations fail typed with explicit capability posture

### Milestone 14: Distributed Topology, Edge Coordination, And Final Certification

### Goal

Close the server as a production-grade distributed network runtime under
cluster, reconnect, load, policy, and topology pressure.

### Vision Coverage

- distributed scalability architecture
- workspace affinity routing
- edge-to-cloud topology
- control-plane gossip and distributed invalidation
- final certification

### Must Ship

- workspace-affinity routing contracts
- multi-node invalidation and coordination contracts
- explicit cluster-aware lease and delivery semantics
- edge or regional topology posture where admitted
- load-shedding and backpressure contracts
- server-grade certification suites over:
  - WORTH-native facade parity
  - compatibility request/response parity
  - lease and reconnect parity
  - runtime-backed and durable resume
  - remask and permission drift
  - regulated evidence posture
  - cluster invalidation parity
  - shared subscription correctness
  - background delivery parity

### Must Preserve

- clustering does not redefine canonical delivery meaning
- delivery authority remains upstream in Query, Bridge, Relational, and Store
- horizontal scale changes routing and coordination only through explicit
  contracts
- certification remains machine-checkable and bundle-based rather than
  explanation-by-log

### Acceptance Evidence

- equivalent single-node and multi-node executions compare equal on canonical
  client-visible truth where they should
- intentionally different tenant, branch, policy, or basis cases compare
  unequal or fail typed where they should
- restart, reconnect, load shedding, permission churn, and topology changes
  preserve server semantics through canonical certification bundles

## Vision Coverage Appendix

### Direct Milestone Mapping

- request and session front door: Milestone 1
- WORTH-native app facade: Milestones 2, 4, and 4.1
- HTTP surface architecture: Milestone 3
- binary and asset architecture: Milestone 3
- lease and subscription architecture: Milestones 5 and 6
- sync protocol architecture: Milestone 6
- authentication, authorization, and remask: Milestones 1 and 7
- regulated deployment architecture: Milestone 8
- branch-aware delivery architecture: Milestones 4, 4.1, and 7
- observability and provenance: Milestones 1, 4, 4.1, 8, 10, and 14
- view-specific delivery architecture: Milestone 9
- mutation architecture: Milestones 4, 4.1, and 10
- zero-trust and cryptography: Milestone 13
- integration and extensibility architecture: Milestone 11
- distributed scalability architecture: Milestone 14
- multi-tenant architecture: Milestones 1, 4, 4.1, and 7
- runtime-backed now versus durable later split: Milestones 5, 6, and 12

### Explicit Vision Capability Mapping

- Query-first server entry: Milestones 1, 2, 3, and 4
- direct WORTH-native consumption instead of endpoint glue: Milestones 2 and 4
- product-application operation runtime: Milestone 4
- canonical product result publication: Milestone 4.1
- durable product mutation join: Milestone 4.1
- Query 9.7 concurrency consumption: Milestone 4
- Query 9.8 consumer-kit consumption: Milestone 4
- operation-declared route assembly: Milestone 4
- optimistic product-session and stale-basis posture: Milestone 4
- compatibility request/response surface: Milestone 3
- file upload and binary transfer boundary: Milestone 3
- server-managed durable-in-design subscriptions: Milestone 5
- signal-backed subscription relevance: Milestones 5 and 6
- outbox and coalescing behavior: Milestones 5 and 6
- typed delivery classes: Milestone 6
- invalidation, patch, and presence lanes: Milestone 6
- basis negotiation and runtime-backed resume: Milestone 6
- freshness modes: Milestone 6
- multiplexed framing: Milestone 6
- adaptive delivery degradation: Milestones 6 and 14
- remask-aware projection: Milestone 7
- tenant and branch delivery safety: Milestone 7
- audit-grade provenance and operator evidence: Milestone 8
- residency and topology controls for regulated systems: Milestones 8 and 14
- shared subscription bases: Milestone 9
- view-shaped delivery: Milestone 9
- server-side materialization: Milestone 9
- optimistic branch-aware mutation flow: Milestone 10
- outbox and saga boundaries: Milestone 11
- integration-facing CDC: Milestone 11
- typed external HTTP source adapters with optional writeback/live-view
  projection: Milestone 11
- first-party and extensible integrations: Milestone 11
- durable persistence and anti-entropy recovery: Milestone 12
- blind-server and end-to-end encrypted modes: Milestone 13
- WebTransport upgrade path: Milestone 13
- cluster invalidation, affinity routing, and edge topology: Milestone 14

## Completion Standard

`worth-server` is roadmap-complete only when:

- the WORTH-native direct-consumption server surface is shipped
- the compatibility request/response surface is shipped without redefining
  Query semantics
- the binary and structured truth boundaries are separate and honest
- the concurrent operation runtime consumes Query's real shared-read,
  deterministic-submission, and consumer-kit surfaces rather than preserving
  pre-concurrency or consumer-owned folklore
- product-application operations enter through typed operation declarations,
  authority footprints, lowered plans, scheduler outcomes, and server response
  envelopes rather than route-local endpoint glue
- active subscriptions are server-owned rather than socket-owned
- sync delivery consumes one typed Query-owned downstream contract rather than
  raw runtime folklore
- tenant, branch, policy, remask, and permission truth hold on active delivery
- regulated-industry evidence and recovery honesty are first-class rather than
  add-on concerns
- shared subscription optimization and view-shaped patch delivery are
  parity-safe
- durable lease persistence and restart-stable resume are closed on real
  persistence contracts
- integration delivery surfaces consume the same server truth model without
  collapsing into ordinary app semantics
- zero-trust and transport-upgrade modes preserve canonical parity where
  admitted
- distributed certification proves the whole network boundary under restart,
  reconnect, pacing, branch, permission, topology, and cluster hostility
