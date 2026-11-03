//! Host-published trusted time for the Query runtime (R8.7 / PB3).
//!
//! One source serves grant validity, dispatch timeout classification, and
//! recovery expiry. Callers and transport adapters cannot supply a sample.

mod time_basis;
mod time_source;

pub(in crate::domain_computation) use time_basis::WorthQueryRuntimeClock;
pub use time_basis::WorthQueryRuntimeTimeSample;
pub use time_source::{WorthQueryRuntimeTimeSource, WorthQueryRuntimeTimeSourceDenial};
