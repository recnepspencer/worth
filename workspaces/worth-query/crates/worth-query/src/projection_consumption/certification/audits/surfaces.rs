use super::super::proof_artifacts::ProjectionConsumptionCompileFailProof;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::projection_consumption::facts::ProjectionFactKind;
use crate::projection_consumption::source::{
    ProjectionConsumptionSource, ProjectionSourceCapabilityProfile,
    ProjectionSourceExecutionPosture, ProjectionSourceFamily, ProjectionSourceReferenceIdentity,
    ProjectionWriteReceiptCapabilities,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectionConsumptionCertifiedSourceSurface {
    QueryReadCurrent,
    QueryReadHistorical,
    QueryWriteCompleteEvidence,
    QueryWriteMissingSourceReferenceEvidence,
    QueryContextCurrentWithSourceReference,
    QueryContextHistoricalWithoutSourceReference,
    QueryContextPreviewDerivedWithSourceReference,
    RelationalRowSet,
    RelationalGroupedProjection,
    BridgeTruthViewRowSet,
    BridgeGroupedTruthView,
    RetainedDerivedArtifactBinding,
    LiveArtifactBinding,
}

impl ProjectionConsumptionCertifiedSourceSurface {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::QueryReadCurrent => "query_read_current",
            Self::QueryReadHistorical => "query_read_historical",
            Self::QueryWriteCompleteEvidence => "query_write_complete_evidence",
            Self::QueryWriteMissingSourceReferenceEvidence => {
                "query_write_missing_source_reference_evidence"
            }
            Self::QueryContextCurrentWithSourceReference => {
                "query_context_current_with_source_reference"
            }
            Self::QueryContextHistoricalWithoutSourceReference => {
                "query_context_historical_without_source_reference"
            }
            Self::QueryContextPreviewDerivedWithSourceReference => {
                "query_context_preview_derived_with_source_reference"
            }
            Self::RelationalRowSet => "relational_row_set",
            Self::RelationalGroupedProjection => "relational_grouped_projection",
            Self::BridgeTruthViewRowSet => "bridge_truth_view_row_set",
            Self::BridgeGroupedTruthView => "bridge_grouped_truth_view",
            Self::RetainedDerivedArtifactBinding => "retained_derived_artifact_binding",
            Self::LiveArtifactBinding => "live_artifact_binding",
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::QueryReadCurrent,
            Self::QueryReadHistorical,
            Self::QueryWriteCompleteEvidence,
            Self::QueryWriteMissingSourceReferenceEvidence,
            Self::QueryContextCurrentWithSourceReference,
            Self::QueryContextHistoricalWithoutSourceReference,
            Self::QueryContextPreviewDerivedWithSourceReference,
            Self::RelationalRowSet,
            Self::RelationalGroupedProjection,
            Self::BridgeTruthViewRowSet,
            Self::BridgeGroupedTruthView,
            Self::RetainedDerivedArtifactBinding,
            Self::LiveArtifactBinding,
        ]
    }
}

