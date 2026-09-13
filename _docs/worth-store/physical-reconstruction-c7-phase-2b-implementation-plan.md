# C.7 Phase 2B Implementation Plan

## Destination

Finish Phase 2 by connecting the admitted durability owner, canonical mutation
fingerprint, Store-issued idempotency lease, and C.5.1 operation allocator at
one real record-append preparation boundary.

`prepare_durable_append` must return a consuming, non-`Clone`
`PreparedPhysicalMutation` only after:

- the record batch and placement policy are admitted;
- every streaming source has been read exactly once into bounded,
  Store-owned prepared payload;
- Store derives the payload digest and exact record-append scope;
- the current security and durability-policy bases enter the canonical
  fingerprint;
- C.5.1 reserves the Store/runtime/lifecycle operation identity; and
- the bounded idempotency registry admits a fresh or same-fingerprint retry
  binding.

Preparation performs no backend, WAL, data, root, or acknowledgment effect.
Phase 3 alone may consume the prepared value into WAL allocation and append.

## Public API

Add:

```rust
PhysicalRecordSubmission::prepare_durable_append(
    batch: RecordAppendBatch,
    placement: AdmittedRecordPlacementPolicy,
    request: PhysicalMutationRequest,
) -> PhysicalMutationPreparationOutcome
```

The outcome remains the exact six-way `ProofOutcome` topology:

- `Success(PreparedPhysicalMutation)`
- `Denied(PhysicalMutationPreparationDenial)`
- `Deferred(PhysicalMutationPreparationDeferred)`
- `Stale(PhysicalMutationPreparationStale)`
- `RebindRequired(PhysicalMutationPreparationRebindRequired)`
- `Failed(PhysicalMutationPreparationFailure)`

The prepared value exposes immutable mutation, idempotency, fingerprint,
deadline, signal-profile, resource-shape, and fresh-versus-duplicate
observations. It exposes no constructor, `Clone`, execution, acknowledgment,
retry, WAL, group, range, or fate authority in Phase 2.

## Destination Directory Skeleton

```text
crates/worth-store/src/physical_runtime/
├── durability/mutation/
│   ├── admission.rs
│   └── admission/
│       ├── outcome.rs
│       └── prepared.rs
├── record_serving/publication/
│   └── durable_preparation.rs
│       └── payload.rs
└── work/submission/
    └── mutation_identity.rs
```

One file in a growth-ready semantic directory is intentional. Phase 3 may add
WAL binding beside mutation admission without turning any file into a
cross-domain bag.

## Canonical Derivation

Payload digest version 1 hashes:

- the domain `store.physical.record-append.payload.v1`;
- record count; and
- each record's exact length and bytes in caller order.

Record-append scope version 1 hashes:

- the domain `store.physical.record-append.scope.v1`;
- the admitted physical record-format identity; and
- segment pages, extent threshold, page fill, and manifest capacity.

The canonical request fingerprint then binds Store, admitted durability-policy
identity, scope, payload, platform-durable request, record-append operation
family, and the stable fingerprint of every admitted security basis.

Deadline, lease frontier, lifecycle/runtime generation, reserved operation,
allocation, queue, schedule, cancellation, completion, WAL, and observation
facts remain absent from request equivalence.

## Failure And Cleanup Law

- Invalid batch or placement is `Denied` before source reads.
- Prepared-payload capacity pressure is `Deferred`.
- Producer rejection, early end, excess bytes, invalid transfer count,
  canonical derivation failure, and operation-identity exhaustion are typed
  `Failed` outcomes with no physical effect.
- Released or advanced owners are `Stale`.
- Foreign Store, runtime, or durability-policy identities are
  `RebindRequired`.
- Same-key/different-fingerprint use is a hard idempotency denial.
- Same-key/same-fingerprint use returns the existing unresolved mutation
  identity and never authorizes a second effect.
- Partially prepared vectors and consumed sources are ordinary owned values;
  every unsuccessful branch drops them immediately. No retained temp
  directory, scratch file, zip/archive inspection, or leaked staging artifact
  is permitted.

## Transitional Cutover

The pre-C.7 `prepare_append` and `append_batch` lane remains temporarily
callable only because Phase 2 cannot execute the new prepared value before the
Phase 3 WAL join exists. It is not evidence of C.7 durability and must not be
wrapped or relabeled. Phase 3 begins the parallel cutover; Phase 8 removes or
narrows the old lane as required by the C.7 specification.

## Proof

Phase 2B closes only when:

- real serving journeys prove fresh, duplicate, conflict, expiry, pending
  bound, foreign identity, stale lifecycle, source-shape, and cleanup behavior;
- independent fingerprint reconstruction matches the versioned golden vector;
- compile attacks cannot construct keys, leases, fingerprints, mutation
  identities, bindings, or prepared values;
- allocation-feedback and omitted-field mutants fail;
- preparation produces zero backend/WAL/data/root effects;
- boundary, agent-context, line-cap, and focused workspace checks are green;
  and
- the reviewed Git diff names the changed source boundary and any remaining
  transitional debt is called out directly in the phase handoff.
