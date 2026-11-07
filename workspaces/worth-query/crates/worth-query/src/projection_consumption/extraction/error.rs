use super::super::{ProjectionFactFieldPath, ProjectionFactKind, ProjectionSourceFamily};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use worth_foundational::facade::AspectValuePosture;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProjectionFactExtractionError {
    EvidenceIdentityDerivationDenied(IdentityDerivationDenial),
    ContractSourceFamilyMismatch {
        contract_family: ProjectionSourceFamily,
        extractor_family: ProjectionSourceFamily,
    },
    SourceIdentityMismatch {
        contract_source_identity: String,
        provided_source_identity: String,
    },
    SourceArtifactMetadataMismatch {
        metadata_label: &'static str,
        contract_value: String,
        provided_value: String,
    },
    MissingDeclaredFieldEvidence {
        source_family: ProjectionSourceFamily,
        source_identity: String,
        field_key: String,
        fact_kind: ProjectionFactKind,
    },
    InvalidDeclaredFieldValueShape {
        source_family: ProjectionSourceFamily,
        source_identity: String,
        field_key: String,
        fact_kind: ProjectionFactKind,
        expected_shape: &'static str,
    },
    MissingRequiredNativeFact {
        source_family: ProjectionSourceFamily,
        source_identity: String,
        field_path: ProjectionFactFieldPath,
        contract_key: worth_foundational::facade::AspectKey,
        contract_revision: worth_foundational::facade::AspectContractRevision,
        projection_authority: String,
    },
    NativeContractValueShapeMismatch {
        source_family: ProjectionSourceFamily,
        source_identity: String,
        field_path: ProjectionFactFieldPath,
        contract_key: worth_foundational::facade::AspectKey,
        contract_revision: worth_foundational::facade::AspectContractRevision,
        expected: AspectValuePosture,
        actual: AspectValuePosture,
        projection_authority: String,
    },
    NativeContractValueValidationDenied {
        source_family: ProjectionSourceFamily,
        source_identity: String,
        field_path: ProjectionFactFieldPath,
        contract_key: worth_foundational::facade::AspectKey,
        contract_revision: worth_foundational::facade::AspectContractRevision,
        denial: worth_foundational::facade::ContractValidationDenial,
        projection_authority: String,
    },
    MissingWriteReceiptEvidence {
        source_identity: String,
        fact_kind: ProjectionFactKind,
    },
    SourceReferenceEvidenceMismatch {
        expected_count: usize,
        actual_count: usize,
    },
}

impl From<IdentityDerivationDenial> for ProjectionFactExtractionError {
    fn from(cause: IdentityDerivationDenial) -> Self {
        Self::EvidenceIdentityDerivationDenied(cause)
    }
}
