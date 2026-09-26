use crate::evidence::{
    UiAllocationConstraintSummary, UiAllocationConstraintSummaryInput,
    UiConstraintAvailableSpacePosture, UiConstraintAxisScope, UiConstraintBoundedMinMaxRequirement,
    UiConstraintEqualShareGroup, UiConstraintNormalizationPosture,
    UiConstraintPropagationEdgeFamily, UiConstraintResizePermissionPosture,
    UiConstraintSiblingNegotiationMode, UiConstraintSpecialInputPosture,
    UiLayoutOperatorChildParticipationRule, UiMeasurementBasis, UiMeasurementDependencyLineageKind,
};

pub(super) fn derive_constraint_summary(
    incoming_available_space: Option<UiConstraintAxisScope>,
    incoming_available_space_posture: Option<UiConstraintAvailableSpacePosture>,
    bounded_min_max_requirement: UiConstraintBoundedMinMaxRequirement,
    normalization_posture: UiConstraintNormalizationPosture,
    child_participation_rule: UiLayoutOperatorChildParticipationRule,
    allowed_families: &[UiConstraintPropagationEdgeFamily],
    required_special_families: &[UiConstraintPropagationEdgeFamily],
) -> UiAllocationConstraintSummary {
    UiAllocationConstraintSummary::new(UiAllocationConstraintSummaryInput {
        incoming_available_space,
        incoming_available_space_posture,
        intrinsic_contribution_requirements: intrinsic_contribution_scope(child_participation_rule),
        sibling_negotiation_mode: sibling_negotiation_mode(child_participation_rule),
        equal_share_group: equal_share_group(child_participation_rule, allowed_families),
        bounded_min_max_requirements: bounded_min_max_requirement,
        viewport_requirement: special_input_posture(
            required_special_families.contains(&UiConstraintPropagationEdgeFamily::ViewportInput),
        ),
        scroll_owner_requirement: special_input_posture(
            required_special_families
                .contains(&UiConstraintPropagationEdgeFamily::ScrollViewportInput),
        ),
        portal_anchor_requirement: special_input_posture(
            required_special_families
                .contains(&UiConstraintPropagationEdgeFamily::PortalAnchorInput),
        ),
        resize_permission_posture: resize_permission_posture(allowed_families),
        unit_posture: normalization_posture.unit_posture(),
        coordinate_space: normalization_posture.coordinate_space(),
        rounding_posture: normalization_posture.rounding_posture(),
    })
}

pub(super) fn intrinsic_contribution_scope(
    rule: UiLayoutOperatorChildParticipationRule,
) -> Option<UiConstraintAxisScope> {
    match rule {
        UiLayoutOperatorChildParticipationRule::None => None,
        UiLayoutOperatorChildParticipationRule::VerticalPeerFlow
        | UiLayoutOperatorChildParticipationRule::HorizontalPeerFlow
        | UiLayoutOperatorChildParticipationRule::SplitPanels => {
            Some(UiConstraintAxisScope::Primary)
        }
        UiLayoutOperatorChildParticipationRule::RootViewportFrame
        | UiLayoutOperatorChildParticipationRule::GridCellPeers
        | UiLayoutOperatorChildParticipationRule::MosaicTiles
        | UiLayoutOperatorChildParticipationRule::OverlayLayers
        | UiLayoutOperatorChildParticipationRule::ScrollContent
        | UiLayoutOperatorChildParticipationRule::PortalAnchoredContent
        | UiLayoutOperatorChildParticipationRule::ContainerChildren => {
            Some(UiConstraintAxisScope::Both)
        }
    }
}

pub(super) fn sibling_negotiation_mode(
    rule: UiLayoutOperatorChildParticipationRule,
) -> UiConstraintSiblingNegotiationMode {
    match rule {
        UiLayoutOperatorChildParticipationRule::VerticalPeerFlow
        | UiLayoutOperatorChildParticipationRule::HorizontalPeerFlow
        | UiLayoutOperatorChildParticipationRule::SplitPanels => {
            UiConstraintSiblingNegotiationMode::StablePeerPrimaryAxis
        }
        UiLayoutOperatorChildParticipationRule::GridCellPeers
        | UiLayoutOperatorChildParticipationRule::MosaicTiles => {
            UiConstraintSiblingNegotiationMode::StablePeerTwoDimensional
        }
        UiLayoutOperatorChildParticipationRule::None
        | UiLayoutOperatorChildParticipationRule::RootViewportFrame
        | UiLayoutOperatorChildParticipationRule::OverlayLayers
        | UiLayoutOperatorChildParticipationRule::ScrollContent
        | UiLayoutOperatorChildParticipationRule::PortalAnchoredContent
        | UiLayoutOperatorChildParticipationRule::ContainerChildren => {
            UiConstraintSiblingNegotiationMode::None
        }
    }
}

