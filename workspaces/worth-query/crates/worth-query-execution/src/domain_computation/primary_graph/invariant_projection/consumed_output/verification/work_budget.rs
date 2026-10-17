//! Finite Work and scratch admission for consumed-output verification.

use super::*;

pub(in crate::domain_computation::primary_graph::invariant_projection) fn map_admission_stop(
    stop: CompanionPreflightStop,
) -> ConsumedOutputVerificationStop {
    match stop {
        CompanionPreflightStop::Execution(cause) => {
            ConsumedOutputVerificationStop::Execution(cause)
        }
        CompanionPreflightStop::WorkExhausted { .. }
        | CompanionPreflightStop::WorkExhaustedWithoutRequirement { .. }
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
        | CompanionPreflightStop::RegistrationChanged => {
            ConsumedOutputVerificationStop::Unavailable
        }
        stop @ (CompanionPreflightStop::PreparationMemoryExhausted { .. }
        | CompanionPreflightStop::PreparationMemoryCounterOverflow
        | CompanionPreflightStop::RetainedCompanionCapacityExhausted { .. }) => {
            ConsumedOutputVerificationStop::CapacityExhausted(VerificationCapacityStop::Meter(stop))
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
        SettlementVerificationStop::SourceRead(WorthQuerySourceCurrentnessFailure::Execution(
            cause,
        )) => ConsumedOutputVerificationStop::Execution(cause),
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
    let unrepresentable = ConsumedOutputVerificationStop::CapacityExhausted(
        VerificationCapacityStop::ScratchSizeOverflow,
    );
    let needed = pending
        .len()
        .checked_add(additional)
        .ok_or(unrepresentable)?;
    if needed <= pending.capacity() {
        return Ok(());
    }
    let bytes = needed
        .checked_mul(size_of::<T>())
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or(unrepresentable)?;
    admission
        .admit_read_scratch(bytes)
        .map_err(map_admission_stop)?;
    charge_external(admission, pending.len())?;
    pending.try_reserve_exact(additional).map_err(|_| {
        ConsumedOutputVerificationStop::CapacityExhausted(
            VerificationCapacityStop::ScratchAllocationRefused { bytes },
        )
    })
}

pub(super) fn fact_is_current(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    fact: &WorthQueryApplicationObservedFact,
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    admission: &mut InvalidationEditAdmission,
) -> Result<bool, ConsumedOutputVerificationStop> {
    match fact
        .source_currentness_in(runtime, execution, snapshot, admission)
        .map_err(map_admission_stop)?
    {
        Err(WorthQuerySourceCurrentnessFailure::Execution(cause)) => {
            Err(ConsumedOutputVerificationStop::Execution(cause))
        }
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

    /// Each capacity stop keeps the meter's own typed cause, so its required
    /// and maximum amounts survive into the caller's denial.
    #[test]
    fn capacity_stops_are_not_missing_output_evidence() {
        for stop in [
            CompanionPreflightStop::PreparationMemoryExhausted {
                required: 2,
                maximum: 1,
            },
            CompanionPreflightStop::PreparationMemoryCounterOverflow,
            CompanionPreflightStop::RetainedCompanionCapacityExhausted {
                requested: 1,
                retained: 1,
                maximum: 1,
            },
        ] {
            let kept = ConsumedOutputVerificationStop::CapacityExhausted(
                VerificationCapacityStop::Meter(stop),
            );
            assert_eq!(map_admission_stop(stop), kept);
            assert_eq!(
                map_verification_stop(SettlementVerificationStop::Admission(stop)),
                kept
            );
        }
    }

    /// A worklist growth the meter refuses reports the meter's amounts; one
    /// whose size is not representable is not reported as the meter's.
    #[test]
    fn worklist_growth_distinguishes_the_meter_from_an_unrepresentable_size() {
        let mut admission = InvalidationEditAdmission::new(
            worth_relational::facade::mvcc::CompanionPreflightBudget {
                maximum_work_visits: u64::MAX,
                maximum_preparation_bytes: 8,
            },
        );
        let mut pending = Vec::<u64>::new();
        assert_eq!(
            reserve_pending(&mut pending, 2, &mut admission),
            Err(ConsumedOutputVerificationStop::CapacityExhausted(
                VerificationCapacityStop::Meter(
                    CompanionPreflightStop::PreparationMemoryExhausted {
                        required: 16,
                        maximum: 8,
                    }
                )
            ))
        );
        assert_eq!(
            reserve_pending(&mut pending, usize::MAX, &mut admission),
            Err(ConsumedOutputVerificationStop::CapacityExhausted(
                VerificationCapacityStop::ScratchSizeOverflow
            ))
        );
        assert_eq!(
            admission.charged_bytes(),
            0,
            "a refused growth admits nothing"
        );
    }

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
