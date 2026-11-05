use super::checkpoint_wire::CheckpointWire;
use super::manifest_capture::corrupt;
use super::member_source::InlineCheckpointSource;
use super::{DecodedNativeCheckpoint, DurabilityError, RecoveryFailureClass};
use crate::durability::data::RelationalRecoveryStop;
use crate::durability::medium::RelationalDurableColdSource;
use std::sync::Arc;
use worth_execution::ExecutionRequest;

pub(crate) fn decode_checkpoint(
    execution: &ExecutionRequest<'_, '_>,
    bytes: &[u8],
) -> Result<DecodedNativeCheckpoint, RelationalRecoveryStop> {
    decode(execution, bytes, None, None)
}

pub(crate) fn decode_checkpoint_with_image(
    execution: &ExecutionRequest<'_, '_>,
    bytes: &[u8],
    external: Arc<dyn RelationalDurableColdSource>,
    image: crate::durability::checkpoint_admission::RelationalCheckpointAdmissionArtifact,
) -> Result<DecodedNativeCheckpoint, RelationalRecoveryStop> {
    decode(execution, bytes, Some(external), Some(image))
}

pub(crate) fn decode_checkpoint_from_source(
    execution: &ExecutionRequest<'_, '_>,
    bytes: &[u8],
    external: Option<Arc<dyn RelationalDurableColdSource>>,
) -> Result<DecodedNativeCheckpoint, RelationalRecoveryStop> {
    decode(execution, bytes, external, None)
}

fn decode(
    execution: &ExecutionRequest<'_, '_>,
    bytes: &[u8],
    external: Option<Arc<dyn RelationalDurableColdSource>>,
    image: Option<crate::durability::checkpoint_admission::RelationalCheckpointAdmissionArtifact>,
) -> Result<DecodedNativeCheckpoint, RelationalRecoveryStop> {
    execution
        .check_live()
        .map_err(|stop| RelationalRecoveryStop::Request(stop.into()))?;
    let payload = bytes.strip_prefix(super::CHECKPOINT_MAGIC).ok_or_else(|| {
        DurabilityError::new(
            RecoveryFailureClass::UnsupportedLegacySemantics,
            format!(
                "unsupported native checkpoint frame; {} required",
                String::from_utf8_lossy(super::CHECKPOINT_MAGIC).trim_end(),
            ),
        )
    })?;
    let memory =
        crate::durability::checkpoint_history::inventory::prepare_decode(payload, execution)?;
    let _member_memory = super::member_sequence::reserve(payload, execution)?;
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
    let facts = super::manifest_validation::validate(&manifest)?;
    let inline = !wire.members.is_empty();
    let source: Arc<dyn RelationalDurableColdSource> = if !inline {
        external.ok_or_else(|| corrupt("external checkpoint requires its cold source"))?
    } else {
        let mut members = worth_execution::ExecutionOrderedMap::default();
        let mut external_members = worth_execution::ExecutionOrderedMap::<
            crate::durability::medium::RelationalDurableMemberLocator,
            (),
        >::default();
        for (locator, bytes) in wire.members {
            if super::member_frame::locator(bytes.0) != locator
                || members.contains_key(&locator)
                || external_members.contains_key(&locator)
            {
                return Err(
                    corrupt("inline checkpoint member digest or uniqueness mismatch").into(),
                );
            }
            if external.is_some() {
                // Validate every inline original, but do not copy bytes when the
                // actual supplied issuer can admit its installed selected read.
                external_members
                    .insert_cloned_native_for_request(&locator, &(), *execution)
                    .map_err(crate::durability::checkpoint_history::capture::map_stop)?;
                continue;
            }
            let mut owned = worth_execution::ExecutionRequestArrayBuilder::<u8>::allocate(
                *execution,
                bytes.0.len(),
            )
            .map_err(|cause| {
                RelationalRecoveryStop::from(
                    crate::durability::checkpoint_history::capture::array_capture_stop(cause),
                )
            })?;
            owned.extend_from_slice(bytes.0).map_err(|cause| {
                RelationalRecoveryStop::from(
                    crate::durability::checkpoint_history::capture::array_capture_stop(cause),
                )
            })?;
            let owned = owned
                .seal()
                .map_err(|cause| {
                    RelationalRecoveryStop::from(
                        crate::durability::checkpoint_history::capture::array_capture_stop(cause),
                    )
                })?
                .into_immutable_bytes();
            members
                .insert_cloned_native_for_request(&locator, &owned, *execution)
                .map_err(crate::durability::checkpoint_history::capture::map_stop)?;
        }
        match external {
            // Directory admission below still REQUIRES the actual installed
            // selected read and full member custody; an Arc alone grants none.
            Some(source) => source,
            None => Arc::new(InlineCheckpointSource { members }),
        }
    };
    drop(_member_memory); // The borrowed wire member table has now been consumed.
    let checkpoint = if inline && image.is_none() {
        super::expanded_checkpoint::expand(execution, manifest, source, facts.history_members)?
    } else {
        super::cold_checkpoint::readmit(execution, manifest, source, facts, image)?
    };
    Ok(DecodedNativeCheckpoint { checkpoint })
}
