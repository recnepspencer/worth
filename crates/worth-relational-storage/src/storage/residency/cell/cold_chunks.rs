//! Cold chunk reads of a readmitted cell's recorded placement.
use super::*;
use crate::storage::residency::chunked::region_reader::RecordedRegionChunks;

impl RelationalRegionResidency {
    /// Opens this cold-backed cell's recorded chunk placement for bounded cold
    /// reads. Chunk members fault through the cell's working-set pool from the
    /// cell's own retained member custody, never by materializing the region.
    /// Refuses unbacked or unbound cells and cells whose source has retired.
    pub fn recorded_chunks(&self) -> Result<RecordedRegionChunks, RelationalStorageUnavailable> {
        let unavailable = |cause| RelationalStorageUnavailable {
            subject: RelationalStorageSubject::Region(self.partition_id),
            cause,
        };
        let cold = self
            .backing
            .as_ref()
            .ok_or_else(|| unavailable(Cause::SourceUnavailable))?;
        let binding = self
            .working_set
            .get()
            .ok_or_else(|| unavailable(Cause::SourceUnavailable))?;
        let custody = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .physical
            .as_ref()
            .filter(|physical| physical.source().is_some())
            .map(|physical| physical.retain_serving())
            .ok_or_else(|| unavailable(Cause::SourceUnavailable))?;
        RecordedRegionChunks::open(&cold.chunks, custody, &binding.accountant, binding.pool)
            .map_err(unavailable)
    }
}
