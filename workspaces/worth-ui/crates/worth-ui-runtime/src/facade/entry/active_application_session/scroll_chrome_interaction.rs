//! Pressing, dragging and releasing scroll chrome.
//!
//! A thumb drag is direct: the press records how far into the thumb the pointer
//! landed, and every later move places the offset that puts that same spot back
//! under the pointer. There is no smoothing and no transition — the offset the
//! drag names is the offset that applies, which is what makes the thumb follow
//! the pointer exactly.
//!
//! Every offset this file places goes out through Scroll's own route path and
//! comes back as a typed receipt. Nothing here writes geometry directly, so a
//! drag is bounded, counted and reconciled by the same owner a wheel is.
//!
//! The chain a chrome interaction routes on is the grabbed region alone. A
//! thumb names one region's offset; handing the remainder outward would let a
//! drag that reached the end of its own track keep scrolling an ancestor, which
//! is not what the pointer asked for.

use crate::runtime::interaction::gesture::{UiScrollChromeLatch, UiScrollChromeLatchDenial};
use crate::runtime::pointer_affordance::{
    resolve_scroll_chrome_pointer, UiScrollChromePointerAnswer, UiScrollChromeRegionTarget,
};
use crate::runtime::scroll::chrome::{
    UiScrollChromeAxis, UiScrollChromeDragPosture, UiScrollChromePart,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiScrollChromeInteractionDenial {
    /// The point is over no region's chrome.
    NoChromeUnderPointer,
    /// The point is on the reserved corner, which belongs to no axis.
    CornerReservedForNeitherAxis,
    /// The press landed on the thumb, so it starts a drag rather than a page.
    PressIsOnTheThumb,
    /// The chrome this interaction names is no longer derivable.
    ChromeUnavailable,
    /// A new extent is staged but not displayed. Direct control cannot mutate
    /// that candidate while the pointer still names the retained predecessor.
    UnpresentedLayout,
    /// Physical acceptance has not yet reconciled into this owner's displayed
    /// geometry. Capture must not discard the sample still owed to that pose.
    AcceptedPoseUnsettled,
    Latch(UiScrollChromeLatchDenial),
    Route(crate::runtime::scroll::UiScrollRouteDenial),
    /// The placed offset left the range a Scroll offset admits.
    OffsetInadmissible,
    /// The region's mounted content or viewport box is not derivable, so
    /// there is no bounds or pose to place against.
    GeometryUnavailable,
    /// Mounted geometry refused the displayed pose this interaction placed.
    /// The placement was not committed; the accepted offset stays put.
    Geometry(crate::mounting::UiMountedOccurrenceGeometryDenial),
    /// The region was reincarnated after the press, so the latch names an
    /// owner incarnation that no longer scrolls. The move places nothing; the
    /// release still hands the capture back.
    OwnerReincarnated,
}

/// What a press on chrome started.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum UiScrollChromePressOutcome {
    /// A thumb press. The pointer is captured and the drag is latched.
    ThumbCaptured(UiScrollChromeLatch),
    /// The press is admitted, but an older host frame must finish physically
    /// before the pointer-to-thumb grab can be derived from accepted pixels.
    ThumbAwaitingPhysical,
    /// A track press. The region paged by one viewport minus one line.
    TrackPaged(crate::runtime::scroll::UiScrollRouteReceipt),
}

impl super::super::WorthUiActiveApplicationSession {
    /// What the pointer is over on `surface`, in the chrome of every region
    /// that presents some.
    pub(in crate::facade::entry) fn scroll_chrome_under_pointer(
        &self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        point: [f32; 2],
    ) -> Option<UiScrollChromePointerAnswer> {
        let regions = self.presented_scroll_chrome_facts(surface);
        resolve_regions(point, &regions)
    }

    /// Appearance preparation resolves hover against the candidate it paints;
    /// only observation routing must stay on the accepted predecessor.
    pub(in crate::facade::entry) fn prepared_scroll_chrome_under_pointer(
        &self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        point: [f32; 2],
    ) -> Option<UiScrollChromePointerAnswer> {
        resolve_regions(point, &self.scroll_chrome_facts(surface))
    }

