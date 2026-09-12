use super::{CheckpointReadInterlockDenial, CheckpointRootEpochTransition};
use crate::PhysicalReadPlanCompletionReceipt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckpointReadInterlockPlan {
    pre_publication_read: PhysicalReadPlanCompletionReceipt,
    transition: CheckpointRootEpochTransition,
}

impl CheckpointReadInterlockPlan {
    pub fn admit(
        pre_publication_read: PhysicalReadPlanCompletionReceipt,
        transition: CheckpointRootEpochTransition,
    ) -> Result<Self, CheckpointReadInterlockDenial> {
        let observed = pre_publication_read.read_plan_release().root();
        let expected = transition.old_current_root();
        if observed != expected {
            return Err(
                CheckpointReadInterlockDenial::PrePublicationReadReceiptMismatch {
                    expected,
                    observed,
                },
            );
        }
        Ok(Self {
            pre_publication_read,
            transition,
        })
    }

    pub const fn pre_publication_read(&self) -> PhysicalReadPlanCompletionReceipt {
        self.pre_publication_read
    }

    pub const fn transition(&self) -> &CheckpointRootEpochTransition {
        &self.transition
    }
}
