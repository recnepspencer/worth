//! Positive tag-7 NoRelease custody is joined to the same selected C.9
//! checkpoint and Store-admitted WAL inventory as tier custody. Absence alone
//! is never an empty release ledger.

use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use worth_store_physical_format::{
    decode_canonical_redo_v3, decode_checkpoint_certificate, CheckpointCertificateKind,
    PersistedPhysicalRecoveryBlobSemantic, PhysicalRecordFormatDeclaration,
    PhysicalRecoveryProjectionDecodeLimits, ReleaseCheckpointCertificateV1,
};
use worth_store_recovery_physics::{
    VerifiedSelectedNoReleaseCustody, VerifiedSelectedTierEpochCustody,
};

use super::super::{SelectedMediaRejoinDenial as Denial, MAX_DISCOVERY_ENTRIES};
use super::routes::NoReleaseControlProvenance;
use crate::physical_runtime::StoreRecoveryBindingFreshnessSample;

pub(super) fn verify_claims(
    tier: &VerifiedSelectedTierEpochCustody,
    no_release: &VerifiedSelectedNoReleaseCustody,
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
    verify_marker_claim(no_release, true)
}

pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn verify_marker_claim(
    no_release: &VerifiedSelectedNoReleaseCustody,
    allow_tier: bool,
) -> Result<(), Denial> {
    let mut selected = None;
    let mut tier_seen = false;
    for frame in no_release.checkpoint().certificate_records() {
        let (kind, payload) =
            decode_checkpoint_certificate(frame).map_err(|_| Denial::CertificateRoster)?;
        if kind == CheckpointCertificateKind::TierEpoch {
            if !allow_tier || tier_seen || selected.is_some() {
                return Err(Denial::CertificateRoster);
            }
            tier_seen = true;
            continue;
        }
        let ReleaseCheckpointCertificateV1::NoRelease(marker) =
            ReleaseCheckpointCertificateV1::decode(payload)
                .map_err(|_| Denial::CertificateRoster)?
        else {
            return Err(Denial::CertificateRoster);
        };
        if selected.replace(marker).is_some()
            || payload != marker.encode()
            || marker.checkpoint() != no_release.checkpoint().source().identity()
            || marker.root_generation() != no_release.checkpoint().source().root().generation()
            || marker.root_sha256() != no_release.checkpoint_source_root_sha256()
        {
            return Err(Denial::CertificateRoster);
        }
    }
    let Some(marker) = selected else {
        return Err(Denial::CertificateRoster);
    };
    let marker_sha256: [u8; 32] = Sha256::digest(marker.encode()).into();
    if marker != no_release.marker() || marker_sha256 != no_release.marker_payload_sha256() {
        return Err(Denial::CertificateRoster);
    }
    Ok(())
}

pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn verify_selected_tail(
    sample: &StoreRecoveryBindingFreshnessSample,
    controls: &NoReleaseControlProvenance,
    format: PhysicalRecordFormatDeclaration,
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
        if let PersistedPhysicalRecoveryBlobSemantic::RecordsDropped(binding) =
            projection.blob_semantic()
        {
            if !controls.admits_drop(binding) || !seen_drops.insert(binding.record()) {
                return Err(Denial::WalFate);
            }
        }
    }
    Ok(())
}
