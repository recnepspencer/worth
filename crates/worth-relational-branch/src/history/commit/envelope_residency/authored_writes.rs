//! The commit cell owns its discardable authored-history accelerator, even cold.
use super::RelationalCommitEnvelopeResidency;
use crate::history::access::relevant_writes::generation::AuthoredWriteGeneration;

impl RelationalCommitEnvelopeResidency {
    pub fn authored_write_generation(&self) -> Option<AuthoredWriteGeneration> {
        self.state
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .authored_writes
            .clone()
    }

    pub fn replace_authored_write_generation(&self, generation: Option<AuthoredWriteGeneration>) {
        self.state
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .authored_writes = generation;
    }
}
