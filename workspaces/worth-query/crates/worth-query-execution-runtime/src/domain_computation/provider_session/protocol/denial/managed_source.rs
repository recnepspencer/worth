//! Preserve the exact postcommit Source refusal while pending custody survives.
use super::*;
use worth_relational::facade::{
    mvcc::CompanionPreflightStop,
    runtime::{
        CustomInvariantInterruption, DerivedRelationReadDenial as Read,
        ManagedIndexSourcePreparationDenial as Cause, StructuralWalkDenialKind as Structural,
    },
};
impl WorthQueryProviderSessionFailure {
    pub fn managed_source_publication(cause: Cause) -> Self {
        let checked = match &cause {
            Cause::Source(Read::Request(cause))
            | Cause::Publication(CompanionPreflightStop::Execution(cause)) => Some(*cause),
            Cause::Source(Read::Structural(Structural::Interrupted(cause))) => Some(match cause {
                CustomInvariantInterruption::Cancelled => worth_relational::facade::transactions::RelationalExecutionDenialCause::Cancelled,
                CustomInvariantInterruption::DeadlineElapsed => worth_relational::facade::transactions::RelationalExecutionDenialCause::DeadlineElapsed,
            }),
            Cause::Source(Read::Structural(Structural::MemoryAdmission(cause))) => Some(
                worth_execution::WorkCeilingDenial::Admission(worth_execution::LeaseDenial::MemoryExhausted(*cause)).into()),
            _ => None,
        };
        let mut failure = match checked {
            Some(cause) => Self::checked_read_stopped(
                worth_query_execution_contracts::facade::relational_execution_kind(cause, None),
                WorthQueryProviderSessionProtocolStage::Commit,
                "published managed Source correspondence remains in custody",
            ),
            None => Self::new(
                WorthQueryProviderSessionDenialKind::ProviderRejected,
                WorthQueryProviderSessionProtocolStage::Commit,
                "published managed Source correspondence remains in custody",
                WorthQueryProviderSessionProtocolCounters::default(),
            ),
        };
        failure.managed_source_publication = Some(std::sync::Arc::new(cause));
        failure.with_recovery_posture(WorthQueryProviderSessionRecoveryPosture::RecoveryRequired)
    }
    pub fn managed_source_publication_denial(&self) -> Option<&Cause> {
        self.managed_source_publication.as_deref()
    }
}
