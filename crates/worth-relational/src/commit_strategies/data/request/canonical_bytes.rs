//! Cold input bytes retain their original grant without an ordinary allocation.
use super::CanonicalStrategyInputArtifact;
use std::sync::Arc;
use worth_execution::ExecutionMemoryReservation;

#[cfg(test)]
mod tests;

impl CanonicalStrategyInputArtifact {
    pub(crate) fn canonical_bytes_control_bytes() -> Option<u64> {
        let layout = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<ExecutionMemoryReservation>())
            .ok()?
            .0
            .pad_to_align();
        u64::try_from(layout.size()).ok()
    }

    /// The original data must still be exclusively owned before this cold
    /// handoff. Ordinary construction stays unbudgeted with no grant Arc.
    pub(crate) fn retain_canonical_bytes_memory(
        &mut self,
        memory: Option<ExecutionMemoryReservation>,
    ) -> Result<(), Option<ExecutionMemoryReservation>> {
        let Some(memory) = memory else {
            return Ok(());
        };
        if self.canonical_bytes_memory.is_some()
            || Arc::get_mut(&mut self.canonical_bytes).is_none()
        {
            return Err(Some(memory));
        }
        self.canonical_bytes_memory = Some(Arc::new(memory));
        Ok(())
    }
}

impl PartialEq for CanonicalStrategyInputArtifact {
    fn eq(&self, other: &Self) -> bool {
        self.schema_name == other.schema_name
            && self.schema_version == other.schema_version
            && self.canonical_bytes == other.canonical_bytes
            && self.digest == other.digest
            && self.artifact_name == other.artifact_name
    }
}
impl Eq for CanonicalStrategyInputArtifact {}

impl std::fmt::Debug for CanonicalStrategyInputArtifact {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CanonicalStrategyInputArtifact")
            .field("schema_name", &self.schema_name)
            .field("schema_version", &self.schema_version)
            .field("canonical_bytes", &self.canonical_bytes)
            .field("digest", &self.digest)
            .field("artifact_name", &self.artifact_name)
            .finish()
    }
}
