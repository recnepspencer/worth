use crate::schema::data::{
    DescriptorCanonicalBasisVersion, DescriptorSemanticsVersion, SchemaBoundaryFingerprint,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchemaTransitionValidationError {
    EmptyDiff,
    UnstratifiedChange {
        element_name: crate::schema::data::SchemaChangeText,
    },
    NarrowingWithoutPolicy {
        element_name: crate::schema::data::SchemaChangeText,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchemaContinuityBundleIssue {
    IncompleteBundle,
    ContinuationDescriptorDrift {
        boundary_fingerprint: Option<SchemaBoundaryFingerprint>,
    },
    ReconciliationDescriptorDrift,
    ContinuationBoundaryFingerprintMismatch {
        boundary_fingerprint: SchemaBoundaryFingerprint,
    },
    DescriptorSemanticsVersionMismatch {
        expected: DescriptorSemanticsVersion,
        found: DescriptorSemanticsVersion,
    },
    DescriptorCanonicalBasisVersionMismatch {
        expected: DescriptorCanonicalBasisVersion,
        found: DescriptorCanonicalBasisVersion,
    },
    VisibleBridgeProofMismatch,
    TargetSchemaVersionMismatch,
    LineageSchemaVersionMismatch,
    HistoricalReinterpretationViolation,
}

impl SchemaContinuityBundleIssue {
    pub fn detail(&self) -> String {
        match self {
            Self::IncompleteBundle => {
                "schema transition, continuation descriptor, and reconciliation descriptor must appear together".to_string()
            }
            Self::ContinuationDescriptorDrift { .. } => {
                "top-level continuation descriptor does not match schema transition artifact"
                    .to_string()
            }
            Self::ReconciliationDescriptorDrift => {
                "top-level reconciliation descriptor does not match schema transition artifact"
                    .to_string()
            }
            Self::ContinuationBoundaryFingerprintMismatch { .. } => {
                "continuation descriptor boundary fingerprint must match bridge boundary fingerprint"
                    .to_string()
            }
            Self::DescriptorSemanticsVersionMismatch { .. } => {
                "descriptor semantics version must agree across envelope, continuation descriptor, and reconciliation descriptor".to_string()
            }
            Self::DescriptorCanonicalBasisVersionMismatch { .. } => {
                "descriptor canonical basis version must agree across continuation and reconciliation descriptors and remain supported by runtime policy".to_string()
            }
            Self::VisibleBridgeProofMismatch => {
                "visible bridge continuity requires explicit proof that surfaced boundary metadata is semantically ignorable".to_string()
            }
            Self::TargetSchemaVersionMismatch => {
                "transition target schema version does not match canonical envelope schema version"
                    .to_string()
            }
            Self::LineageSchemaVersionMismatch => {
                "reconciliation lineage target schema version does not match canonical envelope schema version"
                    .to_string()
            }
            Self::HistoricalReinterpretationViolation => {
                "historically sensitive boundaries may not publish as unchanged or transparently bridgeable continuity".to_string()
            }
        }
    }
}

impl SchemaTransitionValidationError {
    pub fn detail(&self) -> String {
        match self {
            Self::EmptyDiff => {
                "schema transition must carry at least one classified diff atom".to_string()
            }
            Self::UnstratifiedChange { element_name } => {
                format!("schema change for '{element_name}' does not declare any schema strata")
            }
            Self::NarrowingWithoutPolicy { element_name } => {
                format!(
                    "schema narrowing for '{element_name}' requires an explicit preservation policy"
                )
            }
        }
    }
}
