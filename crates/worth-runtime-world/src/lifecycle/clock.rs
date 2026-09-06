use std::sync::Arc;

/// Monotonic value supplied by the explicit Runtime World clock port.
///
/// The value is meaningful only for deadline and cleanup eligibility. It is
/// deliberately absent from identities, bases, parentage, and outcomes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeWorldInstant(u64);

impl RuntimeWorldInstant {
    pub const fn from_ticks(ticks: u64) -> Self {
        Self(ticks)
    }

    pub const fn ticks(self) -> u64 {
        self.0
    }
}

/// Source for the purpose-specific monotonic clock. A clock is an allowed
/// runtime dependency, not a component-owner adapter. Implementations must be
/// monotonic, bounded, nonblocking, and must not re-enter Runtime World. The
/// final deadline check runs under the exact product-cell guard before movement;
/// no clock callback runs after that irreversible cutoff.
pub trait RuntimeWorldClockSource: Send + Sync {
    fn now(&self) -> RuntimeWorldInstant;
}

/// Cloneable clock port held by the future managed Runtime World owner.
#[derive(Clone)]
pub struct RuntimeWorldClock {
    source: Arc<dyn RuntimeWorldClockSource>,
}

impl std::fmt::Debug for RuntimeWorldClock {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RuntimeWorldClock")
            .finish_non_exhaustive()
    }
}

impl RuntimeWorldClock {
    pub fn from_source<S>(source: S) -> Self
    where
        S: RuntimeWorldClockSource + 'static,
    {
        Self {
            source: Arc::new(source),
        }
    }

    pub fn now(&self) -> RuntimeWorldInstant {
        self.source.now()
    }
}
