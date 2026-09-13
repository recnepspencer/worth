use worth_store_physical_format::physical_work_obligation::{
    PhysicalWorkArtifactCode, PhysicalWorkCheckpointActionCode,
    PhysicalWorkObligationOperationCode, PhysicalWorkObligationTargetCode,
};
use worth_store_physical_format::{RecordArtifactFile, RecordFrameCoordinate, RootSelectorRole};

use super::{PhysicalCheckpointRecoveryAction, PhysicalWorkRecoveryTarget};
use crate::physical_runtime::work::PhysicalWorkOperationFamily;

pub(super) const fn operation_to_format(
    family: PhysicalWorkOperationFamily,
) -> PhysicalWorkObligationOperationCode {
    match family {
        PhysicalWorkOperationFamily::ArtifactMetadataRead => {
            PhysicalWorkObligationOperationCode::ArtifactMetadataRead
        }
        PhysicalWorkOperationFamily::ArtifactRangeRead => {
            PhysicalWorkObligationOperationCode::ArtifactRangeRead
        }
        PhysicalWorkOperationFamily::ArtifactRangeWrite => {
            PhysicalWorkObligationOperationCode::ArtifactRangeWrite
        }
        PhysicalWorkOperationFamily::ArtifactPublication => {
            PhysicalWorkObligationOperationCode::ArtifactPublication
        }
        PhysicalWorkOperationFamily::WalAppend => PhysicalWorkObligationOperationCode::WalAppend,
        PhysicalWorkOperationFamily::DurabilityBarrier => {
            PhysicalWorkObligationOperationCode::DurabilityBarrier
        }
        PhysicalWorkOperationFamily::CheckpointCapture => {
            PhysicalWorkObligationOperationCode::CheckpointCapture
        }
        PhysicalWorkOperationFamily::WalReclamation => {
            PhysicalWorkObligationOperationCode::WalReclamation
        }
        PhysicalWorkOperationFamily::RootPublication => {
            PhysicalWorkObligationOperationCode::RootPublication
        }
    }
}

pub(super) const fn operation_from_format(
    family: PhysicalWorkObligationOperationCode,
) -> PhysicalWorkOperationFamily {
    match family {
        PhysicalWorkObligationOperationCode::ArtifactMetadataRead => {
            PhysicalWorkOperationFamily::ArtifactMetadataRead
        }
        PhysicalWorkObligationOperationCode::ArtifactRangeRead => {
            PhysicalWorkOperationFamily::ArtifactRangeRead
        }
        PhysicalWorkObligationOperationCode::ArtifactRangeWrite => {
            PhysicalWorkOperationFamily::ArtifactRangeWrite
        }
        PhysicalWorkObligationOperationCode::ArtifactPublication => {
            PhysicalWorkOperationFamily::ArtifactPublication
        }
        PhysicalWorkObligationOperationCode::WalAppend => PhysicalWorkOperationFamily::WalAppend,
        PhysicalWorkObligationOperationCode::DurabilityBarrier => {
            PhysicalWorkOperationFamily::DurabilityBarrier
        }
        PhysicalWorkObligationOperationCode::CheckpointCapture => {
            PhysicalWorkOperationFamily::CheckpointCapture
        }
        PhysicalWorkObligationOperationCode::WalReclamation => {
            PhysicalWorkOperationFamily::WalReclamation
        }
        PhysicalWorkObligationOperationCode::RootPublication => {
            PhysicalWorkOperationFamily::RootPublication
        }
    }
}

pub(super) fn target_to_format(
    target: PhysicalWorkRecoveryTarget,
) -> PhysicalWorkObligationTargetCode {
    match target {
        PhysicalWorkRecoveryTarget::Range(coordinate) => PhysicalWorkObligationTargetCode::Range {
            artifact: artifact_to_format(coordinate.artifact()),
            offset: coordinate.offset(),
            byte_count: u64::from(coordinate.length()),
        },
        PhysicalWorkRecoveryTarget::WalArtifactInterval {
            segment,
            generation,
            offset,
            byte_count,
        } => PhysicalWorkObligationTargetCode::WalArtifactInterval {
            segment,
            generation,
            offset,
            byte_count,
        },
        PhysicalWorkRecoveryTarget::Checkpoint { sequence, action } => {
            PhysicalWorkObligationTargetCode::Checkpoint {
                sequence,
                action: checkpoint_to_format(action),
            }
        }
        PhysicalWorkRecoveryTarget::WalSegmentReclamation {
            segment,
            generation,
        } => PhysicalWorkObligationTargetCode::WalSegmentReclamation {
            segment,
            generation,
        },
        PhysicalWorkRecoveryTarget::ArtifactFileSynchronization(artifact) => {
            PhysicalWorkObligationTargetCode::ArtifactFileSynchronization(artifact_to_format(
                artifact,
            ))
        }
        PhysicalWorkRecoveryTarget::ArtifactParentSynchronization(artifact) => {
            PhysicalWorkObligationTargetCode::ArtifactParentSynchronization(artifact_to_format(
                artifact,
            ))
        }
        PhysicalWorkRecoveryTarget::CatalogReplacement(artifact) => {
            PhysicalWorkObligationTargetCode::CatalogReplacement(artifact_to_format(artifact))
        }
        PhysicalWorkRecoveryTarget::RecordNamespaceSynchronization => {
            PhysicalWorkObligationTargetCode::RecordNamespaceSynchronization
        }
    }
}

