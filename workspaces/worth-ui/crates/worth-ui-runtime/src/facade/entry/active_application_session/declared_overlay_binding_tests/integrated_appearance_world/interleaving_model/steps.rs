//! The operations the interleaving model mixes, each driven through the entry
//! the native shell runs, and the scrollable World they act on.
//!
//! An operation that cannot run in the state it meets does nothing and says
//! so, the way a shell never offers it. The host is scripted only for a call
//! the runtime is owed to make: a Motion tick that moves something on the
//! basis still displayed.

use super::super::geometry::scrollable::install_scrollable_primary_with_travel;
use super::super::reconstruction::reconstruct_surface_at;
use super::super::scroll_hover_reresolution::rest_pointer_on_the_component;
use super::super::scroll_pose_authority::{block, ScrollWorld};
use super::super::scroll_settle_commit::{
    one_notch_up, pending_transitions, smooth_world, ONE_NOTCH,
};
use crate::certification_support::ScriptedPresentationHost;
use crate::mounting::presentation::motion_sampling::UiPreparedMotionSampling;
use crate::mounting::presentation::UiDisplayedSurfaceBasis;
use crate::mounting::UiMountedPresentationInFlight;
use crate::runtime::scroll::{UiHostScrollObservationOutcome, UiScrollDeltaCause, UiScrollOffset};

#[path = "intent.rs"]
mod intent;
#[path = "publication.rs"]
mod publication;
#[path = "tick.rs"]
mod tick;

/// Full block travel of the scrollable region, and the travel left once its
/// content is resized shorter.
const TRAVELS: [u16; 2] = [30, 5];

/// Frames a run gets to come to rest in: far more than any settle lasts.
const REST_FRAMES: usize = 64;

/// One kind of operation. `Retarget` is never chosen; it is what a notch is
/// when it lands while a settle is still in flight.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum Step {
    Frame,
    Prepare,
    Commit,
    RejectTick,
    Notch,
    Retarget,
    OwnerEdit,
    StageEdit,
    Resize,
    Publish,
    RejectPublication,
    BeginInFlight,
    CompleteInFlight,
    Rebind,
}

impl Step {
    /// What a run chooses from. Frames and notches are listed more than once,
    /// so settles run long enough to be caught mid-flight by everything else.
    pub(super) const CHOICES: [Self; 19] = [
        Self::Frame,
        Self::Frame,
        Self::Frame,
        Self::Frame,
        Self::Frame,
        Self::Prepare,
        Self::Prepare,
        Self::Commit,
        Self::RejectTick,
        Self::Notch,
        Self::Notch,
        Self::OwnerEdit,
        Self::StageEdit,
        Self::Resize,
        Self::Publish,
        Self::RejectPublication,
        Self::BeginInFlight,
        Self::CompleteInFlight,
        Self::Rebind,
    ];

    pub(super) const EVERY: [Self; 14] = [
        Self::Frame,
        Self::Prepare,
        Self::Commit,
        Self::RejectTick,
        Self::Notch,
        Self::Retarget,
        Self::OwnerEdit,
        Self::StageEdit,
        Self::Resize,
        Self::Publish,
        Self::RejectPublication,
        Self::BeginInFlight,
        Self::CompleteInFlight,
        Self::Rebind,
    ];
}

pub(super) struct Model {
    pub(super) scroll: ScrollWorld,
    tick: u64,
    held: Option<(UiPreparedMotionSampling, UiDisplayedSurfaceBasis)>,
    in_flight: Option<UiMountedPresentationInFlight>,
    revision: u64,
    travel: usize,
    pointer: u64,
    /// A resize installed but not yet published: the layout the next frame
    /// prepares, which no witness has shown.
    pub(super) staged: bool,
    /// A page placed but not yet published: the offset the next frame
    /// prepares, which no witness has shown.
    pub(super) direct: Option<UiScrollOffset>,
    /// Where the last intent put the content: where it must come to rest.
    rests_at: UiScrollOffset,
}

