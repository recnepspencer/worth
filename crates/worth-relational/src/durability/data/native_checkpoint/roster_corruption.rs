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
        if change > 2 {
            return Err(DurabilityError::new(
                RecoveryFailureClass::CorruptCheckpoint,
                "unknown roster corruption",
            )
            .into());
        }
        let mut checkpoint = codec::decode_checkpoint(execution, self.bytes())?.checkpoint;
        let mut changed = false;
        for root in &mut checkpoint.branch_roots {
            for partition in &mut root.partition_images {
                changed |= crate::publication::ownership::corruption::alter(
                    partition,
                    publication,
                    change,
                );
            }
            root.partition_image_digest =
                branch_root_partition_image_digest(&root.partition_images).map_err(|error| {
                    DurabilityError::new(RecoveryFailureClass::CorruptCheckpoint, error.to_string())
                })?;
            root.root_image_digest = branch_root_image_digest(
                root.format_version,
                root.commit_id,
                root.partition_image_digest,
                root.schema_carrier_digest,
                &root.content_records,
            );
        }
        for partition in &mut checkpoint.partition_images {
            changed |=
                crate::publication::ownership::corruption::alter(partition, publication, change);
        }
        if !changed {
            return Err(DurabilityError::new(
                RecoveryFailureClass::CorruptCheckpoint,
                "selected current roster route is absent",
            )
            .into());
        }
        commitments::rebind(&mut checkpoint)?;
        let mut answer = None;
        let handoff = execution.run_native_with_checkpoint(|stage| {
            let mut control = codec::CheckpointEncodingControl::Original {
                execution: *execution,
                stage,
            };
            answer = Some((|| {
                let policy = execution.allocation_policy();
                let captured =
                    codec::CapturedManifest::capture_expanded(&checkpoint, policy, &mut control)?;
                let (bytes, _, _, _) = codec::encode_captured(
                    captured,
                    policy,
                    &RelationalCheckpointArtifactPolicy::Embedded,
                    &mut control,
                )?;
                Ok::<_, RelationalNativeCheckpointCaptureDenial>(bytes)
            })());
        });
        let bytes = answer.ok_or_else(|| {
            DurabilityError::new(
                RecoveryFailureClass::CorruptCheckpoint,
                "roster recorder did not execute",
            )
        })??;
        handoff.map_err(|cause| RelationalRecoveryStop::Request(cause.into()))?;
        Ok(
            Self::from_untrusted_bytes_region(bytes.clone(), 0..bytes.len())
                .expect("the recorder emitted the selected range"),
        )
    }
}
