//! The commit cell owns its discardable authored-history accelerator, even cold.
use super::RelationalCommitEnvelopeResidency;
use crate::runtime::AuthoredWriteGeneration;

impl RelationalCommitEnvelopeResidency {
    pub(crate) fn authored_write_generation(&self) -> Option<AuthoredWriteGeneration> {
        self.state
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .authored_writes
            .clone()
    }

    pub(crate) fn replace_authored_write_generation(
        &self,
        generation: Option<AuthoredWriteGeneration>,
    ) {
        self.state
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .authored_writes = generation;
    }
}
