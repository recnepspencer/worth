# Bound Projection Lifecycle, Sharing, And Consumer Invalidation

## What This Feature Is

This feature carries an installed projection from declaration-indexed native
access through compatibility, managed lifecycle, shared execution, and exact
consumer invalidation. Query keeps the live owner, issues one move-only lease
per consumer, classifies each authoritative change, and emits a lease-bound
invalidation delta that says what changed and what kind of response is safe.

## Why You Use It

Use it when two views can share the same live computation, when a consumer
needs field-precise invalidation, or when a conditional operation must preserve
the Signal decision that caused—or suppressed—a downstream update.

The consumer still decides what its own consequence means. For example, a UI
may patch a mounted field while a cache evicts an entry. Neither consumer may
reinterpret a Query rebind, replacement, retirement, or unsupported result as
a local patch.

## Stable Entry Points

The ordinary surface is in `worth_query::facade::domain`:

- `consumer.projection_request()` followed by
  `select_display_native_aspect(...)`, `select_display_native_field(...)`,
  `select_derived_native_aspect(...)`, or
  `select_derived_native_field(...)`
- `request.resolve_native_key(...)`, `published.consume_bound(request)`, and
  `settled.native_value(&key, row)` for declaration-bound Foundational values
- `same_installation_with(...)`, `compatible_basis_with(...)`,
  `replacement_with(...)`, `rebind_with(...)`, and
  `execution_sharing_with(...)` for distinct pair-bound decisions
- `settled.into_lifecycle().promote(&mut workspace)`
- `live.refresh(...)`
- `live.replacement_witness_for(...)` followed by
  `live.replace_with(...)`
- `live.rebind_witness_for(...)` followed by `live.rebind_with(...)`
- `live.cancel(...)` or `live.dispose(...)`; stopped transitions return the
  exact predecessor state for retry, and cleanup-pending replacement or rebind
  states expose explicit retry or rollback
- `live.into_managed_lease(&mut workspace)` for one consumer
- `live.share_with(candidate, &mut workspace)` for two compatible consumers
- `lease.drain(&mut workspace)` for ordinary managed delivery
- `lease.consumer_invalidation_delta(delivery)`
- `lease.admit_consumer_invalidation_delta(delta, &workspace)`
- `admitted.attach_consumer_authored_consequence(&workspace, disposition, action)`
- `admitted.materialize_foundational_projection(&workspace, profile)` for a
  fresh descriptive boundary artifact

`same_installation_with`, `compatible_basis_with`, `replacement_with`,
`rebind_with`, and `execution_sharing_with` are deliberately not aliases. Each
returns a relationship-specific witness or a typed denial for the first
incompatible authority dimension. The lifecycle convenience methods readmit
those witnesses against the exact current/candidate pair before changing a
resource.

Native access, sharing, and invalidation also require the corresponding
consumer-support posture. Sharing specifically requires `Sharing`,
`DependencyImpact`, and `Invalidation` to be `Supported`; a declaration that
merely names those requirements does not mint their authority.

Collection windows and query-shaped patch delivery are separate later
boundaries. An invalidation delta does not manufacture either one.

## Core Mental Model

The native access key is not a field-path wrapper. Query derives it from the
exact consumer contract and binds runtime, installation generation, capability,
selection, Foundational contract revision and shape, fact lane, and indexed
slot. A printable path from another request cannot access the value.

The live owner is the one runtime resource doing maintenance. A lease is a
consumer's non-copyable right to observe that owner. Query retains the compiled
dependency closure, compatibility evidence, impact decision, capability and
owner generations, exact lease, the complete conditional semantic path when
present, and the declared native keys affected by the change. That conditional
path retains the declaration, outcome, artifact-reuse posture, and realized
observations. Operational Signal identity remains work evidence rather than
portable semantic authority.

