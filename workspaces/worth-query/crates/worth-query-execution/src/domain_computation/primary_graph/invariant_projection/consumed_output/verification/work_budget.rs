//! Finite Work and scratch admission for consumed-output verification.

use super::*;

pub(in crate::domain_computation::primary_graph) fn map_admission_stop(
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
        | CompanionPreflightStop::RetainedCompanionCounterOverflow { .. }
        | CompanionPreflightStop::RetainedCompanionCapacityExhausted { .. }) => {
            ConsumedOutputVerificationStop::CapacityExhausted(VerificationCapacityStop::Meter(stop))
        }
        CompanionPreflightStop::Interrupted(event) => {
            ConsumedOutputVerificationStop::Interrupted(event)
        }
    }
}

pub(in crate::domain_computation::primary_graph) fn map_verification_stop(
    stop: SettlementVerificationStop,
) -> ConsumedOutputVerificationStop {
    match stop {
        SettlementVerificationStop::SourceRead(
            WorthQuerySourceCurrentnessFailure::OutputObservation(stop),
        ) => stop,
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
        .ok_or(unrepresentable)?;
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
        .ok_or(unrepresentable)?;

    pending
        .try_reserve_exact(target - pending.len())
        .map_err(|_| {
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
) -> Result<bool, ConsumedOutputVerificationStop> {
    match fact
        .source_currentness_in(runtime, execution, snapshot)
        .map_err(map_admission_stop)?
    {
        Err(WorthQuerySourceCurrentnessFailure::OutputObservation(stop)) => Err(stop),
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
    use worth_relational::facade::mvcc::RelationalCancellationSource;
    use worth_relational::facade::mvcc::RelationalInterruptionBoundary;
    use worth_relational::facade::mvcc::RelationalOperationControl;

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
            CompanionPreflightStop::RetainedCompanionCounterOverflow {
                requested: 1,
                retained: u64::MAX,
            },
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

    #[test]
    fn worklist_growth_preserves_rows_and_refuses_unrepresentable_sizes() {
        let mut pending = Vec::<u64>::new();
        for row in 0..1_000 {
            reserve_pending(&mut pending, 1).unwrap();
            pending.push(row);
        }
        assert_eq!(pending, (0..1_000).collect::<Vec<_>>());
        assert_eq!(
            reserve_pending(&mut pending, usize::MAX),
            Err(ConsumedOutputVerificationStop::CapacityExhausted(
                VerificationCapacityStop::ScratchSizeOverflow
            ))
        );
        assert_eq!(pending.len(), 1_000);
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
