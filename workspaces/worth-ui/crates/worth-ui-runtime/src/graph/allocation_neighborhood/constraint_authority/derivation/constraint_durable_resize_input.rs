use crate::declaration::UiDeclarationPlanningOperatorKind;
use crate::evidence::{
    MeasurementEvidenceInput, UiAllocationConstraintSummary, UiAllocationNeighborhood,
    UiConstraintPropagationEdge, UiConstraintPropagationEdgeFamily,
    UiConstraintPropagationEdgePayload, UiConstraintResizeInputPosture,
};

use super::constraint_summary::axis_scope_for_sibling_mode;

pub(super) fn admit_durable_resize_inputs(
    measurement_basis: &crate::evidence::UiMeasurementBasis,
    neighborhood: &UiAllocationNeighborhood,
    summary: UiAllocationConstraintSummary,
    allowed_families: &[UiConstraintPropagationEdgeFamily],
) -> Vec<UiConstraintPropagationEdge> {
    if !allowed_families.contains(&UiConstraintPropagationEdgeFamily::DurableResizeInput) {
        return Vec::new();
    }
    let Some(axis_scope) = axis_scope_for_sibling_mode(summary.sibling_negotiation_mode()) else {
        return Vec::new();
    };
    let Some(resize_support) =
        admitted_runtime_resize_support(measurement_basis, neighborhood, axis_scope)
    else {
        return Vec::new();
    };
    let root_identity_digest = neighborhood
        .members()
        .iter()
        .find(|member| {
            matches!(
                member.role(),
                crate::evidence::UiAllocationNeighborhoodMemberRole::Root
            )
        })
        .expect("allocation neighborhood must preserve a root member")
        .identity_digest();
    vec![UiConstraintPropagationEdge::new(
        UiConstraintPropagationEdgeFamily::DurableResizeInput,
        root_identity_digest,
        root_identity_digest,
        UiConstraintPropagationEdgePayload::DurableResizeInput {
            durable_identity_digest: resize_support.source_identity_digest(),
            axis_scope,
            posture: UiConstraintResizeInputPosture::DurableAuthorityRequired,
            planning_time_only: resize_support.is_planning_time_only(),
        },
        crate::evidence::UiConstraintCycleParticipationPosture::Acyclic,
    )]
}

fn admitted_runtime_resize_support<'a>(
    measurement_basis: &'a crate::evidence::UiMeasurementBasis,
    neighborhood: &UiAllocationNeighborhood,
    axis_scope: crate::evidence::UiConstraintAxisScope,
) -> Option<&'a crate::evidence::UiMeasurementSiblingResizeSupport> {
    measurement_basis.evidence_inputs().iter().find_map(|input| match input {
        MeasurementEvidenceInput::SiblingResizeSupport(support)
            if support.source()
                == crate::evidence::UiMeasurementSiblingResizeSupportSource::RuntimeDurableResizeWitness
                && support.axis_scope() == axis_scope
                && support.target_graph_node_identity() == neighborhood.root_graph_node_identity()
                && resize_contract_matches_operator(support, neighborhood.layout_operator_planning_contract().operator_kind(), neighborhood.layout_operator_planning_contract().mosaic_sizing_contract_id()) =>
        {
            Some(support)
        }
        MeasurementEvidenceInput::SettledQueryFact(_)
        | MeasurementEvidenceInput::HostMeasurementResult(_)
        | MeasurementEvidenceInput::HostCapabilityReport(_)
        | MeasurementEvidenceInput::ChildIntrinsicMeasurement(_)
        | MeasurementEvidenceInput::SiblingResizeSupport(_) => None,
    })
}

fn resize_contract_matches_operator(
    support: &crate::evidence::UiMeasurementSiblingResizeSupport,
    operator_kind: UiDeclarationPlanningOperatorKind,
    contract_id: Option<&crate::capability::MosaicSizingContractId>,
) -> bool {
    match operator_kind {
        UiDeclarationPlanningOperatorKind::Split => support.sizing_contract_id().is_none(),
        UiDeclarationPlanningOperatorKind::Mosaic => support.sizing_contract_id() == contract_id,
        UiDeclarationPlanningOperatorKind::PageRoot
        | UiDeclarationPlanningOperatorKind::PageSet
        | UiDeclarationPlanningOperatorKind::Region
        | UiDeclarationPlanningOperatorKind::LocalComposition
        | UiDeclarationPlanningOperatorKind::Control
        | UiDeclarationPlanningOperatorKind::DiagnosticSurface
        | UiDeclarationPlanningOperatorKind::Stack
        | UiDeclarationPlanningOperatorKind::Row
        | UiDeclarationPlanningOperatorKind::Grid
        | UiDeclarationPlanningOperatorKind::Overlay
        | UiDeclarationPlanningOperatorKind::Scroll
        | UiDeclarationPlanningOperatorKind::PortalAnchor => false,
    }
}
