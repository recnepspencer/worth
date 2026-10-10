use super::{WorthQueryWorkspaceError, WorthQueryWorkspaceErrorKind};

pub(super) fn basis(
    denial: worth_relational::facade::branch::RelationalBranchBasisDenial,
) -> WorthQueryWorkspaceError {
    let kind = match denial {
        worth_relational::facade::branch::RelationalBranchBasisDenial::RetentionCapacityExhausted => {
            WorthQueryWorkspaceErrorKind::RetentionCapacityExhausted
        }
        worth_relational::facade::branch::RelationalBranchBasisDenial::RetentionIdentityExhausted => {
            WorthQueryWorkspaceErrorKind::RetentionIdentityExhausted
        }
        worth_relational::facade::branch::RelationalBranchBasisDenial::SnapshotIdentityExhausted => {
            WorthQueryWorkspaceErrorKind::SnapshotIdentityExhausted
        }
        _ => WorthQueryWorkspaceErrorKind::RelationalBasisUnavailable,
    };
    WorthQueryWorkspaceError::with_kind(kind, format!("workspace basis denied: {denial:?}"))
}

pub(super) fn admission(
    denial: worth_relational::facade::mvcc::RelationalBranchTransactionAdmissionDenial,
) -> WorthQueryWorkspaceError {
    use worth_relational::facade::mvcc::RelationalBranchTransactionAdmissionDenial as Denial;
    let kind = match denial {
        Denial::RetentionCapacityExhausted => {
            WorthQueryWorkspaceErrorKind::RetentionCapacityExhausted
        }
        Denial::RetentionIdentityExhausted => {
            WorthQueryWorkspaceErrorKind::RetentionIdentityExhausted
        }
        Denial::Cancelled => {
            control_kind(worth_relational::facade::mvcc::RelationalOperationInterruption::Cancelled)
        }
        Denial::TimedOut => {
            control_kind(worth_relational::facade::mvcc::RelationalOperationInterruption::TimedOut)
        }
        _ => WorthQueryWorkspaceErrorKind::RelationalBasisUnavailable,
    };
    WorthQueryWorkspaceError::with_kind(
        kind,
        format!("workspace transaction admission denied: {denial:?}"),
    )
}

pub(super) fn staging(
    denial: worth_relational::facade::mvcc::RelationalTransactionStagingDenial,
) -> WorthQueryWorkspaceError {
    use worth_relational::facade::mvcc::RelationalTransactionStagingDenial as Denial;
    let kind = match denial {
        Denial::AllocationDenied(ref denial) => allocation_kind(denial),
        Denial::CardinalityOverflow => {
            WorthQueryWorkspaceErrorKind::TransactionStagingCardinalityOverflow
        }
        Denial::InputDirectoryAllocationDenied { requested_batches } => {
            WorthQueryWorkspaceErrorKind::TransactionInputDirectoryAllocationDenied {
                requested_batches,
            }
        }
        Denial::SavepointCapacityExhausted { maximum_savepoints } => {
            WorthQueryWorkspaceErrorKind::SavepointCapacityExhausted { maximum_savepoints }
        }
        Denial::SavepointIdentityExhausted => {
            WorthQueryWorkspaceErrorKind::SavepointIdentityExhausted
        }
        Denial::MaterializationAuthorityRequired => {
            WorthQueryWorkspaceErrorKind::TransactionMaterializationAuthorityRequired
        }
        Denial::MaterializationModeMismatch => {
            WorthQueryWorkspaceErrorKind::TransactionMaterializationModeMismatch
        }
    };
    WorthQueryWorkspaceError::with_kind(kind, format!("workspace staging denied: {denial:?}"))
}

