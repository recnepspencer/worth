//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationCheckpoint {
    #[doc(hidden)]
    pub fn decode_current_body_for_test(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<DecodedApplicationCheckpoint, CheckpointDecodeDenial> {
        Self::decode_current_body(self, execution)
    }
}
