use super::super::super::bridge_lowering::BridgeSubscriptionLoweringPlan;
use super::super::super::declaration::QuerySubscriptionDeclarationArtifact;
use super::super::super::support::QuerySubscriptionSupportPosture;
use super::super::super::support::QuerySubscriptionSupportReport;
use super::super::context::QuerySubscriptionDiagnosticSelectionContext;
use super::super::stage::QuerySubscriptionDiagnosticStage;
use super::evidence::QuerySubscriptionDiagnosticSemanticLabels;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

pub(super) fn semantic_labels_for_support(
    query_family_label: &str,
    declaration: &QuerySubscriptionDeclarationArtifact,
    lowering: &BridgeSubscriptionLoweringPlan,
    support_posture: &QuerySubscriptionSupportPosture,
    denial_or_coverage_class_label: &str,
) -> Result<QuerySubscriptionDiagnosticSemanticLabels, IdentityDerivationDenial> {
    QuerySubscriptionDiagnosticSemanticLabels::new(
        query_family_label.to_string(),
        declaration.family().as_str().to_string(),
        lowering.bridge_family().as_str().to_string(),
        lowering
            .bridge_slices()
            .iter()
            .map(|slice| slice.as_str().to_string())
            .collect(),
        declaration.basis_posture().as_str().to_string(),
        lowering
            .signal_strategy_request()
            .request_kind()
            .as_str()
            .to_string(),
        declaration.live_graph_access_posture().as_str().to_string(),
        support_posture.as_str().to_string(),
        denial_or_coverage_class_label.to_string(),
    )
}

pub(super) fn semantic_labels_for_denied_bundle(
    selection: &QuerySubscriptionDiagnosticSelectionContext,
    declaration: Option<&QuerySubscriptionDeclarationArtifact>,
    lowering: Option<&BridgeSubscriptionLoweringPlan>,
    support: Option<&QuerySubscriptionSupportReport>,
    denial_class_label: &str,
) -> Result<QuerySubscriptionDiagnosticSemanticLabels, IdentityDerivationDenial> {
    QuerySubscriptionDiagnosticSemanticLabels::new(
        selection.query_family_label().to_string(),
        declaration
            .map(|value| value.family().as_str().to_string())
            .unwrap_or_else(|| selection.declaration_family_label().to_string()),
        lowering
            .map(|value| value.bridge_family().as_str().to_string())
            .unwrap_or_else(|| "not_lowered".to_string()),
        lowering
            .map(|value| {
                value
                    .bridge_slices()
                    .iter()
                    .map(|slice| slice.as_str().to_string())
                    .collect()
            })
            .unwrap_or_default(),
        declaration
            .map(|value| value.basis_posture().as_str().to_string())
            .unwrap_or_else(|| selection.basis_posture_label().to_string()),
        lowering
            .map(|value| {
                value
                    .signal_strategy_request()
                    .request_kind()
                    .as_str()
                    .to_string()
            })
            .unwrap_or_else(|| "not_lowered".to_string()),
        declaration
            .map(|value| value.live_graph_access_posture().as_str().to_string())
            .unwrap_or_else(|| selection.live_graph_access_posture_label().to_string()),
        support
            .map(|value| value.support_posture().as_str().to_string())
            .unwrap_or_else(|| "not_reported".to_string()),
        denial_class_label.to_string(),
    )
}

pub(super) fn omitted_stages_after_failure(
    failure_stage: QuerySubscriptionDiagnosticStage,
) -> Vec<QuerySubscriptionDiagnosticStage> {
    match failure_stage {
        QuerySubscriptionDiagnosticStage::FamilySelection
        | QuerySubscriptionDiagnosticStage::ViewMismatch
        | QuerySubscriptionDiagnosticStage::RelationshipProofDrift => vec![
            QuerySubscriptionDiagnosticStage::Declaration,
            QuerySubscriptionDiagnosticStage::BridgeFamilyLowering,
            QuerySubscriptionDiagnosticStage::RuntimeBackedAdmission,
            QuerySubscriptionDiagnosticStage::SupportReporting,
            QuerySubscriptionDiagnosticStage::Certification,
        ],
        QuerySubscriptionDiagnosticStage::Declaration
        | QuerySubscriptionDiagnosticStage::DeliveryIntent => vec![
            QuerySubscriptionDiagnosticStage::BridgeFamilyLowering,
            QuerySubscriptionDiagnosticStage::RuntimeBackedAdmission,
            QuerySubscriptionDiagnosticStage::SupportReporting,
            QuerySubscriptionDiagnosticStage::Certification,
        ],
        QuerySubscriptionDiagnosticStage::BridgeFamilyLowering
        | QuerySubscriptionDiagnosticStage::BridgeSliceLowering
        | QuerySubscriptionDiagnosticStage::BasisBinding => vec![
            QuerySubscriptionDiagnosticStage::RuntimeBackedAdmission,
            QuerySubscriptionDiagnosticStage::SupportReporting,
            QuerySubscriptionDiagnosticStage::Certification,
        ],
        QuerySubscriptionDiagnosticStage::RuntimeBackedAdmission
        | QuerySubscriptionDiagnosticStage::AdmissionBudget
        | QuerySubscriptionDiagnosticStage::DurableReloadOverclaim
        | QuerySubscriptionDiagnosticStage::ActiveLifecycleAllocation
        | QuerySubscriptionDiagnosticStage::ActivationReadiness => vec![
            QuerySubscriptionDiagnosticStage::SupportReporting,
            QuerySubscriptionDiagnosticStage::Certification,
        ],
        QuerySubscriptionDiagnosticStage::SupportReporting => {
            vec![QuerySubscriptionDiagnosticStage::Certification]
        }
        _ => Vec::new(),
    }
}

pub(super) fn semantic_label_count(labels: &QuerySubscriptionDiagnosticSemanticLabels) -> usize {
    8 + labels.bridge_slice_labels().len()
}
