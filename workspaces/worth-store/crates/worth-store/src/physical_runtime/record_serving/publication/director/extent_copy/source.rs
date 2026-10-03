use super::super::{selected_segment_rewrite::damaged, RecordPublicationDirector};
use crate::physical_runtime::record_serving::access::extent_rewrite_source::{
    open_extent_rewrite_source, ExtentRewriteCursor, ExtentRewriteSourceRequest,
};
use crate::physical_runtime::record_serving::{RecordAppendDenial, RecordAppendError};
use worth_store_physical_format::DurableExtentRecordPlacement;

pub(super) fn source_failure(
    failure: crate::physical_runtime::record_serving::RecordStreamFailure,
) -> RecordAppendError {
    RecordAppendError::StreamFailed(failure)
}

impl RecordPublicationDirector {
    pub(super) fn copy_source_cursor(
        &self,
        source: DurableExtentRecordPlacement,
        allocation: &worth_store_buffer_pool::OperationAllocationGrant,
    ) -> Result<ExtentRewriteCursor, RecordAppendError> {
        open_extent_rewrite_source(
            ExtentRewriteSourceRequest {
                residency: self.residency.clone(),
                store: self.durability.store_identity(),
                format: self.format,
                generation: self.generation,
                allocation,
            },
            source,
        )
        .map_err(|_| damaged())
    }
    pub(in crate::physical_runtime::record_serving::publication::director) fn copy_allocation(
        &self,
    ) -> Result<worth_store_buffer_pool::ForegroundWriteAllocationGrant, RecordAppendError> {
        // Source residency plus destination encoding and one fixed manifest.
        let bytes = u64::from(self.format.declaration().page_size().bytes())
            .checked_mul(3)
            .and_then(std::num::NonZeroU64::new)
            .ok_or_else(damaged)?;
        self.residency
            .begin_foreground_write_operation(bytes)
            .map_err(|denial| RecordAppendError::Denied(RecordAppendDenial::from_residency(denial)))
    }
}
