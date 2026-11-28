use serde::{Deserialize, Serialize};
use worth_foundational::time::{MonotonicElapsedError, MonotonicInstant};

use crate::data::error::SignalError;

/// Runtime-understood clock domains. Only monotonic execution time may decide
/// replay-critical eligibility in Milestone A.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClockDomain {
    MonotonicExecution,
    WallClock,
    Presentation,
}

/// Whether a clock domain is allowed to drive authoritative temporal truth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClockAuthority {
    Authoritative,
    MetadataOnly,
}

impl ClockAuthority {
    pub fn is_authoritative(self) -> bool {
        matches!(self, Self::Authoritative)
    }
}

impl ClockDomain {
    pub fn authority(self) -> ClockAuthority {
        match self {
            Self::MonotonicExecution => ClockAuthority::Authoritative,
            Self::WallClock | Self::Presentation => ClockAuthority::MetadataOnly,
        }
    }
}

/// Canonical runtime tick used for replay-critical temporal eligibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub struct ClockTick(u64);

impl ClockTick {
    pub const ZERO: Self = Self(0);

    pub fn new(value: u64) -> Self {
        Self(value)
    }

    /// Converts live time to an explicit replay coordinate relative to the
    /// installed origin. Samples from another boot cannot become a tick.
    pub fn from_monotonic_elapsed(
        origin: MonotonicInstant,
        observed: MonotonicInstant,
    ) -> Result<Self, MonotonicElapsedError> {
        observed
            .checked_duration_since(origin)
            .map(|elapsed| Self(elapsed.as_nanos() as u64))
    }

    pub fn get(self) -> u64 {
        self.0
    }
}

/// Monotonic ordinal for authoritative clock advances accepted by the runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub struct ClockAdvanceOrdinal(u64);

impl ClockAdvanceOrdinal {
    pub const ZERO: Self = Self(0);

    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn get(self) -> u64 {
        self.0
    }

    pub(crate) fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }
}

/// Stable identifier for a future checkpointed clock basis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub struct ClockCheckpointId(u64);

impl ClockCheckpointId {
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn get(self) -> u64 {
        self.0
    }
}

/// Host-submitted request to advance a declared clock domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClockAdvanceRequest {
    domain: ClockDomain,
    target_tick: ClockTick,
}

impl ClockAdvanceRequest {
    pub fn new(domain: ClockDomain, target_tick: ClockTick) -> Self {
        Self {
            domain,
            target_tick,
        }
    }

    pub fn domain(self) -> ClockDomain {
        self.domain
    }

    pub fn target_tick(self) -> ClockTick {
        self.target_tick
    }
}

/// Canonical authoritative clock basis carried by the runtime and branches.
/// A live clock origin is held by its ingress owner. The basis records only
/// the admitted replay coordinates and remains comparable across recovery.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeClockBasis {
    domain: ClockDomain,
    current_tick: ClockTick,
    last_advance_ordinal: ClockAdvanceOrdinal,
    last_checkpoint_id: Option<ClockCheckpointId>,
}

impl Default for RuntimeClockBasis {
    fn default() -> Self {
        Self {
            domain: ClockDomain::MonotonicExecution,
            current_tick: ClockTick::ZERO,
            last_advance_ordinal: ClockAdvanceOrdinal::ZERO,
            last_checkpoint_id: None,
        }
    }
}

impl RuntimeClockBasis {
    pub fn domain(self) -> ClockDomain {
        self.domain
    }

    pub fn current_tick(self) -> ClockTick {
        self.current_tick
    }

    pub fn last_advance_ordinal(self) -> ClockAdvanceOrdinal {
        self.last_advance_ordinal
    }

    pub fn last_checkpoint_id(self) -> Option<ClockCheckpointId> {
        self.last_checkpoint_id
    }

    pub fn validate_advance(
        self,
        request: ClockAdvanceRequest,
    ) -> Result<ValidatedClockAdvance, SignalError> {
        if request.domain().authority() != ClockAuthority::Authoritative {
            return Err(SignalError::invalid_input(format!(
                "{:?} is metadata-only and cannot drive authoritative temporal eligibility",
                request.domain()
            )));
        }
        if request.domain() != self.domain {
            return Err(SignalError::invalid_input(format!(
                "authoritative clock domain mismatch: runtime basis is {:?}, request targeted {:?}",
                self.domain,
                request.domain()
            )));
        }
        if request.target_tick() < self.current_tick {
            return Err(SignalError::invalid_input(format!(
                "clock regression is not allowed: current tick is {}, requested {}",
                self.current_tick.get(),
                request.target_tick().get()
            )));
        }

        Ok(ValidatedClockAdvance {
            domain: self.domain,
            previous_tick: self.current_tick,
            next_tick: request.target_tick(),
            ordinal: self.last_advance_ordinal.next(),
        })
    }

    pub(crate) fn apply_validated_advance(&mut self, validated: ValidatedClockAdvance) {
        self.domain = validated.domain;
        self.current_tick = validated.next_tick;
        self.last_advance_ordinal = validated.ordinal;
    }
}

/// Proof-bearing authoritative clock advance accepted by the runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidatedClockAdvance {
    domain: ClockDomain,
    previous_tick: ClockTick,
    next_tick: ClockTick,
    ordinal: ClockAdvanceOrdinal,
}

impl ValidatedClockAdvance {
    pub fn domain(self) -> ClockDomain {
        self.domain
    }

    pub fn previous_tick(self) -> ClockTick {
        self.previous_tick
    }

    pub fn next_tick(self) -> ClockTick {
        self.next_tick
    }

    pub fn ordinal(self) -> ClockAdvanceOrdinal {
        self.ordinal
    }
}
