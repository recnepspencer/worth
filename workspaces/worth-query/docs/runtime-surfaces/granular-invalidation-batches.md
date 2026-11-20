# Granular Invalidation Batches

Reference for the granular invalidation batch a conditional clock observation
carries. Every statement below is the item's signature or its doc comment in
source.

Path: `worth_query_host::facade::primary_graph` (re-exports Execution's
`primary_graph` facade).

## Taking the batch

- `WorthQueryConditionalClockObservationReceipt::take_granular_invalidation_batch(&mut self) -> WorthQueryGranularInvalidationDeliveryBatch`
  ([source](../../crates/worth-query-execution/src/domain_computation/primary_graph/conditional_operation/clock_observation.rs)). Doc comment:
  "Consume the exact lower-runtime deliveries observed while this clock
  observation reconsidered authoritative commits. The returned carrier is
  transport evidence only."
- `WorthQueryConditionalClockObservationOutcome` variants with a receipt are
  `Accepted(receipt)` and `Duplicate(receipt)`. Per the `Duplicate` doc
  comment, its batch still carries every owed granular invalidation and must be
  consumed exactly as for an accepted reading, or those invalidations are lost.
  Per the `Stale` and `Reordered` doc comments, invalidations already owed
  stay owed to the next accepted or duplicate reading's batch.

## The batch

`WorthQueryGranularInvalidationDeliveryBatch`
([source](../../crates/worth-query-execution/src/domain_computation/primary_graph/granular_invalidation/delivery.rs)). Doc comment: "Opaque
execution-owned carrier for one clock observation's granular lower-runtime
deliveries."

- `len() -> usize`, `is_empty() -> bool` (a `RefreshAll` batch is never
  empty).
- `coverage() -> WorthQueryGranularInvalidationCoverage`: `Exact` ("every
  relevant change since the previous batch") or `RefreshAll` ("the producer
  lost subscription continuity and rebuilt from truth ... Every consumer
  refreshes its full scope at the batch's source read basis").
- `observation() -> WorthQueryGranularInvalidationObservation`
  ([source](../../crates/worth-query-execution/src/domain_computation/primary_graph/granular_invalidation/observation.rs)):
  `direct_truth_delivery_count()`, `signal_performed_delivery_count()`,
  `bridge_performed()`, `signal_performed()`. Doc comment: "These rows
  describe transport performed by the primary runtime. They do not predict or
  authorize Query maintenance."
- `installation() -> &WorthQueryGranularInvalidationInstallation`. The
  application runtime exposes its current installation as
  `WorthQueryPrimaryGraphApplicationRuntime::granular_invalidation_installation()`.

## Host-owned transport (`#[doc(hidden)]`)

These methods are hidden from rustdoc and exist for certification and
host-owned transport:

- `bridge_deliveries()`: "Read-only integration inspection ... the borrowed
  Bridge products cannot authorize Query work or publication."
- `retain_direct_truth_transport(&self, &WorthQueryGranularInvalidationInstallation) -> Option<Self>`:
  "Retain a direct-truth transport copy without cloning performed Signal
  authority. The exact current installation must already admit this
  execution-owned batch."
- `merge_transport_batch(self, other: Self) -> Result<Self, WorthQueryGranularTransportMergeDenial>`:
  "Merge transport batches only while installation and source basis are
  exactly identical." Denials: `ForeignInstallation` ("The batches come from
  different installations") and `SourceReadBasisMismatch` ("The batches were
  observed against different source read bases").

The integration court tests
`granular_transport_batches_merge_within_one_installation_in_either_order` and
`granular_transport_merge_refuses_a_foreign_installation_batch` exercise both
transport operations through real Host clock observations.
