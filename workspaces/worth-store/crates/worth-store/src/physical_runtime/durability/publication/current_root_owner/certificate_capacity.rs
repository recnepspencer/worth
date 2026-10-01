//! Root-atomic checkpoint custody snapshot. Reopen cannot infer an empty
//! certificate ledger; C8 must install a selected basis before certification.

use std::sync::Arc;

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    encode_checkpoint_certificate, CheckpointCertificateKind, DurablePhysicalRootManifest,
    PhysicalCheckpointIdentity, PhysicalRecordFormatDeclaration, TierEpochActivationV1,
    TierEpochCheckpointCertificateV1, TierEpochWalFrameWitnessV1, MAX_CHECKPOINT_CERTIFICATE_BYTES,
    MAX_CHECKPOINT_CERTIFICATE_RECORDS,
};

use super::PhysicalCurrentRootOwner;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) enum CheckpointCustodyDenial {
    Unavailable,
    AnchorMismatch,
    ReleaseCertificateUnavailable,
}

#[derive(Clone)]
pub(in crate::physical_runtime) struct SelectedCheckpointCertificate {
    kind: CheckpointCertificateKind,
    payload: Arc<[u8]>,
}

impl SelectedCheckpointCertificate {
    pub(super) fn new(kind: CheckpointCertificateKind, payload: Vec<u8>) -> Self {
        Self {
            kind,
            payload: Arc::from(payload),
        }
    }

    pub(in crate::physical_runtime) fn kind(&self) -> CheckpointCertificateKind {
        self.kind
    }

    pub(in crate::physical_runtime) fn payload(&self) -> &[u8] {
        &self.payload
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::physical_runtime) enum CheckpointCustodyOrigin {
    FreshGenesis,
    ReopenRequiresC8,
}

pub(super) enum CheckpointCustodyState {
    Unavailable,
    VerifiedLegacyNoCertificates,
    ReleaseCertificatePending {
        attempt: [u8; 16],
        prior: Box<CheckpointCustodyState>,
    },
    CertifiedTier(SealedTierEpochCustodyBasis),
}

pub(super) struct SealedTierEpochCustodyBasis {
    intent: TierEpochActivationV1,
    intent_frame: TierEpochWalFrameWitnessV1,
    completed_frame: TierEpochWalFrameWitnessV1,
}

impl SealedTierEpochCustodyBasis {
    /// The exact C9 frame digests were admitted by this sealed physics claim
    /// and rejoined against current Store media. Carry that proof, do not
    /// encode the same metadata again during owner installation.
    pub(super) fn from_verified_selected(
        verified: &worth_store_recovery_physics::VerifiedSelectedTierEpochCustody,
    ) -> Self {
        Self {
            intent: verified.intent(),
            intent_frame: verified.intent_frame(),
            completed_frame: verified.completed_frame(),
        }
    }

    pub(super) fn new(
        intent: TierEpochActivationV1,
        intent_frame: TierEpochWalFrameWitnessV1,
        completed_frame: TierEpochWalFrameWitnessV1,
    ) -> Option<Self> {
        let intent_digest: [u8; 32] = Sha256::digest(intent.encode()).into();
        let completed_digest: [u8; 32] = Sha256::digest(intent.completed().encode()).into();
        (intent.phase() == worth_store_physical_format::TierEpochActivationPhaseV1::Intent
            && intent_frame.lsn_end_exclusive() <= completed_frame.lsn_start()
            && intent_frame.payload_digest() == intent_digest
            && completed_frame.payload_digest() == completed_digest)
            .then_some(Self {
                intent,
                intent_frame,
                completed_frame,
            })
    }

    fn bind(
        &self,
        checkpoint: PhysicalCheckpointIdentity,
        root: &DurablePhysicalRootManifest,
        root_sha256: [u8; 32],
    ) -> Result<SelectedCheckpointCertificate, CheckpointCustodyDenial> {
        let anchor = root
            .tier_epoch_anchor()
            .ok_or(CheckpointCustodyDenial::AnchorMismatch)?;
        let payload = TierEpochCheckpointCertificateV1::new(
            checkpoint,
            root.generation(),
            root_sha256,
            anchor,
            self.intent,
            self.intent_frame,
            self.completed_frame,
        )
        .map_err(|_| CheckpointCustodyDenial::AnchorMismatch)?
        .encode();
        Ok(SelectedCheckpointCertificate {
            kind: CheckpointCertificateKind::TierEpoch,
            payload: Arc::from(payload),
        })
    }
}

impl CheckpointCustodyState {
    pub(super) fn fulfill_selected_release_certificate(&mut self, attempt: [u8; 16]) -> bool {
        if !matches!(self, Self::ReleaseCertificatePending { attempt: pending, .. } if *pending == attempt)
        {
            return false;
        }
        match std::mem::replace(self, Self::Unavailable) {
            Self::ReleaseCertificatePending { prior, .. } => {
                *self = *prior;
                true
            }
            other => {
                *self = other;
                false
            }
        }
    }

    pub(super) fn require_release_certificate(&mut self, attempt: [u8; 16]) -> bool {
        if let Self::ReleaseCertificatePending {
            attempt: pending, ..
        } = self
        {
            return *pending == attempt;
        }
        let prior = std::mem::replace(self, Self::Unavailable);
        *self = Self::ReleaseCertificatePending {
            attempt,
            prior: Box::new(prior),
        };
        true
    }

    pub(super) fn restore_proven_no_effect(&mut self, attempt: [u8; 16]) -> bool {
        if !matches!(self, Self::ReleaseCertificatePending { attempt: pending, .. } if *pending == attempt)
        {
            return false;
        }
        match std::mem::replace(self, Self::Unavailable) {
            Self::ReleaseCertificatePending { prior, .. } => {
                *self = *prior;
                true
            }
            other => {
                *self = other;
                false
            }
        }
    }

