use super::super::selected_segment_rewrite::damaged;
use super::super::RecordPublicationDirector;
use crate::physical_runtime::durability::PreparedPhysicalDataFrame;
use crate::physical_runtime::record_serving::RecordAppendError;
use crate::physical_runtime::{CertifiedPriorPageBasis, PhysicalDataFrameIdentity};
use worth_store_physical_format::{
    prepare_extent_chunk, DurableExtentManifest, ExtentArenaFrameLayout, ExtentArenaRange,
    ExtentChunkCoordinate, RecordArtifactFile,
};

impl RecordPublicationDirector {
    pub(super) fn encode_extent_frames(
        &self,
        manifest: &DurableExtentManifest,
        destination: RecordArtifactFile,
        destination_range: ExtentArenaRange,
        payload: &[u8],
    ) -> Result<(Vec<PreparedPhysicalDataFrame>, u64), RecordAppendError> {
        let format = self.format.declaration();
        let transfer = manifest.chunk_payload_capacity() as usize;
        let mut frames = Vec::with_capacity(manifest.chunk_count() as usize);
        let mut completed = 0_usize;
        let layout =
            ExtentArenaFrameLayout::new(format, manifest.alignment()).ok_or_else(damaged)?;
        let mut artifact_offset = destination_range.offset() + layout.manifest_stride();
        for ordinal in 1..=manifest.chunk_count() {
            let expected = (payload.len() - completed).min(transfer);
            let coordinate = ExtentChunkCoordinate::new(
                manifest.record(),
                manifest.extent_cell(),
                manifest.logical_bytes(),
                completed as u64,
                ordinal,
            )
            .ok_or_else(damaged)?;
            let mut chunk =
                prepare_extent_chunk(format, coordinate, expected).map_err(|_| damaged())?;
            chunk
                .payload_mut()
                .copy_from_slice(&payload[completed..completed + expected]);
            let bytes = chunk.seal();
            let length = u32::try_from(bytes.len()).map_err(|_| damaged())?;
            let target = PhysicalDataFrameIdentity::extent_chunk(
                coordinate,
                destination,
                artifact_offset,
                length,
                destination_range,
            )
            .ok_or_else(damaged)?;
            frames.push(
                PreparedPhysicalDataFrame::new(
                    target,
                    CertifiedPriorPageBasis::for_unmaterialized_target(target),
                    vec![0],
                    bytes,
                    format,
                )
                .map_err(|_| damaged())?,
            );
            completed += expected;
            artifact_offset = artifact_offset
                .checked_add(layout.chunk_stride())
                .ok_or_else(damaged)?;
        }
        if completed != payload.len() {
            return Err(damaged());
        }
        Ok((frames, destination_range.length()))
    }
}
