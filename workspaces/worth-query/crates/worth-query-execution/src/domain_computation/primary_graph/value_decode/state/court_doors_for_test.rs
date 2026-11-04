//! Sealed test doors over this module's private members.
use super::*;

impl ValueDecodeState {
    #[doc(hidden)]
    pub fn memory_for_test(&self) -> &RefCell<Option<ExecutionMemoryReservation>> {
        &self.memory
    }
}

impl ValueDecodeState {
    #[doc(hidden)]
    pub fn denial_for_test(&self) -> Option<Stop> {
        Self::denial(self)
    }
}

impl ValueDecodeState {
    #[doc(hidden)]
    pub fn run_for_test<R>(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        body: impl FnOnce(&ValueDecodeSession<'_>) -> R,
    ) -> Result<R, Stop> {
        Self::run(self, execution, body)
    }
}
