/// Why a set of declared service policies cannot be normalized into a plan.
///
/// Normalization happens before any owner is installed and before any effect is
/// produced, so a denial here retires the whole preparation rather than leaving
/// a declaration that no owner will honour.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiServicePolicyNormalizationDenial {
    /// A scroll region declared a smooth wheel, but the Motion family is not
    /// admitted.
    ///
    /// A settle horizon is a walk the Motion owner performs over accepted
    /// samples. Without that owner the horizon names a walker that was never
    /// installed, and the only alternatives are to drop the horizon silently or
    /// to let the Scroll owner invent a second motion authority. The
    /// declaration is refused instead, naming the horizon that demanded the
    /// owner.
    SmoothWheelWithoutMotionOwner {
        /// The settle horizon the scroll policy declared, in accepted-sample
        /// ticks.
        settle_ticks: u32,
    },
}
