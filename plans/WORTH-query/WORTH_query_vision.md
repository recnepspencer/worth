# WORTH Query Vision

## Thesis

`worth-relational` defines truth. `worth-store` persists it. `worth-signal`
derives from it. `worth-query` is how consumers ask for it.

`worth-query` is the typed, composable, aspect-aware query layer for
graph-native truth systems. It translates developer intent -- "give me
these entities, with these aspects, filtered by these conditions, projected
to this shape" -- into efficient reads against the truth runtime and store,
with native support for live subscription promotion, branch-scoped reads,
time-travel, lineage traversal, and incremental result maintenance.

It is not SQL. It is not GraphQL. It is a query model designed from the
ground up for truth systems that understand identity, aspects, branches,
lineage, schema evolution, and reactive change propagation.

The query layer exists because the gap between "truth exists" and "developers
can efficiently ask for exactly the truth they need" is where most systems
collapse into either too-powerful-to-optimize (arbitrary SQL) or
too-limited-to-express (REST endpoints for everything). `worth-query` fills
that gap with a query model that is expressive enough for real applications
and constrained enough for the runtime to optimize, narrow, and subscribe.

That public story now also includes eleven compact product-facing layers on top
of the retained declaration-entry pipeline:

- a typed binding pipeline for turning current context or retained artifacts
  into the next explicit Query input
- a denial-preserving ordinary outcome layer that keeps ordinary call sites
  concise without flattening checked or proof-visible topology
- a prepared/executed continuation pipeline that turns retained continuation
  truth into one explicit continuation contract without hiding workspace,
  runtime, basis, or execution posture
- a signal-compatibility orchestration layer that keeps retained compatibility,
  prepared continuation, and explicit execution as separate public states
- a contribution-composed orchestration layer that keeps declaration-entry
  lowering and declaration-scoped contribution authoring on one public surface
  without pretending they are one proof chain
- an orchestration inventory and audit layer that keeps exported orchestration
  verbs, ordinary lanes, transcripts, docs, and certification synchronized as
  one public contract instead of letting those drift independently
- a typed-stop remediation surface that turns ordinary, checked, and
  proof-visible stop posture into one explanation and recommended application
  action without granting execution authority
- a family-helper projection layer that gives major declaration families
  domain-native helper surfaces while still compiling onto the same generic
  binding, orchestration, ordinary, remediation, and audit lanes
- a grouped-authoring layer that turns meaningful neighborhood-shaped
  declarations into retained grouped artifacts, grouped orchestration, and
  grouped remediation without degrading into loops over isolated single-member
  declarations
- a public-doc-coverage layer that keeps feature docs, README discovery,
  golden readouts, and teaching journeys synchronized with the live public
  orchestration surface instead of letting documentation drift into a second
  invented product
- a platform-entry closeout layer that turns the shipped orchestration,
  docs/goldens, compile-fail, parity, and hostile proof surfaces into one
  machine-checkable certification ledger rather than a bag of local green
  checks

## What This Query Layer Is For

`worth-query` exists for every consumer that needs to read, filter, project,
traverse, or subscribe to truth -- whether that consumer is a web frontend,
a server-side handler, an AI agent, a geometry kernel, a CLI tool, or another
WORTH component.

It is meant to support:

- web applications that need typed, filtered, projected reads of current
  truth with automatic promotion to live subscriptions
- geometry kernels that need subgraph traversal, topological neighborhood
  queries, and aspect-scoped reads over branch-local truth
- AI systems that need branch-scoped reads, historical comparison queries,
  and lineage traversal for explanation and decision support
- dashboard and reporting surfaces that need aggregated, tolerance-aware
  reads with incremental refresh
- workflow and editor tools that need collection queries with sorting,
  pagination, and live updates
- integration pipelines that need structured CDC-shaped query output for
  downstream consumption
- admin and operator tools that need branch inspection, commit history
  traversal, and snapshot diffing

The technical thesis is the same across all of them:

- queries must be typed and composable, not string-based or ad hoc
- the query model must be aware of aspects, not force whole-entity reads
- live subscription promotion must be a native query capability
- branch-scoped and time-travel reads must be expressible without separate
  APIs
- the query layer must give the runtime enough structure to optimize,
  narrow, and incrementally maintain results
- queries must not bypass the truth runtime's semantic contracts

## Why This Query Layer Is Different

