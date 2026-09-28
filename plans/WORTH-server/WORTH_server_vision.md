# WORTH Server Vision

## Thesis

`worth-relational` owns truth. `worth-signal` owns derived computation.
`worth-store` owns durability. `worth-runtime-bridge` owns cross-runtime
causal binding. `worth-query` owns the ordinary public runtime meaning that
downstream product code should inherit. `worth-server` owns network delivery
operations.

`worth-server` is the network-facing runtime that makes the WORTH stack usable
over HTTP, WebSocket, background delivery, and binary transfer without
weakening the semantic contracts already closed below it.

For WORTH-native applications, the highest-leverage shape is not a bag of
handwritten API endpoints. It is a direct server projection of Query-owned
reads, mutations, leases, and delivery contracts with as little glue as
possible between application intent and network behavior.

It is not just a wire protocol and it is not just an app server. It is the
full transport/runtime layer that:

- admits external requests, sessions, and leases
- applies one typed auth/tenant/branch/policy pipeline
- consumes Query-owned and bridge-owned delivery and mutation contracts
- projects those contracts onto network-facing surfaces
- preserves basis, provenance, denial, remask, and diagnostics posture at the
  network boundary

The server is intended for systems where connected clients need live,
incremental, explainable state synchronization with the same structural
guarantees the runtime provides locally without collapsing into generic
pub/sub, crude polling, or dumb WebSocket broadcast.

## Semantic Source Of Truth

`worth-server` does not define ordinary read meaning, live meaning,
mixed-cause meaning, remask meaning, runtime-backed resume meaning, canonical
query delivery meaning, or optimistic conflict meaning.

Those semantic contracts come from the runtime stack below it:

- `worth-relational` owns authoritative truth, branch identity, snapshots,
  history, lineage, CDC, commit serialization, and relational invariants
- `worth-runtime-bridge` owns cross-runtime causal binding, temporal/async
  basis, mixed truth/time/async ordering, subscription protocol semantics,
  replay posture, certification artifacts, and delivery causality
- `worth-query` owns the ordinary public runtime meaning that downstream
  consumers should inherit, including retained live meaning, remask posture,
  view meaning, runtime-backed downstream delivery projection, runtime-backed
  resume negotiation, and public mutation/read semantics

The server's job is to preserve and project those semantics, not to reopen
them. If the server re-decides basis, resume, remask, delivery class, branch
meaning, or optimistic rollback meaning at the transport edge, it creates a
second semantic authority layer and weakens the stack.

## Query-First Server Rule

Ordinary external product traffic should enter `worth-server` through
Query-facing seams rather than through lower-runtime folklore.

For ordinary reads, mutations, state, inspection, and live delivery, the
server should prefer:

- `WORTHQueryWorkspace`
- `workspace.downstream_delivery(...)`
- `workspace.public_downstream_delivery_contract()`
- other admitted Query workspace/runtime surfaces that preserve the public
  support contract

This rule exists for two reasons:

- it keeps the server from inventing a second public meaning model beside
  Query
- it lets the server inherit branch, basis, remask, mixed-cause, and
  runtime-backed resume posture from the already-admitted runtime contracts

Raw CDC, lower-runtime commit streams, and bridge-native details still matter,
but they are not the default ordinary app-facing server contract. They are
integration-facing, infrastructure-facing, or later durability-facing
capabilities unless explicitly admitted otherwise.

## What This Server Is For

`worth-server` exists for product surfaces where external consumers need to
observe, mutate, and synchronize truth-state with the precision and efficiency
the runtime makes possible.

It is meant to support:

- collaborative web applications where multiple users edit the same workspace
  and need immediate, minimal, semantically precise state updates instead of
  broad refetch or crude room-based broadcast
- regulated systems that need strong auditability, policy enforcement,
  remask-aware delivery, provenance-bearing responses, tenant isolation,
  residency-aware routing, restart honesty, and deployment options that can
  survive compliance review without a second architecture
- mobile and offline-first applications that need durable subscriptions,
  cursor-based resume, and compound catchup patches instead of full re-sync
  after disconnection
