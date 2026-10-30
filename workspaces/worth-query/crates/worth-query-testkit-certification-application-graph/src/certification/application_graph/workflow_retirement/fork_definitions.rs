//! A fork holds the definitions it copied under their own identity. Named on
//! the fork, a copied definition governs the fork's new starts, successors
//! and retirement like one the fork published, and none of it reaches the
//! branch it was copied from.

pub use worth_query_host::facade::{
    application_entry::{WorkflowDefinitionPreparationDenial, WorkflowInstancePreparationDenial},
    primary_graph::WorthQueryApplicationAttemptDenialKind,
    product::WorthQueryProductBranch,
};

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
