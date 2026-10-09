use sha2::{Digest, Sha256};

mod payload;
pub(in crate::physical_runtime) use payload::PhysicalCheckpointCommandPayload;

use super::super::super::{
    PhysicalCheckpointWorkAction, PhysicalWorkOperationFamily, ResourceAdmittedPhysicalWork,
};
use super::types::{
    require_family, PhysicalCheckpointExecutorCommand, PhysicalExecutorCommand,
    PhysicalExecutorCommandDenial,
};

impl PhysicalExecutorCommand {
    pub(in crate::physical_runtime) fn checkpoint(
        work: ResourceAdmittedPhysicalWork,
        payload: Option<PhysicalCheckpointCommandPayload>,
    ) -> Result<Self, PhysicalExecutorCommandDenial> {
        require_family(&work, PhysicalWorkOperationFamily::CheckpointCapture)?;
        let scope = work
            .intent()
            .scope()
            .checkpoint_target()
            .ok_or(PhysicalExecutorCommandDenial::CheckpointCommandRequiresCheckpointScope)?;
        let payload_digest = match (scope.action(), payload.as_deref()) {
            (
                PhysicalCheckpointWorkAction::CreateCandidate { byte_count }
                | PhysicalCheckpointWorkAction::AppendCandidate { byte_count, .. },
                Some(payload),
            ) if payload.len() as u64 == byte_count => Some(Sha256::digest(payload).into()),
            (
                PhysicalCheckpointWorkAction::SynchronizeCandidate
                | PhysicalCheckpointWorkAction::RemoveCandidate
                | PhysicalCheckpointWorkAction::PublishCandidate
                | PhysicalCheckpointWorkAction::SynchronizeNamespace,
                None,
            ) => None,
            _ => return Err(PhysicalExecutorCommandDenial::CheckpointPayloadPostureMismatch),
        };
        Ok(Self::Checkpoint(PhysicalCheckpointExecutorCommand {
            work,
            payload,
            payload_digest,
        }))
    }
}