- AI agent interfaces that need branch-aware reads, speculative writes on
  branches, provenance-bearing responses, and stable evaluation contexts
- dashboard and monitoring surfaces that need live incremental refresh with
  tolerance-aware suppression instead of periodic full-page polling
- integration pipelines that need durable CDC consumption with exactly-once
  cursor semantics and structured schema-aware change feeds
- multi-store replication where one WORTH store subscribes to another's commit
  stream through the same protocol family that serves browser clients
- geometry and chip-design tool servers that need snapshot-safe concurrent
  reads, branch-local analysis, and large binary asset management alongside
  structured truth sync
- WORTH-native applications that should be able to consume server-managed
  Query semantics directly instead of rebuilding them as endpoint glue,
  client-side cache folklore, and bespoke sync plumbing

The technical thesis is the same across all of them:

- the server must know what changed, who cares, and what is meaningful
- delivery must be minimal, not maximal
- subscriptions must survive connection loss
- resume must be cursor-based, not full re-sync
- binary assets and structured truth are separate concerns with separate
  transport
- authentication and authorization are infrastructure, not features
- ergonomics must improve as semantic power increases, not get worse
- WORTH-native apps should default to direct protocol or facade consumption,
  with handwritten API endpoints reserved for interop, compatibility, and
  explicit external integration surfaces
- the server must not weaken the runtime's semantic guarantees

## Why This Server Is Different

These are not optional add-ons. They are the capabilities that make
`worth-server` strategically different from ordinary application servers:

- Query-first delivery projection instead of route-local meaning
- signal-native subscription evaluation through bridge and Query contracts
- server-managed subscriptions as durable projections of Query live meaning
- semantic cursor-based resume and compound catchup
- anti-entropy state reconciliation when cursor recovery is impossible
- aspect-aware delivery filtering and remask-aware denial
- typed delivery classes:
  - authoritative-ordered
  - replaceable-latest-state
  - coalescible-region
  - ephemeral-presence
  - advisory-hint
- invalidation lane, patch lane, and presence lane separation
- basis negotiation for minimal delta delivery
- branch-aware reads and branch-scoped subscriptions
- structured schema-aware CDC as a dedicated integration surface
- binary asset management alongside structured sync
- freshness mode negotiation:
  - live_strict
  - live_coalesced
  - background_coalesced
  - invalidate_only
  - pull_on_focus
  - presence_only
- shared subscription bases for overlapping client views
- provenance-bearing patches that explain causality
- view-shaped patch delivery
- server-side view materialization for hot collaborative surfaces
- typed middleware pipeline with declarative policy enforcement
- aspect-level authorization, not just route-level checks
- optimistic mutation flow over Query/Relational branch semantics
- direct WORTH-native app binding to typed server-managed Query surfaces,
  reducing or eliminating handwritten endpoint glue
- background HTTP delivery and integration-facing delivery modes
- first-party integrations where the server can keep the seam audited
- extensible integration wiring for everything else
- multiplexed framing so large payloads do not starve small critical frames
- adaptive delivery degradation under backpressure without semantic drift
- distributed saga and outbox boundaries for cross-system workflows
- optional end-to-end encrypted or blind-server deployments
- eventual edge, cluster, and data-sovereignty-aware topology
- compliance-friendly observability, audit, and policy posture suitable for
  regulated deployments

If these are treated as "nice to have later," the server becomes an ordinary
REST API with a WebSocket bolted on, and the entire value of the runtime stack
below it is wasted at the network boundary.

## Architectural Model

### Runtime stack

| Layer | Responsibility | Owns |
| --- | --- | --- |
| `worth-relational` | Truth-state semantics | identity, transactions, MVCC, diffs, CDC, lineage, schema |
| `worth-store` | Durable storage | commit persistence, WAL, snapshots, compaction, backends |
| `worth-signal` | Derived computation | dependency DAG, invalidation, recomputation, scheduling |
| `worth-runtime-bridge` | Decoupled coordination | patch-to-invalidation, aspect mapping, snapshot evaluation, temporal/async causality |
| `worth-query` | Ordinary public runtime meaning | live/state/inspection/downstream delivery, remask, runtime-backed resume, view meaning |
| `worth-server` | Network delivery operations | leases, transport, HTTP surface, file handling, auth, outboxes, delivery projection |

