use super::{
    certification_basis_identity, ProjectionConsumptionSource, ProjectionSourceBasisAuthority,
    ProjectionSourceCapabilityProfile, ProjectionSourceExecutionPosture, ProjectionSourceFamily,
    ProjectionSourceReferenceIdentity, ProjectionWriteReceiptCapabilities,
};

impl ProjectionConsumptionSource {
    pub(crate) fn test_only(
        family: ProjectionSourceFamily,
        query_digest: Option<&str>,
        basis_digest: Option<&str>,
        result_digest: Option<&str>,
        result_shape_digest: Option<&str>,
        source_identity: &str,
    ) -> Self {
        Self::test_only_with_profile(
            family,
            default_capability_profile(family),
            query_digest,
            basis_digest,
            result_digest,
            result_shape_digest,
            source_identity,
        )
    }

    pub(crate) fn test_only_with_profile(
        family: ProjectionSourceFamily,
        capability_profile: ProjectionSourceCapabilityProfile,
        query_digest: Option<&str>,
        basis_digest: Option<&str>,
        result_digest: Option<&str>,
        result_shape_digest: Option<&str>,
        source_identity: &str,
    ) -> Self {
        Self::test_only_with_source_references(
            family,
            capability_profile,
            query_digest,
            basis_digest,
            result_digest,
            result_shape_digest,
            source_identity,
            Vec::new(),
        )
    }

    pub(crate) fn test_only_with_source_references(
        family: ProjectionSourceFamily,
        capability_profile: ProjectionSourceCapabilityProfile,
        query_digest: Option<&str>,
        basis_digest: Option<&str>,
        result_digest: Option<&str>,
        result_shape_digest: Option<&str>,
        source_identity: &str,
        source_reference_identities: Vec<ProjectionSourceReferenceIdentity>,
    ) -> Self {
        Self {
            family,
            capability_profile,
            query_digest: query_digest.map(str::to_string),
            basis_digest: basis_digest.map(str::to_string),
            basis_authority: ProjectionSourceBasisAuthority::certification(
                certification_basis_identity("projection-source-test-basis")
                    .expect("fixture digest must fit"),
            ),
            result_digest: result_digest.map(str::to_string),
            result_shape_digest: result_shape_digest.map(str::to_string),
            source_identity: source_identity.to_string().into(),
            source_reference_identities,
            materialized_fact_posture: None,
        }
    }
}

fn default_capability_profile(family: ProjectionSourceFamily) -> ProjectionSourceCapabilityProfile {
    match family {
        ProjectionSourceFamily::QueryReadReceipt => {
            ProjectionSourceCapabilityProfile::QueryReadReceipt {
                execution_posture: ProjectionSourceExecutionPosture::Current,
            }
        }
        ProjectionSourceFamily::QueryLiveReadReceipt => {
            ProjectionSourceCapabilityProfile::QueryLiveReadReceipt {
                execution_posture: ProjectionSourceExecutionPosture::Current,
            }
        }
        ProjectionSourceFamily::QueryWriteReceipt => {
            ProjectionSourceCapabilityProfile::QueryWriteReceipt {
                capabilities: ProjectionWriteReceiptCapabilities::default(),
            }
        }
        ProjectionSourceFamily::QueryContextExecution => {
            ProjectionSourceCapabilityProfile::QueryContextExecution {
                execution_posture: ProjectionSourceExecutionPosture::Current,
            }
        }
        ProjectionSourceFamily::RelationalRowSet => {
            ProjectionSourceCapabilityProfile::RelationalRowSet
        }
        ProjectionSourceFamily::RelationalGroupedProjection => {
            ProjectionSourceCapabilityProfile::RelationalGroupedProjection
        }
        ProjectionSourceFamily::BridgeTruthViewRowSet => {
            ProjectionSourceCapabilityProfile::BridgeTruthViewRowSet
        }
        ProjectionSourceFamily::BridgeGroupedTruthView => {
            ProjectionSourceCapabilityProfile::BridgeGroupedTruthView
        }
        ProjectionSourceFamily::RetainedDerivedArtifactBinding => {
            ProjectionSourceCapabilityProfile::RetainedDerivedArtifactBinding
        }
        ProjectionSourceFamily::LiveArtifactBinding => {
            ProjectionSourceCapabilityProfile::LiveArtifactBinding
        }
    }
}
