//! Where pointer ingress asks chrome before it asks the node tree.
//!
//! A scrollbar is not a mounted node, so nothing in ordinary pointer routing
//! can claim a press on it. This lane runs first, over the same validated
//! reports, and answers one question per report: does this belong to chrome?
//!
//! The answer is decided by `scroll_chrome_pointer_intent`, which reads the
//! report and the latch the pointer owner holds and nothing else. A primary
//! press asks chrome whether it landed on a bar; while a drag is latched, the
//! moves and the release of that same pointer under that same capture belong to
//! the thumb. Every other report -- a press on a node, a move with no drag
//! running, a secondary button -- is `Untouched`, and ordinary routing sees it
//! exactly as it did before chrome existed.
//!
//! Cancellation is not decided here. A drag ends where every gesture ends, in
//! the pointer owner's own `cancel_binding`, `cancel_instance` and `cancel_all`,
//! so capture introduces no lifetime rule of its own.

use super::scroll_chrome_interaction::{
    UiScrollChromeInteractionDenial, UiScrollChromePressOutcome,
};
use crate::runtime::interaction::gesture::UiScrollChromeLatch;

/// What one pointer report asks of chrome.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::facade::entry) enum UiScrollChromePointerIntent {
    /// A primary press that must ask chrome whether it landed on a bar. It is
    /// still ordinary routing's press if chrome answers that it did not.
    Press {
        pointer: worth_ui_host_contract::UiHostPointerIdentity,
        capture_epoch: worth_ui_host_contract::UiHostPointerCaptureEpoch,
        position: worth_ui_host_contract::UiHostSurfacePosition,
    },
    /// A move belonging to a latched drag.
    Drag {
        pointer: worth_ui_host_contract::UiHostPointerIdentity,
        capture_epoch: worth_ui_host_contract::UiHostPointerCaptureEpoch,
        position: worth_ui_host_contract::UiHostSurfacePosition,
    },
    /// The release that ends a latched drag.
    Release {
        pointer: worth_ui_host_contract::UiHostPointerIdentity,
        capture_epoch: worth_ui_host_contract::UiHostPointerCaptureEpoch,
        position: worth_ui_host_contract::UiHostSurfacePosition,
    },
    /// Chrome wants nothing from this report.
    Untouched,
}

/// What the chrome lane did with a report it claimed.
///
/// This is retained on the interaction batch receipt beside the scroll
/// observations, so a caller reads what chrome answered for exactly as it
/// reads what the wheel did.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum UiScrollChromeIngressOutcome {
    Pressed(UiScrollChromePressOutcome),
    PendingMoved,
    PendingReleased,
    Dragged(crate::runtime::scroll::UiScrollRouteReceipt),
    Released(UiScrollChromeLatch),
    Denied(UiScrollChromeInteractionDenial),
}

/// What the chrome lane did with one batch: the outcomes it produced, and the
/// reports it claimed so ordinary pointer routing leaves them alone.
///
/// A press that chrome answered is not also a press on whatever node happens to
/// be drawn under the bar, and the moves of a captured pointer are not hovers.
#[derive(Debug, Default)]
pub(in crate::facade::entry) struct UiScrollChromeIngressReport {
    outcomes: Vec<UiScrollChromeIngressOutcome>,
    claimed: Vec<worth_ui_host_contract::UiHostObservationSequence>,
}

impl UiScrollChromeIngressReport {
    pub(in crate::facade::entry) fn into_parts(
        self,
    ) -> (
        Vec<UiScrollChromeIngressOutcome>,
        Vec<worth_ui_host_contract::UiHostObservationSequence>,
    ) {
        (self.outcomes, self.claimed)
    }
}

