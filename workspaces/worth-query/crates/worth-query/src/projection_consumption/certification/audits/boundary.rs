use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::projection_consumption::identity::{
    compose_certification_row_digest, compose_digest_sequence,
};

use super::super::proof_artifacts::ProjectionConsumptionCompileFailProof;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectionConsumptionPublicBoundarySurface {
    DeclarationMinting,
    ContractMinting,
    FactSetMinting,
    ReceiptMinting,
    EnvelopeMinting,
    CertificationMinting,
    RawRowConsumptionShortcut,
    GenericExtractionShortcut,
    NonAdmittedPromotion,
}

impl ProjectionConsumptionPublicBoundarySurface {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DeclarationMinting => "declaration_minting",
            Self::ContractMinting => "contract_minting",
            Self::FactSetMinting => "fact_set_minting",
            Self::ReceiptMinting => "receipt_minting",
            Self::EnvelopeMinting => "envelope_minting",
            Self::CertificationMinting => "certification_minting",
            Self::RawRowConsumptionShortcut => "raw_row_consumption_shortcut",
            Self::GenericExtractionShortcut => "generic_extraction_shortcut",
            Self::NonAdmittedPromotion => "non_admitted_promotion",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectionConsumptionPublicBoundaryAuditRow {
    surface: ProjectionConsumptionPublicBoundarySurface,
    blocked_entrypoint: &'static str,
    required_artifact: &'static str,
    enforcement_proof: ProjectionConsumptionCompileFailProof,
    row_digest: String,
}

impl ProjectionConsumptionPublicBoundaryAuditRow {
    pub fn surface(&self) -> ProjectionConsumptionPublicBoundarySurface {
        self.surface
    }

    pub fn blocked_entrypoint(&self) -> &'static str {
        self.blocked_entrypoint
    }

    pub fn required_artifact(&self) -> &'static str {
        self.required_artifact
    }

    pub fn enforcement_proof(&self) -> ProjectionConsumptionCompileFailProof {
        self.enforcement_proof
    }

    pub fn row_digest(&self) -> &str {
        &self.row_digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectionConsumptionPublicBoundaryAudit {
    rows: Vec<ProjectionConsumptionPublicBoundaryAuditRow>,
    audit_digest: String,
}

impl ProjectionConsumptionPublicBoundaryAudit {
    pub fn rows(&self) -> &[ProjectionConsumptionPublicBoundaryAuditRow] {
        &self.rows
    }

    pub fn audit_digest(&self) -> &str {
        &self.audit_digest
    }
}

pub fn projection_consumption_public_boundary_audit(
) -> Result<ProjectionConsumptionPublicBoundaryAudit, IdentityDerivationDenial> {
    use ProjectionConsumptionPublicBoundarySurface::*;
    let rows = vec![
        row(
            DeclarationMinting,
            "ProjectionConsumptionDeclaration construction",
            "Query-owned authoring surface",
            ProjectionConsumptionCompileFailProof::DeclarationConstructorPrivate,
        ),
        row(
            ContractMinting,
            "MaterializedProjectionContract construction",
            "AdmittedProjectionConsumption::bind_contract()",
            ProjectionConsumptionCompileFailProof::ContractConstructorPrivate,
        ),
        row(
            FactSetMinting,
            "ConsumedProjectionFactSet construction",
            "MaterializedProjectionContract::extract_from_*",
            ProjectionConsumptionCompileFailProof::FactSetConstructorPrivate,
        ),
        row(
            ReceiptMinting,
            "ProjectionConsumptionReceipt construction",
            "ConsumedProjectionFactSet::issue_receipt()",
            ProjectionConsumptionCompileFailProof::ReceiptConstructorPrivate,
        ),
        row(
            EnvelopeMinting,
            "SelfDescribingProjectionConsumptionEnvelope construction",
            "ProjectionConsumptionReceipt::projection_consumption_envelope()",
            ProjectionConsumptionCompileFailProof::EnvelopeConstructorPrivate,
        ),
        row(
            CertificationMinting,
            "ProjectionConsumptionCertificationBundle construction",
            "certify_projection_consumption_closeout_core()",
            ProjectionConsumptionCompileFailProof::CertificationBundleConstructorPrivate,
        ),
        row(
            RawRowConsumptionShortcut,
            "raw row-like materialization promoted to consumed facts",
            "admitted contract then typed extraction",
            ProjectionConsumptionCompileFailProof::RawSourceHasNoConsumedFactAccessors,
        ),
        row(
            GenericExtractionShortcut,
            "generic extract() bypass over source-specific extraction",
            "explicit source-aware extract_from_* methods",
            ProjectionConsumptionCompileFailProof::ContractHasNoGenericExtract,
        ),
        row(
            NonAdmittedPromotion,
            "denied/deferred/source-mismatch promoted to contract binding",
            "AdmittedProjectionConsumption only",
            ProjectionConsumptionCompileFailProof::NonAdmittedCannotBindContract,
        ),
    ];
    let audit_digest = compose_digest_sequence(
        "projection_consumption_public_boundary_audit_v1",
        "row",
        rows.iter().map(|row| row.row_digest().to_string()),
    )?;
    Ok(ProjectionConsumptionPublicBoundaryAudit { rows, audit_digest })
}

fn row(
    surface: ProjectionConsumptionPublicBoundarySurface,
    blocked_entrypoint: &'static str,
    required_artifact: &'static str,
    enforcement_proof: ProjectionConsumptionCompileFailProof,
) -> Result<ProjectionConsumptionPublicBoundaryAuditRow, IdentityDerivationDenial> {
    let row_digest = compose_certification_row_digest(
        "projection_consumption_public_boundary_row_v1",
        &[
            ("surface", surface.as_str()),
            ("entrypoint", blocked_entrypoint),
            ("required", required_artifact),
            ("proof", enforcement_proof.as_str()),
        ],
    )?;
    Ok(ProjectionConsumptionPublicBoundaryAuditRow {
        surface,
        blocked_entrypoint,
        required_artifact,
        enforcement_proof,
        row_digest,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_boundary_audit_names_all_projection_shortcuts() -> Result<(), IdentityDerivationDenial>
    {
        let audit = projection_consumption_public_boundary_audit()?;
        assert_eq!(audit.rows().len(), 9);
        assert!(audit.rows().iter().all(|row| !row.row_digest().is_empty()));
        assert!(!audit.audit_digest().is_empty());

        Ok(())
    }
}
