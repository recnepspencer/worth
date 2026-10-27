use super::checkpoint_wire::CheckpointWire;
use super::manifest_capture::corrupt;
use super::member_source::InlineCheckpointSource;
use super::{DecodedNativeCheckpoint, DurabilityError, RecoveryFailureClass};
use crate::durability::data::RelationalRecoveryStop;
use crate::durability::medium::RelationalDurableColdSource;
use std::collections::BTreeMap;
use std::sync::Arc;
use worth_execution::ExecutionRequest;

pub(crate) fn decode_checkpoint(
    execution: &ExecutionRequest<'_, '_>,
    bytes: &[u8],
) -> Result<DecodedNativeCheckpoint, RelationalRecoveryStop> {
    decode_checkpoint_from_source(execution, bytes, None)
}

pub(crate) fn decode_checkpoint_from_source(
    execution: &ExecutionRequest<'_, '_>,
    bytes: &[u8],
    external: Option<Arc<dyn RelationalDurableColdSource>>,
) -> Result<DecodedNativeCheckpoint, RelationalRecoveryStop> {
    execution
        .check_live()
        .map_err(|stop| RelationalRecoveryStop::Request(stop.into()))?;
    let payload = bytes.strip_prefix(super::CHECKPOINT_MAGIC).ok_or_else(|| {
        DurabilityError::new(
            RecoveryFailureClass::UnsupportedLegacySemantics,
            "unsupported native checkpoint frame; CHK11 required",
        )
    })?;
    let wire: CheckpointWire<'_> = rmp_serde::from_slice(payload)
        .map_err(|error| corrupt(format!("native manifest: {error}")))?;
    let facts = super::manifest_validation::validate(&wire.manifest)?;
    let mut inline = None;
    let source: Arc<dyn RelationalDurableColdSource> = if wire.members.is_empty()
        && wire.envelopes.is_empty()
    {
        external.ok_or_else(|| corrupt("external checkpoint requires its cold source"))?
    } else {
        let mut members = BTreeMap::new();
        for (locator, bytes) in wire.members {
            if super::member_frame::locator(bytes.0) != locator
                || members
                    .insert(locator, Arc::<[u8]>::from(bytes.0))
                    .is_some()
            {
                return Err(
                    corrupt("inline checkpoint member digest or uniqueness mismatch").into(),
                );
            }
        }
        let mut envelopes = BTreeMap::new();
        let mut admitted = BTreeMap::new();
        for bytes in wire.envelopes {
            let wire: super::embedded_commit::EmbeddedCommitWire<'_> =
                rmp_serde::from_slice(bytes.0)
                    .map_err(|error| corrupt(format!("inline checkpoint envelope: {error}")))?;
            let commit = wire.readmit(execution)?;
            let artifact = crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact::from_positioned(
                &commit, worth_execution::ExecutionAllocationPolicy::SystemAllocation)
                .map_err(|error| corrupt(format!("inline checkpoint journal projection: {error:?}")))?;
            if envelopes
                .insert(commit.position(), Arc::<[u8]>::from(artifact.as_bytes()))
                .is_some()
                || admitted.insert(commit.position(), commit).is_some()
            {
                return Err(corrupt("duplicate inline checkpoint envelope position").into());
            }
        }
        inline = Some(admitted);
        Arc::new(InlineCheckpointSource { members, envelopes })
    };
    let checkpoint = match inline {
        Some(envelopes) => super::expanded_checkpoint::expand(
            execution,
            wire.manifest,
            source,
            Some(envelopes),
            facts.history_members,
        )?,
        None => super::cold_checkpoint::readmit(execution, wire.manifest, source, facts)?,
    };
    Ok(DecodedNativeCheckpoint { checkpoint })
}
