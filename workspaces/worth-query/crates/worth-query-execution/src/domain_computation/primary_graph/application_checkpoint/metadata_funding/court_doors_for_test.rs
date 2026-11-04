//! Sealed test doors over this module's private members.
use super::*;

impl CheckpointMetadataFunding {
    #[doc(hidden)]
    pub fn bytes_for_test(
        &mut self,
        bytes: usize,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<(), CheckpointDecodeDenial> {
        Self::bytes(self, bytes, execution)
    }
}

impl CheckpointMetadataFunding {
    #[doc(hidden)]
    pub fn slots_for_test<T>(
        &mut self,
        count: usize,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<(), CheckpointDecodeDenial> {
        Self::slots::<T>(self, count, execution)
    }
}
