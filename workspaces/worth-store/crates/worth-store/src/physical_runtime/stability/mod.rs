//! Live Store root protection. Copied root observations confer no read authority.

mod admission;
mod byte_guard;
mod lifecycle;
mod read_plan;
mod retention;

pub use admission::{PhysicalReadProtectionDenial, PhysicalReadProtectionPolicy};
pub use byte_guard::*;
pub use lifecycle::{PhysicalReadProtectionDisposition, PhysicalReadProtectionShutdown};
pub use read_plan::PhysicalProtectedRootObservation;
pub use retention::{PhysicalReadProtectionObservation, PhysicalReadProtectionObserver};

pub(in crate::physical_runtime) use lifecycle::PhysicalReadProtectionOwner;
pub(in crate::physical_runtime) use retention::{PhysicalRootReadLease, RootProtectionRegistry};
