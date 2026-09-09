use super::super::fixture as appearance;
use super::*;
use crate::runtime::motion::{UiMotionDeclaration, UiMotionTransitionRequest};

pub(super) fn mounted() -> (
    WorthUiActiveApplicationSession,
    ScriptedPresentationHost,
    UiSemanticSurfaceIdentity,
    UiMountedPaintCommandIdentity,
) {
    let role = appearance::role();
    let (mut session, host) = appearance::session_with_motion(&role);
    let (surface, _) = super::super::super::mounting_fixture::mount(&mut session, 1_000);
    appearance::close_source_with_motion(&mut session, &role, "motion-acceptance-initial");
    crate::facade::entry::mounted_occurrence_geometry_test_support::refresh_nonoverlapping_surface_geometry(
        &mut session,
        surface,
    );
    session.advance_mounted_identity_frame().unwrap();
    let frame = session
        .prepare_mounted_frame_with_application_presentation(
            crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
            |_| {},
        )
        .unwrap_or_else(|_| panic!("initial mounted frame prepares"));
    if let Err(denial) = session.mounted.current_unpublished_appearance() {
        panic!("initial Motion appearance unavailable: {denial:?}");
    }
    let command = frame.surfaces()[0].projection().authored_paint_commands()[0].identity();
    appearance::publish(&mut session, &host, frame, 1);
    (session, host, surface, command)
}

pub(super) fn install(
    session: &mut WorthUiActiveApplicationSession,
    target: UiMotionTargetIdentity,
    presentation: UiHostObservationPresentationBasis,
    identity: u64,
    entering: bool,
    retarget: Option<crate::runtime::motion::UiMotionRetargetDisposition>,
) {
    let request = UiMotionTransitionRequest::from_family_transition(
        target,
        1,
        2,
        presentation,
        None,
        !entering,
        presentation,
        None,
        entering,
        if entering {
            UiMotionDeclaration::portal_entrance()
        } else {
            UiMotionDeclaration::portal_exit()
        },
    )
    .unwrap();
    let receipt = session
        .motion
        .as_mut()
        .expect("Motion is installed")
        .commit_declared_transition_for_test(identity, request, presentation.frame(), presentation);
    if let Some(expected) = retarget {
        assert_eq!(receipt.track().retarget(), Some(expected));
    }
    session.mounted.install_motion_commit(receipt).unwrap();
}

pub(super) fn present(
    session: &mut WorthUiActiveApplicationSession,
    host: &ScriptedPresentationHost,
    surface: UiSemanticSurfaceIdentity,
    tick: u64,
    epoch: u64,
) {
    let basis = session
        .mounted
        .current_presentation_for_surface(surface)
        .unwrap();
    let prepared = session.mounted.prepare_motion_tick(tick, basis).unwrap();
    host.push_presentation(UiHostSurfacePresentationOutcome::Presented(completion(
        epoch, true,
    )));
    assert!(
        matches!(
            session
                .mounted
                .present_prepared_motion_tick(&session.host_session, prepared, basis),
            UiMountedMotionSampleSettlement::Committed(_)
        ),
        "sample must physically settle"
    );
}

pub(super) fn refresh_appearance(
    session: &mut WorthUiActiveApplicationSession,
    host: &ScriptedPresentationHost,
    now: u64,
) {
    let frame = session
        .prepare_mounted_frame_with_application_presentation(
            crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
            |_| {},
        )
        .unwrap_or_else(|_| panic!("appearance refresh prepares"));
    let invalidation = frame
        .appearance_invalidation_batch()
        .expect("accepted Motion selects its retained appearance input");
    assert!(invalidation.is_physical_input_only());
    assert_eq!(invalidation.selected_count(), 1);
    appearance::publish(session, host, frame, now);
}

pub(super) fn completion(epoch: u64, paint: bool) -> UiMountedSurfacePresentationCompletion {
    UiMountedSurfacePresentationCompletion::new(
        UiHostSurfacePresentationMode::NativeDisplay,
        UiHostPresentationEpoch::issued_by_host(epoch),
        UiMountedCompletedEffects::new(if paint {
            vec![UiMountedEffectFamily::NativePaint]
        } else {
            vec![]
        }),
        UiHostPresentationCostReport::default(),
    )
}
