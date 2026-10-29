//! Certification that a branch's decision reads stay rooted at that branch.
//!
//! One relational runtime holds every branch, and commit versions are shared
//! across them. A fork that commits after its parent therefore reads at a
//! version newer than the parent's later writes, and a version-only read would
//! observe records the fork never had.

pub use worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome;
pub use worth_query_host::facade::product::WorthQueryProductBranch;

pub use super::document_retention_model::{
    host::{publish_workflow_on_first_program, DocumentWorkflowRuntime, SEED_RETENTION},
    presented_request::set_retention,
    settled_verdict::{settle, RetentionVerdict},
    workflow::{link_review_requirement_on, unlink_review_requirement_on, ReviewRequirementDenial},
};

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

/// Commits an unrelated edit so the branch reads at a version newer than every
/// write its sibling has made.
pub fn advance_past_sibling(
    application: &DocumentWorkflowRuntime,
    branch: WorthQueryProductBranch,
    idempotency: u64,
) {
    assert_eq!(
        settle(set_retention(
            application.program_runtime(),
            branch,
            SEED_RETENTION + 1,
            idempotency,
        )),
        RetentionVerdict::Performed(SEED_RETENTION + 1),
    );
}
