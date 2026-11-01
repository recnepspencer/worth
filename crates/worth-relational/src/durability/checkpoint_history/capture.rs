//! Freeze complete original canonical bytes into singleton members.
use super::inventory::{CanonicalCheckpointMember, CanonicalCheckpointMembers};
use crate::durability::{
    data::*,
    log::native_checkpoint_codec::CapturedManifest,
    medium::{RelationalDurableOwnedReadRefusal as Refusal, RelationalDurableReadProtection},
};
use worth_execution::ExecutionRequest;
pub(crate) fn capture(
    captured: &mut CapturedManifest,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), RelationalNativeCheckpointCaptureDenial> {
    let count = captured.manifest.commits.len();
    let memory = CanonicalCheckpointMembers::reserve(count, request)
        .map_err(RelationalNativeCheckpointCaptureDenial::Request)?;
    let mut rows = Vec::with_capacity(count);
    let mut payloads = worth_execution::ExecutionRequestArrayBuilder::allocate(*request, count)
        .map_err(array_capture_stop)?;
    // A prior standalone Inline directory has no journal prefix. Select the
    // actual Journal issuer from its cells, independently of selected members.
    let source = captured
        .envelope_cells
        .values()
        .find_map(|cell| cell.journal_placement().map(|(_, source)| source));
    let protection = source
        .as_ref()
        .map(|source| RelationalDurableReadProtection::capture(source, request))
        .transpose()
        .map_err(|cause| capture_stop(owned_stop(cause)))?;
    for (ordinal, entry) in captured.manifest.commits.iter().enumerate() {
        crate::execution::check_request(request)
            .map_err(RelationalNativeCheckpointCaptureDenial::Request)?;
        if entry.position.0 != ordinal as u64 + 1 {
            return Err(corrupt("original canonical prefix is incomplete").into());
        }
        let cell = captured
            .envelope_cells
            .get(&entry.position)
            .ok_or_else(|| corrupt("original canonical cell unavailable"))?;
        let artifact = cell
            .require_original_artifact(request, source.as_ref().zip(protection.as_ref()))
            .map_err(capture_stop)?;
        if artifact.position() != entry.position
            || artifact.commit_id() != entry.commit.commit_id
            || artifact.payload_digest().as_bytes() != &entry.payload_digest
        {
            return Err(corrupt("original canonical artifact differs from captured index").into());
        }
        let bytes = artifact.into_bytes();
        let member = crate::durability::medium::RelationalDurableMemberLocator {
            sha256: <sha2::Sha256 as sha2::Digest>::digest(bytes.bytes()).into(),
            length: bytes.len() as u64,
        };
        rows.push(CanonicalCheckpointMember {
            position: entry.position,
            member,
        });
        payloads
            .push(RelationalCheckpointMember::New {
                locator: member,
                bytes,
            })
            .map_err(array_capture_stop)?;
    }
    captured.manifest.canonical_members = CanonicalCheckpointMembers::from_reserved(rows, memory);
    super::inventory::validate(&captured.manifest, request).map_err(capture_stop)?;
    captured.canonical_payloads = Some(payloads.seal().map_err(array_capture_stop)?);
    captured.canonical_publication =
        Some(super::publication::CanonicalCheckpointPublication::capture(
            &captured.manifest,
            &captured.envelope_cells,
            request,
        )?);
    Ok(())
}
fn corrupt(detail: &str) -> DurabilityError {
    DurabilityError::new(RecoveryFailureClass::CorruptCheckpoint, detail)
}
pub(crate) fn owned_stop(cause: Refusal) -> RelationalRecoveryStop {
    match cause {
        Refusal::Request(cause) => RelationalRecoveryStop::Request(cause),
        Refusal::Buffer(worth_execution::ExecutionRequestArrayDenial::Request(cause)) => {
            RelationalRecoveryStop::Request(cause.into())
        }
        Refusal::Buffer(worth_execution::ExecutionRequestArrayDenial::Allocation(cause)) => {
            RelationalRecoveryStop::Allocation(cause)
        }
        Refusal::Storage(cause) => crate::storage::data::RelationalStorageUnavailable {
            subject: crate::storage::data::RelationalStorageSubject::Envelope(
                crate::publication::patch::data::PatchStreamPosition(0),
            ),
            cause,
        }
        .into(),
        Refusal::UnsupportedProvider => DurabilityError::new(
            RecoveryFailureClass::UnsupportedLegacySemantics,
            "original protected canonical source unsupported",
        )
        .into(),
        Refusal::ForeignSource | Refusal::ForeignProtection | Refusal::Busy | Refusal::Bounds => {
            crate::storage::data::RelationalStorageUnavailable {
                subject: crate::storage::data::RelationalStorageSubject::Envelope(
                    crate::publication::patch::data::PatchStreamPosition(0),
                ),
                cause: crate::storage::data::RelationalStorageUnavailableCause::SourceUnavailable,
            }
            .into()
        }
        Refusal::CanonicalFrame(_) => DurabilityError::new(
            RecoveryFailureClass::CorruptCanonicalArtifact,
            "original canonical frame refused",
        )
        .into(),
        Refusal::TransitionFrame(_) => DurabilityError::new(
            RecoveryFailureClass::CorruptBranchTransitionArtifact,
            "original transition frame refused",
        )
        .into(),
    }
}

pub(crate) fn capture_stop(
    stop: RelationalRecoveryStop,
) -> RelationalNativeCheckpointCaptureDenial {
    match stop {
        RelationalRecoveryStop::Request(cause) => {
            RelationalNativeCheckpointCaptureDenial::Request(cause)
        }
        RelationalRecoveryStop::Allocation(cause) => {
            RelationalNativeCheckpointCaptureDenial::Allocation(cause)
        }
        RelationalRecoveryStop::Durability(cause) => {
            RelationalNativeCheckpointCaptureDenial::Durability(cause)
        }
    }
}
pub(crate) fn array_capture_stop(
    cause: worth_execution::ExecutionRequestArrayDenial,
) -> RelationalNativeCheckpointCaptureDenial {
    match cause {
        worth_execution::ExecutionRequestArrayDenial::Request(cause) => {
            RelationalNativeCheckpointCaptureDenial::Request(cause.into())
        }
        worth_execution::ExecutionRequestArrayDenial::Allocation(cause) => {
            RelationalNativeCheckpointCaptureDenial::Allocation(cause)
        }
    }
}

pub(crate) fn map_stop(
    cause: worth_execution::ExecutionOrderedMapDenial,
) -> RelationalRecoveryStop {
    match cause {
        worth_execution::ExecutionOrderedMapDenial::Request(cause) => {
            RelationalRecoveryStop::Request(cause.into())
        }
        worth_execution::ExecutionOrderedMapDenial::Allocation(cause) => {
            RelationalRecoveryStop::Allocation(cause)
        }
        worth_execution::ExecutionOrderedMapDenial::CapacityOverflow => {
            RelationalRecoveryStop::Request(
                crate::execution::RelationalExecutionDenialCause::MemoryOverflow,
            )
        }
    }
}
