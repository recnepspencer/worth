//! Signal is a separate owner dimension of the graceful public Home oracle.
mod derived;
mod derived_quiescent;
mod opening;
mod operations;
mod protocol;
mod session;
use std::sync::{Arc,atomic::{AtomicUsize,Ordering}};
use worth_query_host::facade::WorthQueryApplicationHomeLocation;
use worth_query_testkit_host::facade::host::temporal_conditional_operation::contribution_installation::{ApplicationBuildIdentity,ContactCounters};
use protocol::Request;
const FIRST: &str = "11111111-1111-4111-8111-111111111111";
const SECOND: &str = "22222222-2222-4222-8222-222222222222";
#[test]
fn graceful_home_oracle_preserves_authored_signal_branches_and_mixed_operations() {
    run(false, FIRST);
}
#[test]
fn quiescent_home_kill_preserves_authored_signal_branches_and_mixed_operations() {
    run(true, FIRST);
}
#[test]
fn quiescent_home_kill_readmits_authored_signal_under_a_changed_build() {
    run(true, SECOND);
}
fn run(kill: bool, reopen_build: &str) {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("home");
    let request = super::super::opening::serial();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let initialized = Arc::new(AtomicUsize::new(0));
    let contacts = ContactCounters::default();
    let reference = opening::open(
        WorthQueryApplicationHomeLocation::MemoryOnly,
        ApplicationBuildIdentity::parse(FIRST).unwrap(),
        initialized.clone(),
        contacts.clone(),
        &execution,
    );
    let reference_branches = operations::branches(&reference);
    operations::mixed(&reference, reference_branches[0], 1);
    operations::definition(&reference, reference_branches[1]);
    operations::mixed(&reference, reference_branches[2], 2);
    let expected = operations::observe(&reference, &reference_branches);
    let (mut session, opened) =
        session::Session::start(&path, &directory.path().join("first"), FIRST);
    assert_eq!(opened.initialized, 1);
    assert_eq!(opened.contacts, (0, 0, 0, 0));
    let published = session.send(Request::Prepare);
    assert_eq!(published.states, expected);
    if kill {
        session.kill();
    } else {
        assert!(session.send(Request::Close).closed);
        session.finish();
    }
    let (mut session, recovered) =
        session::Session::start(&path, &directory.path().join("second"), reopen_build);
    assert_eq!(recovered.initialized, 0, "reopen never initializes again");
    assert_eq!(
        recovered.frontiers, published.frontiers,
        "reopen repeats neither owner effect nor a World publication"
    );
    assert_eq!(
        recovered.contacts,
        (0, 0, 0, 0),
        "reopen reconstructs definitions without executing providers"
    );
    assert_eq!(
        recovered.states, expected,
        "both owner frontiers and authored branches retain their exact state"
    );
    operations::mixed(&reference, reference_branches[1], 3);
    let continued = session.send(Request::Continue);
    assert_eq!(
        continued.states,
        operations::observe(&reference, &reference_branches)
    );
    assert_eq!(continued.initialized, 0);
    assert_eq!(initialized.load(Ordering::SeqCst), 1);
    assert!(session.send(Request::Close).closed);
    session.finish();
    let refused = reference
        .close(&execution)
        .err()
        .expect("MemoryOnly preserves its owned lifecycle refusal");
    assert!(refused.directory_denial().is_none());
    assert!(refused.query_denial().is_none());
    drop(refused.resume().unwrap());
}

#[test]
fn branch_inventory_requires_the_receiving_homes_own_advancement_phase() {
    let request = super::super::opening::serial();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let open = || {
        opening::open(
            WorthQueryApplicationHomeLocation::MemoryOnly,
            ApplicationBuildIdentity::parse(FIRST).unwrap(),
            Arc::new(AtomicUsize::new(0)),
            ContactCounters::default(),
            &execution,
        )
    };
    let first = open();
    let second = open();
    let inventory = first
        .runtime()
        .with_host_advancement(|phase| first.runtime().branches().inventory(&phase))
        .unwrap()
        .unwrap();
    assert_eq!(inventory.iter().len(), 1);
    drop(inventory);
    let refused = first
        .runtime()
        .with_host_advancement(|phase| second.runtime().branches().inventory(&phase))
        .unwrap();
    assert!(matches!(
        refused,
        Err(
            worth_query_host::facade::product::WorthQueryProductBranchInventoryDenial::ForeignPhase(
                _
            )
        )
    ));
    for home in [first, second] {
        let refused = home
            .close(&execution)
            .err()
            .expect("ordinary MemoryOnly lifecycle refusal");
        assert!(refused.directory_denial().is_none());
        assert!(refused.query_denial().is_none());
        drop(refused.resume().unwrap());
    }
}
