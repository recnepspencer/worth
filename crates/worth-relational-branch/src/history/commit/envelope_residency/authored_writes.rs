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
        if !span_widens(state.authored_writes.as_ref(), original) {
            return false;
        }
        state.authored_writes = Some(generation);
        true
    }
}

/// A seed replaces the current generation only when that generation lacks
/// `original` and is not a merge marker. Single-parent prefixes ending at the
/// same target are totally ordered, so lacking `original` means narrower.
fn span_widens(
    current: Option<&AuthoredWriteGeneration>,
    original: crate::history::data::CommitId,
) -> bool {
    current
        .is_none_or(|current| !current.unsupported_merge && current.prefix.get(&original).is_none())
}

#[cfg(test)]
#[path = "authored_writes_tests.rs"]
mod tests;
