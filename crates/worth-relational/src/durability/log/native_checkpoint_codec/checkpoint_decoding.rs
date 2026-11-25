use super::checkpoint_wire::CheckpointWire;
use super::manifest_capture::corrupt;
use super::DecodedNativeCheckpoint;
use crate::durability::data::RelationalRecoveryStop;
use crate::durability::medium::RelationalDurableColdSource;
use std::sync::Arc;
use worth_execution::ExecutionRequest;

pub(crate) fn decode_checkpoint_with_image(
    execution: &ExecutionRequest<'_, '_>,
    bytes: &[u8],
    external: Arc<dyn RelationalDurableColdSource>,
    image: crate::durability::checkpoint_admission::RelationalCheckpointAdmissionArtifact,
) -> Result<DecodedNativeCheckpoint, RelationalRecoveryStop> {
    decode(execution, bytes, external, Some(image))
}

pub(crate) fn decode_checkpoint_from_source(
    execution: &ExecutionRequest<'_, '_>,
    bytes: &[u8],
    external: Arc<dyn RelationalDurableColdSource>,
) -> Result<DecodedNativeCheckpoint, RelationalRecoveryStop> {
    decode(execution, bytes, external, None)
}

pub(crate) fn decode_manifest_directory(
    execution: &ExecutionRequest<'_, '_>,
    bytes: &[u8],
) -> Result<
    (
        crate::durability::data::RelationalCheckpointManifest,
        Arc<crate::durability::data::RelationalHistoryMemberDirectory>,
    ),
    RelationalRecoveryStop,
> {
    let (manifest, facts) = decode_manifest(execution, bytes)?;
    Ok((manifest, facts.history_members))
}

fn decode_manifest(
    execution: &ExecutionRequest<'_, '_>,
    bytes: &[u8],
) -> Result<
    (
        crate::durability::data::RelationalCheckpointManifest,
        super::manifest_validation::ManifestDirectoryFacts,
    ),
    RelationalRecoveryStop,
> {
    execution
        .check_live()
        .map_err(|stop| RelationalRecoveryStop::Request(stop.into()))?;
    let payload = super::checkpoint_payload(bytes)?;
    let memory =
        crate::durability::checkpoint_history::inventory::prepare_decode(payload, execution)?;
    let mut wire: CheckpointWire<'_, crate::durability::data::RelationalCheckpointManifestDraft> =
        rmp_serde::from_slice(payload)
            .map_err(|error| corrupt(format!("native manifest: {error}")))?;
    if let Err(memory) = wire.manifest.canonical_members.attach(memory) {
        drop(wire);
        drop(memory);
        return Err(corrupt("canonical inventory shared before original admission").into());
    }
    crate::durability::checkpoint_history::inventory::validate(&wire.manifest, execution)?;
    let manifest = wire.manifest.seal_ordinary();
    let facts =
        crate::storage::substrate::maintenance::ReconstructionLane::enter_reconstruction(|lane| {
            super::manifest_validation::validate(lane, &manifest)
        })?;
    Ok((manifest, facts))
}

#[cfg(test)]
pub(crate) fn decode_expanded_checkpoint(
    execution: &ExecutionRequest<'_, '_>,
    bytes: &[u8],
    external: Arc<dyn RelationalDurableColdSource>,
) -> Result<DecodedNativeCheckpoint, RelationalRecoveryStop> {
    let (manifest, facts) = decode_manifest(execution, bytes)?;
    let checkpoint =
        super::expanded_checkpoint::expand(execution, manifest, external, facts.history_members)?;
    Ok(DecodedNativeCheckpoint { checkpoint })
}

fn decode(
    execution: &ExecutionRequest<'_, '_>,
    bytes: &[u8],
    external: Arc<dyn RelationalDurableColdSource>,
    image: Option<crate::durability::checkpoint_admission::RelationalCheckpointAdmissionArtifact>,
) -> Result<DecodedNativeCheckpoint, RelationalRecoveryStop> {
    let (manifest, facts) = decode_manifest(execution, bytes)?;
    let checkpoint = super::cold_checkpoint::readmit(execution, manifest, external, facts, image)?;
    Ok(DecodedNativeCheckpoint { checkpoint })
}
