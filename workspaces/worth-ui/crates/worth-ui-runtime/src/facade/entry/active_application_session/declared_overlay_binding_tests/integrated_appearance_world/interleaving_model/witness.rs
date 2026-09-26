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
use crate::mounting::{UiHitTestSpatialWork, UiPresentedFrameBasisDenial};
use crate::runtime::motion::UiMotionTargetIdentity;
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
    /// A settle is owed behind a presentation attempt in flight.
    OwedBehindPresentation,
    /// A settle is owed behind geometry no witness has shown.
    OwedBehindGeometry,
    /// A settle whose Motion has arrived is owed behind a staged resize: the
    /// resize must land the content where the arrival was shown.
    ArrivedBehindResize,
    /// Hit rows lead to a sample the host shows away from the settled pose.
    HitLeadsSettle,
    StagedResize,
    /// A page placed directly, which no publication has landed yet.
    StagedDirect,
    PointerOverNested,
}

impl Reached {
    pub(super) const EVERY: [Self; 8] = [
        Self::SampleOnScreen,
        Self::OwedBehindPresentation,
        Self::OwedBehindGeometry,
        Self::ArrivedBehindResize,
        Self::HitLeadsSettle,
        Self::StagedResize,
        Self::StagedDirect,
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
    // layout the next frame prepares, clamped to its own travel, and a staged
    // page the offset it prepares; no witness has shown either, so only the
    // retained record below is read under them.
    let accepted = scroll.accepted_offset();
    match (model.staged, model.direct) {
        (true, _) => {}
        (false, Some(direct)) => assert_eq!(
            scroll.mounted_offset(),
            Some(direct),
            "{label}: mounted geometry holds the staged page"
        ),
        (false, None) => assert_eq!(
            scroll.mounted_offset(),
            Some(accepted),
            "{label}: Scroll and mounted geometry hold one pose"
        ),
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

    // Both hit-test lanes agree. While a settle is owed they put the nested
    // row where Motion shows it, and otherwise where Scroll settled it, or in
    // either case exactly where the host draws it. A settle moves hit rows by
    // the exact offset, and a lead by the exact offset of the sample the host
    // shows; a publication measures them from the commands the host draws,
    // and a sample it displayed left those on the device grid. The lanes
    // carry the row exactly while it is in view there, as the host draws
    // nothing of it once the region has clipped it out.
    let owed = session.awaits_scroll_settle_retry();
    let hit_at = if owed { shown } else { pose };
    let leads = owed && (shown - pose).abs() > EXACT;
    let presentation = scroll.presentation();
    let basis_row = session
        .mounted
        .interaction_hit_test_basis(presentation)
        .unwrap_or_else(|denial| panic!("{label}: the interaction basis reads: {denial:?}"))
        .rows()
        .iter()
        .find(|row| row.mounted_instance() == nested)
        .copied();
    let mut work = UiHitTestSpatialWork::default();
    let index_row = match session
        .mounted
        .current_presented_hit_row(presentation, nested, &mut work)
    {
        Ok(row) => Some(row),
        Err(UiPresentedFrameBasisDenial::InstanceNotPresented) => None,
        Err(denial) => panic!("{label}: the hit index reads: {denial:?}"),
    };
    assert_eq!(
        basis_row.map(|row| row.bounds()),
        index_row.map(|row| row.bounds()),
        "{label}: both hit-test lanes hold one nested row"
    );
    let expected_y = NESTED_AT_REST - hit_at;
    let Some(index_row) = index_row else {
        assert!(
            clipped(hit_at),
            "{label}: the hit lanes carry no nested row, but it stands in view at {expected_y} (owed {owed})"
        );
        assert_eq!(
            hovered(scroll),
            Some(scroll.target()),
            "{label}: the cursor is over the region once its content is clipped out"
        );
        return reached(model, motion, leads, false);
    };
    assert!(
        !clipped(hit_at),
        "{label}: the hit lanes carry the nested row, but it is clipped out at {expected_y} (owed {owed})"
    );
    let y = index_row.bounds().platform_box().y();
    assert!(
        std::iter::once(expected_y)
            .chain(drawn.iter().copied())
            .any(|at| (y - at).abs() <= EXACT),
        "{label}: the hit lanes put the nested row at {y}, it stands at {expected_y} (owed {owed}), the host draws it at {drawn:?}"
    );

    // The cursor is over what a click at the resting point would reach.
    let point = UiPlatformPoint::from_host_position(UiHostSurfacePosition::viewport_logical(
        RESTING_POINT[0],
        RESTING_POINT[1],
    ))
    .expect("the resting point is a logical viewport position");
    let expected = if index_row.bounds().admits_platform_point(point)
        && index_row.ancestor_reach().admits_platform_point(point)
    {
        nested
    } else {
        scroll.target()
    };
    assert_eq!(
        hovered(scroll),
        Some(expected),
        "{label}: the cursor is over what a click would reach"
    );
    reached(model, motion, leads, expected == nested)
}

/// The states a step left the model in.
fn reached(
    model: &Model,
    motion: UiMotionTargetIdentity,
    leads: bool,
    pointer_over_nested: bool,
) -> Vec<Reached> {
    let session = &model.scroll.world.session;
    let owed = |deferral| {
        session.awaits_scroll_settle_retry() && session.last_scroll_settle_disposition() == deferral
    };
    [
        (
            session
                .mounted
                .accepted_scroll_group_sample(motion)
                .is_some(),
            Reached::SampleOnScreen,
        ),
        (
            owed(UiScrollSettleDisposition::DeferredPresentationInFlight),
            Reached::OwedBehindPresentation,
        ),
        (
            owed(UiScrollSettleDisposition::DeferredPendingGeometry),
            Reached::OwedBehindGeometry,
        ),
        (
            owed(UiScrollSettleDisposition::DeferredPendingGeometry)
                && model.staged
                && session
                    .motion
                    .as_ref()
                    .is_some_and(|motion_state| motion_state.committed_track(motion).is_none()),
            Reached::ArrivedBehindResize,
        ),
        (leads, Reached::HitLeadsSettle),
        (model.staged, Reached::StagedResize),
        (model.direct.is_some(), Reached::StagedDirect),
        (pointer_over_nested, Reached::PointerOverNested),
    ]
    .into_iter()
    .filter_map(|(reached, state)| reached.then_some(state))
    .collect()
}
