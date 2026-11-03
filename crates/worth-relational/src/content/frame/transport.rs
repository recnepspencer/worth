//! Controlled transport admission preserves the original byte reservation.
use super::{control, integrity, layout, record::FrameBacking, RelationalContentRecord};
use crate::content::{RelationalContentDenial as Denial, MAXIMUM_CONTENT_FRAME_BYTES};
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::ContentRefId;

impl RelationalContentRecord {
    pub(crate) fn readmit_transport(
        execution: &ExecutionRequest<'_, '_>,
        identity: ContentRefId,
        frame: &[u8],
    ) -> Result<Self, Denial> {
        control::run(execution, |control| {
            layout::admit_length(frame.len(), MAXIMUM_CONTENT_FRAME_BYTES)?;
            layout::inspect_paid(frame, control)?;
            let memory = execution
                .reserve_retained_memory(
                    (frame.len() + size_of::<FrameBacking>() + 2 * size_of::<usize>()) as u64,
                )
                .map_err(|cause| Denial::Execution(cause.into()))?;
            if integrity::digest(frame, control)? != identity {
                return Err(Denial::IntegrityMismatch);
            }
            let mut bytes = Vec::new();
            bytes.try_reserve_exact(frame.len()).map_err(|_| {
                Denial::Execution(
                    crate::execution::RelationalExecutionDenialCause::AllocationFailed,
                )
            })?;
            for chunk in frame.chunks(4096) {
                control.checkpoint(chunk.len() as u64)?;
                bytes.extend_from_slice(chunk);
            }
            Ok(Self {
                identity,
                backing: Arc::new(FrameBacking { bytes, memory }),
            })
        })
    }
}
