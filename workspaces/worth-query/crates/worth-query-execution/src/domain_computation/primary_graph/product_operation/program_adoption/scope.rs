//! Branch-local and explicit branch-set adoption scope.

mod branch_set;
mod coverage;
mod progress;
mod single_branch;

pub use branch_set::{
    WorthQueryBranchSetAdoptionAdvanceDenial, WorthQueryBranchSetAdoptionCancellation,
    WorthQueryBranchSetAdoptionCloseDenial, WorthQueryBranchSetAdoptionPreparationDenial,
    WorthQueryBranchSetAdoptionRecovery, WorthQueryBranchSetAdoptionRecoveryFailure,
    WorthQueryBranchSetAdoptionRecoveryOutcome, WorthQueryBranchSetAdoptionRecoveryReleaseFailure,
    WorthQueryBranchSetAdoptionResumeDenial, WorthQueryBranchSetAdoptionResumeFailure,
    WorthQueryClosedBranchSetAdoption, WorthQueryPreparedBranchSetAdoption,
    WorthQueryStoppedBranchSetAdoption,
};
pub use coverage::{
    WorthQueryOrderedProgramAdoptionCoverage, WorthQueryProgramAdoptionCoverage,
    WorthQueryProgramAdoptionCoverageDenial,
};
pub use progress::WorthQueryBranchSetAdoptionProgress;
