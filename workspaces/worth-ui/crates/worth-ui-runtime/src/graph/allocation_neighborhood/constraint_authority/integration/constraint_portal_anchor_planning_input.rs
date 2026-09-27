use crate::evidence::{
    MeasurementEvidenceInput, UiAllocationNeighborhood,
    UiConstraintPortalAnchorPlanningInputResult, UiConstraintPropagationDenial,
    UiConstraintPropagationDenialReason, UiConstraintPropagationEdgeFamily, UiMeasurementBasis,
    UiMeasurementCoordinateSpace, UiMeasurementDependencyLineageKind, UiMeasurementRoundingPosture,
    UiMeasurementUnitPosture, UiMeasurementValue, UiPortalAnchorPlanningInputPosture,
    UiPortalAnchorPlanningInputSolveOrder,
};

pub(super) fn admit_portal_anchor_planning_input(
    measurement_basis: &UiMeasurementBasis,
    neighborhood: &UiAllocationNeighborhood,
    portal_anchor_required: bool,
    allowed_families: &[UiConstraintPropagationEdgeFamily],
    successor: Option<&crate::runtime::UiPortalAllocationPlanningBasis>,
) -> Result<Option<UiConstraintPortalAnchorPlanningInputResult>, UiConstraintPropagationDenial> {
    if !allowed_families.contains(&UiConstraintPropagationEdgeFamily::PortalAnchorInput) {
        return Ok(None);
    }
    if !portal_anchor_required {
        return Ok(None);
    }

    let neighborhood_identity_digest = neighborhood.identity().identity_digest();
    let contract_identity_digest = neighborhood
        .layout_operator_planning_contract()
        .identity()
        .identity_digest();
    let result = match successor
        .map(portal_successor_source)
        .or_else(|| portal_anchor_source(measurement_basis))
    {
        Some((source_identity_digest, source_generation_digest, unit, coordinate, rounding)) => {
            let posture = if measurement_basis.generation_compatibility().is_compatible() {
                UiPortalAnchorPlanningInputPosture::AdmittedPlanningTimeOnly
            } else {
                UiPortalAnchorPlanningInputPosture::IncompatibleMeasurementPosture
            };
            UiConstraintPortalAnchorPlanningInputResult::new(
                crate::evidence::UiConstraintPortalAnchorPlanningInput {
                    neighborhood_identity_digest,
                    solve_order:
                        UiPortalAnchorPlanningInputSolveOrder::BeforeDerivedConstraintFamilies,
                    posture,
                    source_evidence_identity_digest: Some(source_identity_digest),
                    source_generation_digest: Some(source_generation_digest),
                    unit_posture: Some(unit),
                    coordinate_space: Some(coordinate),
                    rounding_posture: Some(rounding),
                    planning_time_only: true,
                },
            )
        }
        None => UiConstraintPortalAnchorPlanningInputResult::new(
            crate::evidence::UiConstraintPortalAnchorPlanningInput {
                neighborhood_identity_digest,
                solve_order: UiPortalAnchorPlanningInputSolveOrder::BeforeDerivedConstraintFamilies,
                posture: UiPortalAnchorPlanningInputPosture::MissingRequiredEvidence,
                source_evidence_identity_digest: None,
                source_generation_digest: None,
                unit_posture: None,
                coordinate_space: None,
                rounding_posture: None,
                planning_time_only: true,
            },
        ),
    };

    match result.posture() {
        UiPortalAnchorPlanningInputPosture::AdmittedPlanningTimeOnly => Ok(Some(result)),
        UiPortalAnchorPlanningInputPosture::MissingRequiredEvidence => {
            Err(UiConstraintPropagationDenial::new(
                UiConstraintPropagationDenialReason::MissingRequiredPortalAnchorPlanningInput,
                neighborhood_identity_digest,
                contract_identity_digest,
                Some(UiConstraintPropagationEdgeFamily::PortalAnchorInput),
                result.identity_digest(),
            ))
        }
        UiPortalAnchorPlanningInputPosture::IncompatibleMeasurementPosture => {
            Err(UiConstraintPropagationDenial::new(
                UiConstraintPropagationDenialReason::IncompatibleMeasurementPosture,
                neighborhood_identity_digest,
                contract_identity_digest,
                Some(UiConstraintPropagationEdgeFamily::PortalAnchorInput),
                result.identity_digest(),
            ))
        }
    }
}

fn portal_successor_source(
    successor: &crate::runtime::UiPortalAllocationPlanningBasis,
) -> (
    u64,
    u64,
    UiMeasurementUnitPosture,
    UiMeasurementCoordinateSpace,
    UiMeasurementRoundingPosture,
) {
    let observation = successor.observation();
    (
        successor.identity_digest(),
        observation.evidence_generation().as_u64(),
        observation.unit_posture(),
        observation.identity().coordinate_space(),
        observation.rounding_posture(),
    )
}

fn portal_anchor_source(
    measurement_basis: &UiMeasurementBasis,
) -> Option<(
    u64,
    u64,
    UiMeasurementUnitPosture,
    UiMeasurementCoordinateSpace,
    UiMeasurementRoundingPosture,
)> {
    let source = measurement_basis
        .dependency_lineage()
        .entries()
        .iter()
        .find(|entry| entry.kind() == UiMeasurementDependencyLineageKind::HostPortalAnchorRect)?;
    let result = measurement_basis
        .evidence_inputs()
        .iter()
        .find_map(|input| match input {
            MeasurementEvidenceInput::HostMeasurementResult(result)
                if matches!(result.value(), UiMeasurementValue::PortalAnchorRect(_)) =>
            {
                Some(result)
            }
            MeasurementEvidenceInput::SettledQueryFact(_)
            | MeasurementEvidenceInput::HostMeasurementResult(_)
            | MeasurementEvidenceInput::HostCapabilityReport(_)
            | MeasurementEvidenceInput::ChildIntrinsicMeasurement(_)
            | MeasurementEvidenceInput::SiblingResizeSupport(_) => None,
        })?;
    Some((
        source.identity_digest(),
        source.generation_digest(),
        result.unit_posture(),
        result.coordinate_space(),
        result.rounding_posture(),
    ))
}