Lifecycle states are operational ownership, not labels beside a snapshot.
Promotion creates the owner; refresh retains its exact impact; replacement and
rebind consume the predecessor only after relationship-specific admission;
cancellation and disposal close the managed resource. Durable diagnostic and
closeout evidence remains inspectable after operational use ends.

An impact decision is Query's classification of the operation-level change. An
invalidation delta translates that same retained decision for one exact
consumer lease. It does not rerun classification and it is not a callback bag.

Foundational locators, field masks, provenance, and the materialized
`DerivedProjection` boundary artifact are descriptive projections of the
delta's complete semantic projection. A raw delta produces only
`StaleRetained` descriptive provenance.
`FreshRetained` materialization requires the exact admitted delta to readmit
its still-current owner epoch. These artifacts can cross a shared semantic
boundary, but they cannot admit a lease, reopen a lifecycle, or authorize a
consequence.

## How It Executes

1. The consumer derives native selections from the installed projection
   contract, resolves keys from the built request, and moves that request into
   `consume_bound`. Query settles the projection with an indexed native layout.
2. Query compiles the semantic dependency closure from installed definition,
   execution, graph, projection, conditional, lineage, and Foundational-native
   evidence. Consumers may inspect
   `settled.semantic_aspect_dependency_closure()` but cannot supply a competing
   closure.
3. Query promotes that projection to one live owner. Refresh classifies owner
   delivery through the retained closure and rebinds native access to the fresh
   settled authority.
4. Replacement and rebind require their own pair-bound witnesses. The old
   resource is closed exactly once; a close failure yields a typed
   cleanup-pending state rather than hiding an orphan.
5. Query admits a singleton lease or proves that a current candidate has the
   exact sharing compatibility and dependency-closure reuse required to join
   the owner.
6. The owner drains once. Query retains one impact decision and one epoch of
   change work, then targets each admitted lease without reclassifying.
   Ordinary live targets and installed semantic targets are selected from
   independent indexes and then deduplicated, so one index cannot suppress the
   other's authority. Canonical field-path overlap is prefix-aware in both
   directions; ordinary request routing includes projection, predicate, and
   ordering dependencies.
7. Each lease translates the shared semantic touches through its own declared
   native-key index.
8. The lease readmits the delta against the current runtime, installation,
   exact capability identity and generation, owner generation, sharing
   evidence, and lease.
9. Consequence attachment readmits currentness again and borrows the workspace
   for the consequence's lifetime. The runtime cannot advance the owner epoch
   while an authority-bearing consequence remains alive.

For a condition-relevant owner delivery, Query re-enters the installed Signal
node first. `ComputedChanged` may produce semantic delivery. Suppressed,
dependency-unchanged, and reverted-clean outcomes do not become computed
patches. Deferred evaluation returns a retained typed stop instead of an epoch.

### Declaration-indexed native access

Starting from the exact consumer contract minted before bound execution:

```rust
use worth_foundational::facade::FieldKey;

let mut builder = consumer.projection_request();
let id = builder
    .select_display_native_field(FieldKey::new("id").unwrap())
    .unwrap();
let request = builder.build().unwrap();
let id_key = request.resolve_native_key(&id).unwrap().into_key();

let settled = published
    .consume_bound(request)
    .unwrap()
    .settle()
    .unwrap();
let id_value = settled.native_value(&id_key, 0).unwrap();
```

`id_value.value()` borrows the Foundational-owned scalar or struct value. A
wrong declaration, field, contract revision, installation, capability,
generation, row, or value shape returns a typed denial before consumer-side
path parsing or fact scanning.

## Small Example

Starting from a settled installed projection:

