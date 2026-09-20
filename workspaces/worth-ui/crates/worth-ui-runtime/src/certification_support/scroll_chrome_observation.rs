//! Certification's view of what scroll chrome answered for in one batch.
//!
//! A host shell proving that a press on the thumb captured it, that a drag
//! placed an offset, or that a press on the track paged, reads the chrome lane's
//! outcomes here as a flat vocabulary. The latches, receipts and denials behind
//! them are runtime authority and stay inside the crate.

/// What one claimed pointer report did to chrome.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiScrollChromeCertificationOutcome {
    /// A thumb press: the pointer is captured and the drag is latched.
    ThumbCaptured,
    /// A track press: the region paged by one viewport minus one line.
    TrackPaged,
    /// A latched drag placed the offset that keeps the grab under the pointer.
    Dragged,
    /// The release that ended a latched drag.
    Released,
    /// Chrome claimed the report and refused it.
    Denied,
}

pub trait WorthUiScrollChromeCertificationExt {
    fn scroll_chrome_interactions_for_certification(
        &self,
    ) -> Box<[UiScrollChromeCertificationOutcome]>;
}

impl WorthUiScrollChromeCertificationExt
    for crate::runtime::interaction::UiInteractionBatchReceipt
{
    fn scroll_chrome_interactions_for_certification(
        &self,
    ) -> Box<[UiScrollChromeCertificationOutcome]> {
        use crate::facade::entry::UiScrollChromeIngressOutcome as Outcome;
        self.scroll_chrome_interactions()
            .iter()
            .map(|outcome| match outcome {
                Outcome::Pressed(
                    crate::facade::entry::UiScrollChromePressOutcome::ThumbCaptured(_),
                ) => UiScrollChromeCertificationOutcome::ThumbCaptured,
                Outcome::Pressed(crate::facade::entry::UiScrollChromePressOutcome::TrackPaged(
                    _,
                )) => UiScrollChromeCertificationOutcome::TrackPaged,
                Outcome::Dragged(_) => UiScrollChromeCertificationOutcome::Dragged,
                Outcome::Released(_) => UiScrollChromeCertificationOutcome::Released,
                Outcome::Denied(_) => UiScrollChromeCertificationOutcome::Denied,
            })
            .collect::<Vec<_>>()
            .into_boxed_slice()
    }
}