pub(super) fn equal_share_group(
    rule: UiLayoutOperatorChildParticipationRule,
    allowed_families: &[UiConstraintPropagationEdgeFamily],
) -> UiConstraintEqualShareGroup {
    if !allowed_families.contains(&UiConstraintPropagationEdgeFamily::EqualShareDistribution) {
        return UiConstraintEqualShareGroup::None;
    }

    match rule {
        UiLayoutOperatorChildParticipationRule::GridCellPeers
        | UiLayoutOperatorChildParticipationRule::MosaicTiles => {
            UiConstraintEqualShareGroup::StablePeerTwoDimensional
        }
        UiLayoutOperatorChildParticipationRule::SplitPanels => {
            UiConstraintEqualShareGroup::StablePeerPrimaryAxis
        }
        UiLayoutOperatorChildParticipationRule::None
        | UiLayoutOperatorChildParticipationRule::RootViewportFrame
        | UiLayoutOperatorChildParticipationRule::VerticalPeerFlow
        | UiLayoutOperatorChildParticipationRule::HorizontalPeerFlow
        | UiLayoutOperatorChildParticipationRule::OverlayLayers
        | UiLayoutOperatorChildParticipationRule::ScrollContent
        | UiLayoutOperatorChildParticipationRule::PortalAnchoredContent
        | UiLayoutOperatorChildParticipationRule::ContainerChildren => {
            UiConstraintEqualShareGroup::None
        }
    }
}

pub(super) fn special_input_posture(required: bool) -> UiConstraintSpecialInputPosture {
    if required {
        UiConstraintSpecialInputPosture::Required
    } else {
        UiConstraintSpecialInputPosture::NotRequired
    }
}

pub(super) fn resize_permission_posture(
    allowed_families: &[UiConstraintPropagationEdgeFamily],
) -> UiConstraintResizePermissionPosture {
    if allowed_families.contains(&UiConstraintPropagationEdgeFamily::DurableResizeInput) {
        UiConstraintResizePermissionPosture::DurableAuthorityLane
    } else {
        UiConstraintResizePermissionPosture::None
    }
}

pub(super) fn axis_scope_for_sibling_mode(
    mode: UiConstraintSiblingNegotiationMode,
) -> Option<UiConstraintAxisScope> {
    match mode {
        UiConstraintSiblingNegotiationMode::None => None,
        UiConstraintSiblingNegotiationMode::StablePeerPrimaryAxis => {
            Some(UiConstraintAxisScope::Primary)
        }
        UiConstraintSiblingNegotiationMode::StablePeerTwoDimensional => {
            Some(UiConstraintAxisScope::Both)
        }
    }
}

pub(super) fn special_input_families_from_basis(
    measurement_basis: &UiMeasurementBasis,
) -> Vec<UiConstraintPropagationEdgeFamily> {
    let mut families = measurement_basis
        .dependency_map()
        .entries()
        .iter()
        .filter_map(|entry| match entry.lineage().kind() {
            UiMeasurementDependencyLineageKind::HostViewportExtent => {
                Some(UiConstraintPropagationEdgeFamily::ViewportInput)
            }
            UiMeasurementDependencyLineageKind::HostScrollContainerViewport => {
                Some(UiConstraintPropagationEdgeFamily::ScrollViewportInput)
            }
            UiMeasurementDependencyLineageKind::HostPortalAnchorRect => {
                Some(UiConstraintPropagationEdgeFamily::PortalAnchorInput)
            }
            UiMeasurementDependencyLineageKind::QueryScrollContentExtent
            | UiMeasurementDependencyLineageKind::HostTextIntrinsicSize
            | UiMeasurementDependencyLineageKind::HostFontMetrics
            | UiMeasurementDependencyLineageKind::HostNativeControlIntrinsicSize => None,
        })
        .collect::<Vec<_>>();
    families.sort_unstable_by_key(|family| family.rank());
    families.dedup();
    families
}
