# Milestone 10.8: Original-Request Funding Of Retained Owners

**Status:** design brief; not implemented. Parent: [Milestone 10](milestone-10.md).
Requires the original-request carriage of [10.1](milestone-10.1.md) and share
issuance from [10.7](milestone-10.7.md). Migrates owner by owner beside
[10.4](milestone-10.4.md) and [10.5](milestone-10.5.md).

Retained owners (custody, recovery records, checkpoint and close state) must be
paid for by the request that created them, for as long as they live. Today every
site picks the payer, charges and refunds by hand, and a wrong or missing payer is
found only when a test exhausts memory. Make "funded by the original request" a
type, and make each retained type declare its own footprint.

This types an obligation that already exists. It does not add quotes to ordinary
domain work, per-feature pools or a new invoice; the parent's prohibition holds.
The live request is still passed at each call for that call's cancellation and work
budget. It is never captured in a retained owner (10.2).

| Phase | Usable result | Requires | Enables | Parallel constraints |
|---|---|---|---|---|
| 1 | Funding lease, funded values and declared footprints in Execution. | 10.1 original-request carriage. | Owner migration; fixture grants from footprints. | Freeze before owners migrate. |
| 2 | Phase storage: exclusive phases reserve the maximum, not the sum, and cannot share a frame. | Phase 1. | Bounded stack and peak memory for phased owners. | Per owner. |
| 3 | Native durability, World close/recovery, Store recovery and Query receipts migrated. | Phases 1–2; 10.7 for share issuance. | Group closure. | One owner per slice. |

## Investigation

**Charge sites.** Across production code there are about 1,500 charge, reserve and consult sites in about 930 files. Only 17 sites name the original request explicitly; at every other site the payer is a convention.

**Churn and incidents.** In the Milestone 3 integration on 2026-11-03:
- 16 of 74 commits corrected funding: original capture, the original request's scratch, prepaid recovery reports, binding storage.
- At least 6 commits re-pinned a fixture's byte grant to a newly observed peak.
- 4–5 commits separated phase storage that had piled up in one stack frame, including a 552 KiB Bank transfer frame.
- A history cursor was compacted by hand because it reserved its exclusive phases separately.

## Phase 1: Lease, funded values, footprints

```rust
pub struct FundingLease<'orig> { /* original request accounting, affinity */ }

impl ExecutionRequest<'_, '_> {
    /// Only at an entry or phase boundary, only from the original request.
    pub fn funding_lease(&self) -> Result<FundingLease<'_>, KernelStop>;
}

pub trait Footprint {
    /// Retained bytes this value holds while it lives.
    fn footprint(&self) -> RetainedBytes;
}

pub struct Funded<'orig, T: Footprint> { /* value + its reservation */ }

impl<'orig> FundingLease<'orig> {
    pub fn fund<T: Footprint>(&self, value: T) -> Result<Funded<'orig, T>, RequestStop>;
}
```

- **Original request only:** a retained owner's constructor takes `&FundingLease`, not a request. The live request at a later call cannot produce one for an earlier owner, so paying with the wrong request fails to compile.
- **Charge and refund:** `fund` charges the declared footprint against the original request's accounting. Retirement or explicit reclaim refunds it, and Drop performs only bounded local release.
- **Footprints:** declared on the retained type at its owner, as a constant where the layout is fixed. Fixtures compute grants by summing the footprints of what they retain, instead of pinning observed peaks.
- **Stops:** a funding stop is the request's stop ([10.6](milestone-10.6.md)), never an owner refusal.

| Requirement / law | Setup and action | Expected observation / defect exposed | Boundary and lane |
|---|---|---|---|
| Wrong payer | Construct a retained owner from a later live request. | Compile failure; the lawful twin uses the original lease. | Consolidated compile-contract lane. |
| Exact charge | Fund, retire and reclaim a retained owner. | Charge equals the declared footprint; reclaim returns to baseline. | Execution owner tests plus one Native owner court. |
| Fixture grant | Grant exactly the summed footprints, then one byte less. | Success at the sum; typed memory stop one byte short. | Migrated owner court. |

## Phase 2: Phase storage

A phased owner declares its phases as consuming types, `PhaseA -> PhaseB -> ...`. Its storage reservation is the maximum footprint over phases that never coexist, not their sum. Because each phase consumes the previous one, their storage cannot share a stack frame. That makes frame separation structural, where today it is fixed case by case.

| Requirement / law | Setup and action | Expected observation / defect exposed | Boundary and lane |
|---|---|---|---|
| Max not sum | A three-phase owner with distinct phase footprints. | Reservation equals the largest phase. | Owner test. |
| Default stack | Run the migrated owner's deepest journey with `RUST_MIN_STACK` unset. | No overflow; frame sizes from the build stay under the owner's budget. | Owner direct control. |

## Phase 3: Owner migration

Migrate in order, one owner per slice:
1. Native durability (checkpoint, recovery, retained images);
2. World close and recovery reports;
3. Store recovery and serving custody;
4. Query receipts and managed outputs.

Each slice:
- converts its retained constructors to take the lease;
- declares footprints on its retained types;
- deletes hand-written charge and refund pairs;
- re-derives its fixture grants from footprints.

A slice that finds a site paying with the wrong request records it as a defect fix.

Closure:
- every retained owner in the migrated owners is constructed only through a funding lease;
- the fixture byte grants in those owners are derived from footprints;
- the peak-retained-memory and frame-size changes are recorded against the Milestone 3 baselines.
