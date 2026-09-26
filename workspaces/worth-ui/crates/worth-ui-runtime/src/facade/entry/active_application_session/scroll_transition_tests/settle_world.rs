//! The chain one wheel notch travels, assembled from the production types that
//! own each link and nothing else.
//!
//! A notch enters the Scroll runtime state, becomes a semantic target with a
//! settle horizon, lowers into a Motion transition request, is committed by the
//! Motion runtime state, is installed into the presentation sampler, and comes
//! back as an accepted translation that settles into the owner's offset. Every
//! one of those is the real type; the world supplies only the mounted evidence
//! a live session would have resolved -- a surface, a region occurrence, a
//! content box and a presentation basis.
//!
//! The content box sits at a nonzero origin on purpose. Every sampled position
//! the world reports is a displacement from that rest position, so a sign or
//! origin error in the offset arithmetic shows up as a failed assertion here
//! instead of hiding behind a content box at zero.
//!
//! What the world does not supply is authority. The service-proposal lane that
//! publishes a settle in production needs a frozen application's declared
//! service support, which no in-crate fixture has; this world therefore commits
//! through Motion's own declared-transition test entry, which runs the same
//! `stage` -> `derive` -> `commit_published` sequence the lane runs.

use crate::mounting::presentation::{
    presented_surface_witness_for_certification, UiDisplayedRect, UiDisplayedScrollOffset,
    UiPublishedRect,
};
use crate::runtime::scroll::transition::{
    scroll_settle_motion_request, UiScrollMotionBinding, UiScrollTransitionTarget,
    UiScrollWheelInput, UiScrollWheelLineDelta,
};
use crate::runtime::scroll::{
    UiScrollBounds, UiScrollChainEntry, UiScrollOffset, UiScrollOwnerIdentity,
    UiScrollOwnerIncarnation, UiScrollOwnerRegistration, UiScrollRouteDenial, UiScrollRouteReceipt,
    UiScrollRuntimeState,
};

/// The declared coarse-wheel arithmetic this milestone names: three lines to a
/// notch, twenty points to a line, sixty points to a notch.
pub(super) const LINES_PER_NOTCH: u16 = 3;
pub(super) const LINE_EXTENT_POINTS: u16 = 20;
pub(super) const ONE_NOTCH_POINTS: f64 = LINES_PER_NOTCH as f64 * LINE_EXTENT_POINTS as f64;
/// The declared settle horizon under test, in accepted-sample ticks.
pub(super) const SETTLE_TICKS: u32 = 120;
/// The plan region index of the scrolling occurrence, which is also the Motion
/// owner key its Scroll-content target is built from.
const PLAN_REGION_INDEX: u32 = 3;
/// Where the scrolled content box rests, in viewport points. Neither is zero,
/// so a displacement and an absolute position can never be confused.
pub(super) const CONTENT_REST_X: f32 = 290.0;
pub(super) const CONTENT_REST_Y: f32 = 693.0;
const SUBPIXELS_PER_POINT: f64 =
    worth_ui_host_contract::UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT as f64;

pub(super) struct UiScrollSettleWorld {
    owner: UiScrollOwnerIdentity,
    incarnation: UiScrollOwnerIncarnation,
    mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    content: worth_ui_host_contract::UiMountedCanonicalBox,
    bounds: UiScrollBounds,
    scroll: UiScrollRuntimeState,
    motion: crate::runtime::motion::UiMotionRuntimeState,
    sampler: crate::mounting::presentation::motion_sampling::UiMountedMotionSampler,
    revision: u64,
}

