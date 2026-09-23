# worth-store-io-scheduler

## What This Feature Is

`worth-store-io-scheduler` turns already-admitted Store work into bounded queue
demand. Use it when physical work has a foreground reservation and must carry an
exact resource budget, locality, durability, security scope, and ordering
posture into queue admission.

The scheduler decides whether and how work may enter an I/O queue. It does not
read or write storage, own resident bytes, or decide the final fate of a Store
operation.

## Why You Use It

- Protect latency-sensitive foreground work from background maintenance.
- Keep queue demand bound to the security scope and backend capability already
  admitted for the reservation.
- Prevent callers from combining incoherent read, durability, recovery, and
  writeback settings.
- Observe queue execution, backpressure, denial, and policy violations through
  typed outcomes and counters.

## Stable Entry Points

For ordinary physical foreground work:

- `PhysicalForegroundWorkDeclaration::read(...)`
- `PhysicalForegroundWorkDeclaration::buffered_write(...)`
- `PhysicalForegroundWorkDeclaration::durable_write(...)`
- `PhysicalForegroundWorkDeclaration::with_secure_io_scope(...)`
- `lower_physical_foreground_work(...)`

After lowering:

- `admit_queue_policy_receipt(...)`
- `QueueExecutionAdmissionRequest::new(...)`
- `admit_queue_execution_plan(...)`
- `execute_ready_queue_plan(...)`

The buffer-pool and WAL lowering functions are producer-specific adapters. Do
not use a buffer-pool adapter to fabricate ordinary physical work in a test or
in a non-pool consumer.

## Core Mental Model

A foreground reservation is the admitted claim on queue capacity. It already
carries the lane, backend requirement, and Store security identity. A physical
foreground declaration consumes that reservation and adds the physical
operation's locality, resource shape, flush epoch, and operation kind.

The operation kind fixes the policy posture:

| Constructor | Durability | Writeback | Recovery ordering |
| --- | --- | --- | --- |
| `read` | read-only | none | not recovery-critical |
| `buffered_write` | buffered write | deferred within the flush epoch | not recovery-critical |
| `durable_write` | platform durable | immediate | not recovery-critical |

Security identity is derived from the reservation. Callers cannot supply a
second tenant, key, or authenticity identity for grouping. The declaration is
move-owned, so one reservation cannot be lowered into two queue declarations.

## How It Executes

1. The Store admits a foreground reservation against backend capacity.
2. The caller selects the typed physical constructor matching the operation.
3. The scheduler derives security and fixed policy fields while lowering the
   declaration into `QueueWorkDeclaration`.
4. Foundational policy admission binds that exact work declaration to its
   budget decision.
5. Backend capability and secure-I/O checks admit an execution-ready plan.
6. The physical executor performs the media effect and returns completion.
7. The Store, not the scheduler, settles the operation's final fate.

Any mismatch is rejected at the boundary that has enough evidence to explain
it. Lowering can reject an invalid resource shape; queue admission can reject a
missing grouping basis, backend mismatch, policy mismatch, or missing secure-I/O
receipt.

## Small Example

```rust
use worth_store_contracts::QueueProducerResourceShape;
use worth_store_io_scheduler::foreground_reservation::ForegroundReservationReceipt;
use worth_store_io_scheduler::{
    lower_physical_foreground_work, PhysicalForegroundWorkDeclaration,
    QueueExecutionAdmissionDenial, QueueLocalityIdentity, QueueWorkDeclaration,
};

fn declare_read(
    reservation: ForegroundReservationReceipt,
    locality: QueueLocalityIdentity,
) -> Result<QueueWorkDeclaration, QueueExecutionAdmissionDenial> {
    let resources = QueueProducerResourceShape::new()
        .with_queue_slots(1)
        .with_bandwidth_tokens(4096)
        .with_worker_permits(1);

    lower_physical_foreground_work(PhysicalForegroundWorkDeclaration::read(
        reservation,
        locality,
        resources,
        42,
    ))
}
```

The caller supplies the operation facts. The scheduler supplies the coherent
read posture and derives security from the consumed reservation.

## Real Example