These are not optional add-ons. They are the capabilities that make
`worth-query` strategically different from ordinary query languages and ORMs:

- aspect-aware projection (query specific aspects, not whole entities)
- typed composable query expressions, not SQL strings
- native live query promotion (read -> subscribe with no API change)
- branch-scoped reads as a query parameter, not a separate API
- time-travel reads as a query parameter (at commit, at snapshot, at time)
- lineage-aware traversal (follow identity evolution across versions)
- subgraph-scoped queries (topological neighborhood, dependency cone,
  assembly region)
- incremental result maintenance for live subscriptions
- tolerance-aware aggregation (suppress updates below threshold)
- schema-aware query validation at construction time
- CDC-shaped output for integration consumption
- query-to-signal bridging for reactive derived computation
- pagination and cursor-based traversal as first-class query features
- diff queries across branches, versions, and time ranges
- correspondence queries across branches via lineage
- named scopes -- reusable, composable query fragments
- view shapes -- intent-driven query types (table, kanban, timeline, chart)
- eager relation loading with depth control
- query templates with parameter slots
- saved and named query definitions as workspace artifacts
- workflow-aware predicates (branch state, approval state, role-scoped)
- structured content aspect queries (queryable typed rich text)
- relational rollups and cross-entity computed fields
- result shape declarations for delivery contracts
- inspector-pattern entity detail with live aspect projection
- policy-aware aspect masking -- aspects the user cannot see are never queried
- tenant-scoped query narrowing -- automatic branch scoping per tenant

If these are treated as separate APIs bolted onto traditional read paths,
the developer experience fractures into "reads here, subscriptions there,
history somewhere else, branches through a different endpoint."

## Mission

`worth-query` exists to make asking for truth as precise, composable, and
efficient as the truth runtime makes storing it.

It must answer these questions as native query responsibilities:

- How do developers express exactly which entities, aspects, and relations
  they need without over-fetching or under-fetching?
- How does a one-time read become a live subscription without changing the
  query expression?
- How do developers read from a specific branch or a specific point in
  history without using a different API?
- How does the query layer validate queries against the current schema at
  construction time?
- How do subgraph and neighborhood queries express topological scope without
  falling back to arbitrary graph traversal?
- How do diff and comparison queries express "what changed between these
  two versions" as a first-class operation?
- How does the query layer give the runtime enough information to narrow
  CDC, optimize reads, and incrementally maintain results?
- How do aggregation and collection queries express tolerance and
  suppression policies for live delivery?
- How does the query layer support pagination, cursors, and bounded result
  sets for large collections?
- How do lineage queries traverse identity evolution without exposing
  internal lineage graph structure?

## Architectural Model

### Runtime stack

| Layer | Responsibility | Owns |
| --- | --- | --- |
| `worth-relational` | Truth semantics | identity, transactions, MVCC, diffs, CDC, lineage, schema |
| `worth-store` | Durable storage | commit persistence, snapshots, WAL, recovery, backends |
| `worth-signal` | Derived computation | dependency DAG, invalidation, recomputation, scheduling |
| `worth-query` | Query expression and planning | query types, composition, validation, planning, execution, result shapes |
| Bridge / integration | Decoupled coordination | patch-to-invalidation, aspect mapping, snapshot evaluation |
| `worth-server` | Network delivery | subscriptions, cursors, delivery classes, HTTP surface |
| WORTH Cloud | Managed platform | workspaces, tenants, policies, admin, observability |

### Ownership boundary

`worth-query` owns:

- query expression types and composition API
- schema-aware query validation
- query planning and optimization
- aspect projection and narrowing
- result shape definition and projection
- pagination, cursor, and bounded traversal
- branch-scoped and time-travel read parameterization
- subgraph and neighborhood scope expressions
- diff and comparison query expressions
- lineage traversal query expressions
- aggregation expressions with tolerance policies
- live query promotion semantics
- query-to-signal bridging for incremental maintenance
- CDC-shaped output formatting
- query execution against the truth runtime and store

`worth-query` does not own:

- truth semantics, identity semantics, or transaction semantics
- storage layout, compaction, or recovery
- signal evaluation, dependency tracking, or reactive scheduling
- sync protocol delivery or subscription management
- authentication or authorization enforcement
- domain-specific meaning of entities, relations, or aspects
- network transport or message framing

### Structural rule

