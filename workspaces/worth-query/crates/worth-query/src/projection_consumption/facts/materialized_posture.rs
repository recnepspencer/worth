use super::super::identity::compose_materialized_fact_posture_digest;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectionMaterializedFactPostureKind {
    Ordinary,
    TimeOnly,
    AsyncBacked,
    MixedCause,
    Remasked,
}

impl ProjectionMaterializedFactPostureKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ordinary => "ordinary",
            Self::TimeOnly => "time_only",
            Self::AsyncBacked => "async_backed",
            Self::MixedCause => "mixed_cause",
            Self::Remasked => "remasked",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectionMaterializedFactPosture {
    kind: ProjectionMaterializedFactPostureKind,
    lower_declaration_digest: String,
    basis_digest: String,
    support_evidence_digest: String,
    runtime_origin_digest: Option<String>,
    posture_digest: String,
}

impl ProjectionMaterializedFactPosture {
    pub fn new(
        kind: ProjectionMaterializedFactPostureKind,
        lower_declaration_digest: impl Into<String>,
        basis_digest: impl Into<String>,
        support_evidence_digest: impl Into<String>,
        runtime_origin_digest: Option<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        let lower_declaration_digest = lower_declaration_digest.into();
        let basis_digest = basis_digest.into();
        let support_evidence_digest = support_evidence_digest.into();
        let posture_digest = compose_materialized_fact_posture_digest(
            kind,
            &lower_declaration_digest,
            &basis_digest,
            &support_evidence_digest,
            runtime_origin_digest.as_deref(),
        )?;
        Ok(Self {
            kind,
            lower_declaration_digest,
            basis_digest,
            support_evidence_digest,
            runtime_origin_digest,
            posture_digest,
        })
    }

    pub fn kind(&self) -> ProjectionMaterializedFactPostureKind {
        self.kind
    }

    pub fn lower_declaration_digest(&self) -> &str {
        &self.lower_declaration_digest
    }

    pub fn basis_digest(&self) -> &str {
        &self.basis_digest
    }

    pub fn support_evidence_digest(&self) -> &str {
        &self.support_evidence_digest
    }

    pub fn runtime_origin_digest(&self) -> Option<&str> {
        self.runtime_origin_digest.as_deref()
    }

    pub fn posture_digest(&self) -> &str {
        &self.posture_digest
    }
}
