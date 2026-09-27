# Milestone 13.1 Engineering Spec: Cross-Runtime Granular Invalidation

> **Status:** Implemented; final frozen-source review pending
>
> **Prerequisite:** [Milestone 13](./milestone-13-plan.md) and its
> [closeout](./milestone-13-closeout.md)
>
> **Cross-runtime parents:**
> [Query Milestone 9.14](../WORTH-query/milestone-9.14.md),
> [Runtime Bridge aspect-native refactor](../WORTH-runtime-bridge/aspect_native_refactor.md),
> and [Query AI README](../../workspaces/worth-query/crates/worth-query/docs/AI_README.md)
>
> **Successor:** [Milestone 14](./milestone-14-plan.md)

## 1. Goal And Roadmap Placement

Milestone 13 made Signal invalidation producer-local, aspect-aware,
partition/detail-aware, proof-progressed, and measurably proportional to the
realized semantic frontier. Milestone 13.1 makes that precision usable through
the real WORTH platform path:

```text
Relational committed aspect truth
  -> Runtime Bridge installed semantic correspondence
  -> Signal scoped direct invalidation and performed execution
  -> Runtime Bridge performed delivery
  -> Query installed dependency impact and governed maintenance
  -> query-shaped live publication
```

The milestone is filed under WORTH Signal because it productizes the contract
sealed by Milestone 13 before Milestone 14 parallelizes its work stream. It is
not permission for Signal to own Runtime Bridge correspondence or Query
meaning. Each participating runtime keeps its existing authority:

- Relational owns authoritative entities, aspects, versions, snapshots, and
  commits.
- Runtime Bridge owns installed correspondence and lawful cross-runtime
  lowering.
- Signal owns local aspect slots, invalidation causes, evaluation policy, and
  performed invalidation execution.
- Query owns installed application dependencies, authorization, disclosure,
  impact admission, query maintenance, and publication.
- `worth-proof` owns reusable proof progression law, never owner-specific
  permission.
- `worth-foundational` owns portable aspect, locality, canonical-basis, and
  receipt vocabulary, never live runtime authority.

The central claim is:

> An exact committed truth change can retain every lawful unit of aspect and
> locality precision across Runtime Bridge, Signal, and Query, while every
> widening, suppression, rebind, and performed effect remains typed, current,
> owner-authorized, independently observable, and bounded by the relevant
> semantic delta rather than platform-wide installed state.

Milestone 13.1 is a required integration handoff between Milestones 13 and 14.
Milestone 14 may still change only execution placement and concurrency. It may
not be the first place where M13 precision reaches the platform.

## 2. Current Boundary

The platform already contains most local ingredients, but not one honest
end-to-end path.

### 2.1 Signal

Signal already has:

- exact producer-local reverse subscription lookup keyed privately by
  `ProducerAspectKey`
- separate unscoped, whole-partition, and exact partition/detail buckets
- scoped root invalidation through `ChangedRegion`
- compiler-visible invalidation progression
- `InvalidationPlanningEstimate` for predicted work
- `SignalInvalidationExecutionReceipt` and
  `SignalInvalidationRealizedCounters` for performed work

The installed bridge entry point, `apply_installed_aspect_changes`, currently
admits installed node/aspect capabilities but applies no changed regions and
returns only a count. It therefore discards locality before the M13 runtime can
use it and does not carry performed M13 evidence back to the bridge.

### 2.2 Runtime Bridge

Runtime Bridge already admits exact semantic dependencies containing
Foundational aspect contracts, projection masks, bindings, relevant change
kinds, source authority, and semantic locality. Its correspondence delivery
matches authoritative committed patch envelopes and installed Signal targets.

The current delivery path then converts every matched target to an
`InstalledSignalAspectCapability`, calls Signal's aspect-only change function,
and returns a correspondence receipt. It does not:

- lower matched record/partition/detail locality into Signal changed regions
- distinguish exact carriage from declared widening in the delivered product
- perform and carry a bounded Signal invalidation observation
- separate direct truth delivery from Signal-performed derived consequences
- expose a Query-readable performed delivery that retains current bridge,
  Signal, and source bindings

### 2.3 Query

Query already has strong local semantics:

- installed semantic dependencies keyed by exact `AspectKey`,
  `AspectIdentity`, `AspectContractRevision`, binding, change kind, and
  canonical field-path overlap
- dependency roles for projection, selection or membership, ordering,
  grouping, windows, conditions, workflow stages, and realized effects
- Query-owned authorization, disclosure, purpose, branch, basis, lease,
  lifecycle, and publication progression
- query-shaped live patch, suppression, reorder, re-execution, rebind,
  retirement, and terminal outcomes
- region-scoped planning and delivery artifacts

But the production primary application bridge still registers
`RegisteredEntityCoarseWidening`, and its invalidation sink reduces a bridge
delivery to target count plus snapshot identity. Region-scoped execution is
test-only. Query-local string-shaped `BridgeChangeSummary` and
`BridgeLocalitySlice` forms can describe examples but carry no Runtime Bridge
authority. `WorthQueryMutationDelta` and coarse consumer invalidation locality
cannot represent the full installed correspondence binding.

The result is a split platform: Signal can be precise locally, Bridge can
describe exact semantic correspondence locally, and Query can classify exact
dependency impact locally, while the ordinary composed path loses precision
and performed authority at both crossings.

## 3. Adversarial Cross-Runtime Courtroom