`worth-query` translates developer intent into efficient, validated,
narrowed reads against the truth runtime and store. It does not define
truth semantics, own storage behavior, or manage delivery. It produces
typed, structured result sets that higher layers (server, cloud) can
deliver, cache, and subscribe to.

## Principles

1. Queries are typed, composable expressions -- not strings, not magic, not
   framework conventions.
2. Aspect projection is the default -- whole-entity reads are a special case,
   not the norm.
3. A query that works as a one-time read must work as a live subscription
   with no API change.
4. Branch and time-travel are query parameters, not separate APIs.
5. Schema validation happens at query construction time, not at execution
   time.
6. The query model must be constrained enough for the runtime to optimize
   and incrementally maintain.
7. Subgraph scope, lineage traversal, and diff queries are first-class
   expression types, not afterthoughts.
8. Pagination and bounded results are native query features, not
   application-layer workarounds.
9. Tolerance and suppression policies are expressible per query for live
   subscriptions.
10. Queries must not bypass or weaken the truth runtime's semantic contracts.
11. Ordinary consumer reads **must** originate in `worth-query`. Direct
    runtime/store reads are infrastructure escape hatches reserved for
    system internals, certification harnesses, and exceptional hot paths.
12. Query results must carry enough metadata for the server to deliver them
    efficiently (aspect masks, entity scopes, tolerance policies, cursor
    positions).

## Foundational Decisions

These are locked architectural decisions:

- queries are Rust types, not strings or DSLs parsed at runtime
- aspect projection is explicit in every query expression
- live promotion is a property of the query execution context, not a
  different query type
- branch and version targeting are parameters on the query context, not
  separate query APIs
- schema validation is compile-time where possible, construction-time
  otherwise
- query planning operates on the truth runtime's knowledge of schema,
  aspects, and storage layout
- result shapes are typed and declared, not arbitrary dynamic maps
- pagination uses opaque cursors, not offset/limit
- subgraph scope uses typed boundary expressions, not arbitrary traversal
  predicates
- diff queries produce structured change sets, not raw deltas
- the query layer does not cache results -- caching is a store and server
  responsibility
- queries carry enough metadata for the signal graph to evaluate
  subscription relevance

## Capability Pillars

### Query Expression Architecture

#### Typed composable query expressions

Technical role:
Every query is a typed Rust expression that declares: what entity types to
read, which aspects to project, what filters to apply, what ordering and
pagination to use, and what result shape to produce. Queries compose through
typed combinators, not string concatenation.

What this enables:

- compile-time or construction-time validation against the schema
- IDE autocompletion and type checking for query construction
- query composition without injection risks or parsing ambiguity
- the runtime can inspect query structure for optimization and narrowing

#### Aspect-aware projection

Technical role:
Every query explicitly declares which aspects it needs. The query layer
never reads whole entities by default -- it reads the declared aspect
projection and nothing more.

What this enables:

- reads that touch only the aspects the consumer needs
- CDC narrowing scoped to the declared aspects
- storage-aligned reads when the store uses aspect-aware layout
- bandwidth reduction for network delivery of query results
- clear ownership of what data flows to which consumer

#### Filter and predicate expressions

Technical role:
Queries support typed filter predicates over aspect fields. Filters compose
through logical operators and can express equality, comparison, range,
membership, and pattern conditions over typed fields.

What this enables:

- server-side filtering before results reach the consumer
- filtered live subscriptions where only matching changes trigger delivery
- pushdown of filter evaluation to the store when possible
- typed filter composition without stringly-typed predicate building

#### Ordering, pagination, and cursors

Technical role:
Collection queries support typed ordering expressions, cursor-based
pagination, and bounded result sets. Cursors are opaque tokens that encode
position without exposing internal offsets.

What this enables:

- efficient large-collection traversal without unbounded memory
- stable pagination under concurrent mutations (cursor-based, not offset)
- ordered live subscriptions where new results arrive in sort position
- bounded initial load followed by cursor-based advancement

### Type-Bound Execution Architecture

#### Implicit topological binding (Route-model binding generalized)

