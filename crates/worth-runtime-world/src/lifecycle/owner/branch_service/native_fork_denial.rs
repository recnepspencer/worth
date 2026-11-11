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
