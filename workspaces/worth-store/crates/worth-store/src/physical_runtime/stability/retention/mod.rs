mod lease;
mod observation;
mod registrations;

pub(in crate::physical_runtime) use lease::PhysicalRootReadLease;
pub use observation::{PhysicalReadProtectionObservation, PhysicalReadProtectionObserver};
pub(in crate::physical_runtime) use registrations::RootProtectionRegistry;
