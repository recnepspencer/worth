//! The same public Host/request APIs; scheduling controls stay in the court.
use super::*;
use fixture::{schema::HomeIdentitySchema, HomeProgram};
use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequestMutationDenial,
    },
    application_installation::WorthQueryProgramApplicationRuntime,
    primary_graph::WorthQueryProductPreparationPending,
    WorthQueryApplicationHome,
};
type Runtime = WorthQueryProgramApplicationRuntime<HomeIdentitySchema, HomeProgram>;
pub(super) fn open(
    execution: worth_execution::ExecutionRequest<'_, '_>,
    contacts: Arc<AtomicUsize>,
) -> WorthQueryApplicationHome<HomeIdentitySchema, HomeProgram> {
    let request = fixture::request(
        fixture::location(),
        ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap(),
        Arc::new(AtomicUsize::new(0)),
        contacts,
    );
    with_bootstrap_advancement_from_request(execution, |phase| request.open_in_advancement(&phase))
        .expect("the real bootstrap execution opens")
        .unwrap_or_else(|refused| panic!("fresh physical Home: {refused:?}"))
}
pub(super) fn scope() -> (WorthQueryCancellationSource, WorthQueryRequestScope) {
    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(
        Some(Instant::now() + Duration::from_secs(60)),
        cancellation.token(),
    );
    (cancellation, scope)
}
pub(super) fn set(
    runtime: &Runtime,
    scope: &WorthQueryRequestScope,
    key: u64,
    value: u64,
) -> Result<WorthQueryApplicationMutationOutcome<(), u64>, WorthQueryApplicationRequestMutationDenial>
{
    let principal = identity::inspection_authentication::authenticate(
        runtime.runtime().installed_schema(),
        scope,
    );
    runtime
        .request(&principal, scope)
        .advanced()
        .mutate(fixture::native_mutation::SetAccountBalance {
            account: "account-001".to_owned(),
            balance: value,
        })
        .without_source()
        .idempotency(&key)
        .execute_in_program(
            runtime,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
}
pub(super) fn balance(runtime: &Runtime, scope: &WorthQueryRequestScope) -> u64 {
    let principal = identity::inspection_authentication::authenticate(
        runtime.runtime().installed_schema(),
        scope,
    );
    let result = runtime
        .request(&principal, scope)
        .query(fixture::balance_query::AccountRead(
            "account-001".to_owned(),
        ))
        .execute()
        .unwrap();
    result.rows()[0]
}
pub(super) fn pending(
    outcome: Result<
        WorthQueryApplicationMutationOutcome<(), u64>,
        WorthQueryApplicationRequestMutationDenial,
    >,
) -> WorthQueryProductPreparationPending {
    let outcome = outcome.expect("actual request retains preparation custody");
    match outcome {
        WorthQueryApplicationMutationOutcome::Commit(worth_query_host::facade::primary_graph::WorthQueryApplicationUncommitted::ProductPreparationPending(pending)) => pending,
        other => panic!("actual owned preparation Pending: {other:?}"),
    }
}

pub(super) fn reopen_required(
    result: Result<
        worth_query_host::facade::product::WorthQueryProductBranchOwnerCleanupReceipt,
        worth_query_host::facade::primary_graph::WorthQueryProductUnpublishedRecoveryReleaseFailure,
    >,
) -> worth_query_host::facade::primary_graph::WorthQueryProductUnpublishedRecovery {
    use worth_query_host::facade::primary_graph::{
        RuntimeWorldRecoveryDenial, WorthQueryProductUnpublishedRecoveryReleaseDenial as Denial,
        WorthQueryProductUnpublishedRecoveryReleaseFailure as Failure,
    };
    let Failure::Recovery(failure) =
        result.expect_err("a live persistent open requires Home reopen")
    else {
        panic!("the original recovery must remain owned");
    };
    assert_eq!(
        failure.denial(),
        Denial::World(RuntimeWorldRecoveryDenial::ReopenRequired)
    );
    let recovery = failure.into_recovery();
    let effects = recovery
        .inspect()
        .expect("the original record remains owned");
    assert_reopen_actions(&effects);
    drop(effects);
    recovery
}

pub(super) fn assert_reopen_actions(
    effects: &worth_query_host::facade::primary_graph::WorthQueryProductUnpublishedApplication,
) {
    use worth_query_host::facade::primary_graph::ProductUnpublishedNextAction::{
        Inspect, ReopenRequired,
    };
    assert_eq!(effects.next_actions(), &[Inspect, ReopenRequired]);
}
