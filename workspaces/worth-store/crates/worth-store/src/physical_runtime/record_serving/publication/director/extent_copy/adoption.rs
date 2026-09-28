use super::{
    super::{selected_segment_rewrite::damaged, RecordPublicationDirector},
    CompletedExtentCopy,
};
use crate::physical_runtime::durability::{PreparedPhysicalDataPlan, RetiredArtifact};
use crate::physical_runtime::record_serving::publication::append_observation::PublicationObservation;
use crate::physical_runtime::record_serving::{PreparedPhysicalRootProjection, RecordAppendError};
use crate::physical_runtime::PreparedPhysicalMutation;
use std::collections::BTreeMap;
use std::num::NonZeroU64;
use worth_store_physical_format::CurrentPhysicalRecordPlacement;

impl RecordPublicationDirector {
    /// Called only after the normal publication owner has acquired its short
    /// exclusive pending scope. The copy's original root is not rebased redo.
    pub(in crate::physical_runtime::record_serving::publication::director) fn build_extent_copy_adoption(
        &self,
        prepared: &mut PreparedPhysicalMutation,
        copy: CompletedExtentCopy,
    ) -> Result<
        (PreparedPhysicalDataPlan, PreparedPhysicalRootProjection),
        (CompletedExtentCopy, RecordAppendError),
    > {
        let (root, _) = self.root_owner.snapshot();
        let intent = copy.intent();
        let current = match self.current_extent_source(&root, intent.source().record()) {
            Ok(current) => current,
            Err(error) => return Err((copy, error)),
        };
        if prepared.extent_copy_source() != Some(intent.source())
            || prepared.idempotency_identity().bytes() != intent.operation()
            || copy
                .validate_adoption(current, prepared.mutation_identity().runtime_identity())
                .is_err()
        {
            return Err((copy, damaged()));
        }
        let page_bytes = u64::from(self.format.declaration().page_size().bytes());
        let allocation_bytes =
            NonZeroU64::new(page_bytes * 3).expect("admitted page geometry is nonzero");
        let (proof, reservation) = copy.into_adoption();
        self.root_owner.note_displaced(
            root.generation(),
            RetiredArtifact::Extent {
                extent: current.extent().get(),
                generation: current.extent_generation(),
                range: current.arena_range(),
            },
            current.arena_range().length(),
        );
        prepared.rebase_extent_copy_root(root.generation());
        let mut placements = BTreeMap::new();
        placements.insert(
            current.record(),
            CurrentPhysicalRecordPlacement::Extent(intent.destination()),
        );
        let projection = PreparedPhysicalRootProjection {
            arena_reservations: vec![reservation],
            root_publication_allocation_bytes: allocation_bytes,
            source_root: root,
            manifest_capacity_transition: prepared.manifest_capacity_transition(),
            placement: prepared.placement(),
            records: vec![current.record()],
            inserted_records: 0,
            // Manifest and data already have exact copy receipts and both
            // synchronizations; final publication must not rewrite either.
            payload_manifests: Vec::new(),
            placements,
            segment_updates: BTreeMap::new(),
            inline_allocations: Vec::new(),
            last_inline_record: None,
            last_inline_segment: None,
            requires_maintenance_protocol: true,
            observation: PublicationObservation {
                records: 1,
                logical_bytes: current.payload_bytes(),
                completed_bytes: 0,
                segment_artifacts: 0,
                extent_artifacts: 1,
                transfer_count: u64::from(intent.chunk_count()),
                peak_transfer_width: page_bytes,
                explicit_copy_count: u64::from(intent.chunk_count()),
                copied_bytes: current.payload_bytes(),
                peak_scratch_bytes: page_bytes,
                manifest_blocks_read: 0,
                manifest_comparisons: 0,
                manifest_bytes_read: 0,
            },
        };
        Ok((PreparedPhysicalDataPlan::SourceCopy(proof), projection))
    }
}

impl super::super::PhysicalRecordSubmission {
    /// Takes the completed bounded copy into the ordinary mutation pipeline.
    /// The latest root is captured only when this prepared mutation executes.
    pub fn prepare_completed_extent_copy(
        &self,
    ) -> Result<PreparedPhysicalMutation, crate::physical_runtime::RecordAppendDenial> {
        let director = self
            .director
            .upgrade()
            .ok_or(crate::physical_runtime::RecordAppendDenial::PublicationAuthorityReleased)?;
        let (prepared, copy) =
            director
                .take_completed_extent_copy()
                .map_err(|error| match error {
                    RecordAppendError::Denied(denial) => denial,
                    _ => crate::physical_runtime::RecordAppendDenial::PublishedLayoutDamaged,
                })?;
        Ok(prepared.attach_completed_extent_copy(copy))
    }
}
