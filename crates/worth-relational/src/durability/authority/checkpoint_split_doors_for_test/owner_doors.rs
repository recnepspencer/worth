use super::CapturedRelationalCheckpointBasisForTest;
use crate::durability::authority::runtime_access::DurabilityAuthority;
use crate::durability::data::{DurabilityError, DurableCheckpoint, RelationalRecoveryStop};

impl DurabilityAuthority<'_> {
    /// The capture half of `checkpoint`, exactly as `checkpoint` performs it.
    #[doc(hidden)]
    pub fn capture_checkpoint_basis_for_test(
        &self,
    ) -> Result<CapturedRelationalCheckpointBasisForTest, DurabilityError> {
        self.capture_checkpoint_basis()
            .map(CapturedRelationalCheckpointBasisForTest)
    }

    /// The finalize half of `checkpoint`, exactly as `checkpoint` performs it.
    #[doc(hidden)]
    pub fn finalize_captured_checkpoint_for_test(
        &self,
        captured: CapturedRelationalCheckpointBasisForTest,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<DurableCheckpoint, RelationalRecoveryStop> {
        self.finalize_captured_checkpoint(captured.0, execution)
    }
}
