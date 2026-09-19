mod retained_charge;

use serde::{Deserialize, Serialize};

use super::{ClockDomain, ClockTick, TemporalCondition, TemporalWakeId, WakeOrdinal};

/// Authority posture used to admit or defer temporal eligibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TemporalEligibilityAuthority {
    RuntimeClockBasis,
    RuntimeScheduledWake,
    ResolverFallback,
}

/// Proof that temporal meaning was lowered before execution admission.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoweredTemporalEligibility {
    Ready(ReadyTemporalEligibility),
    Deferred(DeferredTemporalEligibility),
}

impl LoweredTemporalEligibility {
    pub fn condition(&self) -> &TemporalCondition {
        match self {
            Self::Ready(eligibility) => eligibility.condition(),
            Self::Deferred(eligibility) => eligibility.condition(),
        }
    }

    pub fn clock_domain(&self) -> ClockDomain {
        match self {
            Self::Ready(eligibility) => eligibility.clock_domain(),
            Self::Deferred(eligibility) => eligibility.clock_domain(),
        }
    }

    pub fn authority(&self) -> TemporalEligibilityAuthority {
        match self {
            Self::Ready(eligibility) => eligibility.authority(),
            Self::Deferred(eligibility) => eligibility.authority(),
        }
    }

    pub fn authority_tick(&self) -> Option<ClockTick> {
        match self {
            Self::Ready(eligibility) => eligibility.authority_tick(),
            Self::Deferred(eligibility) => eligibility.authority_tick(),
        }
    }

    pub fn ready_by_time(&self) -> bool {
        matches!(self, Self::Ready(_))
    }
}

/// Compact execution-visible temporal evidence summary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TemporalExecutionSummary {
    ready_count: u32,
    deferred_count: u32,
    runtime_clock_authority_count: u32,
    resolver_fallback_count: u32,
    runtime_scheduled_wake_count: u32,
}

impl TemporalExecutionSummary {
    pub fn ready_count(&self) -> u32 {
        self.ready_count
    }

    pub fn deferred_count(&self) -> u32 {
        self.deferred_count
    }

    pub fn runtime_clock_authority_count(&self) -> u32 {
        self.runtime_clock_authority_count
    }

    pub fn resolver_fallback_count(&self) -> u32 {
        self.resolver_fallback_count
    }

    pub fn runtime_scheduled_wake_count(&self) -> u32 {
        self.runtime_scheduled_wake_count
    }

    pub fn total_count(&self) -> u32 {
        self.ready_count.saturating_add(self.deferred_count)
    }

    pub fn observe(&mut self, temporal_eligibility: &LoweredTemporalEligibility) {
        if temporal_eligibility.ready_by_time() {
            self.ready_count = self.ready_count.saturating_add(1);
        } else {
            self.deferred_count = self.deferred_count.saturating_add(1);
        }
        match temporal_eligibility.authority() {
            TemporalEligibilityAuthority::RuntimeClockBasis => {
                self.runtime_clock_authority_count =
                    self.runtime_clock_authority_count.saturating_add(1);
            }
            TemporalEligibilityAuthority::RuntimeScheduledWake => {
                self.runtime_scheduled_wake_count =
                    self.runtime_scheduled_wake_count.saturating_add(1);
            }
            TemporalEligibilityAuthority::ResolverFallback => {
                self.resolver_fallback_count = self.resolver_fallback_count.saturating_add(1);
            }
        }
    }

    pub fn absorb(&mut self, other: Self) {
        self.ready_count = self.ready_count.saturating_add(other.ready_count);
        self.deferred_count = self.deferred_count.saturating_add(other.deferred_count);
        self.runtime_clock_authority_count = self
            .runtime_clock_authority_count
            .saturating_add(other.runtime_clock_authority_count);
        self.resolver_fallback_count = self
            .resolver_fallback_count
            .saturating_add(other.resolver_fallback_count);
        self.runtime_scheduled_wake_count = self
            .runtime_scheduled_wake_count
            .saturating_add(other.runtime_scheduled_wake_count);
    }
}

