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

    /// Install a span seed only when the current generation lacks `original`.
    /// One lock covers the check and the install, so a narrower seed never
    /// replaces a wider one. Returns whether the seed was installed.
    pub fn install_authored_write_span(
        &self,
        original: crate::history::data::CommitId,
        generation: AuthoredWriteGeneration,
    ) -> bool {
        let mut state = self.state.lock().unwrap_or_else(|p| p.into_inner());
        if state
            .authored_writes
            .as_ref()
            .is_some_and(|current| current.prefix.get(&original).is_some())
        {
            return false;
        }
        state.authored_writes = Some(generation);
        true
    }
}

#[cfg(test)]
#[path = "authored_writes_tests.rs"]
mod tests;
