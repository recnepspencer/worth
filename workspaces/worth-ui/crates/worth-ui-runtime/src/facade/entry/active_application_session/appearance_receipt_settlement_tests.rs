use super::role_support::{radius_role, radius_value_from, update_radius_at_revision};
use super::MountedAppearanceFixture;

#[test]
fn published_appearance_attempt_settles_only_after_host_publication() {
    let mut fixture = settled_fixture();
    update_radius_at_revision(&mut fixture.session, [3; 4], 2);
    assert!(fixture
        .session
        .presentation
        .appearance_invalidation_batch()
        .is_some());
    fixture.host.push_native_display_presented();

    assert!(matches!(
        execute(&mut fixture, 3),
        crate::mounting::UiMountedFrameOutcome::Published(_)
    ));
    assert!(fixture
        .session
        .presentation
        .appearance_invalidation_batch()
        .is_none());
    super::query_support::shutdown(fixture.session);
}

#[test]
fn rejected_appearance_attempt_stays_inspectable_without_settlement() {
    let mut fixture = settled_fixture();
    update_radius_at_revision(&mut fixture.session, [3; 4], 2);
    fixture.host.push_rejected();

    assert!(matches!(
        execute(&mut fixture, 3),
        crate::mounting::UiMountedFrameOutcome::RejectedBeforeEffects(_)
    ));
    assert!(fixture
        .session
        .presentation
        .appearance_invalidation_batch()
        .is_some());
    let explanation =
        super::query_support::why_for(&fixture, worth_ui_dsl::UiAppearanceAspect::Radius);
    assert!(explanation.mounted_mechanical_output_changed());
    assert!(!explanation.denied_before_effects());
    assert_eq!(
        explanation.value(),
        worth_ui_inspection::UiAppearanceInspectionValue::Resolved(radius_value_from([2; 4]))
    );
    assert_eq!(
        explanation.mounted_mechanic(),
        worth_ui_inspection::UiAppearanceInspectionMountedMechanic::Changed
    );
    super::query_support::shutdown(fixture.session);
}

#[test]
fn in_flight_appearance_attempt_waits_and_keeps_newer_invalidation() {
    let mut fixture = settled_fixture();
    update_radius_at_revision(&mut fixture.session, [3; 4], 2);
    fixture.host.push_in_flight(
        vec![
            crate::certification_support::ScriptedSurfaceCompletion::Pending,
            native_completion(),
        ],
        worth_ui_host_contract::UiHostSurfaceCancellationOutcome::CancelledBeforeEffects,
    );

    let in_flight = match execute(&mut fixture, 3) {
        crate::mounting::UiMountedFrameOutcome::InFlight(in_flight) => in_flight,
        _ => panic!("expected an in-flight mounted appearance attempt"),
    };
    assert!(fixture
        .session
        .presentation
        .appearance_invalidation_batch()
        .is_some());
    update_radius_at_revision(&mut fixture.session, [4; 4], 3);
    assert!(fixture
        .session
        .presentation
        .appearance_invalidation_batch()
        .is_some());

    let in_flight = match fixture.session.complete_mounted_presentation(in_flight, 4) {
        crate::mounting::UiMountedFrameOutcome::InFlight(in_flight) => in_flight,
        _ => panic!("the first completion poll must remain in flight"),
    };
    assert!(fixture
        .session
        .presentation
        .appearance_invalidation_batch()
        .is_some());
    assert!(matches!(
        fixture.session.complete_mounted_presentation(in_flight, 5),
        crate::mounting::UiMountedFrameOutcome::Published(_)
    ));
    assert!(fixture
        .session
        .presentation
        .appearance_invalidation_batch()
        .is_some());
    fixture.host.push_native_display_presented();
    assert!(matches!(
        execute(&mut fixture, 5),
        crate::mounting::UiMountedFrameOutcome::Published(_)
    ));
    assert!(fixture
        .session
        .presentation
        .appearance_invalidation_batch()
        .is_none());
    super::query_support::shutdown(fixture.session);
}

