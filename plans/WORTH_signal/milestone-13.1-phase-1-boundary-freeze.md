# Milestone 13.1 Phase 1 Boundary Freeze

This record freezes the inherited cross-runtime invalidation boundary before
the Milestone 13.1 cutover. It is evidence for Phase 1 of
[`milestone-13.1-plan.md`](./milestone-13.1-plan.md), not the destination API.

## Authoritative Owners

- Relational owns committed aspect truth, record identity, commit, snapshot,
  branch, contract, binding, field path, and source partition provenance.
- Runtime Bridge owns installed semantic correspondence and the decision that
  an authoritative Relational change matches it.
- Signal owns graph-local node/aspect capability, canonical changed regions,
  direct invalidation, causes, readiness, execution, and performed counters.
- Query owns installed dependency roles, impact admission, maintenance,
  consumer authorization/disclosure, and query-shaped publication.

The independent courtroom manifest may read only the immutable world
definition and named mutation. It may not read Bridge matching, Signal routing,
Query indexes, queues, receipts, or observed counters.

## Inherited Ordinary Paths

| Boundary | Current production entry | Frozen limitation |
|---|---|---|
| Bridge -> Signal correspondence | `RuntimeBridge::deliver_installed_correspondence` | calls `apply_installed_aspect_changes` with node/aspect capabilities and no changed regions; every target of the correspondence is seeded |
| Signal installed change | `apply_installed_aspect_changes` | returns a count and cannot return performed execution evidence |
| Bridge correspondence receipt | `BridgeCorrespondenceDeliveryReceipt` | carries admitted truth and seed counters but no typed direct-truth/performed-consequence split |
| Bridge ordinary sink | `InvalidationSink::deliver_invalidation` | accepts `BridgeSignalInvalidationDelivery` and returns target count plus snapshot |
| Query application sink | `WorthQueryApplicationInvalidationSink` in `worth-query-execution` | implements the count-only receipt and performs no Query impact admission or maintenance |
| Query region execution | `live/region_scoped/execution.rs` | test-only and therefore cannot support the production claim |
| Query descriptive bridge vocabulary | `live/relevance/bridge_change.rs` | descriptive local shape, not Bridge authority; it cannot survive as a public authority lane |

## Phase 1 Red Controls

`phase_one_manifest_retains_the_exact_5y_need_before_runtime_routing` derives
the complete small-case `R/B/S/I/M/D/X` expectation from a 5y/10y immutable
world definition before production routing.

`phase_one_red_control_separates_direct_truth_from_later_signal_execution`
uses the installed Query operating world, a real Relational commit source,
Bridge correspondence, Signal graph, and Query impact admission. It proves that
the inherited Bridge receipt exists and can drive direct Query truth impact
before the post-change Signal compute provider is contacted. This is the red
boundary that Phase 3 must replace with separate typed products.

`inherited_detail_loss_red_control_scales_seed_work_with_sibling_targets`
holds one admitted authoritative change constant while growing sibling Signal
targets from 1 to 4 to 16. The inherited path emits exactly that many Signal
seeds because it cannot carry the 5y detail needed to exclude a 10y sibling.

## Next Trusted Boundary

Phase 2 may trust the world definition, named mutation, independent manifest,
current owner map, and red observations above. It may not trust the inherited
count-only receipt, target fanout, or absence of detail as destination truth.
