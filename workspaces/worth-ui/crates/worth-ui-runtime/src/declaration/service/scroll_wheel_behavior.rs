/// Why a declared smooth wheel behavior was refused.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiScrollWheelBehaviorDenial {
    /// A settle horizon of zero ticks is an immediate wheel wearing a smooth
    /// name. Declare [`UiScrollWheelBehavior::immediate`] instead.
    SettleHorizonIsZero,
    /// The settle horizon outlives
    /// [`UI_SCROLL_WHEEL_SETTLE_TICK_CEILING`], beyond which a wheel gesture
    /// stops reading as a settle and starts reading as an animation the reader
    /// cannot interrupt.
    SettleHorizonExceedsCeiling,
}

/// The longest settle horizon a smooth wheel may declare, in accepted-sample
/// ticks.
pub const UI_SCROLL_WHEEL_SETTLE_TICK_CEILING: u32 = 2_000;

/// How a scroll owner answers one wheel notch.
///
/// An immediate wheel moves the offset on the observation that carried the
/// notch. A smooth wheel declares a settle horizon instead: the offset becomes
/// a target that the Motion owner walks to over that many accepted-sample
/// ticks. The horizon is the declaration; nothing here schedules, samples or
/// holds authority over a frame.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UiScrollWheelBehavior {
    settle_ticks: Option<u32>,
}

impl UiScrollWheelBehavior {
    pub const fn immediate() -> Self {
        Self { settle_ticks: None }
    }

    pub fn smooth(settle_ticks: u32) -> Result<Self, UiScrollWheelBehaviorDenial> {
        if settle_ticks == 0 {
            return Err(UiScrollWheelBehaviorDenial::SettleHorizonIsZero);
        }
        if settle_ticks > UI_SCROLL_WHEEL_SETTLE_TICK_CEILING {
            return Err(UiScrollWheelBehaviorDenial::SettleHorizonExceedsCeiling);
        }
        Ok(Self {
            settle_ticks: Some(settle_ticks),
        })
    }

    /// The declared settle horizon, or `None` for an immediate wheel.
    pub const fn settle_ticks(self) -> Option<u32> {
        self.settle_ticks
    }

    /// The bits this behavior contributes to the scroll policy digest, before
    /// the policy shifts them into its own field.
    pub(crate) const fn digest_basis(self) -> u64 {
        match self.settle_ticks {
            None => 0,
            Some(settle_ticks) => 1 | (settle_ticks as u64) << 8,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        UiScrollWheelBehavior, UiScrollWheelBehaviorDenial, UI_SCROLL_WHEEL_SETTLE_TICK_CEILING,
    };

    /// A settle horizon exists or it does not; there is no zero-length settle
    /// and no unbounded one.
    #[test]
    fn a_smooth_wheel_must_declare_a_horizon_within_the_named_ceiling() {
        assert_eq!(UiScrollWheelBehavior::immediate().settle_ticks(), None);
        assert_eq!(
            UiScrollWheelBehavior::smooth(0),
            Err(UiScrollWheelBehaviorDenial::SettleHorizonIsZero)
        );
        assert_eq!(
            UiScrollWheelBehavior::smooth(UI_SCROLL_WHEEL_SETTLE_TICK_CEILING + 1),
            Err(UiScrollWheelBehaviorDenial::SettleHorizonExceedsCeiling)
        );
        assert_eq!(
            UiScrollWheelBehavior::smooth(1).map(UiScrollWheelBehavior::settle_ticks),
            Ok(Some(1))
        );
        assert_eq!(
            UiScrollWheelBehavior::smooth(UI_SCROLL_WHEEL_SETTLE_TICK_CEILING)
                .map(UiScrollWheelBehavior::settle_ticks),
            Ok(Some(UI_SCROLL_WHEEL_SETTLE_TICK_CEILING))
        );
    }

    /// Two horizons that differ are two declarations that differ.
    #[test]
    fn every_admitted_horizon_has_its_own_digest_basis() {
        let immediate = UiScrollWheelBehavior::immediate().digest_basis();
        let one = UiScrollWheelBehavior::smooth(1)
            .expect("one tick is an admitted horizon")
            .digest_basis();
        let many = UiScrollWheelBehavior::smooth(120)
            .expect("120 ticks is an admitted horizon")
            .digest_basis();

        assert_ne!(immediate, one);
        assert_ne!(immediate, many);
        assert_ne!(one, many);
    }
}
