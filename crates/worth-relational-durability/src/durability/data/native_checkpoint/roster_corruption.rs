//! Re-record contradictory current roster facts without bypassing frame admission.
use super::RelationalNativeCheckpoint;
use crate::durability::data::*;
use crate::durability::log::native_checkpoint_codec as codec;
use contracts::history::data::coordinates::CommitId;
use contracts::schema::data::RelationalSchemaRegistryFieldsPort as _;
use worth_relational_contracts::internal_port as contracts;
pub mod commitments;
pub mod portable_rows;
impl RelationalNativeCheckpoint {
    pub(crate) fn corrupt_current_publication_roster_external_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: std::sync::Arc<dyn worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableColdSource>,
        publication: CommitId,
        change: u8,
    ) -> Result<Self, RelationalRecoveryStop> {
        if change > 10 {
            return Err(DurabilityError::new(
                RecoveryFailureClass::CorruptCheckpoint,
                "unknown roster corruption",
            )
            .into());
        }
        let mut checkpoint =
            codec::decode_checkpoint_from_source(execution, self.bytes(), source)?.checkpoint;
        let storage = checkpoint
            .branch_root_schema_images
            .iter()
            .find_map(|image| {
                image
                    .readmit_registry()
                    .ok()?
                    .0
                    .into_current_publication_storage()
            })
            .ok_or_else(|| {
                DurabilityError::new(
                    RecoveryFailureClass::CorruptCheckpoint,
                    "real current ownership grammar absent",
                )
            })?;
        let mut changed = false;
        for root in &mut checkpoint.branch_roots {
            for partition in &mut root.partition_images {
                changed |= portable_rows::alter(partition, publication, change, &storage);
            }
            root.partition_image_digest =
                contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane::enter_reconstruction(
                    |lane| branch_root_partition_image_digest(lane, &root.partition_images),
                )
                .map_err(|error| {
                    DurabilityError::new(RecoveryFailureClass::CorruptCheckpoint, error.to_string())
                })?;
            root.root_image_digest =
                contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane::enter_reconstruction(
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
            changed |= portable_rows::alter(partition, publication, change, &storage);
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
        contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane::enter_reconstruction(|lane| {
            commitments::rebind(lane, execution, &mut checkpoint)
        })?;
        let mut answer = None;
        let handoff = execution.run_native_with_checkpoint(|stage| {
            let mut control = codec::CheckpointEncodingControl {
                execution: *execution,
                stage,
            };
            answer = Some((|| {
                let policy = execution.allocation_policy();
                let mut captured =
                    contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane::enter_reconstruction(
                        |lane| {
                            codec::CapturedManifest::capture_expanded(
                                lane,
                                execution,
                                &checkpoint,
                                policy,
                                &mut control,
                            )
                        },
                    )?;
                codec::populate_expanded_canonical_members(&mut captured, &checkpoint, execution)?;
                let (bytes, sections, members, _) =
                    codec::encode_captured(captured.seal_ordinary(), execution, &mut control)?;
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
        let native = Self::from_captured_bytes(bytes, sections, position, transitions);
        Ok(native.with_captured_members(members))
    }
}

#[doc(hidden)]
pub trait RelationalNativeCheckpointOwnerPort: Sized {
    fn corrupt_current_publication_roster_external_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: std::sync::Arc<dyn worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableColdSource>,
        publication: CommitId,
        change: u8,
    ) -> Result<Self, RelationalRecoveryStop>;
}

impl RelationalNativeCheckpointOwnerPort for RelationalNativeCheckpoint {
    fn corrupt_current_publication_roster_external_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: std::sync::Arc<dyn worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableColdSource>,
        publication: CommitId,
        change: u8,
    ) -> Result<Self, RelationalRecoveryStop> {
        self.corrupt_current_publication_roster_external_for_test(
            execution,
            source,
            publication,
            change,
        )
    }
}