pub fn representative_source(
    surface: ProjectionConsumptionCertifiedSourceSurface,
) -> Result<ProjectionConsumptionSource, IdentityDerivationDenial> {
    Ok(match surface {
        ProjectionConsumptionCertifiedSourceSurface::QueryReadCurrent => {
            ProjectionConsumptionSource::synthetic_for_certification(
                ProjectionSourceFamily::QueryReadReceipt,
                ProjectionSourceCapabilityProfile::QueryReadReceipt {
                    execution_posture: ProjectionSourceExecutionPosture::Current,
                },
                "certified:query_read_current",
                Vec::new(),
            )?
        }
        ProjectionConsumptionCertifiedSourceSurface::QueryReadHistorical => {
            ProjectionConsumptionSource::synthetic_for_certification(
                ProjectionSourceFamily::QueryReadReceipt,
                ProjectionSourceCapabilityProfile::QueryReadReceipt {
                    execution_posture: ProjectionSourceExecutionPosture::Historical,
                },
                "certified:query_read_historical",
                Vec::new(),
            )?
        }
        ProjectionConsumptionCertifiedSourceSurface::QueryWriteCompleteEvidence => {
            ProjectionConsumptionSource::synthetic_for_certification(
                ProjectionSourceFamily::QueryWriteReceipt,
                ProjectionSourceCapabilityProfile::QueryWriteReceipt {
                    capabilities: ProjectionWriteReceiptCapabilities::synthetic(
                        true, true, true, true,
                    ),
                },
                "certified:query_write_complete_evidence",
                vec![ProjectionSourceReferenceIdentity::synthetic(
                    "bridge_provenance_execution_record",
                    "certified:bridge_provenance",
                )],
            )?
        }
        ProjectionConsumptionCertifiedSourceSurface::QueryWriteMissingSourceReferenceEvidence => {
            ProjectionConsumptionSource::synthetic_for_certification(
                ProjectionSourceFamily::QueryWriteReceipt,
                ProjectionSourceCapabilityProfile::QueryWriteReceipt {
                    capabilities: ProjectionWriteReceiptCapabilities::synthetic(
                        true, false, true, true,
                    ),
                },
                "certified:query_write_missing_source_reference",
                Vec::new(),
            )?
        }
        ProjectionConsumptionCertifiedSourceSurface::QueryContextCurrentWithSourceReference => {
            ProjectionConsumptionSource::synthetic_for_certification(
                ProjectionSourceFamily::QueryContextExecution,
                ProjectionSourceCapabilityProfile::QueryContextExecution {
                    execution_posture: ProjectionSourceExecutionPosture::Current,
                },
                "certified:query_context_current",
                vec![ProjectionSourceReferenceIdentity::synthetic(
                    "query_context_materialization_path",
                    "certified:query_context_path",
                )],
            )?
        }
        ProjectionConsumptionCertifiedSourceSurface::QueryContextHistoricalWithoutSourceReference => {
            ProjectionConsumptionSource::synthetic_for_certification(
                ProjectionSourceFamily::QueryContextExecution,
                ProjectionSourceCapabilityProfile::QueryContextExecution {
                    execution_posture: ProjectionSourceExecutionPosture::Historical,
                },
                "certified:query_context_historical",
                Vec::new(),
            )?
        }
        ProjectionConsumptionCertifiedSourceSurface::QueryContextPreviewDerivedWithSourceReference => {
            ProjectionConsumptionSource::synthetic_for_certification(
                ProjectionSourceFamily::QueryContextExecution,
                ProjectionSourceCapabilityProfile::QueryContextExecution {
                    execution_posture: ProjectionSourceExecutionPosture::PreviewDerived,
                },
                "certified:query_context_preview",
                vec![ProjectionSourceReferenceIdentity::synthetic(
                    "query_context_preview_provenance",
                    "certified:query_context_preview_provenance",
                )],
            )?
        }
        ProjectionConsumptionCertifiedSourceSurface::RelationalRowSet => {
            ProjectionConsumptionSource::synthetic_for_certification(
                ProjectionSourceFamily::RelationalRowSet,
                ProjectionSourceCapabilityProfile::RelationalRowSet,
                "certified:relational_row_set",
                Vec::new(),
            )?
        }
        ProjectionConsumptionCertifiedSourceSurface::RelationalGroupedProjection => {
            ProjectionConsumptionSource::synthetic_for_certification(
                ProjectionSourceFamily::RelationalGroupedProjection,
                ProjectionSourceCapabilityProfile::RelationalGroupedProjection,
                "certified:relational_grouped_projection",
                Vec::new(),
            )?
        }
        ProjectionConsumptionCertifiedSourceSurface::BridgeTruthViewRowSet => {
            ProjectionConsumptionSource::synthetic_for_certification(
                ProjectionSourceFamily::BridgeTruthViewRowSet,
                ProjectionSourceCapabilityProfile::BridgeTruthViewRowSet,
                "certified:bridge_truth_view_row_set",
                Vec::new(),
            )?
        }
        ProjectionConsumptionCertifiedSourceSurface::BridgeGroupedTruthView => {
            ProjectionConsumptionSource::synthetic_for_certification(
                ProjectionSourceFamily::BridgeGroupedTruthView,
                ProjectionSourceCapabilityProfile::BridgeGroupedTruthView,
                "certified:bridge_grouped_truth_view",
                Vec::new(),
            )?
        }
        ProjectionConsumptionCertifiedSourceSurface::RetainedDerivedArtifactBinding => {
            ProjectionConsumptionSource::synthetic_for_certification(
                ProjectionSourceFamily::RetainedDerivedArtifactBinding,
                ProjectionSourceCapabilityProfile::RetainedDerivedArtifactBinding,
                "certified:retained_derived_artifact_binding",
                vec![
                    ProjectionSourceReferenceIdentity::synthetic(
                        "retained_target_view",
                        "certified:retained_target:first",
                    ),
                    ProjectionSourceReferenceIdentity::synthetic(
                        "retained_target_view",
                        "certified:retained_target:second",
                    ),
                ],
            )?
        }
        ProjectionConsumptionCertifiedSourceSurface::LiveArtifactBinding => {
            ProjectionConsumptionSource::synthetic_for_certification(
                ProjectionSourceFamily::LiveArtifactBinding,
                ProjectionSourceCapabilityProfile::LiveArtifactBinding,
                "certified:live_artifact_binding",
                vec![
                    ProjectionSourceReferenceIdentity::synthetic(
                        "live_target_view",
                        "certified:live_target:first",
                    ),
                    ProjectionSourceReferenceIdentity::synthetic(
                        "live_target_view",
                        "certified:live_target:second",
                    ),
                ],
            )?
        }
    })
}

