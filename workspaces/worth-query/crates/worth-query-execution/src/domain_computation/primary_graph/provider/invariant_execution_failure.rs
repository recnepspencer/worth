use crate::domain_computation::{
    WorthQueryInvariantExecutionDenialKind, WorthQueryInvariantExecutionFailure,
};

pub(super) fn map_transaction_admission_failure(
    denial: worth_relational::facade::mvcc::RelationalBranchTransactionAdmissionDenial,
) -> WorthQueryInvariantExecutionFailure {
    use worth_relational::facade::mvcc::RelationalBranchTransactionAdmissionDenial as Denial;
    match denial {
        Denial::StaleBasis => WorthQueryInvariantExecutionFailure::new(
            WorthQueryInvariantExecutionDenialKind::ProductBasisStale,
            "the exact product basis became stale before invariant candidate admission",
        ),
        Denial::Cancelled => request_interruption(
            worth_relational::facade::mvcc::RelationalOperationInterruption::Cancelled,
        ),
        Denial::TimedOut => request_interruption(
            worth_relational::facade::mvcc::RelationalOperationInterruption::TimedOut,
        ),
        Denial::RetentionCapacityExhausted => retention_capacity_failure(),
        Denial::RetentionIdentityExhausted => exhausted_failure(
            WorthQueryInvariantExecutionDenialKind::RetentionIdentityExhausted,
            "Relational invariant execution exhausted retention identity space",
        ),
        _ => owner_failure(),
    }
}

pub(super) fn map_transaction_staging_failure(
    denial: worth_relational::facade::mvcc::RelationalTransactionStagingDenial,
) -> WorthQueryInvariantExecutionFailure {
    use worth_relational::facade::mvcc::RelationalTransactionStagingDenial as Denial;
    let kind = match denial {
        native @ (Denial::AllocationDenied(_)
        | Denial::CardinalityOverflow
        | Denial::InputDirectoryAllocationDenied { .. }) => {
            return WorthQueryInvariantExecutionFailure::native_staging(
                native,
                "Relational staging owner refused backing or cardinality",
            );
        }
        Denial::SavepointCapacityExhausted { maximum_savepoints } => {
            WorthQueryInvariantExecutionDenialKind::SavepointCapacityExhausted {
                maximum_savepoints,
            }
        }
        Denial::SavepointIdentityExhausted => {
            WorthQueryInvariantExecutionDenialKind::SavepointIdentityExhausted
        }
        Denial::MaterializationAuthorityRequired => {
            return provider_failure(
                "Relational invariant transaction requires materialization authority",
            );
        }
        Denial::MaterializationModeMismatch => {
            return provider_failure(
                "Relational invariant transaction materialization mode does not match its intents",
            );
        }
    };
    exhausted_failure(
        kind,
        "Relational invariant transaction staging exhausted an owner budget",
    )
}

