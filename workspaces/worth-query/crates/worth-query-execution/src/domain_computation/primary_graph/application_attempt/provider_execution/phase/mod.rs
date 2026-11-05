pub(crate) mod prepared;
// The managed provider progression phases live beneath `prepared::running::progression`,
// whose private state they consume. This parent exposes only the opaque permits needed by
// the surrounding application-attempt owner.

pub(in crate::domain_computation::primary_graph::application_attempt) use prepared::WorthQueryEarlyEquivalentCommitReceiptPermit;
pub(in crate::domain_computation::primary_graph::application_attempt) use prepared::running::progression::{
    WorthQueryFreshCommitReceiptPermit, WorthQueryManagedEquivalentCommitReceiptPermit,
    WorthQueryRegisteredEquivalentCommitReceiptPermit,
};
pub(in crate::domain_computation) use prepared::running::progression::{
    WorthQueryProviderAttemptRegistrationContext, WorthQueryRegisteredProviderAttempt,
};
pub(super) use prepared::running::progression::{
    progress_application_commit,
};
pub(super) use prepared::running::progression::progressed::finish_application_commit;
pub(super) use prepared::running::{
    start_managed_application_commit,
};

pub use prepared::EntryCustodyDoorsOps;
pub use prepared::EntryCustodyEntryCustodyDoorsOps;
pub(in crate::domain_computation) use prepared::EntryCustodyScopedOps;
pub(in crate::domain_computation::primary_graph) use prepared::ReadmittedCommitProgram;
