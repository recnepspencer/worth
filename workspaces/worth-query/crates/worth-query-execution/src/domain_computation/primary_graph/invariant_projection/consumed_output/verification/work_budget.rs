//! Finite Work and scratch admission for consumed-output verification.

use super::*;

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
        | CompanionPreflightStop::CellCapacityExhausted { .. }
        | CompanionPreflightStop::PreparationMemoryExhausted { .. }
        | CompanionPreflightStop::PreparationMemoryCounterOverflow
        | CompanionPreflightStop::RetainedCompanionCapacityExhausted { .. } => {
            ConsumedOutputVerificationStop::Unavailable
        }
        CompanionPreflightStop::Interrupted(event) => {
            ConsumedOutputVerificationStop::Interrupted(event)
        }
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
    match fact
        .source_currentness_in(runtime, snapshot, admission)
        .map_err(map_admission_stop)?
    {
        Ok(movement) => Ok(movement.movement() == Movement::Unmoved),
        Err(WorthQuerySourceCurrentnessFailure::WorkBudgetExceeded) => {
            Err(ConsumedOutputVerificationStop::WorkExhausted)
        }
        Err(WorthQuerySourceCurrentnessFailure::Unavailable) => {
            Err(ConsumedOutputVerificationStop::Unavailable)
        }
    }
}

#[cfg(test)]
mod tests {
    use worth_relational::facade::mvcc::{
        RelationalCancellationSource, RelationalInterruptionBoundary, RelationalOperationControl,
    };

    use super::*;

    /// An interrupted verification is the request's interruption. Read as
    /// `Unavailable`, it would send a caller to disclose sources or to
    /// recompute for a request that no longer wants an answer.
    #[test]
    fn an_interrupted_verification_is_the_interruption_not_unavailable() {
        let source = RelationalCancellationSource::new();
        source.cancel();
        let event = RelationalOperationControl::from(source.token())
            .observe(RelationalInterruptionBoundary::PublicationPreflight)
            .expect("a cancelled control is interrupted");
        let interrupted = CompanionPreflightStop::Interrupted(event);
        assert_eq!(
            map_admission_stop(interrupted),
            ConsumedOutputVerificationStop::Interrupted(event)
        );
        assert_eq!(
            map_verification_stop(SettlementVerificationStop::Admission(interrupted)),
            ConsumedOutputVerificationStop::Interrupted(event)
        );
    }
}
