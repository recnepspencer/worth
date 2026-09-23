use super::appearance_publication_support::{establish_geometry, open_portal};
use super::test_support::portal_target;
use crate::mounting::{UiMountedFrameOutcome, UiMountedFrameRequest};
use worth_ui_host_contract::UiUnpublishedAppearanceFragmentIdentity;

#[test]
fn portal_only_publication_needs_no_backdrop_appearance_owner() {
    let (mut session, host) =
        super::appearance_publication_support::appearance_overlay_session_with_source(
            super::test_support::AUTHORED_PORTAL_SOURCE,
        );
    verify_structural_portal(&mut session, &host);
    let _ = session.shutdown();
}

fn verify_structural_portal(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    host: &crate::certification_support::ScriptedPresentationHost,
) {
    let bindings = session
        .application
        .authored_overlay_material()
        .overlay_declaration_bindings();
    let declared = bindings.surface_named("workspace.surface.overlay").unwrap();
    let portal = bindings.portal_named("overlay.menu").unwrap();
    let surface = session.create_declared_semantic_surface(declared).unwrap();
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
    let (graph, mounted) = portal_target(session, surface);
    establish_geometry(session, surface);
    assert!(session.appearance_owner_snapshot.is_none());
    publish(session, host, &[surface], true);
    super::appearance_publication_support::close_source_with(
        session,
        super::test_support::AUTHORED_PORTAL_SOURCE,
    );
    drop(
        session
            .prepare_mounted_frame_with_application_presentation(
                UiMountedFrameRequest::exact_surfaces(vec![surface]),
                |_| {},
            )
            .unwrap_or_else(|_| panic!("the source cutover candidate should prepare")),
    );
    let identity = open_portal(session, surface, graph, mounted, portal, 1);
    crate::facade::entry::mounted_occurrence_geometry_test_support::
        refresh_nonoverlapping_surface_geometry(session, surface);
    assert_missing_order_denied_before_host(session, host, surface);
    publish(session, host, &[surface], false);
    assert!(session.appearance_owner_snapshot.is_none());
    assert_eq!(
        session
            .mounted
            .current_projection_rc_for_test()
            .unwrap()
            .portal_has_appearance_attachment(mounted, surface),
        Ok(false),
        "the exact published mount carries the graph-authored attachment; successful attached publication consumed its retained projection"
    );
    let snapshot = session
        .overlay_composition_owners
        .current_for_test(surface)
        .unwrap()
        .current()
        .unwrap();
    assert_eq!(snapshot.participants().len(), 1);
    assert!(matches!(
        snapshot.participants()[0],
        crate::runtime::overlay_composition::UiOverlayStackParticipant::Portal(_)
    ));
    assert_eq!(
        session
            .overlay_composition_owners
            .backdrop_work_for_test(surface)
            .unwrap()
            .roles_resolved,
        0
    );
    let output = session
        .mounted
        .current_unpublished_appearance()
        .unwrap()
        .unwrap();
    let fragment = output
        .fragments()
        .iter()
        .find(|fragment| {
            fragment.identity() == UiUnpublishedAppearanceFragmentIdentity::SurfaceOverlay(surface)
        })
        .unwrap();
    assert!(fragment.work().successor().mechanics().is_empty());
    assert_eq!(
        fragment.work().successor().overlay_order().bottom_to_top(),
        &[worth_ui_host_contract::UiOverlayParticipantIdentity::Portal(mounted)]
    );
    worth_ui_host_headless::translate_appearance_projection_for_certification(output).unwrap();
    let previous_extent = snapshot.extent_revision();
    crate::facade::entry::mounted_occurrence_geometry_test_support::
        refresh_nonoverlapping_surface_geometry(session, surface);
    publish(session, host, &[surface], false);
    let refreshed = session
        .overlay_composition_owners
        .current_for_test(surface)
        .unwrap()
        .current()
        .unwrap();
    assert!(refreshed.extent_revision() > previous_extent);
    assert_eq!(
        refreshed.extent_revision(),
        session
            .mounted
            .current_surface_viewport(surface)
            .unwrap()
            .0
            .get()
    );
    assert_eq!(
        refreshed.participants().len(),
        1,
        "an extent revision preserves the one issued Portal without inventing a Backdrop"
    );
    assert_eq!(
        session
            .overlay_composition_owners
            .backdrop_work_for_test(surface)
            .unwrap()
            .roles_resolved,
        0
    );
    let close = crate::runtime::portal::UiPortalServiceRequest::close(
        identity,
        crate::runtime::intent_execution::UiIntentExecutionIdempotencyIdentity::issued(
            session.session_identity().as_u64(),
            2,
        ),
        crate::runtime::portal::UiPortalDismissalCause::ExplicitOwnerRequest,
        surface,
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
    publish(session, host, &[surface], false);
    let output = session
        .mounted
        .current_unpublished_appearance()
        .unwrap()
        .unwrap();
    let removed = output
        .fragments()
        .iter()
        .find(|fragment| {
            fragment.identity() == UiUnpublishedAppearanceFragmentIdentity::SurfaceOverlay(surface)
        })
        .unwrap();
    assert!(removed.work().successor().mechanics().is_empty());
    assert!(removed
        .work()
        .successor()
        .overlay_order()
        .bottom_to_top()
        .is_empty());
    assert_eq!(
        removed
            .work()
            .predecessor_manifest()
            .unwrap()
            .overlay_order(),
        &[worth_ui_host_contract::UiOverlayParticipantIdentity::Portal(mounted)]
    );
    assert!(removed.work().damage().is_empty());
    worth_ui_host_headless::translate_appearance_projection_for_certification(output).unwrap();
}

fn publish(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    host: &crate::certification_support::ScriptedPresentationHost,
    surfaces: &[worth_ui_host_contract::UiSemanticSurfaceIdentity],
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
    assert!(matches!(
        session.present_prepared_mounted_frame_internal(
            frame,
            worth_ui_host_contract::UiPresentationDeadline::at_tick(u64::MAX),
            10,
        ),
        UiMountedFrameOutcome::Published(_) | UiMountedFrameOutcome::Unchanged(_)
    ));
}

fn assert_missing_order_denied_before_host(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    host: &crate::certification_support::ScriptedPresentationHost,
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
) {
    let predecessor = session.current_mounted_publication().unwrap().frame();
    let calls = host.presentation_calls();
    let request = session.mounted_frame_request();
    assert!(request
        .portal_overlays()
        .iter()
        .any(|portal| portal.surface() == surface));
    let frame = session
        .prepare_mounted_frame_with_application_presentation(request, |_| {})
        .unwrap_or_else(|_| panic!("the actual open Portal frame must prepare"));
    // Exercise the production presentation boundary with the required composition omitted.
    let (outcome, _, _, _) = session
        .mounted
        .present_prepared_frame_with_overlays(
            &session.host_session,
            frame,
            None,
            worth_ui_host_contract::UiPresentationDeadline::at_tick(u64::MAX),
            10,
            |_, _, _| Ok(Default::default()),
        )
        .into_parts();
    let crate::mounting::UiMountedFrameOutcome::AdmissionDenied(rejection) = outcome else {
        panic!("a mounted Portal without relational order must deny before presentation");
    };
    assert!(matches!(
        rejection.denial(),
        crate::mounting::UiMountedPresentationAdmissionDenial::AppearanceOutputUnavailable
    ));
    assert_eq!(host.presentation_calls(), calls);
    assert_eq!(
        session.current_mounted_publication().unwrap().frame(),
        predecessor
    );
}
