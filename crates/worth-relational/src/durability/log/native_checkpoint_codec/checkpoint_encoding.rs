use super::checkpoint_wire::CheckpointWire;
use super::manifest_capture::CapturedManifest;
use crate::durability::data::DurableCheckpoint;
use crate::durability::data::{
    NativeCheckpointSectionBytes, RelationalCheckpointMember,
    RelationalNativeCheckpointCaptureDenial,
};
#[cfg(test)]
use worth_execution::ExecutionAllocationPolicy;
use worth_execution::ExecutionImmutableBytes;
use worth_execution::ExecutionRequestArrayBuilder;
mod writer;
pub(crate) use writer::{count_value, encode_value, CheckpointEncodingControl};

#[cfg(test)]
pub(crate) fn encode_checkpoint_members(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    checkpoint: &DurableCheckpoint,
    policy: ExecutionAllocationPolicy<'_, '_>,
) -> Result<
    (
        ExecutionImmutableBytes,
        NativeCheckpointSectionBytes,
        crate::durability::data::CapturedCheckpointMembers,
    ),
    RelationalNativeCheckpointCaptureDenial,
> {
    policy.check_live()?;
    let mut captured =
        crate::storage::substrate::maintenance::ReconstructionLane::enter_reconstruction(|lane| {
            CapturedManifest::capture(lane, execution, checkpoint, policy)
        })?;
    // These existing hostile codec fixtures deliberately encode changed DATA.
    // They are never an original-source capture or publication witness.
    populate_expanded_canonical_members(&mut captured, checkpoint, execution)?;
    let (bytes, sections, members, _) = encode_captured(
        captured.seal_ordinary(),
        execution,
        &mut CheckpointEncodingControl::Policy(policy),
    )?;
    Ok((bytes, sections, members))
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
        ExecutionRequestArrayBuilder::allocate(*execution, checkpoint.envelopes.len())
            .map_err(crate::durability::checkpoint_history::capture::array_capture_stop)?;
    for commit in &checkpoint.envelopes {
        let prepared =
            crate::durability::canonical_commit_artifact::PreparedOriginalCanonicalPayload::capture(
                commit.canonical_arc().clone(),
                execution,
            )
            .map_err(crate::durability::checkpoint_history::capture::capture_stop)?;
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
                crate::durability::canonical_commit_artifact::MemoryFramePreparationDenial::Artifact(
                    cause,
                ) => cause.into(),
            })?;
        let bytes = artifact.into_bytes();
        let member = super::member_frame::locator(bytes.bytes());
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
            .map_err(crate::durability::checkpoint_history::capture::array_capture_stop)?;
    }
    captured.manifest.canonical_members =
        crate::durability::checkpoint_history::inventory::CanonicalCheckpointMembers::from_reserved(
            rows, memory,
        );
    captured.canonical_payloads = Some(
        payloads
            .seal()
            .map_err(crate::durability::checkpoint_history::capture::array_capture_stop)?,
    );
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
        Vec<crate::storage::residency::RelationalCheckpointRegionPublication>,
    ),
    RelationalNativeCheckpointCaptureDenial,
> {
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
    let wire = CheckpointWire {
        manifest: captured.manifest,
        members: Vec::new(),
        envelopes: Vec::new(),
    };
    let bytes = encode_value(control, super::CHECKPOINT_MAGIC, &wire)?;
    let total = bytes.len();
    // Count the exact sections without constructing another payload copy.
    let branch_roots = count_value(control, &wire.manifest.roots)?;
    let branch_cells = count_value(control, &wire.manifest.branch_cells)?;
    let partition_mirror = count_value(control, &wire.manifest.mirror)?;
    let envelopes = 0usize;
    let derived_indexes = count_value(control, &wire.manifest.index_definitions)?;
    let classified = [
        branch_roots,
        branch_cells,
        partition_mirror,
        envelopes,
        derived_indexes,
    ]
    .into_iter()
    .try_fold(0_usize, |total, size| total.checked_add(size))
    .ok_or_else(|| writer::frame_mismatch("native checkpoint section accounting overflow"))?;
    let sections = NativeCheckpointSectionBytes {
        total,
        envelopes,
        branch_roots,
        branch_cells,
        partition_mirror,
        derived_indexes,
        framing_and_metadata: total.checked_sub(classified).ok_or_else(|| {
            writer::frame_mismatch("native checkpoint section accounting overflow")
        })?,
    };
    drop(wire);
    control.checkpoint(0)?;
    Ok((bytes, sections, members, captured.publications))
}
