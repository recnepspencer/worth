# Shared Home maintenance owner correction

## Boundary finding

The failed selected_checkpoint_survives_observer_drop_online_history_and_mutation_before_ack_recovery
closes a real coupled Store Home immediately after a successful explicit checkpoint.
The only registration caller for CloseCheckpointMaintenance is Query Host
maintenance::prepare (closed construction caller list). Coupled Store Homes do
not visit it. Close now deterministically obtains Maintenance::Unavailable
because the generic Store close call requires the optional Query registration.
This is an implementation design flaw: the scheduled operation is Home-owned,
but its funding worker was made contingent on one composition.

Searches: checkpoint_operation/close_capture under Store application src; exact
registration factory inventory; full maintenance Resources, operation, owner,
world close capture and coupled Home court reads. Closest working analogue is
the existing Query worker, already independently reviewed and qualified.

## Authority and owner

Move the existing independently funded joined worker into Store checkpoint/,
under the SAME HomeCheckpointOwner. It retains no operation, Home, Native or
World port: only its resource ledger, cancellation source, clock and active
counter. The operation borrows it only for one joined call. No cycle, lifecycle
door, source authority, root/basis or callback registry is added.

Rust sketch (all concrete worker/registry types private to Store):
```rust
struct CheckpointMaintenance { worker: Mutex<Option<Arc<MaintenanceWorker>>> }
struct MaintenanceWorker { root: ExecutionResourceLease<'static>,
 cancellation: CancellationSource, timeout: Duration, active: AtomicBool,
 _worker_backing: ExecutionMemoryReservation }
impl ApplicationHomeCheckpointOperation {
 pub fn install_home_checkpoint_maintenance(&self) -> Result<(), Denial>;
 pub fn checkpoint_maintenance_is_active(&self) -> bool;
 pub fn automatic_checkpoint_maintenance(&self) -> Result<Option<Summary>, Denial>;
 pub(crate) fn checkpoint_for_close(&self) -> Result<Summary, Denial>;
}
```
The existing exact installation factory/caller entry remains, with no supplier
argument. Query Host calls it before service. It admits the Root allowance from
the operation's Home limits; no open-request field or constructor is changed.
Direct Store close lazily admits the default finite Home allowance if no Host
installation exists. Once installed, close uses the same actual worker/limits;
it never replaces a custom Host allowance with default limits. Explicit
checkpoint/recovery methods do not initialize or draw on maintenance funding.
Duplicate Host installation retains typed Busy refusal. Internal lazy lookup
reuses the first worker, without creating a second root.

Query Host's adapter retains only its actual operation/retired operation and
existing funded adapter allocation. After advancement it invokes the operation's
maintenance method; retirement reads that same owner's active counter.
Remove the public supplier trait, its export, the Query resources owner and the
Query panic-boundary module after moving their sole responsibilities. Store
maintenance contains setup/join/payload panics with the same typed refusal,
leaving business values and D11 tail retention unchanged.

## Layout/funding

Reuse the reviewed allowance and 2MiB joined worker, plus 64KiB stack rounding
and actual closure/join-handle/Arc-control layout quote. Reserve before Arc or
worker creation from the maintenance root. Keep backing last. Registry allocation
is within the existing actual Home owner; the worker Arc/control is billed by
its own root. No caller request, new target, Cargo feature, profile or allowance
change. No Native, World close progression, Home open request or lifecycle
field change.

## Files / cutover

Store checkpoint/{maintenance.rs,maintenance/worker.rs,maintenance/panic_boundary.rs,
operation.rs,owner.rs,mod.rs}, facade.rs (remove supplier export only).
Query Host application_home/maintenance.rs, delete maintenance/resources.rs and
maintenance/panic_boundary.rs. Existing exact boundary factory entry and caller
remain; remove no safeguards and add no wildcard. No other owner file.

## Proof

Compile coherent Store/Query batch first. Rerun only the failed Store close court
and affected Host funding/close controls, then the three general reopen cases
whose exact two-worker reservations were corrected. Assertions stay unchanged.
Use existing built targets and owned selection; one build at a time and25GB floor.
Cold changed-region P1 remains blocked on its distinct storage-owner handoff;
this correction is not reported as complete funding.
