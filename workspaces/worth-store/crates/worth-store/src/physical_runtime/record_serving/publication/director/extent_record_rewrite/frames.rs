use super::super::selected_segment_rewrite::damaged;
use super::super::RecordPublicationDirector;
use crate::physical_runtime::durability::PreparedPhysicalDataFrame;
use crate::physical_runtime::record_serving::RecordAppendError;
use crate::physical_runtime::{CertifiedPriorPageBasis, PhysicalDataFrameIdentity};
use worth_store_physical_format::{
    prepare_extent_chunk, DurableExtentManifest, ExtentArenaFrameLayout, ExtentArenaRange,
    ExtentChunkFrame, RecordArtifactFile,
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
        let mut frames = Vec::with_capacity(manifest.chunk_count() as usize);
        let mut completed = 0_usize;
        let layout =
            ExtentArenaFrameLayout::new(format, manifest.alignment()).ok_or_else(damaged)?;
        for ordinal in 1..=manifest.chunk_count() {
            let framed = ExtentChunkFrame::of(*manifest, layout, ordinal)
                .filter(|framed| framed.coordinate().logical_offset() == completed as u64)
                .ok_or_else(damaged)?;
            let expected = framed.payload_bytes() as usize;
            let source = payload
                .get(completed..completed + expected)
                .ok_or_else(damaged)?;
            let mut chunk = prepare_extent_chunk(format, framed.coordinate(), expected)
                .map_err(|_| damaged())?;
            chunk.payload_mut().copy_from_slice(source);
            let bytes = chunk.seal();
            if bytes.len() != framed.length() as usize {
                return Err(damaged());
            }
            let artifact_offset = destination_range
                .offset()
                .checked_add(framed.offset())
                .ok_or_else(damaged)?;
            let target = PhysicalDataFrameIdentity::extent_chunk(
                framed.coordinate(),
                destination,
                artifact_offset,
                framed.length(),
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
        }
        if completed != payload.len() {
            return Err(damaged());
        }
        Ok((frames, destination_range.length()))
    }
}