pub(super) fn target_from_format(
    target: PhysicalWorkObligationTargetCode,
) -> Option<PhysicalWorkRecoveryTarget> {
    match target {
        PhysicalWorkObligationTargetCode::Range {
            artifact,
            offset,
            byte_count,
        } => Some(PhysicalWorkRecoveryTarget::Range(
            RecordFrameCoordinate::new(
                artifact_from_format(artifact)?,
                offset,
                u32::try_from(byte_count).ok()?,
            )?,
        )),
        PhysicalWorkObligationTargetCode::WalArtifactInterval {
            segment,
            generation,
            offset,
            byte_count,
        } => Some(PhysicalWorkRecoveryTarget::WalArtifactInterval {
            segment,
            generation,
            offset,
            byte_count,
        }),
        PhysicalWorkObligationTargetCode::Checkpoint { sequence, action } => {
            Some(PhysicalWorkRecoveryTarget::Checkpoint {
                sequence,
                action: checkpoint_from_format(action),
            })
        }
        PhysicalWorkObligationTargetCode::WalSegmentReclamation {
            segment,
            generation,
        } => Some(PhysicalWorkRecoveryTarget::WalSegmentReclamation {
            segment,
            generation,
        }),
        PhysicalWorkObligationTargetCode::ArtifactFileSynchronization(artifact) => {
            Some(PhysicalWorkRecoveryTarget::ArtifactFileSynchronization(
                artifact_from_format(artifact)?,
            ))
        }
        PhysicalWorkObligationTargetCode::ArtifactParentSynchronization(artifact) => {
            Some(PhysicalWorkRecoveryTarget::ArtifactParentSynchronization(
                artifact_from_format(artifact)?,
            ))
        }
        PhysicalWorkObligationTargetCode::CatalogReplacement(artifact) => {
            let artifact = artifact_from_format(artifact)?;
            matches!(artifact, RecordArtifactFile::CatalogCandidate { .. }).then_some(())?;
            Some(PhysicalWorkRecoveryTarget::CatalogReplacement(artifact))
        }
        PhysicalWorkObligationTargetCode::RecordNamespaceSynchronization => {
            Some(PhysicalWorkRecoveryTarget::RecordNamespaceSynchronization)
        }
    }
}

fn artifact_to_format(artifact: RecordArtifactFile) -> PhysicalWorkArtifactCode {
    match artifact {
        RecordArtifactFile::BootstrapCatalog => PhysicalWorkArtifactCode::BootstrapCatalog,
        RecordArtifactFile::CurrentRootSelector => PhysicalWorkArtifactCode::CurrentRootSelector,
        RecordArtifactFile::PreviousRootSelector => PhysicalWorkArtifactCode::PreviousRootSelector,
        RecordArtifactFile::RootSelectorCandidate { role, publication } => {
            PhysicalWorkArtifactCode::RootSelectorCandidate {
                role: role as u8,
                publication,
            }
        }
        RecordArtifactFile::CatalogCandidate { publication } => {
            PhysicalWorkArtifactCode::CatalogCandidate { publication }
        }
        RecordArtifactFile::RootManifest { generation } => {
            PhysicalWorkArtifactCode::RootManifest { generation }
        }
        RecordArtifactFile::RootRoutingBlock { generation, block } => {
            PhysicalWorkArtifactCode::RootRoutingBlock { generation, block }
        }
        RecordArtifactFile::Segment {
            segment,
            generation,
        } => PhysicalWorkArtifactCode::Segment {
            segment,
            generation,
        },
        RecordArtifactFile::SegmentManifest {
            segment,
            generation,
        } => PhysicalWorkArtifactCode::SegmentManifest {
            segment,
            generation,
        },
        RecordArtifactFile::SegmentMembershipBlock { generation, block } => {
            PhysicalWorkArtifactCode::SegmentMembershipBlock { generation, block }
        }
        RecordArtifactFile::Extent { extent, generation } => {
            PhysicalWorkArtifactCode::Extent { extent, generation }
        }
        RecordArtifactFile::ExtentManifest { extent, generation } => {
            PhysicalWorkArtifactCode::ExtentManifest { extent, generation }
        }
        RecordArtifactFile::FreeSpaceManifest { generation } => {
            PhysicalWorkArtifactCode::FreeSpaceManifest { generation }
        }
        RecordArtifactFile::FreeSpaceMembershipBlock { generation, block } => {
            PhysicalWorkArtifactCode::FreeSpaceMembershipBlock { generation, block }
        }
    }
}

