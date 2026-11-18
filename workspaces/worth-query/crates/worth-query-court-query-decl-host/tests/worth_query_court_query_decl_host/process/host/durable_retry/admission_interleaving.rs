//! Two admissions retain and then observe the same durable original.
use super::*;
use worth_query_host::facade::primary_graph::WorthQueryApplicationUncommitted;

#[test]
fn identical_callers_retain_their_admitted_original() {
    let contacts = Arc::new(AtomicUsize::new(0));
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = pending_support::open(execution, contacts);
    let together = std::sync::Barrier::new(2);
    let run = || {
        let (_, live) = scope();
        together.wait();
        set_transport(home.runtime(), &live)
    };
    let (first, second) = std::thread::scope(|threads| {
        let first = threads.spawn(&run);
        let second = threads.spawn(&run);
        (first.join().unwrap(), second.join().unwrap())
    });
    describe(&first);
    describe(&second);
    let first = first.expect("first request is admitted");
    let second = second.expect("second request is admitted");
    let fresh = usize::from(matches!(first, Outcome::Committed { result: 8, .. }))
        + usize::from(matches!(second, Outcome::Committed { result: 8, .. }));
    let recovered = usize::from(matches!(first, Outcome::RecoveredCommitted(_)))
        + usize::from(matches!(second, Outcome::RecoveredCommitted(_)));
    assert_eq!(
        (fresh, recovered),
        (1, 1),
        "first={first:?}; second={second:?}"
    );
    let (_, live) = scope();
    assert_eq!(balance(home.runtime(), &live), 8);
}

fn describe(
    outcome: &Result<
        Outcome<(), u64>,
        worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenial,
    >,
) {
    if let Ok(Outcome::Commit(WorthQueryApplicationUncommitted::ProductUnpublished(value))) =
        outcome
    {
        eprintln!(
            "unpublished cause={:?}; before_effect={:?}; owner_effects={}; native={:?}",
            value.cause(),
            value.before_effect_refusal(),
            value.owner_effect_count(),
            value.native_semantic_result()
        );
    } else {
        eprintln!(
            "admission committed={} recovered={}",
            matches!(outcome, Ok(Outcome::Committed { .. })),
            matches!(outcome, Ok(Outcome::RecoveredCommitted(_)))
        );
    }
}

fn set_transport(
    runtime: &worth_query_host::facade::application_installation::WorthQueryProgramApplicationRuntime<fixture::schema::HomeIdentitySchema, fixture::HomeProgram>,
    scope: &WorthQueryRequestScope,
) -> Result<
    Outcome<(), u64>,
    worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenial,
> {
    let principal = identity::inspection_authentication::authenticate(
        runtime.runtime().installed_schema(),
        scope,
    );
    runtime
        .request(&principal, scope)
        .on_branch(runtime.runtime().current_world())
        .advanced()
        .mutate(fixture::native_mutation::SetAccountBalance {
            account: "account-001".to_owned(),
            balance: 8,
        })
        .without_source()
        .idempotency(&0_u64)
        .with_transport_idempotency([0x72; 32], [0x79; 32])
        .execute_in_program(
            runtime,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
}
