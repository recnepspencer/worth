use std::collections::BTreeMap;
use std::num::NonZeroU64;

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    restamp_inline_page_generation, CurrentPhysicalRecordPlacement, DurableInlineRecordPlacement,
    PhysicalGenerationAuthority, PhysicalRewriteRedo, RecordArtifactFile,
    RecordSegmentPageManifestEntry, SegmentPageKey,
};

use super::RecordPublicationDirector;
use crate::physical_runtime::record_serving::planning::batch_placement::append_operation_allocation_bytes;
use crate::physical_runtime::record_serving::planning::inline_plan_failure::admitted_generation;
use crate::physical_runtime::record_serving::planning::inline_segment_plan::WorkingSegment;
use crate::physical_runtime::record_serving::planning::published_segment_reuse::{
    load_published_segment, ReusableSegmentContext,
};
use crate::physical_runtime::record_serving::publication::append_observation::PublicationObservation;
use crate::physical_runtime::record_serving::residency::serving_artifacts::ServingRecordArtifacts;
use crate::physical_runtime::record_serving::{
    PreparedPhysicalRootProjection, RecordAppendDenial, RecordAppendError,
};
use crate::physical_runtime::{
    durability::{PreparedPhysicalDataFrame, PreparedPhysicalDataPlan, RetiredArtifact},
    CertifiedPriorPageBasis, PhysicalDataFrameIdentity, PreparedPhysicalMutation,
};

struct SelectedPage {
    index: u32,
    source: Vec<u8>,
    source_generation: u64,
    restamped: Vec<u8>,
    prior: CertifiedPriorPageBasis,
    page: worth_store_physical_format::PageGenerationCell,
    records: Vec<crate::physical_runtime::record_serving::work_semantics::integrity_admission::AdmittedCleanInlinePageRecord>,
}