Technical role:
The query layer owns one shared proof-bearing binding substrate, starting
with one retained target-binding core that both contribution authoring and
declaration-entry orchestration consume before the shipped first
extractor/resolver expansion in Phase 25 and later follow-on widening, taking
inspiration from Laravel's route-model binding but translating it into
Rust-native typed extractors, retained-artifact resolvers, capability
witnesses, and family-scoped binding contracts. Consumer functions (UI
components, controller endpoints, kernel solvers, contribution authors, and
later orchestration/continuation callers) declare their data needs as typed
signature inputs or typed request forms rather than ambient host lookups.

The runtime binds declared context and retained proof into the right admitted
Query artifact, executes against canonical query meaning, and exposes the same
binding story across direct query execution, domain capability contribution,
declaration-entry orchestration, and later continuation surfaces.

The first shipped slice now covers both sides of that seam:

- context-bound declaration, route, receipt, envelope, and continuation-ready
  request preparation
- retained progression / route / receipt / envelope target resolution into the
  next explicit Query input

What this enables:

- **Web URLs** can implicitly bind to an entity or collection
- **Workflow context** can implicitly bind to a pending-approval scope
- **Geometry triggers** can implicitly bind to a neighborhood/subgraph traversal
- **AI execution contexts** can implicitly bind to a speculative branch or time window
- contribution authoring and later declaration/continuation ergonomics can
  reuse the same binding substrate instead of inventing local glue families
- declaration-entry and later continuation binding should narrow by semantic
  aspect contract, not by raw target-string folklore or broad artifact class
- eradication of explicit "data fetching" and "loading/error states" from
  consuming code where the binding contract already proves what can be safely
  resolved

### Live Query Architecture

#### Read-to-subscribe promotion

Technical role:
Any query that can be evaluated as a one-time read must be promotable to a
live subscription by changing the execution context, not the query itself.
The promoted subscription receives incremental patches as truth changes.

What this enables:

- the simplest possible developer experience -- write one query, use it for
  both read and subscribe
- no separate "subscription query" API or syntax
- live dashboards, collaborative surfaces, and real-time lists from the
  same query that serves one-time loads
- gradual adoption where developers start with reads and promote to live
  when they need it

#### Incremental result maintenance

Technical role:
Live query subscriptions must receive incremental patches that correspond
to the query's projection, filters, and ordering -- not raw CDC events that
the consumer must re-evaluate.

What this enables:

- consumers receive "row 7 changed field X to value Y" not "entity 42
  aspect Foo was modified" -- the result is query-shaped, not event-shaped
- collection splices ("inserted at position 3," "removed from position 9")
  for ordered live collections
- filtered subscriptions that suppress changes that do not match the
  filter predicate
- tolerance-aware suppression for aggregation queries where small changes
  below threshold are held

#### Query-to-signal bridging

Technical role:
Live queries are backed by signal graph nodes. The query layer translates
query expressions into signal subscriptions so that the reactive evaluation
infrastructure handles dependency tracking, invalidation, and
recomputation automatically.

What this enables:

- live queries benefit from all signal graph optimizations (suppression,
  coalescing, aspect-scoped invalidation)
- no custom change-tracking code for live query maintenance
- derived computation can depend on query results through the same signal
  graph
- the server can evaluate subscription relevance through the signal graph
  rather than per-client re-evaluation

### Branch and Time-Travel Architecture

#### Branch-scoped queries

Technical role:
Every query context can target a specific branch. If no branch is specified,
the query reads from the current branch head. Branch targeting is a
parameter on the query context, not a different query type or API.

What this enables:

- reading draft branch truth with the same query expressions used for
  main-branch truth
- branch comparison views where multiple query instances target different
  branches
- preview environments served by branch-scoped queries
- AI agent evaluation against speculative branches

#### Time-travel queries

Technical role:
Query contexts can target a specific commit, a specific snapshot, or a
relative time point. Time-travel is a context parameter that applies to
any query expression.

What this enables:

- historical reads with the same query expressions used for current reads
- "what did this entity look like at this commit?" without separate APIs
- audit and compliance reads against specific historical points
- regression analysis by comparing query results across time

#### Diff queries

Technical role:
Diff queries express "what changed between version A and version B" as a
first-class query type. The result is a structured change set shaped by
the query's aspect projection, filters, and scope -- not raw storage deltas.

What this enables:

- "what changed in this collection since my last read?" as a query
- branch-to-branch comparison with typed, projected results
- audit reports generated from structured diff queries
- AI agents comparing branches with precise, query-shaped diffs.

### Subgraph and Traversal Architecture

#### Subgraph-scoped queries