pub(super) fn map_validation_failure(
    failure: worth_relational::facade::mvcc::TransactionCommitError,
) -> WorthQueryInvariantExecutionFailure {
    use worth_relational::facade::mvcc::{
        RelationalPublicationDeferred as Deferred, RelationalPublicationFailureKind as Failure,
        TransactionCommitError as Error,
    };
    use worth_relational::facade::transactions::CommitPreparationReason;
    use worth_relational::facade::transactions::ConflictClass;
    let kind = match failure {
        Error::Interrupted { interruption, .. } => {
            return request_interruption(interruption.interruption())
        }
        Error::Conflict { error, .. } => {
            if let Some(cause) = error.allocation_denial() {
                return WorthQueryInvariantExecutionFailure::physical_allocation(
                    cause.clone(),
                    error.detail(),
                );
            }
            let relational_detail = error.detail();
            let ConflictClass::InvariantViolation { fields, detail, .. } = error.class else {
                return provider_failure(relational_detail);
            };
            return map_invariant_failure(fields, detail);
        }
        Error::PublicationDeferred { deferred, .. } => match deferred {
            Deferred::CompanionRegistrationPending
            | Deferred::CompanionRebindRequired
            | Deferred::CompanionPreflight(_) => {
                WorthQueryInvariantExecutionDenialKind::RelationalDeferred(deferred)
            }
            Deferred::RetentionBackpressure => return retention_capacity_failure(),
            Deferred::PatchPositionReservationContended => {
                WorthQueryInvariantExecutionDenialKind::PatchPositionReservationContended
            }
            Deferred::CandidateCapacityExhausted { maximum_candidates } => {
                WorthQueryInvariantExecutionDenialKind::CandidateCapacityExhausted {
                    maximum_candidates,
                }
            }
            Deferred::PublishedSnapshotCapacityExhausted { maximum_handles } => {
                WorthQueryInvariantExecutionDenialKind::PublishedSnapshotCapacityExhausted {
                    maximum_handles,
                }
            }
            Deferred::CandidateLifetimeExpired { .. } => return owner_failure(),
        },
        Error::PublicationFailed { failure, .. } => match failure.kind() {
            Failure::SnapshotIdentityExhausted => {
                WorthQueryInvariantExecutionDenialKind::SnapshotIdentityExhausted
            }
            Failure::CandidateIdentityExhausted => {
                WorthQueryInvariantExecutionDenialKind::CandidateIdentityExhausted
            }
            Failure::RetentionIdentityExhausted => {
                WorthQueryInvariantExecutionDenialKind::RetentionIdentityExhausted
            }
            _ => return owner_failure(),
        },
        Error::Preparation { error, .. }
            if error.reason() == CommitPreparationReason::ProposalIdentityOrdinalExhausted =>
        {
            WorthQueryInvariantExecutionDenialKind::ProposalIdentityExhausted
        }
        _ => return owner_failure(),
    };
    exhausted_failure(
        kind,
        "Relational invariant candidate validation exhausted an owner budget",
    )
}

fn map_invariant_failure(
    fields: worth_relational::facade::transactions::InvariantViolationFields,
    detail: String,
) -> WorthQueryInvariantExecutionFailure {
    map_custom_invariant_failure(fields, detail.clone()).unwrap_or_else(|| {
        WorthQueryInvariantExecutionFailure::new(
            WorthQueryInvariantExecutionDenialKind::ProviderRejected,
            detail,
        )
    })
}

fn map_custom_invariant_failure(
    fields: worth_relational::facade::transactions::InvariantViolationFields,
    detail: String,
) -> Option<WorthQueryInvariantExecutionFailure> {
    use crate::domain_computation::WorthQueryCustomInvariantDenial;
    use worth_relational::facade::transactions::InvariantViolationFields;

    let denial = match fields {
        InvariantViolationFields::CustomInvariantViolation { identity } => {
            WorthQueryCustomInvariantDenial::Violation { identity }
        }
        InvariantViolationFields::CustomInvariantFailure {
            identity,
            phase,
            failure,
            ..
        } => WorthQueryCustomInvariantDenial::Failure {
            identity,
            phase,
            failure,
        },
        _ => return None,
    };
    Some(WorthQueryInvariantExecutionFailure::custom_invariant(
        denial, detail,
    ))
}

fn retention_capacity_failure() -> WorthQueryInvariantExecutionFailure {
    exhausted_failure(
        WorthQueryInvariantExecutionDenialKind::RetentionCapacityExhausted,
        "Relational invariant execution exhausted owner retention capacity",
    )
}

fn exhausted_failure(
    kind: WorthQueryInvariantExecutionDenialKind,
    detail: &'static str,
) -> WorthQueryInvariantExecutionFailure {
    WorthQueryInvariantExecutionFailure::exhausted(kind, detail)
}

fn owner_failure() -> WorthQueryInvariantExecutionFailure {
    provider_failure("Relational rejected the installed proposed-state invariant")
}

fn provider_failure(detail: impl Into<std::sync::Arc<str>>) -> WorthQueryInvariantExecutionFailure {
    WorthQueryInvariantExecutionFailure::new(
        WorthQueryInvariantExecutionDenialKind::ProviderRejected,
        detail,
    )
}

#[cfg(test)]
mod tests {
    use super::{
        map_custom_invariant_failure, map_invariant_failure, map_transaction_staging_failure,
    };
    use crate::domain_computation::WorthQueryCustomInvariantDenial;
    use worth_relational::facade::transactions::{
        CustomInvariantFailureIdentity, CustomInvariantFailurePhase, CustomInvariantRuleId,
        CustomInvariantSemanticIdentity, CustomInvariantSemanticVersion, InvariantViolationFields,
        ResultCustomInvariantFailureKind,
    };

