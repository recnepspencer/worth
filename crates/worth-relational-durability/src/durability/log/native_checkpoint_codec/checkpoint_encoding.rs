use super::manifest_capture::CapturedManifest;
use crate::durability::data::DurableCheckpoint;
use crate::durability::data::NativeCheckpointSectionBytes;
use contracts::durability::data::native_checkpoint_capture_denial::RelationalNativeCheckpointCaptureDenial;
use worth_execution::ExecutionRequestArrayBuilder;
use worth_execution::{ExecutionAllocationPolicy, ExecutionImmutableBytes};
use worth_relational_contracts::internal_port as contracts;
use worth_relational_storage::internal_port::storage::data::checkpoint_member::RelationalCheckpointMember;
pub mod writer;
pub(crate) use writer::CheckpointEncodingControl;

/// Encode changed DATA for hostile codec fixtures, with its external members;
/// never an original-source capture or publication witness.
#[doc(hidden)]
pub fn encode_checkpoint_members(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    checkpoint: &DurableCheckpoint,
    policy: ExecutionAllocationPolicy<'_, '_>,
) -> Result<
    crate::durability::data::RelationalNativeCheckpoint,
    RelationalNativeCheckpointCaptureDenial,
> {
    policy.check_live()?;
    let mut answer = None;
    let handoff = execution.run_native_with_checkpoint(|stage| {
        let mut control = CheckpointEncodingControl {
            execution: *execution,
            stage,
        };
        answer = Some((|| {
            let mut captured =
                contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane::enter_reconstruction(
                    |lane| {
                        CapturedManifest::capture_expanded(
                            lane,
                            execution,
                            checkpoint,
                            policy,
                            &mut control,
                        )
                    },
                )?;
            populate_expanded_canonical_members(&mut captured, checkpoint, execution)?;
            let (bytes, sections, members, _) =
                encode_captured(captured.seal_ordinary(), execution, &mut control)?;
            Ok::<_, RelationalNativeCheckpointCaptureDenial>((bytes, sections, members))
        })());
    });
    let encoded = answer.ok_or_else(|| {
        super::manifest_capture::corrupt("checkpoint member encoder did not execute")
    })??;
    handoff.map_err(|cause| RelationalNativeCheckpointCaptureDenial::Request(cause.into()))?;
    let (bytes, _sections, members) = encoded;
    Ok(
        crate::durability::data::RelationalNativeCheckpoint::from_untrusted_bytes(
            bytes.bytes().to_vec(),
        )
        .with_captured_members(members),
    )
}

pub(crate) fn populate_expanded_canonical_members(
    captured: &mut CapturedManifest,
    checkpoint: &DurableCheckpoint,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<(), RelationalNativeCheckpointCaptureDenial> {
    let memory =
        crate::durability::checkpoint_history::inventory::CanonicalCheckpointMembers::reserve(
            checkpoint.envelopes.len(),
            execution,
        )
        .map_err(RelationalNativeCheckpointCaptureDenial::Request)?;
    let mut rows = Vec::with_capacity(checkpoint.envelopes.len());
    let mut payloads =
        ExecutionRequestArrayBuilder::allocate(*execution, checkpoint.envelopes.len()).map_err(
            contracts::durability::data::native_checkpoint_capture_denial::array_capture_stop,
        )?;
    for commit in &checkpoint.envelopes {
        let prepared =
            crate::durability::canonical_commit_artifact::PreparedOriginalCanonicalPayload::capture(
                commit.canonical_arc().clone(),
                execution,
            )
            .map_err(contracts::durability::data::native_checkpoint_capture_denial::capture_stop)?;
        let artifact =
            crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact::from_memory_publication(
                commit.position(),
                &prepared,
                execution,
            )
            .map_err(|cause| match cause {
                crate::durability::canonical_commit_artifact::MemoryFramePreparationDenial::Request(
                    cause,
                ) => RelationalNativeCheckpointCaptureDenial::Request(cause.into()),
                crate::durability::canonical_commit_artifact::MemoryFramePreparationDenial::Inner(
                    cause,
                ) => cause.into(),
            })?;
        let bytes = artifact.into_bytes();
        let member = worth_relational_storage::internal_port::storage::data::member_frame::locator(
            bytes.bytes(),
        );
        rows.push(
            crate::durability::checkpoint_history::inventory::CanonicalCheckpointMember {
                position: commit.position(),
                member,
            },
        );
        payloads
            .push(RelationalCheckpointMember::New {
                locator: member,
                bytes,
            })
            .map_err(
                contracts::durability::data::native_checkpoint_capture_denial::array_capture_stop,
            )?;
    }
    captured.manifest.canonical_members =
        crate::durability::checkpoint_history::inventory::CanonicalCheckpointMembers::from_reserved(
            rows, memory,
        );
    captured.canonical_payloads = Some(payloads.seal().map_err(
        contracts::durability::data::native_checkpoint_capture_denial::array_capture_stop,
    )?);
    Ok(())
}

pub(crate) fn encode_captured(
    mut captured: CapturedManifest<crate::durability::data::RelationalCheckpointManifest>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    control: &mut CheckpointEncodingControl<'_, '_, '_, '_, '_>,
) -> Result<
    (
        ExecutionImmutableBytes,
        NativeCheckpointSectionBytes,
        crate::durability::data::CapturedCheckpointMembers,
        Vec<worth_relational_storage::internal_port::storage::residency::checkpoint_capture::RelationalCheckpointRegionPublication>,
    ),
    RelationalNativeCheckpointCaptureDenial,
>{
    let ordinary_members = captured.members.len();
    let canonical = captured.canonical_payloads.take();
    let count = ordinary_members
        .checked_add(canonical.as_ref().map_or(0, |rows| rows.len()))
        .ok_or_else(|| writer::frame_mismatch("captured member list overflow"))?;
    let members = crate::durability::data::CapturedCheckpointMembers::capture(
        execution,
        count,
        captured
            .members
            .into_values()
            .chain(canonical.into_iter().flat_map(IntoIterator::into_iter)),
    )?;
    let (bytes, sections) = super::complete_encoding::encode_in_stage(&captured.manifest, control)?;
    Ok((bytes, sections, members, captured.publications))
}
