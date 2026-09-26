use crate::declaration::UiDeclaredMeasurementConstraintModifier;
use crate::evidence::{
    UiAllocationNeighborhood, UiConstraintAvailableSpacePosture, UiConstraintAxisScope,
    UiConstraintBoundedMinMaxRequirement, UiConstraintNormalizationPosture,
    UiConstraintParentAvailableSpace, UiConstraintPropagationDenial,
    UiConstraintPropagationDenialReason, UiConstraintPropagationEdge,
    UiConstraintPropagationEdgeFamily, UiConstraintPropagationEdgePayload,
    UiLayoutOperatorCrossAxis, UiLayoutOperatorPlanningContract, UiLayoutOperatorPrimaryAxis,
    UiMeasurementBasis, UiMeasurementValue,
};

use super::constraint_normalization::admit_downward_normalization_posture;

pub(super) struct UiConstraintDownwardAdmission {
    incoming_available_space: Option<UiConstraintAxisScope>,
    incoming_available_space_posture: Option<UiConstraintAvailableSpacePosture>,
    bounded_min_max_requirement: UiConstraintBoundedMinMaxRequirement,
    bounded_targets: Vec<(u64, UiConstraintAxisScope)>,
    edges: Vec<UiConstraintPropagationEdge>,
    normalization_posture: UiConstraintNormalizationPosture,
}

impl UiConstraintDownwardAdmission {
    pub(super) const fn incoming_available_space(&self) -> Option<UiConstraintAxisScope> {
        self.incoming_available_space
    }

    pub(super) const fn incoming_available_space_posture(
        &self,
    ) -> Option<UiConstraintAvailableSpacePosture> {
        self.incoming_available_space_posture
    }

    pub(super) const fn bounded_min_max_requirement(&self) -> UiConstraintBoundedMinMaxRequirement {
        self.bounded_min_max_requirement
    }

    pub(super) fn bounded_targets(&self) -> &[(u64, UiConstraintAxisScope)] {
        &self.bounded_targets
    }

    pub(super) const fn normalization_posture(&self) -> UiConstraintNormalizationPosture {
        self.normalization_posture
    }

    pub(super) fn into_edges(self) -> Vec<UiConstraintPropagationEdge> {
        self.edges
    }
}

