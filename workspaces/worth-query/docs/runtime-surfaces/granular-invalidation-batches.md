# Granular Invalidation Batches

A conditional clock observation that reconsiders authoritative commits also
carries the exact lower-runtime deliveries those commits caused. Host hands
them to the caller as one opaque batch. This page covers receiving that batch,
what it proves, and moving it between owners without losing or widening its
authority.

## Where the batch comes from

```text
committed truth (Relational)
    -> installed Bridge correspondence
    -> optional performed Signal consequence
    -> clock observation receipt
    -> WorthQueryGranularInvalidationDeliveryBatch
```

Take the batch from the receipt of an accepted or duplicate clock observation:

```rust
use worth_query_host::facade::primary_graph::WorthQueryConditionalClockObservationOutcome;

let mut receipt = match clock.observe(&request_execution) {
    WorthQueryConditionalClockObservationOutcome::Accepted(receipt)
    | WorthQueryConditionalClockObservationOutcome::Duplicate(receipt) => receipt,
    other => return handle(other),
};
let batch = receipt.take_granular_invalidation_batch();
```

A duplicate reading still carries every owed invalidation, including commits
reconsidered during that observation; skipping its batch loses them. A stale or
reordered reading carries none, and its owed invalidations move to the next
accepted or duplicate reading's batch.

## What the batch proves

The batch type lives in `worth_query_host::facade::primary_graph`.

- **Runtime affinity.** Each batch is bound to the
  `WorthQueryGranularInvalidationInstallation` of the application runtime that
  observed it. The runtime exposes the current one as
  `granular_invalidation_installation()`. A batch from a stale or foreign
  installation is refused before any read or publication.
- **Direct truth is not performed work.** A direct truth delivery and a
  performed Signal delivery are different facts. A comparator may suppress
  Signal work while the truth change remains real. A consumer may use direct
  truth for a local field patch without claiming that Signal executed.
  `observation()` counts them separately: `direct_truth_delivery_count()` and
  `signal_performed_delivery_count()`. `bridge_deliveries()` exposes each
  delivery's correspondence receipt and optional `performed_signal()` for
  read-only inspection.
- **Source read basis.** `source_read_basis()` names the settled primary truth
  basis the deliveries may be read against.
- **Coverage.** `coverage()` reports whether the deliveries are every relevant
  change since the previous batch, or whether the producer lost subscription
  continuity and every consumer must refresh its full scope at the batch's
  source read basis (`RefreshAll`). A `RefreshAll` batch is never empty.

The batch is transport evidence. It grants no admission, execution or
publication, and its borrowed Bridge products cannot authorize work on their
own.

## Moving a batch between owners

Host-owned transport may forward or coalesce batches with two operations on the
batch itself:

- `retain_direct_truth_transport(&installation)` keeps a direct-truth copy for
  another consumer without cloning performed Signal authority. It returns
  `None` unless the current installation admits the batch.
- `merge_transport_batch(other)` merges two batches only when their
  installation and source read basis are identical. Otherwise it refuses with
  `WorthQueryGranularTransportMergeDenial::ForeignInstallation` or
  `::SourceReadBasisMismatch`. A `RefreshAll` side makes the merged batch
  `RefreshAll`.

Merging a performed batch with its own direct-truth copy, in either order,
keeps one performed Signal delivery and both direct deliveries; performed
authority is never duplicated. The integration court's
`granular_transport_batches_merge_within_one_installation_in_either_order` and
`granular_transport_merge_refuses_a_foreign_installation_batch` tests pin both
rules.

## Anti-patterns

- Treating a Bridge delivery as proof that Signal executed.
- Unpacking the batch into raw field events, CDC rows or copied aspect and
  scope tuples.
- Reusing a batch or installation after reinstall, restore or rebind; old
  batches are rejected.
- Moving semantic locality into physical shard or worker identifiers.

## Related docs

- [Conditional Installed Operations](../domain-capabilities/conditional-installed-operations.md)
- [Aspects And Authority Lanes](../modeling/aspects-and-authority-lanes.md)
- [Cross-Runtime Causal Inspection](../capabilities/cross-runtime-causal-inspection.md)
