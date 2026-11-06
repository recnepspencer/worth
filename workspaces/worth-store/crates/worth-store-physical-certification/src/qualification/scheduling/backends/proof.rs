use worth_foundational::CanonicalDigestId;
use worth_store_physical_backend::{BackendCapabilityKind, BackendTargetProfile};

use crate::{IoPressureEvidenceMaturity, IoPressureHarnessEvidence, PhysicalSimulationProfile};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QualificationHarnessProofClaim {
    IoPressureEnvelope,
    BufferedFile,
    DirectIo,
    Mmap,
    AsyncIo,
    FlushDurability,
    DirectorySync,
    DurableRename,
    SecureFrameIo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QualificationHarnessProofStrength {
    SimulationOnly,
    ExplicitBackendQualification,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QualificationHarnessProof {
    backend_profile: BackendTargetProfile,
    replay_profile: PhysicalSimulationProfile,
    replay_identity: CanonicalDigestId,
    maturity: IoPressureEvidenceMaturity,
    claim: QualificationHarnessProofClaim,
    strength: QualificationHarnessProofStrength,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QualificationCapabilityProofAuthority {
    _private: (),
}

impl QualificationHarnessProof {
    pub fn from_io_pressure_evidence(evidence: &IoPressureHarnessEvidence) -> Self {
        Self {
            backend_profile: evidence.scenario().backend_profile(),
            replay_profile: evidence.replay_profile(),
            replay_identity: CanonicalDigestId::new(*evidence.replay_identity()),
            maturity: evidence.maturity(),
            claim: QualificationHarnessProofClaim::IoPressureEnvelope,
            strength: proof_strength_for(evidence),
        }
    }

    pub fn from_executed_buffered_file_evidence(
        _authority: QualificationCapabilityProofAuthority,
        evidence: &IoPressureHarnessEvidence,
    ) -> Self {
        Self::from_executed_capability_evidence(
            evidence,
            QualificationHarnessProofClaim::BufferedFile,
        )
    }

    pub fn from_executed_direct_io_evidence(
        _authority: QualificationCapabilityProofAuthority,
        evidence: &IoPressureHarnessEvidence,
    ) -> Self {
        Self::from_executed_capability_evidence(evidence, QualificationHarnessProofClaim::DirectIo)
    }

    pub fn from_executed_mmap_evidence(
        _authority: QualificationCapabilityProofAuthority,
        evidence: &IoPressureHarnessEvidence,
    ) -> Self {
        Self::from_executed_capability_evidence(evidence, QualificationHarnessProofClaim::Mmap)
    }

    pub fn from_executed_async_io_evidence(
        _authority: QualificationCapabilityProofAuthority,
        evidence: &IoPressureHarnessEvidence,
    ) -> Self {
        Self::from_executed_capability_evidence(evidence, QualificationHarnessProofClaim::AsyncIo)
    }

    pub fn from_executed_flush_durability_evidence(
        _authority: QualificationCapabilityProofAuthority,
        evidence: &IoPressureHarnessEvidence,
    ) -> Self {
        Self::from_executed_capability_evidence(
            evidence,
            QualificationHarnessProofClaim::FlushDurability,
        )
    }

    pub fn from_executed_directory_sync_evidence(
        _authority: QualificationCapabilityProofAuthority,
        evidence: &IoPressureHarnessEvidence,
    ) -> Self {
        Self::from_executed_capability_evidence(
            evidence,
            QualificationHarnessProofClaim::DirectorySync,
        )
    }

    pub fn from_executed_durable_rename_evidence(
        _authority: QualificationCapabilityProofAuthority,
        evidence: &IoPressureHarnessEvidence,
    ) -> Self {
        Self::from_executed_capability_evidence(
            evidence,
            QualificationHarnessProofClaim::DurableRename,
        )
    }

    pub fn from_executed_secure_frame_io_evidence(
        _authority: QualificationCapabilityProofAuthority,
        evidence: &IoPressureHarnessEvidence,
    ) -> Self {
        Self::from_executed_capability_evidence(
            evidence,
            QualificationHarnessProofClaim::SecureFrameIo,
        )
    }

    fn from_executed_capability_evidence(
        evidence: &IoPressureHarnessEvidence,
        claim: QualificationHarnessProofClaim,
    ) -> Self {
        Self {
            backend_profile: evidence.scenario().backend_profile(),
            replay_profile: evidence.replay_profile(),
            replay_identity: CanonicalDigestId::new(*evidence.replay_identity()),
            maturity: evidence.maturity(),
            claim,
            strength: proof_strength_for(evidence),
        }
    }

    pub const fn backend_profile(self) -> BackendTargetProfile {
        self.backend_profile
    }

    pub const fn replay_profile(self) -> PhysicalSimulationProfile {
        self.replay_profile
    }

    pub const fn replay_identity(self) -> [u8; 32] {
        *self.replay_identity.bytes()
    }

    pub const fn maturity(self) -> IoPressureEvidenceMaturity {
        self.maturity
    }

    pub const fn claim(self) -> QualificationHarnessProofClaim {
        self.claim
    }

    pub const fn strength(self) -> QualificationHarnessProofStrength {
        self.strength
    }

    pub const fn covers(self, capability: BackendCapabilityKind) -> bool {
        matches!(
            (self.claim, capability),
            (
                QualificationHarnessProofClaim::BufferedFile,
                BackendCapabilityKind::BufferedFile
            ) | (
                QualificationHarnessProofClaim::DirectIo,
                BackendCapabilityKind::DirectIo
            ) | (
                QualificationHarnessProofClaim::Mmap,
                BackendCapabilityKind::Mmap
            ) | (
                QualificationHarnessProofClaim::AsyncIo,
                BackendCapabilityKind::AsyncIo
            ) | (
                QualificationHarnessProofClaim::FlushDurability,
                BackendCapabilityKind::Fsync
            ) | (
                QualificationHarnessProofClaim::DirectorySync,
                BackendCapabilityKind::DirectorySync
            ) | (
                QualificationHarnessProofClaim::DurableRename,
                BackendCapabilityKind::DurableRename
            ) | (
                QualificationHarnessProofClaim::SecureFrameIo,
                BackendCapabilityKind::SecureFrameIo
            )
        )
    }
}

fn proof_strength_for(evidence: &IoPressureHarnessEvidence) -> QualificationHarnessProofStrength {
    if evidence.replay_profile() == PhysicalSimulationProfile::HardwareQualification
        && evidence.maturity() == IoPressureEvidenceMaturity::BackendCertified
    {
        QualificationHarnessProofStrength::ExplicitBackendQualification
    } else {
        QualificationHarnessProofStrength::SimulationOnly
    }
}

impl QualificationCapabilityProofAuthority {
    pub(crate) const fn from_executed_store_evidence() -> Self {
        Self { _private: () }
    }
}