/// Proof that a temporal condition was lowered and admitted as ready.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadyTemporalEligibility {
    condition: TemporalCondition,
    clock_domain: ClockDomain,
    authority: TemporalEligibilityAuthority,
    authority_tick: Option<ClockTick>,
    wake_id: Option<TemporalWakeId>,
    wake_ordinal: Option<WakeOrdinal>,
}

impl ReadyTemporalEligibility {
    pub(crate) fn runtime_clock_backed(
        condition: TemporalCondition,
        authority_tick: ClockTick,
    ) -> Self {
        Self {
            clock_domain: condition.clock_domain(),
            condition,
            authority: TemporalEligibilityAuthority::RuntimeClockBasis,
            authority_tick: Some(authority_tick),
            wake_id: None,
            wake_ordinal: None,
        }
    }

    pub(crate) fn runtime_wake_backed(
        condition: TemporalCondition,
        wake_id: TemporalWakeId,
        wake_ordinal: WakeOrdinal,
        authority_tick: ClockTick,
    ) -> Self {
        Self {
            clock_domain: condition.clock_domain(),
            condition,
            authority: TemporalEligibilityAuthority::RuntimeScheduledWake,
            authority_tick: Some(authority_tick),
            wake_id: Some(wake_id),
            wake_ordinal: Some(wake_ordinal),
        }
    }

    pub fn condition(&self) -> &TemporalCondition {
        &self.condition
    }

    pub fn clock_domain(&self) -> ClockDomain {
        self.clock_domain
    }

    pub fn authority(&self) -> TemporalEligibilityAuthority {
        self.authority
    }

    pub fn authority_tick(&self) -> Option<ClockTick> {
        self.authority_tick
    }

    pub fn wake_id(&self) -> Option<TemporalWakeId> {
        self.wake_id
    }

    pub fn wake_ordinal(&self) -> Option<WakeOrdinal> {
        self.wake_ordinal
    }
}

/// Proof that a temporal condition was lowered and denied for this pass.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeferredTemporalEligibility {
    condition: TemporalCondition,
    clock_domain: ClockDomain,
    authority: TemporalEligibilityAuthority,
    authority_tick: Option<ClockTick>,
    wake_id: Option<TemporalWakeId>,
    wake_ordinal: Option<WakeOrdinal>,
}

impl DeferredTemporalEligibility {
    pub(crate) fn runtime_clock_backed(
        condition: TemporalCondition,
        authority_tick: ClockTick,
    ) -> Self {
        Self {
            clock_domain: condition.clock_domain(),
            condition,
            authority: TemporalEligibilityAuthority::RuntimeClockBasis,
            authority_tick: Some(authority_tick),
            wake_id: None,
            wake_ordinal: None,
        }
    }

    pub(crate) fn runtime_wake_deferred(
        condition: TemporalCondition,
        authority_tick: ClockTick,
    ) -> Self {
        Self {
            clock_domain: condition.clock_domain(),
            condition,
            authority: TemporalEligibilityAuthority::RuntimeScheduledWake,
            authority_tick: Some(authority_tick),
            wake_id: None,
            wake_ordinal: None,
        }
    }

    pub fn condition(&self) -> &TemporalCondition {
        &self.condition
    }

    pub fn clock_domain(&self) -> ClockDomain {
        self.clock_domain
    }

    pub fn authority(&self) -> TemporalEligibilityAuthority {
        self.authority
    }

    pub fn authority_tick(&self) -> Option<ClockTick> {
        self.authority_tick
    }

    pub fn wake_id(&self) -> Option<TemporalWakeId> {
        self.wake_id
    }

    pub fn wake_ordinal(&self) -> Option<WakeOrdinal> {
        self.wake_ordinal
    }
}
