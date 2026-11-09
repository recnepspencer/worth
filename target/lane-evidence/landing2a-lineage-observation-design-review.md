# Landing 2a lineage observation design review

Read-only independent source review, 2026-11-09. No source changes, Cargo,
builds, or tests. This document is a design finding, not compiled evidence.

## Finding

The class-3 lineage claims have feasible small production-owner observation
surfaces. I found no governing specification prohibition on adding them.
Do not defer these public semantic/provider claims merely because the existing
owner methods are private, and do not export `LineageSubsystem`, residency,
root access, or the whole `LineageAccess` bag.

The earlier unresolved-boundary note correctly distinguishes authoring-branch
discovery from selected-root ancestry, but its corpus concern has a smaller
solution than introducing a new public corpus authority: use the existing
canonical selection and validation operations as an optimistic read cut.
Provider I/O stays outside the selection gate. A changed cut refuses the read
instead of publishing an incoherent result.

The reviewed claim map contains six class-3 entries: indices 10, 32, 33, 34,
37, and 43. Index 10 is source posture rather than lineage.
The lineage design below addresses indices 32, 33, 34, 37, and 43. Source-origin
posture at index 10 belongs to durability and needs its own owner observation.

## Source boundary

Reviewed `landing2a-lineage-observation-boundary.md`, the per-test entries and
assertions in `landing2a-cold-claim-review.{md,json}`, lineage access and data,
selected-observation public reads, actual node/event residency and admission,
branch identity/basis admission, canonical checkpoint selection/validation,
Complete capture's lock boundary, the relevant cold fixture/courts, and existing
Execution allocation owners. Governing references are the repository AGENTS
instructions, architectural/performance/testing/composition/domain laws and
relevant iteration-cutover passages. The code-quality-qa skill informed the
ownership review; its implementation verification steps were outside this
explicit read-only assignment.

## Keep the two meanings separate

`lineage/access/resolution.rs:98` resolves an explicit lineage seed using only
events authored on the requested branch. It demands the seed's reachable event
pages with `reachable: None` and uses `HistoricalTraversalScope::AuthoringBranch`
at lines 107-110 and 261-266. This is legitimately a semantic discovery query:
the result is the branch's authored replacement/split/merge chain, rather than
the history visible from one selected commit.

`change_source/lineage_at_observation.rs:19` has a different contract. It admits
an exact owner observation, demands Full region at lines 28-31, and invokes
`resolve_record_history_for_observation`. The latter uses
`SelectedRootReachableCommitScope` and filters event publication commits through
selected ancestry (`lineage/access/selected_root_resolution.rs:37-89` and
`lineage/access/resolution.rs:267-311`). Its public docs promise that later
commits never change the answer.

Neither operation subsumes the other. Replacing discovery with the current
observation API changes both ancestry meaning and the event-only provider
footprint. Conversely, using an observation argument while silently ignoring
its selected ancestry would misdescribe an exact-observation read. Give the
discovery surface an explicit discovery name and a branch identity input.

## Concrete owner operations

Suggested signatures on the existing runtime composition, with implementation
in the actual lineage owner and exports through the normal facade:

```rust
pub fn lineage_node_at_observation(
    &self,
    request: &ExecutionRequest<'_, '_>,
    observation: &RelationalBranchObservation,
    entity: EntityId,
) -> Result<Option<RelationalLineageNodeRead>, RelationalObservationReadDenial>;

pub fn discover_authored_lineage_history(
    &self,
    request: &ExecutionRequest<'_, '_>,
    branch: &RelationalBranchIdentity,
    seed: LineageId,
) -> Result<HistoricalLineageResolution, RelationalLineageDiscoveryDenial>;

pub fn discover_authored_record_history(
    &self,
    request: &ExecutionRequest<'_, '_>,
    branch: &RelationalBranchIdentity,
    entity: EntityId,
) -> Result<Option<HistoricalLineageResolution>, RelationalLineageDiscoveryDenial>;
```

The final names are a DX choice. The important contracts are:

- `RelationalLineageNodeRead` is an owned immutable semantic node read. Privately
  retain the actual `ResidentLineageNode`; expose `node() -> &LineageNode` only.
  Do not expose a page cell, counter, node arena, mutation door, or constructor.
- Return the existing facade `HistoricalLineageResolution` DATA directly. It
  already carries the exact branch, seed, resolved IDs, traversed event IDs,
  canonical digest basis, and metrics/trace meaning. A second event Read wrapper
  is unnecessary: this result carries no page guard or authority after the
  traversal, and no new result allocation or retention mechanism is required.
- Discovery denial distinguishes branch admission, execution/storage refusal,
  and a changed/unsettled canonical cut. Preserve original storage subject and
  cause. Do not render these as strings, `None`, or a guessed lineage failure.
