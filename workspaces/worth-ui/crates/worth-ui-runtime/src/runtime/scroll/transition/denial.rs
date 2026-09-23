//! Typed refusals from the Scroll transition succession, each naming the exact
//! condition that stopped the request before any target was written.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiScrollTransitionDenial {
    /// No Scroll owner of that identity exists in this session.
    UnknownOwner,
    /// The owner exists under a different incarnation. A target from a retired
    /// incarnation is retired, never clamped into its replacement.
    StaleIncarnation,
    /// The declared settle horizon is zero ticks, which names no horizon at all.
    SettleHorizonUnavailable,
    /// The declared line extent is zero points, so a notch would move nothing.
    LineExtentUnusable,
    /// The accumulated target or the settle deadline left the representable
    /// range of the integer subpixel and tick domains.
    ArithmeticOutOfRange,
    /// An input tick older than the transition's latest input cannot advance or
    /// extend it, because the horizon is measured from the latest input.
    InputTickRegressed,
}