pub(super) fn admit_parent_available_space(
    measurement_basis: &UiMeasurementBasis,
    neighborhood: &UiAllocationNeighborhood,
    allowed_families: &[UiConstraintPropagationEdgeFamily],
) -> Result<UiConstraintDownwardAdmission, UiConstraintPropagationDenial> {
    let contract = neighborhood.layout_operator_planning_contract();
    let normalization_posture = admit_downward_normalization_posture(
        measurement_basis,
        neighborhood.identity().identity_digest(),
        contract.identity().identity_digest(),
    )?;
    let incoming_available_space =
        parent_available_space_scope(contract.semantics().primary_axis(), allowed_families);

    let Some(axis_scope) = incoming_available_space else {
        return Ok(UiConstraintDownwardAdmission {
            incoming_available_space: None,
            incoming_available_space_posture: None,
            bounded_min_max_requirement: UiConstraintBoundedMinMaxRequirement::None,
            bounded_targets: Vec::new(),
            edges: Vec::new(),
            normalization_posture,
        });
    };

    let root = neighborhood
        .members()
        .iter()
        .find(|member| {
            matches!(
                member.role(),
                crate::evidence::UiAllocationNeighborhoodMemberRole::Root
            )
        })
        .expect("allocation neighborhood must preserve a root member");
    let root_identity_digest = root.identity_digest();
    let neighborhood_identity_digest = neighborhood.identity().identity_digest();
    let contract_identity_digest = neighborhood.layout_operator_contract_identity_digest();

    if matches!(
        normalization_posture,
        UiConstraintNormalizationPosture::Deferred
    ) && measurement_basis.denial_posture().is_some()
    {
        return Err(UiConstraintPropagationDenial::new(
            UiConstraintPropagationDenialReason::MissingRequiredDownwardConstraint,
            neighborhood_identity_digest,
            contract_identity_digest,
            Some(UiConstraintPropagationEdgeFamily::ParentAvailableSpace),
            measurement_basis.identity_digest(),
        ));
    }

    let mut bounded_min_max_requirement = UiConstraintBoundedMinMaxRequirement::None;
    let mut bounded_targets = Vec::new();
    let available_space_posture = admitted_available_space_posture(
        measurement_basis,
        axis_scope,
        contract.semantics().primary_axis(),
    );
    let edges = neighborhood
        .members()
        .iter()
        .filter(|member| member.identity_digest() != root_identity_digest)
        .map(|member| {
            let parent_available_space = UiConstraintParentAvailableSpace::new(
                axis_scope,
                available_space_posture,
                child_bounded_requirement(member.measurement_constraint_modifier(), contract),
                normalization_posture,
            );
            bounded_min_max_requirement = max_bounded_requirement(
                bounded_min_max_requirement,
                parent_available_space.bounded_min_max_requirement(),
            );
            if let Some(axis_scope) = bounded_axis_scope(parent_available_space) {
                bounded_targets.push((member.identity_digest(), axis_scope));
            }

            UiConstraintPropagationEdge::new(
                UiConstraintPropagationEdgeFamily::ParentAvailableSpace,
                root_identity_digest,
                member.identity_digest(),
                UiConstraintPropagationEdgePayload::ParentAvailableSpace(parent_available_space),
                crate::evidence::UiConstraintCycleParticipationPosture::Acyclic,
            )
        })
        .collect();

    Ok(UiConstraintDownwardAdmission {
        incoming_available_space: Some(axis_scope),
        incoming_available_space_posture: Some(available_space_posture),
        bounded_min_max_requirement,
        bounded_targets,
        edges,
        normalization_posture,
    })
}

fn parent_available_space_scope(
    primary_axis: UiLayoutOperatorPrimaryAxis,
    allowed_families: &[UiConstraintPropagationEdgeFamily],
) -> Option<UiConstraintAxisScope> {
    if !allowed_families.contains(&UiConstraintPropagationEdgeFamily::ParentAvailableSpace) {
        return None;
    }

    match primary_axis {
        UiLayoutOperatorPrimaryAxis::None => None,
        UiLayoutOperatorPrimaryAxis::Vertical | UiLayoutOperatorPrimaryAxis::Horizontal => {
            Some(UiConstraintAxisScope::Primary)
        }
        UiLayoutOperatorPrimaryAxis::TwoDimensional | UiLayoutOperatorPrimaryAxis::Layered => {
            Some(UiConstraintAxisScope::Both)
        }
    }
}

fn child_bounded_requirement(
    child_constraint_modifier: Option<UiDeclaredMeasurementConstraintModifier>,
    contract: &UiLayoutOperatorPlanningContract,
) -> UiConstraintBoundedMinMaxRequirement {
    if child_constraint_modifier != Some(UiDeclaredMeasurementConstraintModifier::Bounded)
        || contract.constraint_modifier() != Some(UiDeclaredMeasurementConstraintModifier::Bounded)
    {
        return UiConstraintBoundedMinMaxRequirement::None;
    }

    match contract.semantics().cross_axis() {
        UiLayoutOperatorCrossAxis::Horizontal | UiLayoutOperatorCrossAxis::Vertical => {
            UiConstraintBoundedMinMaxRequirement::PrimaryAxis
        }
        UiLayoutOperatorCrossAxis::TwoDimensional
        | UiLayoutOperatorCrossAxis::Layered
        | UiLayoutOperatorCrossAxis::None => UiConstraintBoundedMinMaxRequirement::BothAxes,
    }
}