```rust
use worth_query::facade::domain;

// Inside an application function returning Result<_, String>:
let live = match settled.into_lifecycle().promote(&mut workspace) {
    domain::WorthQueryProjectionPromotionOutcome::Promoted(live) => live,
    _ => return Err("projection was not promotable".into()),
};

let lease = match live.into_managed_lease(&mut workspace) {
    domain::WorthQueryProjectionLeaseAdmissionOutcome::Admitted(lease) => lease,
    domain::WorthQueryProjectionLeaseAdmissionOutcome::Stopped(stop) => {
        return Err(stop.detail().to_owned());
    }
};

let delivery = match lease.drain(&mut workspace) {
    Ok(delivery) => delivery,
    Err(stop) => return Err(stop.error().to_string()),
};
let delta = match lease.consumer_invalidation_delta(delivery) {
    Ok(delta) => delta,
    Err(stop) => return Err(format!("invalidation stopped: {:?}", stop.kind())),
};
let admitted = match lease.admit_consumer_invalidation_delta(delta, &workspace) {
    Ok(admitted) => admitted,
    Err(stop) => return Err(format!("delta admission stopped: {:?}", stop.kind())),
};
let consequence = admitted
    .attach_consumer_authored_consequence(
        &workspace,
        domain::WorthQueryConsumerInvalidationDisposition::Reexecute,
        MyCacheAction::Evict,
    )
    .map_err(|stop| format!("consequence stopped: {:?}", stop.kind()))?;
```

This is the smallest honest example because the consequence is attached only
after the same lease revalidates the delta against the current workspace.

## Real Example

Two compatible consumers can share one owner while keeping different
downstream policies:

```rust
// Inside an application function returning Result<_, String>:
let shared = match live.share_with(settled_candidate.into_lifecycle(), &mut workspace) {
    domain::WorthQueryProjectionSharingOutcome::Shared(shared) => shared,
    domain::WorthQueryProjectionSharingOutcome::Stopped(stop) => {
        return Err(stop.detail().to_owned());
    }
};
let (ui_lease, cache_lease) = shared.into_leases();

// The runtime has routed one authoritative change to the shared live owner.
let ui_delivery = match ui_lease.drain(&mut workspace) {
    Ok(delivery) => delivery,
    Err(stop) => return Err(stop.error().to_string()),
};
let cache_delivery = match cache_lease.drain(&mut workspace) {
    Ok(delivery) => delivery,
    Err(stop) => return Err(stop.error().to_string()),
};

let ui_delta = match ui_lease.consumer_invalidation_delta(ui_delivery) {
    Ok(delta) => delta,
    Err(stop) => return Err(format!("UI invalidation stopped: {:?}", stop.kind())),
};
let cache_delta = match cache_lease.consumer_invalidation_delta(cache_delivery) {
    Ok(delta) => delta,
    Err(stop) => return Err(format!("cache invalidation stopped: {:?}", stop.kind())),
};

assert!(ui_delta.shares_epoch_with(&cache_delta));
assert!(ui_delta.retains_same_impact_as(&cache_delta));
assert!(ui_delta.retains_same_compatibility_evidence_as(&cache_delta));

let ui_admitted = match ui_lease.admit_consumer_invalidation_delta(ui_delta, &workspace) {
    Ok(admitted) => admitted,
    Err(stop) => return Err(format!("UI delta is stale: {:?}", stop.kind())),
};
let cache_admitted =
    match cache_lease.admit_consumer_invalidation_delta(cache_delta, &workspace) {
        Ok(admitted) => admitted,
        Err(stop) => return Err(format!("cache delta is stale: {:?}", stop.kind())),
    };
let ui = ui_admitted
    .attach_consumer_authored_consequence(
        &workspace,
        domain::WorthQueryConsumerInvalidationDisposition::LocalPatch,
        UiAction::PatchMountedField,
    )
    .map_err(|stop| format!("UI consequence stopped: {:?}", stop.kind()))?;
let cache = cache_admitted
    .attach_consumer_authored_consequence(
        &workspace,
        domain::WorthQueryConsumerInvalidationDisposition::Reexecute,
        CacheAction::EvictEntry,
    )
    .map_err(|stop| format!("cache consequence stopped: {:?}", stop.kind()))?;
```

