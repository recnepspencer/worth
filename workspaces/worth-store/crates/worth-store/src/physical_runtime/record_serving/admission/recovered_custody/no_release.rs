//! NoRelease custody a recovered seal carries: a selected checkpoint's
//! positive marker, or, before the first checkpoint, the generation-zero
//! basis, under which no release can have been selected.

use super::*;

pub(in crate::physical_runtime) enum RecoveredNoReleaseCustody {
    Selected(VerifiedSelectedNoReleaseCustody),
    GenerationZero(GenerationZeroNoReleaseCustody),
}

/// Recovery observed `checkpoint.current` absent, admitted the whole WAL from
/// the canonical origin, and rejoined this unanchored root against Store
/// media. Only that rejoin constructs it.
pub(in crate::physical_runtime) struct GenerationZeroNoReleaseCustody {
    selected_root: worth_store_physical_format::DurablePhysicalRootManifest,
    selected_root_sha256: [u8; 32],
}

#[cfg(feature = "recovery-runtime-owner")]
impl GenerationZeroNoReleaseCustody {
    pub(in crate::physical_runtime) fn rejoined(
        selected_root: worth_store_physical_format::DurablePhysicalRootManifest,
        selected_root_sha256: [u8; 32],
    ) -> Self {
        Self {
            selected_root,
            selected_root_sha256,
        }
    }
}

#[cfg(feature = "recovery-runtime-owner")]
impl RecoveredNoReleaseCustody {
    /// The selected checkpoint's marker claim; none before the first checkpoint.
    pub(in crate::physical_runtime) fn selected(
        &self,
    ) -> Option<&VerifiedSelectedNoReleaseCustody> {
        match self {
            Self::Selected(claim) => Some(claim),
            Self::GenerationZero(_) => None,
        }
    }

    pub(in crate::physical_runtime) fn selected_root(
        &self,
    ) -> &worth_store_physical_format::DurablePhysicalRootManifest {
        match self {
            Self::Selected(claim) => claim.selected_root(),
            Self::GenerationZero(basis) => &basis.selected_root,
        }
    }

    pub(in crate::physical_runtime) fn selected_root_sha256(&self) -> [u8; 32] {
        match self {
            Self::Selected(claim) => claim.selected_root_sha256(),
            Self::GenerationZero(basis) => basis.selected_root_sha256,
        }
    }
}

impl RecoveredCheckpointCustodyEvidence {
    #[cfg(feature = "recovery-runtime-owner")]
    pub(super) fn verify_no_release(
        &self,
        verified: &RecoveredNoReleaseCustody,
        store: StableStoreIdentity,
        root: &DurablePhysicalRootManifest,
        actual_sha256: [u8; 32],
    ) -> Result<(), RecoveredCheckpointCustodyDenial> {
        match verified {
            RecoveredNoReleaseCustody::Selected(claim) => {
                self.verify_selected_no_release(claim, store, root, actual_sha256)
            }
            // The custody exists only under its coordination's absence
            // witness, so the ownership it travels with is already Absent.
            RecoveredNoReleaseCustody::GenerationZero(basis) => {
                if basis.selected_root != *root
                    || basis.selected_root_sha256 != actual_sha256
                    || root.tier_epoch_anchor().is_some()
                {
                    return Err(RecoveredCheckpointCustodyDenial::SelectedCheckpointMismatch);
                }
                Ok(())
            }
        }
    }

    #[cfg(feature = "recovery-runtime-owner")]
    fn verify_selected_no_release(
        &self,
        verified: &VerifiedSelectedNoReleaseCustody,
        store: StableStoreIdentity,
        root: &DurablePhysicalRootManifest,
        actual_sha256: [u8; 32],
    ) -> Result<(), RecoveredCheckpointCustodyDenial> {
        let marker = verified.marker();
        let stream = self.checkpoint_stream(verified.checkpoint())?;
        let source = stream.source();
        if verified.selected_root() != root
            || verified.selected_root_sha256() != actual_sha256
            || source.identity().store_identity() != store
            || source.root().generation() > root.generation()
            || marker.checkpoint() != source.identity()
            || marker.root_generation() != source.root().generation()
            || marker.root_sha256() != verified.checkpoint_source_root_sha256()
            || <[u8; 32]>::from(Sha256::digest(marker.encode())) != verified.marker_payload_sha256()
        {
            return Err(RecoveredCheckpointCustodyDenial::SelectedCheckpointMismatch);
        }
        let mut selected = 0;
        for frame in stream.certificate_records() {
            let (kind, payload) = decode_checkpoint_certificate(frame)
                .map_err(|_| RecoveredCheckpointCustodyDenial::SelectedCheckpointMismatch)?;
            if kind == CheckpointCertificateKind::ReleasedDrop {
                let ReleaseCheckpointCertificateV1::NoRelease(value) =
                    ReleaseCheckpointCertificateV1::decode(payload).map_err(|_| {
                        RecoveredCheckpointCustodyDenial::SelectedCheckpointMismatch
                    })?
                else {
                    return Err(RecoveredCheckpointCustodyDenial::SelectedCheckpointMismatch);
                };
                if value != marker || payload != marker.encode() {
                    return Err(RecoveredCheckpointCustodyDenial::SelectedCheckpointMismatch);
                }
                selected += 1;
            }
        }
        if selected != 1 {
            return Err(RecoveredCheckpointCustodyDenial::SelectedCheckpointMismatch);
        }
        Ok(())
    }
}
