use super::*;
use worth_ui_host_contract::UiMountedCanonicalBoxInput;

#[test]
fn stationary_hover_retests_a_new_target_that_now_overlaps_the_pointer() {
    let presentation = presentation_basis();
    let old_target = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let new_target = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let position = UiHostSurfacePosition::viewport_logical(10_000, 10_000);
    let trigger = UiPointerPresencePresentationTrigger::new_with_geometry(
        presentation,
        &[UiPointerPresenceGeometryCandidate::new(
            new_target,
            None,
            Some(geometry([8.0, 8.0, 8.0, 8.0])),
        )],
    )
    .unwrap();

    assert!(trigger.affects_position(position, Some(old_target)));
}

#[test]
fn a_moved_old_target_retests_stationary_hover_inside_its_old_geometry() {
    let presentation = presentation_basis();
    let target = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let position = UiHostSurfacePosition::viewport_logical(10_000, 10_000);
    let trigger = UiPointerPresencePresentationTrigger::new_with_geometry(
        presentation,
        &[UiPointerPresenceGeometryCandidate::new(
            target,
            Some(geometry([8.0, 8.0, 8.0, 8.0])),
            Some(geometry([40.0, 40.0, 8.0, 8.0])),
        )],
    )
    .unwrap();

    assert!(trigger.affects_position(position, None));
}

#[test]
fn captured_press_retests_when_its_target_is_in_the_changed_candidate_set() {
    let presentation = presentation_basis();
    let target = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let position = UiHostSurfacePosition::viewport_logical(100_000, 100_000);
    let trigger = UiPointerPresencePresentationTrigger::new_with_geometry(
        presentation,
        &[UiPointerPresenceGeometryCandidate::identity_only(target)],
    )
    .unwrap();

    assert!(trigger.affects_position(position, Some(target)));
}

#[test]
fn geometry_candidates_are_canonicalized_and_bounded_before_storage() {
    let presentation = presentation_basis();
    let instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let trigger = UiPointerPresencePresentationTrigger::new_with_geometry(
        presentation,
        &[
            UiPointerPresenceGeometryCandidate::new(
                instance,
                Some(geometry([0.0, 0.0, 2.0, 2.0])),
                None,
            ),
            UiPointerPresenceGeometryCandidate::new(
                instance,
                None,
                Some(geometry([2.0, 2.0, 2.0, 2.0])),
            ),
        ],
    )
    .unwrap();
    assert_eq!(trigger.changed_instances(), &[instance]);
    assert!(trigger.geometry_candidates[0].old().is_some());
    assert!(trigger.geometry_candidates[0].new_geometry().is_some());
}

fn geometry(components: [f32; 4]) -> UiPointerPresenceGeometry {
    let bounds = UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x: components[0],
        y: components[1],
        width: components[2],
        height: components[3],
        coordinate_space: UiMountedCoordinateSpace::Viewport,
    })
    .unwrap();
    UiPointerPresenceGeometry::new(bounds, bounds)
}

fn presentation_basis() -> UiHostObservationPresentationBasis {
    UiHostObservationPresentationBasis::new(
        worth_ui_host_contract::UiHostSurfaceIdentity::mint_unbound().unwrap(),
        worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap(),
        worth_ui_host_contract::UiSurfaceBindingGeneration::mint_unbound().unwrap(),
        worth_ui_host_contract::UiHostPresentationEpoch::issued_by_host(1),
    )
}
