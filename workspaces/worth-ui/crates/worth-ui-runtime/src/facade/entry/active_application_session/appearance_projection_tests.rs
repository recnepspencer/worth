use crate::runtime::tests::appearance_component_session_test_support as support;

#[path = "appearance_projection_test_support.rs"]
mod test_support;
use test_support::theme_session;

#[path = "appearance_projection_denial_tests.rs"]
mod denial_tests;

#[path = "appearance_projection_removal_tests.rs"]
mod removal_tests;

#[path = "appearance_projection_succession_tests.rs"]
mod succession_tests;

#[path = "appearance_projection_surface_tests.rs"]
mod surface_tests;

#[path = "appearance_projection_mounting_fixture.rs"]
mod mounting_fixture;

#[path = "appearance_projection_clip_denial_tests.rs"]
mod clip_denial_tests;
#[path = "appearance_projection_outline_tests.rs"]
mod outline_tests;

#[path = "appearance_projection_pointer_tests.rs"]
mod pointer_tests;

#[path = "appearance_projection_focus_tests.rs"]
mod focus_tests;

#[path = "appearance_projection_validation_tests.rs"]
mod validation_tests;

#[path = "mounted_occurrence_geometry_tests.rs"]
mod occurrence_geometry_tests;
#[path = "appearance_projection_operability_tests.rs"]
mod operability_tests;
#[path = "appearance_projection_selection_tests.rs"]
mod selection_tests;

enum Lifecycle {
    Unmount,
    RoleDetachment,
    SurfaceDeregistration,
    Reconstruction,
}

#[test]
fn why_appearance_reads_the_production_resolve_and_mount_receipt() {
    production_appearance_case(
        worth_ui_dsl::UiAppearanceStateAxis::Validation,
        Lifecycle::Unmount,
    );
}

#[test]
fn source_role_detachment_retires_appearance_in_the_replacement_frame() {
    production_appearance_case(
        worth_ui_dsl::UiAppearanceStateAxis::Validation,
        Lifecycle::RoleDetachment,
    );
}

#[test]
fn deregistration_discards_only_the_removed_surfaces_retained_appearance() {
    production_appearance_case(
        worth_ui_dsl::UiAppearanceStateAxis::Validation,
        Lifecycle::SurfaceDeregistration,
    );
}

#[test]
fn reconstruction_refreshes_expired_appearance_from_current_owners() {
    production_appearance_case(
        worth_ui_dsl::UiAppearanceStateAxis::Validation,
        Lifecycle::Reconstruction,
    );
}

#[test]
fn first_mount_consumes_empty_pointer_and_pressed_owners() {
    for axis in [
        worth_ui_dsl::UiAppearanceStateAxis::Hover,
        worth_ui_dsl::UiAppearanceStateAxis::Pressed,
    ] {
        production_appearance_case(axis, Lifecycle::Unmount);
    }
}

