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
pub(crate) fn open(
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
pub(crate) fn scope() -> (WorthQueryCancellationSource, WorthQueryRequestScope) {
    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(
        None,
        cancellation.token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system());
    (cancellation, scope)
}
pub(crate) fn set(
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
pub(crate) fn balance(runtime: &Runtime, scope: &WorthQueryRequestScope) -> u64 {
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
pub(crate) fn pending(
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

pub(crate) fn reopen_required(
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

pub(crate) fn assert_reopen_actions(
    effects: &worth_query_host::facade::primary_graph::WorthQueryProductUnpublishedApplication,
) {
    use worth_query_host::facade::primary_graph::ProductUnpublishedNextAction::{
        Inspect, ReopenRequired,
    };
    assert_eq!(effects.next_actions(), &[Inspect, ReopenRequired]);
}

/// C3 hold: an original whose first Home ended uncleanly still owns its
/// managed run. Until C3's cold release contract lands, close refuses with
/// that typed posture; the same serving Home resumes and is abandoned.
pub(crate) fn end_with_pending_managed_run<Schema, Program>(
    refused: worth_query_host::facade::WorthQueryApplicationHomeCloseRefused<Schema, Program>,
) where
    Schema: worth_query_decl::facade::application_schema::ApplicationSchema,
{
    use worth_query_execution::facade::query_obligations::QueryObligationDenial;
    use worth_query_host::facade::primary_graph::WorthQueryHomeRetirementDenial as Denial;
    assert!(
        matches!(
            refused.query_denial(),
            Some(Denial::ManagedCleanup(
                QueryObligationDenial::ManagedRunPending { .. }
            ))
        ),
        "C3 managed-run release is the only refusal: {refused:?}"
    );
    drop(
        refused
            .resume()
            .expect("refused close keeps the serving Home"),
    );
}
