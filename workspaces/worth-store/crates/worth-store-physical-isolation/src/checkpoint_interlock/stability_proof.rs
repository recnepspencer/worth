use super::{CheckpointReadInterlockDenial, CheckpointReadInterlockPlan};
use crate::{CurrentPhysicalRoot, PhysicalReadPlanCompletionReceipt};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckpointPublicationStabilityProof {
    plan: CheckpointReadInterlockPlan,
    post_publication_read: PhysicalReadPlanCompletionReceipt,
}

impl CheckpointPublicationStabilityProof {
    pub fn from_plan_and_post_publication_read(
        plan: CheckpointReadInterlockPlan,
        post_publication_read: PhysicalReadPlanCompletionReceipt,
    ) -> Result<Self, CheckpointReadInterlockDenial> {
        let observed = post_publication_read.read_plan_release().root();
        let expected = plan.transition().published_current_root();
        if observed != expected {
            return Err(
                CheckpointReadInterlockDenial::PostPublicationReadReceiptMismatch {
                    expected,
                    observed,
                },
            );
        }
        Ok(Self {
            plan,
            post_publication_read,
        })
    }

    pub const fn pre_publication_root(&self) -> CurrentPhysicalRoot {
        self.plan.transition().old_current_root()
    }

    pub const fn post_publication_root(&self) -> CurrentPhysicalRoot {
        self.plan.transition().published_current_root()
    }

    pub const fn checkpoint_publication_root(&self) -> &crate::CheckpointPublicationRoot {
        self.plan.transition().checkpoint_root()
    }

    pub const fn plan(&self) -> &CheckpointReadInterlockPlan {
        &self.plan
    }

    pub const fn post_publication_read(&self) -> PhysicalReadPlanCompletionReceipt {
        self.post_publication_read
    }

    pub fn no_mixed_root(&self) -> bool {
        self.pre_publication_root() != self.post_publication_root()
            && self.checkpoint_publication_root().epoch() == self.post_publication_root().epoch()
    }
}
