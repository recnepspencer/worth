use crate::declaration::stable_text_digest;
use crate::evidence::{
    MeasurementEvidenceInput, UiAllocationNeighborhood, UiAllocationNeighborhoodMember,
    UiConstraintAxisScope, UiConstraintChildIntrinsicContribution,
    UiConstraintCycleParticipationPosture, UiConstraintHostIntrinsicKind,
    UiConstraintIntrinsicSourcePosture, UiConstraintPropagationDenial,
    UiConstraintPropagationDenialReason, UiConstraintPropagationEdge,
    UiConstraintPropagationEdgeFamily, UiConstraintPropagationEdgePayload,
    UiLayoutOperatorIntrinsicReturnPolicy, UiLayoutOperatorPrimaryAxis, UiMeasurementBasis,
    UiMeasurementCoordinateSpace, UiMeasurementResult, UiMeasurementRoundingPosture,
    UiMeasurementUnitPosture, UiMeasurementValue,
};
use worth_foundational::CanonicalF32;

use super::constraint_summary::intrinsic_contribution_scope;
use crate::evidence::UiMeasurementEvidenceCategory;

#[derive(Clone, Copy)]
struct AdmittedIntrinsicValue {
    primary_extent: CanonicalF32,
    cross_extent: Option<CanonicalF32>,
    unit_posture: UiMeasurementUnitPosture,
    coordinate_space: UiMeasurementCoordinateSpace,
    rounding_posture: UiMeasurementRoundingPosture,
    host_kind: UiConstraintHostIntrinsicKind,
}

