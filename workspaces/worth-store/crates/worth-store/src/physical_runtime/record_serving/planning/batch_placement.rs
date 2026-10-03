use worth_store_physical_format::{
    BlobRecordKind, PersistedRecordIdentity, SelectedRecordContentClass,
    SelectedRecordRouteMetadata, DURABLE_FRAME_HEADER_BYTES, DURABLE_INLINE_PAGE_PREFIX_BYTES,
    DURABLE_INLINE_SLOT_BYTES,
};

use super::super::{
    identity::allocate_candidate_record_identities,
    publication::batch::{AdmittedRecordAppendBatch, RecordAppendBatch, RecordAppendInput},
    publication::streaming::OwnedRecordSource,
    AdmittedPhysicalRecordFormat, AdmittedRecordPlacementPolicy, RecordAppendDenial,
    RecordAppendError, RecordPlacementClass, RecordStreamFailure, RecordStreamFailureKind,
    RecordWriteSource,
};

pub(in crate::physical_runtime::record_serving) struct ExtentInput {
    pub(in crate::physical_runtime::record_serving) record: PersistedRecordIdentity,
    pub(in crate::physical_runtime::record_serving) source: Box<dyn RecordWriteSource>,
    pub(in crate::physical_runtime::record_serving) length: u64,
    pub(in crate::physical_runtime::record_serving) route_metadata: SelectedRecordRouteMetadata,
}

pub(in crate::physical_runtime::record_serving) struct PendingInlineInput {
    pub(in crate::physical_runtime::record_serving) record: PersistedRecordIdentity,
    input: RecordAppendInput,
    pub(in crate::physical_runtime::record_serving) length: u64,
    pub(in crate::physical_runtime::record_serving) route_metadata: SelectedRecordRouteMetadata,
}

pub(in crate::physical_runtime::record_serving) struct MaterializedInlineInput {
    pub(in crate::physical_runtime::record_serving) record: PersistedRecordIdentity,
    pub(in crate::physical_runtime::record_serving) bytes: Vec<u8>,
    pub(in crate::physical_runtime::record_serving) route_metadata: SelectedRecordRouteMetadata,
}

pub(in crate::physical_runtime::record_serving) struct MaterializedInlineBatch {
    pub(in crate::physical_runtime::record_serving) records: Vec<MaterializedInlineInput>,
    pub(in crate::physical_runtime::record_serving) copy_count: u64,
    pub(in crate::physical_runtime::record_serving) copied_bytes: u64,
}

pub(in crate::physical_runtime::record_serving) struct ClassifiedBatch {
    pub(in crate::physical_runtime::record_serving) identities: Vec<PersistedRecordIdentity>,
    pub(in crate::physical_runtime::record_serving) inline: Vec<PendingInlineInput>,
    pub(in crate::physical_runtime::record_serving) extents: Vec<ExtentInput>,
    pub(in crate::physical_runtime::record_serving) logical_bytes: u64,
}

#[derive(Debug, Clone, Copy)]
pub(in crate::physical_runtime::record_serving) struct PhysicalIdentityReservationShape {
    segments: u64,
    pages: u64,
    extents: u64,
}

impl ClassifiedBatch {
    pub(in crate::physical_runtime::record_serving) fn identity_reservation_shape(
        &self,
    ) -> Result<PhysicalIdentityReservationShape, RecordAppendError> {
        let inline = u64::try_from(self.inline.len()).map_err(|_| {
            RecordAppendError::Denied(RecordAppendDenial::PhysicalIdentityExhausted)
        })?;
        let extents = u64::try_from(self.extents.len()).map_err(|_| {
            RecordAppendError::Denied(RecordAppendDenial::PhysicalIdentityExhausted)
        })?;
        Ok(PhysicalIdentityReservationShape {
            segments: inline,
            pages: inline,
            extents,
        })
    }
}

impl PhysicalIdentityReservationShape {
    pub(in crate::physical_runtime::record_serving) const fn segments(self) -> u64 {
        self.segments
    }
    pub(in crate::physical_runtime::record_serving) const fn pages(self) -> u64 {
        self.pages
    }
    pub(in crate::physical_runtime::record_serving) const fn extents(self) -> u64 {
        self.extents
    }
}

