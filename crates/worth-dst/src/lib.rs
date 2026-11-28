//! Deterministic simulation support for platform courts.

#![forbid(unsafe_code)]

mod runner;
mod seed;
mod spec;
mod trace;

pub use runner::DstRunner;
pub use seed::{Rng, Seed};
pub use spec::{ClockFault, Spec};
pub use trace::Trace;