### Two server ownership layers

#### Semantic projection intake

This is the layer where the server consumes upstream meaning:

- Query workspace entry
- Query downstream delivery contract
- Query retained live/state/inspection posture
- Query-owned view identity and remask posture
- bridge-owned delivery causality and mixed-cause ordering
- branch/basis/remask/runtime-backed resume posture

This layer exists so the server can inherit canonical runtime meaning instead
of inventing a transport-local substitute.

#### Network and session operations

This is the layer the server actually owns:

- sync protocol framing
- request/response routing
- lease lifecycle
- outboxes and delivery pacing
- file upload/download
- auth middleware
- tenant/workspace routing
- connection lifecycle
- background delivery
- observability

This layer exists so the runtime stack can be exposed safely and efficiently to
external consumers.

### What `worth-server` owns

- the sync protocol: message types, framing, transport-safe envelopes, cursor
  contracts, delivery/session mechanics, and multiplexed lane behavior
- the lease manager: lease lifecycle, lease CRUD, outbox ownership,
  coalescing, and session-independent subscription state
- the HTTP surface: reads, mutations, file upload/download, health,
  background delivery, and traditional request/response
- the WebSocket and later WebTransport surface: sync channel upgrade, delivery
  framing, connection lifecycle, and pacing
- WORTH-native application facade surfaces that expose typed reads, mutations,
  leases, and delivery without forcing route-by-route glue
- authentication and authorization integration: middleware, policy
  enforcement, subscription-scoped permission checks, and permission-drift
  handling
- delivery infrastructure: typed delivery classes, invalidation lane, patch
  lane, presence lane, coalescing, and basis negotiation
- freshness mode management: per-lease freshness contracts and degradation
  posture
- branch-aware and tenant-aware routing for reads, mutations, and
  subscriptions
- shared subscription optimization and server-side materialization as delivery
  optimizations
- server diagnostics: connection metrics, lease metrics, outbox metrics,
  delivery metrics, protocol health, and operational provenance surfaces
- integration wiring: webhook ingress/egress, background delivery, and
  outbox/saga coordination where external systems are involved

### What `worth-server` does not own

- truth semantics, transaction semantics, lineage semantics, or identity
  semantics
- storage engine behavior, WAL management, snapshots, or compaction
- signal evaluation, dependency tracking, or reactive scheduling
- bridge routing logic, mixed-cause ordering logic, temporal basis logic, or
  async completion causality logic
- ordinary read meaning, live meaning, mixed-cause meaning, remask meaning,
  runtime-backed resume meaning, or view meaning
- domain-specific meaning of entities, relations, or aspects
- application-specific business logic

### Structural rule

`worth-server` translates between the WORTH runtime stack and the network. It
does not become a second truth runtime, a second signal scheduler, a second
query runtime, or a second storage engine. It consumes the runtime's semantic
precision and delivers it faithfully to external consumers.

## Principles

1. Subscriptions are durable server state, not connection-scoped ephemeral
   state.
2. Delivery must be minimal by default: the server sends only what is
   meaningful for each lease.
3. Binary assets and structured truth sync are separate transport concerns
   with separate channels.
4. Authentication and authorization are infrastructure middleware, not
   per-endpoint application code.
5. Connection loss must not destroy lease state.
6. Resume is cursor-based, never full re-sync unless the basis is
   irrecoverable.
7. Different update types deserve different delivery guarantees.
8. The server evaluates subscription relevance using the signal graph through
   bridge and Query contracts, not by broadcasting and filtering.
9. Branch-aware reads and branch-scoped subscriptions are first-class, not
   add-ons.
10. Freshness mode is a client-declared contract, not a server-imposed
    default.
