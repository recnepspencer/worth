use worth_store_physical_format::{
    store_namespace::StableStoreIdentity, DurableInlineRecordPlacement, RecordArtifactFile,
};

use super::super::{
    planning::inline_plan_failure::layout_failure, planning::inline_segment_plan::PlanningSegment,
    residency::serving_artifacts::ServingRecordArtifacts, AdmittedPhysicalRecordFormat,
    RecordAppendDenial, RecordAppendError,
};

pub(in crate::physical_runtime::record_serving) struct LoadedPublishedTailPage {
    pub(in crate::physical_runtime::record_serving) image:
        super::super::publication::ExistingDataFrameImage,
    pub(in crate::physical_runtime::record_serving) geometry: PublishedTailPageGeometry,
    pub(in crate::physical_runtime::record_serving) records:
        Vec<super::super::work_semantics::integrity_admission::AdmittedCleanInlinePageRecord>,
}

pub(in crate::physical_runtime::record_serving) struct PublishedTailPageGeometry {
    page: worth_store_physical_format::PageGenerationCell,
    slot_count: u16,
    free_bytes: u32,
}

impl PublishedTailPageGeometry {
    pub(super) const fn page_cell(&self) -> worth_store_physical_format::PageGenerationCell {
        self.page
    }

    pub(super) const fn generation(&self) -> u64 {
        self.page.generation().get()
    }

    pub(super) const fn page(&self) -> worth_store_physical_format::PhysicalPageId {
        self.page.page_id()
    }

    pub(super) const fn slot_count(&self) -> u16 {
        self.slot_count
    }

    pub(super) const fn free_bytes(&self) -> u32 {
        self.free_bytes
    }
}

pub(in crate::physical_runtime::record_serving) fn load_published_tail_page(
    allocation: &worth_store_buffer_pool::OperationAllocationGrant,
    artifacts: &ServingRecordArtifacts<'_>,
    store: StableStoreIdentity,
    format: AdmittedPhysicalRecordFormat,
    last: DurableInlineRecordPlacement,
    segment: &PlanningSegment,
) -> Result<LoadedPublishedTailPage, RecordAppendError> {
    let page_entry = segment
        .last_published_page
        .ok_or(RecordAppendError::Denied(
            RecordAppendDenial::PublishedLayoutDamaged,
        ))?;
    if page_entry.page_cell() != last.page_cell() {
        return Err(RecordAppendError::Denied(
            RecordAppendDenial::PublishedLayoutDamaged,
        ));
    }
    let page_bytes = format.declaration().page_size().bytes();
    let source = RecordArtifactFile::Segment {
        segment: last.segment().get(),
        generation: page_entry.data_generation(),
    };
    let source_bytes =
        std::num::NonZeroU64::new(u64::from(page_entry.data_page_count()) * u64::from(page_bytes))
            .expect("an admitted published segment has nonzero data pages");
    let resident = artifacts
        .load_exact(
            allocation,
            source,
            u64::from(page_entry.frame_index()) * u64::from(page_bytes),
            page_bytes,
            super::super::residency::frame_loading::ExactFrameSourceExtent::CompleteArtifact(
                source_bytes,
            ),
        )
        .map_err(layout_failure)?;
    materialize_admitted_tail(artifacts, resident, store, format, last)
}

fn materialize_admitted_tail(
    artifacts: &ServingRecordArtifacts<'_>,
    resident: super::super::residency::frame_loading::LoadedPhysicalFrame,
    store: StableStoreIdentity,
    format: AdmittedPhysicalRecordFormat,
    last: DurableInlineRecordPlacement,
) -> Result<LoadedPublishedTailPage, RecordAppendError> {
    let context = artifacts
        .resident_admission_context()
        .ok_or(RecordAppendError::Denied(
            RecordAppendDenial::PublishedLayoutDamaged,
        ))?;
    let admitted = super::super::work_semantics::integrity_admission::admit_inline_page(
        &resident,
        context,
        store,
        format.declaration(),
        last.page_cell(),
    );
    let admitted = match admitted {
        Ok(admitted) => admitted,
        Err(_) => {
            resident.reject_projection_failure();
            return Err(RecordAppendError::Denied(
                RecordAppendDenial::PublishedLayoutDamaged,
            ));
        }
    };
    let geometry = PublishedTailPageGeometry {
        page: admitted.page,
        slot_count: admitted.slot_count,
        free_bytes: admitted.free_bytes,
    };
    if geometry.page_cell() != last.page_cell() {
        resident.reject_projection_failure();
        return Err(RecordAppendError::Denied(
            RecordAppendDenial::PublishedLayoutDamaged,
        ));
    }
    let image = super::super::publication::ExistingDataFrameImage::new(
        copy_resident_page(&resident)?,
        admitted.prior_basis,
    )
    .ok_or(RecordAppendError::Denied(
        RecordAppendDenial::PublishedLayoutDamaged,
    ))?;
    Ok(LoadedPublishedTailPage {
        image,
        geometry,
        records: admitted.records,
    })
}

fn copy_resident_page(
    resident: &super::super::residency::frame_loading::LoadedPhysicalFrame,
) -> Result<Vec<u8>, RecordAppendError> {
    let mut page = Vec::new();
    page.try_reserve_exact(resident.len()).map_err(|_| {
        RecordAppendError::Denied(RecordAppendDenial::from_residency(
            worth_store_buffer_pool::PhysicalResidencyDenial::AllocationFailed,
        ))
    })?;
    page.extend_from_slice(resident);
    Ok(page)
}
