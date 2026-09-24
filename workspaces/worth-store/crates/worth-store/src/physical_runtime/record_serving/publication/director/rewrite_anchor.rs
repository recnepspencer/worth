use worth_store_physical_format::{DurableInlineRecordPlacement, DurablePhysicalRootManifest};

use super::RecordPublicationDirector;
use crate::physical_runtime::record_serving::planning::reusable_inline_tail::last_inline_placement;
use crate::physical_runtime::record_serving::{RecordAppendDenial, RecordAppendError};
use crate::physical_runtime::PreparedPhysicalMutation;

pub(super) fn resolve(
    director: &RecordPublicationDirector,
    prepared: &PreparedPhysicalMutation,
    allocation: &worth_store_buffer_pool::OperationAllocationGrant,
    current_root: &DurablePhysicalRootManifest,
) -> Result<DurableInlineRecordPlacement, RecordAppendError> {
    if let Some(anchor) = prepared.rewrite_anchor() {
        return (anchor.segment_page_capacity() == prepared.placement().segment_pages().get())
            .then_some(anchor)
            .ok_or(RecordAppendError::Denied(
                RecordAppendDenial::PublishedLayoutDamaged,
            ));
    }
    last_inline_placement(
        allocation,
        director.residency.clone(),
        director.format,
        director.access,
        current_root,
        prepared.placement(),
    )?
    .ok_or(RecordAppendError::Denied(
        RecordAppendDenial::PublishedLayoutDamaged,
    ))
}
