//! Portable clock evidence and boot-local elapsed time.

mod attested;
mod identity;
mod monotonic;

pub use attested::{AttestedTime, TimeUnavailable};
pub use identity::{BootEpoch, ClockSourceIdentity, TimeSourceEpoch};
pub use monotonic::{BootEpochMismatch, MonotonicElapsedError, MonotonicInstant};
