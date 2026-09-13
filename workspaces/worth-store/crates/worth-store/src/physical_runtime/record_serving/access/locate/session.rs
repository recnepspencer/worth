use super::{ReadPlacement, RecordReadSession};
use crate::physical_runtime::{
    PhysicalRecordChunkView, RecordReadObservation, RecordStreamFailure, RecordStreamFailureKind,
};

impl RecordReadSession {
    /// Copies the next payload bytes into `target`.
    ///
    /// Returns zero at end of record or when `target` is empty. The method
    /// never allocates an owning whole-record result.
    pub fn read_next(&mut self, target: &mut [u8]) -> Result<usize, RecordStreamFailure> {
        let runtime = self.require_healthy_runtime()?;
        let identity = self.identity;
        if target.is_empty() {
            return Ok(0);
        }
        let count = match &mut self.placement {
            ReadPlacement::Inline {
                frame,
                payload,
                offset,
            } => {
                let count = target.len().min(payload.len().saturating_sub(*offset));
                let start = payload.start + *offset;
                frame.copy_range_into(start..start + count, &mut target[..count]);
                *offset += count;
                count
            }
            ReadPlacement::Extent(state) => {
                match state.read_next(&self._allocation, target, &mut self.observation, identity) {
                    Ok(count) => count,
                    Err(failure) => {
                        runtime.health.observe_stream_failure(failure.kind());
                        return Err(failure);
                    }
                }
            }
        };
        self.observation.observe_copy(count);
        self.observation.payload_bytes =
            self.observation.payload_bytes.saturating_add(count as u64);
        Ok(count)
    }

    /// Borrows the next decoded payload chunk from the current resident frame.
    ///
    /// The returned view mutably borrows this session, so the session cannot
    /// advance or be dropped while the view or its byte slice remains live.
    pub fn next_chunk(
        &mut self,
    ) -> Result<Option<PhysicalRecordChunkView<'_>>, RecordStreamFailure> {
        let runtime = self.require_healthy_runtime()?;
        let identity = self.identity;
        let chunk = match &mut self.placement {
            ReadPlacement::Inline {
                frame,
                payload,
                offset,
            } => {
                if *offset == payload.len() {
                    None
                } else {
                    let logical_start = *offset as u64;
                    let start = payload.start + *offset;
                    let end = payload.end;
                    *offset = payload.len();
                    Some((
                        &frame[start..end],
                        frame.coordinate(),
                        logical_start..payload.len() as u64,
                    ))
                }
            }
            ReadPlacement::Extent(state) => {
                match state.next_chunk(&self._allocation, &mut self.observation, identity) {
                    Ok(Some(chunk)) => Some((chunk.bytes, chunk.frame, chunk.logical_range)),
                    Ok(None) => None,
                    Err(failure) => {
                        runtime.health.observe_stream_failure(failure.kind());
                        return Err(failure);
                    }
                }
            }
        };
        let Some((bytes, frame, logical_range)) = chunk else {
            return Ok(None);
        };
        self.observation.payload_bytes = self
            .observation
            .payload_bytes
            .saturating_add(bytes.len() as u64);
        Ok(Some(self.identity.chunk_view(bytes, frame, logical_range)))
    }

    /// Returns the read progress and physical work observed by this session.
    pub const fn observation(&self) -> RecordReadObservation {
        self.observation
    }

    fn require_healthy_runtime(
        &self,
    ) -> Result<
        std::sync::Arc<crate::physical_runtime::instance::PhysicalStoreWorkRuntime>,
        RecordStreamFailure,
    > {
        let runtime = self.runtime.upgrade().ok_or_else(|| {
            RecordStreamFailure::during_read(
                RecordStreamFailureKind::RuntimeReleased,
                self.observation.payload_bytes(),
            )
        })?;
        if runtime.health.require(self.health_permit).is_err() {
            return Err(RecordStreamFailure::during_read(
                RecordStreamFailureKind::ServingRequiresInspection,
                self.observation.payload_bytes(),
            ));
        }
        Ok(runtime)
    }
}