/// Classify one report against the latch the pointer owner currently holds.
///
/// `latched` is the pointer and capture epoch of the drag in progress, if any.
/// A report from another pointer, or from the same pointer under a capture the
/// host has since replaced, is not this drag's and is left to ordinary routing.
pub(in crate::facade::entry) fn scroll_chrome_pointer_intent(
    payload: &worth_ui_host_contract::UiHostObservationPayload,
    latched: Option<(
        worth_ui_host_contract::UiHostPointerIdentity,
        worth_ui_host_contract::UiHostPointerCaptureEpoch,
    )>,
) -> UiScrollChromePointerIntent {
    match payload {
        worth_ui_host_contract::UiHostObservationPayload::PointerButton {
            pointer,
            capture_epoch,
            button: worth_ui_host_contract::UiHostPointerButton::Primary,
            transition,
            position,
        } => match transition {
            worth_ui_host_contract::UiHostPointerButtonTransition::Pressed => {
                if latched.is_some() {
                    // One pointer drags one thumb; a second press while a drag
                    // runs is not chrome's to take.
                    UiScrollChromePointerIntent::Untouched
                } else {
                    UiScrollChromePointerIntent::Press {
                        pointer: *pointer,
                        capture_epoch: *capture_epoch,
                        position: *position,
                    }
                }
            }
            worth_ui_host_contract::UiHostPointerButtonTransition::Released => {
                if latched == Some((*pointer, *capture_epoch)) {
                    UiScrollChromePointerIntent::Release {
                        pointer: *pointer,
                        capture_epoch: *capture_epoch,
                        position: *position,
                    }
                } else {
                    UiScrollChromePointerIntent::Untouched
                }
            }
        },
        worth_ui_host_contract::UiHostObservationPayload::PointerMotion {
            pointer,
            capture_epoch,
            position,
            ..
        } => {
            if latched == Some((*pointer, *capture_epoch)) {
                UiScrollChromePointerIntent::Drag {
                    pointer: *pointer,
                    capture_epoch: *capture_epoch,
                    position: *position,
                }
            } else {
                UiScrollChromePointerIntent::Untouched
            }
        }
        _ => UiScrollChromePointerIntent::Untouched,
    }
}

/// Zero the components of a host delta whose axis a thumb drag holds.
///
/// A wheel arriving mid-drag is not refused outright: the reader may well be
/// dragging one bar and wheeling the other axis, and that other axis is still
/// theirs to scroll. Only the axis the drag owns is silenced.
pub(in crate::facade::entry) fn suppress_captured_axes(
    delta_subpixels: [i64; 2],
    captured: [bool; 2],
) -> [i64; 2] {
    [
        if captured[0] { 0 } else { delta_subpixels[0] },
        if captured[1] { 0 } else { delta_subpixels[1] },
    ]
}

/// The surface position a pointer report carries, whichever kind it is.
pub(in crate::facade::entry) fn pointer_report_position(
    payload: &worth_ui_host_contract::UiHostObservationPayload,
) -> Option<worth_ui_host_contract::UiHostSurfacePosition> {
    match payload {
        worth_ui_host_contract::UiHostObservationPayload::PointerMotion { position, .. }
        | worth_ui_host_contract::UiHostObservationPayload::PointerButton { position, .. } => {
            Some(*position)
        }
        _ => None,
    }
}

/// The platform point a report names, in the viewport space chrome is
/// derived in. A position on another basis names none, and the lane declines
/// the report, leaving ordinary routing to refuse it.
pub(in crate::facade::entry) fn chrome_point(
    position: worth_ui_host_contract::UiHostSurfacePosition,
) -> Option<crate::mounting::presentation::UiPlatformPoint> {
    crate::mounting::presentation::UiPlatformPoint::from_host_position(position).ok()
}

impl super::super::WorthUiActiveApplicationSession {
    /// Run the chrome lane over one validated batch, before ordinary pointer
    /// routing ingests it.
    pub(in crate::facade::entry) fn observe_scroll_chrome_pointer_reports(
        &mut self,
        reports: &[crate::facade::observation_report::UiValidatedHostObservationReport],
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) -> UiScrollChromeIngressReport {
        let mut report_out = UiScrollChromeIngressReport::default();
        let Some(surface) = self
            .mounted
            .current_surface_for_binding(presentation.binding())
        else {
            return report_out;
        };
        for report in reports {
            // Every pointer report, claimed or not, is where the pointer now
            // is; presentation re-resolves hover from that position against
            // the pose it paints.
            if let Some(point) =
                pointer_report_position(report.report().payload()).and_then(chrome_point)
            {
                self.interaction.observe_scroll_chrome_hover(
                    surface,
                    presentation.binding(),
                    point,
                );
            }
            let latched = self.interaction.scroll_chrome_capture_identity();
            let intent = scroll_chrome_pointer_intent(report.report().payload(), latched);
            if let Some(outcome) = self.answer_scroll_chrome_intent(intent, surface, presentation) {
                report_out.outcomes.push(outcome);
                report_out.claimed.push(report.report().sequence());
            }
        }
        report_out
    }