The two actions differ because UI and cache policy belong to their consumers.
The Query meaning, impact, compatibility evidence, and shared epoch do not
change. Epoch counters are counted once per `shares_epoch_with` group; targeted
lease counters are counted once per delta.

## How It Relates To Other Features

- Projection consumption owns the facts and declared native access keys used
  for field-precise narrowing.
- Compatibility admission separately proves same-installation, compatible
  basis, replacement, rebind, or execution-sharing relationships. Matching
  names, digests, definitions, or Foundational comparison alone are not enough.
- Managed lifecycle owns promotion, refresh, replacement, rebind,
  cancellation, cleanup, and disposal. A subscription identifier or copied
  snapshot does not carry that ownership.
- Compiled dependency impact owns the operation-level change classification.
  Invalidation retains that decision instead of deriving a consumer-local one.
- Conditional installed operations use the Runtime Bridge and Signal before
  Query emits a condition-caused delta.
- Certification replay may compare semantic evidence, but it cannot mint an
  ordinary live lease or invalidation authority. Re-executed authoritative
  mutations still travel through the ordinary live owner and lease admission.

## Inspection And Debugging

Inspect `delivery.counters()`, `delta.epoch_counters()`, and
`delta.counters()` separately. Shared epoch work distinguishes affected-owner
capability lookups; live and installed collection/relevance probes; selected,
skipped, and overlap-deduplicated candidates; per-target route probes; fan-out
width; and authoritative batches or touches visited. Per-lease work
distinguishes targeted delivery, canonical-path probes, native-key index
lookups, overlap deduplication, and selected key visits. These are receipts for
operations actually performed, not estimates or counters for scans the runtime
does not execute.

Also inspect:

- `delta.impact()` and `delta.affected_native_keys()`
- `delta.disposition()`, `delta.cause()`, `delta.locality()`, and
  `delta.continuation()`
- `delta.foundational_projection()` for cross-crate semantic scope
- `delta.semantic_projection().canonical_bytes()` for authority-free semantic
  convergence inspection; counters remain separate work evidence
- `admitted.admitted_semantic_projection(&workspace)` for current, retained
  compatibility posture
- typed sharing, delivery, delta, and admission stops before reading their
  explanatory text

## Anti-Patterns

- Cloning labels, digests, locators, masks, or generation numbers into a local
  invalidation token.
- Calling every lease's lower maintenance path independently and treating the
  matching output as shared execution.
- Turning an unsupported or un-narrowable value patch into whole-view
  reexecution without an explicit Query posture.
- Treating a suppressed, unchanged, reverted-clean, or deferred conditional
  result as a computed patch.
- Attaching or retaining a consequence without the exact current workspace
  guard.
- Summing shared epoch counters once per lease.

## Current Limits

- A `ValuePatch` without proven affected declared native keys is exposed as an
  explicit unsupported narrowing posture; Query does not silently broaden it.
- The public dependency closure is inspectable evidence. Query compiles and
  applies it; consumers cannot author closure edges or impact decisions.
- A consumer invalidation delta names declared native keys and collection
  continuation posture; it does not by itself grant collection/window
  maintenance or query-shaped publication authority. Those capabilities are
  available through the granular live invalidation entry points, which
  revalidate the current owner, source basis, retained collection state, and
  consumer policy before Query publishes a patch.
- Operational identities remain Query-owned. Reporting and Foundational
  projections are observable but have no admission power.

## Related Docs

- [Runtime-Installed Domains](./runtime-installed-domains.md)
- [Conditional Installed Operations](./conditional-installed-operations.md)
- [Projection Consumption](../capabilities/projection-consumption.md)
- [Support Matrix And Admission](../foundations/support-matrix-and-admission.md)
- [Installed Operation Re-Execution And Replay](./installed-operation-reexecution-and-replay.md)
- [Granular Live Invalidation](../runtime-surfaces/granular-live-invalidation.md)
