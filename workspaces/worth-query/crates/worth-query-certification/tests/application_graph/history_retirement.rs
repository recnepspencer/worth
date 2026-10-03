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
//! closed. Replay answers from the commit's completed evidence, which outlives
//! its history and holds none of it, so a retired commit's replay still
//! returns its receipt and never runs the request again.

use std::num::NonZeroUsize;

use worth_query_host::facade::application_entry::{
    WorthQueryApplicationHistorySelectionDenial, WorthQueryApplicationMutationOutcome,
    WorthQueryApplicationReadObservation, WorthQueryApplicationRequestExt,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationCommitReceipt, WorthQueryPrimaryGraphApplicationRuntime,
};
use worth_query_host::facade::product::WorthQueryProductBranch;

use super::document_retention_model::host::{publish_on_first_program, DocumentRetentionRuntime};
use super::document_retention_model::operator_identity::{authenticate_operator, request_scope};
use super::document_retention_model::presented_request::set_retention;
use super::document_retention_model::programs::RetentionProgramP0;
use super::document_retention_model::readback::{observe_head, read_retention};
use super::document_retention_model::retention_entry::{DocumentRetentionRead, DOCUMENT_IDENTITY};
use super::document_retention_model::schema::DocumentRetentionSchema;

type Runtime = WorthQueryPrimaryGraphApplicationRuntime<DocumentRetentionSchema>;

const WORK: NonZeroUsize = NonZeroUsize::new(64).unwrap();

/// The settled head and the parent the settling request's basis still held.
const SETTLED_WINDOW: usize = 2;

fn commit(
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
fn retained_history(runtime: &Runtime, branch: WorthQueryProductBranch) -> (usize, bool) {
    let history = runtime
        .branches()
        .history(branch, WORK)
        .expect("a live branch exposes its history");
    (history.visited_count(), history.is_complete())
}

fn read_at(
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

fn read_at_commit(
    runtime: &Runtime,
    branch: WorthQueryProductBranch,
    receipt: &WorthQueryApplicationCommitReceipt,
) -> u64 {
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    runtime
        .request(&principal, &scope)
        .on_branch(branch)
        .at_commit(receipt, WORK)
        .expect("a held receipt keeps its commit selectable")
        .query(DocumentRetentionRead {
            identity: DOCUMENT_IDENTITY.to_owned(),
        })
        .execute()
        .expect("a selected commit executes its read")
        .rows()[0]
        .retention_days
}

fn selectable(
    runtime: &Runtime,
    branch: WorthQueryProductBranch,
    receipt: &WorthQueryApplicationCommitReceipt,
) -> bool {
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    match runtime
        .request(&principal, &scope)
        .on_branch(branch)
        .at_commit(receipt, WORK)
    {
        Ok(_) => true,
        Err(WorthQueryApplicationHistorySelectionDenial::CommitUnavailable) => false,
        Err(other) => panic!("history selection is only ever unavailable here: {other:?}"),
    }
}

#[test]
fn a_held_read_keeps_only_its_own_commit_until_it_is_released() {
    let host = publish_on_first_program();
    let runtime = host.runtime();
    let branch = host.current_world();
    commit(&host, branch, 1, 0x2710_0001);
    let detached = replay(&host, branch, 1, 0x2710_0001);
    commit(&host, branch, 2, 0x2710_0002);
    let held = observe_head(runtime, branch);
    for value in 3..9 {
        commit(&host, branch, value, 0x2710_0100 + value);
    }
    assert_eq!(
        retained_history(runtime, branch),
        (SETTLED_WINDOW + 1, false),
        "the head, its parent and the held commit stay; everything else retired"
    );
    assert_eq!(read_at(runtime, branch, &held), 2);
    assert!(
        !selectable(runtime, branch, &detached),
        "a replay answer holds no history, so its retired commit is unavailable"
    );

    drop(held);
    commit(&host, branch, 9, 0x2710_0200);
    assert_eq!(
        retained_history(runtime, branch),
        (SETTLED_WINDOW, false),
        "the released history retired at the next publication"
    );
}

#[test]
fn a_held_receipt_keeps_its_commit_selectable_until_it_is_dropped() {
    let host = publish_on_first_program();
    let runtime = host.runtime();
    let branch = host.current_world();
    let held_receipt = commit(&host, branch, 2, 0x2710_0402);
    for publication in 0..16 {
        commit(
            &host,
            branch,
            3 + publication % 6,
            0x2710_0410 + publication,
        );
        assert_eq!(read_at_commit(runtime, branch, &held_receipt), 2);
    }
    assert_eq!(
        retained_history(runtime, branch),
        (SETTLED_WINDOW + 1, false),
        "the receipt keeps only its own commit"
    );

    drop(held_receipt);
    commit(&host, branch, 9, 0x2710_0430);
    assert_eq!(
        retained_history(runtime, branch),
        (SETTLED_WINDOW, false),
        "a dropped receipt lets its history retire at the next publication"
    );
}

fn replay(
    host: &DocumentRetentionRuntime<RetentionProgramP0>,
    branch: WorthQueryProductBranch,
    retention: u64,
    key: u64,
) -> WorthQueryApplicationCommitReceipt {
    match set_retention(host, branch, retention, key).expect("the replay settles") {
        WorthQueryApplicationMutationOutcome::AlreadyCommitted(receipt) => receipt,
        other => panic!("a committed key answers its replay: {other:?}"),
    }
}

#[test]
fn inspection_of_a_retired_commit_fails_closed_and_its_replay_still_answers() {
    let host = publish_on_first_program();
    let runtime = host.runtime();
    let branch = host.current_world();
    let key = 0x2710_0301;
    commit(&host, branch, 1, key);

    let replayed = replay(&host, branch, 1, key);
    assert!(selectable(runtime, branch, &replayed));

    commit(&host, branch, 2, 0x2710_0302);
    commit(&host, branch, 3, 0x2710_0303);
    assert!(
        !selectable(runtime, branch, &replayed),
        "inspection of a retired commit is unavailable"
    );
    match set_retention(&host, branch, 1, key).expect("the replay settles") {
        WorthQueryApplicationMutationOutcome::AlreadyCommitted(receipt) => assert_eq!(
            receipt, replayed,
            "a retired commit's replay answers its own receipt from evidence"
        ),
        other => panic!("a retired commit's replay never commits again: {other:?}"),
    }
    assert_eq!(
        read_retention(runtime, branch),
        3,
        "a retired replay never runs the request again"
    );
}
