//! Lowering a settled-toward target into a Motion request: the target it
//! addresses, the geometry it hands over, and the horizon it declares.

use super::*;

const SUBPIXELS_PER_POINT: f32 = 1_000.0;
const SETTLE_TICKS: u32 = 8;

fn surface() -> worth_ui_host_contract::UiSemanticSurfaceIdentity {
    worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().expect("surface")
}

fn presentation() -> worth_ui_host_contract::UiHostObservationPresentationBasis {
    let frame = worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().expect("frame");
    worth_ui_host_contract::UiHostObservationPresentationBasis::new(
        worth_ui_host_contract::UiHostSurfaceIdentity::mint_unbound().expect("host surface"),
        frame,
        worth_ui_host_contract::UiSurfaceBindingGeneration::mint_unbound().expect("binding"),
        worth_ui_host_contract::UiHostPresentationEpoch::issued_by_host(1),
    )
}

fn content() -> worth_ui_host_contract::UiMountedCanonicalBox {
    worth_ui_host_contract::UiMountedCanonicalBox::canonicalize(
        worth_ui_host_contract::UiMountedCanonicalBoxInput {
            x: 10.0,
            y: 20.0,
            width: 900.0,
            height: 672.0,
            coordinate_space: worth_ui_host_contract::UiMountedCoordinateSpace::Viewport,
        },
    )
    .expect("canonical content box")
}

fn binding() -> UiScrollMotionBinding {
    UiScrollMotionBinding::new(
        worth_ui_host_contract::UiMountedInstanceIdentity::mint_unbound().expect("instance"),
        7,
        1,
        2,
        presentation(),
        content(),
    )
}

fn target(
    owner: crate::runtime::scroll::UiScrollOwnerIdentity,
    block_subpixels: i64,
    latest_input_tick: u64,
) -> UiScrollTransitionTarget {
    UiScrollTransitionTarget::new(
        owner,
        crate::runtime::scroll::UiScrollOwnerIncarnation::new(1).expect("incarnation"),
        crate::runtime::scroll::UiScrollOffset::new(0, block_subpixels).expect("offset"),
        UiScrollSettleHorizon::from_latest_input(latest_input_tick, SETTLE_TICKS).expect("horizon"),
    )
}

/// A positive scroll offset moves content toward the viewport origin, so the
/// request hands Motion a predecessor and successor translated by the negated
/// accepted and target offsets.
#[test]
fn the_request_translates_content_by_the_negated_accepted_and_target_offsets() {
    let owner = crate::runtime::scroll::UiScrollOwnerIdentity::viewport(surface());
    let accepted = crate::runtime::scroll::UiScrollOffset::new(0, 30_000).expect("offset");
    let request = scroll_settle_motion_request(target(owner, 90_000, 10), accepted, 10, binding())
        .expect("an unexhausted horizon lowers");

    let content = content();
    assert_eq!(
        request
            .predecessor()
            .geometry()
            .expect("predecessor geometry")
            .components(),
        [
            content.x(),
            content.y() - 30_000.0 / SUBPIXELS_PER_POINT,
            content.width(),
            content.height()
        ]
    );
    assert_eq!(
        request
            .successor()
            .geometry()
            .expect("successor geometry")
            .components(),
        [
            content.x(),
            content.y() - 90_000.0 / SUBPIXELS_PER_POINT,
            content.width(),
            content.height()
        ]
    );
}

/// The request addresses the scrolled content group of this exact occurrence,
/// structurally distinct from the occurrence's ordinary and Portal targets.
#[test]
fn the_request_addresses_the_scroll_contents_target_of_this_occurrence() {
    let owner = crate::runtime::scroll::UiScrollOwnerIdentity::viewport(surface());
    let request = scroll_settle_motion_request(
        target(owner, 90_000, 10),
        crate::runtime::scroll::UiScrollOffset::origin(),
        10,
        binding(),
    )
    .expect("lowered");

    assert_eq!(
        request.successor().target().scope(),
        crate::runtime::motion::UiMotionTargetScope::ScrollContents
    );
    assert_eq!(
        request.successor().target().semantic_surface(),
        owner.semantic_surface()
    );
}

/// The declared settle length is what the horizon has left at this tick, not a
/// fixed duration, so a notch arriving mid-settle extends one transition.
#[test]
fn the_declared_settle_length_is_the_horizon_remaining_at_this_tick() {
    let owner = crate::runtime::scroll::UiScrollOwnerIdentity::viewport(surface());
    let staged = target(owner, 90_000, 10);
    let accepted = crate::runtime::scroll::UiScrollOffset::origin();

    for (tick, remaining) in [(10, SETTLE_TICKS), (14, SETTLE_TICKS - 4)] {
        let request =
            scroll_settle_motion_request(staged, accepted, tick, binding()).expect("lowered");
        assert_eq!(request.declaration().duration_ticks(), remaining);
    }

    assert_eq!(
        scroll_settle_motion_request(staged, accepted, 10 + u64::from(SETTLE_TICKS), binding()),
        Err(UiScrollMotionRequestDenial::SettleHorizonExhausted)
    );
}