    /// Press chrome at `point`. A thumb press captures; a track press pages.
    pub(in crate::facade::entry) fn press_scroll_chrome(
        &mut self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        point: [f32; 2],
        pointer: worth_ui_host_contract::UiHostPointerIdentity,
        capture_epoch: worth_ui_host_contract::UiHostPointerCaptureEpoch,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) -> Result<UiScrollChromePressOutcome, UiScrollChromeInteractionDenial> {
        if self
            .scroll
            .as_ref()
            .is_some_and(|scroll| scroll.has_unpresented_layout(surface))
        {
            return Err(UiScrollChromeInteractionDenial::UnpresentedLayout);
        }
        let answer = self
            .scroll_chrome_under_pointer(surface, point)
            .ok_or(UiScrollChromeInteractionDenial::NoChromeUnderPointer)?;
        let part = answer
            .part()
            .ok_or(UiScrollChromeInteractionDenial::CornerReservedForNeitherAxis)?;
        let region = self
            .region_chrome_for_owner(answer.owner())
            .ok_or(UiScrollChromeInteractionDenial::ChromeUnavailable)?;
        let target = super::scroll_direct_control::scroll_content_motion_target(
            region.owner(),
            region.mounted_instance(),
        );
        let owner_offset = self
            .scroll
            .as_ref()
            .and_then(|scroll| scroll.offset(region.owner(), region.incarnation()).ok());
        if !self.accepted_scroll_sample_settled(
            target,
            surface,
            region.mounted_offset(),
            owner_offset,
        ) {
            return Err(UiScrollChromeInteractionDenial::AcceptedPoseUnsettled);
        }
        let axis_facts = region
            .facts()
            .axis(part.axis())
            .ok_or(UiScrollChromeInteractionDenial::ChromeUnavailable)?;
        match part.part() {
            UiScrollChromePart::Thumb => {
                if self.mounted.motion_sample_presentation_pending() {
                    let pending =
                        crate::runtime::interaction::gesture::UiScrollChromePendingCapture::new(
                            pointer,
                            capture_epoch,
                            presentation.binding(),
                            presentation,
                            region.owner(),
                            region.owner_instance(),
                            region.incarnation(),
                            part.axis(),
                            point,
                        );
                    self.interaction
                        .scroll_chrome_latch_mut()
                        .begin_pending(pending)
                        .map_err(UiScrollChromeInteractionDenial::Latch)?;
                    return Ok(UiScrollChromePressOutcome::ThumbAwaitingPhysical);
                }
                let grab = crate::runtime::scroll::chrome::grab_offset_logical_points(
                    part.axis(),
                    axis_facts.thumb(),
                    point,
                );
                let latch = UiScrollChromeLatch::press(
                    pointer,
                    capture_epoch,
                    presentation.binding(),
                    region.owner(),
                    region.owner_instance(),
                    region.incarnation(),
                    part.axis(),
                    grab,
                );
                let latched = self
                    .interaction
                    .scroll_chrome_latch_mut()
                    .latch(latch)
                    .map_err(UiScrollChromeInteractionDenial::Latch)?;
                // The thumb is the pointer's from here: a settle still walking
                // this region would pull it back out from under the grab.
                self.take_direct_scroll_control(region.owner(), region.mounted_instance());
                Ok(UiScrollChromePressOutcome::ThumbCaptured(latched))
            }
            UiScrollChromePart::Track => {
                if self.mounted.motion_sample_presentation_pending() {
                    return Err(UiScrollChromeInteractionDenial::AcceptedPoseUnsettled);
                }
                let line = self
                    .declared_scroll_line_extent_logical_points(region.owner())
                    .unwrap_or(0);
                let placed = region
                    .facts()
                    .offset_for_track_click(part.axis(), point, region.mounted_offset(), line)
                    .ok_or(UiScrollChromeInteractionDenial::PressIsOnTheThumb)?;
                self.place_scroll_chrome_offset(
                    region.owner(),
                    region.incarnation(),
                    region.mounted_instance(),
                    region.slot(),
                    placed,
                    crate::runtime::scroll::UiScrollDeltaCause::ChromeTrackPage,
                )
                .map(UiScrollChromePressOutcome::TrackPaged)
            }
        }
    }

