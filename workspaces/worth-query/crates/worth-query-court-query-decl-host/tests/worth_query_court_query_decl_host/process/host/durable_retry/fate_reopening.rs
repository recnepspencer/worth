//! A proven no-effect key crosses a quiescent process death.
use super::*;
use worth_query_host::facade::WorthQueryApplicationHome;

type Home = WorthQueryApplicationHome<fixture::schema::HomeIdentitySchema, fixture::HomeProgram>;
const NO_EFFECT: u64 = 0xface01;

fn open(fresh: bool, contacts: Arc<AtomicUsize>) -> Home {
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let build = ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap();
    let request = if fresh {
        fixture::request(
            child_reexecution::location(),
            build,
            Arc::new(AtomicUsize::new(0)),
            contacts,
        )
    } else {
        fixture::request_with_initializer(
            child_reexecution::location(),
            build,
            contacts,
            |_, _, _| panic!("a recovered fate cannot reinitialize"),
        )
    };
    with_bootstrap_advancement_from_request(execution, |phase| request.open_in_advancement(&phase))
        .unwrap()
        .unwrap_or_else(|refused| opening_failed("fate reader or writer Home", &refused))
}

#[test]
fn proven_no_effect_key_executes_once_after_kill_and_reopen() {
    child_reexecution::drive(
        "fate_reopening::proven_no_effect_key_executes_once_after_kill_and_reopen",
        no_effect_writer,
        no_effect_reader,
    );
}
fn no_effect_writer() {
    let contacts = Arc::new(AtomicUsize::new(0));
    let home = open(true, contacts.clone());
    let (cancel, stopped) = scope();
    let principal = identity::inspection_authentication::authenticate(
        home.runtime().runtime().installed_schema(),
        &stopped,
    );
    cancel.cancel();
    request_stops::assert_cancelled(
        home.runtime()
            .request(&principal, &stopped)
            .advanced()
            .mutate(fixture::native_mutation::SetAccountBalance {
                account: "account-001".to_owned(),
                balance: 8,
            })
            .without_source()
            .idempotency(&NO_EFFECT)
            .execute_in_program(
                home.runtime(),
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            ),
    );
    let (_, live) = scope();
    assert_eq!(contacts.load(Ordering::SeqCst), 0);
    assert_eq!(balance(home.runtime(), &live), 7);
    child_reexecution::wait_for_kill();
}
fn no_effect_reader() {
    let contacts = Arc::new(AtomicUsize::new(0));
    let home = open(false, contacts.clone());
    let (_, live) = scope();
    assert_eq!(balance(home.runtime(), &live), 7);
    let fresh = match set(home.runtime(), &live, NO_EFFECT, 8).unwrap() {
        Outcome::Committed { receipt, result: 8 } => receipt,
        other => panic!("a proven absent key executes once after reopen: {other:?}"),
    };
    let recorded = match set(home.runtime(), &live, NO_EFFECT, 8).unwrap() {
        Outcome::RecoveredCommitted(receipt) => receipt,
        other => panic!("the new original returns without duplicate effects: {other:?}"),
    };
    receipt_fields::assert_live_agrees(&fresh, &recorded);
    assert_eq!(contacts.load(Ordering::SeqCst), 1);
    assert_eq!(balance(home.runtime(), &live), 8);
}
