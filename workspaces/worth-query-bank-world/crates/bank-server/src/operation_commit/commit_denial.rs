//! Bank-owned description of a Query commit denial.

use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationAttemptDenialKind, WorthQueryApplicationCommitDenialKind,
    WorthQueryApplicationCommitDenialStage, WorthQueryRecoveryHandleDenialKind,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankCommitDenialKind {
    ExecutionResource {
        denial: worth_query_host::facade::application_contribution::WorthQueryManagedComputationResourceDenial,
        partition_identity: Option<u64>,
        policy_ancestor: Option<u32>,
    },
    ExecutionNestedPatternStopped { partition_identity: Option<u64> },
    ExecutionWorkerPanicked { partition_identity: Option<u64> },
    ExecutionUncheckedCustomKernel { partition_identity: Option<u64> },
    ExecutionIdentitiesNotCanonical { partition_identity: Option<u64> },
    ProviderRejected,
    CustomInvariantDenied,
    WorkflowSettlementDenied {
        kind: WorthQueryApplicationAttemptDenialKind,
    },
    ProductBasisStale,
    UniqueValueTaken,
    UniqueIndexUnavailable,
    ActiveSnapshotCapacityExhausted {
        maximum_active_snapshots: usize,
    },
    RetentionCapacityExhausted,
    RetentionIdentityExhausted,
    SnapshotIdentityExhausted,
    CandidateIdentityExhausted,
    PreparedRootBudgetExhausted {
        maximum_bytes: u64,
        required_bytes: u64,
    },
    IdempotencyIntentDrift,
    /// The key is recorded with the same intent and that commit took effect,
    /// but the runtime no longer holds its receipt, as after a restore.
    IdempotencyReceiptNotRetained,
    /// The key's recorded intent was written by an earlier encoding that
    /// cannot be checked against this request.
    IdempotencyIntentUnverifiable,
    MutationBindingMismatch,
    MutationInputMismatch,
    RecoveryHandoffMismatch {
        kind: WorthQueryRecoveryHandleDenialKind,
    },
    ElevationTransitionRequired,
    ElevationRequestProgramMismatch,
    ElevationApprovalProgramMismatch,
    ElevationCloseProgramMismatch,
    MandatoryReviewProgramMismatch,
    DelegationActivationRequired,
    CapabilityRevocationRequired,
    ApplicationProgramRequired,
    WorkflowAuthorityRequired,
    ProgramNotActiveOnOccurrence,
    ProgramSupportRetired,
    ProgramActivationUnresolved,
    IndexMaintenanceBudgetExceeded,
    IndexGenerationIdentityExhausted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankCommitDenialStage {
    ProposalBinding,
    BridgePlanning,
    BasisAdmission,
    ResourceAdmission,
    ManagedRunAdmission,
    ProviderPlan,
    Idempotency,
    DecisionReadSet,
    EffectLowering,
    ElevationTransition,
    DelegationTransition,
    ProvisionalState,
    InvariantExecution,
    ProviderCommit,
}

pub(crate) const fn denial_kind(
    kind: WorthQueryApplicationCommitDenialKind,
) -> BankCommitDenialKind {
    use WorthQueryApplicationCommitDenialKind as Query;
    match kind {
        Query::ExecutionResource {
            denial,
            partition_identity,
            policy_ancestor,
        } => BankCommitDenialKind::ExecutionResource {
            denial,
            partition_identity,
            policy_ancestor,
        },
        Query::ExecutionNestedPatternStopped { partition_identity } => {
            BankCommitDenialKind::ExecutionNestedPatternStopped { partition_identity }
        }
        Query::ExecutionWorkerPanicked { partition_identity } => {
            BankCommitDenialKind::ExecutionWorkerPanicked { partition_identity }
        }
        Query::ExecutionUncheckedCustomKernel { partition_identity } => {
            BankCommitDenialKind::ExecutionUncheckedCustomKernel { partition_identity }
        }
        Query::ExecutionIdentitiesNotCanonical { partition_identity } => {
            BankCommitDenialKind::ExecutionIdentitiesNotCanonical { partition_identity }
        }
        Query::ProviderRejected => BankCommitDenialKind::ProviderRejected,
        Query::CustomInvariantDenied => BankCommitDenialKind::CustomInvariantDenied,
        Query::WorkflowSettlementDenied { kind } => {
            BankCommitDenialKind::WorkflowSettlementDenied { kind }
        }
        Query::ProductBasisStale => BankCommitDenialKind::ProductBasisStale,
        Query::UniqueValueTaken => BankCommitDenialKind::UniqueValueTaken,
        Query::UniqueIndexUnavailable => BankCommitDenialKind::UniqueIndexUnavailable,
        Query::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        } => BankCommitDenialKind::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        },
        Query::RetentionCapacityExhausted => BankCommitDenialKind::RetentionCapacityExhausted,
        Query::RetentionIdentityExhausted => BankCommitDenialKind::RetentionIdentityExhausted,
        Query::SnapshotIdentityExhausted => BankCommitDenialKind::SnapshotIdentityExhausted,
        Query::CandidateIdentityExhausted => BankCommitDenialKind::CandidateIdentityExhausted,
        Query::PreparedRootBudgetExhausted {
            maximum_bytes,
            required_bytes,
        } => BankCommitDenialKind::PreparedRootBudgetExhausted {
            maximum_bytes,
            required_bytes,
        },
        Query::IdempotencyIntentDrift => BankCommitDenialKind::IdempotencyIntentDrift,
        Query::IdempotencyReceiptNotRetained { .. } => {
            BankCommitDenialKind::IdempotencyReceiptNotRetained
        }
        Query::IdempotencyIntentUnverifiable => BankCommitDenialKind::IdempotencyIntentUnverifiable,
        Query::MutationBindingMismatch => BankCommitDenialKind::MutationBindingMismatch,
        Query::MutationInputMismatch => BankCommitDenialKind::MutationInputMismatch,
        Query::RecoveryHandoffMismatch { kind } => {
            BankCommitDenialKind::RecoveryHandoffMismatch { kind }
        }
        Query::ElevationTransitionRequired => BankCommitDenialKind::ElevationTransitionRequired,
        Query::ElevationRequestProgramMismatch => {
            BankCommitDenialKind::ElevationRequestProgramMismatch
        }
        Query::ElevationApprovalProgramMismatch => {
            BankCommitDenialKind::ElevationApprovalProgramMismatch
        }
        Query::ElevationCloseProgramMismatch => BankCommitDenialKind::ElevationCloseProgramMismatch,
        Query::MandatoryReviewProgramMismatch => {
            BankCommitDenialKind::MandatoryReviewProgramMismatch
        }
        Query::DelegationActivationRequired => BankCommitDenialKind::DelegationActivationRequired,
        Query::CapabilityRevocationRequired => BankCommitDenialKind::CapabilityRevocationRequired,
        Query::ApplicationProgramRequired => BankCommitDenialKind::ApplicationProgramRequired,
        Query::WorkflowAuthorityRequired => BankCommitDenialKind::WorkflowAuthorityRequired,
        Query::ProgramNotActiveOnOccurrence { .. } => {
            BankCommitDenialKind::ProgramNotActiveOnOccurrence
        }
        Query::ProgramSupportRetired => BankCommitDenialKind::ProgramSupportRetired,
        Query::ProgramActivationUnresolved => BankCommitDenialKind::ProgramActivationUnresolved,
        Query::IndexMaintenanceBudgetExceeded => {
            BankCommitDenialKind::IndexMaintenanceBudgetExceeded
        }
        Query::IndexGenerationIdentityExhausted => {
            BankCommitDenialKind::IndexGenerationIdentityExhausted
        }
    }
}