11. Shared subscription optimization must not compromise per-client
    correctness or per-client permission scoping.
12. View-shaped delivery must remain a projection of canonical query meaning,
    not a server-local patch invention.
13. Query-first semantic intake is mandatory for ordinary delivery and
    runtime-backed resume posture.
14. Remask must narrow or deny visible meaning before network projection, not
    after.
15. The sync protocol must be transport-pluggable: WebSocket first,
    WebTransport later, HTTP fallback where admitted.
16. Server diagnostics are a production contract, not debugging
    instrumentation.
17. Runtime-backed delivery now and durable replay later must remain explicit
    as separate capability boundaries.
18. Raw CDC is an integration lane by default, not the ordinary app-facing
    delivery contract.
19. Optimistic mutation is branch-aware runtime semantics projected over the
    network, not transport-local conflict folklore.
20. Integrations are layered on the server core, not the definition of the
    server itself.
21. Regulated-industry posture is a first-class product requirement, not a
    later packaging exercise.
22. Ergonomics are architecture: WORTH-native applications should consume
    typed server-managed semantics directly instead of recreating them as
    endpoint glue, ad hoc caches, and client-owned retry folklore.

## Runtime-Backed Now, Durable Later

The server vision is intentionally split into two delivery epochs.

### Runtime-backed now

What is already a meaningful upstream dependency:

- Query-owned downstream delivery projection
- Query-owned runtime-backed resume negotiation
- bridge-owned mixed truth/time/async delivery semantics
- Query-owned remask-aware public delivery posture

This is enough to build:

- request/response APIs
- runtime-backed leases
- runtime-backed sync delivery
- runtime-backed basis negotiation
- runtime-backed reconnect inside an active live server

### Durable later

What still requires later upstream closure:

- restart-stable durable lease persistence
- store-backed replay and full retained delivery resurrection
- durable inflight async restore
- durable downstream resume and anti-entropy closure over persisted artifacts

The server must not fake these capabilities early. Runtime-backed now is a
real product surface. Durable later is a real later milestone, not a wording
detail.

## Foundational Decisions

These are locked architectural decisions:

- `axum` is the HTTP and WebSocket framework
- `tokio` is the async runtime
- postcard is the default binary serialization for sync messages with JSON
  fallback for debugging and development
- lease creation is HTTP, not WebSocket; leases are durable state, not
  connection events
- the WebSocket channel is for delivery, not for subscription management
- subscription evaluation uses the signal graph through the bridge and Query
  contracts, not custom narrowing logic
- authentication middleware applies uniformly to HTTP, WebSocket, and lease
  management
- file uploads go through standard HTTP multipart; file metadata flows through
  truth commits and later sync updates
- structured schema-aware CDC is a dedicated integration-facing surface, not
  the default ordinary app delivery surface
- the sync protocol defines explicit message types with versioned schemas
- CDC cursor persistence ultimately delegates to `worth-store`
- presence and structured truth updates are separate protocol lanes
- optimistic confirmation and rollback semantics are inherited from
  Query/Relational branch-aware mutation meaning, not invented by transport
- WORTH-native application delivery should prefer typed server-managed Query
  surfaces over handwritten endpoint families whenever the product stays
  inside WORTH-native contracts

## Capability Pillars

### Request And Session Front Door

#### Typed middleware pipeline

Every request, upgrade, lease operation, mutation, file transfer, and
background delivery entry runs through one typed middleware pipeline covering:

- transport decoding
- authentication
- tenant/workspace resolution
- branch/basis targeting
- rate limiting and budget posture
- authorization
- request validation
- Query/runtime execution handoff
- response transformation
- observability

This exists to make route-local security and meaning folklore impossible.

#### Query-first server entry

Ordinary reads, mutations, state, inspection, and delivery projection should
enter through Query-facing server seams rather than through lower-runtime
plumbing.

This exists to keep the server from defining a second meaning model beside the
runtime stack.

#### WORTH-native app facade

For WORTH-native applications, the server should expose a typed facade where
the ordinary product surface is already expressed as Query-backed reads,
mutations, leases, and delivery contracts rather than route-by-route endpoint
assembly.

