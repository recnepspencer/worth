//! Direct Complete output borrows the SAME sealed manifest; members live externally.
use crate::durability::data::NativeCheckpointSectionBytes;
use crate::durability::data::RelationalCheckpointManifest;
use contracts::durability::artifact_payload_emission::request::{encode_named, named_size};
use contracts::durability::data::native_checkpoint_capture_denial::RelationalNativeCheckpointCaptureDenial as Denial;
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest};
use worth_relational_contracts::internal_port as contracts;

use super::checkpoint_encoding::writer::{
    count_value, encode_value, frame_mismatch, CheckpointEncodingControl,
};
use super::checkpoint_wire::CheckpointWire;

fn wire(
    manifest: &RelationalCheckpointManifest,
) -> CheckpointWire<'_, &RelationalCheckpointManifest> {
    CheckpointWire {
        manifest,
        members: Vec::new(),
        envelopes: Vec::new(),
    }
}

pub(crate) fn encode_complete(
    manifest: &RelationalCheckpointManifest,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<(ExecutionImmutableBytes, NativeCheckpointSectionBytes), Denial> {
    let bytes = encode_named(super::CHECKPOINT_MAGIC, &wire(manifest), execution).map_err(stop)?;
    let sections = sections(
        &bytes,
        manifest,
        |rows| named_size(rows, execution).map_err(stop),
        |overflow| {
            if overflow {
                Denial::Request(
                    contracts::execution::RelationalExecutionDenialCause::MemoryOverflow,
                )
            } else {
                Denial::OriginalAuthorityUnavailable
            }
        },
    )?;
    Ok((bytes, sections))
}

pub(super) fn encode_in_stage(
    manifest: &RelationalCheckpointManifest,
    control: &mut CheckpointEncodingControl<'_, '_, '_, '_, '_>,
) -> Result<(ExecutionImmutableBytes, NativeCheckpointSectionBytes), Denial> {
    let bytes = encode_value(control, super::CHECKPOINT_MAGIC, &wire(manifest))?;
    let sections = sections(
        &bytes,
        manifest,
        |rows| count_value(control, rows),
        |_| frame_mismatch("native checkpoint section accounting overflow"),
    )?;
    control.checkpoint(0)?;
    Ok((bytes, sections))
}

fn sections(
    bytes: &ExecutionImmutableBytes,
    manifest: &RelationalCheckpointManifest,
    mut count: impl FnMut(&Sections<'_>) -> Result<usize, Denial>,
    accounting_denial: impl Fn(bool) -> Denial,
) -> Result<NativeCheckpointSectionBytes, Denial> {
    let total = bytes.len();
    let branch_roots = count(&Sections::Roots(&manifest.roots))?;
    let branch_cells = count(&Sections::Branches(&manifest.branch_cells))?;
    let partition_mirror = count(&Sections::Regions(&manifest.mirror))?;
    let derived_indexes = count(&Sections::Indexes(&manifest.index_definitions))?;
    let envelopes = 0usize;
    let classified = branch_roots
        .checked_add(branch_cells)
        .and_then(|n| n.checked_add(partition_mirror))
        .and_then(|n| n.checked_add(derived_indexes))
        .ok_or_else(|| accounting_denial(true))?;
    let framing_and_metadata = total
        .checked_sub(classified)
        .ok_or_else(|| accounting_denial(false))?;
    Ok(NativeCheckpointSectionBytes {
        total,
        envelopes,
        branch_roots,
        branch_cells,
        partition_mirror,
        derived_indexes,
        framing_and_metadata,
    })
}
#[derive(serde::Serialize)]
#[serde(untagged)]
enum Sections<'a> {
    Roots(&'a crate::durability::data::CheckpointRoots),
    Branches(&'a [contracts::branch::reference_checkpoint_data::RelationalBranchCellCheckpoint]),
    Regions(&'a [crate::durability::data::RelationalCheckpointRegion]),
    Indexes(&'a crate::durability::data::CheckpointIndexDefinitions),
}

fn stop(cause: contracts::durability::data::recovery_stop::RelationalRecoveryStop) -> Denial {
    contracts::durability::data::native_checkpoint_capture_denial::capture_stop(cause)
}
