use std::sync::atomic::Ordering;

use crate::branch::{OwnerCreatedComponentCustodyRecord, OwnerRetirementWork};
use crate::lifecycle::owner::{RuntimeWorldBootstrapState, RuntimeWorldOwnerState};
use crate::recovery::{ProductUnpublishedOwnerEffects, ProductUnpublishedRecoveryHandle};

use super::report::{
    RuntimeWorldCloseReleaseCounts, RuntimeWorldCloseReport, RuntimeWorldRetainedRecordReport,
};
use super::RuntimeWorldCloseDenial;

/// Admit close under bootstrap -> operation -> close lock order. Publishing
/// Closing excludes every new admission before the guards are released.
/// Component custody drains outside lifecycle locks.
pub(super) fn close_owner<D, I, E, Ctx, T>(
    state: &RuntimeWorldOwnerState<D, I, E, Ctx, T>,
) -> Result<RuntimeWorldCloseReport, RuntimeWorldCloseDenial>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    let bootstrap = state
        .bootstrap
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    if *bootstrap == RuntimeWorldBootstrapState::InProgress {
        return Err(RuntimeWorldCloseDenial::AlreadyClosing);
    }
    // Publish the queued waiter before blocking so the transition from "no
    // close" to "a close is admitting" is observable rather than inferred from
    // elapsed time.
    state.close_admission_waiters.fetch_add(1, Ordering::SeqCst);
    let operation = state
        .operation
        .state
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    state.close_admission_waiters.fetch_sub(1, Ordering::SeqCst);
    admit_close(
        operation.recovery_active,
        operation.active,
        state.recovery.reserved_slots(),
    )?;
    let retained_records = enumerate_retained_records(state)?;
    let mut close = state
        .close
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    close.begin()?;
    drop(close);
    drop(operation);
    drop(bootstrap);
    let report = drain_for_close(state, retained_records);
    state
        .close
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .finish()?;
    Ok(report)
}

/// Decide whether this owner can be drained at all, from the operation ledger
/// the caller is holding. A declared critical section that is still in flight
/// cannot be drained, so it denies here rather than closing over live work.
///
/// An installed retained record is deliberately not a denial: close settles
/// what it can and exposes the rest in its terminal report.
fn admit_close(
    recovery_active: usize,
    active: usize,
    reserved_recovery_slots: usize,
) -> Result<(), RuntimeWorldCloseDenial> {
    if recovery_active != 0 {
        return Err(RuntimeWorldCloseDenial::InFlightCriticalSection);
    }
    if active != 0 || reserved_recovery_slots != 0 {
        return Err(RuntimeWorldCloseDenial::AlreadyClosing);
    }
    Ok(())
}

/// Drain custody after Closing has excluded new admission. Retained records
/// were described during admission; this phase exposes them and releases
/// unneeded pins without settling or reclassifying their owner effects.
fn drain_for_close<D, I, E, Ctx, T>(
    state: &RuntimeWorldOwnerState<D, I, E, Ctx, T>,
    retained_records: Vec<RuntimeWorldRetainedRecordReport>,
) -> RuntimeWorldCloseReport
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    // Order matters: releasing the branch references first is what makes the
    // custody drain total. A record still installed after that belongs to an
    // occurrence no product reference names any more, which is exactly the
    // work close must hand back instead of dropping.
    let released_product_head_pins = state.branches.release_non_root_branches();
    let outstanding_owner_retirement_work: Vec<OwnerRetirementWork> = state
        .custody
        .take_all_installed()
        .into_iter()
        .map(OwnerCreatedComponentCustodyRecord::into_retirement_work)
        .collect();
    let counts = RuntimeWorldCloseReleaseCounts {
        // Enumerating a retained record is exposure, never settlement. The
        // drain settles no record today: every record it can read survives the
        // close that named it, so the settled count is the zero it settled.
        settled_records: 0,
        // One per non-root product branch whose reference the registry held and
        // close released. The root is the world's own reference, not a created
        // branch, so it is not counted here. Observation pins are held by
        // caller-owned observations that outlive this call and are not
        // released by close. Their live count is reported separately.
        released_product_head_pins,
        released_observation_pins: 0,
        // Every installed commit is still protected, either by a product head
        // or by a retained record enumerated above, so close releases no
        // history protection.
        released_history_pins: 0,
        released_unique_component_pins: release_reclaimable_component_pins(state),
        // Close retires the custody charge itself: the registry no longer
        // holds these records. What it cannot do is delete a component branch,
        // so the same records leave as typed work the caller must dispatch.
        retired_owner_created_custody: outstanding_owner_retirement_work.len(),
    };
    RuntimeWorldCloseReport::new(
        retained_records,
        counts,
        state.retention.active_observation_count(),
        outstanding_owner_retirement_work,
    )
}

/// Name every retained record the catalog still holds. A record that is inside
/// its own update critical section cannot be read without racing that update,
/// so it denies rather than producing a row that may already be stale.
fn enumerate_retained_records<D, I, E, Ctx, T>(
    state: &RuntimeWorldOwnerState<D, I, E, Ctx, T>,
) -> Result<Vec<RuntimeWorldRetainedRecordReport>, RuntimeWorldCloseDenial>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    let affinity = state.recovery.affinity();
    let identities = state.recovery.identities();
    let mut rows = Vec::with_capacity(identities.len());
    for identity in identities {
        let handle = ProductUnpublishedRecoveryHandle::new(identity, affinity);
        let Some(record) = state.recovery.lookup_record(&handle) else {
            return Err(RuntimeWorldCloseDenial::InFlightCriticalSection);
        };
        rows.push(describe(
            &ProductUnpublishedOwnerEffects::from_catalog_record(record),
        ));
    }
    Ok(rows)
}

/// One report row per record, carrying the record's own live obligations. The
/// split is the record's, read as it stands: the report never re-derives a
/// count it could contradict.
fn describe(effects: &ProductUnpublishedOwnerEffects) -> RuntimeWorldRetainedRecordReport {
    RuntimeWorldRetainedRecordReport::new(
        effects.identity().clone(),
        effects.cause(),
        effects.live_obligations(),
        effects.next_actions().to_vec(),
    )
}

/// Release every exact component pin that no live dependency and no component
/// owner lease still holds. The count is what the registry actually reclaimed.
fn release_reclaimable_component_pins<D, I, E, Ctx, T>(
    state: &RuntimeWorldOwnerState<D, I, E, Ctx, T>,
) -> usize
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    let live = state.retention.unique_pin_count();
    state.retention.reclaim(live).reclaimed()
}
