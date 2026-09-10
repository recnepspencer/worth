use std::marker::PhantomData;

use worth_query_admission::facade::application_query::WorthQueryApplicationParameterCanonicalArtifact;
use worth_query_declaration::facade::application_schema::ApplicationSchema;

use super::{
    authority::WorthQueryApplicationQueryContinuation,
    denial::WorthQueryApplicationContinuationDenial,
    denial::WorthQueryApplicationContinuationDenialKind,
    WorthQueryApplicationContinuationPageResult,
};
use crate::domain_computation::primary_graph::application_query::{
    access_receipt::{
        WorthQueryApplicationQueryReceiptBasis, WorthQueryApplicationQueryReceiptIdentity,
    },
    execution_validation::validate_request,
    read_execution::{
        project_non_live_kernel, NonLiveKernelReceiptEvidence, RawNonLiveKernelOutcome,
    },
    WorthQueryAdmittedApplicationQueryPlan, WorthQueryApplicationAuthorizationWorkEvidence,
    WorthQueryApplicationProjection, WorthQueryApplicationQueryAccessReceipt,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

struct WorthQueryContinuationIdentity {
    runtime_authority: u64,
    schema_binding: worth_query_installation::facade::ApplicationSchemaBindingIdentity,
    query_identity: worth_query_installation::facade::WorthQueryInstalledApplicationQueryIdentity,
    query_authority_identity: String,
    parameter_basis: WorthQueryApplicationParameterCanonicalArtifact,
    scope_entity_id: worth_relational::facade::identity::EntityId,
    contract_digest: worth_foundational::facade::CanonicalDigestId,
    graph_authority_identity: String,
    provider_identity: String,
    index_id: worth_relational::facade::indexes::DerivedIndexId,
    product: Option<crate::basis::WorthQueryProductObservationLease>,
    next_page_ordinal: u64,
}

struct WorthQueryContinuationFinalization {
    subject: String,
    request: worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    authentication_valid_until: std::time::Instant,
    governance: crate::domain_computation::primary_graph::application_query::disclosure::WorthQueryApplicationQueryGovernance,
    continuation: WorthQueryContinuationIdentity,
    receipt_identity: WorthQueryApplicationQueryReceiptIdentity,
    receipt_basis: WorthQueryApplicationQueryReceiptBasis,
    graph_work: crate::domain_computation::provider_session::WorthQueryManagedGraphWorkSession,
    canonical_work: worth_query_installation::facade::WorthQueryCanonicalWorkPhases,
    basis_release: super::super::WorthQueryApplicationBasisReleaseReceipt,
}

pub(super) fn finalize_continuation_page<
    Schema,
    Query,
    Parameters,
    QueryResult,
    Principal,
    PrincipalIdentity,
    Scope,
>(
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
    WorthQueryApplicationContinuationPageResult<Schema, Query, Parameters, QueryResult, Scope>,
    WorthQueryApplicationContinuationDenial,
>
where
    Schema: ApplicationSchema,
    QueryResult: WorthQueryApplicationProjection<Schema, Query>,
{
    let mut finalization = release_continuation_page_basis(plan)?;
    validate_continuation_projection(application, &finalization)?;
    let projected = project_non_live_kernel::<Schema, Query, QueryResult, _>(
        kernel,
        &finalization.governance,
        || {
            validate_request(&finalization.request).map_err(|validation| {
                denial(map_validation_denial(validation), &finalization.subject)
            })
        },
        |projection: crate::domain_computation::primary_graph::WorthQueryApplicationProjectionDenial| {
            denial(
                WorthQueryApplicationContinuationDenialKind::Projection(projection.kind()),
                projection.subject(),
            )
        },
    )?;
    let (rows, kernel_receipt) = projected.into_parts();
    let continuation = mint_continuation::<Schema, Query, Parameters, QueryResult, Scope>(
        &mut finalization,
        &kernel_receipt,
    )?;
    complete_continuation_page(
        finalization,
        rows,
        continuation,
        kernel_receipt,
        authorization_work,
        read_proof,
    )
}

fn release_continuation_page_basis<
    Schema,
    Query,
    Parameters,
    QueryResult,
    Principal,
    PrincipalIdentity,
    Scope,
>(
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
) -> Result<WorthQueryContinuationFinalization, WorthQueryApplicationContinuationDenial> {
    let subject = plan.query.name().to_string();
    let basis_identity = plan.basis.identity().clone();
    let basis_version = plan.basis.version_id();
    let product = plan.basis.retained_product();
    let continuation = WorthQueryContinuationIdentity {
        runtime_authority: plan.runtime_authority.as_u64(),
        schema_binding: plan.query.binding_identity().clone(),
        query_identity: plan.query.identity().clone(),
        query_authority_identity: plan.query.authority_identity().to_string(),
        parameter_basis: plan.parameters.canonical_basis().clone(),
        scope_entity_id: plan.scope.entity_id(),
        contract_digest: *plan
            .query
            .continuation()
            .expect("continuation plans retain an installed continuation contract")
            .digest(),
        graph_authority_identity: plan.graph_authority_identity.clone(),
        provider_identity: plan.provider_identity.clone(),
        index_id: plan
            .continuation_index_id
            .expect("continuation plans retain an installed ordered index"),
        product: Some(product),
        next_page_ordinal: plan
            .continuation_state
            .as_ref()
            .map_or(2, |state| state.page_ordinal.saturating_add(1)),
    };
    let receipt_identity = WorthQueryApplicationQueryReceiptIdentity {
        query_identity: plan.query.identity().clone(),
        parameter_binding_identity: *plan.parameters.identity(),
        graph_authority_identity: plan.graph_authority_identity,
        provider_identity: plan.provider_identity,
    };
    let receipt_basis = WorthQueryApplicationQueryReceiptBasis {
        identity: basis_identity,
        version: basis_version,
        posture: plan.controls.basis_posture(),
        lane: plan.controls.lane(),
        consistency: plan.controls.consistency(),
        freshness: plan.controls.freshness(),
        released: true,
    };
    let request = plan.controls.request_scope().clone();
    let authentication_valid_until = plan.principal.valid_until();
    let basis_release = plan.basis.release();
    if !basis_release.released() {
        return Err(denial(
            WorthQueryApplicationContinuationDenialKind::BasisReleaseFailed,
            subject,
        ));
    }
    Ok(WorthQueryContinuationFinalization {
        subject,
        request,
        authentication_valid_until,
        governance: plan.governance,
        continuation,
        receipt_identity,
        receipt_basis,
        graph_work: plan.graph_work,
        canonical_work: plan.canonical_work,
        basis_release,
    })
}

fn validate_continuation_projection<Schema>(
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    finalization: &WorthQueryContinuationFinalization,
) -> Result<(), WorthQueryApplicationContinuationDenial> {
    validate_request(&finalization.request)
        .map_err(|validation| denial(map_validation_denial(validation), &finalization.subject))?;
    if application.authentication_is_expired(finalization.authentication_valid_until) {
        return Err(denial(
            WorthQueryApplicationContinuationDenialKind::StalePrincipal,
            &finalization.subject,
        ));
    }
    Ok(())
}

fn mint_continuation<Schema, Query, Parameters, QueryResult, Scope>(
    finalization: &mut WorthQueryContinuationFinalization,
    kernel_receipt: &NonLiveKernelReceiptEvidence,
) -> Result<
    Option<WorthQueryApplicationQueryContinuation<Schema, Query, Parameters, QueryResult, Scope>>,
    WorthQueryApplicationContinuationDenial,
> {
    let continuation = match (
        kernel_receipt.read.has_more,
        kernel_receipt.read.next_boundary.clone(),
        kernel_receipt.read.ordered_index_generation,
    ) {
        (true, Some(boundary), Some(index_generation)) => {
            Some(WorthQueryApplicationQueryContinuation {
                runtime_authority: finalization.continuation.runtime_authority,
                schema_binding: finalization.continuation.schema_binding.clone(),
                query_identity: finalization.continuation.query_identity.clone(),
                query_authority_identity: finalization
                    .continuation
                    .query_authority_identity
                    .clone(),
                parameter_basis: finalization.continuation.parameter_basis.clone(),
                scope_entity_id: finalization.continuation.scope_entity_id,
                continuation_contract_digest: finalization.continuation.contract_digest,
                graph_authority_identity: finalization
                    .continuation
                    .graph_authority_identity
                    .clone(),
                provider_identity: finalization.continuation.provider_identity.clone(),
                product: finalization
                    .continuation
                    .product
                    .take()
                    .expect("a materialized next page consumes its retained product"),
                index_id: finalization.continuation.index_id,
                index_generation,
                boundary,
                page_ordinal: finalization.continuation.next_page_ordinal,
                _marker: PhantomData,
            })
        }
        (false, None, Some(_)) => None,
        _ => {
            return Err(denial(
                WorthQueryApplicationContinuationDenialKind::ContinuationIndexUnavailable,
                &finalization.subject,
            ))
        }
    };
    Ok(continuation)
}

fn complete_continuation_page<Schema, Query, Parameters, QueryResult, Scope>(
    finalization: WorthQueryContinuationFinalization,
    rows: Vec<QueryResult>,
    continuation: Option<
        WorthQueryApplicationQueryContinuation<Schema, Query, Parameters, QueryResult, Scope>,
    >,
    kernel_receipt: NonLiveKernelReceiptEvidence,
    authorization_work: WorthQueryApplicationAuthorizationWorkEvidence,
    read_proof: crate::domain_computation::provider_session::WorthQuerySessionGraphReadProof,
) -> Result<
    WorthQueryApplicationContinuationPageResult<Schema, Query, Parameters, QueryResult, Scope>,
    WorthQueryApplicationContinuationDenial,
> {
    let receipt = WorthQueryApplicationQueryAccessReceipt::from_non_live_kernel(
        finalization.receipt_identity,
        finalization.receipt_basis,
        finalization
            .graph_work
            .complete_query_read(
                read_proof,
                kernel_receipt.observed_graph_read_work(),
                finalization.basis_release,
            )
            .map_err(|_| {
                denial(
                    WorthQueryApplicationContinuationDenialKind::ForeignPlan,
                    finalization.subject,
                )
            })?,
        finalization.canonical_work,
        authorization_work,
        finalization.governance.receipt(),
        kernel_receipt,
    );
    Ok(WorthQueryApplicationContinuationPageResult {
        rows,
        continuation,
        receipt,
        _query: PhantomData,
    })
}

fn map_validation_denial(
    denial: crate::domain_computation::primary_graph::application_query::execution_validation::WorthQueryApplicationQueryExecutionValidationDenial,
) -> WorthQueryApplicationContinuationDenialKind {
    use crate::domain_computation::primary_graph::application_query::execution_validation::WorthQueryApplicationQueryExecutionValidationDenial as Validation;
    match denial {
        Validation::Cancelled => WorthQueryApplicationContinuationDenialKind::Cancelled,
        Validation::DeadlineExceeded => {
            WorthQueryApplicationContinuationDenialKind::DeadlineExceeded
        }
        Validation::StalePrincipal => WorthQueryApplicationContinuationDenialKind::StalePrincipal,
        Validation::ExpiredBasis => WorthQueryApplicationContinuationDenialKind::ExpiredBasis,
        Validation::BasisUnavailable => {
            WorthQueryApplicationContinuationDenialKind::BasisUnavailable
        }
        Validation::ForeignPlan => WorthQueryApplicationContinuationDenialKind::ForeignPlan,
        Validation::StaleInstalledQuery => {
            WorthQueryApplicationContinuationDenialKind::StaleInstalledQuery
        }
    }
}

fn denial(
    kind: WorthQueryApplicationContinuationDenialKind,
    subject: impl Into<String>,
) -> WorthQueryApplicationContinuationDenial {
    WorthQueryApplicationContinuationDenial::new(kind, subject)
}
