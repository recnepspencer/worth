use super::RuntimeWorldCancellationToken;
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
        let expired = self
            .deadline
            .is_some_and(|deadline| self.clock.now() >= deadline);
        // This atomic sample is the final cancellation cutoff. Clock work is
        // still reversible and cannot turn an earlier cancellation into late evidence.
        if self.cancellation.is_cancelled() {
            return Err(ProductMovementCutoffDenial::Cancelled);
        }
        if expired {
            return Err(ProductMovementCutoffDenial::Deadline);
        }
        Ok(())
    }
    pub(crate) fn cancellation_after_movement(&self) -> bool {
        self.cancellation.is_cancelled()
    }
}
