# Managed Artifact Ownership And Native Access

## What This Feature Is

Managed artifact ownership lets Query carry large or provider-native working
data through an installed computation without copying it into an untyped
payload bag. Use it when a workflow stage must borrow, transfer, retain, read,
or dispose an artifact while preserving its exact runtime, occurrence,
contract, and provider ownership.

Native access adds bounded row, field, chunk, and projection reads without
letting borrowed provider memory escape.

## Why You Use It

- Move artifacts between workflow stages without cloning their payload.
- Retain an artifact under an explicit lease while its owner stays accountable.
- Read provider-native batches or fields without scalar conversion.
- Bound actual projection memory for variable-width values.
- Preserve cleanup authority when provider access or destruction fails.

## Stable Entry Points

Import these through
`worth_query_host::facade::installed::domain_computation`:

- `WorthQueryMoveOnlyArtifactHandle`
- `WorthQueryBorrowedArtifactView`
- `WorthQueryTransferredArtifactHandle`
- `WorthQueryRetainedArtifactLease`
- `WorthQueryDisposedArtifact`
- `WorthQueryStageArtifactReader`
- `WorthQueryArtifactRowBatchRequest`
- `WorthQueryArtifactFieldSliceRequest`
- `WorthQueryArtifactChunkRequest`
- `WorthQueryArtifactProjectedChunkRequest`
- `WorthQueryArtifactScalarFallbackRequest`
- `WorthQueryArtifactNativeAccessOutcome`

Artifact production and transfer admission are minted by the running installed
workflow. Application code does not manufacture handles or access authority.

## Core Mental Model

The handle owns a lifecycle right, not arbitrary access to a value.

```text
provider produces an artifact under an installed contract
  -> Query mints one move-only owner handle
  -> owner may borrow, retain, transfer, or dispose
  -> transferred stage admits its declared native reader
  -> every borrow ends before progress or disposal
  -> one explicit terminal disposition releases the provider value
```

`borrow()` creates a lifetime-bound view. `retain()` creates a move-only lease.
`transfer()` consumes the owner and produces a handle for the admitted
destination stage. `dispose()` consumes the handle and returns the physical
release disposition.

Artifact identity, occurrence identity, semantic projection, retained bytes,
owner generation, and provider release posture travel together. Matching
strings cannot recombine them.

Native reads are admitted against the installed access-path contract:

- row batches borrow multiple declared fields;
- field slices borrow one declared field;
- chunks preserve progress across bounded batches;
- projected chunks allow an installed provider-native projection;
- scalar fallback is explicit and call-amplification bounded.

The callback on `with_rows` or `with_field_slice` prevents a provider pointer or
borrowed view from escaping. Projected chunks own bounded output. Their memory
evidence includes heap allocations inside variable-width scalar families.

## How It Executes

```text
installed producer contract + provider output
  -> move-only owner handle
  -> optional borrow or retained lease
  -> workflow transfer admission
  -> transferred handle
  -> stage reader admission
  -> bounded native access
  -> transfer again or explicit disposal
```

An access denial performs no later provider work. A provider panic unwinds
through Query and the managed artifact is disposed exactly once.

## Small Example

```rust
let borrowed = artifact.borrow("inspect-candidates")?;
inspect_metadata(borrowed.semantic_projection());
drop(borrowed);

let transferred = artifact.transfer(&workflow.transfer_admission(
    "candidate-generation",
    "candidate-validation",
)?)?;
```

The borrow must end before the consuming transfer. Rust enforces that ordering.

## Real Example

```rust
use worth_query_host::facade::installed::domain_computation as execution;

let reader = execution::WorthQueryStageArtifactReader::admit(
    &transferred,
    &artifact_access,
)?;

let mut chunks = reader.projected_chunks(
    execution::WorthQueryArtifactProjectedChunkRequest::new(
        layout_identity,
        "candidate-score-projection",
        chunk_rows,
    ),
)?;

while chunks
    .next(|chunk| {
        for row in 0..chunk.row_count() {
            consume_candidate(chunk.row(row).expect("row is within the chunk"));
        }
        inspect_allocated_capacity(chunk.allocated_capacity_bytes());
    })?
    .is_some()
{
}

inspect_native_access(chunks.evidence());
let owned = transferred
    .into_owned_output()
    .map_err(handle_lease_only_transfer)?;
let disposed = owned.dispose()?;
inspect_release(disposed);
```

Use the request constructor belonging to the selected access family and the
installed layout. A projection name alone cannot authorize access. The cursor
retains progress and evidence so a short final chunk cannot skip rows.

## How It Relates To Other Features

- [Installed Computation Artifact Contracts](./installed-computation-artifact-contracts.md)
  declares the legal ownership and access paths.
- [Execution Resource Admission And Managed Runs](./execution-resource-admission-and-managed-runs.md)
  accounts for retained bytes and cleanup.

## Inspection And Debugging

Inspect:

- handle and occurrence identities;
- `semantic_projection()` and `owner_snapshot()`;
- `retained_bytes()` and owner generation;
- request layout, fields, row start, and chunk width;
- native access counters and allocated-capacity evidence;
- transfer, disposal, destructor, and recovery dispositions.

A zero-provider-call denial is useful evidence that layout, contract, or
session affinity failed before physical access.

## Anti-Patterns

- Storing artifacts as `Box<dyn Any>` and downcasting at each stage.
- Reconstructing a handle from an artifact or occurrence ID.
- Keeping a borrowed row, field, pointer, or slice after the callback returns.
- Calling scalar access in a loop when a declared bulk path exists.
- Counting only the outer result vector for a variable-width projection.
- Advancing workflow progress while a previous chunk remains pending.
- Treating `Drop` as sufficient evidence of provider cleanup.
- Using the handle as mutation, invariant, publication, or commit authority.

## Current Limits

- Handles and leases are runtime-affine and move-only.
- Native access is available only for installed layouts and paths.
- Scalar fallback may be denied or bounded more tightly than bulk access.
- Provider release failures require explicit recovery handling.

## Related Docs

- [Installed Computation Artifact Contracts](./installed-computation-artifact-contracts.md)
- [Execution Resource Admission And Managed Runs](./execution-resource-admission-and-managed-runs.md)
