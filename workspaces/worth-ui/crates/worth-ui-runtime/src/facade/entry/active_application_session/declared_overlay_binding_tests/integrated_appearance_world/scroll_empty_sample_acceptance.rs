//! An empty Scroll paint/hit group still needs exact host acceptance, not
//! invented paint or a runtime-only fast path, before its pose becomes displayed.
use super::scroll_pose_authority::{block, ScrollWorld};
use super::scroll_settle_commit::{one_notch_up, pending_transitions, smooth_scroll, ONE_NOTCH};
use super::scroll_settle_frame::settle_scripted_frame;
use super::World;
use crate::certification_support::ScriptedSurfaceCompletion;
use crate::certification_support::{
    ScriptedPresentationAcknowledgement, ScriptedPresentationOutcome,
};
use crate::runtime::scroll::UiHostScrollObservationOutcome;
use worth_ui_host_contract::*;

fn no_paint_completion(epoch: u64) -> ScriptedPresentationAcknowledgement {
    ScriptedPresentationAcknowledgement::new(
        UiHostSurfacePresentationMode::NativeDisplay,
        UiHostPresentationEpoch::issued_by_host(epoch),
        UiMountedCompletedEffects::new(Vec::new()),
        UiHostPresentationCostReport::default(),
    )
}

fn stationary_owner_hit(scroll: &ScrollWorld) -> UiMountedCanonicalBox {
    scroll
        .world
        .session
        .mounted
        .interaction_hit_test_basis(scroll.presentation())
        .unwrap()
        .rows()
        .iter()
        .find(|row| row.mounted_instance() == scroll.target())
        .unwrap()
        .bounds()
        .platform_box()
}

#[test]
fn an_empty_smooth_group_rejects_waits_and_settles_through_no_paint_host_acceptance() {
    // The plain layout carries only dormant Portal child content, not the
    // painted nested component used by pixel-motion fixtures. That child has
    // neither retained paint nor a presented hit until its Portal opens.
    // There is no scrollbar chrome. The stationary owner remains reachable.
    let mut scroll = ScrollWorld::publish(World::launch_with_scroll(smooth_scroll(true)));
    let initial = (
        scroll.accepted_offset(),
        scroll.presentation(),
        stationary_owner_hit(&scroll),
    );
    assert!(matches!(
        scroll.wheel(ONE_NOTCH, one_notch_up(), 5),
        UiHostScrollObservationOutcome::Applied(_)
    ));
    let calls = scroll.world.host.presentation_calls();
    scroll.world.host.push_rejected();
    settle_scripted_frame(&mut scroll, 6);
    assert_eq!(
        scroll.world.host.presentation_calls(),
        calls + 1,
        "even an empty paint sample reaches the host"
    );
    assert_eq!(
        (
            scroll.accepted_offset(),
            scroll.presentation(),
            stationary_owner_hit(&scroll)
        ),
        initial
    );
    assert_eq!(pending_transitions(&scroll), 1);

    // The convenience script always issues epoch1. This same-frame sample
    // needs its own exact completion identity to prove basis succession.
    scroll
        .world
        .host
        .push_presentation(ScriptedPresentationOutcome::Presented(no_paint_completion(
            7,
        )));
    settle_scripted_frame(&mut scroll, 7);
    assert_eq!(scroll.world.host.presentation_calls(), calls + 2);
    assert_ne!(
        scroll.presentation(),
        initial.1,
        "retry acceptance replaces physical evidence"
    );
    let before = (scroll.accepted_offset(), scroll.mounted_offset());
    scroll.world.host.push_in_flight(
        vec![
            ScriptedSurfaceCompletion::Pending,
            ScriptedSurfaceCompletion::Presented(no_paint_completion(8)),
        ],
        UiHostSurfaceCancellationOutcome::CancelledBeforeEffects,
    );
    settle_scripted_frame(&mut scroll, 8);
    assert!(scroll
        .world
        .session
        .mounted
        .motion_sample_presentation_pending());
    assert_eq!((scroll.accepted_offset(), scroll.mounted_offset()), before);
    scroll.world.session.complete_motion_sample_presentation();
    assert!(scroll
        .world
        .session
        .mounted
        .motion_sample_presentation_pending());
    assert_eq!((scroll.accepted_offset(), scroll.mounted_offset()), before);
    scroll.world.session.complete_motion_sample_presentation();
    scroll.world.session.settle_owed_scroll_samples();
    assert!(!scroll
        .world
        .session
        .mounted
        .motion_sample_presentation_pending());
    assert!(scroll.accepted_offset().block_subpixels() > before.0.block_subpixels());
    assert_eq!(scroll.mounted_offset(), Some(scroll.accepted_offset()));
    assert_eq!(stationary_owner_hit(&scroll), initial.2);

    for tick in 9..=20 {
        if pending_transitions(&scroll) == 0 {
            break;
        }
        scroll
            .world
            .host
            .push_presentation(ScriptedPresentationOutcome::Presented(no_paint_completion(
                tick,
            )));
        settle_scripted_frame(&mut scroll, tick);
    }
    assert_eq!(scroll.accepted_offset(), block(10));
    assert_eq!(scroll.mounted_offset(), Some(block(10)));
    assert_eq!(stationary_owner_hit(&scroll), initial.2);
    assert_eq!(
        pending_transitions(&scroll),
        0,
        "no invisible Motion track retries forever"
    );
    assert_eq!(
        scroll
            .world
            .session
            .runtime_service_resource_census()
            .active_motion_tracks(),
        0
    );
    let _ = scroll.world.session.shutdown();
}
