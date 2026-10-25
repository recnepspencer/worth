//! A fork closed without adoption leaves nothing behind, not even the
//! history an unpublished successor kept.
use super::{
    edit::Value,
    fork_head_adoption::{adopt, fork, mutate},
    host,
};
use worth_query_host::facade::{
    application_entry::*, primary_graph::*, runtime::ProductUnpublishedCause,
};

pub fn run() {
    an_unpublished_fork_closes_after_its_recovery();
}

/// A fork's publication installs its successor but never moves the head. The
/// fork cannot close while that recovery holds it; once released, the close
/// leaves only the parent's history and evidence. World's own court closes
/// first and walks on from the successor when recovery lets it go.
fn an_unpublished_fork_closes_after_its_recovery() {
    let host = host::install();
    let parent = fork(&host, host.runtime().current_world());
    host::edit_on(&host, parent, Value::Primary(11), "parent");
    // A fork closed plainly sets what the parent alone keeps.
    let plain = fork(&host, parent);
    host::edit_on(&host, plain, Value::Secondary(17), "plain-step");
    adopt(&host, plain);
    let parent_history = installed_commits(&host);
    let parent_evidence = host.runtime().completed_evidence_retained_for_test();
    let branch = fork(&host, parent);
    host::edit_on(&host, branch, Value::Secondary(19), "fork-step");
    let stepped = installed_commits(&host);
    host.with_host_advancement(|phase| {
        let execution = phase
            .checked_execution_request_for(host.runtime().product_runtime())
            .unwrap();
        host.runtime().fail_next_durable_append_for_test(&execution);
    })
    .unwrap();
    let WorthQueryApplicationMutationOutcome::Commit(
        WorthQueryApplicationUncommitted::ProductUnpublished(partial),
    ) = mutate(&host, branch, Value::Secondary(21), "fork-unpublished")
    else {
        panic!("a failed durable append leaves the fork's successor unpublished")
    };
    assert_eq!(partial.cause(), ProductUnpublishedCause::SettlementPending);
    let recovery = partial.into_recovery();
    assert_eq!(
        installed_commits(&host),
        stepped + 1,
        "the unpublished successor is installed beside the fork's head"
    );
    recovery.continue_owner_settlement().unwrap();
    // Closing first would leave the successor the only owner of the fork's
    // history; Query refuses the close until the recovery goes.
    let refused = host
        .with_host_advancement(|phase| host.runtime().on_branch(branch).close(&phase).err())
        .unwrap();
    assert!(
        matches!(
            refused,
            Some(WorthQueryApplicationProductBranchCloseDenial::ApplicationSettlementPending)
        ),
        "an unpublished fork refuses to close: {refused:?}"
    );
    host.runtime()
        .release_product_publication_recovery(recovery, 0)
        .unwrap_or_else(|failure| panic!("the recovery releases: {failure:?}"));
    assert_eq!(installed_commits(&host), stepped, "the successor went");
    adopt(&host, branch);
    assert!(host.runtime().branches().pending_cleanup().is_empty());
    assert_eq!(
        installed_commits(&host),
        parent_history,
        "the close leaves what a plain fork's close leaves"
    );
    assert_eq!(
        host.runtime().completed_evidence_retained_for_test(),
        parent_evidence,
        "the fork step's evidence goes with its history"
    );
    assert!(host.runtime().branches().pending_cleanup().is_empty());
    println!("an unpublished fork closed after its recovery and left only its parent");
}

fn installed_commits(host: &host::Host) -> usize {
    host.runtime()
        .world_history_snapshot_for_test()
        .installed_commits()
}
