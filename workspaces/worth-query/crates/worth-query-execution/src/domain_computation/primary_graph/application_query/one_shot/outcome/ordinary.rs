//! An ordinary one-shot read owns its decode custody until finalization returns.
use super::*;

pub(in crate::domain_computation::primary_graph::application_query) fn finalize_one_shot<
    Schema,
    Query,
    Parameters,
    QueryResult,
    Principal,
    PrincipalIdentity,
    Scope,
>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    plan: WorthQueryAdmittedApplicationQueryPlan<
        '_,
        Schema,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >,
    kernel: RawNonLiveKernelOutcome,
    authorization_work: WorthQueryApplicationAuthorizationWorkEvidence,
    read_proof: crate::domain_computation::provider_session::WorthQuerySessionGraphReadProof,
) -> Result<
    WorthQueryApplicationOneShotResult<Query, QueryResult>,
    WorthQueryApplicationOneShotDenial,
>
where
    Schema: ApplicationSchema,
    QueryResult: WorthQueryApplicationProjection<Schema, Query>,
{
    let decode = ValueDecodeState::new(application.installed_schema().value_bindings().clone());
    finalize_one_shot_with_decode(
        execution,
        application,
        &decode,
        plan,
        kernel,
        authorization_work,
        read_proof,
    )
}

pub(in crate::domain_computation::primary_graph::application_query) fn finalize_one_shot_with_decode<
    Schema,
    Query,
    Parameters,
    QueryResult,
    Principal,
    PrincipalIdentity,
    Scope,
>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    decode: &ValueDecodeState,
    plan: WorthQueryAdmittedApplicationQueryPlan<
        '_,
        Schema,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >,
    kernel: RawNonLiveKernelOutcome,
    authorization_work: WorthQueryApplicationAuthorizationWorkEvidence,
    read_proof: crate::domain_computation::provider_session::WorthQuerySessionGraphReadProof,
) -> Result<
    WorthQueryApplicationOneShotResult<Query, QueryResult>,
    WorthQueryApplicationOneShotDenial,
>
where
    Schema: ApplicationSchema,
    QueryResult: WorthQueryApplicationProjection<Schema, Query>,
{
    complete_one_shot_with_decode(
        execution,
        application,
        decode,
        plan,
        kernel,
        authorization_work,
        read_proof,
    )
    .map(|completed| completed.read)
}

pub(in crate::domain_computation::primary_graph::application_query) fn complete_one_shot<
    Schema,
    Query,
    Parameters,
    QueryResult,
    Principal,
    PrincipalIdentity,
    Scope,
>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    plan: WorthQueryAdmittedApplicationQueryPlan<
        '_,
        Schema,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >,
    kernel: RawNonLiveKernelOutcome,
    authorization_work: WorthQueryApplicationAuthorizationWorkEvidence,
    read_proof: crate::domain_computation::provider_session::WorthQuerySessionGraphReadProof,
) -> Result<CompletedOneShotRead<Query, QueryResult>, WorthQueryApplicationOneShotDenial>
where
    Schema: ApplicationSchema,
    QueryResult: WorthQueryApplicationProjection<Schema, Query>,
{
    let decode = ValueDecodeState::new(application.installed_schema().value_bindings().clone());
    complete_one_shot_with_decode(
        execution,
        application,
        &decode,
        plan,
        kernel,
        authorization_work,
        read_proof,
    )
}