pub(in crate::physical_runtime::record_serving) fn classify_batch(
    manifest: &super::super::access::manifest_routing::ManifestReader<'_>,
    allocation: &worth_store_buffer_pool::OperationAllocationGrant,
    placement: AdmittedRecordPlacementPolicy,
    blob_record_kind: Option<BlobRecordKind>,
    selected_content_class: SelectedRecordContentClass,
    released_directory_record: bool,
    inline_only: bool,
    batch: AdmittedRecordAppendBatch,
) -> Result<ClassifiedBatch, RecordAppendError> {
    let identities =
        allocate_candidate_record_identities(batch.records.len(), manifest, allocation)
            .map_err(RecordAppendError::Denied)?;
    let mut inline = Vec::new();
    let mut extents = Vec::new();
    let route_metadata = SelectedRecordRouteMetadata::primary(selected_content_class).ok_or(
        RecordAppendError::Denied(RecordAppendDenial::PublishedLayoutDamaged),
    )?;
    for (index, (record, admitted)) in identities.iter().copied().zip(batch.records).enumerate() {
        let length = admitted.declared_length;
        let route_metadata = if released_directory_record && index == 1 {
            SelectedRecordRouteMetadata::primary(SelectedRecordContentClass::DerivedDirectory)
                .ok_or(RecordAppendError::Denied(
                    RecordAppendDenial::PublishedLayoutDamaged,
                ))?
        } else {
            route_metadata
        };
        match placement_class(length, placement, blob_record_kind, inline_only) {
            RecordPlacementClass::ExtentBacked => {
                let source: Box<dyn RecordWriteSource> = match admitted.input {
                    RecordAppendInput::Bytes(bytes) => Box::new(OwnedRecordSource::new(bytes)),
                    RecordAppendInput::Source { source, .. } => source,
                };
                extents.push(ExtentInput {
                    record,
                    source,
                    length,
                    route_metadata,
                });
            }
            RecordPlacementClass::InlinePage => {
                inline.push(PendingInlineInput {
                    record,
                    input: admitted.input,
                    length,
                    route_metadata,
                });
            }
        }
    }
    Ok(ClassifiedBatch {
        identities,
        inline,
        extents,
        logical_bytes: batch.aggregate_bytes,
    })
}

pub(in crate::physical_runtime::record_serving) fn materialize_inline_inputs(
    pending: Vec<PendingInlineInput>,
) -> Result<MaterializedInlineBatch, RecordAppendError> {
    let mut records = Vec::with_capacity(pending.len());
    let mut copy_count = 0_u64;
    let mut copied_bytes = 0_u64;
    for input in pending {
        let bytes = match input.input {
            RecordAppendInput::Bytes(bytes) => bytes,
            RecordAppendInput::Source { source, .. } => {
                let materialized = materialize_bounded_inline(source, input.length)?;
                copy_count = copy_count.saturating_add(materialized.copy_count);
                copied_bytes = copied_bytes.saturating_add(materialized.copied_bytes);
                materialized.bytes
            }
        };
        records.push(MaterializedInlineInput {
            record: input.record,
            bytes,
            route_metadata: input.route_metadata,
        });
    }
    Ok(MaterializedInlineBatch {
        records,
        copy_count,
        copied_bytes,
    })
}

pub(in crate::physical_runtime::record_serving) fn preflight_placement(
    format: AdmittedPhysicalRecordFormat,
    placement: AdmittedRecordPlacementPolicy,
    batch: &RecordAppendBatch,
    blob_record_kind: Option<BlobRecordKind>,
    inline_only: bool,
) -> Result<(), RecordAppendError> {
    let inline_limit = maximum_inline_payload_bytes(format, placement);
    if batch.records.iter().any(|record| {
        let length = record.declared_length();
        placement_class(length, placement, blob_record_kind, inline_only)
            == RecordPlacementClass::InlinePage
            && length > inline_limit
    }) {
        return Err(RecordAppendError::Denied(
            RecordAppendDenial::InlinePageFull,
        ));
    }
    Ok(())
}

pub(in crate::physical_runtime::record_serving) fn append_operation_allocation_bytes(
    format: AdmittedPhysicalRecordFormat,
    placement: AdmittedRecordPlacementPolicy,
    batch: &RecordAppendBatch,
    blob_record_kind: Option<BlobRecordKind>,
    inline_only: bool,
) -> u64 {
    let page_bytes = u64::from(format.declaration().page_size().bytes());
    let mut has_extent = false;
    for record in &batch.records {
        match placement_class(
            record.declared_length(),
            placement,
            blob_record_kind,
            inline_only,
        ) {
            RecordPlacementClass::InlinePage => {}
            RecordPlacementClass::ExtentBacked => has_extent = true,
        }
    }
    // A durable record can simultaneously own its materialization frame, its
    // encoded candidate frame, and the recovery projection frame carried into
    // the WAL. Charge page geometry rather than caller payload length so small
    // records cannot hide either publication or reconstruction growth.
    let record_working_set = (batch.records.len() as u64)
        .saturating_mul(page_bytes)
        .saturating_mul(3);
    let streaming_window = if has_extent { page_bytes } else { 0 };
    // Three routing families can each rewrite at most one path through a
    // u64-addressed tree. The four fixed frames cover root, catalog, segment,
    // and publication bookkeeping. This is independent of current store size.
    let routing_working_set = page_bytes.saturating_mul(64 * 3 + 4);
    record_working_set
        .saturating_add(streaming_window)
        .saturating_add(routing_working_set)
}

