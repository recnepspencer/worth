//! Sealed test doors over this module's private members.
use super::*;

impl<Schema, Operation> WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation> {
    #[doc(hidden)]
    pub fn attributed_for_test<Output>(
        &mut self, read: ComputationRead, call: impl FnOnce(&mut Self) -> Output,
    ) -> Output {
        Self::attributed(self, read, call)
    }
}

impl<Schema, Operation> WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation> {
    #[doc(hidden)]
    pub fn begin_computation_reads_for_test(&mut self) {
        Self::begin_computation_reads(self)
    }
}
