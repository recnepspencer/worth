pub mod prepared;
// The managed provider progression phases live beneath `prepared::running::progression`,
// whose private state they consume. This parent exposes only the opaque permits needed by
// the surrounding application-attempt owner.

pub use prepared::running::progression::progress_application_commit;
pub use prepared::running::progression::progressed::finish_application_commit;
pub use prepared::running::progression::WorthQueryFreshCommitReceiptPermit;
pub use prepared::running::progression::{
    WorthQueryProviderAttemptRegistrationContext, WorthQueryRegisteredProviderAttempt,
};

pub use prepared::ReadmittedCommitProgram;
pub use prepared::WorthQueryManagedPhaseStorageForTest;
#[cfg(test)]
#[path = "mod_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;

pub use prepared::{
    inspect_authored_application, inspect_coupled_application, prepare_coupled_application,
    register_coupled_application, ApprovedCoupledWriter, CommitPreparedCoupledWriter,
    InspectedAuthoredApplication, InspectedCoupledWriter,
};