    #[test]
    fn custom_semantic_violation_preserves_owner_identity() {
        let identity = semantic_identity();
        let failure = map_custom_invariant_failure(
            InvariantViolationFields::CustomInvariantViolation {
                identity: identity.clone(),
            },
            "custom semantic violation".to_owned(),
        )
        .unwrap();

        assert_eq!(
            failure.kind(),
            crate::domain_computation::WorthQueryInvariantExecutionDenialKind::CustomInvariantDenied
        );
        assert_eq!(
            failure.custom_invariant_denial(),
            Some(&WorthQueryCustomInvariantDenial::Violation { identity })
        );
        let denial = crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenial::invariant_execution_denied(
            crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialStage::InvariantExecution,
            failure.clone(),
        );
        assert_eq!(
            denial.custom_invariant_denial(),
            failure.custom_invariant_denial()
        );
        assert_eq!(denial.invariant_execution_failure(), Some(&failure));
    }

    #[test]
    fn custom_operational_failure_preserves_identity_phase_and_kind() {
        let identity = CustomInvariantFailureIdentity::new(semantic_identity());
        let failure = map_custom_invariant_failure(
            InvariantViolationFields::CustomInvariantFailure {
                identity: identity.clone(),
                phase: CustomInvariantFailurePhase::Execution,
                failure: ResultCustomInvariantFailureKind::Panic,
                detail: "hostile panic".to_owned(),
            },
            "custom invariant execution failed".to_owned(),
        )
        .unwrap();

        assert_eq!(
            failure.kind(),
            crate::domain_computation::WorthQueryInvariantExecutionDenialKind::CustomInvariantDenied
        );
        assert_eq!(
            failure.custom_invariant_denial(),
            Some(&WorthQueryCustomInvariantDenial::Failure {
                identity,
                phase: CustomInvariantFailurePhase::Execution,
                failure: ResultCustomInvariantFailureKind::Panic,
            })
        );
    }

    #[test]
    fn built_in_invariant_preserves_relational_detail() {
        let failure = map_invariant_failure(
            InvariantViolationFields::None,
            "relation endpoint deletion leaves an incident edge".to_owned(),
        );

        assert_eq!(
            failure.kind(),
            crate::domain_computation::WorthQueryInvariantExecutionDenialKind::ProviderRejected
        );
        assert_eq!(
            failure.detail(),
            "relation endpoint deletion leaves an incident edge"
        );
    }

    #[test]
    fn invariant_materialization_refusal_is_not_exhaustion() {
        for (owner_denial, expected_detail) in [
            (
                worth_relational::facade::mvcc::RelationalTransactionStagingDenial::MaterializationAuthorityRequired,
                "Relational invariant transaction requires materialization authority",
            ),
            (
                worth_relational::facade::mvcc::RelationalTransactionStagingDenial::MaterializationModeMismatch,
                "Relational invariant transaction materialization mode does not match its intents",
            ),
        ] {
            let failure = map_transaction_staging_failure(owner_denial);
            let denial = crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenial::invariant_execution_denied(
                crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialStage::InvariantExecution,
                failure,
            );
            let retained = denial.invariant_execution_failure().unwrap();
            assert_eq!(retained.kind(), crate::domain_computation::WorthQueryInvariantExecutionDenialKind::ProviderRejected);
            assert_eq!(retained.posture(), crate::domain_computation::WorthQueryInvariantExecutionFailurePosture::Denied);
            assert_eq!(retained.detail(), expected_detail);
        }
    }

    fn semantic_identity() -> CustomInvariantSemanticIdentity {
        CustomInvariantSemanticIdentity {
            rule_id: CustomInvariantRuleId::new("query.custom-invariant"),
            semantic_version: CustomInvariantSemanticVersion::new(3, 7),
        }
    }
}

fn request_interruption(
    reason: worth_relational::facade::mvcc::RelationalOperationInterruption,
) -> WorthQueryInvariantExecutionFailure {
    WorthQueryInvariantExecutionFailure::new(
        WorthQueryInvariantExecutionDenialKind::RequestInterrupted(reason),
        "the admitted request interrupted Relational candidate validation",
    )
}
