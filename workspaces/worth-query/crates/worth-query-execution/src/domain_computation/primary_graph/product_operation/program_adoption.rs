//! Branch-local application-program adoption.
//!
//! Preparation reads one exact product occurrence, computes the installed
//! source-to-target requirements, selects the bounded existing-state scope,
//! and reserves one Relational-only World publication. Publication accepts
//! only that move-only product; a revision by itself carries no effect power.
//!
//! The branch activation gate is held through candidate preparation, then the
//! prepared World's exact product-head fence guards move-only publication.
//! Preparation derives migration and custody dispositions from installed
//! source/target truth; callers can inspect those decisions but cannot author
//! them. Managed same-intent replay belongs to the broader-scope progression,
//! while single-branch requirements remain fail-closed and freshness-bound.

mod custody;
mod preparation;
mod publication;
mod scope;

pub use custody::{
    WorthQueryBranchAdoptionRecovery, WorthQueryBranchAdoptionRecoveryDenial,
    WorthQueryBranchAdoptionRecoveryFailure, WorthQueryBranchAdoptionRecoveryOutcome,
};
pub use preparation::{
    WorthQueryAdmittedProgramMigration, WorthQueryBranchAdoptionActivationDenial,
    WorthQueryBranchAdoptionPreparationDenial, WorthQueryPreparedBranchAdoption,
    WorthQueryPreparedProgramMigration, WorthQueryProgramCustodyDisposition,
    WorthQueryProgramCustodyDispositionInventory, WorthQueryProgramCustodyDispositionKind,
    WorthQueryProgramMigrationDescription, WorthQueryProgramMigrationPreparationDenial,
};
pub use publication::{
    WorthQueryBranchAdoptionPublicationOutcome, WorthQueryPerformedBranchAdoption,
    WorthQueryUnpublishedBranchAdoption,
};
pub use scope::{
    WorthQueryBranchSetAdoptionAdvanceDenial, WorthQueryBranchSetAdoptionCancellation,
    WorthQueryBranchSetAdoptionCloseDenial, WorthQueryBranchSetAdoptionPreparationDenial,
    WorthQueryBranchSetAdoptionProgress, WorthQueryBranchSetAdoptionRecovery,
    WorthQueryBranchSetAdoptionRecoveryFailure, WorthQueryBranchSetAdoptionRecoveryOutcome,
    WorthQueryBranchSetAdoptionRecoveryReleaseFailure, WorthQueryBranchSetAdoptionResumeDenial,
    WorthQueryBranchSetAdoptionResumeFailure, WorthQueryClosedBranchSetAdoption,
    WorthQueryOrderedProgramAdoptionCoverage, WorthQueryPreparedBranchSetAdoption,
    WorthQueryProgramAdoptionCoverage, WorthQueryProgramAdoptionCoverageDenial,
    WorthQueryStoppedBranchSetAdoption,
};
