mod retained_charge;

use serde::{Deserialize, Serialize};

use crate::data::error::SignalError;

use super::clock::{ClockDomain, ClockTick};
use super::units::{IntervalPeriod, TemporalDuration};

fn validate_authoritative_clock_domain(domain: ClockDomain) -> Result<ClockDomain, SignalError> {
    if domain.authority().is_authoritative() {
        Ok(domain)
    } else {
        Err(SignalError::invalid_input(format!(
            "{domain:?} is metadata-only and cannot drive temporal policy eligibility"
        )))
    }
}

/// Anchor used to derive the cadence origin for recurring temporal policies.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntervalAnchor {
    Registration,
    FirstEvaluation,
    ExplicitTick(ClockTick),
}

/// Policy used when elapsed time spans more than one recurring interval.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MissedTickPolicy {
    CollapseToOne,
    CatchUpAll,
    SkipToLatest,
}

/// Relative delay semantics for `After`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AfterCondition {
    delay: TemporalDuration,
    clock_domain: ClockDomain,
}

impl AfterCondition {
    pub fn new(delay: TemporalDuration) -> Self {
        Self {
            delay,
            clock_domain: ClockDomain::MonotonicExecution,
        }
    }

    pub fn try_new(delay_ms: u64) -> Result<Self, SignalError> {
        Ok(Self::new(TemporalDuration::temporal_duration(delay_ms)?))
    }

    pub fn delay(self) -> TemporalDuration {
        self.delay
    }

    pub fn delay_ms(self) -> u64 {
        self.delay.get()
    }

    pub fn clock_domain(self) -> ClockDomain {
        self.clock_domain
    }

    pub fn with_clock_domain(mut self, domain: ClockDomain) -> Result<Self, SignalError> {
        self.clock_domain = validate_authoritative_clock_domain(domain)?;
        Ok(self)
    }
}

/// Absolute threshold semantics for `AtOrAfter`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AtOrAfterCondition {
    tick: ClockTick,
    clock_domain: ClockDomain,
}

impl AtOrAfterCondition {
    pub fn new(tick: ClockTick) -> Self {
        Self {
            tick,
            clock_domain: ClockDomain::MonotonicExecution,
        }
    }

    pub fn tick(self) -> ClockTick {
        self.tick
    }

    pub fn clock_domain(self) -> ClockDomain {
        self.clock_domain
    }

    pub fn with_clock_domain(mut self, domain: ClockDomain) -> Result<Self, SignalError> {
        self.clock_domain = validate_authoritative_clock_domain(domain)?;
        Ok(self)
    }
}

/// Quiet-period coalescing semantics for `Debounce`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DebounceCondition {
    quiet_period: TemporalDuration,
    clock_domain: ClockDomain,
}

impl DebounceCondition {
    pub fn new(quiet_period: TemporalDuration) -> Self {
        Self {
            quiet_period,
            clock_domain: ClockDomain::MonotonicExecution,
        }
    }

    pub fn try_new(quiet_period_ms: u64) -> Result<Self, SignalError> {
        Ok(Self::new(TemporalDuration::temporal_duration(
            quiet_period_ms,
        )?))
    }

    pub fn quiet_period(self) -> TemporalDuration {
        self.quiet_period
    }

    pub fn quiet_period_ms(self) -> u64 {
        self.quiet_period.get()
    }

    pub fn clock_domain(self) -> ClockDomain {
        self.clock_domain
    }

    pub fn with_clock_domain(mut self, domain: ClockDomain) -> Result<Self, SignalError> {
        self.clock_domain = validate_authoritative_clock_domain(domain)?;
        Ok(self)
    }
}

/// Rate-limiting semantics for `Throttle`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThrottleCondition {
    window: TemporalDuration,
    clock_domain: ClockDomain,
}

impl ThrottleCondition {
    pub fn new(window: TemporalDuration) -> Self {
        Self {
            window,
            clock_domain: ClockDomain::MonotonicExecution,
        }
    }

    pub fn try_new(window_ms: u64) -> Result<Self, SignalError> {
        Ok(Self::new(TemporalDuration::temporal_duration(window_ms)?))
    }

    pub fn window(self) -> TemporalDuration {
        self.window
    }

    pub fn window_ms(self) -> u64 {
        self.window.get()
    }

    pub fn clock_domain(self) -> ClockDomain {
        self.clock_domain
    }

    pub fn with_clock_domain(mut self, domain: ClockDomain) -> Result<Self, SignalError> {
        self.clock_domain = validate_authoritative_clock_domain(domain)?;
        Ok(self)
    }
}

