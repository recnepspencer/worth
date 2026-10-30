use std::collections::BTreeMap;

use crate::declaration::stable_text_digest;
use crate::declaration::UiDeclaredMeasurementConstraintModifier;
use crate::evidence::{
    UiAllocationNeighborhood, UiAllocationNeighborhoodMemberRole, UiConstraintAxisScope,
    UiConstraintChildIntrinsicContribution, UiConstraintPropagationDenial,
    UiConstraintPropagationDenialReason, UiConstraintPropagationEdge,
    UiConstraintPropagationEdgeFamily, UiConstraintPropagationEdgePayload,
    UiConstraintResizeInputPosture, UiLayoutOperatorCrossAxis, UiLayoutOperatorPlanningContract,
};

use super::UiSiblingNegotiationPeerContract;
pub(super) fn admit_peer_contracts(
    neighborhood: &UiAllocationNeighborhood,
    peers: &[&crate::evidence::UiAllocationNeighborhoodMember],
    intrinsic_by_member: &BTreeMap<u64, UiConstraintChildIntrinsicContribution>,
    axis_scope: UiConstraintAxisScope,
) -> Result<Vec<UiSiblingNegotiationPeerContract>, UiConstraintPropagationDenial> {
    let contract = neighborhood.layout_operator_planning_contract();
    let mut peer_contracts = Vec::with_capacity(peers.len());
    for member in peers {
        let intrinsic_axis_scope = intrinsic_by_member
            .get(&member.identity_digest())
            .map(UiConstraintChildIntrinsicContribution::axis_scope);
        let bounded_axis_scope =
            peer_bounded_axis_scope(member.measurement_constraint_modifier(), contract);
        if bounded_axis_scope.is_some() && intrinsic_axis_scope.is_none() {
            return Err(sibling_denial(
                UiConstraintPropagationDenialReason::ContradictorySiblingRequirements,
                member.identity_digest()
                    ^ stable_text_digest(
                        "worth-ui.constraint-sibling-negotiation.missing-bounded-intrinsic",
                    ),
                neighborhood,
            ));
        }
        if let Some(intrinsic_axis_scope) = intrinsic_axis_scope
            && intrinsic_axis_scope != axis_scope
        {
            return Err(sibling_denial(
                UiConstraintPropagationDenialReason::ContradictorySiblingRequirements,
                member.identity_digest()
                    ^ stable_text_digest(
                        "worth-ui.constraint-sibling-negotiation.intrinsic-axis-contract-mismatch",
                    ),
                neighborhood,
            ));
        }
        peer_contracts.push(UiSiblingNegotiationPeerContract {
            identity_digest: member.identity_digest(),
            bounded_axis_scope,
            intrinsic_axis_scope,
        });
    }
    Ok(peer_contracts)
}

pub(super) fn peer_bounded_axis_scope(
    child_constraint_modifier: Option<UiDeclaredMeasurementConstraintModifier>,
    contract: &UiLayoutOperatorPlanningContract,
) -> Option<UiConstraintAxisScope> {
    if child_constraint_modifier != Some(UiDeclaredMeasurementConstraintModifier::Bounded)
        || contract.constraint_modifier() != Some(UiDeclaredMeasurementConstraintModifier::Bounded)
    {
        return None;
    }

    Some(match contract.semantics().cross_axis() {
        UiLayoutOperatorCrossAxis::Horizontal | UiLayoutOperatorCrossAxis::Vertical => {
            UiConstraintAxisScope::Primary
        }
        UiLayoutOperatorCrossAxis::TwoDimensional
        | UiLayoutOperatorCrossAxis::Layered
        | UiLayoutOperatorCrossAxis::None => UiConstraintAxisScope::Both,
    })
}

pub(super) fn has_admitted_durable_resize_support(
    admitted_edges: &[UiConstraintPropagationEdge],
    axis_scope: UiConstraintAxisScope,
) -> bool {
    admitted_edges.iter().any(|edge| {
        matches!(
            edge.payload(),
            UiConstraintPropagationEdgePayload::DurableResizeInput {
                durable_identity_digest: _,
                axis_scope: edge_axis_scope,
                posture: UiConstraintResizeInputPosture::DurableAuthorityRequired,
                planning_time_only: true,
            } if edge_axis_scope == axis_scope
        )
    })
}