fn placement_class(
    length: u64,
    placement: AdmittedRecordPlacementPolicy,
    blob_record_kind: Option<BlobRecordKind>,
    inline_only: bool,
) -> RecordPlacementClass {
    if !inline_only
        && (blob_record_kind.is_some() || length >= u64::from(placement.extent_threshold().get()))
    {
        RecordPlacementClass::ExtentBacked
    } else {
        RecordPlacementClass::InlinePage
    }
}

pub(in crate::physical_runtime) fn maximum_inline_payload_bytes(
    format: AdmittedPhysicalRecordFormat,
    placement: AdmittedRecordPlacementPolicy,
) -> u64 {
    let frame_payload =
        u64::from(format.declaration().page_size().bytes() - DURABLE_FRAME_HEADER_BYTES as u32);
    frame_payload
        .saturating_mul(u64::from(placement.page_fill().get()))
        .checked_div(100)
        .unwrap_or(0)
        .saturating_sub((DURABLE_INLINE_PAGE_PREFIX_BYTES + DURABLE_INLINE_SLOT_BYTES) as u64)
}

struct MaterializedInlineSource {
    bytes: Vec<u8>,
    copy_count: u64,
    copied_bytes: u64,
}

fn materialize_bounded_inline(
    mut source: Box<dyn RecordWriteSource>,
    length: u64,
) -> Result<MaterializedInlineSource, RecordAppendError> {
    let mut bytes = vec![0_u8; length as usize];
    let mut completed = 0_usize;
    let mut copy_count = 0_u64;
    while completed < bytes.len() {
        let count = source.read_next(&mut bytes[completed..]).map_err(|_| {
            RecordAppendError::StreamFailed(RecordStreamFailure::before_media_write(
                RecordStreamFailureKind::ProducerRejected,
                completed as u64,
            ))
        })?;
        if count == 0 {
            return Err(stream_failure(
                RecordStreamFailureKind::SourceEndedEarly,
                completed,
            ));
        }
        if count > bytes.len() - completed {
            return Err(stream_failure(
                RecordStreamFailureKind::InvalidTransferCount,
                completed,
            ));
        }
        completed += count;
        copy_count = copy_count.saturating_add(1);
    }
    let mut extra = [0_u8; 1];
    let extra_count = source.read_next(&mut extra).map_err(|_| {
        RecordAppendError::StreamFailed(RecordStreamFailure::before_media_write(
            RecordStreamFailureKind::ProducerRejected,
            completed as u64,
        ))
    })?;
    if extra_count != 0 {
        return Err(stream_failure(
            RecordStreamFailureKind::SourceExceededDeclaredLength,
            completed,
        ));
    }
    Ok(MaterializedInlineSource {
        bytes,
        copy_count,
        copied_bytes: completed as u64,
    })
}

fn stream_failure(kind: RecordStreamFailureKind, completed: usize) -> RecordAppendError {
    RecordAppendError::StreamFailed(RecordStreamFailure::before_media_write(
        kind,
        completed as u64,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physical_runtime::record_serving::PhysicalRecordPlacementPolicy;
    use worth_store_physical_format::PhysicalRecordFormatDeclaration;

    #[test]
    fn all_canonical_blob_families_force_extent_even_below_inline_threshold() {
        let format = AdmittedPhysicalRecordFormat::admit(
            PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
        );
        let placement = PhysicalRecordPlacementPolicy::builder()
            .admit(format)
            .unwrap();
        assert_eq!(
            placement_class(100, placement, None, false),
            RecordPlacementClass::InlinePage
        );
        for kind in [
            BlobRecordKind::SessionDeclared,
            BlobRecordKind::Chunk,
            BlobRecordKind::TreeNode,
            BlobRecordKind::GenerationPublished,
            BlobRecordKind::SessionFrontier,
            BlobRecordKind::SessionAbandoned,
            BlobRecordKind::DropSetManifestV3,
            BlobRecordKind::OriginalDropReserved,
            BlobRecordKind::ReclaimDescriptorV3,
        ] {
            assert_eq!(
                placement_class(100, placement, Some(kind), false),
                RecordPlacementClass::ExtentBacked,
            );
        }
    }
}
