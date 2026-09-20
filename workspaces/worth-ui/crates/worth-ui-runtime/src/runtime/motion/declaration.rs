#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum UiMotionPropertyChannel {
    Opacity,
    TranslationX,
    TranslationY,
    Geometry,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiMotionPropertyChannels(u8);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiMotionEasing {
    /// A fixed-shape ease shared by every animated component. Its start rate is
    /// whatever the shape dictates, so an interruption restarts the shape.
    EaseOutCubic,
    /// A cubic Hermite that leaves the interrupted sample at the rate that
    /// sample was already moving and arrives at the endpoint at rest. This is
    /// the one declared curve family for retargeted settlement.
    VelocityMatchedCubic,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiMotionFillPolicy {
    FinalState,
    ExitRetention,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiMotionInterruptionPolicy {
    RetargetFromCurrentSample,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiMotionReducedMotionPolicy {
    SystemRespecting,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiMotionDeclaration {
    channels: UiMotionPropertyChannels,
    easing: UiMotionEasing,
    duration_ticks: u32,
    delay_ticks: u32,
    fill: UiMotionFillPolicy,
    interruption: UiMotionInterruptionPolicy,
    reduced_motion: UiMotionReducedMotionPolicy,
    decorative: bool,
}

impl UiMotionPropertyChannels {
    pub(in crate::runtime) const fn one(channel: UiMotionPropertyChannel) -> Self {
        Self(bit(channel))
    }

    pub(in crate::runtime) const fn with(mut self, channel: UiMotionPropertyChannel) -> Self {
        self.0 |= bit(channel);
        self
    }

    pub(crate) const fn contains(self, channel: UiMotionPropertyChannel) -> bool {
        self.0 & bit(channel) != 0
    }
}

impl UiMotionDeclaration {
    pub(crate) const fn portal_entrance() -> Self {
        Self {
            channels: UiMotionPropertyChannels::one(UiMotionPropertyChannel::Opacity)
                .with(UiMotionPropertyChannel::TranslationY),
            easing: UiMotionEasing::EaseOutCubic,
            duration_ticks: 140,
            delay_ticks: 0,
            fill: UiMotionFillPolicy::FinalState,
            interruption: UiMotionInterruptionPolicy::RetargetFromCurrentSample,
            reduced_motion: UiMotionReducedMotionPolicy::SystemRespecting,
            decorative: true,
        }
    }

    pub(crate) const fn portal_exit() -> Self {
        Self {
            channels: UiMotionPropertyChannels::one(UiMotionPropertyChannel::Opacity),
            easing: UiMotionEasing::EaseOutCubic,
            duration_ticks: 110,
            delay_ticks: 0,
            fill: UiMotionFillPolicy::ExitRetention,
            interruption: UiMotionInterruptionPolicy::RetargetFromCurrentSample,
            reduced_motion: UiMotionReducedMotionPolicy::SystemRespecting,
            decorative: true,
        }
    }

    pub(crate) const fn rebind_geometry() -> Self {
        Self {
            channels: UiMotionPropertyChannels::one(UiMotionPropertyChannel::Geometry),
            easing: UiMotionEasing::EaseOutCubic,
            duration_ticks: 160,
            delay_ticks: 0,
            fill: UiMotionFillPolicy::FinalState,
            interruption: UiMotionInterruptionPolicy::RetargetFromCurrentSample,
            reduced_motion: UiMotionReducedMotionPolicy::SystemRespecting,
            decorative: false,
        }
    }

    /// Settlement of a Scroll region's content toward its succession target.
    /// `settle_ticks` is the horizon measured from the latest accepted input,
    /// not a fixed duration restarted per event, and an interrupting notch
    /// retargets from the current sample's position and velocity.
    pub(crate) const fn scroll_settle(settle_ticks: u32) -> Self {
        Self {
            channels: UiMotionPropertyChannels::one(UiMotionPropertyChannel::TranslationX)
                .with(UiMotionPropertyChannel::TranslationY),
            easing: UiMotionEasing::VelocityMatchedCubic,
            duration_ticks: settle_ticks,
            delay_ticks: 0,
            fill: UiMotionFillPolicy::FinalState,
            interruption: UiMotionInterruptionPolicy::RetargetFromCurrentSample,
            reduced_motion: UiMotionReducedMotionPolicy::SystemRespecting,
            decorative: false,
        }
    }

    pub(crate) const fn channels(self) -> UiMotionPropertyChannels {
        self.channels
    }

    pub(crate) const fn easing(self) -> UiMotionEasing {
        self.easing
    }

    pub(crate) const fn duration_ticks(self) -> u32 {
        self.duration_ticks
    }

    pub(crate) const fn delay_ticks(self) -> u32 {
        self.delay_ticks
    }

    pub(in crate::runtime) const fn fill(self) -> UiMotionFillPolicy {
        self.fill
    }

    pub(in crate::runtime) const fn interruption(self) -> UiMotionInterruptionPolicy {
        self.interruption
    }

    pub(crate) const fn reduced_motion(self) -> UiMotionReducedMotionPolicy {
        self.reduced_motion
    }

    pub(crate) const fn decorative(self) -> bool {
        self.decorative
    }

    pub(super) const fn with_policy(mut self, policy: crate::declaration::UiMotionPolicy) -> Self {
        if self.decorative
            && matches!(
                policy.decorative_reduced_motion(),
                crate::declaration::UiReducedMotionBehavior::PreserveSemanticTransition
            )
        {
            self.decorative = false;
        }
        self
    }
}

const fn bit(channel: UiMotionPropertyChannel) -> u8 {
    1 << channel as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portal_defaults_are_system_respecting_and_exit_retention_is_explicit() {
        let entrance = UiMotionDeclaration::portal_entrance();
        assert!(entrance
            .channels()
            .contains(UiMotionPropertyChannel::Opacity));
        assert!(entrance
            .channels()
            .contains(UiMotionPropertyChannel::TranslationY));
        assert_eq!(
            entrance.reduced_motion(),
            UiMotionReducedMotionPolicy::SystemRespecting
        );
        assert_eq!(
            UiMotionDeclaration::portal_exit().fill(),
            UiMotionFillPolicy::ExitRetention
        );
    }

    /// Portal and Motion must reach one exit-retention decision per transition.
    /// Portal mints a retention only when it commits `Closing`, and Motion mints
    /// one only for an `ExitRetention` fill. Those agree only while `portal_exit`
    /// is the sole declaration carrying that fill, so an opening transition can
    /// never mint an exit retention its portal has no counterpart for.
    #[test]
    fn only_the_exit_declaration_retains_so_an_opening_transition_cannot_pair_alone() {
        assert_eq!(
            UiMotionDeclaration::portal_entrance().fill(),
            UiMotionFillPolicy::FinalState
        );
        assert_eq!(
            UiMotionDeclaration::rebind_geometry().fill(),
            UiMotionFillPolicy::FinalState
        );
        assert_eq!(
            UiMotionDeclaration::scroll_settle(120).fill(),
            UiMotionFillPolicy::FinalState
        );
        assert_eq!(
            UiMotionDeclaration::portal_exit().fill(),
            UiMotionFillPolicy::ExitRetention
        );
    }

    /// Scroll settlement is the declared curve family for retargeting: it moves
    /// only the two translation channels, carries the settle horizon it was
    /// given, and is never decorative, so reduced motion settles it directly
    /// instead of snapping it away.
    #[test]
    fn scroll_settlement_declares_translation_only_velocity_matched_motion() {
        let settle = UiMotionDeclaration::scroll_settle(120);

        assert!(settle
            .channels()
            .contains(UiMotionPropertyChannel::TranslationX));
        assert!(settle
            .channels()
            .contains(UiMotionPropertyChannel::TranslationY));
        assert!(!settle.channels().contains(UiMotionPropertyChannel::Opacity));
        assert!(!settle
            .channels()
            .contains(UiMotionPropertyChannel::Geometry));
        assert_eq!(settle.easing(), UiMotionEasing::VelocityMatchedCubic);
        assert_eq!(settle.duration_ticks(), 120);
        assert_eq!(settle.delay_ticks(), 0);
        assert_eq!(
            settle.interruption(),
            UiMotionInterruptionPolicy::RetargetFromCurrentSample
        );
        assert!(!settle.decorative());
    }

    #[test]
    fn public_decorative_policy_changes_reduced_motion_treatment() {
        let declaration = UiMotionDeclaration::portal_entrance().with_policy(
            crate::declaration::UiMotionPolicy::system_respecting().with_decorative_reduced_motion(
                crate::declaration::UiReducedMotionBehavior::PreserveSemanticTransition,
            ),
        );

        assert!(!declaration.decorative());
        assert_eq!(
            declaration.reduced_motion(),
            UiMotionReducedMotionPolicy::SystemRespecting
        );
    }
}
