use super::checkpoint_wire::{ByteString, CheckpointWire};
use super::manifest_capture::CapturedManifest;
#[cfg(test)]
use crate::durability::data::DurableCheckpoint;
use crate::durability::data::{
    NativeCheckpointSectionBytes, RelationalCheckpointArtifactPolicy, RelationalCheckpointMember,
    RelationalNativeCheckpointCaptureDenial,
};
use std::io::Write;
use worth_execution::{ExecutionAllocationPolicy, ExecutionByteBuffer, ExecutionImmutableBytes};
mod writer;
use writer::{frame_mismatch, EmissionWriter, SizingWriter};

#[cfg(test)]
pub(crate) fn encode_checkpoint(
    checkpoint: &DurableCheckpoint,
    policy: ExecutionAllocationPolicy<'_, '_>,
) -> Result<
    (ExecutionImmutableBytes, NativeCheckpointSectionBytes),
    RelationalNativeCheckpointCaptureDenial,
> {
    let (bytes, sections, _) = encode_checkpoint_members(
        checkpoint,
        policy,
        &RelationalCheckpointArtifactPolicy::Embedded,
    )?;
    Ok((bytes, sections))
}

#[cfg(test)]
pub(crate) fn encode_checkpoint_members(
    checkpoint: &DurableCheckpoint,
    policy: ExecutionAllocationPolicy<'_, '_>,
    placement: &RelationalCheckpointArtifactPolicy<'_>,
) -> Result<
    (
        ExecutionImmutableBytes,
        NativeCheckpointSectionBytes,
        Vec<RelationalCheckpointMember>,
    ),
    RelationalNativeCheckpointCaptureDenial,
> {
    policy.check_live()?;
    let captured = CapturedManifest::capture(checkpoint, policy)?;
    let (bytes, sections, members, _) = encode_captured(captured, policy, placement)?;
    Ok((bytes, sections, members))
}

pub(crate) fn encode_captured(
    mut captured: CapturedManifest,
    policy: ExecutionAllocationPolicy<'_, '_>,
    placement: &RelationalCheckpointArtifactPolicy<'_>,
) -> Result<
    (
        ExecutionImmutableBytes,
        NativeCheckpointSectionBytes,
        Vec<RelationalCheckpointMember>,
        Vec<crate::storage::residency::RelationalCheckpointRegionPublication>,
    ),
    RelationalNativeCheckpointCaptureDenial,
> {
    let embedded = placement.is_embedded();
    if embedded {
        for entry in &captured.carried_envelopes {
            policy.check_live()?;
            let source = captured.cold_source.as_ref().ok_or_else(|| {
                super::manifest_capture::corrupt("inline capture lacks its journal source")
            })?;
            let bytes = source
                .read_commit(entry.position, 1 << 30)
                .map_err(|cause| {
                    let failure = crate::storage::data::RelationalStorageUnavailable {
                        subject: crate::storage::data::RelationalStorageSubject::Envelope(
                            entry.position,
                        ),
                        cause,
                    };
                    let crate::durability::data::RelationalRecoveryStop::Durability(error) =
                        crate::durability::data::RelationalRecoveryStop::from(failure)
                    else {
                        unreachable!()
                    };
                    error
                })?;
            let artifact = crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact::from_untrusted_bytes(bytes)
                .map_err(|error| super::manifest_capture::corrupt(format!("inline journal frame: {error:?}")))?;
            if artifact.position() != entry.position
                || artifact.commit_id() != entry.commit.commit_id
                || artifact.payload_digest().as_bytes() != &entry.payload_digest
            {
                return Err(super::manifest_capture::corrupt(
                    "inline journal disagrees with its commit index",
                )
                .into());
            }
            captured.envelopes.push(
                rmp_serde::to_vec_named(&super::embedded_commit::EmbeddedCommitRef::Journal(
                    ByteString(artifact.as_bytes()),
                ))
                .map_err(writer::encoding_failure)?,
            );
        }
        for member in captured.members.values_mut() {
            if let RelationalCheckpointMember::Carried { locator } = member {
                let locator = *locator;
                let source = captured.cold_source.as_ref().ok_or_else(|| {
                    super::manifest_capture::corrupt(
                        "inline capture lacks its carried member source",
                    )
                })?;
                let bytes = source.read_member(locator, 1 << 30).map_err(|cause| {
                    super::manifest_capture::corrupt(format!("carried inline member: {cause:?}"))
                })?;
                if super::member_frame::locator(&bytes) != locator {
                    return Err(super::manifest_capture::corrupt(
                        "carried inline member integrity mismatch",
                    )
                    .into());
                }
                *member = RelationalCheckpointMember::New {
                    locator,
                    bytes: bytes.into(),
                };
            }
        }
    }
    let wire = CheckpointWire {
        manifest: captured.manifest,
        members: if embedded {
            captured
                .members
                .iter()
                .map(|(key, member)| {
                    let RelationalCheckpointMember::New { bytes, .. } = member else {
                        unreachable!("inline member admitted above")
                    };
                    (*key, ByteString(bytes))
                })
                .collect()
        } else {
            Vec::new()
        },
        envelopes: if embedded {
            captured
                .envelopes
                .iter()
                .map(|bytes| ByteString(bytes))
                .collect()
        } else {
            Vec::new()
        },
    };
    let mut sizing = SizingWriter::new(policy);
    let counted = rmp_serde::encode::write_named(&mut sizing, &wire);
    let total = sizing
        .finish(counted)?
        .checked_add(super::CHECKPOINT_MAGIC.len())
        .ok_or_else(|| frame_mismatch("native checkpoint frame size overflow"))?;
    let mut bytes = ExecutionByteBuffer::allocate(total, policy)?;
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
    let envelopes = rmp_serde::to_vec_named(&wire.envelopes)
        .map_err(writer::encoding_failure)?
        .len();
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
    let members = captured.members.into_values().collect();
    Ok((bytes.seal()?, sections, members, captured.publications))
}
