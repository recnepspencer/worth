//! Positive tag-7 NoRelease custody is joined to the same selected C.9
//! checkpoint and Store-admitted WAL inventory as tier custody. Absence alone
//! is never an empty release ledger.

use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use worth_store_physical_format::{
    decode_canonical_redo_v3, PersistedBlobSemanticRecordBinding,
    PersistedPhysicalRecoveryOperation, PhysicalExtentCopyRecord, PhysicalRecordFormatDeclaration,
    PhysicalRecoveryProjectionDecodeLimits,
};
use worth_store_recovery_physics::{
    admit_current_source_copy_publication, VerifiedSelectedNoReleaseCustody,
    VerifiedSelectedTierEpochCustody,
};

#[path = "no_release/rewrite.rs"]
mod rewrite;

use super::super::{SelectedMediaRejoinDenial as Denial, MAX_DISCOVERY_ENTRIES};
use super::routes::NoReleaseControlProvenance;
use crate::physical_runtime::StoreRecoveryBindingFreshnessSample;

pub(super) fn verify_claims(
    tier: &VerifiedSelectedTierEpochCustody,
    no_release: &VerifiedSelectedNoReleaseCustody,
    checkpoint: &worth_store_physical_integrity::VerifiedCheckpointStream,
) -> Result<(), Denial> {
    if tier.selected_root() != no_release.selected_root()
        || tier.selected_root_sha256() != no_release.selected_root_sha256()
        || tier.checkpoint().source().identity() != no_release.checkpoint().source().identity()
        || tier.checkpoint().encoded_bytes() != no_release.checkpoint().encoded_bytes()
        || tier.checkpoint().encoded_digest() != no_release.checkpoint().encoded_digest()
        || tier.checkpoint_source_root_sha256() != no_release.checkpoint_source_root_sha256()
    {
        return Err(Denial::CertificateRoster);
    }
    verify_marker_claim(no_release, true, checkpoint)
}

pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn verify_marker_claim(
    no_release: &VerifiedSelectedNoReleaseCustody,
    allow_tier: bool,
    checkpoint: &worth_store_physical_integrity::VerifiedCheckpointStream,
) -> Result<(), Denial> {
    let selected = crate::physical_runtime::durability::select_no_release_marker(
        checkpoint.certificate_records().iter().map(AsRef::as_ref),
        no_release.checkpoint().source().identity(),
        no_release.checkpoint().source().root().generation(),
        no_release.checkpoint_source_root_sha256(),
        allow_tier,
    )
    .ok_or(Denial::CertificateRoster)?;
    if selected.marker() != no_release.marker()
        || selected.marker_payload_sha256() != no_release.marker_payload_sha256()
    {
        return Err(Denial::CertificateRoster);
    }
    Ok(())
}

pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn verify_selected_tail(
    sample: &StoreRecoveryBindingFreshnessSample,
    controls: &NoReleaseControlProvenance,
    format: PhysicalRecordFormatDeclaration,
    selected_root_generation: u64,
) -> Result<(), Denial> {
    let mut seen_drops = BTreeSet::new();
    for member in sample.wal_members() {
        let limit = MAX_DISCOVERY_ENTRIES.min(member.canonical_redo().len() as u64);
        let limits = PhysicalRecoveryProjectionDecodeLimits {
            frames: limit,
            record_identities: limit,
            placements: limit,
            segment_updates: limit,
            manifests: limit,
            total_entries: limit.saturating_mul(3),
            inline_allocations: limit,
        };
        if let Some(projection) = admit_current_source_copy_publication(
            member.operation_identity(),
            member.lsn_range(),
            member.canonical_redo(),
            format,
            limits,
        )
        .map_err(|_| Denial::WalFate)?
        {
            let worth_store_physical_format::PersistedPhysicalRecoveryPayload::SourceCopy(recipe) =
                projection.payload()
            else {
                return Err(Denial::WalFate);
            };
            if !matches_exact_copy_intent(sample.extent_copy_frames(), *recipe, format) {
                return Err(Denial::WalFate);
            }
            continue;
        }
        if rewrite::admit_selected_rewrite(
            member.canonical_redo(),
            member.operation_identity(),
            member.group_identity(),
            member.lsn_range(),
            sample.operations().iter().map(|operation| {
                (
                    operation.idempotency_identity(),
                    operation.request_fingerprint().bytes(),
                )
            }),
            selected_root_generation,
        )? {
            continue;
        }
        let (_, projection) = decode_canonical_redo_v3(
            member.canonical_redo(),
            member.lsn_range().start().get(),
            member.lsn_range().end_exclusive().get(),
            limit,
            None,
            limits,
            format,
        )
        .map_err(|_| Denial::WalFate)?;
        if let Some(binding) = tail_drop(projection.operation())? {
            if !controls.admits_drop(*binding) || !seen_drops.insert(binding.record()) {
                return Err(Denial::WalFate);
            }
        }
    }
    Ok(())
}

/// The drop a selected-tail member carries, if any. NoRelease custody does
/// not yet admit a terminal head retirement member.
fn tail_drop(
    operation: &PersistedPhysicalRecoveryOperation,
) -> Result<Option<&PersistedBlobSemanticRecordBinding>, Denial> {
    use PersistedPhysicalRecoveryOperation as Operation;
    match operation {
        Operation::RecordsDropped { binding, .. } => Ok(Some(binding)),
        Operation::TerminalReleaseHeadRetired(_) => Err(Denial::WalFate),
        Operation::None
        | Operation::SessionDeclared(_)
        | Operation::GenerationPublished(_)
        | Operation::SessionFrontier(_)
        | Operation::SessionAbandoned(_)
        | Operation::ChunkReused(_)
        | Operation::DedupeQuarantined(_)
        | Operation::DerivedDirectory { .. } => Ok(None),
    }
}

fn matches_exact_copy_intent<'a>(
    frames: impl Iterator<Item = (worth_store_wal::WalLsnRange, &'a [u8])>,
    recipe: worth_store_physical_format::PersistedExtentCopyRecipe,
    format: PhysicalRecordFormatDeclaration,
) -> bool {
    let mut selected = None;
    for (range, bytes) in frames {
        if range.start().get() != recipe.intent_lsn() {
            continue;
        }
        if selected.replace((range, bytes)).is_some() {
            return false;
        }
    }
    let Some((range, bytes)) = selected else {
        return false;
    };
    recipe.intent_lsn().checked_add(1) == Some(range.end_exclusive().get())
        && <[u8; 32]>::from(Sha256::digest(bytes)) == recipe.intent_digest()
        && matches!(
            PhysicalExtentCopyRecord::decode(bytes, format),
            Ok(PhysicalExtentCopyRecord::Intent(intent)) if intent == recipe.intent()
        )
}

#[cfg(test)]
#[path = "no_release/tests.rs"]
mod tests;
