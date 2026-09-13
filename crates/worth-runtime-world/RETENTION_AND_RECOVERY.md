# Retention and recovery contract

Runtime World accounts for bounded populations with one installed
`RuntimeWorldBudgets` value. The limits cover live product branches, retained
composite commits, Runtime World metadata, active observations, active
publication attempts, retained product-unpublished records, retained-partial
metadata, unique exact component pins, in-flight pin reservations, and
owner-created component custody records. Zero limits are rejected during
installation; no `Default` can silently omit a bound.

The retention owner keys its unique leases by independent exact Relational and
Signal bases. A composite identity is not a component registry key. Repeated
use of a pinned component basis shares its existing lease; acquiring a fresh
obligation does not re-observe the component's current head.

Fresh product observations reserve the active-observation budget. Clones share
one observation obligation and its charge until the final clone drops. Creation
reserves the returned observation before an owner fork. Close reports outstanding
observations while their caller-owned obligations remain live.

`ProductUnpublishedOwnerEffects` is not performed authority, a rollback token,
or a replay record. It records the exact expected and last observed heads, both owner
progress postures, successor basis when present, live obligations, cause, legal
next actions, deadline/age, owner-effect count, and metadata-byte accounting.
Its `ProductUnpublishedRecoveryHandle` is a non-authorizing reference to that
record. Recovery cannot call an unperformed sibling or move a product
reference.

Close exposes retained records and returns owner-created component retirement
work; it does not delete component branches. `OwnerLost` requires an actual
owner-unavailable denial. Retention capacity, arithmetic, or acquisition-panic
failures instead retain `RetentionAdmissionDenied`; destination authority
mismatches retain `DestinationAdmissionDenied`. Neither justifies owner closure.

The active-custody owner preinstalls an attempt record in the same bounded
recovery catalog. Caller abandonment atomically converts its reservation to
retained accounting before releasing operation admission. Inspection moves the
existing evidence and resources into the retained view without component calls.
`BindingReserved` means the component pair has not been bound;
`PublicationPinsRetained` preserves the original `ActivePublicationAttempt` pins
without claiming a dependency transfer. `ProductHeadPinsRetained` preserves
the original head-class claims when materialization preceded caller loss but
the product cell did not move. Abandonment retains any unused history capacity
until explicit cleanup, and close counts that capacity as a live obligation.

Ordinary production reservation, settlement, readiness, and final product
movement now carry this custody throughout their transitions. Tests cover real
two-owner settlement Drop, an unwind inside Signal's apply callback, invalid
ready-basis evidence, materialization and committed-boundary unwind, concurrent
inspection, close, and exact Relational identity repair. An explicit terminal
acquires its caller view while the catalog still owns the accounting conversion
lock, preventing cleanup from removing it before delivery.

Branch creation retains each actual fork and its exact destination in the same
record. Abandonment after destination assembly preserves its original head pair,
head history, observation pair, observation history, and recovery slot: seven
live obligations. Explicit cleanup releases those claims and returns the exact
component-retirement work. Registry-issued installation evidence survives
retirement and name reuse, so caller loss after insertion cannot expose a false
unpublished record. Recovery continues to settle or clean up only.

After a successful CAS, the history entry owns the canonical performed facts.
The owner can recover a caller's lost delivery without another component call
or product movement. Normal delivery and recovery claim the same exclusive
lane. Dropping an unconsumed delivery makes it available again; consumption
permanently closes that lane. A live claim keeps the entry and its metadata
charge retained even after branch retirement. Its old head is an exact
snapshot, not a permanently held active-observation obligation.

## Public cleanup and bounded inspection

Use `inspection_port().recovery_page(cursor, maximum)` for bounded catalog
inspection. Continue with the opaque `RuntimeWorldRecoveryCursor` returned by
`next_after()`. Every examined storage slot, including a vacancy, spends one unit
of `maximum`; an empty page may still have a continuation. Cursors remain valid
when earlier rows are removed. Pages follow slot order and are live observations:
concurrent insertion into an earlier reused slot may require a fresh scan.

Recovery reserves one reusable slot and identity hash entry before effects.
Active, retained and busy transitions replace that slot without index insertion
or growth. Indexing is expected/amortized O(1); slot paging is O(examined slots).
Peak slot/index capacity is bounded by installed admission limits, not lifetime
attempt count. The logical per-record metadata charge includes its slot/index.

A recovery handle is owner/attempt-affine; `inspect_effects` obtains a
live view, `continue_effects` consumes it, and `release_effects(handle, minimum_age)`
performs eligible explicit cleanup. Drop other caller views before release.
Retirement work returned by cleanup remains the caller's explicit owner operation.
`recover_performed(commit)` recovers only the original unconsumed exclusive delivery.
It cannot promote a retained partial or duplicate a consumed publication.

History and pin reclamation have separate caller-visible batch bounds. Unique pin
registry entries may outlive their final dependency as idle metadata; those entries
are distinct from live component leases. The final dependency releases the owner
lease. Every prepared attempt reserves worst-case capacity for two component
acquisitions, even when an eventual operation can reuse a basis.

The court's named Court/Standard/Scale profiles vary B/H/U/A/P/O populations at
[1,4,8], [1,8,32] and [1,32,96], respectively. A and P are size minus one; U is
size plus one. U retains live distinct pins with H co-variation; P also changes
B/H/U. Each run prints the actual populations. Budgets are B128/H512/O512/A128/
P128/U1024, 256 in-flight acquisitions, 256 custody records and 16 MiB each for
history and partial metadata. W varies independent concurrent writers at 1/4/16.

Three fresh repetitions report variance and p50/p95/p99. Observation cold means
the first probe after population construction; warm means 4/6/8 repeated probes
on that owner. Publication follows those probes. W timings cover a fresh concurrent
batch; per-attempt counters remain separate from aggregate contacts and wins.
Timing distributions are diagnostic, with no machine-dependent latency threshold.
Structural assertions compare actual counter deltas; byte charges describe World
metadata only and do not invent Signal component-state bytes.

The scheduled CI job runs all three ignored profiles and the longer seeded model,
checks that every profile executed, and uploads its full output. Ordinary and
operation-control lanes remain separate in the single certification target.

## Adjacent Signal batch-read limitation

The warm routing court uses single-target `SignalTransaction::read`. The seeded
sequence exposed pre-existing batch behavior in Signal's `read_many_with_executor`
(`logic/transaction/runtime/execution/transaction_evaluation/access.rs`): after
`mark_changed` stages an upstream producer, a previously clean requested leaf can
be filtered before that producer settles. Seed `0x9172`, checked prefix 21, first
publishes grain 2, reads routed load 8, then combines grain 9 and expects routed
load 0; the batch path returned stale 8. The same sequence and independent expected
values pass through `read`, which settles dependencies. Batch-read semantics are
not certified or repaired by this milestone; this is an untouched Signal owner
follow-up, not an alternate World authority path.