    /// Move a latched drag to `point`. The grab offset the press recorded is
    /// preserved, so the thumb keeps the spot it was grabbed by.
    pub(in crate::facade::entry) fn drag_scroll_chrome(
        &mut self,
        point: [f32; 2],
        pointer: worth_ui_host_contract::UiHostPointerIdentity,
        capture_epoch: worth_ui_host_contract::UiHostPointerCaptureEpoch,
    ) -> Result<crate::runtime::scroll::UiScrollRouteReceipt, UiScrollChromeInteractionDenial> {
        let held = self.interaction.scroll_chrome_latch().ok_or(
            UiScrollChromeInteractionDenial::Latch(UiScrollChromeLatchDenial::NotLatched),
        )?;
        if self
            .scroll
            .as_ref()
            .is_some_and(|scroll| scroll.has_unpresented_layout(held.owner().semantic_surface()))
        {
            return Err(UiScrollChromeInteractionDenial::UnpresentedLayout);
        }
        let region = self
            .region_chrome_for_owner(held.owner())
            .ok_or(UiScrollChromeInteractionDenial::ChromeUnavailable)?;
        if region.incarnation() != held.incarnation() {
            return Err(UiScrollChromeInteractionDenial::OwnerReincarnated);
        }
        let placed = region
            .facts()
            .offset_for_thumb_position(
                held.axis(),
                point,
                held.grab_offset_logical_points(),
                region.mounted_offset(),
            )
            .ok_or(UiScrollChromeInteractionDenial::ChromeUnavailable)?;
        // Leaving the gutter changes how the bar looks, never whether the drag
        // continues: capture holds until the release.
        let posture = if region.facts().axis(held.axis()).is_some_and(|axis_facts| {
            crate::runtime::scroll::chrome::rect_contains(axis_facts.track(), point)
        }) {
            UiScrollChromeDragPosture::InsideGutter
        } else {
            UiScrollChromeDragPosture::OutsideGutter
        };
        self.interaction
            .scroll_chrome_latch_mut()
            .moved(pointer, capture_epoch, posture)
            .map_err(UiScrollChromeInteractionDenial::Latch)?;
        self.place_scroll_chrome_offset(
            region.owner(),
            region.incarnation(),
            region.mounted_instance(),
            region.slot(),
            placed,
            crate::runtime::scroll::UiScrollDeltaCause::ChromeThumbDrag,
        )
    }

    /// End capture only. Ingress stages the release's final position first;
    /// that pending offset still waits for ordinary physical acceptance.
    pub(in crate::facade::entry) fn release_scroll_chrome(
        &mut self,
        pointer: worth_ui_host_contract::UiHostPointerIdentity,
        capture_epoch: worth_ui_host_contract::UiHostPointerCaptureEpoch,
    ) -> Result<UiScrollChromeLatch, UiScrollChromeInteractionDenial> {
        self.interaction
            .scroll_chrome_latch_mut()
            .release(pointer, capture_epoch)
            .map_err(UiScrollChromeInteractionDenial::Latch)
    }

    /// Whether a wheel or a key on this axis of this region must be ignored
    /// because a thumb drag already owns it.
    pub(in crate::facade::entry) fn scroll_chrome_captures_axis(
        &self,
        owner: crate::runtime::scroll::UiScrollOwnerIdentity,
        axis: UiScrollChromeAxis,
    ) -> bool {
        self.interaction.scroll_chrome_suppresses_axis(owner, axis)
    }

    /// The chrome of one owner, re-derived at the current displayed offset.
    fn region_chrome_for_owner(
        &self,
        owner: crate::runtime::scroll::UiScrollOwnerIdentity,
    ) -> Option<super::scroll_chrome_projection::UiScrollRegionChromeFacts> {
        self.presented_scroll_chrome_facts(owner.semantic_surface())
            .into_iter()
            .find(|region| region.owner() == owner)
    }

