use super::appearance_publication_support::{establish_geometry, open_portal};
use super::appearance_surface_scope::publish;
use super::test_support::portal_target;
use crate::mounting::UiMountedFrameRequest;
use worth_ui_host_contract::UiUnpublishedAppearanceFragmentIdentity;

#[test]
fn portal_only_publication_needs_no_backdrop_appearance_owner() {
    let (mut session, host) =
        super::appearance_publication_support::appearance_overlay_session_with_source(
            super::test_support::AUTHORED_PORTAL_SOURCE,
        );
    verify_structural_portal(
        &mut session,
        &host,
        false,
        super::test_support::AUTHORED_PORTAL_SOURCE,
    );
    let _ = session.shutdown();
}

#[test]
fn attached_opacity_only_portal_keeps_structural_order_without_surface_paint() {
    use worth_ui_dsl::*;
    let source = super::test_support::AUTHORED_PORTAL_SOURCE.replace(
        "component workspace.component.overlay {",
        "appearance role overlay.content applies_to workspace.component.overlay { opacity use token(overlay.scrim.opacity) }\ncomponent workspace.component.overlay { appearance { role overlay.content }",
    );
    let role =
        UiAppearanceRole::authoring(UiAppearanceRoleIdentity::new("overlay.content").unwrap())
            .applies_to_component(
                UiDslComponentReference::new("workspace.component.overlay").unwrap(),
            )
            .cover(
                UiAppearanceAspect::Opacity,
                UiAppearancePartitionAuthoring::new([]).with_cell(
                    UiAppearanceCell::when([]).uses_slot(
                        UiThemeSlotIdentity::new("overlay.scrim.opacity").unwrap(),
                        UiThemeValueKind::Opacity,
                    ),
                ),
            )
            .unwrap()
            .build()
            .unwrap();
    let (mut session, host) =
        super::appearance_publication_support::appearance_overlay_session_with_component_role(
            &source, role,
        );
    verify_structural_portal(&mut session, &host, true, &source);
    let _ = session.shutdown();
}

fn verify_structural_portal(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    host: &crate::certification_support::ScriptedPresentationHost,
    attached: bool,
    source: &str,
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
    super::appearance_publication_support::close_source_with(session, source);
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
    publish(session, host, &[surface], false);
    assert_eq!(session.appearance_owner_snapshot.is_some(), attached);
    assert_eq!(
        session
            .mounted
            .current_projection_rc_for_test()
            .unwrap()
            .portal_has_appearance_attachment(mounted, surface),
        Ok(attached),
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
    worth_ui_host_headless::translate_unpublished_appearance_for_certification(output).unwrap();
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
    worth_ui_host_headless::translate_unpublished_appearance_for_certification(output).unwrap();
}