This exists so the best WORTH experience is:

- declare product meaning once
- expose it once through server-managed typed seams
- consume it directly from clients without rebuilding transport glue, cache
  invalidation folklore, or parallel endpoint taxonomies

Handwritten HTTP endpoints still matter for compatibility, exports, public
integration contracts, and non-WORTH consumers. They should not be the center
of gravity for WORTH-native application ergonomics.

### HTTP Surface Architecture

HTTP remains important, but it is not the only ergonomic target. For
WORTH-native applications, HTTP endpoints are one compatibility surface among
several. The deeper goal is to let applications consume admitted server-owned
Query semantics directly.

#### Request/response reads and mutations

Traditional request/response operations such as one-off reads, mutations,
inspection, exports, and health checks go through HTTP routes that preserve
explicit branch, basis, policy, and diagnostics posture.

This exists to provide useful external APIs early without forcing a second
server architecture later.

#### Streaming responses

Large exports, bulk reads, and initial hydration use HTTP chunked streaming so
the server does not buffer the full response in memory before sending it.

This exists to make the external API surface honest about transport cost while
preserving canonical query meaning.

#### Background HTTP delivery

The server supports HTTP-based sync modes such as long-polling, scheduled
pull, or webhook-style delivery where maintaining a live socket is impractical.

This exists for background mobile tasks, serverless workers, corporate-proxy
fallbacks, and integration consumers.

### Binary And Asset Architecture

#### File and binary asset handling

File uploads use HTTP multipart. File downloads use range requests. Binary
blobs do not flow through the structured sync channel. File metadata remains
truth-linked through the ordinary runtime stack.

This exists so large binary payloads do not contaminate the structured truth
delivery contract.

### Lease And Subscription Architecture

#### Server-managed subscriptions

Clients declare subscriptions as durable server-side state. Each lease carries
view identity, branch/workspace targeting, aspect mask, freshness mode, basis
state, and where admitted a CDC cursor or resume basis.

The important rule is:

- Query owns canonical live and view meaning
- the server owns lease lifecycle, outbox behavior, and transport projection

This exists so subscription meaning survives transport interruption and is not
owned by socket lifetime.

#### Signal-backed subscription evaluation

Each active lease corresponds to bridge- and signal-backed subscription
evaluation. When truth changes, clock advances, or async completions matter,
the server consults the upstream reactive substrate instead of broadcasting to
all clients and filtering afterward.

This exists to make the server know what changed, who cares, and what is
meaningful.

#### Subscription outbox and coalescing

Each lease owns an outbox that accumulates pending deliveries. Coalescing,
freshness, and pacing policies act on this outbox without redefining canonical
delivery meaning.

This exists for reconnect efficiency, bounded memory behavior, and background
delivery.

#### Shared subscription bases

When multiple clients subscribe to equivalent or overlapping views, the server
can compute a shared base evaluation and layer per-client narrowing on top.

This exists for collaborative fanout efficiency without sacrificing per-client
correctness.

### Sync Protocol Architecture

#### Semantic cursor-based resume

Every lease tracks cursor or basis position in canonical server/runtime
artifacts. On reconnect, the server advances from that position to current
state through compound catchup rather than replaying arbitrary transport
history.

This exists so reconnection cost is proportional to actual change rather than
missed event count.

#### Anti-entropy state reconciliation

When cursor or retained basis recovery is impossible, the server falls back to
structural anti-entropy recovery rather than silent full-state reset.

This exists so long-disconnected clients and retention-truncated clients still
have an honest recovery story.

#### Typed delivery classes

The server projects canonical delivery into explicit classes:

- `authoritative-ordered`
- `replaceable-latest-state`
- `coalescible-region`
- `ephemeral-presence`
- `advisory-hint`

This exists because not all updates deserve the same reliability, ordering, or
retention semantics.

#### Invalidation lane, patch lane, and presence lane separation

The server maintains:

- a lightweight invalidation lane
- a payload-bearing patch lane
- a presence lane for ephemeral coordination state