The decisive proof uses the real Relational commit source, Runtime Bridge
correspondence delivery, installed Signal runtime, Query operating-world root,
managed live execution, and public Query host observation. Reenacting the flow
with raw `SignalGraph`, Query-local bridge summaries, manually authored node
IDs, or direct counter injection is supplemental evidence only.

### 3.1 Production-Valid World

Extend the causally complete financial world from Milestones 12 and 13 into an
installed Query world containing:

- price, FX, curve, volatility, valuation, risk, portfolio, desk, and audit
  aspects with exact Foundational contracts and revisions
- rates positions bound to curve partitions and tenor details including `2y`,
  `5y`, `10y`, and `30y`
- exact, tolerance-suppressed, and condition-gated Signal computations
- Query projections, predicates, ordering, grouping, windows, relations, and
  conditional/workflow consequences over those outputs
- multiple live consumers sharing one maintenance lease while holding
  different disclosure and purpose authority
- unrelated installed mappings, Signal consumers, Query definitions, live
  leases, and result rows that vary independently from the affected frontier

Add one domain-neutral locality twin whose semantic regions are opaque to
Signal. It proves that no financial term, tenor rule, or geometry assumption
has entered Signal, Proof, or Foundational authority. Milestone 13.1 certifies
M13's exact unscoped/partition/detail base; Milestone 14 owns the bounded deeper
scope hierarchy.

### 3.2 Required Scenario Families

| Scenario | Hostile sequence | Defect it must convict |
|---|---|---|
| `curve_detail_to_live_risk` | commit one `CURVE` change for partition `usd-rates`, detail `5y`; retain matching `5y`, whole-partition, unscoped, sibling `10y`, `VOLATILITY`, audit, and unrelated-query consumers | coarse entity widening, partition/detail loss, aspect/scope cross-products, copied transitive aspects, or Query re-execution outside the exact impact set |
| `suppressed_quote_no_query_patch` | commit a quote move below producer tolerance, then a move above tolerance, with identical Bridge truth provenance | treating truth delivery as performed Signal consequence, publishing on comparator suppression, or requiring a Signal receipt for a direct Query truth patch that needs no Signal execution |
| `ordered_portfolio_membership` | change a field used by projection only, then predicate membership, then ordering, then window boundary, while irrelevant fields and collections grow | role collapse, whole-query re-execution, stale ordering/window state, or raw field events escaping instead of query-shaped patches |
| `shared_lease_disclosure_noninterference` | one exact change reaches consumers with shared computation but different purpose/disclosure; revoke one consumer after impact selection and before publication | shared authority, leaked scope/detail/timing, per-consumer duplicate maintenance, or publication without current Query revalidation |
| `correspondence_rebind_restore` | capture after installation, destroy all derived Bridge/Signal/Query indexes, restore into fresh runtime identities, rebind, then deliver a delayed old change and a current change | derived index authority, stale graph/installation/contract/snapshot/commit reuse, restored ready authority, or same-count substitution |
| `opaque_region_platform_twin` | repeat exact/off-region/unscoped delivery using non-financial opaque partition/detail tokens and WASM-compatible execution | domain meaning in Signal, bare-detail collision, host-only string heuristics, or platform-dependent authority |

### 3.3 Frozen Scale Axes

Each scenario declares independent scale axes before execution:

- installed Bridge correspondences: `10^2`, `10^3`, `10^4`, and scheduled
  `10^5`
- Signal direct subscribers: `16`, `256`, `1,024`, and scheduled `10^4`
- Query installed dependency declarations: `10^2`, `10^3`, `10^4`, and
  scheduled `10^5`
- live consumers sharing an affected computation: `1`, `32`, and `1,024`
- result rows: `10^2`, `10^4`, and scheduled `10^6`
- affected semantic frontier held constant at `1`, `4`, or `32` while each
  unrelated axis grows

Ordinary CI runs causally identical smaller cases and at least one independent
slope for every boundary. Scheduled lanes own the largest tuples. A resource
denial remains a typed reported outcome and cannot count as a passing scale.

### 3.4 Independent Manifest

Before the production commit, an independent
`CrossRuntimeInvalidationNecessityManifest` derives from the immutable world
definition and named mutation:

- `R`: authoritative Relational aspect changes and exact locality
- `B`: exact installed Bridge correspondences and declared widening decisions
- `S`: Signal installed node/aspect/region seeds and expected performed work
- `I`: Query installed dependency impacts by semantic role
- `M`: Query maintenance operations after sharing/coalescing
- `D`: consumer-specific query-shaped deliveries after current authorization
  and disclosure
- `X`: exact suppressions, rebinds, denials, or terminations

The manifest may consume domain declarations, installed contract definitions,
and the named mutation. It may not consume production Bridge matching, Signal
routing, Query impact indexes, live queues, performed receipts, diagnostics,
or observed counters.

Small cases compare exact identities in `R/B/S/I/M/D/X`. Scheduled cases may
compare canonical digests and cardinalities only when an ordinary-size twin
proves the same generator and expectation rules with complete identities.

### 3.5 Required Outcomes

The courtroom passes only when:

- the exact source aspect contract, identity, revision, binding, change kind,
  field path, partition, and detail survive every crossing where they remain
  semantically relevant
- Signal receives only node-local installed capabilities and canonical changed
  regions; it never receives Query meaning
- Query receives no raw Signal node, aspect slot, dirty set, cause set,
  `ProducerAspectKey`, ready batch, or private counter state
- direct truth delivery and Signal-performed derived delivery remain distinct
  typed variants
