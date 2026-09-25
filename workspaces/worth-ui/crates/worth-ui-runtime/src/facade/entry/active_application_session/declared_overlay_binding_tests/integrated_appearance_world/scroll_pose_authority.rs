//! Facade-level proof that a Scroll route commits only the pose mounted
//! geometry displays.
//!
//! The shared authored World supplies what the unit worlds cannot: a Scroll
//! service installed from policy, a region occurrence with real mounted
//! geometry at a nonzero origin, and a scripted host that can hold a
//! presentation attempt open. Every path that moves an accepted offset is
//! driven here through its production entry and checked against both the
//! semantic offset Scroll holds and the pose geometry displays.

use super::World;
use crate::mounting::presentation::displayed_rect_for_test;
use crate::mounting::{UiMountedFrameOutcome, UiMountedOccurrenceGeometryDenial};
use crate::runtime::motion::UiMotionTargetIdentity;
use crate::runtime::scroll::{
    UiHostScrollObservationDenial, UiHostScrollObservationOutcome, UiScrollChainEntry,
    UiScrollDeltaCause, UiScrollOffset, UiScrollOwnerIdentity, UiScrollOwnerIncarnation,
};
use worth_ui_host_contract::*;

/// A point inside the first component's box, so a wheel there addresses the
/// region that component owns.
const WHEEL_POSITION: [i64; 2] = [150_000, 55_000];
const SUBPIXELS: i64 = UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT;

/// The shared World with its first component's region made scrollable and
/// one frame published, plus the Scroll owner that region resolves to.
pub(super) struct ScrollWorld {
    pub(super) world: World,
    pub(super) owner: UiScrollOwnerIdentity,
    pub(super) incarnation: UiScrollOwnerIncarnation,
}

impl ScrollWorld {
    pub(super) fn launch_published() -> Self {
        Self::publish_with_nested_content(World::launch())
    }

    /// The same World with the third component laid out inside the scrollable
    /// region, so the region has presented content of its own to carry.
    pub(super) fn publish_with_nested_content(mut world: World) -> Self {
        super::geometry::scrollable::install_scrollable_primary_with_nested_content(
            &mut world.session,
            world.surfaces,
            world.instances,
        );
        Self::publish_installed(world)
    }

    pub(super) fn publish(mut world: World) -> Self {
        super::geometry::scrollable::install_scrollable_primary(
            &mut world.session,
            world.surfaces,
            world.instances,
        );
        Self::publish_installed(world)
    }

    fn publish_installed(mut world: World) -> Self {
        let frame = world.prepare();
        world.publish(frame, 1, true);
        let target = world.instances[0];
        let owner = world
            .session
            .scroll
            .as_ref()
            .expect("the World installs Scroll from policy")
            .ownership_chain(target)
            .expect("the first component resolves its Scroll chain")
            .owners()[0];
        assert!(
            matches!(owner, UiScrollOwnerIdentity::Region { .. }),
            "the innermost owner is the declared region: {owner:?}"
        );
        let incarnation = world
            .session
            .scroll_region_incarnation(target, 0)
            .expect("the region occurrence has a current allocation");
        Self {
            world,
            owner,
            incarnation,
        }
    }

    pub(super) fn target(&self) -> UiMountedInstanceIdentity {
        self.world.instances[0]
    }

    pub(super) fn surface(&self) -> UiSemanticSurfaceIdentity {
        self.world.surfaces[0]
    }

    /// The basis the host is presenting this surface under right now. A
    /// scenario that holds one across a republication holds a stale one, which
    /// is how a report arriving from a frame the reader has already left is
    /// written down.
    pub(super) fn presentation(&self) -> UiHostObservationPresentationBasis {
        self.world
            .session
            .mounted
            .current_presentation_for_surface(self.surface())
            .expect("the first surface is published")
            .basis()
    }

    /// One wheel report over the first component. The host reports the
    /// wheel's own travel, so content the reader pushes toward the top arrives
    /// as a negative block delta; the offset it produces is positive.
    pub(super) fn wheel(
        &mut self,
        precision: UiHostScrollDeltaPrecision,
        y_subpixels: i64,
        tick: u64,
    ) -> UiHostScrollObservationOutcome {
        self.targeted_wheel(
            UiHostScrollDeltaPhase::Updated,
            self.pointer_target(),
            precision,
            y_subpixels,
            tick,
        )
    }

    /// One wheel report with a chosen phase and a chosen target affinity, so a
    /// scenario can say what the host was able to tell the runtime about where
    /// the gesture landed.
    pub(super) fn targeted_wheel(
        &mut self,
        phase: UiHostScrollDeltaPhase,
        target: UiHostScrollDeltaTargetAffinity,
        precision: UiHostScrollDeltaPrecision,
        y_subpixels: i64,
        tick: u64,
    ) -> UiHostScrollObservationOutcome {
        let payload = UiHostObservationPayload::ScrollDelta {
            source: UiHostScrollDeltaSource::PointerWheel,
            phase,
            precision,
            target,
            x_subpixels: 0,
            y_subpixels,
        };
        self.world
            .session
            .observe_scroll_payload(&payload, &mut Default::default(), Some(tick))
            .expect("a ScrollDelta payload is a Scroll observation")
    }

