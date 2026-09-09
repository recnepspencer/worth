use std::marker::PhantomData;

type ClockMarker<Clock> = fn() -> Clock;

/// Type-level identity of one host-managed clock contract.
pub trait WorthQueryNamedClock: Send + Sync + 'static {
    const PORTABLE_IDENTITY: &'static str;
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryClockSourceIdentity(String);

impl WorthQueryClockSourceIdentity {
    pub fn declare(identity: impl Into<String>) -> Result<Self, &'static str> {
        validated_clock_identity(identity).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryClockTimelineIdentity(String);

impl WorthQueryClockTimelineIdentity {
    pub fn declare(identity: impl Into<String>) -> Result<Self, &'static str> {
        validated_clock_identity(identity).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Typed coordinate on one named clock's timeline.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryClockCoordinate<Clock> {
    nanoseconds: u64,
    marker: PhantomData<ClockMarker<Clock>>,
}

impl<Clock> WorthQueryClockCoordinate<Clock> {
    pub const fn from_nanoseconds(nanoseconds: u64) -> Self {
        Self {
            nanoseconds,
            marker: PhantomData,
        }
    }

    pub const fn nanoseconds(&self) -> u64 {
        self.nanoseconds
    }
}

/// One atomic reading returned by an admitted named clock source.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryNamedClockReading<Clock> {
    sequence: u64,
    observed_time: WorthQueryClockCoordinate<Clock>,
}

impl<Clock> WorthQueryNamedClockReading<Clock> {
    pub const fn new(sequence: u64, observed_time: WorthQueryClockCoordinate<Clock>) -> Self {
        Self {
            sequence,
            observed_time,
        }
    }

    pub const fn sequence(&self) -> u64 {
        self.sequence
    }

    pub const fn observed_time(&self) -> WorthQueryClockCoordinate<Clock> {
        WorthQueryClockCoordinate::from_nanoseconds(self.observed_time.nanoseconds)
    }
}

/// Runtime-bound observation evidence after Query joins a reading to its
/// admitted immutable source and timeline identities.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryNamedClockObservation<Clock> {
    source: WorthQueryClockSourceIdentity,
    timeline: WorthQueryClockTimelineIdentity,
    sequence: u64,
    observed_time: WorthQueryClockCoordinate<Clock>,
}

impl<Clock> WorthQueryNamedClockObservation<Clock> {
    #[doc(hidden)]
    pub fn from_admitted_source(
        source: WorthQueryClockSourceIdentity,
        timeline: WorthQueryClockTimelineIdentity,
        reading: WorthQueryNamedClockReading<Clock>,
    ) -> Self {
        Self {
            source,
            timeline,
            sequence: reading.sequence(),
            observed_time: reading.observed_time(),
        }
    }

    pub fn source(&self) -> &WorthQueryClockSourceIdentity {
        &self.source
    }

    pub fn timeline(&self) -> &WorthQueryClockTimelineIdentity {
        &self.timeline
    }

    pub fn sequence(&self) -> u64 {
        self.sequence
    }

    pub fn observed_time(&self) -> WorthQueryClockCoordinate<Clock> {
        WorthQueryClockCoordinate::from_nanoseconds(self.observed_time.nanoseconds)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryNamedClockFailureKind {
    SourceUnavailable,
    ObservationFailed,
    SourceClosed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryNamedClockFailure {
    kind: WorthQueryNamedClockFailureKind,
    detail: String,
}

impl WorthQueryNamedClockFailure {
    pub fn new(kind: WorthQueryNamedClockFailureKind, detail: impl Into<String>) -> Self {
        Self {
            kind,
            detail: detail.into(),
        }
    }

    pub fn kind(&self) -> WorthQueryNamedClockFailureKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }
}

/// Host clock source installed for one exact typed temporal node.
///
/// Query never dedicates a watcher thread and never asks for an OS hook. The
/// host calls the runtime observation port from its existing loop, task,
/// poller, or platform-timer callback; that port obtains the next reading from
/// this installed source and submits it to Query's Bridge-owned Signal runtime.
/// Shared callers may observe sibling product branches independently. A source
/// owns any synchronization needed for its reading, never Query operation state.
pub trait WorthQueryNamedClockSource<Clock: WorthQueryNamedClock>: Send + Sync + 'static {
    const SEMANTIC_IDENTITY: &'static str;

    fn source_identity(&self) -> WorthQueryClockSourceIdentity;

    fn timeline_identity(&self) -> WorthQueryClockTimelineIdentity;

    fn observe(&self) -> Result<WorthQueryNamedClockReading<Clock>, WorthQueryNamedClockFailure>;
}

fn validated_clock_identity(identity: impl Into<String>) -> Result<String, &'static str> {
    let identity = identity.into();
    if identity.is_empty()
        || identity.trim() != identity
        || identity.chars().any(char::is_whitespace)
    {
        Err("invalid-named-clock-identity")
    } else {
        Ok(identity)
    }
}
