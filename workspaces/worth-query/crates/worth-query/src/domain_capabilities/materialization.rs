use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use worth_foundational::{
    FoundationalDiagnosticMaterializationDenial, FoundationalProfileIdentityDenial,
    FoundationalProfileProgressionDenial,
};

use super::payloads::WorthQueryDomainCapabilityCategory;

pub use super::summary::*;
pub use super::support::*;
pub use super::trace::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryDomainCapabilityDescriptiveArtifactKind {
    Summary,
    SupportReport,
    TraceArtifact,
    ExplanationBundle,
}

impl WorthQueryDomainCapabilityDescriptiveArtifactKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Summary => "summary",
            Self::SupportReport => "support-report",
            Self::TraceArtifact => "trace-artifact",
            Self::ExplanationBundle => "explanation-bundle",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorthQueryDomainCapabilityDescriptiveMaterializationDenial {
    EvidenceIdentityDerivationDenied(IdentityDerivationDenial),
    ProfileAdmission {
        category: WorthQueryDomainCapabilityCategory,
        artifact_kind: WorthQueryDomainCapabilityDescriptiveArtifactKind,
        denial: FoundationalProfileProgressionDenial,
    },
    ProfileMaterialization {
        category: WorthQueryDomainCapabilityCategory,
        artifact_kind: WorthQueryDomainCapabilityDescriptiveArtifactKind,
        denial: FoundationalProfileProgressionDenial,
    },
    ProfileIdentity {
        category: WorthQueryDomainCapabilityCategory,
        artifact_kind: WorthQueryDomainCapabilityDescriptiveArtifactKind,
        denial: FoundationalProfileIdentityDenial,
    },
    SupportReport {
        category: WorthQueryDomainCapabilityCategory,
        denial: FoundationalDiagnosticMaterializationDenial,
    },
    ExplanationBundle {
        category: WorthQueryDomainCapabilityCategory,
        denial: FoundationalDiagnosticMaterializationDenial,
    },
}

pub struct WorthQueryDomainCapabilityProfileProgression {
    requested: worth_foundational::RequestedFoundationalProfileArtifact,
    admitted: worth_foundational::AdmittedFoundationalProfileArtifact,
    materialized: worth_foundational::MaterializedFoundationalProfileArtifact,
}

impl WorthQueryDomainCapabilityProfileProgression {
    pub fn new(
        requested: worth_foundational::RequestedFoundationalProfileArtifact,
        admitted: worth_foundational::AdmittedFoundationalProfileArtifact,
        materialized: worth_foundational::MaterializedFoundationalProfileArtifact,
    ) -> Self {
        Self {
            requested,
            admitted,
            materialized,
        }
    }

    pub fn requested(&self) -> &worth_foundational::RequestedFoundationalProfileArtifact {
        &self.requested
    }

    pub fn admitted(&self) -> &worth_foundational::AdmittedFoundationalProfileArtifact {
        &self.admitted
    }

    pub fn materialized(&self) -> &worth_foundational::MaterializedFoundationalProfileArtifact {
        &self.materialized
    }
}

impl From<IdentityDerivationDenial> for WorthQueryDomainCapabilityDescriptiveMaterializationDenial {
    fn from(cause: IdentityDerivationDenial) -> Self {
        Self::EvidenceIdentityDerivationDenied(cause)
    }
}
