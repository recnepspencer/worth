//! Project the existing native consumed-edge verifier into retained-read denials.
use super::*;
use crate::domain_computation::primary_graph::{
    invariant_projection::{
        ConsumedOutputEvidence, ConsumedOutputVerification, ConsumedOutputVerificationStop,
    },
    output_reuse,
};

impl<Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'_, Schema> {
    pub(super) fn require_current_consumed(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        relational: &worth_relational::facade::runtime::RelationalRuntime,
        read: &RetainedOutputCurrentnessRead,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        if read.consumed_outputs.is_empty() {
            return Ok(());
        }
        let snapshot = self.application_basis().snapshot_handle();
        let selected = relational
            .read_truth()
            .positioned_snapshot(snapshot)
            .map_err(|_| denial(WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable))?;
        let owner = &self
            .application()
            .primary_provider
            .graph
            .source_owner
            .invalidation_owner;
        match ConsumedOutputEvidence::verify_many_with_admission(
            execution,
            &read.consumed_outputs,
            owner,
            relational,
            snapshot,
            &selected,
            admission,
        ) {
            Ok(ConsumedOutputVerification::Current) => Ok(()),
            // These ordinals belong to the consumed output, not the consumer.
            Ok(
                ConsumedOutputVerification::ChangedDirectFact(_)
                | ConsumedOutputVerification::ChangedUpstream,
            ) => Err(denial(WorthQueryOutputDemandDenialKind::Superseded)),
            Err(stop) => Err(verification_denial(stop)),
        }
    }
}

fn verification_denial(stop: ConsumedOutputVerificationStop) -> WorthQueryOutputDemandDenial {
    use WorthQueryOutputDemandDenialKind as Kind;
    match stop {
        ConsumedOutputVerificationStop::Execution(cause) => output_reuse::read_denial(cause),
        ConsumedOutputVerificationStop::Interrupted(event) => {
            output_reuse::interruption_denial(event)
        }
        ConsumedOutputVerificationStop::CapacityExhausted(capacity) => {
            output_reuse::memory_denial().with_capacity_stop(Some(capacity))
        }
        ConsumedOutputVerificationStop::WorkExhausted => denial(Kind::WorkBudgetExceeded),
        ConsumedOutputVerificationStop::PendingUpstream => denial(Kind::SchedulingDeferred),
        // A read reports the existing publication-race posture; it neither
        // schedules another producer attempt nor reclassifies it as current.
        ConsumedOutputVerificationStop::RetryCurrentness(_) => denial(Kind::PublicationStale),
        ConsumedOutputVerificationStop::Unavailable => denial(Kind::RetainedBasisUnavailable),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain_computation::primary_graph::WorthQueryOutputDemandRecoveryPosture::{
        Retryable, Terminal,
    };
    use worth_relational::facade::{
        mvcc::CompanionCellEditStop, transactions::RelationalExecutionDenialCause,
    };

    #[test]
    fn consumed_stops_preserve_read_posture_without_producer_origin() {
        use ConsumedOutputVerificationStop as Stop;
        use WorthQueryOutputDemandDenialKind as Kind;
        for (stop, kind, posture) in [
            (Stop::PendingUpstream, Kind::SchedulingDeferred, Retryable),
            (Stop::Unavailable, Kind::RetainedBasisUnavailable, Terminal),
            (Stop::WorkExhausted, Kind::WorkBudgetExceeded, Terminal),
            (
                Stop::Execution(RelationalExecutionDenialCause::Cancelled),
                Kind::Cancelled,
                Terminal,
            ),
            (
                Stop::Execution(RelationalExecutionDenialCause::DeadlineElapsed),
                Kind::TimedOut,
                Terminal,
            ),
            (
                Stop::RetryCurrentness(CompanionCellEditStop::PreflightPending),
                Kind::PublicationStale,
                Terminal,
            ),
        ] {
            let denied = verification_denial(stop);
            assert_eq!(denied.kind(), kind);
            assert_eq!(denied.recovery_posture(), posture);
            assert_eq!(denied.publication_companion_stop(), None);
            assert_eq!(denied.producer_admission_stop(), None);
        }
    }
}
