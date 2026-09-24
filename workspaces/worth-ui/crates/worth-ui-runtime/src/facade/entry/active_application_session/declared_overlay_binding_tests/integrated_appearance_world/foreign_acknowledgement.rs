//! A host acknowledges only the work the runtime issued it.
//!
//! The scripted host here builds its own consumption view from the issued
//! one's identity -- the same attempt, requirement and frame -- and
//! acknowledges that instead. Every identity the completion names matches, but
//! no live lease sealed the view, so the coordinator cannot admit it: nothing
//! publishes, and since the host may have painted, the runtime stops claiming
//! to know what the surface shows rather than crediting the forged frame. The
//! same acknowledgement minted from the issued view publishes, so the seal is
//! the only thing the forgery lacked.

use super::scroll_pose_authority::ScrollWorld;
use super::session::World;
use crate::certification_support::{
    scripted_presentation_epoch, ScriptedPresentationAcknowledgement, ScriptedPresentationOutcome,
};
use crate::mounting::UiMountedFrameOutcome;
use worth_ui_host_contract::*;

fn settled_without_effects() -> ScriptedPresentationAcknowledgement {
    ScriptedPresentationAcknowledgement::new(
        UiHostSurfacePresentationMode::NativeDisplay,
        scripted_presentation_epoch(),
        UiMountedCompletedEffects::new(Vec::new()),
        UiHostPresentationCostReport::default(),
    )
}

/// Publish a fresh World, then present its first surface once more against a
/// host that answers with `outcome`.
fn republish(
    outcome: ScriptedPresentationOutcome,
) -> (World, UiMountedFrameIdentity, UiMountedFrameOutcome) {
    let mut world = ScrollWorld::launch_published().world;
    let publication = world.session.current_mounted_publication().unwrap().frame();
    world.host.push_presentation(outcome);
    let frame = world.prepare_surface(world.surfaces[0]);
    let outcome = world.session.present_prepared_mounted_frame_internal(
        frame,
        UiPresentationDeadline::at_tick(u64::MAX),
        2,
    );
    (world, publication, outcome)
}

#[test]
fn a_host_acknowledging_a_view_it_built_itself_publishes_nothing() {
    let (world, publication, forged) = republish(
        ScriptedPresentationOutcome::PresentedFromForeignView(settled_without_effects()),
    );
    assert!(
        matches!(forged, UiMountedFrameOutcome::PresentationIndeterminate(_)),
        "an acknowledgement no lease issued is not displayed truth: {:?}",
        std::mem::discriminant(&forged)
    );
    assert_eq!(
        world.session.current_mounted_publication().unwrap().frame(),
        publication
    );
    assert!(
        world
            .session
            .mounted
            .current_presentation_for_surface(world.surfaces[0])
            .is_none(),
        "an unadmitted acknowledgement leaves the surface's display unknown"
    );
    let _ = world.session.shutdown();
}

#[test]
fn the_same_acknowledgement_of_the_issued_view_publishes() {
    let (world, publication, issued) = republish(ScriptedPresentationOutcome::Presented(
        settled_without_effects(),
    ));
    assert!(
        matches!(issued, UiMountedFrameOutcome::Published(_)),
        "{:?}",
        std::mem::discriminant(&issued)
    );
    assert_ne!(
        world.session.current_mounted_publication().unwrap().frame(),
        publication
    );
    assert!(world
        .session
        .mounted
        .current_presentation_for_surface(world.surfaces[0])
        .is_some());
    let _ = world.session.shutdown();
}