- The branch identity is not permission by itself. Internally use existing
  `observe_branch` admission and retain the admitted basis for the operation.
  The read contract explicitly says its branch is the authored-event scope;
  the admitted basis supplies owner/lifecycle binding, not an ancestry filter.
  Foreign, unknown, archived, and deleting branch denials come from that owner.

These operations add no new legal-grant witness, caller-list exception, sealed
constructor export, or consumer-forgeable authority. The node read guard is a
runtime lifecycle object, so it belongs with lineage/read ownership rather than
Proof or Foundational. Preserve the existing history DATA's digest and field
meaning rather than designing a second lineage algorithm or result vocabulary.

## Node operation and exact custody

Admit `observation` using the existing runtime-affinity check, consult the
original request, and use that held root's `demand_region_head` to select the
entity slot's lineage association. `branch/root_region_access.rs:189` is the
existing Head-only owner demand; it does not require Full or symbols. Then use
the actual `lineage.demand_node`, validating entity binding and retaining its
result. Reuse the existing missing-required-node Damaged refusal and
`ResidentLineageNode::reject_binding` behavior, rather than constructing a
plausible `LineageNode` from IDs.

The original point operation is `lineage/access/records.rs:14-39`; its selected
association is read from partition Head metadata, followed by node demand and
binding validation. The selected-root implementation must preserve full entity
identity/generation checks. It must also allow a retained retired slot's lineage
association: the custody fixture selects `fixture.original`, which was replaced.
Filtering through a visible-only record read would lose this claim.

`ResidentLineageNode` already owns the real node-page lease
(`runtime/state/subsystems/lineage/node_residency.rs:210-219`). Keeping that
private object in the public semantic read preserves the original interval in
which `node.node()` and the provider custody assertion run before `drop(node)`.
Do not merely return a newly reconstructed node. The node itself contains only
fixed-size lineage/entity IDs (`lineage/data/graph.rs:18-31`); the actual decoded
page keeps its existing owned allocation. There is no need to materialize a
graph or symbol payload.

The custody court drops the node before dropping the fixture. It proves that
the old runtime/residency owner keeps the original member after home replacement
and releases placement after its final owner drops. It does not prove that a
node handle alone keeps external source placement after runtime destruction;
do not silently strengthen the new API's lifetime promise to that different
claim. The new read must retain the actual page for its own node access.

The concurrent point court counts one node-member read. Its private Head warmup
is setup, not the asserted claim; the public node operation can select Head
itself. Preserve exact node equality and the single page load. If the external
fixture needs a separate warmup, use an actual public owner read whose declared
footprint is appropriate, not a private root export.

## A coherent discovery cut without locking provider I/O

Existing concrete operations suffice:

1. Consult the original request and obtain
   `history.canonical_checkpoint_gate().checkpoint_selection()`.
2. Admit and hold the requested branch basis through existing owner admission.
3. Demand the same reachable event pages and run the same authoring-branch
   traversal. Keep actual event-page custody live through traversal. For the
   record variant, select its lineage association through the admitted root's
   Head and validate the actual node binding before invoking that traversal.
4. Before returning either a semantic result or an ordinary storage refusal,
   call `validate_checkpoint_selection(&selection)`. If it fails, return a typed
   changed/unsettled-cut denial. There is no automatic unbounded retry.

Selection/validation are in
`runtime/state/subsystems/canonical_publication_routes/checkpoint_admission.rs:45-83`.
Each acquires the existing gate only for its own short call. Both reject
unsettled performed publication; validation compares the performed-stream head
and branch-transition frontier. The stream comparison uses immutable head Arc
identity (`performed_stream.rs:66-80`), not a potentially reused numeric event
counter. Retained selection prevents head identity reuse. Recovery replacement
still obeys the runtime's exclusive recovery ownership; do not introduce a
concurrent catalog-reset lane.

With an unchanged validated cut, publication cannot have changed the logical
event corpus during the accepted read. Concurrent page admissions only reveal
already-authoritative immutable rows. The existing route selection demands all
pages for the frontier's branch/node ranges, and its index installation binds
each row to its publication commit. Thus a second reader warming unrelated
pages does not create new semantic events.

This avoids a global lock across I/O and avoids cloning every lineage event or
commit. Complete capture already establishes the relevant principle: provider
protection, journal reads, and reconstruction happen after leaving logical
epochs (`durability/authority/checkpoint_complete/capture.rs:91-99`). Do not put
the entire discovery traversal inside `with_checkpoint_selection`.

The source-boundary note is right that `ResidentLineageEventPage` is not itself
a corpus cut. It holds immutable rows and a page-cell lease
(`event_residency.rs:267-291`). `demand_historical_events` currently keeps those
leases but then reads the live resolution index repeatedly. The optimistic
selection validation supplies the missing coherence fact without changing
authoring-branch meaning. Ordinary concurrent publication can cause a typed
retryable discovery denial; later calls may legitimately discover more events.
Discovery does not promise repeatability after it returns.

