//! One host scroll delta, from the payload the host reported to the offset the
//! reader sees move.
//!
//! The order here is the whole point. The chain is resolved, the delta is
//! turned into offset direction, bounds are reconciled and the route is run
//! against a chain-local successor. Direct intent stages candidate poses and carries
//! its exact result through ordinary frame acceptance; smooth intent publishes
//! a settle whose accepted samples alone move the offset. Input admission is
//! not presentation, and refusal leaves the accepted predecessor intact.

use super::super::WorthUiActiveApplicationSession;

use super::scroll_chrome_ingress::suppress_captured_axes;
use super::scroll_gesture_latching::UiScrollRoutedChain;
use super::scroll_gesture_latching::UiScrollRoutedRegion;
use crate::runtime::scroll::transition::line_travel;
use crate::runtime::scroll::{
    UiHostScrollObservationDenial, UiHostScrollObservationOutcome, UiScrollDelta,
    UiScrollHostTravel,
};

impl WorthUiActiveApplicationSession {
    pub(in crate::facade::entry) fn observe_scroll_payload(
        &mut self,
        payload: &worth_ui_host_contract::UiHostObservationPayload,
        work: &mut crate::mounting::UiHitTestSpatialWork,
        observation_tick: Option<u64>,
    ) -> Option<UiHostScrollObservationOutcome> {
        let worth_ui_host_contract::UiHostObservationPayload::ScrollDelta {
            source,
            phase,
            precision,
            target,
            x_subpixels,
            y_subpixels,
        } = payload
        else {
            return None;
        };
        let outcome = match self.apply_host_scroll_delta(
            *source,
            *phase,
            *precision,
            *target,
            [*x_subpixels, *y_subpixels],
            work,
            observation_tick,
        ) {
            Ok(receipt) => UiHostScrollObservationOutcome::Applied(receipt),
            Err(denial) => UiHostScrollObservationOutcome::Denied(denial),
        };
        // A gesture that has stated its end is over on the host's say-so, not
        // on ours. Whatever the delta it carried was allowed to do, the latch
        // it was holding ends here, once, after the report has been made.
        self.end_scroll_gesture_latch_on_phase(*phase);
        Some(outcome)
    }

