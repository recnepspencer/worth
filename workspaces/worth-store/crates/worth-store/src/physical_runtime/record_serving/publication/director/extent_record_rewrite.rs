use std::collections::BTreeMap;
use std::num::NonZeroU64;

use sha2::{Digest, Sha256};
use worth_proof::TransitionOutcome;
use worth_store_physical_format::{
    prepare_extent_chunk, CurrentPhysicalRecordPlacement, DurableExtentManifest,
    DurableExtentRecordPlacement, DurablePhysicalRootManifest, ExtentChunkCoordinate,
    PersistedRecordIdentity, PhysicalGeneration, PhysicalGenerationAuthority, PhysicalRewriteRedo,
    RecordArtifactFile,
};

use super::super::durable_preparation::CanonicalPayloadMaterializationObservation;
use super::durable_preparation::{map_record_denial, PhysicalMutationPreparationAdmission};
use super::selected_segment_rewrite::{admitted_terminal, damaged, record_identity_bytes};
use super::RecordPublicationDirector;
use crate::physical_runtime::durability::{
    PhysicalMutationOperationFamily, PreparedPhysicalDataFrame, PreparedPhysicalDataPlan,
    RetiredArtifact,
};
use crate::physical_runtime::record_serving::access::extent_rewrite_source::{
    load_extent_rewrite_source, ExtentRewriteSourceRequest,
};
use crate::physical_runtime::record_serving::access::manifest_routing::{
    ManifestDiscoveryCounterSnapshot, ManifestReader,
};
use crate::physical_runtime::record_serving::planning::inline_plan_failure::manifest_lookup_failure;
use crate::physical_runtime::record_serving::publication::append_observation::PublicationObservation;
use crate::physical_runtime::record_serving::{
    PreparedPhysicalRootProjection, RecordAppendBatch, RecordAppendDenial, RecordAppendError,
};
use crate::physical_runtime::{
    CertifiedPriorPageBasis, PhysicalDataFrameIdentity, PhysicalMutationPreparationOutcome,
    PhysicalMutationPreparationSuccess, PhysicalMutationRequest, PhysicalMutationResourceShape,
    PreparedPhysicalMutation, PreparedPhysicalMutationContext,
};

/// The largest extent payload one copy-on-write rewrite may select.
const MAXIMUM_EXTENT_REWRITE_BYTES: u64 = 256 * 1024;