    /// Place one absolute offset on one region through Scroll's route path.
    ///
    /// The route consumes a delta, so the absolute offset the chrome named is
    /// expressed as the step from the offset Scroll currently holds. The
    /// receipt that comes back is the proof the offset was admitted, clamped
    /// and counted.
    ///
    /// The placement is all or nothing. A settle still walking the region is
    /// retired on the staged successor and its Motion ended only after mounted
    /// publication has accepted the placed pose; a refused placement leaves the
    /// offset, the pending target and the live track exactly as they were.
    pub(super) fn place_scroll_chrome_offset(
        &mut self,
        owner: crate::runtime::scroll::UiScrollOwnerIdentity,
        incarnation: crate::runtime::scroll::UiScrollOwnerIncarnation,
        mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
        slot: usize,
        placed: crate::runtime::scroll::UiScrollOffset,
        cause: crate::runtime::scroll::UiScrollDeltaCause,
    ) -> Result<crate::runtime::scroll::UiScrollRouteReceipt, UiScrollChromeInteractionDenial> {
        let scroll = self
            .scroll
            .as_ref()
            .ok_or(UiScrollChromeInteractionDenial::ChromeUnavailable)?;
        if scroll.has_unpresented_layout(owner.semantic_surface()) {
            return Err(UiScrollChromeInteractionDenial::UnpresentedLayout);
        }
        let mut successor = scroll
            .route_candidate(
                &[crate::runtime::scroll::UiScrollChainEntry::new(
                    owner,
                    incarnation,
                )],
                true,
            )
            .map_err(UiScrollChromeInteractionDenial::Route)?;
        let current = successor
            .state()
            .offset(owner, incarnation)
            .map_err(|_| UiScrollChromeInteractionDenial::ChromeUnavailable)?;
        let delta = crate::runtime::scroll::UiScrollDelta::new(
            placed
                .inline_subpixels()
                .checked_sub(current.inline_subpixels())
                .ok_or(UiScrollChromeInteractionDenial::OffsetInadmissible)?,
            placed
                .block_subpixels()
                .checked_sub(current.block_subpixels())
                .ok_or(UiScrollChromeInteractionDenial::OffsetInadmissible)?,
        );
        let (owner_instance, content, viewport) = self
            .mounted
            .scroll_region_geometry(mounted_instance, slot)
            .ok_or(UiScrollChromeInteractionDenial::GeometryUnavailable)?;
        let bounds = crate::runtime::scroll::UiScrollBounds::from_mounted_region(content, viewport)
            .ok_or(UiScrollChromeInteractionDenial::GeometryUnavailable)?;
        let request = crate::runtime::scroll::UiScrollDeltaRequest::new(
            vec![crate::runtime::scroll::UiScrollChainEntry::new(
                owner,
                incarnation,
            )],
            delta,
            cause,
        )
        .map_err(UiScrollChromeInteractionDenial::Route)?;
        // The offset placed here is direct: the successor carries no pending
        // target for a notch to accumulate on, and the route starts from the
        // offset Scroll holds rather than from an intention the pointer overrode.
        successor.state_mut().retire_transition(owner);
        let receipt = successor
            .state_mut()
            .route_with_reconciled_bounds(request, &[bounds])
            .map_err(UiScrollChromeInteractionDenial::Route)?;
        self.stage_direct_scroll_succession(
            successor.state(),
            &receipt,
            mounted_instance,
            &[Some(owner_instance)],
        )
        .map_err(UiScrollChromeInteractionDenial::Geometry)?;
        Ok(receipt)
    }
}

fn resolve_regions(
    point: [f32; 2],
    regions: &[super::scroll_chrome_projection::UiScrollRegionChromeFacts],
) -> Option<UiScrollChromePointerAnswer> {
    let targets = regions
        .iter()
        .filter(|region| region.admits_pointer(point))
        .map(|region| UiScrollChromeRegionTarget {
            owner: region.owner(),
            facts: region.facts(),
        })
        .collect::<Vec<_>>();
    resolve_scroll_chrome_pointer(point, &targets)
}
