//! Stable World admission names for actual Native fork refusals.
use crate::branch::RuntimeWorldBranchAdmissionDenial as World;
use worth_relational::facade::branch::RelationalForkDenial as Native;
pub(super) fn admission_denial(cause: &Native) -> World {
    match cause {
        Native::RetainedSourceDurabilityUnsupported => World::RetainedSourceDurabilityUnsupported,
        Native::RetentionCapacityExhausted | Native::RetentionIdentityExhausted => {
            World::CapacityExhausted
        }
        Native::EmptySource => World::EmptyNativeForkSource,
        Native::IndexPreparation(cause) => index_preparation_denial(&cause.kind),
        Native::SourceBranchMissing
        | Native::SourceArchived
        | Native::SourceDeleting
        | Native::DuplicateTarget
        | Native::RetiredTarget
        | Native::ForeignRuntime
        | Native::StaleSource
        | Native::DurableAssociationRequiresMedium
        | Native::MissingArtifact
        | Native::RootIdentityExhausted
        | Native::ManagedSource(_)
        | Native::InvalidTarget(_)
        | Native::Cell(_)
        | Native::RetentionOwnerUnavailable
        | Native::RetentionInvariantViolation
        | Native::OwnerUnavailable
        | Native::DurablePublicationRefused(_)
        | Native::TransitionSequenceExhausted
        | Native::TransitionStorageExhausted
        | Native::TransitionOriginalPrefixMismatch => World::OwnerUnavailable,
    }
}

fn index_preparation_denial(
    cause: &worth_relational::facade::indexes::DerivedIndexMaintenanceDenialKind,
) -> World {
    use worth_relational::facade::indexes::DerivedIndexMaintenanceDenialKind as Index;
    match cause {
        Index::Execution(cause) => execution_denial(*cause),
        Index::Basis(_)
        | Index::SnapshotUnavailable
        | Index::CommitMismatch
        | Index::BeforeRootMismatch
        | Index::IndexUnavailable(_)
        | Index::GenerationKindMismatch(_)
        | Index::PriorEntryMismatch
        | Index::ColdReconstructionRequired
        | Index::WorkBudgetExceeded
        | Index::ForeignCandidate
        | Index::CandidateLifetimeExpired { .. }
        | Index::CandidateUnavailable
        | Index::CandidateIndexesAlreadyPrepared
        | Index::GenerationIdentityExhausted => World::OwnerUnavailable,
    }
}

pub(super) fn execution_denial(
    cause: worth_relational::facade::transactions::RelationalExecutionDenialCause,
) -> World {
    use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;
    match cause {
        Cause::DeadlineElapsed => World::DeadlineBeforeEffect,
        Cause::Cancelled | Cause::NestedStopped => World::CancelledBeforeEffect,
        Cause::PolicyMemoryExhausted { .. }
        | Cause::ProcessMemoryExhausted { .. }
        | Cause::DeclaredMemoryExhausted { .. }
        | Cause::ChargedBytesOverflow
        | Cause::ResultCapacityExceeded
        | Cause::AllocationFailed
        | Cause::ScratchCapacityExceeded
        | Cause::MemoryOverflow => World::CapacityExhausted,
        Cause::Content(_)
        | Cause::ContentInconsistency
        | Cause::HistoricalVisibility(_)
        | Cause::ProjectionContract(_)
        | Cause::StorageUnavailable(_)
        | Cause::WorkerLimitExceedsParent
        | Cause::MemoryLimitExceedsParent
        | Cause::WorkLimitExceedsParent
        | Cause::UnrelatedNestedLease
        | Cause::NoActiveExecutionScope
        | Cause::EquivalenceContractUnavailable
        | Cause::WorkCounterOverflow
        | Cause::AccountingUnavailable
        | Cause::WorkExhausted
        | Cause::WorkerFailed
        | Cause::UncheckedCustomKernel
        | Cause::ExpectedIdentitiesNotCanonical => World::OwnerUnavailable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_relational::facade::{
        indexes::{DerivedIndexMaintenanceDenial, DerivedIndexMaintenanceDenialKind},
        transactions::RelationalExecutionDenialCause as Execution,
    };

    #[test]
    fn index_preparation_preserves_deadline_cancellation_and_capacity_refusals() {
        for (cause, expected) in [
            (Execution::DeadlineElapsed, World::DeadlineBeforeEffect),
            (Execution::Cancelled, World::CancelledBeforeEffect),
            (Execution::NestedStopped, World::CancelledBeforeEffect),
            (
                Execution::PolicyMemoryExhausted {
                    requested: 2,
                    admitted: 1,
                    ancestor: 0,
                },
                World::CapacityExhausted,
            ),
            (
                Execution::ProcessMemoryExhausted {
                    requested: 2,
                    admitted: 1,
                },
                World::CapacityExhausted,
            ),
            (
                Execution::DeclaredMemoryExhausted {
                    requested: 2,
                    admitted: 1,
                },
                World::CapacityExhausted,
            ),
            (Execution::WorkExhausted, World::OwnerUnavailable),
        ] {
            let denial = Native::IndexPreparation(Box::new(DerivedIndexMaintenanceDenial {
                kind: DerivedIndexMaintenanceDenialKind::Execution(cause),
                work: Default::default(),
            }));
            assert_eq!(admission_denial(&denial), expected);
        }
    }
}