pub fn traceability_for(
    surface: ProjectionConsumptionCertifiedSourceSurface,
    fact_kind: ProjectionFactKind,
) -> (
    &'static str,
    &'static str,
    &'static str,
    ProjectionConsumptionCompileFailProof,
) {
    let lane = match surface {
        ProjectionConsumptionCertifiedSourceSurface::QueryReadCurrent
        | ProjectionConsumptionCertifiedSourceSurface::QueryReadHistorical => "read_backed_consumption",
        ProjectionConsumptionCertifiedSourceSurface::QueryWriteCompleteEvidence
        | ProjectionConsumptionCertifiedSourceSurface::QueryWriteMissingSourceReferenceEvidence => {
            "effect_backed_consumption"
        }
        ProjectionConsumptionCertifiedSourceSurface::QueryContextCurrentWithSourceReference
        | ProjectionConsumptionCertifiedSourceSurface::QueryContextHistoricalWithoutSourceReference
        | ProjectionConsumptionCertifiedSourceSurface::QueryContextPreviewDerivedWithSourceReference => {
            "query_context_consumption"
        }
        ProjectionConsumptionCertifiedSourceSurface::RelationalRowSet
        | ProjectionConsumptionCertifiedSourceSurface::BridgeTruthViewRowSet => "row_set_consumption",
        ProjectionConsumptionCertifiedSourceSurface::RelationalGroupedProjection
        | ProjectionConsumptionCertifiedSourceSurface::BridgeGroupedTruthView => {
            "grouped_consumption"
        }
        ProjectionConsumptionCertifiedSourceSurface::RetainedDerivedArtifactBinding
        | ProjectionConsumptionCertifiedSourceSurface::LiveArtifactBinding => {
            "retained_live_artifact_consumption"
        }
    };
    let hostile = match fact_kind {
        ProjectionFactKind::EntityIdentity | ProjectionFactKind::ViewLocalIdentity => {
            "identity_visibility_and_shape_guard"
        }
        ProjectionFactKind::TargetIdentity | ProjectionFactKind::SourceReference => {
            "writeback_evidence_gating"
        }
        ProjectionFactKind::EffectContinuity | ProjectionFactKind::RelationEndpoint => {
            "aftermath_and_grouped_endpoint_guard"
        }
        ProjectionFactKind::Membership => "grouped_membership_guard",
        ProjectionFactKind::DisplayField | ProjectionFactKind::DerivedField => {
            "row_or_visibility_guard"
        }
    };
    let rule = match (surface, fact_kind) {
        (
            ProjectionConsumptionCertifiedSourceSurface::QueryWriteMissingSourceReferenceEvidence,
            ProjectionFactKind::SourceReference,
        ) => "query_write_support_missing_source_reference_is_deferred",
        (
            ProjectionConsumptionCertifiedSourceSurface::QueryContextHistoricalWithoutSourceReference,
            ProjectionFactKind::EntityIdentity,
        )
        | (
            ProjectionConsumptionCertifiedSourceSurface::QueryContextHistoricalWithoutSourceReference,
            ProjectionFactKind::ViewLocalIdentity,
        ) => "query_context_historical_identity_is_source_mismatch",
        (
            ProjectionConsumptionCertifiedSourceSurface::RetainedDerivedArtifactBinding,
            ProjectionFactKind::SourceReference
            | ProjectionFactKind::ViewLocalIdentity
            | ProjectionFactKind::DisplayField
            | ProjectionFactKind::DerivedField,
        ) => "retained_binding_admits_field_view_local_and_source_reference_facts",
        (
            ProjectionConsumptionCertifiedSourceSurface::LiveArtifactBinding,
            ProjectionFactKind::EntityIdentity
            | ProjectionFactKind::ViewLocalIdentity
            | ProjectionFactKind::SourceReference
            | ProjectionFactKind::DisplayField
            | ProjectionFactKind::DerivedField,
        ) => "live_binding_admits_identity_field_view_local_and_source_reference_facts",
        (_, ProjectionFactKind::Membership) => "grouped_projection_membership_only",
        (_, ProjectionFactKind::RelationEndpoint) => "grouped_or_writeback_endpoint_only",
        (_, ProjectionFactKind::DisplayField) | (_, ProjectionFactKind::DerivedField) => {
            "field_fact_support_follows_visibility_or_row_posture"
        }
        _ => "support_row_derived_from_executable_support_for_kind",
    };
    let proof = match fact_kind {
        ProjectionFactKind::TargetIdentity
        | ProjectionFactKind::SourceReference
        | ProjectionFactKind::EffectContinuity
        | ProjectionFactKind::RelationEndpoint => {
            ProjectionConsumptionCompileFailProof::ContractHasNoGenericExtract
        }
        ProjectionFactKind::EntityIdentity
        | ProjectionFactKind::ViewLocalIdentity
        | ProjectionFactKind::Membership
        | ProjectionFactKind::DisplayField
        | ProjectionFactKind::DerivedField => {
            ProjectionConsumptionCompileFailProof::RawSourceHasNoConsumedFactAccessors
        }
    };
    (rule, hostile, lane, proof)
}
