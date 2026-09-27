use super::extent_units::fractional_remainder_required;
use super::{posture_for_equal_share, remainder_rank_for};
use crate::evidence::measurement::projection::fact_test_support::{
    capability_report, host_result_viewport_extent_with_value, synthetic_declaration_identity,
};
use crate::evidence::{
    admit_measurement_basis, MeasurementEvidenceInput, UiAllocationConstraintSummary,
    UiAllocationConstraintSummaryInput, UiConstraintAvailableSpacePosture, UiConstraintAxisScope,
    UiConstraintBoundedMinMaxRequirement, UiConstraintEqualShareDistributionPolicy,
    UiConstraintEqualShareGroup, UiConstraintEqualSharePosture,
    UiConstraintResizePermissionPosture, UiConstraintSiblingNegotiationMode,
    UiConstraintSpecialInputPosture, UiLayoutOperatorPrimaryAxis,
};
use crate::graph::allocation_constraint_equal_share_test_support::{
    graph_node_identity_for_provenance, peer_app, viewport_basis_policy,
};
use worth_ui_inspection::UiEvidenceAuthorityGeneration;

#[test]
fn admitted_zero_available_space_maps_to_typed_equal_share_posture() {
    let summary = UiAllocationConstraintSummary::new(UiAllocationConstraintSummaryInput {
        incoming_available_space: Some(UiConstraintAxisScope::Both),
        incoming_available_space_posture: Some(
            UiConstraintAvailableSpacePosture::AdmittedZeroExtent,
        ),
        intrinsic_contribution_requirements: Some(UiConstraintAxisScope::Both),
        sibling_negotiation_mode: UiConstraintSiblingNegotiationMode::StablePeerTwoDimensional,
        equal_share_group: UiConstraintEqualShareGroup::StablePeerTwoDimensional,
        bounded_min_max_requirements: UiConstraintBoundedMinMaxRequirement::BothAxes,
        viewport_requirement: UiConstraintSpecialInputPosture::NotRequired,
        scroll_owner_requirement: UiConstraintSpecialInputPosture::NotRequired,
        portal_anchor_requirement: UiConstraintSpecialInputPosture::NotRequired,
        resize_permission_posture: UiConstraintResizePermissionPosture::None,
        unit_posture: None,
        coordinate_space: None,
        rounding_posture: None,
    });

    assert_eq!(
        posture_for_equal_share(summary, UiConstraintAxisScope::Both, 2),
        UiConstraintEqualSharePosture::ZeroAvailableSpace
    );
}

#[test]
fn a_remainder_rank_past_u16_is_refused_rather_than_wrapped() {
    let left_to_right =
        UiConstraintEqualShareDistributionPolicy::DeterministicRemainderLeftToRightByStablePeerIdentity;
    assert_eq!(
        remainder_rank_for(left_to_right, 65_537, 65_535),
        Ok(Some(u16::MAX))
    );
    assert!(remainder_rank_for(left_to_right, 65_537, 65_536).is_err());

    let center_out =
        UiConstraintEqualShareDistributionPolicy::DeterministicRemainderCenterOutByStablePeerIdentity;
    // Three peers: the center ranks first, then each neighbor by its distance.
    assert_eq!(remainder_rank_for(center_out, 3, 1), Ok(Some(1)));
    assert_eq!(remainder_rank_for(center_out, 3, 0), Ok(Some(3)));
    assert_eq!(remainder_rank_for(center_out, 3, 2), Ok(Some(5)));
    // An edge peer 499 from the center of 1,000 ranks past what u16 counts.
    assert!(remainder_rank_for(center_out, 1_000, 0).is_err());

    assert_eq!(
        remainder_rank_for(
            UiConstraintEqualShareDistributionPolicy::ExactFractional,
            1_000,
            0
        ),
        Ok(None)
    );
}

#[test]
fn a_fractional_extent_needs_a_remainder_rather_than_reading_as_its_whole_part() {
    let (_, _, world_profile) =
        crate::evidence::measurement::projection::fact_test_support::display_field_projection_context(
            "constraint-equal-share-fractional-extent",
        );
    let app = peer_app(
        world_profile.clone(),
        "operator:grid",
        &[false, false, false],
    );
    let root = graph_node_identity_for_provenance(&app, 0);
    let report = capability_report(88);
    let measuring = |width: f32| {
        admit_measurement_basis(
            synthetic_declaration_identity("constraint-equal-share-fractional-extent"),
            root,
            world_profile.clone(),
            UiEvidenceAuthorityGeneration::new(88),
            &viewport_basis_policy(false),
            &[
                MeasurementEvidenceInput::host_capability_report(&report),
                MeasurementEvidenceInput::host_measurement_result(
                    &host_result_viewport_extent_with_value(
                        880,
                        &report,
                        UiEvidenceAuthorityGeneration::new(88),
                        width,
                        10.0,
                    ),
                ),
            ],
        )
    };
    for (width, required) in [(4.0, false), (4.5, true), (5.0, true)] {
        assert_eq!(
            fractional_remainder_required(
                &measuring(width),
                UiConstraintAxisScope::Primary,
                UiLayoutOperatorPrimaryAxis::Horizontal,
                2,
            ),
            required,
            "{width} points shared by two peers"
        );
    }
}
