use super::checkpoint_wire::CheckpointWire;
use super::manifest_capture::corrupt;
use super::member_source::InlineCheckpointSource;
use super::{DecodedNativeCheckpoint, DurabilityError, RecoveryFailureClass};
use crate::durability::data::RelationalRecoveryStop;
use crate::durability::medium::RelationalDurableColdSource;
use std::collections::BTreeMap;
use std::sync::Arc;
use worth_execution::{ExecutionRequest, ExecutionRequestArrayBuilder};

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
            format!(
                "unsupported native checkpoint frame; {} required",
                String::from_utf8_lossy(super::CHECKPOINT_MAGIC).trim_end(),
            ),
        )
    })?;
    let wire: CheckpointWire<'_> = rmp_serde::from_slice(payload)
        .map_err(|error| corrupt(format!("native manifest: {error}")))?;
    let facts =
        crate::storage::substrate::maintenance::ReconstructionLane::enter_reconstruction(|lane| {
            super::manifest_validation::validate(lane, &wire.manifest)
        })?;
    let mut inline = None;
    let source: Arc<dyn RelationalDurableColdSource> = if wire.members.is_empty()
        && wire.envelopes.is_empty()
    {
        external.ok_or_else(|| corrupt("external checkpoint requires its cold source"))?
    } else {
        let mut members = BTreeMap::new();
        for (locator, bytes) in wire.members {
            if super::member_frame::locator_for_request(execution, bytes.0)
                .map_err(RelationalRecoveryStop::Request)?
                != locator
                || members.contains_key(&locator)
            {
                return Err(
                    corrupt("inline checkpoint member digest or uniqueness mismatch").into(),
                );
            }
            let mut owned = ExecutionRequestArrayBuilder::<u8>::allocate(*execution, bytes.0.len())
                .map_err(inline_allocation)?;
            for chunk in bytes.0.chunks(4096) {
                crate::content::charge_bytes(execution, chunk.len())?;
                owned.extend_from_slice(chunk).map_err(inline_allocation)?;
            }
            members.insert(
                locator,
                owned
                    .seal()
                    .map_err(inline_allocation)?
                    .into_immutable_bytes(),
            );
        }
        let mut admitted = BTreeMap::new();
        for bytes in wire.envelopes {
            let wire = super::embedded_commit::decode_embedded(execution, bytes.0)?;
            let commit = wire.readmit(execution)?;
            // expand consumes the admitted commits directly. Projecting another
            // journal byte copy here creates no authority and retains no useful data.
            if admitted.insert(commit.position(), commit).is_some() {
                return Err(corrupt("duplicate inline checkpoint envelope position").into());
            }
        }
        inline = Some(admitted);
        Arc::new(InlineCheckpointSource { members })
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

fn inline_allocation(
    cause: worth_execution::ExecutionRequestArrayDenial,
) -> RelationalRecoveryStop {
    use worth_execution::ExecutionRequestArrayDenial as Denial;
    match cause {
        Denial::Request(cause) => RelationalRecoveryStop::Request(cause.into()),
        Denial::Allocation(cause) => RelationalRecoveryStop::Request(match cause.classify() {
            worth_execution::ExecutionAllocationDenialClass::Request(cause) => cause.into(),
            worth_execution::ExecutionAllocationDenialClass::Physical(_) => {
                crate::execution::RelationalExecutionDenialCause::AllocationFailed
            }
        }),
    }
}
