//! The retained Native contract must equal the expected application contract.
use super::RelationalContentFrame;
use crate::content::{control, RelationalContentDenial as Denial};
use worth_execution::ExecutionRequest;
impl RelationalContentFrame {
    /// Bounded metadata equality under the original request; no codec is rerun.
    pub fn require_contract(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        name: &str,
        version: u32,
        maximum_frame_bytes: usize,
    ) -> Result<(), Denial> {
        control::run(execution, |control| {
            control.checkpoint(1)?;
            if name.len() > 256 {
                return Err(Denial::InvalidFormat);
            }
            control.checkpoint((name.len() + self.format().name().len() + 16) as u64)?;
            super::layout::admit_length(self.bytes().len(), maximum_frame_bytes)?;
            if self.format().name() != name {
                return Err(Denial::FormatMismatch);
            }
            if self.format().version() != version {
                return Err(Denial::UnsupportedVersion);
            }
            if self.0.format.maximum_frame_bytes() != maximum_frame_bytes {
                return Err(Denial::FormatMismatch);
            }
            Ok(())
        })
    }
}
