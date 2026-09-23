use std::collections::BTreeMap;
use std::num::NonZeroU64;

use sha2::{Digest, Sha256};
use worth_proof::TransitionOutcome;
use worth_store_physical_format::{
    restamp_inline_page_generation, CurrentPhysicalRecordPlacement, DurableInlineRecordPlacement,
    PersistedRecordIdentity, PhysicalGenerationAuthority, PhysicalRewriteRedo, RecordArtifactFile,
    RecordSegmentPageManifestEntry, SegmentPageKey,
};

use super::super::durable_preparation::CanonicalPayloadMaterializationObservation;
use super::durable_preparation::{
    canonical_request_failure, map_record_denial, PhysicalMutationPreparationAdmission,
};
use super::RecordPublicationDirector;
use crate::physical_runtime::durability::{PhysicalMutationOperationFamily, RetiredArtifact};
use crate::physical_runtime::record_serving::planning::batch_placement::append_operation_allocation_bytes;
use crate::physical_runtime::record_serving::planning::inline_plan_failure::admitted_generation;
use crate::physical_runtime::record_serving::planning::inline_segment_plan::WorkingSegment;
use crate::physical_runtime::record_serving::planning::published_segment_reuse::{
    load_published_segment, ReusableSegmentContext,
};
use crate::physical_runtime::record_serving::planning::published_tail_page::load_published_tail_page;
use crate::physical_runtime::record_serving::publication::append_observation::PublicationObservation;
use crate::physical_runtime::record_serving::residency::serving_artifacts::ServingRecordArtifacts;
use crate::physical_runtime::record_serving::{
    PreparedPhysicalRootProjection, RecordAppendBatch, RecordAppendDenial, RecordAppendError,
};
use crate::physical_runtime::{
    durability::{PreparedPhysicalDataFrame, PreparedPhysicalDataPlan},
    CertifiedPriorPageBasis, PhysicalDataFrameIdentity, PhysicalMutationPreparationOutcome,
    PhysicalMutationPreparationSuccess, PhysicalMutationRequest, PhysicalMutationResourceShape,
    PreparedPhysicalMutation, PreparedPhysicalMutationContext,
};

impl RecordPublicationDirector {
    pub(super) fn rewrite_source_changed(&self, prepared: &PreparedPhysicalMutation) -> bool {
        if !prepared.selected_segment_rewrite() {
            return false;
        }
        let (root, _) = self.root_owner.snapshot();
        root.generation() != prepared.source_root_generation()
    }

    pub(super) fn prepare_selected_segment_rewrite(
        &self,
        placement: crate::physical_runtime::AdmittedRecordPlacementPolicy,
        request: PhysicalMutationRequest,
        pages: u32,
    ) -> PhysicalMutationPreparationOutcome {
        if let Err(outcome) = self.require_preparation_health() {
            return outcome;
        }
        if pages == 0
            || u64::from(pages) * u64::from(self.format.declaration().page_size().bytes())
                > 256 * 1024
        {
            return map_record_denial(RecordAppendDenial::PhysicalPressure);
        }
        if !placement.admits(self.format) {
            return map_record_denial(RecordAppendDenial::PlacementFormatMismatch);
        }
        let (root, _) = self.root_owner.snapshot();
        let Some(record) = root.last_inline_record() else {
            return map_record_denial(RecordAppendDenial::PublishedLayoutDamaged);
        };
        let span_start = match self.selected_rewrite_span_start(placement, pages) {
            Ok(start) => start,
            Err(RecordAppendError::Denied(denial)) => return map_record_denial(denial),
            Err(_) => return map_record_denial(RecordAppendDenial::PublishedLayoutDamaged),
        };
        let digest = super::rewrite_span_selection::rewrite_basis_digest(record, pages, span_start);
        let group_queue_admission = match self.group_queue_admission_tick() {
            Ok(tick) => tick,
            Err(outcome) => return outcome,
        };
        let admitted = match self.admit_mutation_preparation(
            placement,
            crate::physical_runtime::PhysicalManifestCapacityTransition::PreserveCurrent,
            digest,
            request,
            PhysicalMutationOperationFamily::SegmentRewrite,
        ) {
            Ok(admitted) => admitted,
            Err(outcome) => return outcome,
        };
        let PhysicalMutationPreparationAdmission::Prepared(admitted) = admitted else {
            return admitted_terminal(admitted);
        };
        let page_bytes = u64::from(self.format.declaration().page_size().bytes());
        let batch = RecordAppendBatch::from_prepared_record_bytes(vec![vec![0]]);
        TransitionOutcome::success(PhysicalMutationPreparationSuccess::Prepared(
            PreparedPhysicalMutation::new(
                admitted.admission,
                batch,
                CanonicalPayloadMaterializationObservation::default(),
                PreparedPhysicalMutationContext {
                    placement,
                    manifest_capacity_transition:
                        crate::physical_runtime::PhysicalManifestCapacityTransition::PreserveCurrent,
                    deadline: admitted.deadline,
                    group_queue_admission,
                    signal_profile: self.signal_profile,
                    durability_policy_basis: self.durability_policy_basis.clone(),
                    resources: PhysicalMutationResourceShape::prepared(1, page_bytes * u64::from(pages)),
                    start: crate::physical_runtime::PhysicalMutationRuntimeOwner::start_port(
                        &self.mutations,
                    ),
                    selected_segment_rewrite: false,
                    rewrite_pages: 0,
                    source_root_generation: 0,
                    rewrite_anchor: None,
                },
            )
            .mark_selected_segment_rewrite(root.generation(), pages, None),
        ))
        .into()
    }

