use super::{
    super::PhysicalWorkOperationFamily, PhysicalCheckpointRecoveryAction,
    PhysicalWorkRecoveryTarget,
};

/// When an effect's recovery record becomes durable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PhysicalEffectJournaling {
    /// The record is durable before the effect starts.
    BeforeEffect,
    /// A pure flush is idempotent and publishes nothing until it completes, so
    /// a crash during it leaves the same media state as a crash just before
    /// it. Its record is written only if its outcome is retained.
    OnlyIfRetained,
}

/// Every operation family and target is classified explicitly: a new variant
/// does not compile until its journaling is decided.
pub(super) const fn journaling(
    operation: PhysicalWorkOperationFamily,
    target: PhysicalWorkRecoveryTarget,
) -> PhysicalEffectJournaling {
    match operation {
        PhysicalWorkOperationFamily::DurabilityBarrier => PhysicalEffectJournaling::OnlyIfRetained,
        PhysicalWorkOperationFamily::ArtifactMetadataRead
        | PhysicalWorkOperationFamily::ArtifactRangeRead
        | PhysicalWorkOperationFamily::ArtifactRangeWrite
        | PhysicalWorkOperationFamily::ArtifactPublication
        | PhysicalWorkOperationFamily::CheckpointCapture
        | PhysicalWorkOperationFamily::WalAppend
        | PhysicalWorkOperationFamily::WalReclamation
        | PhysicalWorkOperationFamily::RootPublication => target_journaling(target),
    }
}

const fn target_journaling(target: PhysicalWorkRecoveryTarget) -> PhysicalEffectJournaling {
    match target {
        PhysicalWorkRecoveryTarget::ArtifactFileSynchronization(_)
        | PhysicalWorkRecoveryTarget::ArtifactParentSynchronization(_)
        | PhysicalWorkRecoveryTarget::RecordNamespaceSynchronization => {
            PhysicalEffectJournaling::OnlyIfRetained
        }
        PhysicalWorkRecoveryTarget::Checkpoint { action, .. } => checkpoint_journaling(action),
        PhysicalWorkRecoveryTarget::Range(_)
        | PhysicalWorkRecoveryTarget::WalArtifactInterval { .. }
        | PhysicalWorkRecoveryTarget::WalSegmentReclamation { .. }
        | PhysicalWorkRecoveryTarget::CatalogReplacement(_)
        | PhysicalWorkRecoveryTarget::ArtifactRemoval(_) => PhysicalEffectJournaling::BeforeEffect,
    }
}

const fn checkpoint_journaling(
    action: PhysicalCheckpointRecoveryAction,
) -> PhysicalEffectJournaling {
    match action {
        PhysicalCheckpointRecoveryAction::SynchronizeCandidate
        | PhysicalCheckpointRecoveryAction::SynchronizeNamespace => {
            PhysicalEffectJournaling::OnlyIfRetained
        }
        PhysicalCheckpointRecoveryAction::CreateCandidate { .. }
        | PhysicalCheckpointRecoveryAction::AppendCandidate { .. }
        | PhysicalCheckpointRecoveryAction::RemoveCandidate
        | PhysicalCheckpointRecoveryAction::PublishCandidate => {
            PhysicalEffectJournaling::BeforeEffect
        }
    }
}
