//! Finite Work and scratch admission for consumed-output verification.

use super::*;

fn claim_work(remaining: &mut usize, units: usize) -> Result<(), ConsumedOutputVerificationStop> {
    *remaining = remaining
        .checked_sub(units)
        .ok_or(ConsumedOutputVerificationStop::WorkExhausted)?;
    Ok(())
}

pub(super) fn debit_wrapper_work(
    admission: &InvalidationEditAdmission,
    remaining: &mut usize,
) -> Result<(), ConsumedOutputVerificationStop> {
    let spent = usize::try_from(admission.charged_work())
        .map_err(|_| ConsumedOutputVerificationStop::WorkExhausted)?;
    claim_work(remaining, spent)
}

pub(in super::super) fn map_admission_stop(
    stop: CompanionPreflightStop,
) -> ConsumedOutputVerificationStop {
    match stop {
        CompanionPreflightStop::WorkExhausted { .. }
        | CompanionPreflightStop::WorkCounterOverflow => {
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
        | CompanionPreflightStop::RegistrationChanged
        | CompanionPreflightStop::PreparationMemoryExhausted { .. }
        | CompanionPreflightStop::PreparationMemoryCounterOverflow
        | CompanionPreflightStop::RetainedCompanionCapacityExhausted { .. }
        | CompanionPreflightStop::Interrupted(_) => ConsumedOutputVerificationStop::Unavailable,
    }
}

pub(super) fn map_verification_stop(
    stop: SettlementVerificationStop,
) -> ConsumedOutputVerificationStop {
    match stop {
        SettlementVerificationStop::Admission(stop) => map_admission_stop(stop),
        SettlementVerificationStop::SourceRead(
            WorthQuerySourceCurrentnessFailure::WorkBudgetExceeded,
        ) => ConsumedOutputVerificationStop::WorkExhausted,
        SettlementVerificationStop::SourceRead(WorthQuerySourceCurrentnessFailure::Unavailable)
        | SettlementVerificationStop::Alignment => ConsumedOutputVerificationStop::Unavailable,
        SettlementVerificationStop::PendingUpstream => {
            ConsumedOutputVerificationStop::PendingUpstream
        }
        SettlementVerificationStop::Edit(stop) => {
            ConsumedOutputVerificationStop::RetryCurrentness(stop)
        }
    }
}

pub(super) fn charge_external(
    admission: &mut InvalidationEditAdmission,
    units: usize,
) -> Result<(), ConsumedOutputVerificationStop> {
    let units = u64::try_from(units).map_err(|_| ConsumedOutputVerificationStop::WorkExhausted)?;
    admission
        .charge_external_work(units)
        .map_err(map_admission_stop)
}

pub(super) fn reserve_pending<T>(
    pending: &mut Vec<T>,
    additional: usize,
    admission: &mut InvalidationEditAdmission,
) -> Result<(), ConsumedOutputVerificationStop> {
    let needed = pending
        .len()
        .checked_add(additional)
        .ok_or(ConsumedOutputVerificationStop::Unavailable)?;
    if needed <= pending.capacity() {
        return Ok(());
    }
    let bytes = needed
        .checked_mul(size_of::<T>())
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or(ConsumedOutputVerificationStop::Unavailable)?;
    admission
        .admit_read_scratch(bytes)
        .map_err(map_admission_stop)?;
    charge_external(admission, pending.len())?;
    pending
        .try_reserve_exact(additional)
        .map_err(|_| ConsumedOutputVerificationStop::Unavailable)
}

pub(super) fn fact_is_current(
    fact: &WorthQueryApplicationObservedFact,
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    admission: &mut InvalidationEditAdmission,
) -> Result<bool, ConsumedOutputVerificationStop> {
    let available = admission.remaining_work();
    if available == 0 {
        return Err(ConsumedOutputVerificationStop::WorkExhausted);
    }
    let prepaid = fact
        .exact_probe_work()
        .map_err(|_| ConsumedOutputVerificationStop::WorkExhausted)?
        .unwrap_or(0);
    charge_external(admission, prepaid)?;
    let (current, work) = fact
        .source_currentness_in(runtime, snapshot, available)
        .map_err(|failure| match failure {
            WorthQuerySourceCurrentnessFailure::WorkBudgetExceeded => {
                ConsumedOutputVerificationStop::WorkExhausted
            }
            WorthQuerySourceCurrentnessFailure::Unavailable => {
                ConsumedOutputVerificationStop::Unavailable
            }
        })?;
    charge_external(admission, work.saturating_sub(prepaid))?;
    Ok(current)
}