pub(crate) const fn denial_stage(
    stage: WorthQueryApplicationCommitDenialStage,
) -> BankCommitDenialStage {
    use WorthQueryApplicationCommitDenialStage as Query;
    match stage {
        Query::ProposalBinding => BankCommitDenialStage::ProposalBinding,
        Query::BridgePlanning => BankCommitDenialStage::BridgePlanning,
        Query::BasisAdmission => BankCommitDenialStage::BasisAdmission,
        Query::ResourceAdmission => BankCommitDenialStage::ResourceAdmission,
        Query::ManagedRunAdmission => BankCommitDenialStage::ManagedRunAdmission,
        Query::ProviderPlan => BankCommitDenialStage::ProviderPlan,
        Query::Idempotency => BankCommitDenialStage::Idempotency,
        Query::DecisionReadSet => BankCommitDenialStage::DecisionReadSet,
        Query::EffectLowering => BankCommitDenialStage::EffectLowering,
        Query::ElevationTransition => BankCommitDenialStage::ElevationTransition,
        Query::DelegationTransition => BankCommitDenialStage::DelegationTransition,
        Query::ProvisionalState => BankCommitDenialStage::ProvisionalState,
        Query::InvariantExecution => BankCommitDenialStage::InvariantExecution,
        Query::ProviderCommit => BankCommitDenialStage::ProviderCommit,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn recovery_handoff_mismatch_keeps_its_typed_cause() {
        let kind = WorthQueryRecoveryHandleDenialKind::FreshAuthorityDenied;
        assert_eq!(
            denial_kind(WorthQueryApplicationCommitDenialKind::RecoveryHandoffMismatch { kind }),
            BankCommitDenialKind::RecoveryHandoffMismatch { kind }
        );
    }

    #[test]
    fn execution_resource_retains_its_kind_location_and_memory_level() {
        use worth_query_host::facade::application_contribution::{
            WorthQueryManagedComputationResourceDenial as Resource,
            WorthQueryMemoryLimitLevel as Level,
        };
        for level in [Level::Process, Level::Policy, Level::Declared] {
            let denial = Resource::MemoryLimit {
                requested: 31,
                admitted: 17,
                level,
            };
            assert_eq!(
                denial_kind(WorthQueryApplicationCommitDenialKind::ExecutionResource {
                    denial,
                    partition_identity: Some(7),
                    policy_ancestor: Some(2)
                }),
                BankCommitDenialKind::ExecutionResource {
                    denial,
                    partition_identity: Some(7),
                    policy_ancestor: Some(2)
                },
            );
        }
    }
    #[test]
    fn packet_panic_is_panicked_before_effects() {
        let query = WorthQueryApplicationCommitDenialKind::ExecutionWorkerPanicked {
            partition_identity: Some(7),
        };
        assert_eq!(
            denial_kind(query),
            BankCommitDenialKind::ExecutionWorkerPanicked {
                partition_identity: Some(7)
            }
        );
        assert_eq!(
            denial_kind(WorthQueryApplicationCommitDenialKind::ProviderRejected),
            BankCommitDenialKind::ProviderRejected
        );
    }
}
