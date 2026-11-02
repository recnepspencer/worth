//! Installed coverage legality is distinct from READ and realized-footprint narrowing.
mod coverage;
mod realized_footprint;
pub use coverage::{
    AdmittedManagedIndexCoverage, BorrowedManagedCoverageFrame, ManagedCoverageDisposition,
    ManagedCoverageFrontier, ManagedCoverageRegion, ManagedCoverageStep,
    ManagedIndexCoverageAuthority, ManagedIndexCoverageDenial, ManagedIndexCoverageIssuer,
};
pub use realized_footprint::{
    BorrowedRealizedFootprintFrame, CapturedRealizedFootprint, RealizedFootprintAuthority,
    RealizedFootprintDenial, RealizedFootprintIssuer, VerifiedRealizedFootprint,
};