    pub(super) fn from_origin(
        origin: CheckpointCustodyOrigin,
        root: &DurablePhysicalRootManifest,
    ) -> Self {
        match origin {
            CheckpointCustodyOrigin::FreshGenesis
                if root.generation() == 1 && root.tier_epoch_anchor().is_none() =>
            {
                Self::VerifiedLegacyNoCertificates
            }
            CheckpointCustodyOrigin::FreshGenesis | CheckpointCustodyOrigin::ReopenRequiresC8 => {
                Self::Unavailable
            }
        }
    }
}

pub(in crate::physical_runtime) enum SelectedCheckpointCustodySnapshot {
    VerifiedLegacy {
        checkpoint: PhysicalCheckpointIdentity,
        root: DurablePhysicalRootManifest,
        root_sha256: [u8; 32],
    },
    Certified {
        checkpoint: PhysicalCheckpointIdentity,
        root: DurablePhysicalRootManifest,
        root_sha256: [u8; 32],
        certificates: Arc<[SelectedCheckpointCertificate]>,
    },
}

impl SelectedCheckpointCustodySnapshot {
    pub(in crate::physical_runtime) fn checkpoint(&self) -> PhysicalCheckpointIdentity {
        match self {
            Self::VerifiedLegacy { checkpoint, .. } | Self::Certified { checkpoint, .. } => {
                *checkpoint
            }
        }
    }

    pub(in crate::physical_runtime) fn root(&self) -> &DurablePhysicalRootManifest {
        match self {
            Self::VerifiedLegacy { root, .. } | Self::Certified { root, .. } => root,
        }
    }

    pub(in crate::physical_runtime) fn root_sha256(&self) -> [u8; 32] {
        match self {
            Self::VerifiedLegacy { root_sha256, .. } | Self::Certified { root_sha256, .. } => {
                *root_sha256
            }
        }
    }

    pub(in crate::physical_runtime) fn certificates(
        &self,
    ) -> Option<&Arc<[SelectedCheckpointCertificate]>> {
        match self {
            Self::VerifiedLegacy { .. } => None,
            Self::Certified { certificates, .. } => Some(certificates),
        }
    }
}

impl PhysicalCurrentRootOwner {
    pub(in crate::physical_runtime) fn checkpoint_custody_snapshot(
        &self,
        checkpoint: PhysicalCheckpointIdentity,
        format: PhysicalRecordFormatDeclaration,
    ) -> Result<SelectedCheckpointCustodySnapshot, CheckpointCustodyDenial> {
        let state = self.lock_publication_state();
        let root = state.current_root.clone();
        let root_sha256 = Sha256::digest(root.encode(format)).into();
        if matches!(
            state.checkpoint_custody,
            CheckpointCustodyState::ReleaseCertificatePending { .. }
        ) {
            return Err(CheckpointCustodyDenial::ReleaseCertificateUnavailable);
        }
        let release_ledger = state
            .release_ledger
            .selected()
            .ok_or(CheckpointCustodyDenial::Unavailable)?;
        let release = release_ledger.certificates(checkpoint, &root, root_sha256)?;
        match &state.checkpoint_custody {
            CheckpointCustodyState::Unavailable => Err(CheckpointCustodyDenial::Unavailable),
            CheckpointCustodyState::ReleaseCertificatePending { .. } => {
                Err(CheckpointCustodyDenial::ReleaseCertificateUnavailable)
            }
            CheckpointCustodyState::VerifiedLegacyNoCertificates
                if root.tier_epoch_anchor().is_none() =>
            {
                if release.is_empty() {
                    Ok(SelectedCheckpointCustodySnapshot::VerifiedLegacy {
                        checkpoint,
                        root,
                        root_sha256,
                    })
                } else {
                    admit_bounds(&release)?;
                    Ok(SelectedCheckpointCustodySnapshot::Certified {
                        checkpoint,
                        root,
                        root_sha256,
                        certificates: Arc::from(release),
                    })
                }
            }
            CheckpointCustodyState::VerifiedLegacyNoCertificates => {
                Err(CheckpointCustodyDenial::AnchorMismatch)
            }
            CheckpointCustodyState::CertifiedTier(basis) => {
                let certificate = basis.bind(checkpoint, &root, root_sha256)?;
                let mut certificates = Vec::with_capacity(release.len() + 1);
                certificates.push(certificate);
                certificates.extend(release);
                admit_bounds(&certificates)?;
                Ok(SelectedCheckpointCustodySnapshot::Certified {
                    checkpoint,
                    root,
                    root_sha256,
                    certificates: Arc::from(certificates),
                })
            }
        }
    }
}

fn admit_bounds(
    certificates: &[SelectedCheckpointCertificate],
) -> Result<(), CheckpointCustodyDenial> {
    if certificates.len() as u64 > MAX_CHECKPOINT_CERTIFICATE_RECORDS {
        return Err(CheckpointCustodyDenial::ReleaseCertificateUnavailable);
    }
    let mut bytes = 0u64;
    for certificate in certificates {
        let encoded = encode_checkpoint_certificate(certificate.kind(), certificate.payload())
            .map_err(|_| CheckpointCustodyDenial::ReleaseCertificateUnavailable)?;
        bytes = bytes
            .checked_add(encoded.len() as u64)
            .ok_or(CheckpointCustodyDenial::ReleaseCertificateUnavailable)?;
    }
    (bytes <= MAX_CHECKPOINT_CERTIFICATE_BYTES)
        .then_some(())
        .ok_or(CheckpointCustodyDenial::ReleaseCertificateUnavailable)
}

#[cfg(test)]
#[path = "certificate_capacity/tests.rs"]
mod tests;