- a Signal performed receipt proves bounded Signal execution but cannot
  authorize Query impact, maintenance, or publication
- Query revalidates purpose, tenant, disclosure, branch, basis, installation,
  lease, and lifecycle after impact selection and before protected effects
- exact off-region changes produce no Signal candidate and no Query
  maintenance, except for consumers explicitly declared unscoped
- comparator suppression produces no derived Query patch
- shared computation is maintained once while delivery remains independently
  authorized per consumer
- every output remains query-shaped; raw CDC, partition, and Signal events do
  not become the public contract
- destruction of all derived indexes followed by authoritative reconstruction
  produces the same admitted identities and current results
- default, parallel-feature, and WASM-supported lanes preserve semantic and
  authority equality

### 3.6 Mutation Sensitivity

The suite must turn red for compile-valid mutations that:

- drop Bridge locality before Signal seed construction
- replace exact locality with registered-entity coarse widening
- pair one aspect with another aspect's partition/detail scopes
- use a bare detail token without its partition
- copy a source aspect or region onto a transitive descendant
- bypass the installed Signal capability check
- construct Query impact from a descriptive summary or raw Signal receipt
- treat `InvalidationPlanningEstimate` as performed execution
- publish a Query patch from Bridge truth without Query admission
- publish a derived Query consequence without required performed Signal
  execution
- skip post-selection authorization/disclosure revalidation
- scan all mappings, Signal subscribers, installed queries, leases, or result
  rows for a one-detail change
- serialize or restore a ready queue or derived index as authority
- merge direct truth and Signal-derived consequence into one ambiguous event

## 4. Product Decision Lock

### 4.1 One Directional Authority Chain

The ordinary chain is fixed:

```text
Relational committed truth
  -> Bridge matched semantic change
  -> Bridge prepared scoped Signal seeds
  -> Signal admitted direct invalidation
  -> Signal performed execution (when required)
  -> Bridge performed invalidation delivery
  -> Query selected impact candidates
  -> Query admitted current impact
  -> Query performed maintenance
  -> Query published delivery
```

No phase accepts a report, digest, identifier tuple, or fields copied from a
stronger predecessor. Each owner constructs its own next product and consumes
the exact proof-bearing product of the preceding phase.

### 4.2 Direct Truth And Derived Signal Consequence Are Different Facts

One authoritative Relational commit can produce two lawful inputs to Query:

1. `BridgeDeliveredTruthChange`: an admitted semantic truth change suitable
   for Query dependencies that directly consume that truth.
2. `BridgePerformedSignalInvalidation`: evidence that installed Signal work
   actually executed and produced or suppressed a derived consequence.

They must not share a variant that forces callers to infer which fact exists.
No zero-work or count-only Signal receipt is minted to make the shapes match.
Direct Query maintenance may proceed from admitted truth when its installed
dependency requires no Signal execution. A derived or conditional consequence
that declares Signal participation requires performed Signal evidence.

### 4.3 Signal Installed Scoped Seed Boundary

Signal must expose one integration-facade operation that consumes:

- an installed graph/node/aspect capability issued by the current Signal graph
- a canonical, aspect-correlated changed-region set
- one runtime-owned execution observation when performed evidence is requested

It returns a typed transition outcome containing the admitted source seed and,
after actual execution, `SignalInvalidationExecutionReceipt` plus the
Signal-owned semantic outcome needed by Bridge. It must not return raw
`DirtyBatch`, causes, ready work, or mutable graph internals.

Changed regions are canonical pairs. A detail is always interpreted under its
partition. Empty regions mean an explicit whole-aspect/unscoped change, not
unknown precision. Unknown or unavailable precision uses a typed declared
widening decision owned by Bridge before Signal mutation.

### 4.4 `ProducerAspectKey` Remains Private And Non-Authoritative

`ProducerAspectKey` remains Signal graph-runtime topology vocabulary:

```rust
pub(super) struct ProducerAspectKey {
    producer: NodeId,
    aspect: Aspect,
}
```

Only the Signal index owner constructs it from authoritative dependency edges
or committed output identity. It selects reverse-index candidates and proves
nothing about edge, snapshot, cause, readiness, or commit validity. It is
immediate-producer-local and never crosses Runtime Bridge or Query.

Scope membership remains separate. Exact detail lookup uses a key containing
both partition and detail; a bare detail identifier is forbidden because it
can collide across partitions.

### 4.5 Runtime Bridge Owns Semantic Lowering And Widening

Runtime Bridge lowers Foundational semantic locality and matched authoritative
change locality into Signal `ChangedRegion` values only after installed
correspondence and current source/target admission.

Every lowering outcome is one of:

- exact whole aspect
- exact unscoped record
- exact whole partition
- exact partition and detail
- declared coarse widening with its installed policy and reason
- suppressed as irrelevant
- denied, stale, deferred, failed, or rebind-required

Widening is never inferred from missing data and never selected by Query.
Stable names, equal numeric aspect slots, digests, target counts, and matching
strings cannot authorize correspondence.

The performed delivery binds at least:

- source runtime and installation generation
- source authority binding and committed patch identity
- authoritative snapshot/branch and aspect contract revision
- Bridge correspondence identity and revision
- Signal graph instance and installed target generation
- exact semantic change set and locality/widening posture
- performed Signal receipt when execution occurred

### 4.6 Query Compiles One Invalidation Manifest

Each installed query, operation, workflow, and live consumer contributes to one
Query-owned `WorthQueryInstalledInvalidationManifest`. It is compiled from the
same declarations that own query meaning and contains dependency roles for:

