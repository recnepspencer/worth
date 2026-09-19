use super::appearance_publication_support::{
    component_role, establish_geometry, open_portal, theme_bundle,
};
use super::test_support::{authored_overlay_builder_with_component, portal_target};
use crate::capability::*;
use crate::mounting::UiMountedFrameOutcome;
use crate::runtime::tests::source_ingress_boundary_test_support::lower_file_submission;
use crate::runtime::{WorthUiSourceProvider, WorthUiWatcherEvent};
use worth_ui_host_contract::{
    UiMountedAppearanceMechanic, UiUnpublishedAppearanceFragmentIdentity,
};

#[test]
fn authored_child_composition_keeps_anchor_paint_and_relational_portal_order() {
    for child_composition in [false, true] {
        let role = component_role();
        let builder = || {
            let component = ComponentDescriptor::new(
                ComponentId::new("workspace.component.overlay").unwrap(),
                ComponentPropSchema::named("workspace.component.overlay.props"),
                ComponentChildPolicy::no_children(),
                ComponentStateOwnership::runtime_owned(),
            )
            .with_hit_test(ComponentHitTestContract::allocation_bounds(
                ComponentHitTestOrder::front_to_back(0),
                ComponentAllocationMeasurementContract::fill_viewport(),
            ))
            .with_surface_paint_order(0)
            .with_appearance_aspect_contract(role.aspect_contract().clone())
            .unwrap();
            let component = if child_composition {
                component.with_portal_surface_from_children()
            } else {
                component
            };
            authored_overlay_builder_with_component(component)
                .register_appearance_role(role.clone())
                .unwrap()
                .register_appearance_theme_bundle(theme_bundle())
                .unwrap()
        };
        let source = "appearance role overlay.content applies_to workspace.component.overlay { background use token(overlay.content.background) }\n".to_owned()
            + super::test_support::AUTHORED_PORTAL_SOURCE;
        // This source uses the same declared component and portal; attach its role explicitly.
        let source = source.replace(
            "component workspace.component.overlay {",
            "component workspace.component.overlay { appearance { role overlay.content }",
        );
        let capabilities = builder().freeze()
            .map(crate::facade::entry::WorthUiCertificationApplicationTransition::activate_builder_host).unwrap();
        let submission = lower_file_submission(
            WorthUiSourceProvider::in_memory("portal-child-composition")
                .with_file("app/main.wui", &source),
            [WorthUiWatcherEvent::provider_revision(
                "portal-child-composition",
            )],
            capabilities.capabilities(),
        );
        let host = crate::certification_support::ScriptedPresentationHost::native_display();
        host.set_capabilities(worth_ui_host_native::appearance_capability_report());
        let observer = host.clone();
        let mut session = builder()
            .with_candidate_submission(submission)
            .freeze()
            .map(|app| {
                crate::facade::entry::WorthUiCertificationApplicationTransition::activate_test_host(
                    app, host,
                )
            })
            .unwrap()
            .launch()
            .unwrap();
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
        let (graph, mounted) = portal_target(&mut session, surface);
        establish_geometry(&mut session, surface);
        super::appearance_publication_support::close_source_with(&mut session, &source);
        publish(&mut session, &observer, surface);
        open_portal(&mut session, surface, graph, mounted, portal, 1);
        crate::facade::entry::mounted_occurrence_geometry_test_support::refresh_nonoverlapping_surface_geometry(&mut session, surface);
        publish(&mut session, &observer, surface);
        let output = session
            .mounted
            .current_unpublished_appearance()
            .unwrap()
            .unwrap();
        let overlay = output
            .fragments()
            .iter()
            .find(|fragment| {
                fragment.identity()
                    == UiUnpublishedAppearanceFragmentIdentity::SurfaceOverlay(surface)
            })
            .unwrap();
        assert_eq!(
            overlay
                .work()
                .successor()
                .mechanics()
                .iter()
                .filter(|mechanic| matches!(
                    mechanic,
                    UiMountedAppearanceMechanic::PortalSurface(_)
                ))
                .count(),
            usize::from(!child_composition)
        );
        assert!(
            output
                .fragments()
                .iter()
                .flat_map(|fragment| fragment.work().successor().mechanics())
                .any(|mechanic| matches!(mechanic, UiMountedAppearanceMechanic::Surface(_))),
            "ordinary anchor paint must remain"
        );
        assert_eq!(
            overlay.work().successor().overlay_order().bottom_to_top(),
            &[worth_ui_host_contract::UiOverlayParticipantIdentity::Portal(mounted)]
        );
        worth_ui_host_headless::translate_appearance_projection_for_certification(output).unwrap();
        let _ = session.shutdown();
    }
}

fn publish(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    host: &crate::certification_support::ScriptedPresentationHost,
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
) {
    let prepared = session
        .prepare_mounted_frame_with_application_presentation(
            session.mounted_frame_request(),
            |_| {},
        )
        .unwrap_or_else(|_| panic!("prepare authored portal"));
    host.push_native_display_presented();
    match session.present_prepared_mounted_frame_internal(
        prepared,
        worth_ui_host_contract::UiPresentationDeadline::at_tick(u64::MAX),
        10,
    ) {
        UiMountedFrameOutcome::Published(_)
        | UiMountedFrameOutcome::Unchanged(_)
        | UiMountedFrameOutcome::Reconciled(_) => {}
        UiMountedFrameOutcome::AdmissionDenied(rejection) => {
            let (graph, _) = portal_target(session, surface);
            let detail =
                session.why_appearance(worth_ui_inspection::UiAppearanceInspectionQuery::new(
                    session.appearance_inspection_world(surface),
                    graph.digest(),
                    worth_ui_dsl::UiAppearanceAspect::Background,
                ));
            panic!(
                "admission: {:?}; appearance: {:?}",
                rejection.denial(),
                detail
            )
        }
        UiMountedFrameOutcome::RejectedBeforeEffects(rejection) => {
            panic!("host: {:?}", rejection.rejections())
        }
        UiMountedFrameOutcome::PresentationIndeterminate(recovery) => {
            panic!("indeterminate: {:?}", recovery.report())
        }
        _ => panic!("authored portal did not publish synchronously"),
    }
}
