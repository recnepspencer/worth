use super::appearance_publication_support::{
    appearance_overlay_session, close_source, establish_geometry, open_portal,
};
use super::test_support::portal_target;
use crate::mounting::{UiMountedFrameOutcome, UiMountedFrameRequest};
use worth_ui_host_contract::{
    UiPresentationDeadline, UiSemanticSurfaceIdentity, UiUnpublishedAppearanceFragmentIdentity,
};

#[test]
fn exact_surface_publication_preserves_omitted_overlay_then_retires_its_owner() {
    let (mut session, host) = appearance_overlay_session();
    let material = session
        .application
        .authored_overlay_material()
        .overlay_declaration_bindings();
    let first_decl = material.surface_named("workspace.surface.overlay").unwrap();
    let second_decl = material
        .surface_named("workspace.surface.secondary")
        .unwrap();
    let first_portal = material.portal_named("overlay.menu").unwrap();
    let second_portal = material.portal_named("overlay.secondary").unwrap();
    let first = session
        .create_declared_semantic_surface(first_decl)
        .unwrap();
    let second = session
        .create_declared_semantic_surface(second_decl)
        .unwrap();
    for surface in [first, second] {
        session
            .register_host_surface(
                surface,
                crate::facade::mounted::UiHostSurfacePresentationMode::NativeDisplay,
                crate::facade::mounted::UiSurfaceBindingProfile::new(
                    1_000,
                    crate::facade::mounted::UiSurfaceBindingCoordinatePosture::LogicalPoints,
                    1,
                )
                .unwrap(),
            )
            .unwrap();
    }
    let (graph, first_mount) = portal_target(&mut session, first);
    let node = session.mounted_graph_node(graph).unwrap();
    let second_mount = session.mount_instance(node, second).unwrap();
    establish_geometry(&mut session, first);
    crate::facade::entry::mounted_occurrence_geometry_test_support::
        install_nonoverlapping_surface_geometry(&mut session, second, 1, &[]);
    publish(&mut session, &host, &[first, second], true);
    close_source(&mut session);
    drop(
        session
            .prepare_mounted_frame_with_application_presentation(
                UiMountedFrameRequest::exact_surfaces(vec![first, second]),
                |_| {},
            )
            .unwrap_or_else(|_| panic!("the source cutover candidate should prepare")),
    );
    let first_identity = open_portal(&mut session, first, graph, first_mount, first_portal, 10);
    let second_identity = open_portal(&mut session, second, graph, second_mount, second_portal, 11);
    crate::facade::entry::mounted_occurrence_geometry_test_support::
        refresh_nonoverlapping_surface_geometry(&mut session, first);
    crate::facade::entry::mounted_occurrence_geometry_test_support::
        refresh_nonoverlapping_surface_geometry(&mut session, second);
    publish(&mut session, &host, &[first, second], false);
    let second_predecessor = session
        .overlay_composition_owners
        .current_for_test(second)
        .unwrap()
        .current()
        .unwrap()
        .clone();
    let output = session
        .mounted
        .current_unpublished_appearance()
        .unwrap()
        .unwrap();
    let second_mechanics = output
        .fragments()
        .iter()
        .find(|fragment| {
            fragment.identity() == UiUnpublishedAppearanceFragmentIdentity::SurfaceOverlay(second)
        })
        .unwrap()
        .work()
        .successor()
        .clone();

    let close = crate::runtime::portal::UiPortalServiceRequest::close(
        second_identity,
        crate::runtime::intent_execution::UiIntentExecutionIdempotencyIdentity::issued(
            session.session_identity().as_u64(),
            12,
        ),
        crate::runtime::portal::UiPortalDismissalCause::ExplicitOwnerRequest,
        second,
    );
    let transition = session.portal.as_ref().unwrap().prepare(close).unwrap();
    let binding =
        crate::runtime::portal::UiPortalOverlayBindingCommit::from_transition(&transition, None)
            .with_retained_exit(false);
    session
        .portal
        .as_mut()
        .unwrap()
        .commit_published(transition)
        .unwrap();
    session.commit_authored_overlay_binding(binding).unwrap();

    publish(&mut session, &host, &[first], false);
    assert_eq!(
        session
            .overlay_composition_owners
            .current_for_test(second)
            .unwrap()
            .current(),
        Some(&second_predecessor),
        "an omitted surface retains its exact accepted owner, even when its Portal source changed"
    );
    let output = session
        .mounted
        .current_unpublished_appearance()
        .unwrap()
        .unwrap();
    assert!(output.fragments().iter().all(|fragment| {
        fragment.identity() != UiUnpublishedAppearanceFragmentIdentity::SurfaceOverlay(second)
    }));
    assert_eq!(
        session
            .overlay_composition_owners
            .current_for_test(first)
            .unwrap()
            .last_counters()
            .backdrop_declarations_selected(),
        0
    );
    assert_eq!(
        session
            .overlay_composition_owners
            .backdrop_work_for_test(first)
            .unwrap()
            .roles_resolved,
        0
    );

    let first_presentation = session
        .portal
        .as_ref()
        .unwrap()
        .committed_presentation_for(first_identity);
    publish(&mut session, &host, &[second], false);
    assert_eq!(
        session
            .portal
            .as_ref()
            .unwrap()
            .committed_presentation_for(first_identity),
        first_presentation
    );
    assert!(session
        .overlay_composition_owners
        .current_for_test(second)
        .is_none());
    let output = session
        .mounted
        .current_unpublished_appearance()
        .unwrap()
        .unwrap();
    let removed = output
        .fragments()
        .iter()
        .find(|fragment| {
            fragment.identity() == UiUnpublishedAppearanceFragmentIdentity::SurfaceOverlay(second)
        })
        .expect("the selected surface publishes its pending removal");
    assert_eq!(removed.work().predecessor(), Some(second_mechanics.frame()));
    assert!(removed.work().successor().mechanics().is_empty());
    worth_ui_host_headless::translate_unpublished_appearance_for_certification(output).unwrap();
    let _ = session.shutdown();
}

pub(super) fn publish(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    host: &crate::certification_support::ScriptedPresentationHost,
    surfaces: &[UiSemanticSurfaceIdentity],
    initial: bool,
) {
    let frame = session
        .prepare_mounted_frame_with_application_presentation(
            UiMountedFrameRequest::exact_surfaces(surfaces.to_vec()),
            |_| {},
        )
        .unwrap_or_else(|_| panic!("exact surface frame must prepare"));
    for _ in surfaces {
        if initial {
            host.push_native_display_presented();
        } else {
            host.push_native_display_settled_without_effects();
        }
    }
    let outcome = session.present_prepared_mounted_frame_internal(
        frame,
        UiPresentationDeadline::at_tick(u64::MAX),
        10,
    );
    if let UiMountedFrameOutcome::AdmissionDenied(rejection) = &outcome {
        panic!(
            "exact surface appearance admission denied: {:?}",
            rejection.denial()
        );
    }
    assert!(matches!(
        outcome,
        UiMountedFrameOutcome::Published(_) | UiMountedFrameOutcome::Unchanged(_)
    ));
}
