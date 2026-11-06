//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationInspectionDenial {
    #[doc(hidden)]
    pub fn cause_for_test(&self) -> &InspectionCause {
        &self.cause
    }
}

impl WorthQueryApplicationInspectionDenial {
    #[doc(hidden)]
    pub fn new_for_test(
        cause: InspectionCause,
        memory: Option<ExecutionMemoryReservation>,
    ) -> Self {
        Self::new(cause, memory)
    }
}
