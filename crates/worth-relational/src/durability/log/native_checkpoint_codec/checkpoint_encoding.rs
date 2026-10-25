use super::checkpoint_wire::{ByteString, CheckpointWire};
use super::manifest_capture::CapturedManifest;
use crate::durability::data::{
    DurableCheckpoint, NativeCheckpointSectionBytes, RelationalCheckpointArtifactPolicy,
    RelationalCheckpointMember, RelationalNativeCheckpointCaptureDenial,
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
        RelationalCheckpointArtifactPolicy::Embedded,
    )?;
    Ok((bytes, sections))
}

pub(crate) fn encode_checkpoint_members(
    checkpoint: &DurableCheckpoint,
    policy: ExecutionAllocationPolicy<'_, '_>,
    placement: RelationalCheckpointArtifactPolicy,
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
    let embedded = placement == RelationalCheckpointArtifactPolicy::Embedded;
    let wire = CheckpointWire {
        manifest: captured.manifest,
        members: if embedded {
            captured
                .members
                .iter()
                .map(|(key, bytes)| (*key, ByteString(bytes)))
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
    let members = captured
        .members
        .into_iter()
        .map(|(locator, bytes)| RelationalCheckpointMember::New { locator, bytes })
        .collect();
    Ok((bytes.seal()?, sections, members))
}