impl RecordPublicationDirector {
    pub(super) fn build_rewrite_page_span(
        &self,
        prepared: &PreparedPhysicalMutation,
    ) -> Result<(PreparedPhysicalDataPlan, PreparedPhysicalRootProjection), RecordAppendError> {
        let pages = prepared.rewrite_pages();
        let page_bytes = u64::from(self.format.declaration().page_size().bytes());
        let source_bytes = page_bytes * u64::from(pages);
        let runtime = self.runtime.upgrade().ok_or(RecordAppendError::Denied(
            RecordAppendDenial::PublicationAuthorityReleased,
        ))?;
        let batch = prepared.duplicate_prepared_batch();
        let allocation_bytes =
            append_operation_allocation_bytes(self.format, prepared.placement(), &batch)
                .max(source_bytes);
        let allocation = self
            .residency
            .begin_foreground_write_operation(
                NonZeroU64::new(allocation_bytes).ok_or_else(damaged)?,
            )
            .map_err(|denial| {
                RecordAppendError::Denied(RecordAppendDenial::from_residency(denial))
            })?;
        let (current_root, current_free_space) = self.root_owner.snapshot();
        if current_root.generation() != prepared.source_root_generation() {
            return Err(damaged());
        }
        let last = super::rewrite_anchor::resolve(self, prepared, &allocation, &current_root)?;
        let (segment, _) = load_published_segment(
            ReusableSegmentContext {
                allocation: &allocation,
                residency: self.residency.clone(),
                format: self.format,
                access: self.access,
                current_root: &current_root,
                current_free_space: &current_free_space,
                placement: prepared.placement(),
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
        let displaces_source = self
            .rewrite_source_liveness(
                &allocation,
                &current_root,
                segment.segment.segment_id(),
                page_entry.data_generation(),
                start..tail_bound,
            )?
            .displaces_source()?;
        let media = runtime.executor.record_serving_media();
        let artifacts = ServingRecordArtifacts::serving(media, self.residency.clone());
        let mut selected = Vec::with_capacity(pages as usize);
        for index in start..start + pages {
            selected.push(load_selected_page(
                &allocation,
                &artifacts,
                self,
                &segment.segment,
                page_entry.data_generation(),
                page_entry.data_page_count(),
                index,
                page_bytes,
            )?);
        }
        let mut digest = Sha256::new();
        for page in &selected {
            digest.update(&page.source);
        }
        let source_digest = digest.finalize().into();
        let page_len = u32::try_from(page_bytes).map_err(|_| damaged())?;
        let authority = PhysicalGenerationAuthority::for_canonical_physical_format();
        let destination_artifact = RecordArtifactFile::Segment {
            segment: segment.segment.segment_id().get(),
            generation: segment.segment.generation().get(),
        };
        let mut frames = Vec::with_capacity(selected.len());
        let mut placements = BTreeMap::new();
        let mut records = Vec::new();
        let mut logical_bytes = 0_u64;
        let mut membership = Vec::with_capacity(selected.len());
        let mut segment_updates = BTreeMap::new();
        // The span becomes its own compact generation, as the single-page
        // rewrite does: frames start at offset 0 and the untouched prefix
        // stays live in the source generation.
        for page in &selected {
            let frame = page.index - start;
            let entry =
                RecordSegmentPageManifestEntry::new(page.page, segment.segment, pages, frame)
                    .ok_or_else(damaged)?;
            segment_updates.insert(SegmentPageKey::from(entry), entry);
            membership.push(entry);
            let target = PhysicalDataFrameIdentity::inline_page(
                page.page,
                destination_artifact,
                u64::from(frame) * page_bytes,
                page_len,
            )
            .ok_or_else(damaged)?;
            frames.push(
                PreparedPhysicalDataFrame::new(
                    target,
                    page.prior,
                    vec![0],
                    page.restamped.clone(),
                    self.format.declaration(),
                )
                .map_err(|_| damaged())?,
            );
            for descriptor in &page.records {
                let slot = authority
                    .slot_cell(
                        segment.segment.segment_id(),
                        page.page.page_id(),
                        descriptor.slot,
                    )
                    .with_slot_generation(admitted_generation(Some(descriptor.slot_generation))?);
                let placement = DurableInlineRecordPlacement::new(
                    descriptor.record,
                    segment.segment,
                    page.page,
                    slot,
                    segment.page_capacity,
                    u64::from(descriptor.payload_bytes),
                )
                .ok_or_else(damaged)?;
                placements.insert(
                    descriptor.record,
                    CurrentPhysicalRecordPlacement::Inline(placement),
                );
                records.push(descriptor.record);
                logical_bytes = logical_bytes.saturating_add(u64::from(descriptor.payload_bytes));
            }
        }
        let record_count = records.len() as u64;
        let tail = records.last().copied().ok_or_else(damaged)?;
        let tail_page = selected.last().ok_or_else(damaged)?;
        let resulting_root =
            current_root
                .generation()
                .checked_add(1)
                .ok_or(RecordAppendError::Denied(
                    RecordAppendDenial::RootGenerationExhausted,
                ))?;
        let rewrite = PhysicalRewriteRedo::new(
            prepared.request_fingerprint().bytes(),
            [0; 32],
            current_root.generation(),
            page_entry.data_generation(),
            u64::from(start) * page_bytes,
            u32::try_from(source_bytes).map_err(|_| damaged())?,
            source_digest,
            segment.segment.generation().get(),
            0,
            0,
            record_identity_bytes(tail),
            tail_page.source_generation,
            tail_page.page.generation().get(),
            resulting_root,
        )
        .ok_or_else(damaged)?;
        self.root_owner
            .hold_rewrite_candidate(
                RecordArtifactFile::Segment {
                    segment: segment.segment.segment_id().get(),
                    generation: segment.segment.generation().get(),
                },
                source_bytes,
            )
            .map_err(|()| RecordAppendError::Denied(RecordAppendDenial::RetentionPressure))?;
        // The prefix before the span stays live in the source file unless the
        // span covered every frame the root still reads from it.
        if displaces_source {
            self.root_owner.note_displaced(
                current_root.generation(),
                RetiredArtifact::Segment {
                    segment: segment.segment.segment_id().get(),
                    generation: page_entry.data_generation(),
                },
                u64::from(page_entry.data_page_count()) * page_bytes,
            );
        }
        let data = PreparedPhysicalDataPlan::new(frames, 1)
            .map_err(|_| damaged())?
            .with_rewrite(rewrite);
        let working = WorkingSegment {
            segment: segment.segment,
            page_capacity: segment.page_capacity,
            used_pages: segment.used_pages,
            membership_updates: membership,
            data_pages: Vec::new(),
        };
        let root = PreparedPhysicalRootProjection {
            root_publication_allocation_bytes: NonZeroU64::new(source_bytes).ok_or_else(damaged)?,
            source_root: current_root,
            manifest_capacity_transition: prepared.manifest_capacity_transition(),
            placement: prepared.placement(),
            records,
            inserted_records: 0,
            payload_manifests: Vec::new(),
            placements,
            segment_updates,
            inline_allocations: vec![working.allocation()],
            last_inline_record: Some(tail),
            last_inline_segment: Some(segment.segment),
            requires_maintenance_protocol: true,
            observation: PublicationObservation {
                records: record_count,
                logical_bytes,
                completed_bytes: 0,
                segment_artifacts: u64::from(pages),
                extent_artifacts: 0,
                transfer_count: 0,
                peak_transfer_width: page_bytes,
                explicit_copy_count: u64::from(pages),
                copied_bytes: source_bytes,
                peak_scratch_bytes: page_bytes,
                manifest_blocks_read: 0,
                manifest_comparisons: 0,
                manifest_bytes_read: 0,
            },
        };
        Ok((data, root))
    }
}

fn load_selected_page(
    allocation: &worth_store_buffer_pool::OperationAllocationGrant,
    artifacts: &ServingRecordArtifacts<'_>,
    director: &RecordPublicationDirector,
    segment: &worth_store_physical_format::SegmentGenerationCell,
    data_generation: u64,
    data_pages: u32,
    index: u32,
    page_bytes: u64,
) -> Result<SelectedPage, RecordAppendError> {
    let source = RecordArtifactFile::Segment {
        segment: segment.segment_id().get(),
        generation: data_generation,
    };
    let artifact_bytes = NonZeroU64::new(u64::from(data_pages) * page_bytes).ok_or_else(damaged)?;
    let resident = artifacts
        .load_exact(
            allocation,
            source,
            u64::from(index) * page_bytes,
            u32::try_from(page_bytes).map_err(|_| damaged())?,
            crate::physical_runtime::record_serving::residency::frame_loading::ExactFrameSourceExtent::CompleteArtifact(
                artifact_bytes,
            ),
        )
        .map_err(crate::physical_runtime::record_serving::planning::inline_plan_failure::layout_failure)?;
    let mut bytes = Vec::new();
    bytes.try_reserve_exact(resident.len()).map_err(|_| {
        RecordAppendError::Denied(RecordAppendDenial::from_residency(
            worth_store_buffer_pool::PhysicalResidencyDenial::AllocationFailed,
        ))
    })?;
    bytes.extend_from_slice(&resident);
    let context = artifacts.resident_admission_context().ok_or_else(damaged)?;
    let inspected =
        worth_store_physical_format::inspect_inline_page(director.format.declaration(), &bytes)
            .map_err(|_| damaged())?;
    let admitted = crate::physical_runtime::record_serving::work_semantics::integrity_admission::admit_inline_page(
        &resident,
        context,
        director.durability.store_identity(),
        director.format.declaration(),
        inspected.page_cell(),
    )
    .map_err(|_| {
        resident.reject_projection_failure();
        damaged()
    })?;
    let source_generation = admitted.page.generation().get();
    let page_generation = admitted_generation(source_generation.checked_add(1))?;
    let authority = PhysicalGenerationAuthority::for_canonical_physical_format();
    let page = authority
        .page_cell(segment.segment_id(), admitted.page.page_id())
        .with_page_generation(page_generation);
    let restamped = restamp_inline_page_generation(
        director.format.declaration(),
        &bytes,
        page.generation().get(),
    )
    .map_err(|_| damaged())?;
    let prior =
        CertifiedPriorPageBasis::for_integrity_admitted_materialized_source(admitted.prior_basis)
            .ok_or_else(damaged)?;
    Ok(SelectedPage {
        index,
        source: bytes,
        source_generation,
        restamped,
        prior,
        page,
        records: admitted.records,
    })
}

fn record_identity_bytes(record: worth_store_physical_format::PersistedRecordIdentity) -> [u8; 32] {
    let mut bytes = [0; 32];
    bytes[..16].copy_from_slice(&record.allocation_epoch());
    bytes[16..24].copy_from_slice(&record.ordinal().to_le_bytes());
    bytes
}

fn damaged() -> RecordAppendError {
    RecordAppendError::Denied(RecordAppendDenial::PublishedLayoutDamaged)
}

fn span_not_live() -> RecordAppendError {
    RecordAppendError::Denied(RecordAppendDenial::RewriteSpanNotLive)
}