- projection
- selection or membership
- ordering
- grouping/aggregation
- window boundaries
- relation endpoints
- conditional nodes and workflow stages
- authorization/disclosure-sensitive visibility
- structural collection/capability/lifecycle changes

The exact native key includes the Foundational aspect key, identity, contract
revision, binding, relevant change kind, canonical field path or whole-aspect
posture, and semantic locality. Query-local collection mappings remain explicit
derived declarations, not reconstructed from strings.

The compiled impact index is rebuildable acceleration. An index hit selects a
candidate; it does not prove the current query, consumer, authorization,
disclosure, branch, basis, lease, or lifecycle permits maintenance.

### 4.7 Query Progression Is Compiler-Visible

The Query side has distinct private-constructor products:

```text
WorthQueryInstalledInvalidationManifest
  -> WorthQueryInvalidationCandidateSet
  -> WorthQueryAdmittedInvalidationImpact
  -> WorthQueryPerformedMaintenance
  -> WorthQueryPublishedLiveDelivery
```

`worth-proof` progression types may carry these owner-specific products, but a
generic proof wrapper cannot construct them. Candidate selection is read-only.
Admission performs current authorization, disclosure, purpose, tenant, branch,
basis, installation, lease, and lifecycle checks before maintenance effects.
Publication consumes performed Query maintenance, never Bridge or Signal
evidence directly.

### 4.8 Maintenance Strategy Follows Dependency Role

Query chooses one typed consequence from installed meaning:

- local projection patch
- membership insertion/removal
- stable reorder
- grouping/aggregate update
- window refill
- bounded re-execution
- rebind or replacement
- retirement or terminal release
- suppression
- typed unsupported escalation

The strategy is fixed before effects. A raw lower-runtime change cannot choose
the patch form. Ambiguity cannot silently widen to whole-capability work; it
must use an installed declared widening policy or return a typed unsupported or
rebind outcome.

### 4.9 Sharing Does Not Share Authority

Equivalent admitted query maintenance may be coalesced under existing Query
sharing and lease authority. Maintenance identity includes the installed query
meaning, current basis, branch, semantic impact, and result-state contract.

Authorization, purpose, disclosure, continuation, backpressure, and terminal
delivery remain per consumer. One consumer's wider disclosure cannot widen
another consumer. One consumer's revocation can terminate its delivery without
invalidating an otherwise lawful shared maintenance result.

### 4.10 Canonical And Derived State

Authoritative state is limited to the existing owners:

- Relational committed truth and history
- installed Bridge correspondence and source/target binding
- Signal graph topology, direct bases, canonical causes, and committed outputs
- Query installed meaning, admitted consumer authority, and committed
  query-shaped results

Reverse indexes, impact indexes, route caches, ready queues, diagnostics,
counter bundles, and replay views are derived. They can be destroyed and
rebuilt. Checkpoints do not serialize current work authority. Restore mints new
runtime identities and requires owner-specific revalidation/readmission.

Replay remains cert-only. Ordinary Query and Bridge lanes must not import
replay crates or pay replay reconstruction cost.

### 4.11 Public DX Is Query-Shaped

Ordinary users continue to declare and run a query through Query facades. They
do not provide aspect slots, Signal nodes, Bridge mapping IDs, partition
interners, dirty masks, or invalidation strategies.

Illustrative destination DX:

```rust
let live = operating_world
    .queries()
    .install(account_risk_query)
    .admit(request_authority)?
    .execute_live()?;

let delivery = live.next().await?;

match delivery {
    WorthQueryLiveDelivery::Patched(patch) => apply(patch),
    WorthQueryLiveDelivery::Reordered(order) => apply_order(order),
    WorthQueryLiveDelivery::Suppressed(reason) => observe(reason),
    WorthQueryLiveDelivery::RebindRequired(rebind) => rebind(rebind)?,
    WorthQueryLiveDelivery::Terminated(terminal) => release(terminal),
}
```

The runtime automatically uses exact installed lower-runtime correspondence
when present. Advanced inspection exposes Query-level semantic causes and
performed work disclosure, not lower-runtime authority objects.

## 5. Required Proof-Bearing Forms

### 5.1 Signal-Owned Forms

| Form | Constructed by | Proves | Cannot authorize |
|---|---|---|---|
| `InstalledSignalScopedChangeSet` | Signal integration admission from current installed capabilities and Bridge-lowered canonical regions | current graph/node/aspect targets and exact aspect-correlated regions were admitted as direct source changes | Bridge correspondence, Query impact, or publication |
| `SignalInvalidationExecutionReceipt` | Signal runtime after a bounded observation containing performed node evaluation | the attached realized Signal counters were observed after performed invalidation execution | Query permission or patch meaning |
| `SignalInvalidationExecutionSummary` | derived from performed receipt | read-only Signal work explanation | any execution or admission |

These names and the three authority distinctions are normative at the
cross-owner/facade boundary. Private subordinate mechanics remain an
implementation decision.

### 5.2 Runtime-Bridge-Owned Forms

| Form | Proves |
|---|---|
| `BridgeDeliveredTruthChange` | a current authoritative committed patch matched an installed semantic correspondence with exact change/locality posture |
| `BridgePreparedScopedSignalInvalidation` | the admitted truth change was lawfully lowered to installed Signal targets and canonical regions; no Signal effect yet |
| `BridgePerformedSignalInvalidation` | Signal performed the bound invalidation observation and Bridge retained the exact source/correspondence/target bindings |
| `BridgeGranularInvalidationDelivery` | a Query-readable union of direct truth and optional performed derived consequences, with typed suppression/widening and no raw Signal internals |