Technical role:
Queries can express topological scope: a neighborhood, a dependency cone,
an assembly region, a connected component, or a bounded traversal from a
starting entity. Scope is expressed through typed boundary expressions, not
arbitrary graph traversal predicates.

What this enables:

- geometry kernel reads that load exactly one assembly region
- chip design reads that load exactly one timing cone
- dependency-aware reads that follow relation edges to a bounded depth
- efficient reads that match the store's multi-resolution materialization

#### Relation traversal expressions

Technical role:
Queries can follow typed relations between entities, expressing joins,
adjacent entity reads, and multi-hop traversals through the relation graph.
Traversals are typed by relation kind and bounded by explicit depth.

What this enables:

- loading an entity with its related entities in one query
- following parent-child, containment, or dependency relations
- bounded graph traversal without unbounded fan-out
- query-shaped results that include traversed related entities

### Lineage and Correspondence Architecture

#### Lineage traversal queries

Technical role:
Queries can follow identity evolution: "what did this entity used to be?"
"what replaced this entity?" "what did this entity split into?" Lineage
traversal operates through the lineage graph without exposing internal
lineage event structure.

What this enables:

- identity history for any entity
- tracing entity evolution across schema changes and refactors
- merge tooling that understands what corresponds to what
- audit trails that follow identity through splits and replacements

#### Correspondence queries

Technical role:
Queries can express cross-branch correspondence: "what in branch B
corresponds to this entity in branch A?" Correspondence queries use
lineage and structural fingerprint data to resolve cross-branch identity.

What this enables:

- merge UIs that show corresponding entities across branches
- analysis reuse detection across branches
- cross-branch diff views backed by identity correspondence
- AI agents finding equivalent structures across speculative branches

### Aggregation Architecture

#### Typed aggregation expressions

Technical role:
Queries can express count, sum, min, max, average, and custom aggregations
over projected aspect fields. Aggregations can be grouped by typed
group-by expressions.

What this enables:

- dashboard metrics computed by the query layer, not the consumer
- server-side aggregation that reduces result size
- grouped aggregations for reporting and analytics
- aggregation results that are incrementally maintainable for live
  subscriptions

#### Tolerance-aware aggregation

Technical role:
Aggregation queries for live subscriptions can declare tolerance thresholds.
Changes below the tolerance threshold are suppressed -- the consumer does
not receive a delivery until the aggregate changes meaningfully.

What this enables:

- dashboard metrics that do not flicker on trivial changes
- bandwidth-appropriate live aggregation for high-churn sources
- tunable precision per consumer need
- server-side suppression that reduces delivery cost

### Result Shape Architecture

#### Typed result shapes

Technical role:
Every query declares its result shape as a typed structure. Results are not
arbitrary dynamic maps -- they are typed projections shaped by the query's
aspect projection, traversal, and aggregation expressions.

What this enables:

- compile-time or construction-time type safety for query consumers
- serialization efficiency for known result shapes
- typed SDK generation from query definitions
- clear contracts between query producers and result consumers

#### CDC-shaped output

Technical role:
Query results can be formatted as CDC-shaped output for integration
consumption: structured change feeds shaped by the query's projection
and filters, suitable for downstream pipeline consumption.

What this enables:

- integration pipelines that consume query-shaped change feeds
- downstream systems that receive structured, filtered, projected changes
- durable subscription-based integrations through the same query model
- clear separation between "query for humans" and "query for pipelines"
  while using the same underlying query expressions

### Query Composition Architecture

#### Scopes as first-class domain vocabulary

Technical role:
Queries support reusable, composable query fragments -- named scopes -- that act as the primary bridge between the typed runtime and pragmatic app development. A scope is not just a convenience filter; it is a schema-validated, policy-aware, composable, and subscription-safe boundary.

You want developers writing code that reads conceptually like:
`active()`, `pending_approval()`, `changed_since_divergence()`, `visible_to(actor)`, `assembly_region(root, depth=3)`

What this enables:

- reusable business rules expressed as strict compiler-checked domain vocabulary
- composition of multiple scopes without manual filter merging
- shared ubiquitous language across geometry kernels, AI agents, and web backends
- scopes that inherently respect aspect-masking and Zanzibarian relationship proofs

#### Saved and named query definitions

Technical role:
Query definitions can be persisted as first-class workspace artifacts.
A saved query captures the full query expression -- projections, filters,
scopes, ordering, view shape -- as a named, shareable, subscribable
definition.

