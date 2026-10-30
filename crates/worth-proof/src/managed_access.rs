//! Installed coverage legality is distinct from READ and realized-footprint narrowing.
mod coverage;
pub use coverage::{
    AdmittedManagedIndexCoverage, BorrowedManagedCoverageFrame, ManagedCoverageDisposition,
    ManagedCoverageFrontier, ManagedCoverageRegion, ManagedCoverageStep,
    ManagedIndexCoverageAuthority, ManagedIndexCoverageDenial, ManagedIndexCoverageIssuer,
};
