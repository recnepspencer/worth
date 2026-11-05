use super::OracleRuntime;
use worth_query_host::facade::{
    admission::authenticated_principal::*,
    application_entry::{WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequestExt},
};
use worth_query_testkit_host::facade::host::{
    application_home as fixture, application_runtime_surface as identity,
};

pub type MutationOutcome = WorthQueryApplicationMutationOutcome<(), u64>;
pub fn resolve_original(
    runtime: &OracleRuntime,
    key: u64,
    value: u64,
) -> Result<
    worth_query_host::facade::primary_graph::WorthQueryApplicationIdempotencyResolution,
    worth_query_host::facade::application_entry::WorthQueryApplicationRecoveryRequestDenial,
> {
    let scope = WorthQueryRequestScope::new(None, WorthQueryCancellationSource::new().token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system());
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
        .idempotency(&key)
        .resolve_idempotency_in_program(runtime)
        .map(|read| read.into_resolution())
}
pub fn set(runtime: &OracleRuntime, key: u64, value: u64) -> MutationOutcome {
    let scope = WorthQueryRequestScope::new(None, WorthQueryCancellationSource::new().token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system());
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
        .idempotency(&key)
        .execute_in_program(
            runtime,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
}
pub fn balance(runtime: &OracleRuntime) -> u64 {
    let scope = WorthQueryRequestScope::new(None, WorthQueryCancellationSource::new().token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system());
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
