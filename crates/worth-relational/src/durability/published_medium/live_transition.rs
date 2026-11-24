//! Make the exact Branch transition durable before its live cutover.
use super::{RelationalDurableMediumBinding, RelationalDurableStepStop};
use crate::branch::transition::{
    durable_step::sealed, PositionedBranchTransition, RelationalBranchTransitionDurableStep,
    RelationalTransitionPublication,
};
use std::sync::Arc;

#[cfg(test)]
mod publication_pause_for_test;

/// The step of a live transition: publish to the runtime's medium when it
/// owns one. A memory-only runtime records the transition in its own stream
/// only.
pub(crate) struct LiveBranchTransitionStep<'request, 'authority> {
    medium: Option<Arc<RelationalDurableMediumBinding>>,
    policy: worth_execution::ExecutionAllocationPolicy<'request, 'authority>,
    association: Option<crate::history::data::RelationalCallerAssociation>,
    #[cfg(test)]
    publication_pause: publication_pause_for_test::BranchTransitionPublicationControlForTest,
}

impl<'request, 'authority> LiveBranchTransitionStep<'request, 'authority> {
    pub(crate) fn new(
        medium: Option<Arc<RelationalDurableMediumBinding>>,
        execution: &worth_execution::ExecutionRequest<'request, 'authority>,
    ) -> Self {
        Self {
            medium,
            policy: execution.allocation_policy(),
            association: None,
            #[cfg(test)]
            publication_pause: Default::default(),
        }
    }

    #[cfg(test)]
    pub(crate) fn pause_publication_for_test(
        &self,
    ) -> publication_pause_for_test::BranchTransitionPublicationPauseForTest {
        self.publication_pause.arm()
    }

    pub(crate) fn associated(
        medium: Arc<RelationalDurableMediumBinding>,
        execution: &worth_execution::ExecutionRequest<'request, 'authority>,
        association: crate::history::data::RelationalCallerAssociation,
    ) -> Self {
        let mut step = Self::new(Some(medium), execution);
        step.association = Some(association);
        step
    }
}

impl sealed::Step for LiveBranchTransitionStep<'_, '_> {}

impl RelationalBranchTransitionDurableStep for LiveBranchTransitionStep<'_, '_> {
    type Stop = RelationalDurableStepStop;

    fn caller_association(&self) -> Option<&crate::history::data::RelationalCallerAssociation> {
        self.association.as_ref()
    }

    /// Cost: one artifact encode and one medium publication with a medium.
    fn take(
        &self,
        positioned: PositionedBranchTransition,
    ) -> Result<RelationalTransitionPublication, Self::Stop> {
        #[cfg(test)]
        self.publication_pause.pause();
        match &self.medium {
            Some(medium) => medium
                .publish_transition(positioned, self.policy)
                .map(RelationalTransitionPublication::durable),
            None => Ok(RelationalTransitionPublication::memory(positioned)),
        }
    }
}