fn artifact_from_format(artifact: PhysicalWorkArtifactCode) -> Option<RecordArtifactFile> {
    Some(match artifact {
        PhysicalWorkArtifactCode::BootstrapCatalog => RecordArtifactFile::BootstrapCatalog,
        PhysicalWorkArtifactCode::CurrentRootSelector => RecordArtifactFile::CurrentRootSelector,
        PhysicalWorkArtifactCode::PreviousRootSelector => RecordArtifactFile::PreviousRootSelector,
        PhysicalWorkArtifactCode::RootSelectorCandidate { role, publication } => {
            RecordArtifactFile::RootSelectorCandidate {
                role: match role {
                    1 => RootSelectorRole::Current,
                    2 => RootSelectorRole::Previous,
                    _ => return None,
                },
                publication,
            }
        }
        PhysicalWorkArtifactCode::CatalogCandidate { publication } => {
            RecordArtifactFile::CatalogCandidate { publication }
        }
        PhysicalWorkArtifactCode::RootManifest { generation } => {
            RecordArtifactFile::RootManifest { generation }
        }
        PhysicalWorkArtifactCode::RootRoutingBlock { generation, block } => {
            RecordArtifactFile::RootRoutingBlock { generation, block }
        }
        PhysicalWorkArtifactCode::Segment {
            segment,
            generation,
        } => RecordArtifactFile::Segment {
            segment,
            generation,
        },
        PhysicalWorkArtifactCode::SegmentManifest {
            segment,
            generation,
        } => RecordArtifactFile::SegmentManifest {
            segment,
            generation,
        },
        PhysicalWorkArtifactCode::SegmentMembershipBlock { generation, block } => {
            RecordArtifactFile::SegmentMembershipBlock { generation, block }
        }
        PhysicalWorkArtifactCode::Extent { extent, generation } => {
            RecordArtifactFile::Extent { extent, generation }
        }
        PhysicalWorkArtifactCode::ExtentManifest { extent, generation } => {
            RecordArtifactFile::ExtentManifest { extent, generation }
        }
        PhysicalWorkArtifactCode::FreeSpaceManifest { generation } => {
            RecordArtifactFile::FreeSpaceManifest { generation }
        }
        PhysicalWorkArtifactCode::FreeSpaceMembershipBlock { generation, block } => {
            RecordArtifactFile::FreeSpaceMembershipBlock { generation, block }
        }
    })
}

const fn checkpoint_to_format(
    action: PhysicalCheckpointRecoveryAction,
) -> PhysicalWorkCheckpointActionCode {
    match action {
        PhysicalCheckpointRecoveryAction::CreateCandidate { byte_count } => {
            PhysicalWorkCheckpointActionCode::CreateCandidate { byte_count }
        }
        PhysicalCheckpointRecoveryAction::AppendCandidate { offset, byte_count } => {
            PhysicalWorkCheckpointActionCode::AppendCandidate { offset, byte_count }
        }
        PhysicalCheckpointRecoveryAction::SynchronizeCandidate => {
            PhysicalWorkCheckpointActionCode::SynchronizeCandidate
        }
        PhysicalCheckpointRecoveryAction::RemoveCandidate => {
            PhysicalWorkCheckpointActionCode::RemoveCandidate
        }
        PhysicalCheckpointRecoveryAction::PublishCandidate => {
            PhysicalWorkCheckpointActionCode::PublishCandidate
        }
        PhysicalCheckpointRecoveryAction::SynchronizeNamespace => {
            PhysicalWorkCheckpointActionCode::SynchronizeNamespace
        }
    }
}

const fn checkpoint_from_format(
    action: PhysicalWorkCheckpointActionCode,
) -> PhysicalCheckpointRecoveryAction {
    match action {
        PhysicalWorkCheckpointActionCode::CreateCandidate { byte_count } => {
            PhysicalCheckpointRecoveryAction::CreateCandidate { byte_count }
        }
        PhysicalWorkCheckpointActionCode::AppendCandidate { offset, byte_count } => {
            PhysicalCheckpointRecoveryAction::AppendCandidate { offset, byte_count }
        }
        PhysicalWorkCheckpointActionCode::SynchronizeCandidate => {
            PhysicalCheckpointRecoveryAction::SynchronizeCandidate
        }
        PhysicalWorkCheckpointActionCode::RemoveCandidate => {
            PhysicalCheckpointRecoveryAction::RemoveCandidate
        }
        PhysicalWorkCheckpointActionCode::PublishCandidate => {
            PhysicalCheckpointRecoveryAction::PublishCandidate
        }
        PhysicalWorkCheckpointActionCode::SynchronizeNamespace => {
            PhysicalCheckpointRecoveryAction::SynchronizeNamespace
        }
    }
}