    pub(super) fn build_selected_segment_rewrite(
        &self,
        prepared: &PreparedPhysicalMutation,
    ) -> Result<(PreparedPhysicalDataPlan, PreparedPhysicalRootProjection), RecordAppendError> {
        if prepared.rewrite_pages() != 1 {
            return self.build_rewrite_page_span(prepared);
        }
        let runtime = self.runtime.upgrade().ok_or(RecordAppendError::Denied(
            RecordAppendDenial::PublicationAuthorityReleased,
        ))?;
        let batch = prepared.duplicate_prepared_batch();
        let bytes = append_operation_allocation_bytes(self.format, prepared.placement(), &batch);
        let allocation = self
            .residency
            .begin_foreground_write_operation(NonZeroU64::new(bytes).ok_or(
                RecordAppendError::Denied(RecordAppendDenial::PublishedLayoutDamaged),
            )?)
            .map_err(|denial| {
                RecordAppendError::Denied(RecordAppendDenial::from_residency(denial))
            })?;
        let (current_root, current_free_space) = self.root_owner.snapshot();
        if current_root.generation() != prepared.source_root_generation() {
            return Err(RecordAppendError::Denied(
                RecordAppendDenial::PublishedLayoutDamaged,
            ));
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
        let segment = segment.ok_or(RecordAppendError::Denied(
            RecordAppendDenial::PublishedLayoutDamaged,
        ))?;
        let media = runtime.executor.record_serving_media();
        let artifacts = ServingRecordArtifacts::serving(media, self.residency.clone());
        let loaded = load_published_tail_page(
            &allocation,
            &artifacts,
            self.durability.store_identity(),
            self.format,
            last,
            &segment,
        )?;
        let page_entry = segment
            .last_published_page
            .ok_or(RecordAppendError::Denied(
                RecordAppendDenial::PublishedLayoutDamaged,
            ))?;
        let page_bytes = u64::from(self.format.declaration().page_size().bytes());
        let page_len = u32::try_from(page_bytes).map_err(|_| damaged())?;
        let tail_frame = page_entry.frame_index();
        let displaces_source = self
            .rewrite_source_liveness(
                &allocation,
                &current_root,
                segment.segment.segment_id(),
                page_entry.data_generation(),
                tail_frame..tail_frame.checked_add(1).ok_or_else(damaged)?,
            )?
            .displaces_source()?;
        let source_offset = u64::from(page_entry.frame_index()).saturating_mul(page_bytes);
        let page_generation = admitted_generation(loaded.geometry.generation().checked_add(1))?;
        let authority = PhysicalGenerationAuthority::for_canonical_physical_format();
        let candidate_page = authority
            .page_cell(segment.segment.segment_id(), loaded.geometry.page())
            .with_page_generation(page_generation);
        let destination_artifact = RecordArtifactFile::Segment {
            segment: segment.segment.segment_id().get(),
            generation: segment.segment.generation().get(),
        };
        let source_bytes = loaded.image.bytes();
        let source_digest = Sha256::digest(source_bytes).into();
        let restamped = restamp_inline_page_generation(
            self.format.declaration(),
            source_bytes,
            candidate_page.generation().get(),
        )
        .map_err(|_| damaged())?;
        let prior = CertifiedPriorPageBasis::for_integrity_admitted_materialized_source(
            loaded.image.admitted_prior_basis(),
        )
        .ok_or_else(damaged)?;
        let target = PhysicalDataFrameIdentity::inline_page(
            candidate_page,
            destination_artifact,
            0,
            page_len,
        )
        .ok_or_else(damaged)?;
        let frame = PreparedPhysicalDataFrame::new(
            target,
            prior,
            vec![0],
            restamped,
            self.format.declaration(),
        )
        .map_err(|_| damaged())?;
        let mut placements = BTreeMap::new();
        let mut records = Vec::with_capacity(loaded.records.len());
        let mut logical_bytes = 0_u64;
        for descriptor in &loaded.records {
            let slot = authority
                .slot_cell(
                    segment.segment.segment_id(),
                    candidate_page.page_id(),
                    descriptor.slot,
                )
                .with_slot_generation(admitted_generation(Some(descriptor.slot_generation))?);
            let placement = DurableInlineRecordPlacement::new(
                descriptor.record,
                segment.segment,
                candidate_page,
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
        let tail = records.last().copied().ok_or_else(damaged)?;
        let entry = RecordSegmentPageManifestEntry::new(candidate_page, segment.segment, 1, 0)
            .ok_or_else(damaged)?;
        let working = WorkingSegment {
            segment: segment.segment,
            page_capacity: segment.page_capacity,
            used_pages: segment.used_pages,
            membership_updates: vec![entry],
            data_pages: Vec::new(),
        };
        let mut segment_updates = BTreeMap::new();
        segment_updates.insert(SegmentPageKey::from(entry), entry);
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
            source_offset,
            page_len,
            source_digest,
            segment.segment.generation().get(),
            0,
            0,
            record_identity_bytes(tail),
            loaded.geometry.generation(),
            candidate_page.generation().get(),
            resulting_root,
        )
        .ok_or_else(damaged)?;
        self.root_owner
            .hold_rewrite_candidate(
                RecordArtifactFile::Segment {
                    segment: segment.segment.segment_id().get(),
                    generation: segment.segment.generation().get(),
                },
                page_bytes,
            )
            .map_err(|()| RecordAppendError::Denied(RecordAppendDenial::RetentionPressure))?;
        // Live frames besides the tail keep the source file reachable.
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
        let data = PreparedPhysicalDataPlan::new(vec![frame], 1)
            .map_err(|_| damaged())?
            .with_rewrite(rewrite);
        let root = PreparedPhysicalRootProjection {
            root_publication_allocation_bytes: NonZeroU64::new(bytes).ok_or_else(damaged)?,
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
                records: loaded.records.len() as u64,
                logical_bytes,
                completed_bytes: 0,
                segment_artifacts: 1,
                extent_artifacts: 0,
                transfer_count: 0,
                peak_transfer_width: page_bytes,
                explicit_copy_count: 1,
                copied_bytes: page_bytes,
                peak_scratch_bytes: page_bytes,
                manifest_blocks_read: 0,
                manifest_comparisons: 0,
                manifest_bytes_read: 0,
            },
        };
        Ok((data, root))
    }
}

pub(super) fn admitted_terminal(
    admitted: PhysicalMutationPreparationAdmission,
) -> PhysicalMutationPreparationOutcome {
    match admitted {
        PhysicalMutationPreparationAdmission::Prepared(_) => canonical_request_failure(),
        PhysicalMutationPreparationAdmission::ProvenNoEffect(terminal) => {
            TransitionOutcome::success(PhysicalMutationPreparationSuccess::ProvenNoEffect(terminal))
                .into()
        }
        PhysicalMutationPreparationAdmission::Completed(terminal) => {
            TransitionOutcome::success(PhysicalMutationPreparationSuccess::Completed(terminal))
                .into()
        }
        PhysicalMutationPreparationAdmission::Indeterminate(terminal) => {
            TransitionOutcome::success(PhysicalMutationPreparationSuccess::Indeterminate(terminal))
                .into()
        }
    }
}

pub(super) fn record_identity_bytes(record: PersistedRecordIdentity) -> [u8; 32] {
    let mut bytes = [0; 32];
    bytes[..16].copy_from_slice(&record.allocation_epoch());
    bytes[16..24].copy_from_slice(&record.ordinal().to_le_bytes());
    bytes
}

pub(super) fn damaged() -> RecordAppendError {
    RecordAppendError::Denied(RecordAppendDenial::PublishedLayoutDamaged)
}