fn production_appearance_case(axis: worth_ui_dsl::UiAppearanceStateAxis, lifecycle: Lifecycle) {
    let role = support::validation_background_role_with_axis(support::APPEARANCE_TOKEN, axis);
    let (mut session, host) = theme_session(&role);
    let (surface, graph_node) = mounting_fixture::mount(&mut session, 1_000);
    let initial_observation = support::attached_appearance_candidate_submission(
        &session,
        "appearance-production-initial",
        "workspace.component.active_session_current",
    );
    let mut initial_turn = session.begin_observation_turn().unwrap();
    initial_turn.admit_source(initial_observation).unwrap();
    let initial_admitted = initial_turn.seal().unwrap();
    session.classify_observations(initial_admitted).unwrap();
    assert!(session.has_appearance_owner_snapshot_for_test());
    session.advance_mounted_identity_frame().unwrap();
    assert_eq!(host.presentation_calls(), 0);
    assert!(session.mounted.current_publication().is_none());
    test_support::publish_initial_appearance(&mut session, &role, graph_node, 1);
    let initial_projection = session
        .mounted
        .current_unpublished_appearance()
        .expect("initial appearance transport is admitted")
        .expect("initial resolved appearance produces unpublished work");
    test_support::assert_unpublished_surface(initial_projection, [17, 34, 51, 255]);
    assert_eq!(
        initial_projection.fragments()[0].work().posture(),
        worth_ui_host_contract::UiMountedAppearanceWorkPosture::Initial
    );
    let initial_frame = initial_projection.frame();
    if axis != worth_ui_dsl::UiAppearanceStateAxis::Validation {
        let _ = session.shutdown();
        return;
    }

    test_support::change_appearance_color(&mut session, 0, "#405060");
    assert!(session
        .presentation
        .appearance_invalidation_batch()
        .is_some_and(|batch| batch.selected_count() > 0));

    let prepared = session
        .prepare_mounted_frame_with_application_presentation(
            crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
            |_| {},
        )
        .unwrap_or_else(|_| {
            panic!("the ordinary production frame should resolve and mount appearance")
        });
    prepared.verify_unpublished_appearance_denial_and_retry();
    host.push_native_display_settled_without_effects();
    let outcome = session.present_prepared_mounted_frame_internal(
        prepared,
        worth_ui_host_contract::UiPresentationDeadline::at_tick(100),
        2,
    );
    assert!(matches!(
        outcome,
        crate::mounting::UiMountedFrameOutcome::Published(_)
    ));
    let static_paint_colors = host.last_filled_rect_colors();
    assert!(
        static_paint_colors.is_empty(),
        "Gate 4 keeps resolved appearance out of the live host command lane"
    );

    let projection = session
        .mounted
        .current_unpublished_appearance()
        .expect("runtime work has current binding and receipt affinity")
        .expect("resolved background must reach the unpublished frame");
    test_support::assert_unpublished_surface(projection, [64, 80, 96, 255]);
    let physical_predecessor = projection.clone();
    assert_eq!(
        projection.fragments()[0].work().posture(),
        worth_ui_host_contract::UiMountedAppearanceWorkPosture::Delta
    );
    assert_eq!(
        projection.fragments()[0].work().predecessor(),
        Some(initial_frame)
    );

    let world = session.appearance_inspection_world(surface);
    let query = worth_ui_inspection::UiAppearanceInspectionQuery::new(
        world,
        graph_node.digest(),
        worth_ui_dsl::UiAppearanceAspect::Background,
    );
    let explanation = match session.why_appearance(query) {
        worth_ui_inspection::UiAppearanceInspectionOutcome::Found(explanation) => explanation,
        outcome => panic!("production appearance receipt was not inspected: {outcome:?}"),
    };
    assert!(explanation.mounted_mechanical_output_changed());
    assert_eq!(
        explanation.mounted_mechanic(),
        worth_ui_inspection::UiAppearanceInspectionMountedMechanic::Changed
    );
    assert_eq!(
        explanation.physical_suppression(),
        worth_ui_inspection::UiAppearanceInspectionPhysicalSuppression::NotSuppressed
    );
    assert!(explanation.cost().consumers_selected() > 0);
    assert_eq!(explanation.cost().theme_slots_compared(), 1);
    assert_eq!(
        explanation.value(),
        worth_ui_inspection::UiAppearanceInspectionValue::Resolved(
            worth_ui_dsl::UiThemeValue::Color(worth_ui_dsl::UiThemeColor::from_channels([
                64, 80, 96, 255,
            ])),
        )
    );

    host.push_native_display_settled_without_effects();
    test_support::publish_without_selected_appearance(&mut session, 3);
    if matches!(lifecycle, Lifecycle::Reconstruction) {
        succession_tests::reconstruct_across_source_generation(
            &mut session,
            &host,
            physical_predecessor,
            &role,
        );
        let _ = session.shutdown();
        return;
    }
    let physical_predecessor = succession_tests::retain_across_source_generation(
        &mut session,
        &host,
        physical_predecessor,
        &role,
    );
    match lifecycle {
        Lifecycle::Reconstruction => unreachable!("reconstruction owns its successor lifecycle"),
        Lifecycle::RoleDetachment => {
            succession_tests::detach_role(&mut session, &host, physical_predecessor, &role)
        }
        Lifecycle::Unmount => removal_tests::remove_last_nodes_without_a_theme_change(
            &mut session,
            &host,
            physical_predecessor,
        ),
        Lifecycle::SurfaceDeregistration => {
            surface_tests::deregister_one_surface(&mut session, &host, physical_predecessor)
        }
    }
    let _ = session.shutdown();
}
