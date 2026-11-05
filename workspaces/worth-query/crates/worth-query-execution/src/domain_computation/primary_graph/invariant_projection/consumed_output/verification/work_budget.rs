//! Finite Work and scratch admission for consumed-output verification.

use super::*;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::movement::Movement;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::source_currentness::WorthQuerySourceCurrentnessFailure;
use crate::domain_computation::primary_graph::vocabulary::invariant_projection::verification::ConsumedOutputVerificationStop;
use crate::domain_computation::primary_graph::vocabulary::invariant_projection::work_budget::map_admission_stop;
use crate::domain_computation::vocabulary::primary_graph::capacity_stop::VerificationCapacityStop;

pub(in crate::domain_computation::primary_graph) fn map_verification_stop(
    stop: SettlementVerificationStop,
) -> ConsumedOutputVerificationStop {
    match stop {
        SettlementVerificationStop::SourceRead(
            WorthQuerySourceCurrentnessFailure::ManagedIndex(cause),
        ) => ConsumedOutputVerificationStop::ManagedIndex(cause),
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
        Err(WorthQuerySourceCurrentnessFailure::ManagedIndex(cause)) => {
            Err(ConsumedOutputVerificationStop::ManagedIndex(cause))
        }
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
    use crate::domain_computation::primary_graph::vocabulary::invariant_projection::work_budget::reserve_pending;
    use worth_relational::facade::mvcc::RelationalCancellationSource;
    use worth_relational::facade::mvcc::RelationalInterruptionBoundary;
    use worth_relational::facade::mvcc::RelationalOperationControl;

    use super::*;

    /// The surviving counter stop keeps its original typed cause through both
    /// verification mappings; byte-forecast stops were retired by ee07da2b0e.
    #[test]
    fn capacity_stops_are_not_missing_output_evidence() {
        let stop = CompanionPreflightStop::PreparationMemoryCounterOverflow;
        let kept = ConsumedOutputVerificationStop::CapacityExhausted(
            VerificationCapacityStop::Meter(stop),
        );
        assert_eq!(map_admission_stop(stop), kept);
        assert_eq!(
            map_verification_stop(SettlementVerificationStop::Admission(stop)),
            kept
        );
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