```rust
use worth_store_contracts::QueueProducerResourceShape;
use worth_store_io_scheduler::foreground_reservation::ForegroundReservationReceipt;
use worth_store_io_scheduler::{
    lower_physical_foreground_work, PhysicalForegroundWorkDeclaration,
    QueueExecutionAdmissionDenial, QueueLocalityIdentity, QueueWorkDeclaration,
    SecureIoPreservationReceipt,
};

enum PhysicalOperation {
    Read,
    BufferedWrite,
    DurableWrite,
}

fn declare_physical_work(
    operation: PhysicalOperation,
    reservation: ForegroundReservationReceipt,
    locality: QueueLocalityIdentity,
    resources: QueueProducerResourceShape,
    flush_epoch: u64,
    secure_io: Option<SecureIoPreservationReceipt>,
) -> Result<QueueWorkDeclaration, QueueExecutionAdmissionDenial> {
    let declaration = match operation {
        PhysicalOperation::Read => PhysicalForegroundWorkDeclaration::read(
            reservation,
            locality,
            resources,
            flush_epoch,
        ),
        PhysicalOperation::BufferedWrite => {
            PhysicalForegroundWorkDeclaration::buffered_write(
                reservation,
                locality,
                resources,
                flush_epoch,
            )
        }
        PhysicalOperation::DurableWrite => {
            PhysicalForegroundWorkDeclaration::durable_write(
                reservation,
                locality,
                resources,
                flush_epoch,
            )
        }
    };
    let declaration = match secure_io {
        Some(receipt) => declaration.with_secure_io_scope(receipt),
        None => declaration,
    };
    lower_physical_foreground_work(declaration)
}
```

The reservation remains authoritative for lane, backend requirement, and
security identity. The operation selects a closed constructor rather than
assembling durability and writeback fields independently. The resulting queue
declaration is still only demand: policy and backend admission must succeed
before execution.

## How It Relates To Other Features

- `worth-store` owns physical operation identity, lifecycle, and final
  settlement. Its physical runtime is the ordinary caller.
- `worth-store-buffer-pool` owns resident bytes, frame and lease state, dirty
  state, and eviction eligibility. Scheduler receipts are not residency
  authority.
- `worth-store-physical-backend` owns exact filesystem effects. Scheduler
  admission never proves that an effect happened.
- `worth-store-security` supplies the scope admitted into the foreground
  reservation and any secure-I/O preservation receipt.
- Foundational policy admission must bind the exact scheduler work and resource
  budget before backend admission.

## Inspection And Debugging

Inspect a lowered `QueueWorkDeclaration` through `class`,
`backend_requirement`, `security_scope_identity`, `durability_class`,
`requested_budget`, `grouping_basis`, and `secure_io`.

At execution, inspect the typed outcome:

- executed evidence and counters for completed queue work;
- backpressure or denial plus its cause and counters;
- a violation plus its exact cause and counters.

These observations explain scheduler behavior. They do not replace Store
settlement or backend completion evidence.

## Anti-Patterns

- Do not construct `QueueWorkDeclaration::foreground` directly for ordinary
  physical work. It lacks the physical grouping basis and will be rejected.
- Do not copy security identifiers into a second grouping structure. The typed
  declaration derives them from the reservation.
- Do not use buffer-pool lowering to make certification fixtures for work that
  does not exercise buffer-pool authority.
- Do not treat queue admission, counters, or execution plans as proof of
  residency, filesystem effects, or final Store settlement.
- Do not reuse a declaration or admitted plan. Progression is move-owned by
  design.

## Current Limits

- Physical foreground constructors model ordinary reads, buffered writes, and
  platform-durable writes.
- They intentionally mark work as not recovery-critical. Recovery ordering must
  come from a recovery-owned boundary rather than a caller-selected flag.
- Secure-I/O work still requires a matching preservation receipt during queue
  admission.
- Producer-specific buffer-pool lowering remains available only for real pool
  read and writeback producers while those adapters are part of the scheduler
  owner.
- Foreground and background share one permit stream. `PhysicalDispatchSelection`
  gives a ready background head a dispatch after at most three foreground
  dispatches, and an owed turn cannot be stolen by foreground refill.
- Only bounded-interference foreground envelopes are admitted. Hard
  service-time bounds and soft SLO percentiles are refused before media as
  `UnsupportedServiceTimeEnvelope`; reserved queue slots are not device-latency
  guarantees, and no deployment is hardware-qualified.
- Background pressure classes for compaction, replication, ingest, migration,
  backup, repair and verification are vocabulary only. The Store has no
  producer for them yet, so they create no work.

## Related Docs

- [Store physical runtime](../worth-store/README.md)
- [Buffer-pool authority](../worth-store-buffer-pool/README.md)
- [Physical backend](../worth-store-physical-backend/README.md)
- [Store security](../worth-store-security/README.md)
