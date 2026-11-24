//! Freeze complete original canonical bytes into singleton members.
use super::inventory::{CanonicalCheckpointMember, CanonicalCheckpointMembers};
use crate::durability::{
    data::*, log::native_checkpoint_codec::CapturedManifest,
    medium::RelationalDurableReadProtection,
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
        .map_err(|cause| {
            capture_stop(owned_stop(
                cause,
                crate::storage::data::RelationalStorageSubject::Envelope(
                    crate::publication::patch::data::PatchStreamPosition(0),
                ),
            ))
        })?;
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
        let member =
            crate::durability::log::native_checkpoint_codec::member_frame::locator_for_request(
                request,
                bytes.bytes(),
            )
            .map_err(RelationalNativeCheckpointCaptureDenial::Request)?;
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
    // A capture without commits has no canonical placement to publish, so it
    // retains no publication, as its empty member inventory retains nothing.
    captured.canonical_publication = (count > 0)
        .then(|| {
            super::publication::CanonicalCheckpointPublication::capture(
                &captured.manifest,
                &captured.envelope_cells,
                request,
            )
        })
        .transpose()?;
    Ok(())
}
fn corrupt(detail: &str) -> DurabilityError {
    DurabilityError::new(RecoveryFailureClass::CorruptCheckpoint, detail)
}
pub(crate) use crate::durability::medium::owned_read_stop::owned_stop;

pub(crate) use crate::durability::data::native_checkpoint_capture_denial::{
    array_capture_stop, capture_stop,
};
pub(crate) use crate::durability::data::recovery_stop::map_stop;
