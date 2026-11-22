//! Sealed test doors over this module's private members.
use super::*;

impl OriginalProjection {
    #[doc(hidden)]
    pub fn allocation_bytes_for_test() -> u64 {
        Self::allocation_bytes()
    }
}

impl OriginalProjection {
    #[doc(hidden)]
    pub fn _memory_for_test(&self) -> &Option<worth_execution::ExecutionMemoryReservation> {
        &self._memory
    }
}

impl OriginalProjection {
    #[doc(hidden)]
    pub fn rows_for_test(&self) -> &OwnedProjectionRows {
        &self.rows
    }
}