impl RecordPublicationDirector {
    pub(super) fn prepare_extent_record_rewrite(
        &self,
        placement: crate::physical_runtime::AdmittedRecordPlacementPolicy,
        request: PhysicalMutationRequest,
        record: PersistedRecordIdentity,
    ) -> PhysicalMutationPreparationOutcome {
        if let Err(outcome) = self.require_preparation_health() {
            return outcome;
        }
        if !placement.admits(self.format) {
            return map_record_denial(RecordAppendDenial::PlacementFormatMismatch);
        }
        let (root, _) = self.root_owner.snapshot();
        let source = match self.current_extent_source(&root, record) {
            Ok(source) => source,
            Err(RecordAppendError::Denied(denial)) => return map_record_denial(denial),
            Err(_) => return map_record_denial(RecordAppendDenial::PublishedLayoutDamaged),
        };
        if source.payload_bytes() > MAXIMUM_EXTENT_REWRITE_BYTES {
            return map_record_denial(RecordAppendDenial::PhysicalPressure);
        }
        let digest = extent_rewrite_basis_digest(source);
        let group_queue_admission = match self.group_queue_admission_tick() {
            Ok(tick) => tick,
            Err(outcome) => return outcome,
        };
        let admitted = match self.admit_mutation_preparation(
            placement,
            crate::physical_runtime::PhysicalManifestCapacityTransition::PreserveCurrent,
            digest,
            request,
            PhysicalMutationOperationFamily::ExtentRewrite,
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
                    resources: PhysicalMutationResourceShape::prepared(
                        1,
                        source.payload_bytes().max(page_bytes),
                    ),
                    start: crate::physical_runtime::PhysicalMutationRuntimeOwner::start_port(
                        &self.mutations,
                    ),
                    selected_segment_rewrite: false,
                    rewrite_pages: 0,
                    source_root_generation: 0,
                    rewrite_anchor: None,
                },
            )
            .mark_extent_record_rewrite(root.generation(), source),
        ))
        .into()
    }

    pub(super) fn build_extent_record_rewrite(
        &self,
        prepared: &PreparedPhysicalMutation,
        prepared_source: DurableExtentRecordPlacement,
    ) -> Result<(PreparedPhysicalDataPlan, PreparedPhysicalRootProjection), RecordAppendError> {
        let (current_root, _) = self.root_owner.snapshot();
        if current_root.generation() != prepared.source_root_generation() {
            return Err(damaged());
        }
        // The prepared placement is only a claim; the current root must still
        // name exactly that extent generation.
        let source = self.current_extent_source(&current_root, prepared_source.record())?;
        if source != prepared_source {
            return Err(damaged());
        }
        let format = self.format.declaration();
        let page_bytes = u64::from(format.page_size().bytes());
        let allocation_bytes = NonZeroU64::new(page_bytes.saturating_mul(2)).ok_or_else(damaged)?;
        let allocation = self
            .residency
            .begin_foreground_write_operation(allocation_bytes)
            .map_err(|denial| {
                RecordAppendError::Denied(RecordAppendDenial::from_residency(denial))
            })?;
        let loaded = load_extent_rewrite_source(
            ExtentRewriteSourceRequest {
                residency: self.residency.clone(),
                store: self.durability.store_identity(),
                format: self.format,
                access: self.access,
                generation: self.generation,
                allocation: &allocation,
            },
            source,
        )
        .map_err(|()| damaged())?;
        let destination_generation =
            source
                .extent_generation()
                .checked_add(1)
                .ok_or(RecordAppendError::Denied(
                    RecordAppendDenial::PhysicalIdentityExhausted,
                ))?;
        let destination_cell = PhysicalGenerationAuthority::for_canonical_physical_format()
            .record_extent_cell(source.extent())
            .with_extent_generation(
                PhysicalGeneration::from_raw(destination_generation).map_err(|_| damaged())?,
            );
        let manifest = DurableExtentManifest::new(
            format,
            source.record(),
            destination_cell,
            loaded.manifest.logical_bytes(),
            format.page_size().bytes(),
            loaded.manifest.chunk_count(),
        )
        .ok_or_else(damaged)?;
        let destination = RecordArtifactFile::Extent {
            extent: source.extent().get(),
            generation: destination_generation,
        };
        let (frames, written_bytes) =
            self.encode_extent_frames(&manifest, destination, &loaded.payload)?;
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
            source.extent_generation(),
            0,
            u32::try_from(loaded.payload.len()).map_err(|_| damaged())?,
            Sha256::digest(&loaded.payload).into(),
            destination_generation,
            0,
            0,
            record_identity_bytes(source.record()),
            source.extent().get(),
            source.extent().get(),
            resulting_root,
        )
        .ok_or_else(damaged)?;
        let manifest_bytes = manifest.encode(format);
        self.root_owner
            .hold_rewrite_candidate(
                destination,
                written_bytes.saturating_add(manifest_bytes.len() as u64),
            )
            .map_err(|()| RecordAppendError::Denied(RecordAppendDenial::RetentionPressure))?;
        // One record owns an extent, so the successor always leaves the whole
        // source generation unreachable from the resulting root.
        self.root_owner.note_displaced(
            current_root.generation(),
            RetiredArtifact::Extent {
                extent: source.extent().get(),
                generation: source.extent_generation(),
            },
            loaded.artifact_bytes,
        );
        let placement = DurableExtentRecordPlacement::new(
            source.record(),
            destination_cell,
            source.payload_bytes(),
        )
        .ok_or_else(damaged)?;
        let mut placements = BTreeMap::new();
        placements.insert(
            source.record(),
            CurrentPhysicalRecordPlacement::Extent(placement),
        );
        let frame_count = frames.len() as u64;
        let data = PreparedPhysicalDataPlan::new(frames, 1)
            .map_err(|_| damaged())?
            .with_rewrite(rewrite);
        let root = PreparedPhysicalRootProjection {
            root_publication_allocation_bytes: allocation_bytes,
            source_root: current_root,
            manifest_capacity_transition: prepared.manifest_capacity_transition(),
            placement: prepared.placement(),
            records: vec![source.record()],
            inserted_records: 0,
            payload_manifests: vec![(
                RecordArtifactFile::ExtentManifest {
                    extent: source.extent().get(),
                    generation: destination_generation,
                },
                manifest_bytes,
            )],
            placements,
            segment_updates: BTreeMap::new(),
            inline_allocations: Vec::new(),
            // None inherits the current inline tail, which this rewrite leaves alone.
            last_inline_record: None,
            last_inline_segment: None,
            requires_maintenance_protocol: true,
            observation: PublicationObservation {
                records: 1,
                logical_bytes: source.payload_bytes(),
                completed_bytes: 0,
                segment_artifacts: 0,
                extent_artifacts: 1,
                transfer_count: frame_count,
                peak_transfer_width: page_bytes,
                explicit_copy_count: frame_count,
                copied_bytes: source.payload_bytes(),
                peak_scratch_bytes: page_bytes,
                manifest_blocks_read: 1,
                manifest_comparisons: 0,
                manifest_bytes_read: 0,
            },
        };
        Ok((data, root))
    }

    fn current_extent_source(
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

    fn encode_extent_frames(
        &self,
        manifest: &DurableExtentManifest,
        destination: RecordArtifactFile,
        payload: &[u8],
    ) -> Result<(Vec<PreparedPhysicalDataFrame>, u64), RecordAppendError> {
        let format = self.format.declaration();
        let transfer = manifest.chunk_payload_capacity() as usize;
        let mut frames = Vec::with_capacity(manifest.chunk_count() as usize);
        let mut completed = 0_usize;
        let mut artifact_offset = 0_u64;
        for ordinal in 1..=manifest.chunk_count() {
            let expected = (payload.len() - completed).min(transfer);
            let coordinate = ExtentChunkCoordinate::new(
                manifest.record(),
                manifest.extent_cell(),
                manifest.logical_bytes(),
                completed as u64,
                ordinal,
            )
            .ok_or_else(damaged)?;
            let mut chunk =
                prepare_extent_chunk(format, coordinate, expected).map_err(|_| damaged())?;
            chunk
                .payload_mut()
                .copy_from_slice(&payload[completed..completed + expected]);
            let bytes = chunk.seal();
            let length = u32::try_from(bytes.len()).map_err(|_| damaged())?;
            let target = PhysicalDataFrameIdentity::extent_chunk(
                coordinate,
                destination,
                artifact_offset,
                length,
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
            artifact_offset = artifact_offset
                .checked_add(u64::from(length))
                .ok_or_else(damaged)?;
        }
        if completed != payload.len() {
            return Err(damaged());
        }
        Ok((frames, artifact_offset))
    }
}

fn extent_rewrite_basis_digest(source: DurableExtentRecordPlacement) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(b"store.physical.extent-rewrite-basis.v1");
    digest.update(source.record().allocation_epoch());
    digest.update(source.record().ordinal().to_le_bytes());
    digest.update(source.extent().get().to_le_bytes());
    digest.update(source.extent_generation().to_le_bytes());
    digest.update(source.payload_bytes().to_le_bytes());
    digest.finalize().into()
}