    fn answer_scroll_chrome_intent(
        &mut self,
        intent: UiScrollChromePointerIntent,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) -> Option<UiScrollChromeIngressOutcome> {
        match intent {
            UiScrollChromePointerIntent::Untouched => None,
            UiScrollChromePointerIntent::Press {
                pointer,
                capture_epoch,
                position,
            } => {
                let point = chrome_point(position)?;
                // A press over no chrome is an ordinary press, not a denial:
                // the lane declines it and leaves the report where it was.
                self.scroll_chrome_under_pointer(surface, point)?;
                Some(
                    match self.press_scroll_chrome(
                        surface,
                        point,
                        pointer,
                        capture_epoch,
                        presentation,
                    ) {
                        Ok(outcome) => UiScrollChromeIngressOutcome::Pressed(outcome),
                        Err(denial) => UiScrollChromeIngressOutcome::Denied(denial),
                    },
                )
            }
            UiScrollChromePointerIntent::Drag {
                pointer,
                capture_epoch,
                position,
            } => {
                if self
                    .interaction
                    .scroll_chrome_latch_mut()
                    .pending()
                    .is_some()
                {
                    return Some(
                        match self.interaction.scroll_chrome_latch_mut().move_pending(
                            pointer,
                            capture_epoch,
                            chrome_point(position)?,
                            false,
                        ) {
                            Ok(_) => UiScrollChromeIngressOutcome::PendingMoved,
                            Err(denial) => UiScrollChromeIngressOutcome::Denied(
                                UiScrollChromeInteractionDenial::Latch(denial),
                            ),
                        },
                    );
                }
                Some(
                    match self.drag_scroll_chrome(chrome_point(position)?, pointer, capture_epoch) {
                        Ok(receipt) => UiScrollChromeIngressOutcome::Dragged(receipt),
                        Err(denial) => UiScrollChromeIngressOutcome::Denied(denial),
                    },
                )
            }
            UiScrollChromePointerIntent::Release {
                pointer,
                capture_epoch,
                position,
            } => {
                // A release on a basis chrome cannot read places nothing, but
                // it still ends the drag it names.
                let point =
                    crate::mounting::presentation::UiPlatformPoint::from_host_position(position)
                        .map_err(UiScrollChromeInteractionDenial::PositionBasisRefused);
                if let Some(pending) = self.interaction.scroll_chrome_latch_mut().pending() {
                    let released = self.interaction.scroll_chrome_latch_mut().move_pending(
                        pointer,
                        capture_epoch,
                        point.unwrap_or(pending.latest_point()),
                        true,
                    );
                    return Some(match (released, point) {
                        (Err(denial), _) => UiScrollChromeIngressOutcome::Denied(
                            UiScrollChromeInteractionDenial::Latch(denial),
                        ),
                        (Ok(_), Err(denial)) => UiScrollChromeIngressOutcome::Denied(denial),
                        (Ok(_), Ok(_)) => UiScrollChromeIngressOutcome::PendingReleased,
                    });
                }
                // The OS may coalesce the final move before button-up. Its
                // event-time position is the final drag intent, not merely hover.
                // Stage through ordinary acceptance, then release even on denial.
                let placement =
                    point.and_then(|point| self.drag_scroll_chrome(point, pointer, capture_epoch));
                let release = self.release_scroll_chrome(pointer, capture_epoch);
                Some(match (placement, release) {
                    (Ok(_), Ok(latch)) => UiScrollChromeIngressOutcome::Released(latch),
                    (Err(denial), _) | (_, Err(denial)) => {
                        UiScrollChromeIngressOutcome::Denied(denial)
                    }
                })
            }
        }
    }
}