### 5.3 Query-Owned Forms

| Form | Proves |
|---|---|
| `WorthQueryInstalledInvalidationManifest` | installed Query meaning compiled the complete dependency-role and locality contract |
| `WorthQueryInvalidationCandidateSet` | the exact Bridge delivery selected these possible installed impacts through rebuildable indexes |
| `WorthQueryAdmittedInvalidationImpact` | current Query authority and lifecycle admit this narrowed maintenance work |
| `WorthQueryPerformedMaintenance` | Query performed the selected query-shaped maintenance against the bound current basis |
| `WorthQueryPublishedLiveDelivery` | disclosure-governed publication derived from performed Query maintenance or a typed lawful suppression/terminal outcome |

### 5.4 Proof And Foundational Boundaries

`worth-proof` may provide:

- prepared/admitted/performed progression
- owner-minted capability witnesses
- freshness/readmission wrappers
- transition outcomes

It must not contain Signal node/aspect vocabulary, Bridge correspondence
meaning, Query roles, or application permission.

`worth-foundational` may provide:

- exact aspect contracts, identities, revisions, masks, bindings, and canonical
  field paths
- portable partition/detail or successor `ScopePath` values
- canonical case/report bases and counter-backed work disclosure
- versioned boundary receipt vocabulary

It must not contain `ProducerAspectKey`, Signal readiness, Bridge installed
authority, Query publication permission, financial terms, or geometry-kernel
meaning.

### 5.5 Compiler Enforcement

Compile-pass and compile-fail evidence must prove public product impossibilities,
not private syntax preferences:

- a lawful Query host flow compiles without importing Signal or Runtime Bridge
  internals
- a predicted Signal estimate cannot substitute for performed Signal evidence
- raw `BridgeDeliveredTruthChange` cannot construct a performed derived
  consequence
- a descriptive Query-local bridge summary cannot construct a Bridge delivery
- a Signal receipt cannot construct Query admitted impact or publication
- foreign/stale graph, installation, contract revision, branch, snapshot,
  commit, lease, or lifecycle bindings cannot progress
- direct truth cannot masquerade as Signal-performed consequence
- a Foundational digest/report cannot reopen any owner authority
- removed coarse/count-only production entry points have no compatibility alias

## 6. Architectural Destination

The tree below is normative. `[existing]`, `[extended]`, `[created]`,
`[replaced]`, and `[removed]` describe Milestone 13.1 actions. Committed M14
destinations are shown only where this milestone must leave an additive seam;
empty placeholders are not created.

```text
crates/worth-signal/src/
  data/aspect/
    lowering_capability.rs                         [extended; admission only]
    installed_change/
      change_set.rs                                [created; canonical scoped change]
      denial.rs                                    [created; graph/aspect/region denial]
  data/proof/invalidation/
    performed_receipt.rs                           [extended; carried unchanged]
  facade/
    adapters.rs                                    [extended; Bridge-facing types]
    integration.rs                                 [extended; one scoped installed entry]
  data/graph/topology/subscriber_index/
    buckets.rs                                     [existing; Signal-private]
                                                  [M14 adds hierarchy beneath this owner]

crates/worth-runtime-bridge/src/
  correspondence/
    semantic_delivery_match.rs                    [existing]
    delivery.rs                                   [decomposed orchestration]
    locality_lowering/
      canonical_regions.rs                        [created; Foundational -> Signal]
      widening.rs                                 [created; installed widening decision]
    signal_execution/
      preparation.rs                              [created; no effect]
      execution.rs                                [created; bounded Signal observation]
      performed_delivery.rs                       [created; bound carried receipt]
    query_delivery/
      truth_change.rs                             [created; direct semantic fact]
      granular_invalidation.rs                    [created; Query-readable union]
      outcome.rs                                  [created; typed boundary outcome]
  facade/
    exports_core.rs                               [extended; narrow delivery exports]

workspaces/worth-query/crates/worth-query/src/
  domain_installation/
    dependency_impact/
      compilation/                                [existing; extended manifest input]
      compiled/
        impact_index.rs                           [existing; extended exact locality key]
        invalidation_manifest.rs                  [created; installed authority]
      impact/                                     [existing; candidate classification]
    consumer_invalidation/
      meaning.rs                                  [extended; exact locality outcomes]
      admission.rs                                [created; current authority checks]
  live/
    region_scoped/
      execution.rs                                [promoted from test-only production]
      matching.rs                                 [created; exact locality classification]
      widening.rs                                 [created; Query consumes Bridge posture]
    relevance/
      bridge_change.rs                            [retained private descriptive fixture;
                                                   removed as authority input]
    maintenance/
      admission.rs                                [created; admitted impact]
      execution.rs                                [created; role-selected maintenance]
      publication.rs                              [created; query-shaped delivery]

workspaces/worth-query/crates/worth-query-execution/src/
  domain_computation/primary_graph/
    managed_bridge.rs                             [replaced coarse/count-only sink]
    granular_invalidation/
      installation.rs                             [created; composition root wiring]
      delivery.rs                                 [created; Bridge -> Query admission]
      observation.rs                              [created; performed cross-runtime rows]
    tests/granular_invalidation.rs                [created; focused integration proof]

workspaces/worth-query/crates/worth-query-host/src/
  facade.rs                                       [extended; Query-owned live outcomes only]

crates/worth-runtime-bridge/src/facade/tests/
  granular_invalidation/
    locality_lowering.rs                          [created]
    binding_denials.rs                            [created]
    performed_delivery.rs                         [created]

workspaces/worth-query/crates/worth-query/src/domain_installation/
  dependency_impact/compiled/impact_index/tests/
    granular_invalidation.rs                      [created]

workspaces/worth-query/crates/worth-query-certification/tests/
  granular_invalidation.rs                        [created; one cold harness]
  granular_invalidation/
    world.rs                                      [created; real composition root]
    necessity_manifest.rs                         [created; independent oracle]
    scenario_execution.rs                         [created]
    lifecycle.rs                                  [created]
    structural_slopes.rs                          [created]
    sealed_run.rs                                 [created]

crates/worth-signal-wasm/tests/
  installed_scoped_invalidation.rs                [created; facade parity]
```

