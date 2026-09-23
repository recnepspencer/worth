use std::num::NonZeroU64;

use sha2::{Digest, Sha256};

use super::RecordPublicationDirector;
use crate::physical_runtime::record_serving::planning::published_segment_reuse::{
    load_published_segment, ReusableSegmentContext,
};
use crate::physical_runtime::record_serving::planning::reusable_inline_tail::last_inline_placement;
use crate::physical_runtime::record_serving::{RecordAppendDenial, RecordAppendError};

pub(super) fn rewrite_basis_digest(
    record: worth_store_physical_format::PersistedRecordIdentity,
    pages: u32,
    span_start: u32,
) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(b"store.physical.rewrite-basis.v3");
    digest.update(record.allocation_epoch());
    digest.update(record.ordinal().to_le_bytes());
    digest.update(pages.to_le_bytes());
    digest.update(span_start.to_le_bytes());
    digest.finalize().into()
}

impl RecordPublicationDirector {
    pub(super) fn selected_rewrite_span_start(
        &self,
        placement: crate::physical_runtime::AdmittedRecordPlacementPolicy,
        pages: u32,
    ) -> Result<u32, RecordAppendError> {
        let page_bytes = u64::from(self.format.declaration().page_size().bytes());
        let allocation = self
            .residency
            .begin_foreground_write_operation(
                NonZeroU64::new(page_bytes * u64::from(pages)).ok_or_else(damaged)?,
            )
            .map_err(|denial| {
                RecordAppendError::Denied(RecordAppendDenial::from_residency(denial))
            })?;
        let (current_root, current_free_space) = self.root_owner.snapshot();
        let last = last_inline_placement(
            &allocation,
            self.residency.clone(),
            self.format,
            self.access,
            &current_root,
            placement,
        )?
        .ok_or_else(damaged)?;
        let (segment, _) = load_published_segment(
            ReusableSegmentContext {
                allocation: &allocation,
                residency: self.residency.clone(),
                format: self.format,
                access: self.access,
                current_root: &current_root,
                current_free_space: &current_free_space,
                placement,
            },
            Some(last),
        )?;
        let segment = segment.ok_or_else(damaged)?;
        let page_entry = segment.last_published_page.ok_or_else(damaged)?;
        let tail_bound = page_entry
            .frame_index()
            .checked_add(1)
            .ok_or_else(damaged)?;
        let start = tail_bound.checked_sub(pages).ok_or_else(span_not_live)?;
        self.rewrite_source_liveness(
            &allocation,
            &current_root,
            segment.segment.segment_id(),
            page_entry.data_generation(),
            start..tail_bound,
        )?
        .displaces_source()?;
        Ok(start)
    }
}

fn damaged() -> RecordAppendError {
    RecordAppendError::Denied(RecordAppendDenial::PublishedLayoutDamaged)
}

fn span_not_live() -> RecordAppendError {
    RecordAppendError::Denied(RecordAppendDenial::RewriteSpanNotLive)
}
