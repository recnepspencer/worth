//! The settle horizon: how long after the latest input a transition may keep
//! settling.
//!
//! The horizon is `settle_ticks` measured from the *latest* input, not a fixed
//! duration restarted per event. A notch arriving mid-settle therefore extends
//! the same horizon; it never replaces one animation with a fresh one, and it
//! can never shorten a horizon an earlier input already established.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiScrollSettleHorizon {
    latest_input_tick: u64,
    deadline_tick: u64,
}

impl UiScrollSettleHorizon {
    pub(crate) const fn from_latest_input(
        latest_input_tick: u64,
        settle_ticks: u32,
    ) -> Result<Self, super::UiScrollTransitionDenial> {
        if settle_ticks == 0 {
            return Err(super::UiScrollTransitionDenial::SettleHorizonUnavailable);
        }
        match latest_input_tick.checked_add(settle_ticks as u64) {
            None => Err(super::UiScrollTransitionDenial::ArithmeticOutOfRange),
            Some(deadline_tick) => Ok(Self {
                latest_input_tick,
                deadline_tick,
            }),
        }
    }

    /// Extend this horizon to a later input. The deadline moves to the later of
    /// the two, so an out-of-order tick can never cut a settle short.
    pub(crate) const fn extended_to_latest_input(
        self,
        latest_input_tick: u64,
        settle_ticks: u32,
    ) -> Result<Self, super::UiScrollTransitionDenial> {
        if latest_input_tick < self.latest_input_tick {
            return Err(super::UiScrollTransitionDenial::InputTickRegressed);
        }
        match Self::from_latest_input(latest_input_tick, settle_ticks) {
            Err(denial) => Err(denial),
            Ok(extended) => Ok(Self {
                latest_input_tick,
                deadline_tick: if extended.deadline_tick > self.deadline_tick {
                    extended.deadline_tick
                } else {
                    self.deadline_tick
                },
            }),
        }
    }

    #[cfg(test)]
    pub(crate) const fn latest_input_tick(self) -> u64 {
        self.latest_input_tick
    }

    #[cfg(test)]
    pub(crate) const fn deadline_tick(self) -> u64 {
        self.deadline_tick
    }

    pub(crate) const fn remaining_ticks(self, tick: u64) -> u64 {
        self.deadline_tick.saturating_sub(tick)
    }
}
