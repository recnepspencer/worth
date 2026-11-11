use super::eligibility::{DeferredProjectionConsumptionReason, ProjectionConsumptionWarningKind};
use super::facts::ProjectionFactKind;
use super::identity::compose_support_row_digest;
use super::source::{
    ProjectionConsumptionSource, ProjectionSourceCapabilityProfile,
    ProjectionSourceExecutionPosture, ProjectionSourceFamily,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProjectionConsumptionSupportPosture {
    Admitted,
    AdmittedWithWarnings(ProjectionConsumptionWarningKind),
    Deferred(DeferredProjectionConsumptionReason),
    SourceMismatch,
}

impl ProjectionConsumptionSupportPosture {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Admitted => "admitted",
            Self::AdmittedWithWarnings(_) => "admitted_with_warnings",
            Self::Deferred(_) => "deferred",
            Self::SourceMismatch => "source_mismatch",
        }
    }

    fn detail_key(&self) -> &'static str {
        match self {
            Self::Admitted => "admitted",
            Self::AdmittedWithWarnings(kind) => kind.as_str(),
            Self::Deferred(reason) => match reason {
                DeferredProjectionConsumptionReason::WriteReceiptContractBindingPending => {
                    "write_receipt_contract_binding_pending"
                }
                DeferredProjectionConsumptionReason::SourceFamilySupportPending => {
                    "source_family_support_pending"
                }
            },
            Self::SourceMismatch => "source_mismatch",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectionConsumptionSupportRow {
    source_family: ProjectionSourceFamily,
    fact_kind: ProjectionFactKind,
    posture: ProjectionConsumptionSupportPosture,
    support_digest: String,
}

impl ProjectionConsumptionSupportRow {
    pub fn source_family(&self) -> ProjectionSourceFamily {
        self.source_family
    }

    pub fn fact_kind(&self) -> ProjectionFactKind {
        self.fact_kind
    }

    pub fn posture(&self) -> &ProjectionConsumptionSupportPosture {
        &self.posture
    }

    pub fn support_digest(&self) -> &str {
        &self.support_digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectionConsumptionSupportReport {
    rows: Vec<ProjectionConsumptionSupportRow>,
}

impl ProjectionConsumptionSupportReport {
    pub fn source_family(&self) -> ProjectionSourceFamily {
        self.rows
            .first()
            .map(ProjectionConsumptionSupportRow::source_family)
            .expect("support report always contains one row per fact kind")
    }

    pub fn rows(&self) -> &[ProjectionConsumptionSupportRow] {
        &self.rows
    }
}

pub fn discover_projection_consumption_support(
    source: &ProjectionConsumptionSource,
) -> Result<ProjectionConsumptionSupportReport, IdentityDerivationDenial> {
    let rows = ProjectionFactKind::all()
        .iter()
        .copied()
        .map(|fact_kind| {
            let posture = support_for_kind(source, fact_kind);
            Ok(ProjectionConsumptionSupportRow {
                source_family: source.family(),
                fact_kind,
                support_digest: compose_support_row_digest(
                    source.family(),
                    fact_kind,
                    &posture,
                    posture.detail_key(),
                )?,
                posture,
            })
        })
        .collect::<Result<Vec<_>, IdentityDerivationDenial>>()?;
    Ok(ProjectionConsumptionSupportReport { rows })
}

pub(crate) fn support_for_kind(
    source: &ProjectionConsumptionSource,
    fact_kind: ProjectionFactKind,
) -> ProjectionConsumptionSupportPosture {
    match source.capability_profile() {
        ProjectionSourceCapabilityProfile::QueryReadReceipt { execution_posture } => {
            query_read_support(*execution_posture, fact_kind)
        }
        ProjectionSourceCapabilityProfile::QueryLiveReadReceipt { execution_posture } => {
            query_read_support(*execution_posture, fact_kind)
        }
        ProjectionSourceCapabilityProfile::QueryWriteReceipt { capabilities } => {
            query_write_support(capabilities, fact_kind)
        }
        ProjectionSourceCapabilityProfile::QueryContextExecution { execution_posture } => {
            query_context_support(
                *execution_posture,
                !source.source_reference_identities().is_empty(),
                fact_kind,
            )
        }
        ProjectionSourceCapabilityProfile::RelationalRowSet => {
            relational_row_set_support(fact_kind)
        }
        ProjectionSourceCapabilityProfile::RelationalGroupedProjection => {
            relational_grouped_projection_support(fact_kind)
        }
        ProjectionSourceCapabilityProfile::BridgeTruthViewRowSet => {
            bridge_truth_view_row_set_support(fact_kind)
        }
        ProjectionSourceCapabilityProfile::BridgeGroupedTruthView => {
            bridge_grouped_truth_view_support(fact_kind)
        }
        ProjectionSourceCapabilityProfile::RetainedDerivedArtifactBinding => {
            retained_derived_artifact_binding_support(fact_kind)
        }
        ProjectionSourceCapabilityProfile::LiveArtifactBinding => {
            live_artifact_binding_support(fact_kind)
        }
    }
}

fn query_read_support(
    execution_posture: ProjectionSourceExecutionPosture,
    fact_kind: ProjectionFactKind,
) -> ProjectionConsumptionSupportPosture {
    match fact_kind {
        ProjectionFactKind::EntityIdentity | ProjectionFactKind::ViewLocalIdentity => {
            ProjectionConsumptionSupportPosture::Admitted
        }
        ProjectionFactKind::DisplayField | ProjectionFactKind::DerivedField => {
            match execution_posture {
                ProjectionSourceExecutionPosture::Current
                | ProjectionSourceExecutionPosture::Branch => {
                    ProjectionConsumptionSupportPosture::Admitted
                }
                ProjectionSourceExecutionPosture::Historical
                | ProjectionSourceExecutionPosture::PreviewDerived => {
                    ProjectionConsumptionSupportPosture::SourceMismatch
                }
            }
        }
        ProjectionFactKind::TargetIdentity
        | ProjectionFactKind::SourceReference
        | ProjectionFactKind::EffectContinuity
        | ProjectionFactKind::Membership
        | ProjectionFactKind::RelationEndpoint => {
            ProjectionConsumptionSupportPosture::SourceMismatch
        }
    }
}

fn query_write_support(
    capabilities: &super::source::ProjectionWriteReceiptCapabilities,
    fact_kind: ProjectionFactKind,
) -> ProjectionConsumptionSupportPosture {
    match fact_kind {
        ProjectionFactKind::TargetIdentity if capabilities.has_target_identity() => {
            ProjectionConsumptionSupportPosture::Admitted
        }
        ProjectionFactKind::SourceReference if capabilities.has_source_reference() => {
            ProjectionConsumptionSupportPosture::Admitted
        }
        ProjectionFactKind::EffectContinuity if capabilities.has_effect_continuity() => {
            ProjectionConsumptionSupportPosture::Admitted
        }
        ProjectionFactKind::RelationEndpoint if capabilities.has_relation_endpoint() => {
            ProjectionConsumptionSupportPosture::Admitted
        }
        ProjectionFactKind::TargetIdentity
        | ProjectionFactKind::SourceReference
        | ProjectionFactKind::EffectContinuity
        | ProjectionFactKind::RelationEndpoint => ProjectionConsumptionSupportPosture::Deferred(
            DeferredProjectionConsumptionReason::WriteReceiptContractBindingPending,
        ),
        ProjectionFactKind::EntityIdentity
        | ProjectionFactKind::ViewLocalIdentity
        | ProjectionFactKind::Membership
        | ProjectionFactKind::DisplayField
        | ProjectionFactKind::DerivedField => ProjectionConsumptionSupportPosture::SourceMismatch,
    }
}

fn query_context_support(
    execution_posture: ProjectionSourceExecutionPosture,
    has_source_reference: bool,
    fact_kind: ProjectionFactKind,
) -> ProjectionConsumptionSupportPosture {
    match fact_kind {
        ProjectionFactKind::SourceReference if has_source_reference => {
            ProjectionConsumptionSupportPosture::AdmittedWithWarnings(query_context_row_warning(
                execution_posture,
            ))
        }
        ProjectionFactKind::DisplayField | ProjectionFactKind::DerivedField => {
            ProjectionConsumptionSupportPosture::AdmittedWithWarnings(query_context_row_warning(
                execution_posture,
            ))
        }
        ProjectionFactKind::EntityIdentity | ProjectionFactKind::ViewLocalIdentity => {
            match execution_posture {
                ProjectionSourceExecutionPosture::Current
                | ProjectionSourceExecutionPosture::Branch => {
                    ProjectionConsumptionSupportPosture::Admitted
                }
                ProjectionSourceExecutionPosture::Historical
                | ProjectionSourceExecutionPosture::PreviewDerived => {
                    ProjectionConsumptionSupportPosture::SourceMismatch
                }
            }
        }
        ProjectionFactKind::TargetIdentity
        | ProjectionFactKind::SourceReference
        | ProjectionFactKind::EffectContinuity
        | ProjectionFactKind::Membership
        | ProjectionFactKind::RelationEndpoint => {
            ProjectionConsumptionSupportPosture::SourceMismatch
        }
    }
}

fn query_context_row_warning(
    execution_posture: ProjectionSourceExecutionPosture,
) -> ProjectionConsumptionWarningKind {
    match execution_posture {
        ProjectionSourceExecutionPosture::PreviewDerived => {
            ProjectionConsumptionWarningKind::PreviewDerivedContext
        }
        ProjectionSourceExecutionPosture::Current
        | ProjectionSourceExecutionPosture::Branch
        | ProjectionSourceExecutionPosture::Historical => {
            ProjectionConsumptionWarningKind::QueryContextRowBound
        }
    }
}

fn relational_row_set_support(
    fact_kind: ProjectionFactKind,
) -> ProjectionConsumptionSupportPosture {
    match fact_kind {
        ProjectionFactKind::EntityIdentity
        | ProjectionFactKind::ViewLocalIdentity
        | ProjectionFactKind::DisplayField
        | ProjectionFactKind::DerivedField => ProjectionConsumptionSupportPosture::Admitted,
        ProjectionFactKind::TargetIdentity
        | ProjectionFactKind::SourceReference
        | ProjectionFactKind::EffectContinuity
        | ProjectionFactKind::Membership
        | ProjectionFactKind::RelationEndpoint => {
            ProjectionConsumptionSupportPosture::SourceMismatch
        }
    }
}

fn relational_grouped_projection_support(
    fact_kind: ProjectionFactKind,
) -> ProjectionConsumptionSupportPosture {
    match fact_kind {
        ProjectionFactKind::ViewLocalIdentity
        | ProjectionFactKind::Membership
        | ProjectionFactKind::RelationEndpoint => ProjectionConsumptionSupportPosture::Admitted,
        ProjectionFactKind::EntityIdentity
        | ProjectionFactKind::TargetIdentity
        | ProjectionFactKind::SourceReference
        | ProjectionFactKind::EffectContinuity
        | ProjectionFactKind::DisplayField
        | ProjectionFactKind::DerivedField => ProjectionConsumptionSupportPosture::SourceMismatch,
    }
}

fn bridge_truth_view_row_set_support(
    fact_kind: ProjectionFactKind,
) -> ProjectionConsumptionSupportPosture {
    match fact_kind {
        ProjectionFactKind::EntityIdentity
        | ProjectionFactKind::ViewLocalIdentity
        | ProjectionFactKind::DisplayField
        | ProjectionFactKind::DerivedField => ProjectionConsumptionSupportPosture::Admitted,
        ProjectionFactKind::TargetIdentity
        | ProjectionFactKind::SourceReference
        | ProjectionFactKind::EffectContinuity
        | ProjectionFactKind::Membership
        | ProjectionFactKind::RelationEndpoint => {
            ProjectionConsumptionSupportPosture::SourceMismatch
        }
    }
}

fn bridge_grouped_truth_view_support(
    fact_kind: ProjectionFactKind,
) -> ProjectionConsumptionSupportPosture {
    match fact_kind {
        ProjectionFactKind::ViewLocalIdentity
        | ProjectionFactKind::Membership
        | ProjectionFactKind::RelationEndpoint => ProjectionConsumptionSupportPosture::Admitted,
        ProjectionFactKind::EntityIdentity
        | ProjectionFactKind::TargetIdentity
        | ProjectionFactKind::SourceReference
        | ProjectionFactKind::EffectContinuity
        | ProjectionFactKind::DisplayField
        | ProjectionFactKind::DerivedField => ProjectionConsumptionSupportPosture::SourceMismatch,
    }
}

fn retained_derived_artifact_binding_support(
    fact_kind: ProjectionFactKind,
) -> ProjectionConsumptionSupportPosture {
    match fact_kind {
        ProjectionFactKind::ViewLocalIdentity
        | ProjectionFactKind::SourceReference
        | ProjectionFactKind::DisplayField
        | ProjectionFactKind::DerivedField => ProjectionConsumptionSupportPosture::Admitted,
        ProjectionFactKind::EntityIdentity
        | ProjectionFactKind::TargetIdentity
        | ProjectionFactKind::EffectContinuity
        | ProjectionFactKind::Membership
        | ProjectionFactKind::RelationEndpoint => {
            ProjectionConsumptionSupportPosture::SourceMismatch
        }
    }
}

fn live_artifact_binding_support(
    fact_kind: ProjectionFactKind,
) -> ProjectionConsumptionSupportPosture {
    match fact_kind {
        ProjectionFactKind::EntityIdentity
        | ProjectionFactKind::ViewLocalIdentity
        | ProjectionFactKind::SourceReference
        | ProjectionFactKind::DisplayField
        | ProjectionFactKind::DerivedField => ProjectionConsumptionSupportPosture::Admitted,
        ProjectionFactKind::TargetIdentity
        | ProjectionFactKind::EffectContinuity
        | ProjectionFactKind::Membership
        | ProjectionFactKind::RelationEndpoint => {
            ProjectionConsumptionSupportPosture::SourceMismatch
        }
    }
}
