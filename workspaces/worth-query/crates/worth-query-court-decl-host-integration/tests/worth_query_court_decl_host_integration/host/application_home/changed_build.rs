//! Changed implementation identity preserves the authored Relational Home.
use super::pending_support::{balance, scope, set};
use super::*;
use worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome as Outcome;
#[test]
fn directory_changed_build_preserves_authored_binding_original_receipt_and_does_no_eager_work() {
    let path = fixture::location();
    let contacts = Arc::new(AtomicUsize::new(0));
    let initialized = Arc::new(AtomicUsize::new(0));
    let build_a = ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap();
    let build_b = ApplicationBuildIdentity::parse("2a600000-0000-0000-0000-000000000002").unwrap();
    assert_ne!(build_a, build_b);
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        fixture::request(path.clone(), build_a, initialized.clone(), contacts.clone())
            .open_in_advancement(&phase)
    })
    .unwrap()
    .unwrap_or_else(|cause| panic!("build A: {cause:?}"));
    let (_, request) = scope();
    let original = match set(home.runtime(), &request, 0x2a6001, 8).unwrap() {
        Outcome::Committed { receipt, result: 8 } => receipt.commit_reference().clone(),
        other => panic!("actual authored A: {other:?}"),
    };
    home.close(&execution)
        .unwrap_or_else(|cause| panic!("A close: {cause:?}"))
        .reclaim();
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        fixture::request_with_initializer(path.clone(), build_b, contacts.clone(), |_, _, _| {
            panic!("a new build cannot initialize existing authored truth")
        })
        .open_in_advancement(&phase)
    })
    .unwrap()
    .unwrap_or_else(|cause| panic!("build B: {cause:?}"));
    assert_eq!(initialized.load(Ordering::SeqCst), 1);
    assert_eq!(
        contacts.load(Ordering::SeqCst),
        1,
        "open executes no supplier"
    );
    assert_eq!(balance(home.runtime(), &request), 8);
    match set(home.runtime(), &request, 0x2a6001, 8).unwrap() {
        Outcome::RecoveredCommitted(receipt) => assert_eq!(receipt.commit_reference(), &original),
        other => panic!("exact old authored receipt under B: {other:?}"),
    };
    assert_eq!(contacts.load(Ordering::SeqCst), 1);
    assert!(matches!(
        set(home.runtime(), &request, 0x2a6002, 9).unwrap(),
        Outcome::Committed { result: 9, .. }
    ));
    assert_eq!(contacts.load(Ordering::SeqCst), 2);
    home.close(&execution)
        .unwrap_or_else(|cause| panic!("B close: {cause:?}"))
        .reclaim();
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        fixture::request_with_initializer(path, build_b, contacts.clone(), |_, _, _| {
            panic!("B reopen cannot initialize")
        })
        .open_in_advancement(&phase)
    })
    .unwrap()
    .unwrap_or_else(|cause| panic!("B reopen: {cause:?}"));
    assert_eq!(balance(home.runtime(), &request), 9);
    assert_eq!(contacts.load(Ordering::SeqCst), 2);
    home.close(&execution)
        .unwrap_or_else(|cause| panic!("B continued close: {cause:?}"))
        .reclaim();
}