/// Freshness-expiry semantics for `StaleAfter`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct StaleAfterCondition {
    stale_after: TemporalDuration,
    clock_domain: ClockDomain,
}

impl StaleAfterCondition {
    pub fn new(stale_after: TemporalDuration) -> Self {
        Self {
            stale_after,
            clock_domain: ClockDomain::MonotonicExecution,
        }
    }

    pub fn try_new(stale_after_ms: u64) -> Result<Self, SignalError> {
        Ok(Self::new(TemporalDuration::temporal_duration(
            stale_after_ms,
        )?))
    }

    pub fn stale_after(self) -> TemporalDuration {
        self.stale_after
    }

    pub fn stale_after_ms(self) -> u64 {
        self.stale_after.get()
    }

    pub fn clock_domain(self) -> ClockDomain {
        self.clock_domain
    }

    pub fn with_clock_domain(mut self, domain: ClockDomain) -> Result<Self, SignalError> {
        self.clock_domain = validate_authoritative_clock_domain(domain)?;
        Ok(self)
    }
}

/// Declarative interval semantics lowered into runtime-owned recurring wakes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntervalCondition {
    period: IntervalPeriod,
    anchor: IntervalAnchor,
    missed_tick_policy: MissedTickPolicy,
    clock_domain: ClockDomain,
}

impl IntervalCondition {
    pub fn new(period: IntervalPeriod) -> Self {
        Self {
            period,
            anchor: IntervalAnchor::Registration,
            missed_tick_policy: MissedTickPolicy::CollapseToOne,
            clock_domain: ClockDomain::MonotonicExecution,
        }
    }

    pub fn try_new(period_ms: u64) -> Result<Self, SignalError> {
        Ok(Self::new(IntervalPeriod::interval_period(period_ms)?))
    }

    pub fn period(&self) -> IntervalPeriod {
        self.period
    }

    pub fn period_ms(&self) -> u64 {
        self.period.get()
    }

    pub fn anchor(&self) -> &IntervalAnchor {
        &self.anchor
    }

    pub fn missed_tick_policy(&self) -> &MissedTickPolicy {
        &self.missed_tick_policy
    }

    pub fn clock_domain(&self) -> ClockDomain {
        self.clock_domain
    }

    pub fn with_anchor(mut self, anchor: IntervalAnchor) -> Self {
        self.anchor = anchor;
        self
    }

    pub fn with_missed_tick_policy(mut self, missed_tick_policy: MissedTickPolicy) -> Self {
        self.missed_tick_policy = missed_tick_policy;
        self
    }

    pub fn with_clock_domain(mut self, domain: ClockDomain) -> Result<Self, SignalError> {
        self.clock_domain = validate_authoritative_clock_domain(domain)?;
        Ok(self)
    }
}

/// First-class temporal policy vocabulary owned by the temporal subsystem.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TemporalCondition {
    After(AfterCondition),
    AtOrAfter(AtOrAfterCondition),
    Debounce(DebounceCondition),
    Throttle(ThrottleCondition),
    StaleAfter(StaleAfterCondition),
    Interval(IntervalCondition),
}

impl TemporalCondition {
    pub fn after(delay_ms: u64) -> Result<Self, SignalError> {
        Ok(Self::After(AfterCondition::try_new(delay_ms)?))
    }

    pub fn at_or_after(tick: ClockTick) -> Self {
        Self::AtOrAfter(AtOrAfterCondition::new(tick))
    }

    pub fn debounce(quiet_period_ms: u64) -> Result<Self, SignalError> {
        Ok(Self::Debounce(DebounceCondition::try_new(quiet_period_ms)?))
    }

    pub fn throttle(window_ms: u64) -> Result<Self, SignalError> {
        Ok(Self::Throttle(ThrottleCondition::try_new(window_ms)?))
    }

    pub fn stale_after(stale_after_ms: u64) -> Result<Self, SignalError> {
        Ok(Self::StaleAfter(StaleAfterCondition::try_new(
            stale_after_ms,
        )?))
    }

    pub fn interval(interval: IntervalCondition) -> Self {
        Self::Interval(interval)
    }

    pub fn clock_domain(&self) -> ClockDomain {
        match self {
            Self::After(condition) => condition.clock_domain(),
            Self::AtOrAfter(condition) => condition.clock_domain(),
            Self::Debounce(condition) => condition.clock_domain(),
            Self::Throttle(condition) => condition.clock_domain(),
            Self::StaleAfter(condition) => condition.clock_domain(),
            Self::Interval(condition) => condition.clock_domain(),
        }
    }
}
