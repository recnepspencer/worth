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
use crate::mounting::{UiMountedFrameOutcome, UiMountedPresentationInFlight};
use crate::runtime::scroll::{UiHostScrollObservationOutcome, UiScrollDeltaCause};
use worth_ui_host_contract::UiPresentationDeadline;

/// Full block travel of the scrollable region, and the travel left once its
/// content is resized shorter.
const TRAVELS: [f32; 2] = [30.0, 5.0];

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
    pub(super) const CHOICES: [Self; 18] = [
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
        Self::Resize,
        Self::Publish,
        Self::RejectPublication,
        Self::BeginInFlight,
        Self::CompleteInFlight,
        Self::Rebind,
    ];

    pub(super) const EVERY: [Self; 13] = [
        Self::Frame,
        Self::Prepare,
        Self::Commit,
        Self::RejectTick,
        Self::Notch,
        Self::Retarget,
        Self::OwnerEdit,
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
        }
    }

    /// Run one operation; `Some` names what took effect.
    pub(super) fn apply(&mut self, step: Step, roll: u64) -> Option<Step> {
        self.tick += 1;
        let effect = match step {
            Step::Frame => self.frame(),
            Step::Prepare => self.prepare(),
            Step::Commit => self.present_held(false),
            Step::RejectTick => self.present_held(true),
            Step::Notch | Step::Retarget => self.notch(roll),
            Step::OwnerEdit => self.owner_edit(roll),
            Step::Resize => self.resize(),
            Step::Publish => self.publish(),
            Step::RejectPublication => self.reject_publication(),
            Step::BeginInFlight => self.begin_in_flight(),
            Step::CompleteInFlight => self.complete_in_flight(),
            Step::Rebind => self.rebind(),
        };
        self.resolve_owner();
        effect
    }

    pub(super) fn finish(mut self) {
        if let Some(pending) = self.in_flight.take() {
            self.tick += 1;
            self.scroll.complete(pending, self.tick);
        }
        let _ = self.present_held(false);
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

    /// Whether the host is owed this tick: it moves something, on the basis
    /// the host still displays. A publication the host holds open does not
    /// stop a sample; the sample presents beside it.
    fn host_takes(
        &self,
        prepared: &UiPreparedMotionSampling,
        displayed: UiDisplayedSurfaceBasis,
    ) -> bool {
        !prepared.receipt().samples().is_empty() && self.displayed() == displayed
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
            Ok(prepared) => {
                if self.host_takes(&prepared, displayed) {
                    self.scroll.world.host.push_native_display_as_issued();
                }
                self.scroll
                    .world
                    .session
                    .present_prepared_motion_tick(prepared, displayed);
            }
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
        match (takes, reject) {
            (true, true) => self.scroll.world.host.push_rejected(),
            (true, false) => self.scroll.world.host.push_native_display_as_issued(),
            (false, _) => {}
        }
        self.scroll
            .world
            .session
            .present_prepared_motion_tick(prepared, displayed);
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

    /// A track page the owner places directly, published at once.
    fn owner_edit(&mut self, roll: u64) -> Option<Step> {
        if self.in_flight.is_some() {
            return None;
        }
        let points = (roll % (TRAVELS[self.travel] as u64 + 1)) as i64;
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
        self.publish_as_issued();
        Some(Step::OwnerEdit)
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
            TRAVELS[self.travel],
        );
        self.staged = true;
        Some(Step::Resize)
    }

    fn publish(&mut self) -> Option<Step> {
        if self.in_flight.is_some() {
            return None;
        }
        self.publish_as_issued();
        Some(Step::Publish)
    }

    /// What the shell does once presentation is pending: a settle that
    /// changed what the host must hold, or a staged layout, wakes it.
    fn publish_owed(&mut self) {
        if self.in_flight.is_none()
            && self
                .scroll
                .world
                .session
                .mounted
                .projection_changes_pending()
        {
            self.publish_as_issued();
        }
    }

    /// Publish the surface; the host paints exactly what the frame carries.
    fn publish_as_issued(&mut self) {
        let outcome = self.present_surface(ScriptedPresentationHost::push_native_display_as_issued);
        assert!(
            matches!(outcome, UiMountedFrameOutcome::Published(_)),
            "the host takes the publication as issued"
        );
        self.staged = false;
    }

    /// Prepare the surface's frame and present it, the host answering each
    /// surface it carries with `answer`.
    fn present_surface(&mut self, answer: fn(&ScriptedPresentationHost)) -> UiMountedFrameOutcome {
        let surface = self.scroll.surface();
        let world = &mut self.scroll.world;
        let frame = world.prepare_surface(surface);
        for _ in frame.surfaces() {
            answer(&world.host);
        }
        world.session.present_prepared_mounted_frame_internal(
            frame,
            UiPresentationDeadline::at_tick(u64::MAX),
            self.tick,
        )
    }

    fn reject_publication(&mut self) -> Option<Step> {
        if self.in_flight.is_some() {
            return None;
        }
        let outcome = self.present_surface(ScriptedPresentationHost::push_rejected);
        assert!(
            matches!(outcome, UiMountedFrameOutcome::RejectedBeforeEffects(_)),
            "a host refusal before effects is reported as one"
        );
        Some(Step::RejectPublication)
    }

    fn begin_in_flight(&mut self) -> Option<Step> {
        if self.in_flight.is_some() {
            return None;
        }
        self.in_flight = Some(self.scroll.hold_presentation_open(self.tick));
        Some(Step::BeginInFlight)
    }

    fn complete_in_flight(&mut self) -> Option<Step> {
        let pending = self.in_flight.take()?;
        self.scroll.complete(pending, self.tick);
        self.staged = false;
        Some(Step::CompleteInFlight)
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