What this enables:

- shared views across team members ("the team's active tasks" as a named
  query anyone can subscribe to)
- server-managed query definitions that multiple clients consume
- named queries as the basis for dashboard widgets and reporting
- parameterized saved queries that accept runtime arguments

#### Query templates with parameter slots

Technical role:
Query templates define a query shape with typed parameter slots. Consumers
instantiate templates by binding parameters, producing fully specified
queries. Templates are reusable across contexts and parameterizable without
redefining the full query expression.

What this enables:

- reusable query patterns across features and modules
- parameterized saved queries where the same shape serves different contexts
- SDK generation from template definitions
- query variants that share structure but differ by parameters

#### Bounded relational materialization

Technical role:
Queries can declare relation chains to eagerly materialize in a single query operation. Unlike traditional ORM "eager loading," this is not just "load some relations too." It strictly brings back exactly the bounded relational/materialized neighborhood required by this consumer contract, with explicit aspect projection per semantic hop.

What this enables:

- loading an entity with its exact bounding box of dependencies in one operation
- declarative prevention of N+1 access patterns via strict topological depth constraints
- graph fan-out bounded mathematically by the query's materialization contract
- per-relation aspect projection (load related entities with only the aspects needed for that specific layer of the subgraph)

### View Shape Architecture

#### Intent-driven view shapes

Technical role:
Queries can declare a view shape that communicates the presentation intent
of the result. View shapes are not just grouping and sorting -- they
communicate how the consumer will render the results, which the query layer
and signal graph can use for optimization and narrowing.

Supported view shapes include:

- **table view**: flat collection with column projection, sort, and
  paginate
- **kanban view**: grouped by a status or category aspect, with count per
  group and collection splices per group for live updates
- **timeline view**: ordered by a date range aspect, with range-overlap
  filtering
- **chart view**: aggregation with group-by, tolerance-aware for live
  suppression of trivial changes
- **detail view**: single entity with full aspect projection and live
  updates
- **inspector view**: single entity with selective aspect projection,
  optimized for zero-latency live property inspection

What this enables:

- the query layer and signal graph can narrow invalidation to the aspects
  that matter for the declared view shape (a kanban view only re-evaluates
  when the grouping aspect changes)
- the server can optimize delivery format based on view shape intent
- live subscriptions can use view-shape-specific patch formats (collection
  splices for table, group membership changes for kanban)
- the same underlying truth serves radically different presentation
  lenses without separate query APIs

#### Workflow-aware predicates

Technical role:
Filter predicates can express workflow-level concepts that are backed by
branch state, aspect state, and context:

- items pending current user's approval
- items on a draft branch
- items that changed since the branch diverged from its base
- items assigned to the current user or team
- items in a specific workflow stage

What this enables:

- workflow UIs where the query naturally expresses "show me what I need to
  act on" without manual filter construction
- branch-aware queries where "drafts" and "pending review" are predicates,
  not separate APIs
- role-scoped queries that narrow based on the current user's context
- approval and review surfaces driven by query predicates rather than
  custom application logic

### Rich Content Query Architecture

#### Structured content aspect queries

Technical role:
Rich text and structured content are first-class aspect types with
schema-enforced structure. Content aspects are not opaque blobs -- they
are composed of typed content blocks (headings, paragraphs, checklists,
embedded references, media refs) that can be queried, filtered, and
projected individually.

What this enables:

