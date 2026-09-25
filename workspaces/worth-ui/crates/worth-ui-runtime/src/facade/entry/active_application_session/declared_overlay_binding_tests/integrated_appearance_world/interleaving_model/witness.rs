//! The invariant every step of the interleaving model must leave true: the
//! geometry each reader displays or acts on is the geometry the latest
//! witness proved.
//!
//! Motion shows the latest pose the host accepted. Scroll, mounted geometry,
//! both hit-test lanes and the cursor read the last settle that landed. The
//! two agree unless a settle is owed, and an owed settle is only ever the
//! recorded lag of a deferral. The host's own drawn text is the oracle for
//! what is on screen.

use super::super::super::super::scroll_direct_control::scroll_content_motion_target;
use super::super::super::super::UiScrollSettleDisposition;
use super::super::scroll_hover_reresolution::{hovered, RESTING_POINT};
use super::steps::Model;
use crate::mounting::presentation::{UiDisplayedRect, UiPlatformPoint};
use crate::mounting::UiHitTestSpatialWork;
use worth_ui_host_contract::{UiHostSurfacePosition, UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT};

/// The scrollable region's top edge on the surface.
const REGION_TOP: f32 = 50.0;
/// The nested component's top edge on the surface with the content at rest.
const NESTED_AT_REST: f32 = 62.0;
/// The nested component's height: once its bottom edge passes the region's
/// top edge it is clipped out, and the host draws nothing of it.
const NESTED_HEIGHT: f32 = 8.0;
/// Committed geometry and Scroll offsets are exact to a subpixel.
const EXACT: f32 = 1e-3;
/// The host draws a sample snapped to whole device pixels.
const SNAPPED: f32 = 0.5 + EXACT;

/// A state the witness tells apart. Every one must be reached somewhere
/// across the runs, so no branch of the witness goes unexercised.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum Reached {
    SampleOnScreen,
    OwedSettle,
    StagedResize,
    PointerOverNested,
}

impl Reached {
    pub(super) const EVERY: [Self; 4] = [
        Self::SampleOnScreen,
        Self::OwedSettle,
        Self::StagedResize,
        Self::PointerOverNested,
    ];
}

/// Assert the invariant, and name the states this step left the model in.
pub(super) fn assert_witnessed(model: &Model, label: &str) -> Vec<Reached> {
    let scroll = &model.scroll;
    let session = &scroll.world.session;
    let host = &scroll.world.host;
    let nested = scroll.world.instances[2];
    assert_eq!(
        host.pending_presentation_count(),
        0,
        "{label}: every scripted host call was made"
    );

    // Scroll and mounted geometry hold one pose. A staged resize is the
    // layout the next frame prepares, clamped to its own travel; no witness
    // has shown it, so only the retained record below is read under it.
    let accepted = scroll.accepted_offset();
    if !model.staged {
        assert_eq!(
            scroll.mounted_offset(),
            Some(accepted),
            "{label}: Scroll and mounted geometry hold one pose"
        );
    }
    let pose =
        accepted.block_subpixels() as f32 / UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT as f32;

    // Motion shows a sample the latest witness displays, at the settled pose
    // unless a deferral still owes its settle.
    let displayed = session
        .mounted
        .current_displayed_presentation(scroll.presentation())
        .expect("the retained record displays the current basis");
    let motion = scroll_content_motion_target(scroll.owner, scroll.target());
    let shown = match session.mounted.accepted_scroll_group_sample(motion) {
        Some(sample) => {
            UiDisplayedRect::displayed(sample, displayed).unwrap_or_else(|denial| {
                panic!(
                    "{label}: the Motion sample is not displayed by the latest witness: {denial:?}"
                )
            });
            REGION_TOP - sample.components()[1]
        }
        None => pose,
    };
    if session.awaits_scroll_settle_retry() {
        assert!(
            matches!(
                session.last_scroll_settle_disposition(),
                UiScrollSettleDisposition::DeferredPendingGeometry
                    | UiScrollSettleDisposition::DeferredPresentationInFlight
            ),
            "{label}: only a deferral owes a settle: {:?}",
            session.last_scroll_settle_disposition()
        );
    } else {
        assert!(
            (shown - pose).abs() <= EXACT,
            "{label}: Motion shows {shown} while Scroll settled {pose}"
        );
    }

    // The host draws the content where Motion says it is shown.
    let drawn = host
        .accepted_text_drawn_bounds(scroll.surface())
        .into_iter()
        .filter(|(identity, _)| identity.mounted_instance() == nested)
        .map(|(_, bounds)| bounds.y())
        .collect::<Vec<_>>();
    let clipped = |pose: f32| NESTED_AT_REST - pose + NESTED_HEIGHT <= REGION_TOP;
    assert!(
        !drawn.is_empty() || clipped(shown) || clipped(pose),
        "{label}: the host draws the nested text while it is in view (shown {shown}, settled {pose})"
    );
    for &y in &drawn {
        assert!(
            (y - (NESTED_AT_REST - shown)).abs() <= SNAPPED,
            "{label}: the host draws the nested text at {y}, Motion shows it at {}",
            NESTED_AT_REST - shown
        );
    }

    // Both hit-test lanes agree, and put the nested row where Scroll settled
    // it or exactly where the host draws it. A settle moves hit rows by the
    // exact offset; a publication measures them from the commands the host
    // draws, and a sample it displayed left those on the device grid.
    let presentation = scroll.presentation();
    let basis_row = session
        .mounted
        .interaction_hit_test_basis(presentation)
        .unwrap_or_else(|denial| panic!("{label}: the interaction basis reads: {denial:?}"))
        .rows()
        .iter()
        .find(|row| row.mounted_instance() == nested)
        .copied()
        .unwrap_or_else(|| panic!("{label}: the interaction basis carries the nested row"));
    let mut work = UiHitTestSpatialWork::default();
    let index_row = session
        .mounted
        .current_presented_hit_row(presentation, nested, &mut work)
        .unwrap_or_else(|denial| panic!("{label}: the hit index reads: {denial:?}"));
    assert_eq!(
        basis_row.bounds(),
        index_row.bounds(),
        "{label}: both hit-test lanes hold one nested row"
    );
    let y = index_row.bounds().platform_box().y();
    let settled = NESTED_AT_REST - pose;
    assert!(
        std::iter::once(settled)
            .chain(drawn.iter().copied())
            .any(|at| (y - at).abs() <= EXACT),
        "{label}: the hit lanes put the nested row at {y}, Scroll settled it at {settled}, the host draws it at {drawn:?}"
    );

    // The cursor is over what a click at the resting point would reach.
    let point = UiPlatformPoint::from_host_position(UiHostSurfacePosition::viewport_logical(
        RESTING_POINT[0],
        RESTING_POINT[1],
    ))
    .expect("the resting point is a logical viewport position");
    let expected = if index_row.bounds().admits_platform_point(point) {
        nested
    } else {
        scroll.target()
    };
    assert_eq!(
        hovered(scroll),
        Some(expected),
        "{label}: the cursor is over what a click would reach"
    );
    [
        (
            session
                .mounted
                .accepted_scroll_group_sample(motion)
                .is_some(),
            Reached::SampleOnScreen,
        ),
        (session.awaits_scroll_settle_retry(), Reached::OwedSettle),
        (model.staged, Reached::StagedResize),
        (expected == nested, Reached::PointerOverNested),
    ]
    .into_iter()
    .filter_map(|(reached, state)| reached.then_some(state))
    .collect()
}