pub(super) fn admit_child_intrinsic_contributions(
    measurement_basis: &UiMeasurementBasis,
    neighborhood: &UiAllocationNeighborhood,
    allowed_families: &[UiConstraintPropagationEdgeFamily],
) -> Result<Vec<UiConstraintPropagationEdge>, UiConstraintPropagationDenial> {
    if !allowed_families.contains(&UiConstraintPropagationEdgeFamily::ChildIntrinsicContribution) {
        return Ok(Vec::new());
    }

    let contract = neighborhood.layout_operator_planning_contract();
    if contract.semantics().intrinsic_return_policy()
        != UiLayoutOperatorIntrinsicReturnPolicy::ChildrenToParent
    {
        return Ok(Vec::new());
    }

    let non_root_members = neighborhood
        .members()
        .iter()
        .filter(|member| {
            !matches!(
                member.role(),
                crate::evidence::UiAllocationNeighborhoodMemberRole::Root
            )
        })
        .collect::<Vec<_>>();
    if non_root_members.is_empty() {
        return Ok(Vec::new());
    }

    let Some(axis_scope) =
        intrinsic_contribution_scope(contract.semantics().child_participation_rule())
    else {
        return Ok(Vec::new());
    };
    if contains_anonymous_intrinsic_evidence(measurement_basis) {
        return Err(intrinsic_denial(
            UiConstraintPropagationDenialReason::MissingRequiredIntrinsicContribution,
            neighborhood,
            measurement_basis,
        ));
    }
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

    Ok(non_root_members
        .into_iter()
        .map(|member| {
            let contribution = admit_member_intrinsic_contribution_witness(
                measurement_basis,
                neighborhood,
                member,
                axis_scope,
                contract.semantics().primary_axis(),
            )?;
            Ok(contribution.map(|contribution| {
                UiConstraintPropagationEdge::new(
                    UiConstraintPropagationEdgeFamily::ChildIntrinsicContribution,
                    member.identity_digest(),
                    root_identity_digest,
                    UiConstraintPropagationEdgePayload::ChildIntrinsicContribution(contribution),
                    UiConstraintCycleParticipationPosture::Acyclic,
                )
            }))
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect())
}

fn contains_anonymous_intrinsic_evidence(measurement_basis: &UiMeasurementBasis) -> bool {
    measurement_basis
        .evidence_inputs()
        .iter()
        .any(|input| match input {
            MeasurementEvidenceInput::SettledQueryFact(_) => true,
            MeasurementEvidenceInput::HostMeasurementResult(result) => matches!(
                result.evidence_category(),
                UiMeasurementEvidenceCategory::TextIntrinsicSize
                    | UiMeasurementEvidenceCategory::NativeControlIntrinsicSize
            ),
            MeasurementEvidenceInput::HostCapabilityReport(_)
            | MeasurementEvidenceInput::ChildIntrinsicMeasurement(_)
            | MeasurementEvidenceInput::SiblingResizeSupport(_) => false,
        })
}

fn admit_member_intrinsic_contribution_witness(
    measurement_basis: &UiMeasurementBasis,
    neighborhood: &UiAllocationNeighborhood,
    member: &UiAllocationNeighborhoodMember,
    axis_scope: UiConstraintAxisScope,
    primary_axis: UiLayoutOperatorPrimaryAxis,
) -> Result<Option<UiConstraintChildIntrinsicContribution>, UiConstraintPropagationDenial> {
    let contributor_graph_node_identity = member.graph_node_identity();
    let mut query_extent = None;
    let mut host_intrinsic = None;
    let mut saw_host_intrinsic = false;

    for input in measurement_basis.evidence_inputs() {
        if let Some(evidence) = input.as_child_intrinsic_measurement() {
            if evidence.contributor_graph_node_identity() != contributor_graph_node_identity {
                continue;
            }
            if let Some(receipt) = evidence.query_projection_fact() {
                query_extent = receipt
                    .observations()
                    .first()
                    .map(|observation| observation.extent());
            }
            if let Some(result) = evidence.host_measurement_result() {
                match result.evidence_category() {
                    UiMeasurementEvidenceCategory::TextIntrinsicSize
                    | UiMeasurementEvidenceCategory::NativeControlIntrinsicSize => {
                        saw_host_intrinsic = true;
                        host_intrinsic =
                            merge_host_intrinsic(host_intrinsic, result, axis_scope, primary_axis)
                                .map_err(|reason| {
                                    intrinsic_denial(reason, neighborhood, measurement_basis)
                                })?;
                    }
                    UiMeasurementEvidenceCategory::TextBaselineMetrics
                    | UiMeasurementEvidenceCategory::FontMetrics
                    | UiMeasurementEvidenceCategory::ViewportExtent
                    | UiMeasurementEvidenceCategory::DpiScaleFactor
                    | UiMeasurementEvidenceCategory::PortalAnchorRect
                    | UiMeasurementEvidenceCategory::ScrollContainerViewport => {}
                }
            }
        }
    }

    if (query_extent.is_some() || saw_host_intrinsic)
        && measurement_basis.denial_posture().is_some()
    {
        return Err(intrinsic_denial(
            UiConstraintPropagationDenialReason::IncompatibleMeasurementPosture,
            neighborhood,
            measurement_basis,
        ));
    }

    match (query_extent, host_intrinsic) {
        (None, None) => Ok(None),
        (Some(primary_extent), None) => {
            if axis_scope != UiConstraintAxisScope::Primary {
                return Err(intrinsic_denial(
                    UiConstraintPropagationDenialReason::MissingRequiredIntrinsicContribution,
                    neighborhood,
                    measurement_basis,
                ));
            }
            Ok(Some(UiConstraintChildIntrinsicContribution::new(
                crate::evidence::UiConstraintChildIntrinsicContributionInput {
                    contributor_graph_node_identity,
                    axis_scope,
                    primary_extent,
                    cross_extent: None,
                    source_posture: UiConstraintIntrinsicSourcePosture::QueryOnly,
                    host_kind: UiConstraintHostIntrinsicKind::None,
                    unit_posture: UiMeasurementUnitPosture::LogicalPx,
                    coordinate_space: UiMeasurementCoordinateSpace::GraphNodeLocal,
                    rounding_posture: UiMeasurementRoundingPosture::ExactFloat,
                },
            )))
        }
        (None, Some(host)) => Ok(Some(UiConstraintChildIntrinsicContribution::new(
            crate::evidence::UiConstraintChildIntrinsicContributionInput {
                contributor_graph_node_identity,
                axis_scope,
                primary_extent: host.primary_extent,
                cross_extent: host.cross_extent,
                source_posture: UiConstraintIntrinsicSourcePosture::HostOnly,
                host_kind: host.host_kind,
                unit_posture: host.unit_posture,
                coordinate_space: host.coordinate_space,
                rounding_posture: host.rounding_posture,
            },
        ))),
        (Some(primary_extent), Some(host)) => {
            if host.primary_extent != primary_extent
                || host.unit_posture != UiMeasurementUnitPosture::LogicalPx
                || host.coordinate_space != UiMeasurementCoordinateSpace::GraphNodeLocal
                || host.rounding_posture != UiMeasurementRoundingPosture::ExactFloat
            {
                return Err(intrinsic_denial(
                    UiConstraintPropagationDenialReason::IncompatibleMeasurementPosture,
                    neighborhood,
                    measurement_basis,
                ));
            }
            Ok(Some(UiConstraintChildIntrinsicContribution::new(
                crate::evidence::UiConstraintChildIntrinsicContributionInput {
                    contributor_graph_node_identity,
                    axis_scope,
                    primary_extent,
                    cross_extent: host.cross_extent,
                    source_posture: UiConstraintIntrinsicSourcePosture::QueryAndHost,
                    host_kind: host.host_kind,
                    unit_posture: host.unit_posture,
                    coordinate_space: host.coordinate_space,
                    rounding_posture: host.rounding_posture,
                },
            )))
        }
    }
}

fn merge_host_intrinsic(
    existing: Option<AdmittedIntrinsicValue>,
    result: &UiMeasurementResult,
    axis_scope: UiConstraintAxisScope,
    primary_axis: UiLayoutOperatorPrimaryAxis,
) -> Result<Option<AdmittedIntrinsicValue>, UiConstraintPropagationDenialReason> {
    let admitted = admitted_host_intrinsic(result, axis_scope, primary_axis)?;
    Ok(Some(match existing {
        None => admitted,
        Some(existing)
            if existing.primary_extent == admitted.primary_extent
                && existing.cross_extent == admitted.cross_extent
                && existing.unit_posture == admitted.unit_posture
                && existing.coordinate_space == admitted.coordinate_space
                && existing.rounding_posture == admitted.rounding_posture =>
        {
            AdmittedIntrinsicValue {
                host_kind: merge_host_kind(existing.host_kind, admitted.host_kind),
                ..existing
            }
        }
        Some(_) => return Err(UiConstraintPropagationDenialReason::IncompatibleMeasurementPosture),
    }))
}

fn admitted_host_intrinsic(
    result: &UiMeasurementResult,
    axis_scope: UiConstraintAxisScope,
    primary_axis: UiLayoutOperatorPrimaryAxis,
) -> Result<AdmittedIntrinsicValue, UiConstraintPropagationDenialReason> {
    let (width, height, host_kind) = match result.value() {
        UiMeasurementValue::TextIntrinsicSize(value) => (
            value.width,
            value.height,
            UiConstraintHostIntrinsicKind::Text,
        ),
        UiMeasurementValue::NativeControlIntrinsicSize(value) => (
            value.width,
            value.height,
            UiConstraintHostIntrinsicKind::NativeControl,
        ),
        UiMeasurementValue::TextBaselineMetrics(_)
        | UiMeasurementValue::FontMetrics(_)
        | UiMeasurementValue::ViewportExtent(_)
        | UiMeasurementValue::DpiScaleFactor(_)
        | UiMeasurementValue::PortalAnchorRect(_)
        | UiMeasurementValue::ScrollContainerViewport(_) => {
            return Err(UiConstraintPropagationDenialReason::MissingRequiredIntrinsicContribution)
        }
    };
    let (primary_extent, cross_extent) =
        extents_for_axis_scope(width, height, axis_scope, primary_axis)?;
    Ok(AdmittedIntrinsicValue {
        primary_extent,
        cross_extent,
        unit_posture: result.unit_posture(),
        coordinate_space: result.coordinate_space(),
        rounding_posture: result.rounding_posture(),
        host_kind,
    })
}

fn extents_for_axis_scope(
    width: f32,
    height: f32,
    axis_scope: UiConstraintAxisScope,
    primary_axis: UiLayoutOperatorPrimaryAxis,
) -> Result<(CanonicalF32, Option<CanonicalF32>), UiConstraintPropagationDenialReason> {
    let (primary, cross) = match primary_axis {
        UiLayoutOperatorPrimaryAxis::Vertical => (height, Some(width)),
        UiLayoutOperatorPrimaryAxis::Horizontal => (width, Some(height)),
        UiLayoutOperatorPrimaryAxis::TwoDimensional | UiLayoutOperatorPrimaryAxis::Layered => {
            (height, Some(width))
        }
        UiLayoutOperatorPrimaryAxis::None => {
            return Err(UiConstraintPropagationDenialReason::MissingRequiredIntrinsicContribution);
        }
    };

    Ok(match axis_scope {
        UiConstraintAxisScope::Primary => (CanonicalF32::from_f32(primary), None),
        UiConstraintAxisScope::Cross => {
            let cross = cross
                .ok_or(UiConstraintPropagationDenialReason::MissingRequiredIntrinsicContribution)?;
            (CanonicalF32::from_f32(cross), None)
        }
        UiConstraintAxisScope::Both => {
            let cross = cross
                .ok_or(UiConstraintPropagationDenialReason::MissingRequiredIntrinsicContribution)?;
            (
                CanonicalF32::from_f32(primary),
                Some(CanonicalF32::from_f32(cross)),
            )
        }
    })
}

fn merge_host_kind(
    left: UiConstraintHostIntrinsicKind,
    right: UiConstraintHostIntrinsicKind,
) -> UiConstraintHostIntrinsicKind {
    match (left, right) {
        (UiConstraintHostIntrinsicKind::Text, UiConstraintHostIntrinsicKind::Text) => {
            UiConstraintHostIntrinsicKind::Text
        }
        (
            UiConstraintHostIntrinsicKind::NativeControl,
            UiConstraintHostIntrinsicKind::NativeControl,
        ) => UiConstraintHostIntrinsicKind::NativeControl,
        _ => UiConstraintHostIntrinsicKind::Mixed,
    }
}

fn intrinsic_denial(
    reason: UiConstraintPropagationDenialReason,
    neighborhood: &UiAllocationNeighborhood,
    measurement_basis: &UiMeasurementBasis,
) -> UiConstraintPropagationDenial {
    UiConstraintPropagationDenial::new(
        reason,
        neighborhood.identity().identity_digest(),
        neighborhood.layout_operator_contract_identity_digest(),
        Some(UiConstraintPropagationEdgeFamily::ChildIntrinsicContribution),
        measurement_basis.identity_digest()
            ^ stable_text_digest("worth-ui.constraint-child-intrinsic-denial").rotate_left(7),
    )
}
