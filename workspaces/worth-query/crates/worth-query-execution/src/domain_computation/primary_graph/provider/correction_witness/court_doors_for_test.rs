//! Sealed test doors over this module's private members.
use super::*;

impl CorrectionPerformedWitness {
    #[doc(hidden)]
    pub fn _memory_for_test(&self) -> &Option<ExecutionMemoryReservation> {
        &self._memory
    }
}