    fn apply_host_scroll_delta(
        &mut self,
        source: worth_ui_host_contract::UiHostScrollDeltaSource,
        phase: worth_ui_host_contract::UiHostScrollDeltaPhase,
        precision: worth_ui_host_contract::UiHostScrollDeltaPrecision,
        target: worth_ui_host_contract::UiHostScrollDeltaTargetAffinity,
        host_delta: [i64; 2],
        work: &mut crate::mounting::UiHitTestSpatialWork,
        observation_tick: Option<u64>,
    ) -> Result<crate::runtime::scroll::UiScrollRouteReceipt, UiHostScrollObservationDenial> {
        let routed = self.resolve_scroll_routing(target, work, observation_tick)?;
        let [x, y] = host_delta;
        if self
            .interaction
            .scroll_chrome_pending_capture()
            .is_some_and(|pending| {
                pending.released()
                    && routed
                        .entries()
                        .iter()
                        .any(|entry| entry.owner() == pending.owner())
                    && match pending.axis() {
                        crate::runtime::scroll::chrome::UiScrollChromeAxis::Inline => x != 0,
                        crate::runtime::scroll::chrome::UiScrollChromeAxis::Block => y != 0,
                    }
            })
        {
            return Err(UiHostScrollObservationDenial::PendingChromeRelease);
        }
        // A thumb drag owns the axis it grabbed for the length of its capture.
        // The drag places that offset directly, so a wheel moving the same
        // offset underneath it would fight the pointer; the other axis of the
        // same region, and every other region, keep scrolling.
        let captured = self.axes_held_by_scroll_chrome(routed.entries());
        let host_delta = suppress_captured_axes(host_delta, captured);
        if captured != [false, false] && host_delta == [0, 0] && [x, y] != [0, 0] {
            return Err(UiHostScrollObservationDenial::AxisHeldByChromeDrag);
        }
        // The delta is read once, here, in the unit its precision names and in
        // offset direction; every path below consumes that travel.
        let travel = UiScrollHostTravel::from_host(precision, host_delta)
            .ok_or(UiHostScrollObservationDenial::DeltaOutOfRange)?;
        // A declared smooth wheel answers a coarse notch with a settle
        // transition, so the observation itself moves no accepted offset: it
        // routes a zero delta to reconcile bounds and name the chain, then
        // stages the target the accepted sample travels to.
        let smooth = observation_tick.and_then(|tick| {
            self.admit_smooth_wheel_observation(
                routed.mounted_instance(),
                target.presentation(),
                phase,
                travel,
                tick,
            )
        });
        let bounds = self.reconciled_scroll_bounds(&routed)?;
        let geometry = routed
            .slots()
            .iter()
            .map(|slot| {
                self.mounted
                    .scroll_region_geometry(routed.mounted_instance(), *slot)
                    .map(|row| row.0)
            })
            .collect::<Vec<_>>();
        let installed = self
            .scroll
            .as_ref()
            .expect("Scroll installation was proven before bounds preflight");
        let surface = routed.entries()[0].owner().semantic_surface();
        if installed.has_unpresented_layout(surface)
            || (smooth.is_some() && installed.has_pending_direct(surface))
        {
            return Err(UiHostScrollObservationDenial::PendingGeometryPublication);
        }
        let mut successor = installed
            .route_candidate(routed.entries(), smooth.is_none())
            .map_err(UiHostScrollObservationDenial::Route)?;
        // Which owner this gesture belongs to, judged from the offsets the
        // chain holds now against the bounds the route is about to reconcile.
        // The route is what moves those offsets, so the question is asked
        // first; the answer is taken only once the route below commits.
        let offsets = routed
            .entries()
            .iter()
            .map(|entry| successor.state().offset(entry.owner(), entry.incarnation()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(UiHostScrollObservationDenial::Route)?;
        let region =
            crate::runtime::scroll::latching_chain_index(&offsets, &bounds, travel.heading())
                .and_then(|index| routed.region(index));
        // The latched owner is asked before the travel is measured, because it
        // is what the travel is measured against: a notch reported in lines or
        // pages is a count until some owner turns it into a distance, and the
        // owner that will move is the one whose geometry governs.
        let delta = if smooth.is_some()
            || phase == worth_ui_host_contract::UiHostScrollDeltaPhase::Cancelled
        {
            UiScrollDelta::new(0, 0)
        } else {
            self.measured_travel(&routed, region, travel)?
        };
        let request = crate::runtime::scroll::UiScrollDeltaRequest::new(
            routed.entries().to_vec(),
            delta,
            crate::runtime::scroll::UiScrollDeltaCause::Host {
                source,
                phase,
                precision,
            },
        )
        .map_err(UiHostScrollObservationDenial::Route)?;
        let receipt = successor
            .state_mut()
            .route_with_reconciled_bounds(request, &bounds)
            .map_err(UiHostScrollObservationDenial::Route)?;
        // Direct `successor` stays pending until its physical frame is accepted.
        // Smooth succession installs only target intent; its sampled displayed
        // offset has a separate host acceptance boundary.
        if let Some(observation) = smooth {
            // The staged settle lowers to a Motion transition request and is
            // published through the Scroll settle service-proposal lane. The
            // accepted offset does not move here: the track that settles it is
            // what moves it, one accepted sample at a time.
            let staged = region.or_else(|| routed.region(0));
            let settled = self
                .stage_scroll_transition(successor.state_mut(), &receipt, observation, staged)
                .map_err(
                    |denial| crate::runtime::scroll::UiScrollSettleStop::Staging {
                        detail: format!("{denial:?}").into_boxed_str(),
                    },
                )
                .and_then(|transition| self.publish_scroll_settle(&transition));
            // The observation reports only that the settle went unpublished;
            // the reason is kept where a caller can read it back. An
            // unpublished settle also commits nothing: the staged target
            // travels with the discarded successor.
            match settled {
                Ok(()) => self.last_scroll_settle_stop = None,
                Err(stop) => {
                    self.last_scroll_settle_stop = Some(stop);
                    return Err(UiHostScrollObservationDenial::SettleUnpublished);
                }
            }
            self.scroll
                .as_mut()
                .expect("installed Scroll owner")
                .commit_routed_candidate(successor);
            self.settle_routed_scroll_motion_without_a_target(&routed);
            self.latch_committed_scroll_gesture(
                &routed,
                region,
                phase,
                target.presentation(),
                observation_tick,
            );
            return Ok(receipt);
        }
        self.stage_direct_scroll_succession(
            successor.state(),
            &receipt,
            routed.mounted_instance(),
            &geometry,
        )
        .map_err(UiHostScrollObservationDenial::Geometry)?;
        self.latch_committed_scroll_gesture(
            &routed,
            region,
            phase,
            target.presentation(),
            observation_tick,
        );
        Ok(receipt)
    }

    /// The bounds each routed owner is reconciled against, in chain order.
    fn reconciled_scroll_bounds(
        &self,
        routed: &UiScrollRoutedChain,
    ) -> Result<Vec<crate::runtime::scroll::UiScrollBounds>, UiHostScrollObservationDenial> {
        routed
            .entries()
            .iter()
            .zip(routed.slots())
            .map(|(entry, slot)| {
                self.scroll_bounds_for_mounted_owner(
                    entry.owner(),
                    routed.mounted_instance(),
                    routed.graph_node(),
                    *slot,
                )
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(map_bounds_denial)
    }

    /// A gesture that reached here committed, so the latch may move. An
    /// observation the host gave no tick has no place on a timeline and
    /// therefore cannot open or age a latch.
    fn latch_committed_scroll_gesture(
        &mut self,
        routed: &UiScrollRoutedChain,
        region: Option<super::scroll_gesture_latching::UiScrollRoutedRegion>,
        phase: worth_ui_host_contract::UiHostScrollDeltaPhase,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
        observation_tick: Option<u64>,
    ) {
        if let Some(tick) = observation_tick {
            self.latch_routed_scroll_gesture(routed, region, phase, presentation, tick);
        }
    }

    /// Which axes of this chain a thumb drag currently holds.
    ///
    /// The chain is the owner the delta would move and its ancestors, so a drag
    /// on any of them silences that axis for the whole chain: handing the
    /// remainder outward while the reader drags the thumb would scroll the
    /// ancestor instead, which is not what either gesture asked for.
    fn axes_held_by_scroll_chrome(
        &self,
        entries: &[crate::runtime::scroll::UiScrollChainEntry],
    ) -> [bool; 2] {
        use crate::runtime::scroll::chrome::UiScrollChromeAxis;

        let mut held = [false, false];
        for entry in entries {
            held[0] |= self.scroll_chrome_captures_axis(entry.owner(), UiScrollChromeAxis::Inline);
            held[1] |= self.scroll_chrome_captures_axis(entry.owner(), UiScrollChromeAxis::Block);
        }
        held
    }

    /// The distance one host delta's travel moves `latched`: the owner this
    /// gesture will move.
    ///
    /// A pixel delta already is that distance. Lines and pages are counts:
    /// the host multiplied in the platform's lines per notch, but how far a
    /// line reaches is the region author's to declare, and how far a page
    /// reaches is the viewport the owner shows, less one line so the line the
    /// reader was on stays in view -- the step a track press pages by.
    ///
    /// A declared smooth wheel measures lines against the same owner when it
    /// stages the settle that travels the distance. An immediate wheel has no
    /// settle to measure them later, so they are measured here, through the
    /// conversion the settling path uses -- one notch is the same distance
    /// under either declared behavior, and an owner that will move but
    /// declares no extent is refused rather than moved by a count read as
    /// though it were a distance.
    ///
    /// `latched` is the one the caller also stages the settle against and
    /// latches the gesture to, so the three agree by construction rather than
    /// by each resolving the owner again. That matters only where a chain
    /// reaches past its innermost owner: an inner region at its edge hands the
    /// notch outward, and measuring it against the region the pointer is
    /// inside would spend an extent belonging to something that is not going
    /// to move.
    fn measured_travel(
        &self,
        routed: &UiScrollRoutedChain,
        latched: Option<UiScrollRoutedRegion>,
        travel: UiScrollHostTravel,
    ) -> Result<UiScrollDelta, UiHostScrollObservationDenial> {
        // No owner in this chain can take a count in its direction, so there
        // is no owner to measure it and nothing for the measurement to move.
        // The route still runs against no travel: it reconciles bounds and
        // names the chain it visited.
        let unmoved = Ok(UiScrollDelta::new(0, 0));
        match travel {
            UiScrollHostTravel::Distance(delta) => Ok(delta),
            UiScrollHostTravel::Lines(lines) => {
                let Some(latched) = latched else {
                    return unmoved;
                };
                let extent = self
                    .declared_scroll_line_extent_logical_points(latched.entry().owner())
                    .ok_or(UiHostScrollObservationDenial::OwnerDeclaresNoLineExtent)?;
                line_travel(lines, extent).ok_or(UiHostScrollObservationDenial::DeltaOutOfRange)
            }
            UiScrollHostTravel::Pages {
                inline_milli_pages,
                block_milli_pages,
            } => {
                let Some(latched) = latched else {
                    return unmoved;
                };
                let owner = latched.entry().owner();
                let viewport = self
                    .scroll_viewport_for_mounted_owner(
                        owner,
                        routed.mounted_instance(),
                        routed.graph_node(),
                        latched.slot(),
                    )
                    .ok_or(UiHostScrollObservationDenial::ViewportUnavailable)?;
                let line = self
                    .declared_scroll_line_extent_logical_points(owner)
                    .unwrap_or(0);
                let step = |extent: f32| {
                    crate::runtime::scroll::chrome::page_step_subpixels(extent, line)
                        .ok_or(UiHostScrollObservationDenial::ViewportUnavailable)
                };
                crate::runtime::scroll::page_travel(
                    [inline_milli_pages, block_milli_pages],
                    [step(viewport.width())?, step(viewport.height())?],
                )
                .ok_or(UiHostScrollObservationDenial::DeltaOutOfRange)
            }
        }
    }
}

fn map_bounds_denial(
    denial: crate::runtime::scroll::UiScrollBoundsResolutionDenial,
) -> UiHostScrollObservationDenial {
    match denial {
        crate::runtime::scroll::UiScrollBoundsResolutionDenial::AllocationUnavailable => {
            UiHostScrollObservationDenial::AllocationUnavailable
        }
        crate::runtime::scroll::UiScrollBoundsResolutionDenial::ViewportUnavailable => {
            UiHostScrollObservationDenial::ViewportUnavailable
        }
        crate::runtime::scroll::UiScrollBoundsResolutionDenial::OutOfRange => {
            UiHostScrollObservationDenial::BoundsOutOfRange
        }
    }
}
