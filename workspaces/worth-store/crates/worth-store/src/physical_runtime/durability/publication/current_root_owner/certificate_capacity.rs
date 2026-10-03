//! Root-atomic checkpoint custody snapshot. Reopen cannot infer an empty
//! certificate ledger: either ordinary reopen verifies the clean case itself
//! or C8 installs a selected basis before certification.

use std::sync::Arc;

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    CheckpointCertificateKind, DurablePhysicalRootManifest, PhysicalCheckpointIdentity,
    PhysicalRecordFormatDeclaration, TierEpochActivationV1, TierEpochCheckpointCertificateV1,
    TierEpochWalFrameWitnessV1,
};

use super::PhysicalCurrentRootOwner;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::physical_runtime) enum CheckpointCustodyDenial {
    Unavailable,
    AnchorMismatch,
    ReleaseCertificateUnavailable,
    Backing(crate::physical_runtime::PhysicalRecoveryRejoinResidentDenial),
}

pub(in crate::physical_runtime) use super::release_capacity::capture_envelope::CheckpointCertificateFrame;
use super::release_capacity::capture_envelope::{
    CheckpointCertificateViews, SealedCheckpointStorage,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::physical_runtime) enum CheckpointCustodyOrigin {
    FreshGenesis,
    ReopenRequiresC8,
    CleanReopen(super::clean_reopen::CleanReopenCheckpointCustody),
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
        let intent_digest: [u8; 32] = Sha256::digest(intent.encode_fixed()).into();
        let completed_digest: [u8; 32] = Sha256::digest(intent.completed().encode_fixed()).into();
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
    ) -> Result<TierEpochCheckpointCertificateV1, CheckpointCustodyDenial> {
        let anchor = root
            .tier_epoch_anchor()
            .ok_or(CheckpointCustodyDenial::AnchorMismatch)?;
        TierEpochCheckpointCertificateV1::new(
            checkpoint,
            root.generation(),
            root_sha256,
            anchor,
            self.intent,
            self.intent_frame,
            self.completed_frame,
        )
        .map_err(|_| CheckpointCustodyDenial::AnchorMismatch)
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
            CheckpointCustodyOrigin::CleanReopen(custody) => Self::from_clean_reopen(custody, root),
            CheckpointCustodyOrigin::FreshGenesis | CheckpointCustodyOrigin::ReopenRequiresC8 => {
                Self::Unavailable
            }
        }
    }

    fn from_clean_reopen(
        custody: super::clean_reopen::CleanReopenCheckpointCustody,
        root: &DurablePhysicalRootManifest,
    ) -> Self {
        use super::clean_reopen::CleanReopenCheckpointCustody as Clean;
        match (custody, root.tier_epoch_anchor()) {
            (Clean::TrustedGenesis { .. } | Clean::SelectedNoRelease { tier: None, .. }, None) => {
                Self::VerifiedLegacyNoCertificates
            }
            (
                Clean::SelectedNoRelease {
                    tier: Some(tier), ..
                },
                Some(anchor),
            ) if tier.anchor() == anchor => SealedTierEpochCustodyBasis::new(
                tier.intent(),
                tier.intent_frame(),
                tier.completed_frame(),
            )
            .map_or(Self::Unavailable, Self::CertifiedTier),
            _ => Self::Unavailable,
        }
    }
}

/// A certified selected checkpoint: its identity plus the sealed certificate
/// storage the checkpoint stream must carry.
pub(in crate::physical_runtime) struct SelectedCheckpointCustodySnapshot {
    checkpoint: PhysicalCheckpointIdentity,
    root: DurablePhysicalRootManifest,
    root_sha256: [u8; 32],
    storage: Arc<SealedCheckpointStorage>,
}

impl SelectedCheckpointCustodySnapshot {
    pub(in crate::physical_runtime) fn checkpoint(&self) -> PhysicalCheckpointIdentity {
        self.checkpoint
    }