    /// A target the host resolved to a coordinate over the scrollable region.
    pub(super) fn pointer_target(&self) -> UiHostScrollDeltaTargetAffinity {
        UiHostScrollDeltaTargetAffinity::exact_coordinate(
            self.presentation(),
            UiHostSurfacePosition::viewport_logical(WHEEL_POSITION[0], WHEEL_POSITION[1]),
        )
    }

    /// A target naming only the presented surface. Nothing in it says which
    /// owner the gesture belongs to, so it is answerable only by a latch.
    pub(super) fn surface_only_target(&self) -> UiHostScrollDeltaTargetAffinity {
        UiHostScrollDeltaTargetAffinity::presented_surface_fallback(self.presentation())
    }

    pub(super) fn accepted_offset(&self) -> UiScrollOffset {
        self.world
            .session
            .scroll
            .as_ref()
            .expect("Scroll stays installed")
            .offset(self.owner, self.incarnation)
            .expect("the routed owner keeps its offset")
    }

    /// Publish the direct candidate through the actual NativeDisplay host
    /// acceptance boundary; observing a delta alone never calls this implicitly.
    pub(super) fn publish_direct(&mut self, tick: u64) {
        let frame = self.world.prepare_surface(self.surface());
        self.world.publish(frame, tick, true);
    }

    pub(super) fn mounted_offset(&self) -> Option<UiScrollOffset> {
        self.world
            .session
            .mounted
            .mounted_scroll_pose(self.target(), self.target())
    }

    /// Present the first surface and leave the host holding the completion.
    pub(super) fn hold_presentation_open(
        &mut self,
        now: u64,
    ) -> crate::mounting::UiMountedPresentationInFlight {
        let frame = self.world.prepare_surface(self.surface());
        let pending = self.world.begin_in_flight(frame, now);
        assert!(self.world.session.mounted.has_active_presentation_attempt());
        pending
    }

    pub(super) fn complete(
        &mut self,
        pending: crate::mounting::UiMountedPresentationInFlight,
        now: u64,
    ) {
        assert!(matches!(
            self.world
                .session
                .complete_mounted_presentation(pending, now),
            UiMountedFrameOutcome::Published(_)
        ));
    }
}

pub(super) fn block(points: i64) -> UiScrollOffset {
    UiScrollOffset::new(0, points * SUBPIXELS).expect("a nonnegative offset")
}

fn pixels(points: i64) -> i64 {
    -points * SUBPIXELS
}

#[test]
fn an_immediate_wheel_commits_only_the_pose_geometry_displays() {
    let mut scroll = ScrollWorld::launch_published();
    assert_eq!(scroll.accepted_offset(), block(0));

    let UiHostScrollObservationOutcome::Applied(receipt) =
        scroll.wheel(UiHostScrollDeltaPrecision::Pixel, pixels(20), 5)
    else {
        panic!("an immediate wheel over scrollable content applies")
    };
    assert_eq!(receipt.transitions()[0].current(), block(20));
    assert_eq!(scroll.accepted_offset(), block(0));
    scroll.publish_direct(5);
    assert_eq!(scroll.accepted_offset(), block(20));
    assert_eq!(scroll.mounted_offset(), Some(block(20)));

    // With the host holding a presentation attempt open, geometry refuses the
    // next pose. The route that named it is not committed either: the offset
    // Scroll holds is the one the displayed pose was accepted at.
    let pending = scroll.hold_presentation_open(6);
    assert_eq!(
        scroll.wheel(UiHostScrollDeltaPrecision::Pixel, pixels(5), 7),
        UiHostScrollObservationOutcome::Denied(UiHostScrollObservationDenial::Geometry(
            UiMountedOccurrenceGeometryDenial::PresentationInFlight
        ))
    );
    assert_eq!(scroll.accepted_offset(), block(20));
    assert_eq!(scroll.mounted_offset(), Some(block(20)));

    // Once the host completes, the same wheel applies from the offset that was
    // never moved, not from one the refused route would have left behind.
    scroll.complete(pending, 8);
    assert!(matches!(
        scroll.wheel(UiHostScrollDeltaPrecision::Pixel, pixels(5), 9),
        UiHostScrollObservationOutcome::Applied(_)
    ));
    assert_eq!(scroll.accepted_offset(), block(20));
    // The painted child is already entirely above the viewport at offset 20.
    // This successor changes accepted geometry but owes no visible paint.
    let frame = scroll.world.prepare_surface(scroll.surface());
    scroll.world.publish(frame, 10, false);
    assert_eq!(scroll.accepted_offset(), block(25));
    assert_eq!(scroll.mounted_offset(), Some(block(25)));
    let _ = scroll.world.session.shutdown();
}

