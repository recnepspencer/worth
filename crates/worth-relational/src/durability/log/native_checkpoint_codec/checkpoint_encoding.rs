use super::checkpoint_wire::{ByteString, CheckpointWire};
use super::manifest_capture::CapturedManifest;
#[cfg(test)]
use crate::durability::data::DurableCheckpoint;
use crate::durability::data::{
    NativeCheckpointSectionBytes, RelationalCheckpointArtifactPolicy, RelationalCheckpointMember,
    RelationalNativeCheckpointCaptureDenial,
};
use std::io::Write;
use worth_execution::{
    ExecutionAllocationPolicy, ExecutionImmutableBytes, ExecutionRequestArrayBuilder,
};
mod writer;
use writer::{frame_mismatch, EmissionWriter, SizingWriter};

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
        let artifact=crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact::from_memory_publication(commit.position(),commit.envelope(),execution)?;
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
    let (bytes, sections, members, _) =
        encode_captured(captured.seal_ordinary(), execution, policy, placement)?;
    Ok((bytes, sections, members))
}

pub(crate) fn encode_captured(
    mut captured: CapturedManifest<crate::durability::data::RelationalCheckpointManifest>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    policy: ExecutionAllocationPolicy<'_, '_>,
    placement: &RelationalCheckpointArtifactPolicy<'_>,
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
                let bytes = if let Some((subject, source)) = captured.guarded_members.get(&locator)
                {
                    source.read_member(locator, 1 << 30).map_err(|cause| {
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
                    source.read_member(locator, 1 << 30).map_err(|cause| {
                        super::manifest_capture::corrupt(format!(
                            "carried inline member: {cause:?}"
                        ))
                    })?
                };
                policy.check_live()?;
                if super::member_frame::locator(&bytes) != locator {
                    return Err(super::manifest_capture::corrupt(
                        "carried inline member integrity mismatch",
                    )
                    .into());
                }
                *member = RelationalCheckpointMember::New {
                    locator,
                    bytes: ExecutionImmutableBytes::from_external_bytes(std::sync::Arc::new(
                        bytes.into_boxed_slice(),
                    )),
                };
            }
        }
    }
    let ordinary_members = captured.members.len();
    let canonical = captured.canonical_payloads.take();
    let count = ordinary_members
        .checked_add(canonical.as_ref().map_or(0, |rows| rows.len()))
        .ok_or_else(|| frame_mismatch("captured member list overflow"))?;
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
    .map_err(|_| frame_mismatch("borrowed member table size overflow"))?
    .size();
    let _wire_memory = execution
        .reserve_retained_memory(borrowed_bytes as u64)
        .map_err(|cause| RelationalNativeCheckpointCaptureDenial::Request(cause.into()))?;
    let mut inline = Vec::with_capacity(borrowed_count);
    if embedded {
        for member in members.entries() {
            let RelationalCheckpointMember::New { locator, bytes } = member else {
                return Err(frame_mismatch(
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
    let mut sizing = SizingWriter::new(*execution);
    let counted = rmp_serde::encode::write_named(&mut sizing, &wire);
    let total = sizing
        .finish(counted)?
        .checked_add(super::CHECKPOINT_MAGIC.len())
        .ok_or_else(|| frame_mismatch("native checkpoint frame size overflow"))?;
    let mut bytes = ExecutionRequestArrayBuilder::<u8>::allocate(*execution, total)
        .map_err(crate::durability::checkpoint_history::capture::array_capture_stop)?;
    let mut emission = EmissionWriter::new(&mut bytes);
    let emitted = emission.write_all(super::CHECKPOINT_MAGIC).and_then(|()| {
        rmp_serde::encode::write_named(&mut emission, &wire).map_err(std::io::Error::other)
    });
    emission.finish(emitted)?;
    if bytes.len() != total {
        return Err(frame_mismatch(
            "native checkpoint count and emission lengths differ",
        ));
    }
    // These counters measure the emitted value sections, including inline
    // payloads only when the artifact policy selects them.
    let branch_roots = rmp_serde::to_vec_named(&wire.manifest.roots)
        .map_err(writer::encoding_failure)?
        .len();
    let branch_cells = rmp_serde::to_vec_named(&wire.manifest.branch_cells)
        .map_err(writer::encoding_failure)?
        .len();
    let partition_mirror = rmp_serde::to_vec_named(&wire.manifest.mirror)
        .map_err(writer::encoding_failure)?
        .len();
    let envelopes = if embedded {
        let mut count = SizingWriter::new(*execution);
        for member in members.entries().iter().skip(ordinary_members) {
            let RelationalCheckpointMember::New { bytes, .. } = member else {
                return Err(frame_mismatch("canonical original missing"));
            };
            let result = serde::Serialize::serialize(
                &ByteString(bytes.bytes()),
                &mut rmp_serde::Serializer::new(&mut count),
            );
            if let Err(cause) = result {
                return Err(writer::encoding_failure(cause));
            }
        }
        count.finish(Ok::<(), std::io::Error>(()))?
    } else {
        0
    };
    let derived_indexes = rmp_serde::to_vec_named(&wire.manifest.index_definitions)
        .map_err(writer::encoding_failure)?
        .len();
    let classified = branch_roots + branch_cells + partition_mirror + envelopes + derived_indexes;
    let sections = NativeCheckpointSectionBytes {
        total,
        envelopes,
        branch_roots,
        branch_cells,
        partition_mirror,
        derived_indexes,
        framing_and_metadata: total
            .checked_sub(classified)
            .ok_or_else(|| frame_mismatch("native checkpoint section accounting overflow"))?,
    };
    drop(wire);
    Ok((
        bytes
            .seal()
            .map_err(crate::durability::checkpoint_history::capture::array_capture_stop)?
            .into_immutable_bytes(),
        sections,
        members,
        captured.publications,
    ))
}
