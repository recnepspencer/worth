//! A live branch keeps only the history something still holds.
//!
//! After each settled publication the product retires every commit nothing
//! holds: not a retained read, a history traversal, the caller's own commit
//! receipt, or the basis the settling request itself still holds, which keeps
//! the head's parent until the next publication. A held commit keeps only
//! itself; the commits between it and the head retire around it. These proofs
//! read that window through the ordinary application entry: a held read or a
//! held receipt keeps its commit selectable across many publications,
//! releasing it lets that history go, and inspection of a retired commit fails
//! closed. Replay reads the original owner artifacts, which outlive
//! its history and holds none of it, so a retired commit's replay still
//! returns recorded meaning and never runs the request again.

pub use worth_query_host::facade::application_entry::{
    WorthQueryApplicationHistorySelectionDenial, WorthQueryApplicationMutationOutcome,
    WorthQueryApplicationReadObservation, WorthQueryApplicationRequestExt,
};
pub use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationCommitReceipt, WorthQueryPrimaryGraphApplicationRuntime,
    WorthQueryRecoveredApplicationCommitReceipt,
};
pub use worth_query_host::facade::product::WorthQueryProductBranch;

pub use super::document_retention_model::host::{
    publish_on_first_program, DocumentRetentionRuntime,
};
pub use super::document_retention_model::operator_identity::{
    authenticate_operator, request_scope,
};
pub use super::document_retention_model::presented_request::set_retention;
pub use super::document_retention_model::programs::RetentionProgramP0;
pub use super::document_retention_model::readback::{observe_head, read_retention};
pub use super::document_retention_model::retention_entry::{
    DocumentRetentionRead, DOCUMENT_IDENTITY,
};
pub use super::document_retention_model::schema::DocumentRetentionSchema;

pub type Runtime = WorthQueryPrimaryGraphApplicationRuntime<DocumentRetentionSchema>;

/// The settled head and the parent the settling request's basis still held.
pub const SETTLED_WINDOW: usize = 2;

pub fn commit(
    host: &DocumentRetentionRuntime<RetentionProgramP0>,
    branch: WorthQueryProductBranch,
    retention: u64,
    key: u64,
) -> WorthQueryApplicationCommitReceipt {
    match set_retention(host, branch, retention, key).expect("the request settles") {
        WorthQueryApplicationMutationOutcome::Committed { receipt, .. } => receipt,
        other => panic!("a fresh key commits: {other:?}"),
    }
}

/// How many commits the branch's history holds from its head, and whether
/// that history still reaches the branch's first commit.
pub fn retained_history(runtime: &Runtime, branch: WorthQueryProductBranch) -> (usize, bool) {
    let history = runtime
        .with_host_advancement(|phase| runtime.branches().history(&phase, branch, None))
        .expect("the installed host supplies fresh history authority")
        .expect("a live branch exposes its history");
    (history.visited_count(), history.is_complete())
}

pub fn read_at(
    runtime: &Runtime,
    branch: WorthQueryProductBranch,
    held: &WorthQueryApplicationReadObservation,
) -> u64 {
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    runtime
        .request(&principal, &scope)
        .on_branch(branch)
        .at(held)
        .query(DocumentRetentionRead {
            identity: DOCUMENT_IDENTITY.to_owned(),
        })
        .execute()
        .expect("a held read executes at its own commit")
        .rows()[0]
        .retention_days
}

pub fn read_at_commit(
    runtime: &Runtime,
    branch: WorthQueryProductBranch,
    receipt: &WorthQueryApplicationCommitReceipt,
) -> u64 {
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    runtime
        .with_host_advancement(|host_phase| {
            runtime
                .request(&principal, &scope)
                .on_branch(branch)
                .at_commit(&host_phase, receipt, None)
                .expect("a held receipt keeps its commit selectable")
                .query(DocumentRetentionRead {
                    identity: DOCUMENT_IDENTITY.to_owned(),
                })
                .execute_in_advancement(&host_phase)
                .expect("a selected commit executes its read")
                .rows()[0]
                .retention_days
        })
        .expect("installed host call opens")
}

/// Observe the real retained World history without turning recorded data into a selector.
pub fn recorded_is_retained(
    runtime: &Runtime,
    branch: WorthQueryProductBranch,
    receipt: &WorthQueryRecoveredApplicationCommitReceipt,
) -> bool {
    let head = receipt
        .new_product_head()
        .memory()
        .expect("explicit memory World fixture");
    let history = runtime
        .with_host_advancement(|phase| runtime.branches().history(&phase, branch, None))
        .expect("fresh installed history authority")
        .expect("the live branch exposes actual history");
    history.entries().any(|entry| {
        let commit = entry.selected_commit();
        assert_eq!(commit.owner_identity().get(), head.owner());
        commit.ordinal() == head.selected_commit()
    })
}

pub fn replay(
    host: &DocumentRetentionRuntime<RetentionProgramP0>,
    branch: WorthQueryProductBranch,
    retention: u64,
    key: u64,
) -> WorthQueryRecoveredApplicationCommitReceipt {
    match set_retention(host, branch, retention, key).expect("the replay settles") {
        WorthQueryApplicationMutationOutcome::RecoveredCommitted(receipt) => receipt,
        other => panic!("a committed key answers its replay: {other:?}"),
    }
}
