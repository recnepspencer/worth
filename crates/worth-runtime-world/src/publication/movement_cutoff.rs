use super::{CurrentProductReadDependencyDenial, NoEffectCause, RuntimeWorldCancellationToken};
use crate::lifecycle::{RuntimeWorldClock, RuntimeWorldInstant};

/// Installed cancellation and deadline for the last reversible product boundary.
pub(crate) struct ProductMovementCutoff {
    cancellation: RuntimeWorldCancellationToken,
    clock: RuntimeWorldClock,
    deadline: Option<RuntimeWorldInstant>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProductMovementCutoffDenial {
    Cancelled,
    Deadline,
}
impl ProductMovementCutoff {
    pub(crate) fn new(
        cancellation: &RuntimeWorldCancellationToken,
        clock: &RuntimeWorldClock,
        deadline: Option<RuntimeWorldInstant>,
    ) -> Self {
        Self {
            cancellation: cancellation.clone(),
            clock: clock.clone(),
            deadline,
        }
    }
    pub(crate) fn check(&self) -> Result<(), ProductMovementCutoffDenial> {
        self.check_at(self.deadline.map(|_| self.clock.now()))
    }

    pub(crate) fn check_memory_cutover(
        &self,
        narrowing_deadline: Option<RuntimeWorldInstant>,
    ) -> Result<(), NoEffectCause> {
        let now = self
            .deadline
            .or(narrowing_deadline)
            .map(|_| self.clock.now());
        self.check_at(now).map_err(|denial| match denial {
            ProductMovementCutoffDenial::Cancelled => NoEffectCause::CancelledBeforeEffect,
            ProductMovementCutoffDenial::Deadline => NoEffectCause::DeadlineBeforeEffect,
        })?;
        if now
            .zip(narrowing_deadline)
            .is_some_and(|(now, deadline)| now.deadline_elapsed(deadline))
        {
            return Err(NoEffectCause::CurrentReadDependency(
                CurrentProductReadDependencyDenial::CutoverDeadlineElapsed,
            ));
        }
        Ok(())
    }

    fn check_at(
        &self,
        now: Option<RuntimeWorldInstant>,
    ) -> Result<(), ProductMovementCutoffDenial> {
        // This atomic sample is the final cancellation cutoff. Clock work is
        // still reversible and cannot turn an earlier cancellation into late evidence.
        if self.cancellation.is_cancelled() {
            return Err(ProductMovementCutoffDenial::Cancelled);
        }
        if now
            .zip(self.deadline)
            .is_some_and(|(now, deadline)| now.deadline_elapsed(deadline))
        {
            return Err(ProductMovementCutoffDenial::Deadline);
        }
        Ok(())
    }

    pub(crate) fn cancellation_after_movement(&self) -> bool {
        self.cancellation.is_cancelled()
    }
}
