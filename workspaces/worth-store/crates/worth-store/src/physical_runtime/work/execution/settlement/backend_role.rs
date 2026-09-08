use worth_store_physical_backend::MediaOperationRole;

use super::PhysicalWorkSettlementEvidence;

impl PhysicalWorkSettlementEvidence {
    pub(in crate::physical_runtime) const fn backend_role(&self) -> Option<MediaOperationRole> {
        match self {
            Self::Inspection { .. } => Some(MediaOperationRole::PositionedRead),
            Self::InspectionDenied(_) => None,
            Self::NoEffect(_) | Self::StaleOrForeign => None,
            Self::Metadata { .. } => Some(MediaOperationRole::ReadMetadata),
            Self::Read { .. } => Some(MediaOperationRole::PositionedRead),
            Self::Write { .. } | Self::Publication { .. } | Self::NewArtifact { .. } => {
                Some(MediaOperationRole::PositionedWrite)
            }
            #[cfg(feature = "recovery-runtime-owner")]
            Self::RecoveryStaging { physical, .. } => {
                if physical.created().is_some() {
                    Some(MediaOperationRole::PositionedWrite)
                } else if physical.appended().is_some() {
                    Some(MediaOperationRole::Append)
                } else {
                    Some(MediaOperationRole::PositionedRead)
                }
            }
            Self::WalAppend { .. } | Self::WalSegmentCreate { .. } => {
                Some(MediaOperationRole::PositionedWrite)
            }
            Self::WalBarrier { .. } => Some(MediaOperationRole::SynchronizeFileState),
            Self::Checkpoint { physical, .. } => Some(physical.role()),
            Self::WalReclamation { physical, .. } => Some(physical.role()),
            Self::PublicationEffect { physical, .. } => Some(
                super::classification::publication::effect_role(physical.effect()),
            ),
            Self::TerminalFailure(failure) => Some(failure.backend_role),
        }
    }
}