pub(super) fn admitted_intrinsic_by_member(
    neighborhood: &UiAllocationNeighborhood,
    intrinsic_edges: &[UiConstraintPropagationEdge],
    axis_scope: UiConstraintAxisScope,
) -> Result<BTreeMap<u64, UiConstraintChildIntrinsicContribution>, UiConstraintPropagationDenial> {
    let mut intrinsic_by_member = BTreeMap::new();
    let mut peer_graph_digests = neighborhood
        .members()
        .iter()
        .filter(|member| !matches!(member.role(), UiAllocationNeighborhoodMemberRole::Root))
        .map(|member| {
            (
                member.identity_digest(),
                member.graph_node_identity().digest(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    for edge in intrinsic_edges {
        let Some(contribution) = edge.payload().child_intrinsic_contribution() else {
            continue;
        };
        if contribution.axis_scope() != axis_scope {
            return Err(sibling_denial(
                UiConstraintPropagationDenialReason::ContradictorySiblingRequirements,
                contribution.identity_digest()
                    ^ stable_text_digest("worth-ui.constraint-sibling-negotiation.axis-scope"),
                neighborhood,
            ));
        }
        let member_identity_digest = edge.source_member_identity_digest();
        let Some(expected_graph_digest) = peer_graph_digests.remove(&member_identity_digest) else {
            return Err(sibling_denial(
                UiConstraintPropagationDenialReason::ContradictorySiblingRequirements,
                contribution.identity_digest()
                    ^ stable_text_digest("worth-ui.constraint-sibling-negotiation.unknown-peer"),
                neighborhood,
            ));
        };
        if contribution.contributor_graph_node_identity().digest() != expected_graph_digest {
            return Err(sibling_denial(
                UiConstraintPropagationDenialReason::ContradictorySiblingRequirements,
                contribution.identity_digest()
                    ^ stable_text_digest("worth-ui.constraint-sibling-negotiation.peer-mismatch"),
                neighborhood,
            ));
        }
        if intrinsic_by_member
            .insert(member_identity_digest, contribution)
            .is_some()
        {
            return Err(sibling_denial(
                UiConstraintPropagationDenialReason::ContradictorySiblingRequirements,
                contribution.identity_digest()
                    ^ stable_text_digest("worth-ui.constraint-sibling-negotiation.duplicate-peer"),
                neighborhood,
            ));
        }
    }
    Ok(intrinsic_by_member)
}

pub(super) fn shared_normalization(
    neighborhood: &UiAllocationNeighborhood,
    contributions: impl Iterator<Item = UiConstraintChildIntrinsicContribution>,
) -> Result<
    Option<(
        crate::evidence::UiMeasurementUnitPosture,
        crate::evidence::UiMeasurementCoordinateSpace,
        crate::evidence::UiMeasurementRoundingPosture,
    )>,
    UiConstraintPropagationDenial,
> {
    let mut normalization = None;
    for contribution in contributions {
        let current = (
            contribution.unit_posture(),
            contribution.coordinate_space(),
            contribution.rounding_posture(),
        );
        match normalization {
            None => normalization = Some(current),
            Some(existing) if existing == current => {}
            Some(_) => {
                return Err(sibling_denial(
                    UiConstraintPropagationDenialReason::ContradictorySiblingRequirements,
                    contribution.identity_digest()
                        ^ stable_text_digest(
                            "worth-ui.constraint-sibling-negotiation.normalization-mismatch",
                        ),
                    neighborhood,
                ));
            }
        }
    }
    Ok(normalization)
}

pub(super) fn sibling_denial(
    reason: UiConstraintPropagationDenialReason,
    witness_digest: u64,
    neighborhood: &UiAllocationNeighborhood,
) -> UiConstraintPropagationDenial {
    UiConstraintPropagationDenial::new(
        reason,
        neighborhood.identity().identity_digest(),
        neighborhood.layout_operator_contract_identity_digest(),
        Some(UiConstraintPropagationEdgeFamily::SiblingNegotiation),
        witness_digest,
    )
}
