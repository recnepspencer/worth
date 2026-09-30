//! Bounded computation authority shared by WORTH runtimes.
//!
//! Only the composition root constructs an authority. Domain runtimes receive
//! leases and use checked patterns; they do not control the physical backend.

mod authority;
mod backend;
mod oracle;
mod pattern;
mod report;

pub use authority::{
    CancellationToken, ConstructionDenial, ExecutionAuthority, ExecutionAuthorityConfig,
    ExecutionResourceLease, LeaseDenial, LeaseRequest,
};
pub use oracle::CanonicalBits;

#[cfg(test)]
mod tests;
pub use pattern::{
    ExecutionMap, MapDenial, MapKernelContext, MapKernelFailure, MapKernelStop, MapOutcome,
    MapPartition, MapStop, OracleMismatch,
};
pub use report::ChargedBytes;