Structural rules:

- Signal scoped admission, Bridge lowering, Query impact, and Query publication
  never share a file or directory owner.
- Cross-runtime orchestration lives at the Query execution composition root,
  above the participating runtimes, not inside Signal or a domain package.
- Facades aggregate types and operations only; they contain no lowering,
  matching, policy, or maintenance behavior.
- `bridge_change.rs` cannot remain a second public authority vocabulary. If
  retained for local fixtures or presentation, it is private and explicitly
  descriptive.
- no `helpers`, `common`, `shared`, `manager`, or milestone-number production
  directories are permitted.
- M14 hierarchical locality enters under the existing Signal subscriber-index
  and Bridge locality-lowering axes without moving the 13.1 facade or Query
  manifest.

## 7. Ordered Implementation Phases

### Phase 1 - Boundary Freeze And End-To-End Red Courtroom

What becomes true:

- the exact coarse/count-only production path, test-only region lane, local
  descriptive bridge vocabulary, and missing performed crossings are frozen
- the real financial and opaque-region worlds compile through Relational,
  Bridge, Signal, Query installation, and Query host composition
- the independent `R/B/S/I/M/D/X` manifest exists before production routing
- red controls demonstrate locality loss and coarse Query maintenance on the
  inherited path

Mechanical evidence:

- current public/export/composition-root inventory
- one real 5y/10y curve-detail matched/unmatched red control
- one direct-truth versus derived-Signal distinction control
- one slope showing unrelated installed queries currently affect work or that
  the current path cannot report the claimed bound

The next phase may trust the courtroom and frozen owner boundaries, not a new
runtime capability.

### Phase 2 - Signal Installed Scoped Invalidation Contract

What becomes true:

- Bridge can present current installed node/aspect capabilities together with
  canonical aspect-correlated regions
- Signal admits one direct source change without exposing internal dirty,
  cause, queue, or index forms
- actual Signal execution can return the existing performed receipt through
  the integration facade

Mechanically forbidden:

- raw node/aspect tuples as cross-runtime authority
- partition/detail flattening
- zero-work performed receipts
- predicted/performed substitution

Evidence:

- whole-aspect, unscoped, whole-partition, exact-detail, unknown-partition with
  unscoped consumer, aspect-scope swap denial, duplicate/foreign capability,
  and default/parallel/WASM twins
- existing M13 locality and performed-counter suites remain green

The next phase may trust one honest Signal integration boundary.

### Phase 3 - Runtime Bridge Exact Lowering And Performed Delivery

What becomes true:

- exact matched semantic locality lowers into canonical Signal changed regions
- every widening is declared and bound before mutation
- direct truth and optional Signal-performed consequence are separate carried
  products
- Bridge returns a Query-readable granular delivery with current source,
  correspondence, target, and performed bindings

Mechanically forbidden:

- count-only delivery
- string/digest/numeric-slot correspondence
- Signal execution without bounded observation
- direct truth laundered as performed consequence

Evidence:

- every locality class and widening decision has positive/negative twins
- one-axis drift denial for source runtime, installation generation, authority
  binding, commit, snapshot/branch, contract revision, correspondence revision,
  Signal graph instance, and target generation
- failure at every pre-Signal and post-Signal/pre-delivery seam exposes exact
  partial posture and no false performed delivery

The next phase may trust an exact cross-runtime delivery, not Query authority.

### Phase 4 - Query Installed Invalidation Manifest And Admission

What becomes true:

- installed Query meaning compiles the complete dependency-role/locality
  manifest
- exact Bridge deliveries select candidates through current rebuildable indexes
- current Query authority narrows candidates into admitted impact before effects

Mechanically forbidden:

- Query-local descriptive summaries as authority
- host-authored invalidation strategy
- ambiguity silently widened to whole capability
- candidate index hit treated as current admission

Evidence:

- exact role classification for projection, membership, ordering, grouping,
  window, relation, conditional/workflow, authorization/disclosure, and
  structural changes
- one-axis stale/foreign denial for every Query currentness binding
- index destruction/rebuild parity and index-drift fail-closed behavior
- compile-fail phase-skip and foreign-owner substitutions

The next phase may trust exact admitted Query work.

### Phase 5 - Production Maintenance, Sharing, And Facade Cutover

What becomes true:

- region-scoped live execution is production code
- the primary application bridge consumes granular Bridge delivery and returns
  Query-owned performed maintenance/publication outcomes
- maintenance strategy follows installed dependency role
- equivalent maintenance coalesces while consumer authority remains separate
- the old registered-entity coarse/count-only ordinary path is removed

