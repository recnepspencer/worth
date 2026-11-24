//! Make the exact Branch transition durable before its live cutover.
use super::RelationalDurableMediumBinding;
use std::sync::Arc;
use worth_relational_branch::internal_port::branch::transition::durable_step::{
    sealed, RelationalBranchTransitionDurableStep, RelationalTransitionPublication,
};
use worth_relational_branch::internal_port::branch::transition::positioned::PositionedBranchTransition;
use worth_relational_storage::internal_port::durability::medium::step_stop::RelationalDurableStepStop;

pub mod publication_pause_for_test;

/// The step of a live transition: publish to the runtime's medium when it
/// owns one. A memory-only runtime records the transition in its own stream
/// only.
pub struct LiveBranchTransitionStep<'request, 'authority> {
    medium: Option<Arc<RelationalDurableMediumBinding>>,
    policy: worth_execution::ExecutionAllocationPolicy<'request, 'authority>,
    association: Option<worth_relational_branch::internal_port::history::data::caller_association::RelationalCallerAssociation>,
}

impl<'request, 'authority> LiveBranchTransitionStep<'request, 'authority> {
    pub fn new(
        medium: Option<Arc<RelationalDurableMediumBinding>>,
        execution: &worth_execution::ExecutionRequest<'request, 'authority>,
    ) -> Self {
        Self {
            medium,
            policy: execution.allocation_policy(),
            association: None,
        }
    }

    pub fn pause_publication_for_test(
        self,
    ) -> (
        publication_pause_for_test::PausedBranchTransitionForTest<'request, 'authority>,
        publication_pause_for_test::BranchTransitionPublicationPauseForTest,
    ) {
        publication_pause_for_test::PausedBranchTransitionForTest::arm(self)
    }

    pub fn associated(
        medium: Arc<RelationalDurableMediumBinding>,
        execution: &worth_execution::ExecutionRequest<'request, 'authority>,
        association: worth_relational_branch::internal_port::history::data::caller_association::RelationalCallerAssociation,
    ) -> Self {
        let mut step = Self::new(Some(medium), execution);
        step.association = Some(association);
        step
    }
}

impl sealed::Step for LiveBranchTransitionStep<'_, '_> {}

impl RelationalBranchTransitionDurableStep for LiveBranchTransitionStep<'_, '_> {
    type Stop = RelationalDurableStepStop;

    fn caller_association(&self) -> Option<&worth_relational_branch::internal_port::history::data::caller_association::RelationalCallerAssociation>{
        self.association.as_ref()
    }

    /// Cost: one artifact encode and one medium publication with a medium.
    fn take(
        &self,
        positioned: PositionedBranchTransition,
    ) -> Result<RelationalTransitionPublication, Self::Stop> {
        match &self.medium {
            Some(medium) => medium
                .publish_transition(positioned, self.policy)
                .map(RelationalTransitionPublication::durable),
            None => Ok(RelationalTransitionPublication::memory(positioned)),
        }
    }
}