impl Model {
    pub(super) fn launch() -> Self {
        let mut scroll = smooth_world(true);
        rest_pointer_on_the_component(&mut scroll, 1);
        Self {
            scroll,
            tick: 1,
            held: None,
            in_flight: None,
            revision: 100,
            travel: 0,
            pointer: 1,
            staged: false,
            direct: None,
            rests_at: block(0),
        }
    }

    /// Run one operation; `Some` names what took effect.
    pub(super) fn apply(&mut self, step: Step, roll: u64) -> Option<Step> {
        self.tick += 1;
        let staged = self.staged;
        let effect = match step {
            Step::Frame => self.frame(),
            Step::Prepare => self.prepare(),
            Step::Commit => self.present_held(false),
            Step::RejectTick => self.present_held(true),
            Step::Notch | Step::Retarget => self.notch(roll),
            Step::OwnerEdit => self.owner_edit(roll, true),
            Step::StageEdit => self.owner_edit(roll, false),
            Step::Resize => self.resize(),
            Step::Publish => self.publish(),
            Step::RejectPublication => self.reject_publication(),
            Step::BeginInFlight => self.begin_in_flight(),
            Step::CompleteInFlight => self.complete_in_flight(),
            Step::Rebind => self.rebind(),
        };
        self.resolve_owner();
        let surface = self.scroll.surface();
        let session = &self.scroll.world.session;
        if !session
            .scroll
            .as_ref()
            .is_some_and(|scroll| scroll.has_pending_direct(surface))
        {
            self.direct = None;
        }
        self.follow_intent(effect, staged && !self.staged);
        effect
    }

    /// Bring the World to rest: the attempt the host holds open completes,
    /// the held tick presents, and frames run until nothing is left to
    /// settle or publish. Every settle Scroll holds must land: a transition nothing
    /// drives any more, or a settle owed with nothing to pay it, never
    /// comes to rest, and the pose rests where Scroll holds it, which is
    /// where the last intent put the content.
    pub(super) fn rest(&mut self, label: &str) {
        let _ = self.apply(Step::CompleteInFlight, 0);
        let _ = self.apply(Step::Commit, 0);
        for _ in 0..REST_FRAMES {
            if self.at_rest() {
                break;
            }
            let _ = self.apply(Step::Frame, 0);
        }
        assert!(
            self.at_rest(),
            "{label}: {} Scroll settles never land, and a settle is owed: {}",
            pending_transitions(&self.scroll),
            self.scroll.world.session.awaits_scroll_settle_retry()
        );
        assert_eq!(
            self.scroll.mounted_offset(),
            Some(self.scroll.accepted_offset()),
            "{label}: the pose rests where Scroll holds it"
        );
        assert_eq!(
            self.scroll.accepted_offset(),
            self.rests_at,
            "{label}: the content rests where the last intent put it"
        );
    }

    fn at_rest(&self) -> bool {
        let session = &self.scroll.world.session;
        pending_transitions(&self.scroll) == 0
            && !session.awaits_scroll_settle_retry()
            && !session.mounted.projection_changes_pending()
    }

    pub(super) fn finish(self) {
        let _ = self.scroll.world.session.shutdown();
    }

    fn displayed(&self) -> UiDisplayedSurfaceBasis {
        self.scroll
            .world
            .session
            .mounted
            .current_displayed_presentation(self.scroll.presentation())
            .expect("the retained record displays the current basis")
    }

    fn frame(&mut self) -> Option<Step> {
        if self.held.is_some() {
            return None;
        }
        let displayed = self.displayed();
        match self
            .scroll
            .world
            .session
            .prepare_motion_tick(self.tick, displayed)
        {
            Ok(prepared) => self.present_tick(
                prepared,
                displayed,
                ScriptedPresentationHost::push_native_display_as_issued,
            ),
            Err(_) => {
                self.scroll.world.session.settle_owed_scroll_samples();
            }
        }
        self.publish_owed();
        Some(Step::Frame)
    }