impl UiScrollSettleWorld {
    /// A world whose region owner scrolls the block axis between zero and
    /// `block_bound_points`, starting at rest at offset zero.
    pub(super) fn new(block_bound_points: f64) -> Self {
        let surface = worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound()
            .expect("a surface identity");
        let owner = UiScrollOwnerIdentity::declared_region(
            surface,
            crate::graph::UiGraphNodeIdentity::new(316_101),
            1,
            PLAN_REGION_INDEX,
        );
        let incarnation = UiScrollOwnerIncarnation::new(1).expect("a nonzero incarnation");
        let bounds = bounds(block_bound_points);
        let mut scroll =
            UiScrollRuntimeState::new_session_restore_candidate_with_policy(smooth_wheel_policy());
        scroll
            .register(UiScrollOwnerRegistration::new(
                owner,
                incarnation,
                crate::runtime::scroll::UiScrollAxes::Block,
                bounds,
                offset(0.0),
            ))
            .expect("an owner registered at rest inside its bounds");
        Self {
            owner,
            incarnation,
            mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity::mint_unbound()
                .expect("a mounted instance identity"),
            presentation: presentation(),
            content: content_box(),
            bounds,
            scroll,
            motion: crate::runtime::motion::UiMotionRuntimeState::new(
                crate::runtime::UiServiceStatePersistencePosture::Ephemeral,
            ),
            sampler: Default::default(),
            revision: 1,
        }
    }

    pub(super) const fn entry(&self) -> UiScrollChainEntry {
        UiScrollChainEntry::new(self.owner, self.incarnation)
    }

    pub(super) const fn scroll(&self) -> &UiScrollRuntimeState {
        &self.scroll
    }

    pub(super) const fn scroll_mut(&mut self) -> &mut UiScrollRuntimeState {
        &mut self.scroll
    }

    pub(super) const fn bounds(&self) -> UiScrollBounds {
        self.bounds
    }

    pub(super) fn motion_target(&self) -> crate::runtime::motion::UiMotionTargetIdentity {
        crate::runtime::motion::UiMotionTargetIdentity::from_scroll_region_owner(
            self.owner.semantic_surface(),
            self.mounted_instance,
            u64::from(PLAN_REGION_INDEX),
        )
    }

    /// The owner's current pending settle target, if it has one.
    pub(super) fn pending_target(&self) -> Option<UiScrollTransitionTarget> {
        self.scroll.transition_target(self.owner, self.incarnation)
    }

    /// The owner's accepted semantic offset along the block axis, in points.
    pub(super) fn accepted_offset_points(&self) -> f64 {
        points(
            self.scroll
                .offset(self.owner, self.incarnation)
                .expect("a registered owner reports its offset")
                .block_subpixels(),
        )
    }

    /// Stage one wheel notch at `tick` and publish it into Motion and the
    /// sampler, exactly as the settle lane does with a live publication.
    pub(super) fn notch(&mut self, notches: i64, tick: u64) -> UiScrollNotch {
        let target = self
            .scroll
            .stage_wheel_transition(
                self.entry(),
                UiScrollWheelInput::admit(
                    UiScrollWheelLineDelta::from_notches(0, notches, LINES_PER_NOTCH),
                    worth_ui_host_contract::UiHostScrollDeltaPhase::Updated,
                    tick,
                    LINE_EXTENT_POINTS,
                    SETTLE_TICKS,
                )
                .expect("a usable line extent and settle horizon"),
            )
            .expect("a registered owner stages its notch");
        let accepted = self
            .scroll
            .offset(self.owner, self.incarnation)
            .expect("a registered owner reports its offset");
        let request = scroll_settle_motion_request(
            target,
            accepted,
            tick,
            UiScrollMotionBinding::new(
                self.mounted_instance,
                u64::from(PLAN_REGION_INDEX),
                self.revision,
                self.revision + 1,
                self.presentation,
                crate::mounting::UiLaidOut::from_layout(self.content),
            ),
        )
        .expect("an unexhausted horizon lowers into a Motion request");
        self.revision += 1;
        let receipt = self.motion.commit_declared_transition_for_test(
            self.revision,
            request,
            self.presentation.frame(),
            self.presentation,
        );
        let fact = receipt.fact().kind();
        let retarget = receipt.track().retarget();
        let installed = self
            .sampler
            .install(receipt)
            .expect("a committed settle installs into the sampler");
        UiScrollNotch {
            target,
            fact,
            retarget,
            installed_y: self.displacement_from_rest(
                installed
                    .sample()
                    .geometry()
                    .expect("a settle samples geometry")
                    .components()[1],
            ),
        }
    }

