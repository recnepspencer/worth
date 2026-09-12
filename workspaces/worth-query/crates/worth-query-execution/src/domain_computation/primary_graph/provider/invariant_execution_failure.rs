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
        Denial::OverlayCapacityExhausted {
            maximum_bytes,
            required_bytes,
        } => WorthQueryInvariantExecutionDenialKind::TransactionOverlayCapacityExhausted {
            maximum_bytes,
            required_bytes,
        },
        Denial::FootprintCapacityExhausted {
            maximum_loci,
            required_loci,
        } => WorthQueryInvariantExecutionDenialKind::TransactionFootprintCapacityExhausted {
            maximum_loci,
            required_loci,
        },
        Denial::SavepointCapacityExhausted { maximum_savepoints } => {
            WorthQueryInvariantExecutionDenialKind::SavepointCapacityExhausted {
                maximum_savepoints,
            }
        }
        Denial::SavepointFootprintCapacityExhausted {
            maximum_loci,
            required_loci,
        } => WorthQueryInvariantExecutionDenialKind::SavepointFootprintCapacityExhausted {
            maximum_loci,
            required_loci,
        },
        Denial::SavepointIdentityExhausted => {
            WorthQueryInvariantExecutionDenialKind::SavepointIdentityExhausted
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
        Error::Conflict { error, .. } => {
            let ConflictClass::InvariantViolation { fields, detail, .. } = error.class else {
                return owner_failure();
            };
            return map_custom_invariant_failure(fields, detail).unwrap_or_else(owner_failure);
        }
        Error::PublicationDeferred { deferred, .. } => match deferred {
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
            Failure::PreparedRootBudgetExhausted {
                maximum_bytes,
                required_bytes,
            } => WorthQueryInvariantExecutionDenialKind::PreparedRootBudgetExhausted {
                maximum_bytes: *maximum_bytes,
                required_bytes: *required_bytes,
            },
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
    WorthQueryInvariantExecutionFailure::new(
        WorthQueryInvariantExecutionDenialKind::ProviderRejected,
        "Relational rejected the installed proposed-state invariant",
    )
}

#[cfg(test)]
mod tests {
    use super::map_custom_invariant_failure;
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

    fn semantic_identity() -> CustomInvariantSemanticIdentity {
        CustomInvariantSemanticIdentity {
            rule_id: CustomInvariantRuleId::new("query.custom-invariant"),
            semantic_version: CustomInvariantSemanticVersion::new(3, 7),
        }
    }
}
