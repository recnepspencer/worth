//! Moved whole from `domain_computation::primary_graph::invariant_projection::consumed_output::verification::work_budget`: the items below and every impl they own.
use crate::domain_computation::vocabulary::primary_graph::capacity_stop::VerificationCapacityStop;
use crate::domain_computation::primary_graph::vocabulary::invariant_projection::verification::ConsumedOutputVerificationStop;
use worth_relational::facade::mvcc::CompanionCellEditStop;
use worth_relational::facade::mvcc::CompanionPreflightStop;

pub(in crate::domain_computation::primary_graph) fn map_admission_stop(
    stop: CompanionPreflightStop,
) -> ConsumedOutputVerificationStop {
    match stop {
        CompanionPreflightStop::Execution(cause) => {
            ConsumedOutputVerificationStop::Execution(cause)
        }
        CompanionPreflightStop::WorkCounterOverflow => {
            ConsumedOutputVerificationStop::WorkExhausted
        }
        // Readers wait for the branch cells, but a carried writer meter can
        // still meet a concurrent preflight; the caller retries rather than
        // treating momentary contention as missing evidence.
        CompanionPreflightStop::TopologyPending => {
            ConsumedOutputVerificationStop::RetryCurrentness(
                worth_relational::facade::mvcc::CompanionCellEditStop::PreflightPending,
            )
        }
        CompanionPreflightStop::SelectedSourceMismatch
        | CompanionPreflightStop::SelectedPositionUnavailable { .. }
        | CompanionPreflightStop::ForeignCell
        | CompanionPreflightStop::ManagedSourceNotPrepared
        | CompanionPreflightStop::ManagedSourceChanged
        | CompanionPreflightStop::RegistrationChanged => {
            ConsumedOutputVerificationStop::Unavailable
        }
        stop @ CompanionPreflightStop::PreparationMemoryCounterOverflow => {
            ConsumedOutputVerificationStop::CapacityExhausted(VerificationCapacityStop::Meter(stop))
        }
        CompanionPreflightStop::Interrupted(event) => {
            ConsumedOutputVerificationStop::Interrupted(event)
        }
    }
}

pub(in crate::domain_computation::primary_graph) fn reserve_pending<T>(
    pending: &mut Vec<T>,
    additional: usize,
) -> Result<(), ConsumedOutputVerificationStop> {
    let unrepresentable = ConsumedOutputVerificationStop::CapacityExhausted(
        VerificationCapacityStop::ScratchSizeOverflow,
    );
    let needed = pending
        .len()
        .checked_add(additional)
        .ok_or_else(|| unrepresentable.clone())?;
    if needed <= pending.capacity() {
        return Ok(());
    }
    // Doubling keeps a worklist's admitted growth within twice its final
    // backing. Growing to exactly `needed` would admit every intermediate
    // length again: quadratic in a walk that pushes one row at a time.
    let target = needed.max(pending.capacity().saturating_mul(2));
    let bytes = target
        .checked_mul(size_of::<T>())
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or_else(|| unrepresentable.clone())?;

    pending
        .try_reserve_exact(target - pending.len())
        .map_err(|_| {
            ConsumedOutputVerificationStop::CapacityExhausted(
                VerificationCapacityStop::ScratchAllocationRefused { bytes },
            )
        })
}