fn max_bounded_requirement(
    left: UiConstraintBoundedMinMaxRequirement,
    right: UiConstraintBoundedMinMaxRequirement,
) -> UiConstraintBoundedMinMaxRequirement {
    use UiConstraintBoundedMinMaxRequirement::{BothAxes, None as BoundedNone, PrimaryAxis};

    match (left, right) {
        (BothAxes, _) | (_, BothAxes) => BothAxes,
        (PrimaryAxis, _) | (_, PrimaryAxis) => PrimaryAxis,
        (BoundedNone, BoundedNone) => BoundedNone,
    }
}

fn bounded_axis_scope(
    parent_available_space: UiConstraintParentAvailableSpace,
) -> Option<UiConstraintAxisScope> {
    match parent_available_space.bounded_min_max_requirement() {
        UiConstraintBoundedMinMaxRequirement::None => None,
        UiConstraintBoundedMinMaxRequirement::PrimaryAxis => Some(UiConstraintAxisScope::Primary),
        UiConstraintBoundedMinMaxRequirement::BothAxes => Some(UiConstraintAxisScope::Both),
    }
}

fn admitted_available_space_posture(
    measurement_basis: &UiMeasurementBasis,
    axis_scope: UiConstraintAxisScope,
    primary_axis: UiLayoutOperatorPrimaryAxis,
) -> UiConstraintAvailableSpacePosture {
    measurement_basis
        .evidence_inputs()
        .iter()
        .filter_map(|input| input.as_host_measurement_result())
        .find_map(|result| extent_zero_posture(result.value(), axis_scope, primary_axis))
        .unwrap_or(UiConstraintAvailableSpacePosture::DeclaredExtentUnknown)
}

fn extent_zero_posture(
    value: &UiMeasurementValue,
    axis_scope: UiConstraintAxisScope,
    primary_axis: UiLayoutOperatorPrimaryAxis,
) -> Option<UiConstraintAvailableSpacePosture> {
    let (width, height) = match value {
        UiMeasurementValue::ViewportExtent(value) => (value.width, value.height),
        UiMeasurementValue::ScrollContainerViewport(value) => (value.width, value.height),
        UiMeasurementValue::TextIntrinsicSize(_)
        | UiMeasurementValue::TextBaselineMetrics(_)
        | UiMeasurementValue::FontMetrics(_)
        | UiMeasurementValue::NativeControlIntrinsicSize(_)
        | UiMeasurementValue::DpiScaleFactor(_)
        | UiMeasurementValue::PortalAnchorRect(_) => return None,
    };
    let zero = match axis_scope {
        UiConstraintAxisScope::Primary => primary_extent(width, height, primary_axis) == 0.0,
        UiConstraintAxisScope::Cross => cross_extent(width, height, primary_axis) == Some(0.0),
        UiConstraintAxisScope::Both => width == 0.0 || height == 0.0,
    };
    Some(if zero {
        UiConstraintAvailableSpacePosture::AdmittedZeroExtent
    } else {
        UiConstraintAvailableSpacePosture::AdmittedPositiveExtent
    })
}

fn primary_extent(width: f32, height: f32, primary_axis: UiLayoutOperatorPrimaryAxis) -> f32 {
    match primary_axis {
        UiLayoutOperatorPrimaryAxis::Vertical => height,
        UiLayoutOperatorPrimaryAxis::Horizontal => width,
        UiLayoutOperatorPrimaryAxis::TwoDimensional
        | UiLayoutOperatorPrimaryAxis::Layered
        | UiLayoutOperatorPrimaryAxis::None => height,
    }
}

fn cross_extent(width: f32, height: f32, primary_axis: UiLayoutOperatorPrimaryAxis) -> Option<f32> {
    match primary_axis {
        UiLayoutOperatorPrimaryAxis::Vertical => Some(width),
        UiLayoutOperatorPrimaryAxis::Horizontal => Some(height),
        UiLayoutOperatorPrimaryAxis::TwoDimensional | UiLayoutOperatorPrimaryAxis::Layered => {
            Some(width.min(height))
        }
        UiLayoutOperatorPrimaryAxis::None => None,
    }
}
