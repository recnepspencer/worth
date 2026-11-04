use super::OracleRuntime;
use worth_query_host::facade::{
    admission::authenticated_principal::*,
    application_entry::{WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequestExt},
};
use worth_query_testkit_host::facade::host::{
    application_home as fixture, application_runtime_surface as identity,
};

pub type MutationOutcome = WorthQueryApplicationMutationOutcome<(), u64>;
/// Harness readiness, not a product mutation refusal or a fabricated outcome.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum HostScriptExecutionGap {
    UnkeyedMutationEntryUnavailable,
}
pub fn set(
    runtime: &OracleRuntime,
    key: Option<u64>,
    value: u64,
) -> Result<MutationOutcome, HostScriptExecutionGap> {
    let key = key.ok_or(HostScriptExecutionGap::UnkeyedMutationEntryUnavailable)?;
    let scope = WorthQueryRequestScope::new(None, WorthQueryCancellationSource::new().token());
    let principal = identity::inspection_authentication::authenticate(
        runtime.runtime().installed_schema(),
        &scope,
    );
    Ok(runtime
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
        .unwrap())
}
pub fn balance(runtime: &OracleRuntime) -> u64 {
    let scope = WorthQueryRequestScope::new(None, WorthQueryCancellationSource::new().token());
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