    /// How far a sampled block position sits from the content's rest position,
    /// in points. Negative is scrolled: content moved toward the viewport
    /// origin.
    fn displacement_from_rest(&self, sampled_y: f32) -> f64 {
        f64::from(sampled_y) - f64::from(self.content.y())
    }

    /// Commit one accepted-sample tick without reading a sample from it. A tick
    /// after the settle has arrived carries no sample at all, which is a
    /// quiet frame rather than a missing one.
    pub(super) fn tick(&mut self, tick: u64) {
        let prepared = self
            .sampler
            .prepare_tick(tick, self.presentation)
            .expect("a monotonic tick prepares");
        self.commit_prepared(prepared);
    }

    /// Commit one accepted-sample tick and report how far the sampled block
    /// position of the scrolled content group sits from rest, in points.
    pub(super) fn commit(&mut self, tick: u64) -> f64 {
        let prepared = self
            .sampler
            .prepare_tick(tick, self.presentation)
            .expect("a monotonic tick prepares");
        let receipt = self.commit_prepared(prepared);
        self.displacement_from_rest(
            receipt.samples()[0]
                .geometry()
                .expect("a settling track samples geometry")
                .components()[1],
        )
    }

    /// Take direct control of the region, as a thumb grab does: the pending
    /// Scroll target, the sampler track and the Motion track all go, in that
    /// order, so nothing is left settling toward an offset the pointer now
    /// names directly. Reports which of the three were there to retire.
    pub(super) fn take_direct_control(&mut self) -> [bool; 3] {
        let target = self.motion_target();
        let transition = self.scroll.retire_transition(self.owner);
        let sample = self.sampler.retire_scroll_group_track(target);
        let track = self
            .motion
            .terminalize_target(
                target,
                crate::runtime::motion::UiMotionTerminalCause::DisplacedByDirectControl,
            )
            .is_some();
        [transition, sample, track]
    }

    /// Prepare one accepted-sample tick without committing it, so a scenario
    /// can act between the two exactly as a presentation in flight does.
    pub(super) fn prepare(
        &mut self,
        tick: u64,
    ) -> crate::mounting::presentation::motion_sampling::UiPreparedMotionSampling {
        self.sampler
            .prepare_tick(tick, self.presentation)
            .expect("a monotonic tick prepares")
    }

    /// Commit a prepared tick and settle the terminal requests it produced into
    /// Motion, as the facade does after every committed sampling: a track whose
    /// settle has arrived stops being an incumbent, so the next notch against
    /// the same owner starts a fresh track instead of retargeting a finished
    /// one. The sampler keeps the arrived track's accepted sample until the
    /// write-back has read it.
    pub(super) fn commit_prepared(
        &mut self,
        prepared: crate::mounting::presentation::motion_sampling::UiPreparedMotionSampling,
    ) -> crate::mounting::presentation::motion_sampling::UiPresentationMotionSamplingReceipt {
        let receipt = self
            .sampler
            .commit_prepared(prepared.presented_for_certification());
        for terminal in receipt.terminals().iter().copied() {
            self.motion
                .terminalize(terminal.track(), terminal.cause())
                .expect("a terminal request names the committed Motion track");
        }
        receipt
    }

    /// Whether the sampler still owes this settle further ticks.
    pub(super) fn sampling_active(&self) -> bool {
        self.sampler.has_active_tracks()
    }

    /// The accepted position the sampler reports for the scrolled group: the
    /// absolute sampled origin of the content box, in points.
    pub(super) fn accepted_translation(&self) -> Option<[f32; 2]> {
        self.sampler
            .accepted_scroll_group_sample(self.motion_target())
            .map(|sample| [sample.components()[0], sample.components()[1]])
    }

