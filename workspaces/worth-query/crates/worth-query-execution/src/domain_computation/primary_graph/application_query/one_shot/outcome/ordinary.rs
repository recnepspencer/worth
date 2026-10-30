//! An ordinary one-shot read owns its decode custody until finalization returns.
use super::*;

pub(in crate::domain_computation::primary_graph::application_query) fn complete_one_shot_with_decode<
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
) -> Result<CompletedOneShotRead<Query, QueryResult>, WorthQueryApplicationOneShotDenial>
where
    Schema: ApplicationSchema,
    QueryResult: WorthQueryApplicationProjection<Schema, Query>,
{
    complete_one_shot_with_projection(
        execution,
        application,
        decode,
        plan,
        kernel,
        authorization_work,
        read_proof,
        |kernel, governance, request, query| {
            project_non_live_kernel_with_decode::<Schema, Query, QueryResult, _>(
                execution,
                decode,
                kernel,
                governance,
                || admit_request(request, query),
                |projection| {
                    denial(
                        WorthQueryApplicationOneShotDenialKind::Projection(projection.kind()),
                        query,
                        projection.subject(),
                    )
                },
            )
        },
    )
}

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

pub(in crate::domain_computation::primary_graph::application_query) fn complete_one_shot_with_projection<
    Schema,
    Query,
    Parameters,
    QueryResult,
    Principal,
    PrincipalIdentity,
    Scope,
    Projected,
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
    project: impl FnOnce(
        RawNonLiveKernelOutcome,
        &super::super::super::disclosure::WorthQueryApplicationQueryGovernance,
        &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
        &str,
    ) -> Result<
        super::super::super::read_execution::ProjectedNonLiveKernelOutcome<Projected>,
        WorthQueryApplicationOneShotDenial,
    >,
) -> Result<CompletedOneShotRead<Query, Projected>, WorthQueryApplicationOneShotDenial>
where
    Schema: ApplicationSchema,
{
    complete_one_shot_with_capture(
        execution,
        application,
        decode,
        plan,
        kernel,
        authorization_work,
        read_proof,
        project,
        |_, _| Ok(()),
    )
}