- querying documents by content structure ("documents with unchecked
  checklist items," "documents mentioning entity X")
- projecting specific content blocks without loading the entire content
  aspect
- CDC narrowing to specific content blocks for efficient live updates
- schema-enforced content structure that prevents the untyped chaos of
  "anything goes" block editors
- content that participates in the full query model: filterable, sortable,
  subscribable, branch-aware

### Relational Computation Architecture

#### Relational rollups

Technical role:
Queries can express aggregations over related entities through typed
relation edges. Rollups compose relation traversal with aggregation to
produce cross-entity computed fields as part of the query result.

What this enables:

- "count of related tasks where status is done" as a query expression
- completion percentages, totals, and summaries computed from related
  entities without application-level aggregation
- rollup results that are incrementally maintainable through the signal
  graph for live subscriptions
- multi-level rollups (aggregate over relations of relations)

#### Query-time derived fields

Technical role:
Queries can declare computed fields derived from aspect field values.
Derived fields are evaluated at query time and are included in the result
shape alongside projected aspect fields.

What this enables:

- computed display values without stored redundancy
- formatted, combined, or transformed fields in query results
- derived fields that participate in filtering and sorting
- result shapes that match presentation needs without post-processing

### Result Transformation Architecture

#### Result shape declarations for delivery (API Resources)

Technical role:
Queries can declare an explicit result shape that defines the delivery-ready structure. Inspired by Laravel API Resources, but with a critical distinction: WORTH result shapes are declared as part of the query expression itself, not as a post-fetch wrapper. This means query planning, narrowing, schema validation, and live maintenance all understand the delivery structure structurally before the data is even fetched from disk.

What this enables:

- query results that arrive in the shape the consumer needs natively, not transformed after-the-fact in the application layer
- CDC subscriptions that narrow invalidation optimally because the signal graph understands the delivery shape
- typed delivery contracts between the query layer and the server
- SDK generation mapped exactly to the runtime's execution plan

### Policy-Aware Query Architecture

#### Graph-native relationship proofs (ReBAC constraints)

Technical role:
Permissions and constraints are natively embedded as Relationship Calculus (inspired by Zanzibar) directly within the query. The query layer mathematically evaluates "Subject-Relation-Object" tuples. Access control is not an external boolean flag check; it is an intrinsic graph traversal proof. A query mathematically yields empty results if the unbroken lineage sequence ("User A is on Team B which owns Object C") does not exist.

What this enables:

- web application access control that is mathematically robust and scales arbitrarily across deep ownership trees
- geometry kernel domain constraints ("Is part A structurally allowed to mutate part B?") resolved as native unified relational queries
- eradication of ad-hoc application code checking `is_admin()` or `has_write_access()`; the query itself proves legality 
- security policies that leverage the identical relation-graph query syntax used by regular domain logic

#### Aspect-level policy masking

Technical role:
The query layer enforces aspect-level access policies structurally. When
a policy declares that a user role cannot see a specific aspect, the query
layer removes that aspect from the projection before execution. The aspect
is never read from storage, never included in results, and never delivered.

Policies are declared against the schema, not per query. The query layer
consults the active policy context (user identity, roles, tenant,
branch permissions) and masks the query's aspect projection accordingly.

What this enables:

- sensitive data (salary, PII, internal notes) is structurally invisible
  to unauthorized users, not filtered after the fact
- zero per-query authorization code -- policies are declared once against
  the schema
- policy changes take effect immediately on existing live subscriptions
- CDC narrowing respects policy masks, so unauthorized aspect changes
  never trigger delivery
- aspect masking composes with all other query features (scopes, view
  shapes, aggregation, rollups) without special cases

#### Branch-level access scoping

Technical role:
The query layer enforces branch-level access policies. Queries targeting a
branch the user cannot access are rejected at validation time. Queries that
do not specify a branch are scoped to the branches the user has permission
to read.

What this enables:

- draft branches visible only to their creators or designated reviewers
- protected branches that require elevated permissions to read
- branch-level isolation for multi-tenant or team-scoped deployments
- branch access enforcement at the query layer, before storage access

### Multi-Tenant Query Architecture

#### Automatic tenant branch scoping

Technical role:
In multi-tenant deployments where tenants are isolated via branches, the
query layer automatically scopes every query to the resolved tenant's
branch. Developers writing queries for a multi-tenant application do not
need to specify or manage branch scoping -- the tenant context provides it.

What this enables:

- multi-tenant applications where every query is tenant-scoped without
  explicit branch parameters in application code
- shared base data (system configuration, templates, default values)
  visible through automatic branch inheritance
- tenant-specific truth that is structurally isolated, not just filtered
- the same query expressions used for single-tenant and multi-tenant
  deployments without code changes

#### Tenant-scoped schema awareness

Technical role:
In multi-tenant deployments with branch-local schema evolution, the query
layer validates queries against the tenant's active schema, not the global
base schema. If a tenant has branch-local schema customizations, queries
against that tenant's truth are validated and projected according to the
tenant's schema.

What this enables:

- per-tenant custom fields and schema extensions without affecting other
  tenants
- query validation that respects tenant-specific schema boundaries
- schema-aware projections and filters that adapt to per-tenant schemas
- multi-tenant platforms where tenants can customize their truth model
  without breaking shared infrastructure

## Domain Fit

### Web Applications

`worth-query` should support:

- typed collection queries with filtering, sorting, and pagination
- detail queries with aspect projection
- live promotion for real-time lists and detail views
- aggregation queries for dashboard metrics
- branch-scoped reads for draft and preview workflows

Revolutionary use:
web developers write one typed query that works for initial load, live
updates, and historical reads -- instead of separate REST endpoints, GraphQL
resolvers, WebSocket event handlers, and polling loops.

### Geometry and CAD

`worth-query` should support:

- subgraph-scoped queries for assembly regions and topological neighborhoods
- relation traversal for dependency and containment graphs
- lineage traversal for identity evolution tracking
- branch-scoped reads for design experiments
- diff queries for version comparison

Revolutionary use:
geometry kernels can express "give me this assembly region with its topology
aspects, following containment relations to depth 3, on this branch, and
keep me updated" as a single typed query.

### AI Systems

`worth-query` should support:

- branch-scoped reads for speculative evaluation
- correspondence queries for cross-branch comparison
- lineage traversal for explanation and provenance
- time-travel queries for historical analysis
- diff queries for measuring branch divergence

Revolutionary use:
AI agents can query speculative branches, compare alternatives, trace
decision lineage, and subscribe to truth changes -- all through the same
typed query model, instead of cobbling together separate read, diff, and
history APIs.

### Chip Design and Simulation

`worth-query` should support:

- subgraph-scoped queries for timing cones and module neighborhoods
- aggregation queries with tolerance for large netlist metrics
- diff queries for regression analysis
- relation traversal for connectivity graphs
- time-travel queries for analysis at historical points

Revolutionary use:
chip design tools can express "give me the timing cone from this output pin,
with connectivity and timing aspects, diff against last week's version" as
a composable query -- instead of writing custom traversal and comparison code.

## Roadmap Direction

This file is a vision document, not the execution roadmap. But the future work
should be derivable from it.

The highest-signal query programs are:

- typed query expression types and composition API
- aspect-aware projection
- filter and predicate expressions
- ordering, pagination, and cursor-based traversal
- live query promotion and incremental result maintenance
- query-to-signal bridging
- branch-scoped and time-travel query contexts
- diff queries
- subgraph-scoped queries and relation traversal
- lineage traversal and correspondence queries
- typed aggregation with tolerance-aware suppression
- typed result shapes
- CDC-shaped output formatting
- schema-aware query validation
- query planning and optimization
- named scopes and query composition
- saved and named query definitions
- query templates with parameter slots
- eager relation loading with depth control
- intent-driven view shapes
- workflow-aware predicates
- structured content aspect queries
- relational rollups and query-time derived fields
- result shape declarations for delivery contracts
- policy-aware aspect masking
- branch-level access scoping
- automatic tenant branch scoping
- tenant-scoped schema awareness

If a capability is named here and not yet built, it is roadmap work.

If a capability is built but not yet proven under complex schema, large
graphs, concurrent live subscriptions, and cross-branch scenarios, it is
certification work.

## Non-Goals

- implementing a general-purpose SQL engine
- becoming a GraphQL runtime
- owning truth semantics or transaction semantics
- managing storage layout or compaction
- owning signal evaluation or reactive scheduling
- managing sync protocol delivery or subscription lifecycle
- defining authentication or authorization policy
- replacing the truth runtime's internal read paths for system-level access

## Companion Documents

- [worth_relational_vision.md](file:///c:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-relational/worth_relational_vision.md)
- [worth_signals2.md](file:///c:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth_signal/worth_signals2.md)
- [worth_store_vision.md](file:///c:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-store/worth_store_vision.md)
- [worth_server_vision.md](file:///c:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-server/worth_server_vision.md)
- [WORTH_cloud_vision.md](file:///c:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/WORTH-cloud/WORTH_cloud_vision.md)
- [worth_runtime_bridge_vision.md](file:///c:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/engineering/worth_runtime_bridge_vision.md)

The query layer is where developer experience lives or dies. If queries are
expressive, typed, composable, and natively promotable to live subscriptions,
the entire WORTH stack feels like a single coherent product. If developers
must assemble reads, subscriptions, diffs, and traversals from separate APIs,
the architectural coherence below the query layer is invisible and wasted.