    /// The accepted block displacement from rest, in points, when the sampler
    /// holds an accepted position for the scrolled group.
    pub(super) fn accepted_displacement_points(&self) -> Option<f64> {
        self.accepted_translation()
            .map(|position| self.displacement_from_rest(position[1]))
    }

    /// Settle the accepted sample into the owner's offset under `bounds`,
    /// exactly as the facade write-back does after a frame applies the pose.
    pub(super) fn settle_accepted(
        &mut self,
        bounds: UiScrollBounds,
    ) -> Result<UiScrollRouteReceipt, UiScrollRouteDenial> {
        let sample = self
            .sampler
            .accepted_scroll_group_sample(self.motion_target())
            .expect("a presented settle reports an accepted sample");
        let witness = presented_surface_witness_for_certification(sample.presentation_basis());
        let displayed = UiDisplayedRect::displayed(sample, witness.displayed_basis())
            .expect("the witness displays the binding the sample was accepted on");
        // The owner box is the content box at rest: an applied pose moves the
        // owner's descendants and never the owner.
        let rest = UiPublishedRect::from_committed_box(self.content);
        let offset = UiDisplayedScrollOffset::from_rest(rest, displayed)
            .expect("a settle sample stands at or past rest");
        self.scroll
            .settle_accepted_sample(self.entry(), offset, bounds)
    }
}

/// What one published notch produced: the semantic target it staged, the Motion
/// fact its commit published, and where the installed track opened.
#[derive(Clone, Copy, Debug)]
pub(super) struct UiScrollNotch {
    pub(super) target: UiScrollTransitionTarget,
    pub(super) fact: crate::runtime::motion::UiMotionProducedFactKind,
    pub(super) retarget: Option<crate::runtime::motion::UiMotionRetargetDisposition>,
    pub(super) installed_y: f64,
}

impl UiScrollNotch {
    /// The block offset this notch is settling toward, in points.
    pub(super) fn target_points(self) -> f64 {
        points(self.target.target_offset().block_subpixels())
    }
}

/// The declared policy under test: a smooth wheel with a 120-tick horizon.
pub(super) fn smooth_wheel_policy() -> crate::declaration::UiScrollPolicy {
    crate::declaration::UiScrollPolicy::nested_region().with_wheel_behavior(
        crate::declaration::UiScrollWheelBehavior::smooth(SETTLE_TICKS)
            .expect("120 ticks is an admitted horizon"),
    )
}

pub(super) fn bounds(block_points: f64) -> UiScrollBounds {
    UiScrollBounds::new(0, subpixels(block_points)).expect("non-negative bounds")
}

pub(super) fn offset(block_points: f64) -> UiScrollOffset {
    UiScrollOffset::new(0, subpixels(block_points)).expect("a non-negative offset")
}

pub(super) fn points(subpixels: i64) -> f64 {
    subpixels as f64 / SUBPIXELS_PER_POINT
}

pub(super) fn subpixels(points: f64) -> i64 {
    (points * SUBPIXELS_PER_POINT).round() as i64
}

fn content_box() -> worth_ui_host_contract::UiMountedCanonicalBox {
    worth_ui_host_contract::UiMountedCanonicalBox::canonicalize(
        worth_ui_host_contract::UiMountedCanonicalBoxInput {
            x: CONTENT_REST_X,
            y: CONTENT_REST_Y,
            width: 240.0,
            height: 269.0,
            coordinate_space: worth_ui_host_contract::UiMountedCoordinateSpace::Viewport,
        },
    )
    .expect("a finite non-negative content box")
}

fn presentation() -> worth_ui_host_contract::UiHostObservationPresentationBasis {
    worth_ui_host_contract::UiHostObservationPresentationBasis::new(
        worth_ui_host_contract::UiHostSurfaceIdentity::mint_unbound().expect("a host surface"),
        worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().expect("a mounted frame"),
        worth_ui_host_contract::UiSurfaceBindingGeneration::mint_unbound()
            .expect("a binding generation"),
        worth_ui_host_contract::UiHostPresentationEpoch::issued_by_host(1),
    )
}
