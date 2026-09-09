use worth_ui_host_contract::{UiPresentationDeadline, UiUnpublishedAppearanceFragmentIdentity};

pub(super) fn assert_unchanged_and_resize(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    host: &crate::certification_support::ScriptedPresentationHost,
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
) {
    present_successor(session, host);
    let work = session
        .overlay_composition_owners
        .backdrop_work_for_test(surface)
        .unwrap();
    assert_eq!(work.candidates_visited, 1);
    assert_eq!(work.roles_resolved, 0);
    assert_eq!(
        session
            .overlay_composition_owners
            .current_for_test(surface)
            .unwrap()
            .last_counters()
            .backdrop_declarations_selected(),
        0,
        "unchanged retained overlay selects no dependent declarations"
    );
    let viewport = worth_ui_host_contract::UiMountedCanonicalBox::canonicalize(
        worth_ui_host_contract::UiMountedCanonicalBoxInput {
            x: 0.0,
            y: 0.0,
            width: 640.0,
            height: 480.0,
            coordinate_space: worth_ui_host_contract::UiMountedCoordinateSpace::HostSurface,
        },
    )
    .unwrap();
    let revision = session
        .mounted
        .next_occurrence_geometry_revision_for_test(surface)
        .get();
    crate::facade::entry::mounted_occurrence_geometry_test_support::
        install_surface_geometry_with_viewport(session, surface, revision, viewport);
    present_successor(session, host);
    let output = session
        .mounted
        .current_unpublished_appearance()
        .unwrap()
        .unwrap();
    let overlay = output
        .fragments()
        .iter()
        .find(|fragment| {
            fragment.identity() == UiUnpublishedAppearanceFragmentIdentity::SurfaceOverlay(surface)
        })
        .unwrap();
    let backdrop = overlay
        .work()
        .successor()
        .mechanics()
        .iter()
        .find_map(|mechanic| {
            if let worth_ui_host_contract::UiMountedAppearanceMechanic::Backdrop(backdrop) =
                mechanic
            {
                Some(backdrop)
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(
        (backdrop.extent().width(), backdrop.extent().height()),
        (640_000, 480_000)
    );
    assert_eq!(
        (backdrop.clip().width(), backdrop.clip().height()),
        (640_000, 480_000)
    );
    assert_eq!(overlay.work().damage().len(), 1);
    assert_eq!(
        (
            overlay.work().damage()[0].width(),
            overlay.work().damage()[0].height()
        ),
        (1_280_000, 720_000),
        "shrinking the viewport damages the complete old and new clipped coverage"
    );
    assert_eq!(
        session
            .overlay_composition_owners
            .backdrop_work_for_test(surface)
            .unwrap()
            .roles_resolved,
        0
    );
    assert_eq!(
        session
            .overlay_composition_owners
            .current_for_test(surface)
            .unwrap()
            .last_counters()
            .backdrop_declarations_selected(),
        1,
        "resize selects the dependent Backdrop while reusing its resolved appearance"
    );
    worth_ui_host_headless::translate_unpublished_appearance_for_certification(output).unwrap();
}

fn present_successor(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    host: &crate::certification_support::ScriptedPresentationHost,
) {
    let frame = session
        .prepare_mounted_frame_with_application_presentation(
            crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
            |_| {},
        )
        .unwrap_or_else(|_| panic!("overlay successor must prepare"));
    host.push_native_display_settled_without_effects();
    assert!(matches!(
        session.present_prepared_mounted_frame_internal(
            frame,
            UiPresentationDeadline::at_tick(u64::MAX),
            3,
        ),
        crate::mounting::UiMountedFrameOutcome::Published(_)
            | crate::mounting::UiMountedFrameOutcome::Unchanged(_)
    ));
}

pub(super) fn assert_close_removal(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    host: &crate::certification_support::ScriptedPresentationHost,
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    portal: crate::runtime::portal::UiPortalIdentity,
) {
    let close = crate::runtime::portal::UiPortalServiceRequest::close(
        portal,
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
    assert!(session
        .authored_overlay_binding_exports()
        .unwrap()
        .is_empty());

    let closed = session
        .prepare_mounted_frame_with_application_presentation(
            crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
            |_| {},
        )
        .unwrap_or_else(|_| panic!("Portal close appearance frame should prepare"));
    host.push_native_display_settled_without_effects();
    assert!(matches!(
        session.present_prepared_mounted_frame_internal(
            closed,
            UiPresentationDeadline::at_tick(u64::MAX),
            3,
        ),
        crate::mounting::UiMountedFrameOutcome::Published(_)
    ));
    let closed_output = session
        .mounted
        .current_unpublished_appearance()
        .unwrap()
        .expect("Portal close publishes overlay removal work");
    let closed_overlay = closed_output
        .fragments()
        .iter()
        .find(|fragment| {
            fragment.identity() == UiUnpublishedAppearanceFragmentIdentity::SurfaceOverlay(surface)
        })
        .expect("Portal close emits the retired surface overlay");
    assert_eq!(
        closed_overlay.work().posture(),
        worth_ui_host_contract::UiMountedAppearanceWorkPosture::Delta
    );
    assert!(closed_overlay.work().predecessor().is_some());
    assert!(closed_overlay.work().successor().mechanics().is_empty());
    assert!(closed_overlay
        .work()
        .successor()
        .overlay_order()
        .bottom_to_top()
        .is_empty());
    assert!(closed_overlay
        .work()
        .changes()
        .iter()
        .any(|change| matches!(
            change,
            worth_ui_host_contract::UiMountedAppearanceMechanicChange::Remove(_)
        )));
    worth_ui_host_headless::translate_unpublished_appearance_for_certification(closed_output)
        .unwrap();
}
