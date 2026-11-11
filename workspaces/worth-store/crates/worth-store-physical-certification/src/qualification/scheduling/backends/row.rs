use worth_foundational::CanonicalDigestId;
use worth_store_physical_backend::{
    AdmittedBackendCapabilityWitness, BackendCapabilityKind, BackendCapabilitySupportPosture,
    BackendMediaAssumptionSet, BackendRebindTriggers, BackendTargetProfile,
    CapabilityConfidenceLimits, CapabilityEvidenceClass,
};

use super::support::published_posture;
use super::{
    BackendQualificationMatrixDenial, PublishedQualificationPosture, QualificationHarnessProof,
    QualificationHarnessProofStrength, QualificationResidualDebt,
};
use crate::IoPressureHarnessEvidence;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackendQualificationRowIdentity {
    profile: BackendTargetProfile,
    capability: BackendCapabilityKind,
    evidence_class: CapabilityEvidenceClass,
    support_posture: BackendCapabilitySupportPosture,
    media_assumptions: BackendMediaAssumptionSet,
    rebind_triggers: BackendRebindTriggers,
    residual_debt: QualificationResidualDebt,
    harness_proof: QualificationHarnessProof,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackendQualificationRow {
    profile: BackendTargetProfile,
    capability: BackendCapabilityKind,
    evidence_class: CapabilityEvidenceClass,
    support_posture: BackendCapabilitySupportPosture,
    media_assumptions: BackendMediaAssumptionSet,
    rebind_triggers: BackendRebindTriggers,
    confidence_limits: CapabilityConfidenceLimits,
    residual_debt: QualificationResidualDebt,
    harness_proof: QualificationHarnessProof,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CertifiedBackendQualificationSupport {
    profile: BackendTargetProfile,
    capability: BackendCapabilityKind,
    evidence_class: CapabilityEvidenceClass,
    replay_identity: CanonicalDigestId,
}

impl BackendQualificationRow {
    #[cfg(test)]
    pub(crate) fn from_admitted_backend_witness(
        witness: &AdmittedBackendCapabilityWitness,
        _capability: BackendCapabilityKind,
        evidence: &IoPressureHarnessEvidence,
    ) -> Result<Self, BackendQualificationMatrixDenial> {
        if witness.profile() != evidence.scenario().backend_profile() {
            return Err(BackendQualificationMatrixDenial::ProfileMismatch {
                expected: witness.profile(),
                actual: evidence.scenario().backend_profile(),
            });
        }
        Err(BackendQualificationMatrixDenial::MissingHarnessProof)
    }

    pub(crate) fn from_admitted_backend_witness_with_proof(
        witness: &AdmittedBackendCapabilityWitness,
        capability: BackendCapabilityKind,
        evidence: &IoPressureHarnessEvidence,
        harness_proof: QualificationHarnessProof,
    ) -> Result<Self, BackendQualificationMatrixDenial> {
        let support_posture = witness.support().posture(capability);
        let residual_debt = default_residual_debt(
            capability,
            support_posture,
            witness.evidence_class(),
            witness.rebind_triggers(),
        );
        Self::from_admitted_backend_witness_with_proof_and_residual_debt(
            witness,
            capability,
            evidence,
            harness_proof,
            residual_debt,
        )
    }

    #[cfg(test)]
    pub(crate) fn from_admitted_backend_witness_with_residual_debt(
        witness: &AdmittedBackendCapabilityWitness,
        capability: BackendCapabilityKind,
        evidence: &IoPressureHarnessEvidence,
        residual_debt: QualificationResidualDebt,
    ) -> Result<Self, BackendQualificationMatrixDenial> {
        if witness.profile() != evidence.scenario().backend_profile() {
            return Err(BackendQualificationMatrixDenial::ProfileMismatch {
                expected: witness.profile(),
                actual: evidence.scenario().backend_profile(),
            });
        }
        validate_residual_debt(
            capability,
            witness.support().posture(capability),
            residual_debt,
        )?;
        Err(BackendQualificationMatrixDenial::MissingHarnessProof)
    }

    pub(crate) fn from_admitted_backend_witness_with_proof_and_residual_debt(
        witness: &AdmittedBackendCapabilityWitness,
        capability: BackendCapabilityKind,
        evidence: &IoPressureHarnessEvidence,
        harness_proof: QualificationHarnessProof,
        residual_debt: QualificationResidualDebt,
    ) -> Result<Self, BackendQualificationMatrixDenial> {
        if witness.profile() != evidence.scenario().backend_profile() {
            return Err(BackendQualificationMatrixDenial::ProfileMismatch {
                expected: witness.profile(),
                actual: evidence.scenario().backend_profile(),
            });
        }
        let support_posture = witness.support().posture(capability);
        validate_harness_proof(
            capability,
            witness.evidence_class(),
            evidence,
            harness_proof,
        )?;
        validate_residual_debt(capability, support_posture, residual_debt)?;
        Ok(Self {
            profile: witness.profile(),
            capability,
            evidence_class: witness.evidence_class(),
            support_posture,
            media_assumptions: witness.media_assumptions(),
            rebind_triggers: witness.rebind_triggers(),
            confidence_limits: witness.confidence_limits(),
            residual_debt,
            harness_proof,
        })
    }

    pub const fn profile(&self) -> BackendTargetProfile {
        self.profile
    }

    pub const fn capability(&self) -> BackendCapabilityKind {
        self.capability
    }

    pub const fn evidence_class(&self) -> CapabilityEvidenceClass {
        self.evidence_class
    }

    pub const fn support_posture(&self) -> BackendCapabilitySupportPosture {
        self.support_posture
    }

    pub const fn media_assumptions(&self) -> BackendMediaAssumptionSet {
        self.media_assumptions
    }

    pub const fn rebind_triggers(&self) -> BackendRebindTriggers {
        self.rebind_triggers
    }

    pub const fn residual_debt(&self) -> QualificationResidualDebt {
        self.residual_debt
    }

    pub const fn harness_proof(&self) -> QualificationHarnessProof {
        self.harness_proof
    }

    pub const fn identity(&self) -> BackendQualificationRowIdentity {
        BackendQualificationRowIdentity {
            profile: self.profile,
            capability: self.capability,
            evidence_class: self.evidence_class,
            support_posture: self.support_posture,
            media_assumptions: self.media_assumptions,
            rebind_triggers: self.rebind_triggers,
            residual_debt: self.residual_debt,
            harness_proof: self.harness_proof,
        }
    }

    pub const fn published_posture(&self) -> PublishedQualificationPosture {
        published_posture(self.support_posture, self.residual_debt)
    }

    pub fn require_support(
        &self,
        required_evidence: CapabilityEvidenceClass,
    ) -> Result<CertifiedBackendQualificationSupport, BackendQualificationMatrixDenial> {
        if !self.evidence_class.satisfies(required_evidence) {
            return Err(BackendQualificationMatrixDenial::EvidenceClassTooWeak {
                required: required_evidence,
                actual: self.evidence_class,
            });
        }
        if !self.confidence_limits.can_back_runtime_claim() {
            return Err(BackendQualificationMatrixDenial::ConfidenceLimitTooWeak);
        }
        if !self.media_assumptions.supports(self.capability) {
            return Err(BackendQualificationMatrixDenial::MissingMediaAssumption {
                capability: self.capability,
            });
        }
        match self.support_posture {
            BackendCapabilitySupportPosture::Supported if self.residual_debt.is_clear() => {
                Ok(CertifiedBackendQualificationSupport {
                    profile: self.profile,
                    capability: self.capability,
                    evidence_class: self.evidence_class,
                    replay_identity: CanonicalDigestId::new(self.harness_proof.replay_identity()),
                })
            }
            BackendCapabilitySupportPosture::Supported => {
                Err(BackendQualificationMatrixDenial::ResidualDebtPresent {
                    capability: self.capability,
                })
            }
            BackendCapabilitySupportPosture::Unsupported
            | BackendCapabilitySupportPosture::Unavailable
            | BackendCapabilitySupportPosture::Unknown => {
                Err(BackendQualificationMatrixDenial::UnsupportedCapability {
                    capability: self.capability,
                    posture: self.support_posture,
                })
            }
            BackendCapabilitySupportPosture::Stale => {
                Err(BackendQualificationMatrixDenial::StaleRow {
                    capability: self.capability,
                })
            }
            BackendCapabilitySupportPosture::RebindRequired => {
                Err(BackendQualificationMatrixDenial::RebindRequired {
                    capability: self.capability,
                    triggers: self.rebind_triggers,
                })
            }
        }
    }

    pub fn require_certified_backend_support(
        &self,
    ) -> Result<CertifiedBackendQualificationSupport, BackendQualificationMatrixDenial> {
        self.require_support(CapabilityEvidenceClass::CertifiedBackendProfile)
    }
}

impl CertifiedBackendQualificationSupport {
    pub const fn profile(self) -> BackendTargetProfile {
        self.profile
    }

    pub const fn capability(self) -> BackendCapabilityKind {
        self.capability
    }

    pub const fn evidence_class(self) -> CapabilityEvidenceClass {
        self.evidence_class
    }

    pub const fn replay_identity(self) -> [u8; 32] {
        *self.replay_identity.bytes()
    }
}

impl BackendQualificationRowIdentity {
    pub const fn profile(self) -> BackendTargetProfile {
        self.profile
    }

    pub const fn capability(self) -> BackendCapabilityKind {
        self.capability
    }

    pub const fn evidence_class(self) -> CapabilityEvidenceClass {
        self.evidence_class
    }

    pub const fn support_posture(self) -> BackendCapabilitySupportPosture {
        self.support_posture
    }

    pub const fn residual_debt(self) -> QualificationResidualDebt {
        self.residual_debt
    }

    pub const fn harness_proof(self) -> QualificationHarnessProof {
        self.harness_proof
    }
}

const fn default_residual_debt(
    capability: BackendCapabilityKind,
    support_posture: BackendCapabilitySupportPosture,
    evidence_class: CapabilityEvidenceClass,
    rebind_triggers: BackendRebindTriggers,
) -> QualificationResidualDebt {
    match support_posture {
        BackendCapabilitySupportPosture::Supported => {
            QualificationResidualDebt::none(capability, rebind_triggers)
        }
        BackendCapabilitySupportPosture::Unsupported
        | BackendCapabilitySupportPosture::Unavailable
        | BackendCapabilitySupportPosture::Unknown => {
            QualificationResidualDebt::backend_specific_denial(
                capability,
                evidence_class,
                rebind_triggers,
            )
        }
        BackendCapabilitySupportPosture::Stale
        | BackendCapabilitySupportPosture::RebindRequired => {
            QualificationResidualDebt::stale_evidence(capability, evidence_class, rebind_triggers)
        }
    }
}

fn validate_harness_proof(
    capability: BackendCapabilityKind,
    evidence_class: CapabilityEvidenceClass,
    evidence: &IoPressureHarnessEvidence,
    harness_proof: QualificationHarnessProof,
) -> Result<(), BackendQualificationMatrixDenial> {
    if !harness_proof.covers(capability) {
        return Err(
            BackendQualificationMatrixDenial::HarnessProofCapabilityMismatch { capability },
        );
    }
    if harness_proof.backend_profile() != evidence.scenario().backend_profile()
        || harness_proof.replay_profile() != evidence.replay_profile()
        || harness_proof.replay_identity() != *evidence.replay_identity()
        || harness_proof.maturity() != evidence.maturity()
    {
        return Err(BackendQualificationMatrixDenial::HarnessProofEvidenceMismatch { capability });
    }
    if evidence_class == CapabilityEvidenceClass::CertifiedBackendProfile
        && harness_proof.strength()
            != QualificationHarnessProofStrength::ExplicitBackendQualification
    {
        return Err(
            BackendQualificationMatrixDenial::HarnessProofStrengthTooWeak {
                required: QualificationHarnessProofStrength::ExplicitBackendQualification,
                actual: harness_proof.strength(),
            },
        );
    }
    Ok(())
}

fn validate_residual_debt(
    capability: BackendCapabilityKind,
    support_posture: BackendCapabilitySupportPosture,
    residual_debt: QualificationResidualDebt,
) -> Result<(), BackendQualificationMatrixDenial> {
    if residual_debt.affected_capability() != capability {
        return Err(
            BackendQualificationMatrixDenial::ResidualDebtCapabilityMismatch {
                expected: capability,
                actual: residual_debt.affected_capability(),
            },
        );
    }
    if support_posture != BackendCapabilitySupportPosture::Supported && residual_debt.is_clear() {
        return Err(BackendQualificationMatrixDenial::MissingResidualDebt {
            capability,
            posture: support_posture,
        });
    }
    Ok(())
}
