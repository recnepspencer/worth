//! The one Store-side verifier of a selected checkpoint's positive NoRelease
//! marker, shared by ordinary clean reopen and the C.8 selected rejoin.

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    decode_checkpoint_certificate, CheckpointCertificateKind, PhysicalCheckpointIdentity,
    ReleaseCheckpointCertificateV1, ReleaseCheckpointNoReleaseV1,
};

/// The selected marker plus the optional leading tier certificate payload.
pub(in crate::physical_runtime) struct SelectedNoReleaseCertificates<'a> {
    marker: ReleaseCheckpointNoReleaseV1,
    marker_payload_sha256: [u8; 32],
    tier_payload: Option<&'a [u8]>,
}

impl<'a> SelectedNoReleaseCertificates<'a> {
    pub(in crate::physical_runtime) const fn marker(&self) -> ReleaseCheckpointNoReleaseV1 {
        self.marker
    }

    pub(in crate::physical_runtime) const fn marker_payload_sha256(&self) -> [u8; 32] {
        self.marker_payload_sha256
    }

    pub(in crate::physical_runtime) const fn tier_payload(&self) -> Option<&'a [u8]> {
        self.tier_payload
    }
}

/// Selects the roster's single canonical NoRelease marker bound to
/// `checkpoint` and its source root. The only other admitted record is one
/// leading TierEpoch certificate, and only when `allow_tier` holds.
pub(in crate::physical_runtime) fn select_no_release_marker<'a>(
    frames: impl IntoIterator<Item = &'a [u8]>,
    checkpoint: PhysicalCheckpointIdentity,
    source_root_generation: u64,
    source_root_sha256: [u8; 32],
    allow_tier: bool,
) -> Option<SelectedNoReleaseCertificates<'a>> {
    let mut selected = None;
    let mut tier_payload = None;
    for frame in frames {
        let (kind, payload) = decode_checkpoint_certificate(frame).ok()?;
        if kind == CheckpointCertificateKind::TierEpoch {
            if !allow_tier || tier_payload.is_some() || selected.is_some() {
                return None;
            }
            tier_payload = Some(payload);
            continue;
        }
        let ReleaseCheckpointCertificateV1::NoRelease(marker) =
            ReleaseCheckpointCertificateV1::decode(payload).ok()?
        else {
            return None;
        };
        if selected.replace(marker).is_some()
            || payload != marker.encode()
            || marker.checkpoint() != checkpoint
            || marker.root_generation() != source_root_generation
            || marker.root_sha256() != source_root_sha256
        {
            return None;
        }
    }
    let marker = selected?;
    Some(SelectedNoReleaseCertificates {
        marker,
        marker_payload_sha256: Sha256::digest(marker.encode()).into(),
        tier_payload,
    })
}
