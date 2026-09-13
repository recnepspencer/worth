use super::{
    PhysicalWorkArtifactCode, PhysicalWorkCheckpointActionCode, PhysicalWorkObligationTargetCode,
};

pub(super) fn valid_target_shape(
    target: PhysicalWorkObligationTargetCode,
    has_digest: bool,
) -> bool {
    match target {
        PhysicalWorkObligationTargetCode::Range {
            artifact,
            offset,
            byte_count,
        } => valid_artifact(artifact) && has_digest && valid_interval(offset, byte_count),
        PhysicalWorkObligationTargetCode::WalArtifactInterval {
            segment,
            generation,
            offset,
            byte_count,
        } => segment > 0 && generation > 0 && has_digest && valid_interval(offset, byte_count),
        PhysicalWorkObligationTargetCode::Checkpoint { sequence, action } => {
            sequence > 0 && valid_checkpoint_action(action, has_digest)
        }
        PhysicalWorkObligationTargetCode::WalSegmentReclamation {
            segment,
            generation,
        } => segment > 0 && generation > 0 && !has_digest,
        PhysicalWorkObligationTargetCode::ArtifactFileSynchronization(artifact)
        | PhysicalWorkObligationTargetCode::ArtifactParentSynchronization(artifact) => {
            valid_artifact(artifact) && !has_digest
        }
        PhysicalWorkObligationTargetCode::CatalogReplacement(
            PhysicalWorkArtifactCode::CatalogCandidate { .. },
        ) => !has_digest,
        PhysicalWorkObligationTargetCode::CatalogReplacement(_) => false,
        PhysicalWorkObligationTargetCode::RecordNamespaceSynchronization => !has_digest,
    }
}

fn valid_artifact(artifact: PhysicalWorkArtifactCode) -> bool {
    !matches!(
        artifact,
        PhysicalWorkArtifactCode::RootSelectorCandidate { role, .. } if role != 1 && role != 2
    )
}

fn valid_checkpoint_action(action: PhysicalWorkCheckpointActionCode, has_digest: bool) -> bool {
    match action {
        PhysicalWorkCheckpointActionCode::CreateCandidate { byte_count } => {
            byte_count > 0 && has_digest
        }
        PhysicalWorkCheckpointActionCode::AppendCandidate { offset, byte_count } => {
            has_digest && valid_interval(offset, byte_count)
        }
        PhysicalWorkCheckpointActionCode::SynchronizeCandidate
        | PhysicalWorkCheckpointActionCode::RemoveCandidate
        | PhysicalWorkCheckpointActionCode::PublishCandidate
        | PhysicalWorkCheckpointActionCode::SynchronizeNamespace => !has_digest,
    }
}

fn valid_interval(offset: u64, byte_count: u64) -> bool {
    byte_count > 0 && offset.checked_add(byte_count).is_some()
}
