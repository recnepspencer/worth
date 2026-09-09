use super::appearance_publication_support::{
    appearance_overlay_session, close_source, establish_geometry,
};
use super::test_support::portal_target;
use worth_ui_host_contract::{
    UiMountedAppearanceMechanic, UiPresentationDeadline, UiUnpublishedAppearanceFragmentIdentity,
};

#[test]
fn production_publication_emits_one_ordered_surface_overlay() {
    let (mut session, host) = appearance_overlay_session();
    let material = session.application.authored_overlay_material();
    let surface_declaration = material
        .overlay_declaration_bindings()
        .surface_named("workspace.surface.overlay")
        .unwrap();
    let portal_declaration = material
        .overlay_declaration_bindings()
        .portal_named("overlay.menu")
        .unwrap();
    let surface = session
        .create_declared_semantic_surface(surface_declaration)
        .unwrap();
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
    let (graph_node, mounted) = portal_target(&mut session, surface);
    establish_geometry(&mut session, surface);
    let baseline = session
        .prepare_mounted_frame_with_application_presentation(
            crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
            |_| {},
        )
        .unwrap_or_else(|_| panic!("baseline appearance frame should prepare"));
    host.push_native_display_presented();
    assert!(matches!(
        session.present_prepared_mounted_frame_internal(
            baseline,
            UiPresentationDeadline::at_tick(u64::MAX),
            0,
        ),
        crate::mounting::UiMountedFrameOutcome::Published(_)
    ));
    close_source(&mut session);
    drop(
        session
            .prepare_mounted_frame_with_application_presentation(
                crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
                |_| {},
            )
            .unwrap_or_else(|_| panic!("the source cutover candidate should prepare")),
    );
    let portal = crate::runtime::portal::UiPortalIdentity::for_owner(
        crate::runtime::portal::UiPortalOwnerIdentity::from_mounted_owner(graph_node, mounted),
    );
    let presentation = session
        .mounted
        .current_presentation_for_surface(surface)
        .expect("baseline publishes the Portal surface");
    let geometry =
        crate::runtime::interaction::UiPresentedInteractionGeometry::for_test(presentation);
    let request = crate::runtime::portal::UiPortalServiceRequest::open(
        portal,
        crate::runtime::intent_execution::UiIntentExecutionIdempotencyIdentity::issued(
            session.session_identity().as_u64(),
            1,
        ),
        geometry,
        Some(
            crate::runtime::interaction::UiPresentedViewportGeometry::for_test(
                geometry.clip_bounds(),
                geometry.presentation(),
            ),
        ),
        surface,
    )
    .with_declared_portal(Some(portal_declaration));
    let stage = session
        .admit_authored_portal_open(portal_declaration, portal, surface)
        .unwrap();
    let transition = session.portal.as_ref().unwrap().prepare(request).unwrap();
    let binding = crate::runtime::portal::UiPortalOverlayBindingCommit::from_transition(
        &transition,
        Some(stage),
    );
    session
        .portal
        .as_mut()
        .unwrap()
        .commit_published(transition)
        .unwrap();
    session.commit_authored_overlay_binding(binding).unwrap();
    let prior_layout_revision = session
        .mounted
        .current_surface_viewport(surface)
        .expect("the Portal predecessor retains completed surface geometry")
        .0;
    crate::facade::entry::mounted_occurrence_geometry_test_support::
        refresh_nonoverlapping_surface_geometry(&mut session, surface);
    let (layout_revision, viewport) = session
        .mounted
        .current_surface_viewport(surface)
        .expect("completed layout owns exact surface viewport geometry");
    assert_eq!(layout_revision.get(), prior_layout_revision.get() + 1);
    assert_eq!(
        viewport,
        crate::facade::entry::mounted_occurrence_geometry_test_support::surface_viewport_bounds()
    );
    assert!(session.prepare_overlay_appearance_sources().is_ok());
    assert!(session.appearance_owner_snapshot.is_some());
    assert!(session
        .application
        .capabilities()
        .appearance_roles()
        .get(&worth_ui_dsl::UiAppearanceRoleIdentity::new("overlay.scrim").unwrap())
        .is_some());
    let overlay_predecessor = session
        .overlay_composition_owners
        .current_for_test(surface)
        .and_then(crate::runtime::overlay_composition::UiOverlayCompositionOwnerLifecycle::current)
        .cloned()
        .expect("baseline retains the empty overlay predecessor");

    let rejected_frame = session
        .prepare_mounted_frame_with_application_presentation(
            crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
            |_| {},
        )
        .unwrap_or_else(|_| panic!("overlay appearance frame should prepare"));
    host.push_rejected();
    let rejected = session.present_prepared_mounted_frame_internal(
        rejected_frame,
        UiPresentationDeadline::at_tick(u64::MAX),
        1,
    );
    match &rejected {
        crate::mounting::UiMountedFrameOutcome::RejectedBeforeEffects(_) => {}
        crate::mounting::UiMountedFrameOutcome::RetentionDenied(value) => {
            panic!("overlay rejection retention denied: {:?}", value.denial())
        }
        crate::mounting::UiMountedFrameOutcome::AdmissionDenied(value) => {
            panic!("overlay rejection admission denied: {:?}", value.denial())
        }
        crate::mounting::UiMountedFrameOutcome::CompletionDenied(value) => {
            panic!("overlay rejection completion denied: {value:?}")
        }
        crate::mounting::UiMountedFrameOutcome::Published(_) => {
            panic!("overlay rejection unexpectedly published")
        }
        crate::mounting::UiMountedFrameOutcome::Unchanged(_) => {
            panic!("overlay rejection was unchanged")
        }
        crate::mounting::UiMountedFrameOutcome::Reconciled(_) => {
            panic!("overlay rejection reconciled")
        }
        crate::mounting::UiMountedFrameOutcome::InFlight(_) => {
            panic!("overlay rejection remained in flight")
        }
        crate::mounting::UiMountedFrameOutcome::PresentationIndeterminate(_) => {
            panic!("overlay rejection became indeterminate")
        }
        crate::mounting::UiMountedFrameOutcome::Superseded(_) => {
            panic!("overlay rejection was superseded")
        }
    }
    assert_eq!(
        session.mounted.current_presentation_for_surface(surface),
        Some(presentation),
        "pre-effect rejection preserves the mounted presentation predecessor"
    );
    drop(rejected);
    assert_eq!(
        session
            .overlay_composition_owners
            .current_for_test(surface)
            .and_then(
                crate::runtime::overlay_composition::UiOverlayCompositionOwnerLifecycle::current
            ),
        Some(&overlay_predecessor),
        "pre-effect rejection discards the prepared overlay owner successor"
    );

    let frame = session
        .prepare_mounted_frame_with_application_presentation(
            crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
            |_| {},
        )
        .unwrap_or_else(|_| panic!("overlay appearance retry should prepare"));
    host.push_native_display_settled_without_effects();
    let outcome = session.present_prepared_mounted_frame_internal(
        frame,
        UiPresentationDeadline::at_tick(u64::MAX),
        2,
    );
    match outcome {
        crate::mounting::UiMountedFrameOutcome::Published(_) => {}
        crate::mounting::UiMountedFrameOutcome::RetentionDenied(rejection) => {
            panic!("overlay retention denied: {:?}", rejection.denial())
        }
        crate::mounting::UiMountedFrameOutcome::AdmissionDenied(rejection) => {
            panic!("overlay admission denied: {:?}", rejection.denial())
        }
        crate::mounting::UiMountedFrameOutcome::CompletionDenied(denial) => {
            panic!("overlay completion denied: {denial:?}")
        }
        crate::mounting::UiMountedFrameOutcome::RejectedBeforeEffects(_) => {
            panic!("overlay host rejected before effects")
        }
        crate::mounting::UiMountedFrameOutcome::Unchanged(_) => panic!("overlay was unchanged"),
        crate::mounting::UiMountedFrameOutcome::Reconciled(_) => panic!("overlay reconciled"),
        crate::mounting::UiMountedFrameOutcome::InFlight(_) => panic!("overlay remained in flight"),
        crate::mounting::UiMountedFrameOutcome::PresentationIndeterminate(_) => {
            panic!("overlay presentation became indeterminate")
        }
        crate::mounting::UiMountedFrameOutcome::Superseded(_) => panic!("overlay superseded"),
    }
    let output = session
        .mounted
        .current_unpublished_appearance()
        .unwrap()
        .expect("published Portal produces appearance work");
    let overlay = output
        .fragments()
        .iter()
        .find(|fragment| {
            fragment.identity() == UiUnpublishedAppearanceFragmentIdentity::SurfaceOverlay(surface)
        })
        .expect("the production path emits one surface overlay fragment");
    let mechanics = overlay.work().successor().mechanics();
    assert_eq!(mechanics.len(), 2);
    let backdrop = mechanics
        .iter()
        .find_map(|mechanic| match mechanic {
            UiMountedAppearanceMechanic::Backdrop(backdrop) => Some(backdrop),
            _ => None,
        })
        .expect("the overlay includes its authored backdrop");
    let portal_surface = mechanics
        .iter()
        .find_map(|mechanic| match mechanic {
            UiMountedAppearanceMechanic::PortalSurface(surface) => Some(surface),
            _ => None,
        })
        .expect("the overlay includes its Portal surface");
    assert_eq!(backdrop.semantic_surface(), surface);
    assert_eq!(backdrop.placement().ordinal(), 0);
    assert_eq!((backdrop.extent().x(), backdrop.extent().y()), (0, 0));
    assert_eq!(
        (backdrop.extent().width(), backdrop.extent().height()),
        (1_280_000, 720_000)
    );
    assert_eq!(backdrop.clip().x(), backdrop.extent().x());
    assert_eq!(backdrop.clip().y(), backdrop.extent().y());
    assert_eq!(backdrop.clip().width(), backdrop.extent().width());
    assert_eq!(backdrop.clip().height(), backdrop.extent().height());
    assert_eq!(backdrop.background().straight_srgba(), [4, 8, 12, 255]);
    assert_eq!(backdrop.opacity().units(), 32_768);
    assert_eq!(backdrop.attribution().semantic_surface(), surface);
    assert_eq!(
        backdrop.attribution().overlay_revision(),
        backdrop.placement().overlay_revision()
    );
    assert_eq!(
        backdrop.attribution().identity(),
        session
            .application
            .authored_overlay_material()
            .backdrop_declarations()[0]
            .declaration()
            .declaration()
            .identity()
            .value()
    );
    assert_eq!(
        backdrop.attribution().revision(),
        session
            .generation_identity()
            .semantic_package_identity()
            .narrowing_fingerprint()
            .max(1)
    );
    assert_eq!(portal_surface.portal_instance(), mounted);
    let backdrop_identity = backdrop.identity().clone();
    assert_eq!(
        overlay.work().successor().overlay_order().bottom_to_top(),
        &[
            worth_ui_host_contract::UiOverlayParticipantIdentity::Backdrop(
                backdrop_identity.clone(),
            ),
            worth_ui_host_contract::UiOverlayParticipantIdentity::Portal(mounted),
        ]
    );
    assert_eq!(overlay.work().damage().len(), 1);
    assert_eq!(overlay.work().damage()[0].x(), 0);
    assert_eq!(overlay.work().damage()[0].y(), 0);
    assert_eq!(overlay.work().damage()[0].width(), 1_280_000);
    assert_eq!(overlay.work().damage()[0].height(), 720_000);
    let transcript =
        worth_ui_host_headless::translate_unpublished_appearance_for_certification(output).unwrap();
    let raster = transcript
        .fragments()
        .iter()
        .find(|fragment| fragment.identity() == overlay.identity())
        .unwrap()
        .work()
        .successor()
        .reference_source_over();
    assert_eq!(
        raster.straight_srgba(),
        super::appearance_publication_support::expected_overlay_color(),
        "the translucent Portal sample depends on the Backdrop color and opacity"
    );
    let retained_overlay = session
        .overlay_composition_owners
        .current_for_test(surface)
        .unwrap();
    assert_eq!(
        retained_overlay
            .last_counters()
            .backdrop_declarations_selected(),
        1,
        "retry rebuilds the discarded initial candidate from the accepted predecessor"
    );
    assert_eq!(
        retained_overlay
            .last_counters()
            .unrelated_neighborhoods_touched(),
        0
    );
    assert_eq!(
        session
            .overlay_composition_owners
            .backdrop_work_for_test(surface)
            .unwrap()
            .roles_resolved,
        1,
        "rejection cannot install a reusable Backdrop projection"
    );

    super::appearance_publication_lifecycle::assert_unchanged_and_resize(
        &mut session,
        &host,
        surface,
    );
    super::appearance_publication_lifecycle::assert_close_removal(
        &mut session,
        &host,
        surface,
        portal,
    );
    let _ = session.shutdown();
}
