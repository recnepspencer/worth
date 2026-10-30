//! A fork copies an instance's history, never its execution. Work continues
//! on the fork only as an explicitly admitted successor under the migration
//! law, and the instance on its own branch is untouched.

pub use worth_query_host::facade::primary_graph::WorthQueryWorkflowInstanceDisposition;
pub use worth_query_host::facade::product::WorthQueryProductBranch;

pub use super::super::document_retention_model::{
    programs::RetentionProgramP1,
    workflow::{continue_on_fork, support_workflow_program},
};
pub use super::instance_migration::{
    migration_denial, perform_approved_effect, started, supersede,
};
pub use super::journey::approval_requirement;
pub use super::*;

pub fn fork_of(
    application: &DocumentWorkflowRuntime,
    branch: WorthQueryProductBranch,
) -> WorthQueryProductBranch {
    application
        .with_host_advancement(|host_phase| {
            application
                .runtime()
                .branches()
                .fork(branch)
                .components(|components| components.fork_relational().reuse_exact_signal_basis())
                .create(&host_phase)
                .expect("the fork publishes")
        })
        .expect("installed host call opens")
}
