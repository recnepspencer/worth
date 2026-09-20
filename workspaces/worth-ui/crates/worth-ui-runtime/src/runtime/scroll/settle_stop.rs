//! Why a coarse notch's settle never became a live Motion track.
//!
//! The wheel observation that carried the notch reports only that its settle
//! went unpublished; the reason is a separate fact, because the observation's
//! denial is a stable, copyable vocabulary and the reasons are not. The three
//! the session decides are named here; everything the service-proposal lane
//! refused arrives as the same product-facing stop every other proposal stops
//! with, so a reader of one lane's stops reads them all.

/// Why a staged settle never became a live Motion track.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiScrollSettleStop {
    /// The session installs no Motion service, so there is no owner to publish
    /// the settle to.
    MotionUnavailable,
    /// Nothing is published yet, so the settle has no frame to submit into.
    NoPublishedFrame,
    /// Scroll transition staging refused the notch before any proposal existed:
    /// an owner that no longer exists or was reincarnated, an owner declaring
    /// no line extent, or a horizon that could not be measured.
    Staging { detail: Box<str> },
    /// The Scroll settle service-proposal lane refused the settle.
    Proposal(crate::runtime::intent_execution::UiRuntimeServiceProposalStop),
}