    fn prepare(&mut self) -> Option<Step> {
        if self.held.is_some() {
            return None;
        }
        let displayed = self.displayed();
        let prepared = self
            .scroll
            .world
            .session
            .prepare_motion_tick(self.tick, displayed)
            .ok()?;
        self.held = Some((prepared, displayed));
        Some(Step::Prepare)
    }

    /// Present the held tick; a rejected one is refused by the host before
    /// any effect, so only a tick the host is owed can be rejected.
    fn present_held(&mut self, reject: bool) -> Option<Step> {
        let takes = {
            let (prepared, displayed) = self.held.as_ref()?;
            self.host_takes(prepared, *displayed)
        };
        if reject && !takes {
            return None;
        }
        let (prepared, displayed) = self.held.take()?;
        let outcome = if reject {
            ScriptedPresentationHost::push_rejected
        } else {
            ScriptedPresentationHost::push_native_display_as_issued
        };
        self.present_tick(prepared, displayed, outcome);
        self.publish_owed();
        Some(if reject {
            Step::RejectTick
        } else {
            Step::Commit
        })
    }

    fn notch(&mut self, roll: u64) -> Option<Step> {
        let settling = pending_transitions(&self.scroll) > 0;
        let direction = if roll.is_multiple_of(3) {
            -one_notch_up()
        } else {
            one_notch_up()
        };
        let outcome = self.scroll.wheel(ONE_NOTCH, direction, self.tick);
        matches!(outcome, UiHostScrollObservationOutcome::Applied(_)).then_some(if settling {
            Step::Retarget
        } else {
            Step::Notch
        })
    }

    /// A track page the owner places directly, published at once or left
    /// staged for a later publication to land.
    fn owner_edit(&mut self, roll: u64, publish: bool) -> Option<Step> {
        if self.in_flight.is_some() {
            return None;
        }
        let points = i64::try_from(roll % (u64::from(TRAVELS[self.travel]) + 1)).ok()?;
        let scroll = &mut self.scroll;
        scroll
            .world
            .session
            .place_scroll_chrome_offset(
                scroll.owner,
                scroll.incarnation,
                scroll.target(),
                0,
                block(points),
                UiScrollDeltaCause::ChromeTrackPage,
            )
            .ok()?;
        if publish {
            self.publish_as_issued();
            return Some(Step::OwnerEdit);
        }
        self.direct = Some(block(points));
        Some(Step::StageEdit)
    }

    /// Stage the region's content at the other length; a later publication
    /// lands it.
    fn resize(&mut self) -> Option<Step> {
        if self.in_flight.is_some() {
            return None;
        }
        self.travel = 1 - self.travel;
        self.revision += 1;
        let world = &mut self.scroll.world;
        install_scrollable_primary_with_travel(
            &mut world.session,
            world.surfaces,
            world.instances,
            self.revision,
            f32::from(TRAVELS[self.travel]),
        );
        self.staged = true;
        Some(Step::Resize)
    }

    /// Rebind the surface to a new host generation. The host forgets the
    /// pointer with its old generation, so the pointer is reported again.
    fn rebind(&mut self) -> Option<Step> {
        if self.in_flight.is_some() {
            return None;
        }
        reconstruct_surface_at(&mut self.scroll.world, self.tick);
        self.staged = false;
        self.pointer += 1;
        rest_pointer_on_the_component(&mut self.scroll, self.pointer);
        Some(Step::Rebind)
    }

    /// The region's owner and incarnation as the current geometry names them.
    fn resolve_owner(&mut self) {
        let target = self.scroll.target();
        let session = &self.scroll.world.session;
        self.scroll.owner = session
            .scroll
            .as_ref()
            .expect("Scroll stays installed")
            .ownership_chain(target)
            .expect("the region keeps its Scroll chain")
            .owners()[0];
        self.scroll.incarnation = session
            .mounted
            .scroll_region_incarnation(target, 0)
            .expect("the region keeps a current allocation");
    }
}
