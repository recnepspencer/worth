use super::super::CustomInvariantInterruption;
use super::{WorthQueryApplicationInvariantContext, WorthQueryApplicationInvariantScopePlanner};

impl<Schema> WorthQueryApplicationInvariantContext<'_, '_, Schema> {
    /// Observes the original request's cancellation and deadline without work accounting.
    pub fn check_live(&self) -> Result<(), CustomInvariantInterruption> {
        self.lower.check_live()
    }
}

impl<Schema> WorthQueryApplicationInvariantScopePlanner<'_, '_, Schema> {
    /// Observes the original request during scope preparation without work accounting.
    pub fn check_live(&self) -> Result<(), CustomInvariantInterruption> {
        self.lower.check_live()
    }
}