This exists so background or bandwidth-constrained clients can receive
awareness without always paying full payload cost, and so ephemeral presence
does not masquerade as authoritative truth.

#### Basis negotiation

The server and client negotiate what the client already has so the server can
project the smallest correct update or explicit rebase response.

This exists to make delta delivery minimal and basis-honest.

#### Freshness modes

Clients declare freshness contracts such as:

- `live_strict`
- `live_coalesced`
- `background_coalesced`
- `invalidate_only`
- `pull_on_focus`
- `presence_only`

This exists so delivery posture can vary by product need without redefining
canonical meaning.

#### Multiplexed protocol framing

The protocol must be able to interleave large payload delivery with small
high-priority frames such as presence or invalidation updates.

This exists to prevent head-of-line blocking from turning one large data sync
into a full-session UX freeze.

#### Adaptive delivery degradation

The server must be able to degrade freshness posture under backpressure without
disconnecting clients or corrupting canonical meaning.

This exists to protect memory and event-loop health under slow clients and
hostile network conditions.

### Authentication, Authorization, And Remask

#### Middleware-enforced authentication

Authentication applies uniformly to HTTP, WebSocket, files, and lease
operations.

This exists to make unprotected endpoints structurally difficult to create.

#### Subscription-scoped authorization

Authorization applies at lease creation and at delivery time. Permission drift
must affect active leases, not just new ones.

This exists to prevent stale authorization from becoming an information leak.

#### Remask-aware network projection

The server projects Query-owned remask posture directly. A delivery may be
supported, remasked, or denied before network emission.

This exists so auth/policy narrowing remains part of canonical runtime meaning
rather than transport-local filtering.

### Regulated Deployment Architecture

#### Audit-grade provenance and operator evidence

The server should preserve enough structured provenance, policy posture,
delivery reasoning, and denial evidence that regulated operators can explain
what was exposed, why it was exposed, and which upstream basis or policy
decision authorized it.

This exists so regulated deployments do not have to bolt on a second audit
story beside the runtime.

#### Compliance-friendly topology and residency controls

The server should support topology, tenancy, retention, and routing decisions
that make data-sovereignty, residency, and environment-boundary posture
explicit rather than host folklore.

This exists so regulated systems can adopt WORTH without rewriting the server
for each compliance boundary.

#### Honest restart, retention, and recovery posture

The server must distinguish runtime-backed availability from durable restart
closure in ways operators, auditors, and clients can all inspect.

This exists so compliance-sensitive systems are never forced to rely on
marketing language where capability boundaries should have been machine-checkable.

### Branch-Aware Delivery Architecture

#### Branch-scoped subscriptions

Clients can subscribe to specific branches, previews, or other admitted truth
bases rather than only the main branch.

This exists for preview environments, AI agents, branch comparison, and staged
review flows.

#### Branch-aware reads

HTTP reads can target branch heads, previews, or historical snapshots through
the same server surface.

This exists so the external API surface can preserve the same basis richness as
the local runtime.

### Observability And Provenance

#### Server diagnostics

The server must expose lease counts, connection counts, delivery rates by
class, outbox depths, coalescing ratios, evaluation latency, CDC lag, socket
health, and protocol error rates.

This exists for operational visibility and capacity planning.

#### Protocol-level provenance

Delivered patches and request/response envelopes should carry causal metadata
showing which upstream commit, cause family, basis, or comparator decision made
the delivery meaningful.

This exists so explanations survive at the network edge instead of being
relegated to server logs.

### View-Specific Delivery Architecture

#### View-shaped patch delivery

When Query declares a view shape such as table, grouped, timeline, chart, or
detail, the server projects that Query-owned intent into view-appropriate patch
formats.

The important rule is:

- Query owns canonical view meaning
- the server owns transport packaging, fanout, and pacing

This exists to make view patches stronger, not weaker. The server should not
invent patch meaning. It should faithfully project query-shaped meaning into a
transport-safe form clients can apply surgically.

#### Server-side view materialization