#[test]
fn a_thumb_placement_commits_only_the_pose_geometry_displays() {
    use super::super::super::scroll_chrome_interaction::UiScrollChromeInteractionDenial;

    let mut scroll = ScrollWorld::launch_published();
    let displayed_before = scroll.mounted_offset();
    let (owner, incarnation, target) = (scroll.owner, scroll.incarnation, scroll.target());

    let pending = scroll.hold_presentation_open(2);
    assert_eq!(
        scroll.world.session.place_scroll_chrome_offset(
            owner,
            incarnation,
            target,
            0,
            block(10),
            UiScrollDeltaCause::ChromeThumbDrag,
        ),
        Err(UiScrollChromeInteractionDenial::Geometry(
            UiMountedOccurrenceGeometryDenial::PresentationInFlight
        ))
    );
    assert_eq!(scroll.accepted_offset(), block(0));
    assert_eq!(scroll.mounted_offset(), displayed_before);

    scroll.complete(pending, 3);
    let placed = scroll
        .world
        .session
        .place_scroll_chrome_offset(
            owner,
            incarnation,
            target,
            0,
            block(10),
            UiScrollDeltaCause::ChromeThumbDrag,
        )
        .expect("a placement with no attempt in flight applies");
    assert_eq!(placed.transitions()[0].current(), block(10));
    assert_eq!(scroll.accepted_offset(), block(0));
    scroll.publish_direct(4);
    assert_eq!(scroll.accepted_offset(), block(10));
    assert_eq!(scroll.mounted_offset(), Some(block(10)));
    let _ = scroll.world.session.shutdown();
}

#[test]
fn an_accepted_sample_settles_to_its_displacement_from_a_nonzero_rest() {
    use super::super::super::UiAcceptedScrollSettlementDenial;

    let scroll = ScrollWorld::launch_published();
    let (target, surface) = (scroll.target(), scroll.surface());
    let (owner_instance, content, _) = scroll
        .world
        .session
        .mounted
        .scroll_region_geometry(target, 0)
        .expect("the scrollable region has geometry");
    assert_eq!(owner_instance, target);
    assert!(
        content.x() > 0.0 && content.y() > 0.0,
        "the content box rests off the origin: {content:?}"
    );
    let key =
        super::super::super::scroll_transition_preparation::scroll_motion_owner_key(scroll.owner);
    let motion_target = UiMotionTargetIdentity::from_scroll_region_owner(surface, target, key);
    let owner = scroll
        .world
        .session
        .scroll_settlement_owner(motion_target, surface)
        .expect("this surface's Scroll content resolves")
        .expect("this surface's Scroll content is not foreign");
    assert_eq!(owner.owner_instance, target);
    assert_eq!(owner.region_instance, target);
    assert_eq!(owner.slot, 0);
    assert_eq!(
        owner.entry,
        UiScrollChainEntry::new(scroll.owner, scroll.incarnation)
    );
    let settle = |dy: f32| {
        let [x, y, width, height] = [content.x(), content.y(), content.width(), content.height()];
        owner.settle(displayed_rect_for_test(
            [x, y + dy, width, height],
            content.coordinate_space(),
        ))
    };

    // Content displayed twelve points above rest is content scrolled by twelve
    // points, whatever absolute position rest happens to be.
    let scrolled = settle(-12.0).expect("a sample above rest settles");
    assert_eq!(scrolled.offset.settled(), block(12));
    assert_eq!(scrolled.owner.owner_instance, target);
    let at_rest = settle(0.0).expect("a sample at rest settles");
    assert_eq!(at_rest.offset.settled(), block(0));
    assert!(
        matches!(
            settle(12.0),
            Err(UiAcceptedScrollSettlementDenial::SampleBeforeRest)
        ),
        "content displayed below rest names no offset"
    );
    assert!(
        matches!(
            scroll.world.session.scroll_settlement_owner(
                UiMotionTargetIdentity::from_mounted_owner(surface, target, key),
                surface,
            ),
            Ok(None)
        ),
        "an ordinary Motion target is not Scroll content"
    );
    let _ = scroll.world.session.shutdown();
}

#[test]
fn a_displayed_offset_settles_only_the_surface_that_displayed_it() {
    let mut scroll = ScrollWorld::launch_published();
    let before = scroll.mounted_offset();
    // A witness for another binding displayed this offset.
    let foreign = crate::mounting::presentation::displayed_scroll_offset_for_test(block(12));
    let (surface, target) = (scroll.surface(), scroll.target());
    assert!(matches!(
        scroll
            .world
            .session
            .mounted
            .apply_presented_scroll_geometries(&[(surface, target, foreign)]),
        Err(crate::mounting::UiMountedOccurrenceGeometryDenial::ForeignSurface)
    ));
    assert_eq!(scroll.mounted_offset(), before);
    let _ = scroll.world.session.shutdown();
}
