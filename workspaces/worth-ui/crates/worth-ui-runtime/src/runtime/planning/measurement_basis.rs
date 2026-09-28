use crate::evidence::UiMeasurementBasis;
use crate::runtime::WorthUiAdmittedDurableResizeInput;

/// Collect planning measurement basis with runtime durable-resize evidence when admitted.
pub(crate) fn collect_planning_measurement_basis(
    measurement_basis: &UiMeasurementBasis,
    allocation_neighborhood: &crate::evidence::UiAllocationNeighborhood,
    durable_resize_inputs: &[WorthUiAdmittedDurableResizeInput],
) -> UiMeasurementBasis {
    let axis_scope = match allocation_neighborhood
        .layout_operator_planning_contract()
        .operator_kind()
    {
        crate::declaration::UiDeclarationPlanningOperatorKind::Split => {
            crate::evidence::UiConstraintAxisScope::Primary
        }
        crate::declaration::UiDeclarationPlanningOperatorKind::Mosaic => {
            crate::evidence::UiConstraintAxisScope::Both
        }
        crate::declaration::UiDeclarationPlanningOperatorKind::PageRoot
        | crate::declaration::UiDeclarationPlanningOperatorKind::PageSet
        | crate::declaration::UiDeclarationPlanningOperatorKind::Region
        | crate::declaration::UiDeclarationPlanningOperatorKind::LocalComposition
        | crate::declaration::UiDeclarationPlanningOperatorKind::Control
        | crate::declaration::UiDeclarationPlanningOperatorKind::DiagnosticSurface
        | crate::declaration::UiDeclarationPlanningOperatorKind::Stack
        | crate::declaration::UiDeclarationPlanningOperatorKind::Row
        | crate::declaration::UiDeclarationPlanningOperatorKind::Grid
        | crate::declaration::UiDeclarationPlanningOperatorKind::Overlay
        | crate::declaration::UiDeclarationPlanningOperatorKind::Scroll
        | crate::declaration::UiDeclarationPlanningOperatorKind::PortalAnchor => {
            return measurement_basis.clone()
        }
    };
    let root_provenance_digest = allocation_neighborhood
        .members()
        .iter()
        .find(|member| {
            matches!(
                member.role(),
                crate::evidence::UiAllocationNeighborhoodMemberRole::Root
            )
        })
        .expect("allocation neighborhood must preserve a root member")
        .authored_provenance_digest();
    let Some(durable_resize_input) = durable_resize_inputs
        .iter()
        .find(|input| input.authored_provenance_digest() == Some(root_provenance_digest))
    else {
        return measurement_basis.clone();
    };
    let Some(runtime_resize_support) =
        crate::evidence::MeasurementEvidenceInput::runtime_durable_resize_support(
            durable_resize_input,
            allocation_neighborhood.root_graph_node_identity(),
            axis_scope,
            allocation_neighborhood
                .layout_operator_planning_contract()
                .mosaic_sizing_contract_id(),
        )
    else {
        return measurement_basis.clone();
    };
    let mut evidence_inputs = measurement_basis.evidence_inputs().to_vec();
    evidence_inputs.push(runtime_resize_support);
    crate::evidence::admit_measurement_basis(
        measurement_basis.declaration_identity().clone(),
        measurement_basis.graph_node_identity(),
        measurement_basis.world_profile().clone(),
        measurement_basis.declaration_support_authority_generation(),
        measurement_basis.declared_measurement_policy(),
        &evidence_inputs,
    )
}
