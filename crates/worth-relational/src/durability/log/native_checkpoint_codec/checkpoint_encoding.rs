use super::checkpoint_wire::{ByteString, CheckpointWire};
use super::manifest_capture::CapturedManifest;
#[cfg(test)]
use crate::durability::data::DurableCheckpoint;
use crate::durability::data::{
    NativeCheckpointSectionBytes, RelationalCheckpointArtifactPolicy, RelationalCheckpointMember,
    RelationalNativeCheckpointCaptureDenial,
};
#[cfg(test)]
use worth_execution::ExecutionRequestArrayBuilder;
use worth_execution::{ExecutionAllocationPolicy, ExecutionImmutableBytes};
mod writer;
pub(crate) use writer::{copy_bytes, count_value, encode_value, CheckpointEncodingControl};

#[cfg(test)]
pub(crate) fn encode_checkpoint(
    checkpoint: &DurableCheckpoint,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    policy: ExecutionAllocationPolicy<'_, '_>,
) -> Result<
    (ExecutionImmutableBytes, NativeCheckpointSectionBytes),
    RelationalNativeCheckpointCaptureDenial,
> {
    let (bytes, sections, _) = encode_checkpoint_members(
        checkpoint,
        execution,
        policy,
        &RelationalCheckpointArtifactPolicy::Embedded,
    )?;
    Ok((bytes, sections))
}

#[cfg(test)]
pub(crate) fn encode_checkpoint_members(
    checkpoint: &DurableCheckpoint,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    policy: ExecutionAllocationPolicy<'_, '_>,
    placement: &RelationalCheckpointArtifactPolicy<'_>,
) -> Result<
    (
        ExecutionImmutableBytes,
        NativeCheckpointSectionBytes,
        crate::durability::data::CapturedCheckpointMembers,
    ),
    RelationalNativeCheckpointCaptureDenial,
> {
    policy.check_live()?;
    let mut captured = CapturedManifest::capture(checkpoint, policy)?;
    // These existing hostile codec fixtures deliberately encode changed DATA.
    // They are never an original-source capture or publication witness.
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
        let prepared = crate::durability::canonical_commit_artifact::PreparedOriginalCanonicalPayload::capture(commit.canonical_arc().clone(), execution)
            .map_err(crate::durability::checkpoint_history::capture::capture_stop)?;
        let artifact = crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact::from_memory_publication(commit.position(), &prepared, execution)
            .map_err(|cause| match cause {
                crate::durability::canonical_commit_artifact::MemoryFramePreparationDenial::Request(cause) => RelationalNativeCheckpointCaptureDenial::Request(cause.into()),
                crate::durability::canonical_commit_artifact::MemoryFramePreparationDenial::Artifact(cause) => cause.into(),
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
    let (bytes, sections, members, _) = encode_captured(
        captured.seal_ordinary(),
        execution,
        policy,
        placement,
        &mut CheckpointEncodingControl::Policy(policy),
    )?;
    Ok((bytes, sections, members))
}

pub(crate) fn encode_captured(
    mut captured: CapturedManifest<crate::durability::data::RelationalCheckpointManifest>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    policy: ExecutionAllocationPolicy<'_, '_>,
    placement: &RelationalCheckpointArtifactPolicy<'_>,
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
    let embedded = placement.is_embedded();
    if embedded {
        for member in captured.members.values_mut() {
            policy.check_live()?;
            if let RelationalCheckpointMember::Carried { locator } = member {
                let locator = *locator;
                let _read = control.reserve_read(locator.length)?;
                let bytes = if let Some((subject, source)) = captured.guarded_members.get(&locator)
                {
                    source
                        .read_member(locator, locator.length)
                        .map_err(|cause| {
                            RelationalNativeCheckpointCaptureDenial::from(
                                crate::storage::data::RelationalStorageUnavailable {
                                    subject: *subject,
                                    cause,
                                },
                            )
                        })?
                } else {
                    let source = captured.cold_source.as_ref().ok_or_else(|| {
                        super::manifest_capture::corrupt(
                            "inline capture lacks its carried member source",
                        )
                    })?;
                    source
                        .read_member(locator, locator.length)
                        .map_err(|cause| {
                            super::manifest_capture::corrupt(format!(
                                "carried inline member: {cause:?}"
                            ))
                        })?
                };
                policy.check_live()?;
                if control.locator(&bytes)? != locator {
                    return Err(super::manifest_capture::corrupt(
                        "carried inline member integrity mismatch",
                    )
                    .into());
                }
                *member = RelationalCheckpointMember::New {
                    locator,
                    bytes: copy_bytes(control, &bytes)?,
                };
            }
        }
    }
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
    let borrowed_count = if embedded { count } else { 0 };
    let borrowed_bytes = std::alloc::Layout::array::<(
        crate::durability::medium::RelationalDurableMemberLocator,
        ByteString<'_>,
    )>(borrowed_count)
    .map_err(|_| writer::frame_mismatch("borrowed member table size overflow"))?
    .size();
    let _wire_memory = execution
        .reserve_retained_memory(borrowed_bytes as u64)
        .map_err(|cause| RelationalNativeCheckpointCaptureDenial::Request(cause.into()))?;
    let mut inline = Vec::with_capacity(borrowed_count);
    if embedded {
        for member in members.entries() {
            let RelationalCheckpointMember::New { locator, bytes } = member else {
                return Err(writer::frame_mismatch(
                    "inline capture has unresolved carried members",
                ));
            };
            inline.push((*locator, ByteString(bytes.bytes())));
        }
    }
    let wire = CheckpointWire {
        manifest: captured.manifest,
        members: inline,
        envelopes: Vec::new(),
    };
    let bytes = encode_value(control, super::CHECKPOINT_MAGIC, &wire)?;
    let total = bytes.len();
    // Count the exact sections without constructing another payload copy.
    let branch_roots = count_value(control, &wire.manifest.roots)?;
    let branch_cells = count_value(control, &wire.manifest.branch_cells)?;
    let partition_mirror = count_value(control, &wire.manifest.mirror)?;
    let envelopes = if embedded {
        let mut total = 0usize;
        for member in members.entries().iter().skip(ordinary_members) {
            let RelationalCheckpointMember::New { bytes, .. } = member else {
                return Err(writer::frame_mismatch("canonical original missing"));
            };
            total = total
                .checked_add(count_value(control, &ByteString(bytes.bytes()))?)
                .ok_or_else(|| writer::frame_mismatch("canonical section count overflow"))?;
        }
        total
    } else {
        0
    };
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
