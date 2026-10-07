use super::{
    installation_execution_kind, primary_graph_denial, WorthQueryPrimaryGraphInstallationDenial,
    WorthQueryPrimaryGraphInstallationDenialKind,
};

pub(in crate::domain_computation::primary_graph) fn map_bootstrap_commit_denial(
    error: worth_relational::facade::transactions::TransactionCommitError,
) -> WorthQueryPrimaryGraphInstallationDenial {
    use worth_relational::facade::mvcc::{
        RelationalPublicationDeferred as Deferred, RelationalPublicationFailureKind as Failure,
    };
    use worth_relational::facade::transactions::{
        CommitPreparationReason, TransactionCommitError as Error,
    };
    let kind = match &error {
        Error::Execution { denial, .. } => {
            let translated = match denial.kind {
                worth_relational::facade::transactions::CommitExecutionDenialKind::Cause(cause) => {
                    super::super::provider::relational_execution_denial::relational_execution_kind(
                        cause,
                        denial.partition_identity,
                    )
                }
            };
            installation_execution_kind(translated)
        }
        Error::PublicationDeferred { deferred, .. } => match deferred {
            Deferred::CompanionRegistrationPending
            | Deferred::CompanionRebindRequired
            | Deferred::CompanionPreflight(_) => {
                WorthQueryPrimaryGraphInstallationDenialKind::RelationalDeferred(*deferred)
            }
            Deferred::PatchPositionReservationContended => {
                WorthQueryPrimaryGraphInstallationDenialKind::PatchPositionReservationContended
            }
            Deferred::RetentionBackpressure => {
                WorthQueryPrimaryGraphInstallationDenialKind::RetentionCapacityExhausted
            }
            Deferred::CandidateCapacityExhausted { maximum_candidates } => {
                WorthQueryPrimaryGraphInstallationDenialKind::CandidateCapacityExhausted {
                    maximum_candidates: *maximum_candidates,
                }
            }
            Deferred::PublishedSnapshotCapacityExhausted { maximum_handles } => {
                WorthQueryPrimaryGraphInstallationDenialKind::PublishedSnapshotCapacityExhausted {
                    maximum_handles: *maximum_handles,
                }
            }
            Deferred::CandidateLifetimeExpired { .. } => {
                WorthQueryPrimaryGraphInstallationDenialKind::RelationalCommitRejected
            }
        },
        Error::PublicationFailed { failure, .. } => match failure.kind() {
            Failure::SnapshotIdentityExhausted => {
                WorthQueryPrimaryGraphInstallationDenialKind::SnapshotIdentityExhausted
            }
            Failure::CandidateIdentityExhausted => {
                WorthQueryPrimaryGraphInstallationDenialKind::CandidateIdentityExhausted
            }
            Failure::RetentionIdentityExhausted => {
                WorthQueryPrimaryGraphInstallationDenialKind::RetentionIdentityExhausted
            }
            Failure::PreparedRootBudgetExhausted {
                maximum_bytes,
                required_bytes,
            } => WorthQueryPrimaryGraphInstallationDenialKind::PreparedRootBudgetExhausted {
                maximum_bytes: *maximum_bytes,
                required_bytes: *required_bytes,
            },
            Failure::PreparedRootMismatch
            | Failure::PreparedBasisDescriptor(_)
            | Failure::NextBasisAdmission(_)
            | Failure::SelectedRootUnavailable
            | Failure::BranchObservation(_)
            | Failure::PatchPositionCapacityExhausted
            | Failure::RetentionOwner
            | Failure::PendingSettlementIdentityConflict => {
                WorthQueryPrimaryGraphInstallationDenialKind::RelationalCommitRejected
            }
        },
        Error::Preparation { error, .. }
            if error.reason() == CommitPreparationReason::ProposalIdentityOrdinalExhausted =>
        {
            WorthQueryPrimaryGraphInstallationDenialKind::ProposalIdentityExhausted
        }
        // HEAD treated operation interruption as a generic commit rejection.
        // It is not an execution-owner refusal, matching the workspace route.
        Error::Interrupted { .. } => {
            WorthQueryPrimaryGraphInstallationDenialKind::RelationalCommitRejected
        }
        Error::Preparation { .. }
        | Error::Conflict { .. }
        | Error::Publication { .. }
        | Error::PublicationDenied { .. }
        | Error::PerformedButDurabilityDeferred { .. } => {
            WorthQueryPrimaryGraphInstallationDenialKind::RelationalCommitRejected
        }
    };
    primary_graph_denial(
        kind,
        format!(
            "Relational rejected the application principal bootstrap transaction: {}",
            error.detail()
        ),
    )
}
