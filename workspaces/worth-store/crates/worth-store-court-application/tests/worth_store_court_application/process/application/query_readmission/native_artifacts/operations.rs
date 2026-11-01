//! Ordinary public requests over the actual installed fixture runtime.
use super::*;
type Runtime =
    worth_query_host::facade::application_installation::WorthQueryProgramApplicationRuntime<
        HomeIdentitySchema,
        HomeProgram,
    >;

pub(super) fn fresh_closed(path: &std::path::Path, execution: ExecutionRequest<'_, '_>) {
    let request = fixture::request(
        path.to_path_buf(),
        build(),
        Arc::new(AtomicUsize::new(0)),
        Arc::new(AtomicUsize::new(0)),
    );
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        request.open_in_advancement(&phase)
    })
    .unwrap()
    .unwrap_or_else(|refused| panic!("fresh actual Home: {refused:?}"));
    home.close(&execution)
        .unwrap_or_else(|refused| panic!("actual fresh close: {refused:?}"));
}

fn scope() -> worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope {
    use worth_query_host::facade::admission::authenticated_principal::*;
    WorthQueryRequestScope::new(None, WorthQueryCancellationSource::new().token())
}
pub(super) fn set(runtime: &Runtime, value: u64) -> WorthQueryApplicationMutationOutcome<(), u64> {
    let scope = scope();
    let principal = identity::inspection_authentication::authenticate(
        runtime.runtime().installed_schema(),
        &scope,
    );
    runtime
        .request(&principal, &scope)
        .advanced()
        .mutate(fixture::native_mutation::SetAccountBalance {
            account: "account-001".into(),
            balance: value,
        })
        .without_source()
        .idempotency(&0x9911_u64)
        .execute_in_program(
            runtime,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
}
pub(super) fn balance(runtime: &Runtime) -> u64 {
    let scope = scope();
    let principal = identity::inspection_authentication::authenticate(
        runtime.runtime().installed_schema(),
        &scope,
    );
    let observed = runtime
        .request(&principal, &scope)
        .query(fixture::balance_query::AccountRead("account-001".into()))
        .execute()
        .unwrap();
    *observed.rows().first().unwrap()
}
