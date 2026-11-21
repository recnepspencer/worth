//! Test doors exposing the existing two halves of `checkpoint` unchanged.
//!
//! `DurabilityAuthority::checkpoint` captures one recovery basis, then
//! finalizes it. These doors let one exact certification court complete an
//! older capture after a newer checkpoint compacted its transitions. They add
//! no state, ordering or semantics: each calls the same private half that
//! `checkpoint` calls. No production code reaches them; their caller row is
//! in `tools/boundary-check/config/control-callers.txt`.
use super::checkpoint_capture::CapturedCheckpointBasis;
use super::DurabilityAuthority;
use crate::durability::data::{DurabilityError, DurableCheckpoint, RelationalRecoveryStop};

/// One captured, not yet finalized recovery basis from the capture door.
#[doc(hidden)]
pub struct CapturedRelationalCheckpointBasisForTest(CapturedCheckpointBasis);

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
