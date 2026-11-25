use std::marker::PhantomData;
use std::panic::AssertUnwindSafe;
use std::sync::{Arc, Mutex};

use worth_query_installation::facade::{
    WorthQueryClockCoordinate, WorthQueryClockSourceIdentity, WorthQueryClockTimelineIdentity,
    WorthQueryNamedClock, WorthQueryNamedClockFailure, WorthQueryNamedClockFailureKind,
    WorthQueryNamedClockReading, WorthQueryNamedClockSource,
};

/// Refusal when advancing a caller-pumped [`WorthQueryDesktopClockHandle`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryDesktopClockAdvanceDenial {
    /// Requested coordinate ran behind the already-admitted coordinate.
    ClockRegression { current: u64, requested: u64 },
    /// The clock handle was explicitly closed.
    ClockClosed,
}

impl std::fmt::Display for WorthQueryDesktopClockAdvanceDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ClockRegression { current, requested } => write!(
                formatter,
                "desktop clock cannot regress from {current}ns to {requested}ns"
            ),
            Self::ClockClosed => formatter.write_str("desktop clock handle is closed"),
        }
    }
}

impl std::error::Error for WorthQueryDesktopClockAdvanceDenial {}

struct DesktopClockState {
    current_nanoseconds: u64,
    sequence: u64,
    closed: bool,
}

/// Caller-owned handle for advancing a [`WorthQueryDesktopClockSource`].
///
/// Spawns no background threads and reads no ambient OS clocks. Callers supply
/// fresh monotonic readings explicitly during reopen catch-up or foreground ticks.
pub struct WorthQueryDesktopClockHandle<Clock> {
    state: AssertUnwindSafe<Arc<Mutex<DesktopClockState>>>,
    marker: PhantomData<fn() -> Clock>,
}

impl<Clock> Clone for WorthQueryDesktopClockHandle<Clock> {
    fn clone(&self) -> Self {
        Self {
            state: AssertUnwindSafe(Arc::clone(&self.state.0)),
            marker: PhantomData,
        }
    }
}

impl<Clock: WorthQueryNamedClock> WorthQueryDesktopClockHandle<Clock> {
    /// Advances the clock's target coordinate monotonically (`requested >= current`).
    pub fn advance_to(
        &self,
        coordinate: WorthQueryClockCoordinate<Clock>,
    ) -> Result<(), WorthQueryDesktopClockAdvanceDenial> {
        let mut state = self
            .state
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.closed {
            return Err(WorthQueryDesktopClockAdvanceDenial::ClockClosed);
        }
        let requested = coordinate.nanoseconds();
        if requested < state.current_nanoseconds {
            return Err(WorthQueryDesktopClockAdvanceDenial::ClockRegression {
                current: state.current_nanoseconds,
                requested,
            });
        }
        state.current_nanoseconds = requested;
        Ok(())
    }

    /// Returns the most recently admitted monotonic coordinate.
    pub fn current_time(&self) -> WorthQueryClockCoordinate<Clock> {
        let state = self
            .state
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        WorthQueryClockCoordinate::from_nanoseconds(state.current_nanoseconds)
    }

    /// Returns the last observation sequence issued to Query.
    pub fn last_sequence(&self) -> u64 {
        self.state
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .sequence
    }

    /// Marks the clock source closed so subsequent observations report `Closed`.
    pub fn close(&self) {
        self.state
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .closed = true;
    }

    /// Reports whether the clock handle has been closed.
    pub fn is_closed(&self) -> bool {
        self.state
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .closed
    }

    pub(super) fn shares_state_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.state.0, &other.state.0)
    }
}

/// Caller-pumped monotonic [`WorthQueryNamedClockSource`] with zero background
/// threads and zero hidden wall-clock reads.
pub struct WorthQueryDesktopClockSource<Clock> {
    source_identity: WorthQueryClockSourceIdentity,
    timeline_identity: WorthQueryClockTimelineIdentity,
    state: AssertUnwindSafe<Arc<Mutex<DesktopClockState>>>,
    marker: PhantomData<fn() -> Clock>,
}

impl<Clock: WorthQueryNamedClock> WorthQueryDesktopClockSource<Clock> {
    /// Creates a paired clock source and caller-owned advance handle starting at
    /// `initial_time`.
    pub fn new(
        source_identity: WorthQueryClockSourceIdentity,
        timeline_identity: WorthQueryClockTimelineIdentity,
        initial_time: WorthQueryClockCoordinate<Clock>,
    ) -> (Self, WorthQueryDesktopClockHandle<Clock>) {
        let state = Arc::new(Mutex::new(DesktopClockState {
            current_nanoseconds: initial_time.nanoseconds(),
            sequence: 0,
            closed: false,
        }));
        let handle = WorthQueryDesktopClockHandle {
            state: AssertUnwindSafe(Arc::clone(&state)),
            marker: PhantomData,
        };
        let source = Self {
            source_identity,
            timeline_identity,
            state: AssertUnwindSafe(state),
            marker: PhantomData,
        };
        (source, handle)
    }

    /// Derives a handle bound to this exact clock source's shared state.
    pub fn handle(&self) -> WorthQueryDesktopClockHandle<Clock> {
        WorthQueryDesktopClockHandle {
            state: AssertUnwindSafe(Arc::clone(&self.state.0)),
            marker: PhantomData,
        }
    }
}

impl<Clock: WorthQueryNamedClock> WorthQueryNamedClockSource<Clock>
    for WorthQueryDesktopClockSource<Clock>
{
    const SEMANTIC_IDENTITY: &'static str = "worth.query.host.desktop-clock-source.v1";

    fn source_identity(&self) -> WorthQueryClockSourceIdentity {
        self.source_identity.clone()
    }

    fn timeline_identity(&self) -> WorthQueryClockTimelineIdentity {
        self.timeline_identity.clone()
    }

    fn observe(&self) -> Result<WorthQueryNamedClockReading<Clock>, WorthQueryNamedClockFailure> {
        let mut state = self
            .state
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.closed {
            return Err(WorthQueryNamedClockFailure::new(
                WorthQueryNamedClockFailureKind::SourceClosed,
                "desktop clock source was closed",
            ));
        }
        let next_sequence = state.sequence.checked_add(1).ok_or_else(|| {
            WorthQueryNamedClockFailure::new(
                WorthQueryNamedClockFailureKind::ObservationFailed,
                "desktop clock observation sequence exhausted",
            )
        })?;
        state.sequence = next_sequence;
        Ok(WorthQueryNamedClockReading::new(
            next_sequence,
            WorthQueryClockCoordinate::from_nanoseconds(state.current_nanoseconds),
        ))
    }
}