#[test]
fn indeterminate_appearance_attempt_preserves_unsettled_invalidation() {
    let mut fixture = settled_fixture();
    update_radius_at_revision(&mut fixture.session, [3; 4], 2);
    fixture.host.push_presentation(
        worth_ui_host_contract::UiHostSurfacePresentationOutcome::PresentationIndeterminate,
    );

    assert!(matches!(
        execute(&mut fixture, 3),
        crate::mounting::UiMountedFrameOutcome::PresentationIndeterminate(_)
    ));
    assert!(fixture
        .session
        .presentation
        .appearance_invalidation_batch()
        .is_some());
    super::query_support::shutdown(fixture.session);
}

#[test]
fn host_supersession_evidence_is_indeterminate_and_preserves_newer_invalidation() {
    let mut fixture = settled_fixture();
    update_radius_at_revision(&mut fixture.session, [3; 4], 2);
    fixture.host.push_in_flight(
        vec![
            crate::certification_support::ScriptedSurfaceCompletion::Pending,
            crate::certification_support::ScriptedSurfaceCompletion::Superseded(
                worth_ui_host_contract::UiMountedSurfacePresentationSupersession::observed(
                    worth_ui_host_contract::UiHostPresentationCostReport::default(),
                ),
            ),
        ],
        worth_ui_host_contract::UiHostSurfaceCancellationOutcome::CancelledBeforeEffects,
    );
    let in_flight = match execute(&mut fixture, 3) {
        crate::mounting::UiMountedFrameOutcome::InFlight(in_flight) => in_flight,
        _ => panic!("expected an in-flight superseding attempt"),
    };
    update_radius_at_revision(&mut fixture.session, [4; 4], 3);

    let in_flight = match fixture.session.complete_mounted_presentation(in_flight, 4) {
        crate::mounting::UiMountedFrameOutcome::InFlight(in_flight) => in_flight,
        _ => panic!("the first superseding completion poll must remain in flight"),
    };
    let outcome = fixture.session.complete_mounted_presentation(in_flight, 5);
    assert!(matches!(
        outcome,
        crate::mounting::UiMountedFrameOutcome::PresentationIndeterminate(_)
    ));
    assert!(fixture
        .session
        .presentation
        .appearance_invalidation_batch()
        .is_some());
    super::query_support::shutdown(fixture.session);
}

#[test]
fn explicit_supersede_before_effects_preserves_newer_invalidation() {
    let mut fixture = settled_fixture();
    update_radius_at_revision(&mut fixture.session, [3; 4], 2);
    fixture.host.push_in_flight(
        vec![crate::certification_support::ScriptedSurfaceCompletion::Pending],
        worth_ui_host_contract::UiHostSurfaceCancellationOutcome::CancelledBeforeEffects,
    );
    let in_flight = match execute(&mut fixture, 3) {
        crate::mounting::UiMountedFrameOutcome::InFlight(in_flight) => in_flight,
        _ => panic!("expected a pending presentation to supersede"),
    };
    update_radius_at_revision(&mut fixture.session, [4; 4], 3);

    assert!(matches!(
        fixture.session.supersede_mounted_presentation(in_flight),
        crate::mounting::UiMountedFrameOutcome::RejectedBeforeEffects(_)
    ));
    assert!(fixture
        .session
        .presentation
        .appearance_invalidation_batch()
        .is_some());
    super::query_support::shutdown(fixture.session);
}