pub(super) fn commit(
    error: worth_relational::facade::transactions::TransactionCommitError,
) -> WorthQueryWorkspaceError {
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
                    worth_query_execution::facade::primary_graph::relational_execution_kind(
                        cause,
                        denial.partition_identity,
                    )
                }
            };
            match translated {
                Ok(kind) => WorthQueryWorkspaceErrorKind::ExecutionDenied(kind),
                Err(kind) => WorthQueryWorkspaceErrorKind::ExecutionControlStopped(kind),
            }
        }
        Error::PublicationDeferred { deferred, .. } => match deferred {
            Deferred::PatchPositionReservationContended => {
                WorthQueryWorkspaceErrorKind::PatchPositionReservationContended
            }
            Deferred::RetentionBackpressure => {
                WorthQueryWorkspaceErrorKind::RetentionCapacityExhausted
            }
            Deferred::CandidateCapacityExhausted { maximum_candidates } => {
                WorthQueryWorkspaceErrorKind::CandidateCapacityExhausted {
                    maximum_candidates: *maximum_candidates,
                }
            }
            Deferred::PublishedSnapshotCapacityExhausted { maximum_handles } => {
                WorthQueryWorkspaceErrorKind::PublishedSnapshotCapacityExhausted {
                    maximum_handles: *maximum_handles,
                }
            }
            Deferred::CandidateLifetimeExpired { .. } => WorthQueryWorkspaceErrorKind::Unclassified,
            Deferred::CompanionRegistrationPending | Deferred::CompanionRebindRequired => {
                WorthQueryWorkspaceErrorKind::InvalidationCompanionPending
            }
            Deferred::CompanionPreflight(stop)
                if crate::effect_lifecycle::companion_stop_is_transient(stop) =>
            {
                WorthQueryWorkspaceErrorKind::InvalidationCompanionPending
            }
            Deferred::CompanionPreflight(_) => {
                WorthQueryWorkspaceErrorKind::InvalidationCompanionCapacityExhausted
            }
        },
        Error::PublicationFailed { failure, .. } => match failure.kind() {
            Failure::SnapshotIdentityExhausted => {
                WorthQueryWorkspaceErrorKind::SnapshotIdentityExhausted
            }
            Failure::CandidateIdentityExhausted => {
                WorthQueryWorkspaceErrorKind::CandidateIdentityExhausted
            }
            Failure::RetentionIdentityExhausted => {
                WorthQueryWorkspaceErrorKind::RetentionIdentityExhausted
            }
            _ => WorthQueryWorkspaceErrorKind::Unclassified,
        },
        Error::Preparation { error, .. }
            if error.reason() == CommitPreparationReason::ProposalIdentityOrdinalExhausted =>
        {
            WorthQueryWorkspaceErrorKind::ProposalIdentityExhausted
        }
        Error::Conflict { error, .. } => match &error.class {
            worth_relational::facade::transactions::ConflictClass::ExecutionAllocationDenied { denial } => allocation_kind(denial),
            worth_relational::facade::transactions::ConflictClass::TransactionStagingCardinalityOverflow => WorthQueryWorkspaceErrorKind::TransactionStagingCardinalityOverflow,
            worth_relational::facade::transactions::ConflictClass::TransactionInputDirectoryAllocationDenied { requested_batches } => WorthQueryWorkspaceErrorKind::TransactionInputDirectoryAllocationDenied { requested_batches: *requested_batches },
            _ => WorthQueryWorkspaceErrorKind::Unclassified,
        },
        Error::Interrupted { interruption, .. } => control_kind(interruption.interruption()),
        Error::Preparation { .. }
        | Error::Publication { .. }
        | Error::PublicationDenied { .. }
        | Error::PerformedButDurabilityDeferred { .. } => {
            WorthQueryWorkspaceErrorKind::Unclassified
        }
    };
    WorthQueryWorkspaceError::with_kind(
        kind,
        format!("workspace commit denied: {}", error.detail()),
    )
}

fn allocation_kind(
    denial: &worth_execution::ExecutionAllocationDenial,
) -> WorthQueryWorkspaceErrorKind {
    use worth_execution::ExecutionAllocationDenialKind as Kind;
    match denial.kind() {
        Kind::Cancelled => {
            control_kind(worth_relational::facade::mvcc::RelationalOperationInterruption::Cancelled)
        }
        Kind::DeadlineElapsed => {
            control_kind(worth_relational::facade::mvcc::RelationalOperationInterruption::TimedOut)
        }
        Kind::Layout
        | Kind::Lease(_)
        | Kind::Allocator
        | Kind::CapacityMismatch
        | Kind::WriteBeyondReserved
        | Kind::IncompleteSeal => WorthQueryWorkspaceErrorKind::TransactionAllocationDenied {
            kind: denial.kind(),
            requested_payload_bytes: denial.requested_payload_bytes(),
        },
    }
}
fn control_kind(
    stop: worth_relational::facade::mvcc::RelationalOperationInterruption,
) -> WorthQueryWorkspaceErrorKind {
    use worth_query_execution::facade::primary_graph::WorthQueryProviderSessionControlStopKind as Kind;
    WorthQueryWorkspaceErrorKind::ExecutionControlStopped(match stop {
        worth_relational::facade::mvcc::RelationalOperationInterruption::Cancelled => {
            Kind::Cancelled
        }
        worth_relational::facade::mvcc::RelationalOperationInterruption::TimedOut => Kind::TimedOut,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn leased_execution_is_typed_at_the_workspace_boundary() {
        use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationResourceDenial as Resource;
        use worth_query_execution::facade::primary_graph::WorthQueryProviderSessionDenialKind as Kind;
        let observed = commit(crate::relational_execution_refusal::work_exhausted());
        assert_eq!(
            observed.kind(),
            WorthQueryWorkspaceErrorKind::ExecutionDenied(Kind::ExecutionResource {
                denial: Resource::WorkExhausted,
                partition_identity: Some(1),
                policy_ancestor: None,
            })
        );
    }
}