For hot collaborative surfaces, the server can maintain shared materialized
view results and emit incremental patches from that maintained view.

This exists as an optimization for high-fanout collaboration, not as a new
authority layer.

### Mutation Architecture

#### Schema-validated mutations

Every mutation flowing through the server is validated against current schema
and policy constraints before execution.

This exists to make the server boundary structurally honest about invalid input
and forbidden operations.

#### Optimistic mutation protocol

The strongest form of server optimism is not transport-local speculation. It is
branch-aware optimistic flow built on Query/Relational semantics:

- local optimistic rendering may occur in the client
- the server receives the declared mutation
- branch-aware or basis-aware admission determines whether the write can commit
- confirmation or rejection preserves canonical rollback and explanation truth

This exists so optimistic UX can stay fast without inventing a second
conflict-resolution model beside the runtime.

#### Distributed sagas and the outbox pattern

When the server coordinates with external APIs, cross-system workflows should
use explicit outbox/saga boundaries instead of pretending one synchronous
request owns all truth.

This exists to avoid split-brain state across WORTH and external SaaS systems.

### Zero-Trust And Cryptography

#### End-to-end encryption and blind servers

For high-security domains, the server may act as a blind relay where encrypted
payloads remain opaque while routing and structural reactivity still operate on
unclassified metadata.

This exists to extend the server into high-IP and high-security deployment
domains without abandoning the live sync model.

### Integration And Extensibility Architecture

#### Core server versus integration surfaces

The core server contract is:

- request/response APIs
- lease and sync delivery
- file/binary transfer
- auth/tenant/branch/policy routing
- observability

Integrations are a separate family layered on top of that core contract, not
the definition of the server itself.

This exists so Stripe/Auth0/webhooks do not become entangled with ordinary app
delivery semantics.

#### Integration-facing CDC surface

Raw CDC, schema-aware change feeds, and replication-oriented cursor semantics
belong to an explicit integration-facing surface.

This exists so:

- ordinary app clients consume Query-shaped meaning by default
- integration pipelines can still consume lower-shape change feeds honestly
- multi-store replication can exist without redefining ordinary product sync

#### First-party integration packages

The server may ship first-party integrations for a few common ecosystems where
WORTH-native security and truth wiring can be kept strong and audited.

This exists for high-value batteries-included integrations, not as a reason to
turn the core server into a SaaS-adapter pile.

#### Extensible integration wiring

For everything else, the server should expose structured wiring for webhooks,
polling, OAuth callbacks, and external signal ingestion.

This exists so the server can integrate broadly without claiming first-party
ownership of every external API.

### Distributed Scalability Architecture

#### Workspace affinity routing

For horizontal scaling, the server may route connections by workspace or tenant
affinity so shared subscription bases and collaboration locality remain strong.

#### Edge-to-cloud topology

The server may eventually support edge read replicas, regional routing, and
data-sovereignty-aware topology.

This exists as an explicit later-scale surface, not as a hidden assumption of
the first shipping server.

#### Control-plane gossip and distributed invalidation

Multi-node deployments may propagate invalidation or coordination signals
between nodes explicitly instead of relying solely on polling.

This exists so clustered delivery can remain reactive without lying about where
coordination lives.

### Multi-Tenant Architecture

#### Tenant workspace routing

Tenant identity may come from subdomain, header, path, or token and applies to
every request, lease, and file operation.

#### Tenant-scoped delivery

Active delivery is scoped to the resolved tenant. Shared-base optimization may
exist inside that scope, but tenant isolation must remain structural.

## Domain Fit

### Collaborative web applications

`worth-server` should support immediate multi-user state synchronization,
durable subscriptions, cursor-based resume, presence lanes, tolerance-aware
suppression, and shared subscription optimization.

### Mobile and offline-first applications

`worth-server` should support compound catchup, bandwidth-aware freshness
modes, background delivery modes, and explicit anti-entropy recovery.

### Regulated industries

