//! Re-record contradictory current roster facts without bypassing frame admission.
use super::RelationalNativeCheckpoint;
use crate::{
    durability::{data::*, log::native_checkpoint_codec as codec},
    history::data::CommitId,
};
mod commitments;
impl RelationalNativeCheckpoint {
    /// Closed certification corruption. The result is untrusted checkpoint bytes,
    /// never admitted state or a metadata-writing grant. Zero drops a route, one
    /// adds a foreign route and two re-points its source generation.
    #[doc(hidden)]
    pub fn corrupt_current_publication_roster_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        publication: CommitId,
        change: u8,
    ) -> Result<Self, RelationalRecoveryStop> {
        self.corrupt_with_placement(
            execution,
            publication,
            change,
            &RelationalCheckpointArtifactPolicy::Embedded,
        )
    }
    fn corrupt_with_placement(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        publication: CommitId,
        change: u8,
        placement: &RelationalCheckpointArtifactPolicy<'_>,
    ) -> Result<Self, RelationalRecoveryStop> {
        if change > 10 {
            return Err(DurabilityError::new(
                RecoveryFailureClass::CorruptCheckpoint,
                "unknown roster corruption",
            )
            .into());
        }
        let mut checkpoint = codec::decode_checkpoint(execution, self.bytes())?.checkpoint;
        let storage = checkpoint
            .branch_root_schema_images
            .iter()
            .find_map(|image| image.readmit_registry().ok()?.0.current_publication_storage)
            .ok_or_else(|| {
                DurabilityError::new(
                    RecoveryFailureClass::CorruptCheckpoint,
                    "real current ownership grammar absent",
                )
            })?;
        let mut changed = false;
        for root in &mut checkpoint.branch_roots {
            for partition in &mut root.partition_images {
                changed |= crate::publication::ownership::corruption::alter(
                    partition,
                    publication,
                    change,
                    &storage,
                );
            }
            root.partition_image_digest =
                crate::storage::substrate::maintenance::ReconstructionLane::enter_reconstruction(
                    |lane| branch_root_partition_image_digest(lane, &root.partition_images),
                )
                .map_err(|error| {
                    DurabilityError::new(RecoveryFailureClass::CorruptCheckpoint, error.to_string())
                })?;
            root.root_image_digest =
                crate::storage::substrate::maintenance::ReconstructionLane::enter_reconstruction(
                    |lane| {
                        branch_root_image_digest(
                            lane,
                            root.format_version,
                            root.commit_id,
                            root.partition_image_digest,
                            root.schema_carrier_digest,
                            &root.content_records,
                        )
                    },
                );
        }
        for partition in &mut checkpoint.partition_images {
            changed |= crate::publication::ownership::corruption::alter(
                partition,
                publication,
                change,
                &storage,
            );
        }
        if !changed {
            return Err(DurabilityError::new(
                RecoveryFailureClass::CorruptCheckpoint,
                "selected current roster route is absent",
            )
            .into());
        }
        checkpoint
            .mirror_kind_images
            .omit_deleted_current_records(&checkpoint.partition_images);
        if change == 9 {
            checkpoint
                .mirror_kind_images
                .repoint_current_claims_for_test(&checkpoint.partition_images, storage.member_link);
            for root in &checkpoint.branch_roots {
                checkpoint
                    .root_kind_images
                    .get_mut(&root.commit_id)
                    .expect("real captured root kind image")
                    .repoint_current_claims_for_test(&root.partition_images, storage.member_link);
            }
        }
        crate::storage::substrate::maintenance::ReconstructionLane::enter_reconstruction(|lane| {
            commitments::rebind(lane, execution, &mut checkpoint)
        })?;
        let mut answer = None;
        let handoff = execution.run_native_with_checkpoint(|stage| {
            let mut control = codec::CheckpointEncodingControl::Original {
                execution: *execution,
                stage,
            };
            answer = Some((|| {
                let policy = execution.allocation_policy();
                let captured = crate::storage::substrate::maintenance::ReconstructionLane::enter_reconstruction(|lane| codec::CapturedManifest::capture_expanded(
                    lane, execution, &checkpoint, policy, &mut control,
                ))?;
                let (bytes, sections, members, _) =
                    codec::encode_captured(captured.seal_ordinary(), execution, policy, placement, &mut control)?;
                Ok::<_, RelationalNativeCheckpointCaptureDenial>((bytes, sections, members))
            })());
        });
        let (bytes, sections, members) = answer.ok_or_else(|| {
            DurabilityError::new(
                RecoveryFailureClass::CorruptCheckpoint,
                "roster recorder did not execute",
            )
        })??;
        handoff.map_err(|cause| RelationalRecoveryStop::Request(cause.into()))?;
        let native = if placement.is_embedded() {
            Self::from_untrusted_bytes_region(bytes.clone(), 0..bytes.len())
                .expect("the recorder emitted the selected range")
        } else {
            let position = self.covered_position().ok_or_else(|| {
                DurabilityError::new(
                    RecoveryFailureClass::CorruptCheckpoint,
                    "external corruption requires an actual captured checkpoint",
                )
            })?;
            let transitions = self.covered_branch_transitions().ok_or_else(|| {
                DurabilityError::new(
                    RecoveryFailureClass::CorruptCheckpoint,
                    "external corruption lost recorded branch coverage",
                )
            })?;
            Self::from_captured_bytes(bytes, sections, position, transitions)
        };
        Ok(native.with_captured_members(members))
    }
}
