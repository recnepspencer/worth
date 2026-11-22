//! Sealed test door: what a completed record offers hot coupled adoption.
use super::*;

impl RecordedOutput {
    /// Whether the record keeps comparable source facts, and whether its
    /// completed computation sealed a Native output witness. `None` when the
    /// record has no completed computation. Read-only; it decides nothing.
    #[doc(hidden)]
    pub fn adoption_basis_for_test(&self) -> Option<(bool, bool)> {
        let completed = self.completed_computation_record()?;
        let facts = self
            .observed_source_facts()
            .and_then(|facts| facts.for_comparison())
            .is_some_and(|facts| !facts.is_empty());
        Some((facts, completed.native_output_witness().is_some()))
    }
}