## Ordinary result allocation is not a new quota prerequisite

Do not promote the dormant full graph/divergence collectors. They materialize
whole graphs/sets and have different breadth, retention, and resource contracts.
Those exact private graph/divergence subcases remain C2 with restore references.

Do not claim the current resolver is already fully request-funded.
`lineage/access/resolution.rs` uses ordinary Vec/BTreeSet collections, and
`event_demand.rs` uses ordinary frontier/visited/custody collections. Its result
duplicates canonical lists into resolution, trace, and digest. Page readmission
funding alone does not fund these containers.

That observation does not establish a HAPPY migration blocker. The same
ordinary resolver/result already ships through
`record_history_at_observation` (`change_source/lineage_at_observation.rs:19-35`),
and `HistoricalLineageResolution` is already facade DATA (`facade.rs:140-154`,
`lineage/data/resolution.rs:50-72`). The minimal new methods can call that actual
resolver, keep its existing Vec/BTree policy, retain actual event-page leases
through traversal, and return its existing result after cut validation. They
must preserve existing request checks and typed failure propagation. They must
not claim an allocation ceiling, complete retained-memory accounting, or a
fully request-funded result that the implementation does not establish.

I found no explicit governing scoped admission/resource contract requiring all
of those existing collections to be refactored merely to expose this semantic
owner operation. Architectural Law 22 (`arch_laws.md:45`) requires declared
request/control/resource behavior for unbounded or externally blocked work; it
does not prescribe a new public quota parameter or per-container reservation
representation. The performance law at `perf_laws.md:33` says allocation needs a
named lifecycle and bounded hot-path budget, and the API cost-shape law at line
13 requires explicit traversal cost. These general laws require honest contracts
and cost claims; they do not by themselves prove this migration must introduce
a different result ownership contract or convert all ordinary resolver storage.
No measured resource failure or new resource claim was supplied for that change.

The applicable code-quality-qa skill is explicit at lines 35-52: resource
bookkeeping needs an explicit product requirement or measured resource failure;
"Do not infer that these correctness guarantees require resource pre-admission."
It also says not to promote accounting to a prerequisite merely because existing
code/tests use it. An owned event Read wrapper and wholesale ordered-map/array
conversion would therefore require their own demonstrated need and scoped
resource work; they are not conditions of this minimal observation design.

Existing Execution ordered-map/request-array owners are available if separately
specified C2 work later needs them. My initial funding section recommended that
broader improvement too strongly and incorrectly made it sound like a required
part of this HAPPY seam. This revision withdraws that prerequisite. The node
read guard remains necessary for a different reason: its actual counted page
lease is part of the asserted observation lifecycle, not a quota mechanism.

## Claim preservation and ruling

- Index 37: use authored-lineage discovery. Compare the original exact digest;
  retain eight concurrent demands, one actual event-member load, and the
  provider assertion that every read is an event member. Do not use node or
  record discovery here.
- Index 33: preserve shared sticky query/export damage, no repaired-provider
  reacquisition, and healthy unrelated seed's exact resolution/no traversed
  events using the same authored-lineage operation.
- Index 34: Historical uses authored-lineage discovery; Record uses the
  record-selected variant. Preserve exact resolution and traversed IDs, typed
  failure matrix, retry and no-effects assertions. Existing public Observation,
  Aspect, AspectTrace and export operations remain their own distinct doors.
  AccessAspect aliases use the already-public history operations. Split only
  dormant Graph/Divergence evidence into C2, retaining explicit restore pointers.
- Index 43: use node-at-observation, compare the actual `LineageNode` and count
  the original node member once across simultaneous demands.
- Index 32: use node-at-observation for the original retained slot, keep the
  real node lease alive during provider custody assertion, and preserve final
  owner release. Identify the real member from authentic emitted artifacts or
  public inspection; do not export the checkpoint manifest.

Architectural Laws 1, 5, 10, 15, 16, and 19 support owner-issued immutable reads,
typed denials, explicit traversal, and actual lifecycle carriage. Performance
laws require named traversal/allocation costs; they do not require a second
authority lane. `iteration-cutover.md:178-182` forbids ID-only replacement for a
private constructor and publication of private runtime state; the operations
above perform real owner admission and expose semantic results. Its destination
table at line 103 explicitly assigns canonical lineage custody and queries to
the history owner. These are design constraints to implement, not a prohibition
on an ordinary lineage query.

No file-ownership restriction establishes a semantic blocker. The implementation
still needs final source review and the authorized verification lane; this
read-only design review claims neither compilation nor completed migration.
