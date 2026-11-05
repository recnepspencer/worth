use super::checkpoint_wire::{ByteString, CheckpointWire};
use super::manifest_capture::CapturedManifest;
#[cfg(test)]
use crate::durability::data::DurableCheckpoint;
use crate::durability::data::{
    NativeCheckpointSectionBytes, RelationalCheckpointArtifactPolicy, RelationalCheckpointMember,
    RelationalNativeCheckpointCaptureDenial,
};
use worth_execution::{ExecutionAllocationPolicy, ExecutionImmutableBytes};
mod writer;
pub(crate) use writer::{copy_bytes, count_value, encode_value, CheckpointEncodingControl};

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
    let (bytes, sections, members, _) = encode_captured(
        captured,
        policy,
        placement,
        &mut CheckpointEncodingControl::Policy(policy),
    )?;
    Ok((bytes, sections, members))
}

pub(crate) fn encode_captured(
    mut captured: CapturedManifest,
    policy: ExecutionAllocationPolicy<'_, '_>,
    placement: &RelationalCheckpointArtifactPolicy<'_>,
    control: &mut CheckpointEncodingControl<'_, '_, '_, '_, '_>,
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
            let cell = captured
                .envelope_cells
                .get(&entry.position)
                .ok_or_else(|| {
                    super::manifest_capture::corrupt(
                        "inline capture lacks indexed envelope custody",
                    )
                })?;
            let artifact = control.export_artifact(cell)?;
            captured.envelopes.push(encode_value(
                control,
                b"",
                &super::embedded_commit::EmbeddedCommitRef::Journal(ByteString(
                    artifact.as_bytes(),
                )),
            )?);
        }
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
    let wire = CheckpointWire {
        manifest: captured.manifest,
        members: if embedded {
            captured
                .members
                .iter()
                .map(|(key, member)| {
                    let RelationalCheckpointMember::New { bytes, .. } = member else {
                        return Err(super::manifest_capture::corrupt(
                            "inline capture still carries an unresolved member",
                        ));
                    };
                    Ok((*key, ByteString(bytes)))
                })
                .collect::<Result<Vec<_>, crate::durability::data::DurabilityError>>()?
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
    let bytes = encode_value(control, super::CHECKPOINT_MAGIC, &wire)?;
    let total = bytes.len();
    // Count the exact sections without constructing another payload copy.
    let branch_roots = count_value(control, &wire.manifest.roots)?;
    let branch_cells = count_value(control, &wire.manifest.branch_cells)?;
    let partition_mirror = count_value(control, &wire.manifest.mirror)?;
    let envelopes = count_value(control, &wire.envelopes)?;
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
    let members = captured.members.into_values().collect();
    control.checkpoint(0)?;
    Ok((bytes, sections, members, captured.publications))
}
