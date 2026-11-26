//! Freeze original canonical bytes of uncovered commits into singleton members.
use super::inventory::{CanonicalCheckpointMember, CanonicalCheckpointMembers};
use crate::durability::data::*;
use crate::durability::log::native_checkpoint_codec::CapturedManifest;
use worth_execution::ExecutionRequest;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_storage::internal_port::durability::medium::read_protection::RelationalDurableReadProtection;
/// The prior checkpoint's roster is carried (`O(touched)`): its members are
/// carried where the destination holds them and copied from the prior source
/// otherwise; only commits above its covered prefix freeze new members.
pub(crate) fn capture(
    captured: &mut CapturedManifest,
    request: &ExecutionRequest<'_, '_>,
    may_carry: &dyn Fn(worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableMemberLocator) -> bool,
) -> Result<(), RelationalNativeCheckpointCaptureDenial> {
    let prior = captured.prior_canonical.clone();
    let carried = prior.as_ref().map_or(&[][..], |prior| prior.entries());
    let covered = carried.len() as u64;
    let appended = captured
        .commit_rows
        .iter()
        .filter(|entry| entry.position.0 > covered)
        .cloned()
        .collect::<Vec<_>>();
    let count = carried.len() + appended.len();
    let memory = CanonicalCheckpointMembers::reserve(count, request)
        .map_err(RelationalNativeCheckpointCaptureDenial::Request)?;
    let mut rows = Vec::with_capacity(count);
    let mut payloads = worth_execution::ExecutionRequestArrayBuilder::allocate(*request, count)
        .map_err(array_capture_stop)?;
    for row in carried {
        contracts::execution::request_custody::check_request(request)
            .map_err(RelationalNativeCheckpointCaptureDenial::Request)?;
        let member = if may_carry(row.member) {
            RelationalCheckpointMember::Carried {
                locator: row.member,
            }
        } else {
            let bytes = captured
                .cold_source
                .as_ref()
                .ok_or_else(|| corrupt("carried canonical member has no source"))?
                .read_member(row.member, row.member.length)
                .map_err(|cause| corrupt(&format!("carried canonical member: {cause:?}")))?;
            RelationalCheckpointMember::New {
                locator: row.member,
                bytes: worth_execution::ExecutionImmutableBytes::from_external_bytes(
                    std::sync::Arc::new(bytes.into_boxed_slice()),
                ),
            }
        };
        rows.push(*row);
        payloads.push(member).map_err(array_capture_stop)?;
    }
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
                contracts::storage::data::storage_unavailable::RelationalStorageSubject::Envelope(
                    contracts::publication::patch::data::patch_position::PatchStreamPosition(0),
                ),
            ))
        })?;
    for (offset, entry) in appended.iter().enumerate() {
        contracts::execution::request_custody::check_request(request)
            .map_err(RelationalNativeCheckpointCaptureDenial::Request)?;
        if entry.position.0 != covered + offset as u64 + 1 {
            return Err(corrupt("original canonical prefix is incomplete").into());
        }
        let cell = captured
            .envelope_cells
            .get(&entry.position)
            .ok_or_else(|| corrupt("original canonical cell unavailable"))?;
        let artifact = crate::durability::canonical_commit_artifact::retained_frame_reads::require_original_artifact(cell, request, source.as_ref().zip(protection.as_ref()))
            .map_err(capture_stop)?;
        if artifact.position() != entry.position
            || artifact.commit_id() != entry.commit.commit_id
            || artifact.payload_digest().as_bytes() != &entry.payload_digest
        {
            return Err(corrupt("original canonical artifact differs from captured index").into());
        }
        let bytes = artifact.into_bytes();
        let member =
            worth_relational_storage::internal_port::storage::data::member_frame::locator_for_request(
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
                &captured.commit_rows,
                covered,
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
pub use worth_relational_storage::internal_port::durability::medium::owned_read_stop::owned_stop;

pub(crate) use contracts::durability::data::native_checkpoint_capture_denial::{
    array_capture_stop, capture_stop,
};
