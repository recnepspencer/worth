use super::super::selected_segment_rewrite::damaged;
use super::super::RecordPublicationDirector;
use crate::physical_runtime::record_serving::access::manifest_routing::{
    ManifestDiscoveryCounterSnapshot, ManifestReader,
};
use crate::physical_runtime::record_serving::planning::inline_plan_failure::manifest_lookup_failure;
use crate::physical_runtime::record_serving::{RecordAppendDenial, RecordAppendError};
use sha2::{Digest, Sha256};
use std::num::NonZeroU64;
use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DurableExtentRecordPlacement, DurablePhysicalRootManifest,
    PersistedRecordIdentity,
};

impl RecordPublicationDirector {
    pub(in crate::physical_runtime::record_serving::publication::director) fn current_extent_source(
        &self,
        root: &DurablePhysicalRootManifest,
        record: PersistedRecordIdentity,
    ) -> Result<DurableExtentRecordPlacement, RecordAppendError> {
        let page_bytes = u64::from(self.format.declaration().page_size().bytes());
        let allocation = self
            .residency
            .begin_foreground_write_operation(NonZeroU64::new(page_bytes).ok_or_else(damaged)?)
            .map_err(|denial| {
                RecordAppendError::Denied(RecordAppendDenial::from_residency(denial))
            })?;
        let located = ManifestReader::serving(
            self.residency.clone(),
            self.format,
            self.access,
            root.clone(),
        )
        .locate(
            &allocation,
            record,
            &mut ManifestDiscoveryCounterSnapshot::default(),
        )
        .map_err(manifest_lookup_failure)?;
        match located {
            Some(CurrentPhysicalRecordPlacement::Extent(source)) => Ok(source),
            _ => Err(RecordAppendError::Denied(
                RecordAppendDenial::RewriteSpanNotLive,
            )),
        }
    }
}

pub(super) fn extent_rewrite_basis_digest(source: DurableExtentRecordPlacement) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(b"store.physical.extent-rewrite-basis.v2");
    digest.update(source.record().allocation_epoch());
    digest.update(source.record().ordinal().to_le_bytes());
    digest.update(source.extent().get().to_le_bytes());
    digest.update(source.extent_generation().to_le_bytes());
    digest.update(source.payload_bytes().to_le_bytes());
    digest.update(source.arena_range().arena().get().to_le_bytes());
    digest.update(source.arena_range().offset().to_le_bytes());
    digest.update(source.arena_range().length().to_le_bytes());
    digest.finalize().into()
}
