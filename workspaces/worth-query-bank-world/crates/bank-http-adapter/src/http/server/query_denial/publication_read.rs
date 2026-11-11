//! Read-only publication roster and predecessor-index failures, not a commit attempt.
use super::*;
use crate::http::server::relational_execution_denial;
use worth_query_host::facade::primary_graph::{
    CommitExecutionDenialKind, CommitPreparationReason, ConflictClass,
    RelationalOperationInterruption, TransactionCommitError,
};

pub(super) fn denial(cause: TransactionCommitError) -> BankHttpDenial {
    use TransactionCommitError as D;
    match cause {
        D::Execution { denial, .. } => match denial.kind {
            CommitExecutionDenialKind::Cause(cause) => relational_execution_denial::denial(cause),
        },
        D::Interrupted { interruption, .. } => match interruption.interruption() {
            RelationalOperationInterruption::Cancelled => cancelled(),
            RelationalOperationInterruption::TimedOut => deadline(),
        },
        D::Preparation { error, .. } => {
            if let Some(cause) = error.managed_source_denial() {
                return managed_index::native_source::source(cause);
            }
            match error.reason() {
                CommitPreparationReason::StorageUnavailable(_) => owner_unavailable(),
                CommitPreparationReason::SelectedBranchRoot(_) => stale(),
                CommitPreparationReason::SymbolIdentityExhausted
                | CommitPreparationReason::ProposalIdentityOrdinalExhausted
                | CommitPreparationReason::ManagedIndexSource => operator(),
            }
        }
        D::Conflict { error, .. } => conflict(error.class),
        // These readonly producers do not prepare candidates, publish, or write a medium.
        // Their Publication error describes damaged/missing ownership roster metadata.
        // A write-protocol outcome at this read handoff is an owner contract fault.
        D::Publication { .. }
        | D::PublicationDenied { .. }
        | D::PublicationDeferred { .. }
        | D::PublicationFailed { .. }
        | D::DurabilityUnresolved { .. } => operator(),
    }
}

fn conflict(cause: ConflictClass) -> BankHttpDenial {
    use ConflictClass as D;
    match cause {
        D::ContentAdmission { denial } => relational_execution_denial::content(denial),
        D::ExecutionAllocationDenied { denial } => managed_index::allocation(&denial),
        D::StaleTarget { .. } | D::StaleValidationBasis { .. } => stale(),
        D::MaterializationAuthorityRequired | D::MaterializationModeMismatch => owner_unavailable(),
        D::TransactionInputDirectoryAllocationDenied { .. } => unavailable(),
        D::TransactionSavepointBudgetExceeded { .. } => exhausted(),
        D::SymbolIdentityExhausted
        | D::TransactionStagingCardinalityOverflow
        | D::InvalidRelationEndpoint { .. }
        | D::DuplicateRelationIdentity { .. }
        | D::InvariantViolation { .. }
        | D::KindSchemaMismatch { .. }
        | D::MutationStateInconsistency { .. }
        | D::RecordAllocationDenied { .. }
        | D::EntityUpdateStateInconsistency { .. }
        | D::EntityFieldUpdateStateInconsistency { .. }
        | D::RelationEndpointUpdateStateInconsistency { .. }
        | D::RelationEndpointUpdateKindMismatch { .. }
        | D::BulkImportDomainMismatch { .. }
        | D::EntityAuthoritativeAspectStateDenied { .. }
        | D::RecordAspectPatchDenied { .. }
        | D::AspectDeltaFailure { .. }
        | D::ConflictingIntent { .. }
        | D::InvalidSavepoint { .. }
        | D::InvalidMergeParent { .. }
        | D::ForeignRuntime { .. }
        | D::TransactionSavepointIdentityExhausted
        | D::MergeConflictOverlap { .. }
        | D::MissingMergeBase { .. }
        | D::UndeclaredSchemaTransition { .. }
        | D::DescriptorSemanticsVersionUnsupported { .. }
        | D::InvalidSchemaTransitionSourceBasis { .. }
        | D::InvalidSchemaTransitionTargetBasis { .. }
        | D::MissingSchemaBasisForTransition { .. }
        | D::UnsupportedBridgeDescriptor { .. }
        | D::HistoricalReinterpretationViolation { .. }
        | D::TypeContinuityDeniedSchemaTransition { .. }
        | D::StructuralContinuityDeniedSchemaTransition { .. }
        | D::DirectionalityMismatchUnderCanonicalReconciliation { .. }
        | D::InvalidSchemaTransitionShape { .. } => operator(),
    }
}
fn owner_unavailable() -> BankHttpDenial {
    BankHttpDenial::new(Kind::Unavailable, Next::ContactOperator)
}
fn operator() -> BankHttpDenial {
    BankHttpDenial::new(Kind::InternalDenied, Next::ContactOperator)
}

#[cfg(test)]
mod tests;
