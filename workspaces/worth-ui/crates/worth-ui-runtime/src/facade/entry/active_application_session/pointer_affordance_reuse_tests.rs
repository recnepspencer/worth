use crate::mounting::{UiMountedFrameRequest, UiMountedFrameReuse};
use worth_ui_host_contract::*;

#[test]
fn mounted_pointer_only_posture_invalidates_exact_frame_reuse() {
    let (mut session, host, surfaces) = super::mounted_world();
    super::motion(
        &mut session,
        surfaces[0],
        1,
        1,
        UiHostPointerDeviceKind::Mouse,
        false,
    );
    let request = UiMountedFrameRequest::all_bound_surfaces();
    let frame = session
        .prepare_mounted_frame_with_application_presentation(request.clone(), |_| {})
        .unwrap_or_else(|_| panic!("pointer reuse predecessor must prepare"));
    super::super::publish(&mut session, &host, frame, 2);
    assert!(matches!(
        classify(&mut session, &request),
        UiMountedFrameReuse::Exact(_)
    ));

    session
        .update_intent_boolean_fact(
            &super::super::fixture::fact(super::super::fixture::MUTABLE),
            false,
        )
        .unwrap();
    super::close_source(&mut session, "pointer-reuse-readonly");
    assert!(matches!(
        classify(&mut session, &request),
        UiMountedFrameReuse::ComparisonRequired(_)
    ));
    for _ in &surfaces {
        host.push_native_display_settled_without_effects();
    }
    let outcome = session
        .execute_mounted_frame(
            request.clone(),
            UiPresentationDeadline::at_tick(100),
            3,
            |_| {},
        )
        .unwrap_or_else(|_| panic!("changed pointer must execute"));
    assert!(matches!(
        outcome,
        crate::mounting::UiMountedFrameOutcome::Published(_)
            | crate::mounting::UiMountedFrameOutcome::Unchanged(_)
    ));
    let output = session
        .mounted
        .current_unpublished_appearance()
        .unwrap()
        .unwrap();
    let [UiMountedAppearanceMechanic::Pointer(pointer)] =
        output.fragments()[0].work().successor().mechanics()
    else {
        panic!("changed pointer mechanic must survive execution");
    };
    assert_eq!(pointer.family(), UiPointerAffordanceFamily::Default);
    assert!(matches!(
        classify(&mut session, &request),
        UiMountedFrameReuse::Exact(_)
    ));
    super::close_source(&mut session, "pointer-reuse-unchanged");
    assert!(
        matches!(
            classify(&mut session, &request),
            UiMountedFrameReuse::Exact(_)
        ),
        "receipt and observation churn must preserve exact pointer dependencies"
    );
    assert!(matches!(
        session
            .execute_mounted_frame(request, UiPresentationDeadline::at_tick(100), 4, |_| {})
            .unwrap_or_else(|_| panic!("equal snapshot must reuse without host work")),
        crate::mounting::UiMountedFrameOutcome::Unchanged(_)
    ));
    session.advance_mounted_identity_frame().unwrap();
    let successor = session
        .prepare_mounted_frame_with_application_presentation(
            UiMountedFrameRequest::all_bound_surfaces(),
            |_| {},
        )
        .unwrap_or_else(|_| {
            panic!("exact reuse must preserve admitted snapshot across receipt succession")
        });
    successor.assert_no_unpublished_appearance_for_test();
    let _ = session.shutdown();
}

fn classify(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    request: &UiMountedFrameRequest,
) -> UiMountedFrameReuse {
    session
        .execute_framework_turn(|_| {})
        .unwrap()
        .into_execution()
        .unwrap_or_else(|_| panic!("reuse classification requires a ready execution"))
        .classify_mounted_frame_reuse_internal(request)
}

#[test]
fn mounted_pointer_departure_does_not_reauthorize_old_observation() {
    let (mut session, host, surfaces) = super::mounted_world();
    super::motion(
        &mut session,
        surfaces[0],
        1,
        1,
        UiHostPointerDeviceKind::Mouse,
        false,
    );
    let admitted = session.pointer_affordance_snapshot.clone().unwrap();
    let target = admitted.projections()[0].target().unwrap();
    let initial = session
        .prepare_mounted_frame_with_application_presentation(
            UiMountedFrameRequest::all_bound_surfaces(),
            |_| {},
        )
        .unwrap_or_else(|_| panic!("pointer predecessor must prepare"));
    super::super::publish(&mut session, &host, initial, 2);
    super::motion(
        &mut session,
        surfaces[0],
        2,
        1,
        UiHostPointerDeviceKind::Mouse,
        true,
    );
    let departure = session
        .prepare_mounted_frame_with_application_presentation(
            UiMountedFrameRequest::all_bound_surfaces(),
            |_| {},
        )
        .unwrap_or_else(|_| panic!("pointer departure must prepare"));
    assert!(departure
        .lower_unpublished_appearance_for_test()
        .fragments()[0]
        .work()
        .successor()
        .mechanics()
        .is_empty());
    super::super::publish(&mut session, &host, departure, 3);
    let mut successor = session
        .prepare_mounted_frame_with_application_presentation(
            UiMountedFrameRequest::all_bound_surfaces(),
            |_| {},
        )
        .unwrap_or_else(|_| panic!("empty pointer successor must prepare"));
    assert_eq!(
        successor.stage_pointer_affordance(Some(&admitted), &session.mounted),
        Err(
            crate::mounting::UiMountedFramePreparationDenial::PointerSnapshotTargetUnavailable(
                target
            )
        )
    );
    successor.assert_no_unpublished_appearance_for_test();
    let _ = session.shutdown();
}
