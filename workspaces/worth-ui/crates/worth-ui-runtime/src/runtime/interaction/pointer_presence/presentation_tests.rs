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
    let old_geometry = geometry([0.0, 0.0, 2.0, 2.0]);
    let new_geometry = geometry([2.0, 2.0, 2.0, 2.0]);
    let trigger = UiPointerPresencePresentationTrigger::new_with_geometry(
        presentation,
        &[
            UiPointerPresenceGeometryCandidate::new(instance, Some(old_geometry), None),
            UiPointerPresenceGeometryCandidate::new(instance, None, Some(new_geometry)),
        ],
    )
    .unwrap();
    assert_eq!(trigger.changed_instances(), &[instance]);
    assert_eq!(trigger.geometry_candidates[0].old(), Some(old_geometry));
    assert_eq!(
        trigger.geometry_candidates[0].new_geometry(),
        Some(new_geometry)
    );
}

#[test]
fn raw_input_capacity_reports_bounded_overflow_facts() {
    let presentation = presentation_basis();
    let duplicate = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let duplicate_rows = (0..=UI_POINTER_PRESENTATION_CHANGED_INSTANCE_CAPACITY)
        .map(|_| UiPointerPresenceGeometryCandidate::identity_only(duplicate))
        .collect::<Vec<_>>();
    assert_eq!(
        UiPointerPresencePresentationTrigger::new_with_geometry(presentation, &duplicate_rows),
        Err(
            UiPointerPresencePresentationTriggerDenial::ChangedNeighborhoodCapacityExceeded {
                observed: UI_POINTER_PRESENTATION_CHANGED_INSTANCE_CAPACITY + 1,
                maximum: UI_POINTER_PRESENTATION_CHANGED_INSTANCE_CAPACITY,
            }
        )
    );

    let changed = (0..=UI_POINTER_PRESENTATION_CHANGED_INSTANCE_CAPACITY)
        .map(|_| UiMountedInstanceIdentity::mint_unbound().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        UiPointerPresencePresentationTrigger::new(presentation, &changed),
        Err(
            UiPointerPresencePresentationTriggerDenial::ChangedNeighborhoodCapacityExceeded {
                observed: UI_POINTER_PRESENTATION_CHANGED_INSTANCE_CAPACITY + 1,
                maximum: UI_POINTER_PRESENTATION_CHANGED_INSTANCE_CAPACITY,
            }
        )
    );
}

#[test]
fn geometry_canonicalization_is_sorted_and_bounded_at_raw_capacity() {
    let presentation = presentation_basis();
    let mut candidates = (0..UI_POINTER_PRESENTATION_CHANGED_INSTANCE_CAPACITY)
        .map(|_| {
            UiPointerPresenceGeometryCandidate::identity_only(
                UiMountedInstanceIdentity::mint_unbound().unwrap(),
            )
        })
        .collect::<Vec<_>>();
    candidates.reverse();

    let trigger =
        UiPointerPresencePresentationTrigger::new_with_geometry(presentation, &candidates).unwrap();

    assert_eq!(
        trigger.geometry_candidates.len(),
        UI_POINTER_PRESENTATION_CHANGED_INSTANCE_CAPACITY
    );
    assert!(trigger
        .geometry_candidates
        .windows(2)
        .all(|window| window[0].instance() < window[1].instance()));
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