#[test]
fn older_publication_does_not_settle_a_newer_theme_revision() {
    let role = radius_role("test.receipt-currentness");
    let mut fixture = super::mounted_fixture(&role, &[], true);
    update_radius_at_revision(&mut fixture.session, [1; 4], 0);
    super::publish_frame(&mut fixture.session, 1);
    update_radius_at_revision(&mut fixture.session, [2; 4], 1);
    super::publish_frame(&mut fixture.session, 2);

    fixture.session.appearance_owner_snapshot = None;
    update_radius_at_revision(&mut fixture.session, [3; 4], 2);
    assert!(fixture
        .session
        .complete_application_theme_values_source()
        .has_theme_changes());
    assert!(fixture
        .session
        .presentation
        .appearance_invalidation_batch()
        .is_some());
    fixture.host.push_in_flight(
        vec![
            crate::certification_support::ScriptedSurfaceCompletion::Pending,
            native_completion(),
        ],
        worth_ui_host_contract::UiHostSurfaceCancellationOutcome::CancelledBeforeEffects,
    );
    let in_flight = match execute(&mut fixture, 3) {
        crate::mounting::UiMountedFrameOutcome::InFlight(in_flight) => in_flight,
        _ => panic!("expected update A to remain in flight"),
    };

    update_radius_at_revision(&mut fixture.session, [4; 4], 3);
    let in_flight = match fixture.session.complete_mounted_presentation(in_flight, 4) {
        crate::mounting::UiMountedFrameOutcome::InFlight(in_flight) => in_flight,
        _ => panic!("the first completion poll must remain in flight"),
    };
    assert!(matches!(
        fixture.session.complete_mounted_presentation(in_flight, 5),
        crate::mounting::UiMountedFrameOutcome::Published(_)
    ));
    let token = crate::capability::ThemeTokenId::new(
        crate::runtime::tests::appearance_component_session_test_support::APPEARANCE_TOKEN,
    )
    .unwrap();
    let expected_b = crate::capability::ThemeTokenValue::typed(radius_value_from([4; 4]));
    let source = fixture.session.complete_application_theme_values_source();
    assert_eq!(source.current_value(&token), Some(&expected_b));
    assert!(source.has_theme_changes());
    assert!(fixture
        .session
        .presentation
        .appearance_invalidation_batch()
        .is_some());

    fixture.host.push_native_display_presented();
    assert!(matches!(
        execute(&mut fixture, 6),
        crate::mounting::UiMountedFrameOutcome::Published(_)
    ));
    assert!(!fixture
        .session
        .complete_application_theme_values_source()
        .has_theme_changes());
    assert!(fixture
        .session
        .presentation
        .appearance_invalidation_batch()
        .is_some());

    super::refresh_appearance_owner_snapshot(
        &mut fixture.session,
        &role,
        "appearance-receipt-current-owner",
    );
    fixture.host.push_native_display_settled_without_effects();
    assert!(matches!(
        execute(&mut fixture, 7),
        crate::mounting::UiMountedFrameOutcome::Published(_)
    ));
    assert!(fixture
        .session
        .presentation
        .appearance_invalidation_batch()
        .is_none());
    super::query_support::shutdown(fixture.session);
}

fn settled_fixture() -> MountedAppearanceFixture {
    let role = radius_role("test.receipt-settlement");
    let mut fixture = super::mounted_fixture(&role, &[], true);
    update_radius_at_revision(&mut fixture.session, [1; 4], 0);
    super::publish_frame(&mut fixture.session, 1);
    update_radius_at_revision(&mut fixture.session, [2; 4], 1);
    super::publish_frame(&mut fixture.session, 2);
    fixture
}

fn execute(
    fixture: &mut MountedAppearanceFixture,
    now: u64,
) -> crate::mounting::UiMountedFrameOutcome {
    fixture
        .session
        .execute_mounted_frame(
            crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
            worth_ui_host_contract::UiPresentationDeadline::at_tick(100),
            now,
            |_| {},
        )
        .unwrap_or_else(|_| panic!("settlement frame should execute"))
}

fn native_completion() -> crate::certification_support::ScriptedSurfaceCompletion {
    crate::certification_support::ScriptedSurfaceCompletion::Presented(
        worth_ui_host_contract::UiMountedSurfacePresentationCompletion::new(
            crate::facade::mounted::UiHostSurfacePresentationMode::NativeDisplay,
            crate::certification_support::scripted_presentation_epoch(),
            worth_ui_host_contract::UiMountedCompletedEffects::new(vec![
                worth_ui_host_contract::UiMountedEffectFamily::NativePaint,
            ]),
            worth_ui_host_contract::UiHostPresentationCostReport::default(),
        ),
    )
}