Mechanically forbidden:

- raw CDC/partition/Signal events in public delivery
- parallel coarse compatibility lane
- per-consumer duplicate maintenance where installed equivalence admits sharing
- publication directly from lower-runtime evidence

Evidence:

- `curve_detail_to_live_risk`, `suppressed_quote_no_query_patch`,
  `ordered_portfolio_membership`, and
  `shared_lease_disclosure_noninterference` pass through the real composition
  root and host observation
- public Query examples compile without lower-runtime imports
- old coarse sink/constructors fail or are unreachable from production

The next phase may trust the only ordinary platform path.

### Phase 6 - Rebind, Restore, Branch, And Lifecycle Closure

What becomes true:

- destroyed indexes rebuild from authority
- restore mints new runtime identities and requires Bridge/Signal/Query
  readmission
- delayed, duplicate, reordered, stale, and mixed direct/derived deliveries
  have typed idempotent outcomes
- revocation after selection but before publication narrows or terminates safely

Mechanically forbidden:

- serialized ready/candidate authority
- stale receipt reuse
- same-count cause, impact, or delivery substitution
- replay/reconstruction cost in ordinary receipts

Evidence:

- `correspondence_rebind_restore` and disclosure-revocation courtroom
- checkpoint after each ambiguous effect boundary
- duplicate/reordered delivery convergence
- cert-only replay equivalence observed separately from ordinary execution

The next phase may trust current lifecycle behavior across every owner.

### Phase 7 - Structural Slopes, Certification, Documentation, And Handoff

What becomes true:

- all scenarios and scale axes produce exact owner-separated performed counters
- independent `R/B/S/I/M/D/X` identities match production observations
- a sealed cross-runtime certification run rejects missing, duplicate, stale,
  wrong-scenario, wrong-seed, wrong-policy, wrong-tier, mixed-runtime, and
  mismatched-oracle evidence
- all false documentation and compatibility residue is removed
- M14 receives a platform-used canonical Signal work stream without acquiring
  cross-runtime authority

This phase closes only after default, parallel-feature, WASM, constitutional,
documentation, and frozen-source review gates pass.

## 8. Complexity And Resource Contracts

The milestone records performed structural work separately at each owner.
Cross-runtime totals cannot replace owner-local counters.

### 8.1 Runtime Bridge

For `r` authoritative changed semantic surfaces, `q` exact correspondence
bucket probes, `c` returned candidates, and `m` admitted matches:

```text
delivery work = O(r + q + c + m)
```

Adding installed correspondences outside queried keys produces zero delta in
candidate, Signal-seed, and Query-delivery rows. Widening adds only its declared
candidate granule and a widening counter. A full correspondence-registry scan
is forbidden on the ordinary path.

Required Bridge rows include:

- semantic changes examined
- correspondence bucket probes and candidates returned
- aspect/mask/binding/locality/change-kind rejections
- exact and widened matches
- Signal capabilities admitted
- scoped seeds submitted
- performed Signal deliveries emitted
- stale, rebind, duplicate, deferred, and failed outcomes

### 8.2 Signal

Signal retains every M13 counter and slope contract. Bridge carriage may not
relabel Bridge target count as Signal work. M13 performed counters remain the
only Signal execution truth.

### 8.3 Query

For `b` delivered semantic changes, `p` impact-index probes, `i` returned
installed impacts, `a` admitted impacts, `m` distinct maintenance operations,
and `d` authorized consumer deliveries:

```text
selection and maintenance = O(b + p + i + a + m + d + changed_result_granule)
```

The changed result granule is strategy-specific: touched projected fields,
membership candidates, reordered keys, affected aggregate groups, window
refill width, or bounded re-execution footprint. It is never silently all rows.

Required Query rows include:

- delivery changes and locality entries examined
- exact impact-index probes and candidates returned
- candidates rejected before admission
- admitted impacts by role
- coalesced versus distinct maintenance operations
- projected fields, membership rows, ordering keys, aggregate groups, window
  rows, and bounded re-execution work
- authorization/disclosure revalidations and denials
- query patches, suppressions, rebinds, replacements, retirements, and
  terminal deliveries
- per-consumer publications and backpressure outcomes

### 8.4 Required Slopes

The certification must independently prove:

1. Increasing unrelated Bridge mappings with `R/B/S/I/M/D` fixed changes no
   downstream work row.
2. Increasing returned-but-rejected Bridge candidates changes only Bridge
   candidate/examination/rejection rows.
3. Increasing unrelated Signal subscribers changes no Signal or Query work.
4. Increasing unrelated installed queries changes no admitted impact,
   maintenance, or publication row.
5. Increasing shared consumers increases authorized deliveries but not
   maintenance operations when equivalence admits sharing.
6. Increasing result rows outside the affected region changes no maintenance
   row.
7. Increasing the actual semantic frontier expands every owner row by the
   independent manifest delta; dense necessary work is never dropped to keep a
   cheap slope.

Wall-clock evidence is supplemental. It must state environment, runtime,
feature set, cold/warm posture, repetitions, percentiles, and variance.

## 9. Documentation Deliverables

Milestone 13.1 must revise these durable audience documents:

- `workspaces/worth-query/crates/worth-query/docs/AI_README.md` for runtime
  maintainers: the exact direct-truth and performed-Signal handoff, Query
  currentness admission, and public query-shaped outcome
- Runtime Bridge aspect documentation for bridge maintainers: exact locality
  lowering, declared widening, performed delivery, rebind, and cost rows