`worth-server` should support auditable policy enforcement, provenance-bearing
delivery, remask-aware denial, tenant isolation, residency-aware topology,
explicit durability posture, and optional blind-server deployment shapes for
domains such as healthcare, finance, defense, and critical infrastructure.

### AI agent interfaces

`worth-server` should support branch-scoped subscriptions, provenance-bearing
patches, branch-aware mutations, snapshot-scoped reads, and stable evaluation
contexts for speculative or preview work.

### WORTH-native applications

`worth-server` should support direct consumption of typed server-managed Query
surfaces so product teams can build live applications without reconstructing
the same meaning as route handlers, generated clients, cache invalidation
layers, and bespoke sync code.

### Integration pipelines

`worth-server` should support durable CDC consumption, schema-aware change
feeds, exactly-once cursor semantics, outbox/saga coordination, and structured
resume.

## Roadmap Direction

This file is a vision document, not the execution roadmap. But the future work
should be derivable from it.

The highest-signal server programs are:

- axum server with typed middleware and explicit facade seams
- Query-first request/response APIs
- direct WORTH-native app facade surfaces that can replace large families of
  handwritten endpoints
- runtime-backed lease and sync delivery built on the Query downstream
  delivery contract
- signal-native subscription evaluation through bridge and Query contracts
- server-managed subscriptions as durable projections of Query live meaning
- sync protocol message types, cursor semantics, basis negotiation, and
  compound catchup
- typed delivery classes, presence lane, invalidation lane, and patch lane
- freshness-mode negotiation and adaptive backpressure degradation
- multiplexed framing and priority-safe protocol transport
- protocol-level provenance and server diagnostics
- regulated-deployment evidence, topology, and audit posture
- branch-aware reads and branch-scoped subscriptions
- remask-aware active delivery
- file upload handling and binary asset management
- view-shaped patch delivery and server-side materialization
- optimistic mutation flow over branch-aware runtime semantics
- integration-facing CDC surfaces, background delivery, and outbox/saga
  coordination
- durable lease persistence and restart-stable resume later, when upstream
  contracts honestly admit them
- first-party integrations where honest and extensible integration wiring for
  everything else
- multi-node coordination, affinity routing, edge/cluster work, and
  data-sovereignty-aware topology
- optional end-to-end encrypted or blind-server deployment modes
- WebTransport as a later transport upgrade

If a capability is named here and not yet built, it is roadmap work.

If a capability is built but not yet proven under concurrent load, connection
churn, permission change, branch drift, and network failure, it is
certification work.

## Non-goals

- turning the server into a truth runtime or storage engine
- owning signal evaluation logic or bridge causality logic
- redefining Query meaning at the transport edge
- inventing a new transport protocol when the innovation is the
  application-layer sync model
- replacing HTTP for operations that are naturally request/response
- building a generic pub/sub system with no query or bridge semantics
- treating presence and truth updates as the same delivery class
- pretending runtime-backed resume already implies durable restart-stable
  resume
- letting CDC become the default ordinary client contract when Query-shaped
  delivery is the intended app-facing surface
- forcing WORTH-native apps to rebuild their product surface as large
  handwritten endpoint families when typed server-managed Query seams could
  have carried the same meaning directly

## Companion Documents

- [plans/worth-relational/worth_relational_vision.md](../WORTH-relational/WORTH_relational_vision.md)
- [plans/worth_signal/worth_signals2.md](../WORTH_signal/WORTH_signals2.md)
- [plans/worth-runtime-bridge/worth_runtime_bridge_vision.md](../WORTH-runtime-bridge/WORTH_runtime_bridge_vision.md)
- [plans/worth-store/worth_store_vision.md](../worth-store/worth_store_vision.md)

Query-first semantic intake, signal-backed subscription evaluation,
server-managed durable-in-design subscriptions, cursor-based resume, typed
delivery classes, basis negotiation, remask-aware delivery, branch-aware
projection, and explicit runtime-backed versus durable capability boundaries are
what make this server more than "REST plus WebSocket." If those are weak, the
runtime stack delivers its precision only to local in-process consumers, and
the network boundary becomes the place where all that architectural investment
is thrown away.