    pub(in crate::physical_runtime) fn root(&self) -> &DurablePhysicalRootManifest {
        &self.root
    }

    pub(in crate::physical_runtime) fn root_sha256(&self) -> [u8; 32] {
        self.root_sha256
    }

    pub(in crate::physical_runtime) fn certificates(&self) -> CheckpointCertificateViews<'_> {
        self.storage.views()
    }
    pub(in crate::physical_runtime) fn certificate_frame(
        &self,
        index: usize,
    ) -> Option<CheckpointCertificateFrame> {
        self.storage.frame(index)
    }
    pub(super) fn storage(&self) -> &Arc<SealedCheckpointStorage> {
        &self.storage
    }
}

impl PhysicalCurrentRootOwner {
    /// The envelope the next capture holds, so a journey can check the
    /// capture's real allocations against it.
    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime) fn checkpoint_capture_envelope_bytes(&self) -> Option<u64> {
        let state = self.lock_publication_state();
        let tier = matches!(
            state.checkpoint_custody,
            CheckpointCustodyState::CertifiedTier(_)
        );
        let ledger = state.release_ledger.selected()?;
        let envelope = ledger.checkpoint_capture_envelope(None, tier).ok()?;
        Some(envelope.bytes())
    }

    /// Consumes the standing checkpoint reservation, before any effect, then
    /// materializes the selected certificates into its plain buffers.
    pub(in crate::physical_runtime) fn checkpoint_custody_snapshot(
        &self,
        checkpoint: PhysicalCheckpointIdentity,
        format: PhysicalRecordFormatDeclaration,
    ) -> Result<SelectedCheckpointCustodySnapshot, CheckpointCustodyDenial> {
        let mut guard = self.lock_publication_state();
        let state = &mut *guard;
        let root = state.current_root.clone();
        let tier = match &state.checkpoint_custody {
            CheckpointCustodyState::ReleaseCertificatePending { .. } => {
                return Err(CheckpointCustodyDenial::ReleaseCertificateUnavailable)
            }
            CheckpointCustodyState::Unavailable => {
                return Err(CheckpointCustodyDenial::Unavailable)
            }
            CheckpointCustodyState::VerifiedLegacyNoCertificates
                if root.tier_epoch_anchor().is_some() =>
            {
                return Err(CheckpointCustodyDenial::AnchorMismatch)
            }
            CheckpointCustodyState::VerifiedLegacyNoCertificates => None,
            CheckpointCustodyState::CertifiedTier(basis) => Some(basis),
        };
        let ledger = state
            .release_ledger
            .selected_mut()
            .ok_or(CheckpointCustodyDenial::Unavailable)?;
        let mut preparation = ledger.prepare_capture(
            &self.release_allocation,
            self.recovery_allocation,
            tier.is_some(),
        )?;
        let observer = &mut preparation.observer;
        observer.scratch = root
            .encode_in_reserved(format, std::mem::take(&mut observer.scratch))
            .ok_or(CheckpointCustodyDenial::ReleaseCertificateUnavailable)?;
        let root_sha256 = Sha256::digest(&observer.scratch).into();
        if let Some(basis) = tier {
            let payload = basis.bind(checkpoint, &root, root_sha256)?;
            observer.scratch = payload
                .encode_in_reserved(std::mem::take(&mut observer.scratch))
                .ok_or(CheckpointCustodyDenial::ReleaseCertificateUnavailable)?;
            observer.push(CheckpointCertificateKind::TierEpoch)?;
        }
        ledger.materialize_certificates(checkpoint, &root, root_sha256, &mut preparation)?;
        Ok(SelectedCheckpointCustodySnapshot {
            checkpoint,
            root,
            root_sha256,
            storage: Arc::new(SealedCheckpointStorage::from_prepared(preparation)),
        })
    }
}

#[cfg(test)]
#[path = "certificate_capacity/tests.rs"]
mod tests;