- Query live-view and region-scoped invalidation documentation for application
  authors: automatic narrowing, unsupported escalation, suppression, rebind,
  and disclosure behavior
- WORTH Signal vision and invalidation execution observation documentation for
  runtime authors: installed scoped integration and the limit of Signal
  performed authority
- Query and Runtime Bridge roadmaps: cross-reference this platform cutover so
  their local milestones cannot claim the composed path independently

Examples must compile against the real public facade. Remove or correct any
document that says coarse target count, Query-local bridge summary, or raw
lower-runtime event is sufficient production authority.

## 10. Must Ship And Must Preserve

### Must ship

- one installed scoped Signal invalidation entry through the integration facade
- exact Runtime Bridge locality lowering with typed widening
- direct-truth versus performed-Signal Bridge delivery distinction
- carried Signal performed receipt bound to current Bridge delivery
- one complete Query invalidation manifest and exact impact index
- Query currentness admission before maintenance/publication
- production region-scoped Query maintenance
- query-shaped patch/suppress/reorder/reexecute/rebind/retire/terminal outcomes
- removal of the coarse/count-only ordinary composition path
- cross-runtime performed counters and independent certification
- default, parallel-feature, WASM, restore/rebind, and disclosure evidence

### Must preserve

- every M12 semantic-causality and M13 locality/progression guarantee
- immediate-producer-local aspect meaning
- private non-authoritative `ProducerAspectKey`
- exact partition-plus-detail identity and unscoped matching
- Runtime Bridge ownership of correspondence and widening
- Query ownership of application authority and publication
- Foundational/Proof authority limits
- Query-shaped public delivery
- cert-only replay and ordinary-lane cost separation
- M14's ability to parallelize the same admitted Signal work without changing
  semantic admission

## 11. Explicit Exclusions

Milestone 13.1 does not:

- move Query meaning or authorization into Signal or Runtime Bridge
- move Signal node/aspect/cause/readiness vocabulary into Foundational
- make `ProducerAspectKey`, locality paths, digests, counters, or indexes
  authoritative
- define financial or geometry semantics in Signal
- implement M14 resource leasing, hierarchical scope paths, sharding, or
  deterministic parallel publication
- expose raw CDC, partition events, or Signal receipts as the public Query
  delivery contract
- make durable stream resume or ordinary replay a Query responsibility
- preserve the registered-entity coarse/count-only path as compatibility
- require a general Rust compiler, macro expander, or public-API analyzer in
  the test suite; mechanical enforcement targets the current named production
  boundaries and plausible compile-valid bypasses required by this spec

## 12. Acceptance Evidence

Milestone 13.1 closes only when:

- all six scenario families pass through the real cross-runtime composition
  root
- exact identities in `R/B/S/I/M/D/X` match for ordinary cases and sealed
  digests/cardinalities match for scheduled cases
- every direct-truth versus performed-Signal positive/negative twin passes
- every owner currentness binding has a one-axis drift denial
- every important effect seam has failure/partial-state evidence
- exact, widened, suppressed, stale, rebind, deferred, denied, failed, and
  terminal outcomes are typed and observed consequentially
- off-region, unrelated-aspect, unrelated-partition, unrelated-detail,
  unrelated-query, and unauthorized-consumer work is exactly zero
- direct unscoped consumers still receive unknown/new partition changes without
  widening scoped siblings
- comparator suppression emits no false derived Query patch
- shared maintenance and per-consumer disclosure rules both hold
- destroyed derived indexes reconstruct from authority with exact parity
- stale pre-restore and pre-rebind products cannot progress
- ordinary receipts exclude replay, reconstruction, diagnostics, and forensic
  work
- the seven structural slopes pass with exact counter deltas
- compile-pass/fail evidence proves only the public impossibilities named in
  section 5.5
- the coarse/count-only production path and public descriptive authority lane
  are deleted or mechanically unreachable
- M12, M13, Query 9.14, Runtime Bridge aspect-native, live Query, default,
  parallel-feature, doctest, and WASM suites pass
- formatting, `git diff --check`, dirty Rust line-cap, boundary-check, and
  agent-context checks pass
- documentation examples compile against current facades
- a fresh final critic reviews the frozen final-source fingerprint and the
  proportional blocker classes are current runtime correctness, demonstrably
  false milestone claims, or required compile-valid wrong-reason mutations

## 13. Successor Handoff

[Milestone 14 - Deterministic Parallel Execution Foundation](./milestone-14-plan.md)
inherits:

- the unchanged M13 canonical ready-work stream and performed receipt
- a real Bridge-to-Signal installed scoped entry using that stream
- a real Bridge performed delivery carrying Signal execution evidence
- a real Query consumer that admits and maintains exact impacts
- owner-separated realized counter families and cross-runtime slope evidence
- the exact unscoped/partition/detail base case used by production platform
  composition

Milestone 14 may generalize the physical Signal subscriber index to a bounded
opaque hierarchy, derive non-authoritative execution placement, and execute
independent ready work under resource leases. It may not:

- change the Bridge or Query authority chain
- make a scope path or shard key authoritative
- copy aspects or scopes transitively
- weaken direct-truth/performed-consequence separation
- convert Signal performed evidence into Query permission
- reintroduce coarse Query invalidation to simplify parallel scheduling
- charge cross-runtime reconstruction or diagnostics to the ordinary execution
  receipt

Milestone 15 and later graph-parallel work therefore optimize a platform-used,
end-to-end granular invalidation contract rather than a Signal-local mechanism.
