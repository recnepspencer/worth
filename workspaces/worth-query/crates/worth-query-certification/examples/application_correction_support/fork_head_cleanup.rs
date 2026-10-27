//! A fork closed without adoption leaves nothing behind: neither the
//! evidence of its steps nor the history an unpublished successor kept.
use super::{
    edit::Value,
    fork_head_adoption::{adopt, fork},
    host,
};
use worth_query_host::facade::{
    primary_graph::*,
    product::{
        WorthQueryApplicationProductBranchCleanupDenial, WorthQueryProductBranchOwnerCleanupDenial,
    },
};

pub fn run() {
    abandoned_forks_release_their_evidence();
    a_held_read_keeps_only_the_evidence_it_needs();
}

const ABANDONED: usize = 8;

/// A closed fork's commits pass their evidence on while World history still
/// holds them; once the close releases that history, its sweep lets the
/// evidence go too.
fn abandoned_forks_release_their_evidence() {
    let host = host::install();
    let head = fork(&host, host.runtime().current_world());
    host::edit_on(&host, head, Value::Primary(11), "kept");
    let kept = host.runtime().completed_evidence_retained_for_test();
    for abandoned in 1..=ABANDONED {
        let branch = fork(&host, head);
        host::edit_on(
            &host,
            branch,
            Value::Secondary(20 + abandoned as u64),
            &format!("abandoned-{abandoned}"),
        );
        adopt(&host, branch);
        assert!(host.runtime().branches().pending_cleanup().is_empty());
        assert_eq!(
            host.runtime().completed_evidence_retained_for_test(),
            kept,
            "abandoned fork {abandoned}: its evidence goes with its history"
        );
    }
    println!("{ABANDONED} abandoned forks released their evidence");
}

/// A read held on a fork's first step stops the close's walk there. The step
/// past it is released all the same, and its evidence goes with it before
/// the close reports; the held step's evidence waits for the retry.
fn a_held_read_keeps_only_the_evidence_it_needs() {
    let host = host::install();
    let head = fork(&host, host.runtime().current_world());
    host::edit_on(&host, head, Value::Primary(11), "kept");
    let kept = host.runtime().completed_evidence_retained_for_test();
    let branch = fork(&host, head);
    host::edit_on(&host, branch, Value::Secondary(31), "held-step");
    let held_evidence = host.runtime().completed_evidence_retained_for_test();
    let read = host
        .with_host_advancement(|phase| host.runtime().on_branch(branch).select(&phase).unwrap())
        .unwrap();
    host::edit_on(&host, branch, Value::Secondary(32), "released-step");
    let stepped_evidence = host.runtime().completed_evidence_retained_for_test();
    assert!(
        stepped_evidence > held_evidence,
        "the released step keeps evidence of its own: {kept} {held_evidence} {stepped_evidence}"
    );
    let Some(WorthQueryApplicationProductBranchCloseDenial::OwnerCleanupPending(failure)) = host
        .with_host_advancement(|phase| host.runtime().on_branch(branch).close(&phase).err())
        .unwrap()
    else {
        panic!("the held read defers the close's history release")
    };
    assert!(matches!(
        failure.denial(),
        WorthQueryApplicationProductBranchCleanupDenial::Product(
            WorthQueryProductBranchOwnerCleanupDenial::WorldHistoryStillRetained
        )
    ));
    assert_eq!(
        host.runtime().completed_evidence_retained_for_test(),
        held_evidence,
        "the released step's evidence goes though the walk stopped at the read"
    );
    drop((failure, read));
    for cleanup in host.runtime().branches().pending_cleanup() {
        assert!(cleanup.retry().unwrap().is_complete());
    }
    assert_eq!(
        host.runtime().completed_evidence_retained_for_test(),
        kept,
        "the retry releases the held step and its evidence"
    );
    println!("a held read kept only the evidence it needs");
}
