pub(crate) mod prepared;
// The managed provider progression phases live beneath `prepared::running::progression`,
// whose private state they consume. This parent exposes only the opaque permits needed by
// the surrounding application-attempt owner.

pub(in crate::domain_computation::primary_graph::application_attempt) use prepared::running::progression::WorthQueryFreshCommitReceiptPermit;
pub(in crate::domain_computation) use prepared::running::progression::{
    WorthQueryProviderAttemptRegistrationContext, WorthQueryRegisteredProviderAttempt,
};
pub(super) use prepared::running::progression::{
    progress_application_commit,
};
pub(super) use prepared::running::progression::progressed::finish_application_commit;

pub(in crate::domain_computation::primary_graph) use prepared::ReadmittedCommitProgram;
pub use prepared::WorthQueryManagedPhaseStorageForTest;
#[cfg(test)]
#[path = "mod_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;

pub(in crate::domain_computation::primary_graph) use prepared::{
    inspect_coupled_application, prepare_coupled_application, register_coupled_application,
    ApprovedCoupledWriter, CommitPreparedCoupledWriter, InspectedCoupledWriter,
};
