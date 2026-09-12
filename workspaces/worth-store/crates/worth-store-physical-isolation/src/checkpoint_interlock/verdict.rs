use super::{CheckpointPublicationStabilityProof, CheckpointReadInterlockDenial};
use crate::PhysicalReadPlanCompletionReceipt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadDuringCheckpointVerdict {
    proof: CheckpointPublicationStabilityProof,
}

impl ReadDuringCheckpointVerdict {
    pub fn from_stability_proof(
        proof: CheckpointPublicationStabilityProof,
    ) -> Result<Self, CheckpointReadInterlockDenial> {
        if !proof.no_mixed_root() {
            return Err(CheckpointReadInterlockDenial::MixedRootDuringCheckpointPublication);
        }
        Ok(Self { proof })
    }

    pub const fn proof(&self) -> &CheckpointPublicationStabilityProof {
        &self.proof
    }

    pub const fn pre_publication_read(&self) -> PhysicalReadPlanCompletionReceipt {
        self.proof.plan().pre_publication_read()
    }

    pub const fn post_publication_read(&self) -> PhysicalReadPlanCompletionReceipt {
        self.proof.post_publication_read()
    }

    pub fn old_reader_retained_old_root(&self) -> bool {
        self.pre_publication_read().read_plan_release().root() == self.proof.pre_publication_root()
    }

    pub fn post_publication_reader_observed_new_epoch(&self) -> bool {
        self.post_publication_read().read_plan_release().root()
            == self.proof.post_publication_root()
    }
}
