use worth_foundational::facade::CanonicalDigestId;
use worth_query_admission::facade::application_query::WorthQueryApplicationParameterCanonicalArtifact;
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_installation::facade::WorthQueryInstalledApplicationQuery;
use worth_relational::facade::{
    identity::EntityId,
    indexes::{DerivedIndexGenerationId, RelatedEntityOrderingBoundary},
};

use super::authority::WorthQueryApplicationQueryContinuation;
use crate::domain_computation::primary_graph::{
    application_query::{
        WorthQueryApplicationQueryAdmissionDenial, WorthQueryApplicationQueryAdmissionDenialKind,
    },
    WorthQueryPrimaryGraphApplicationRuntime,
};

pub(super) struct WorthQueryValidatedContinuationAffinity {
    pub(super) parameter_basis: WorthQueryApplicationParameterCanonicalArtifact,
    pub(super) index_generation: DerivedIndexGenerationId,
    pub(super) boundary: RelatedEntityOrderingBoundary,
    pub(super) page_ordinal: u64,
    pub(super) product: crate::basis::WorthQueryProductObservationLease,
}

pub(super) fn validate_continuation_affinity<Schema, Query, Parameters, QueryResult, Scope>(
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    scope_entity_id: EntityId,
    continuation: WorthQueryApplicationQueryContinuation<
        Schema,
        Query,
        Parameters,
        QueryResult,
        Scope,
    >,
) -> Result<WorthQueryValidatedContinuationAffinity, WorthQueryApplicationQueryAdmissionDenial>
where
    Schema: ApplicationSchema,
{
    let contract = query.continuation().ok_or_else(|| {
        denial(
            WorthQueryApplicationQueryAdmissionDenialKind::StaleContinuation,
            query.name(),
        )
    })?;
    validate_runtime_and_query(application, query, contract.digest(), &continuation)?;
    let graph = application.runtime.primary_graph().ok_or_else(|| {
        denial(
            WorthQueryApplicationQueryAdmissionDenialKind::RuntimeSupportUnavailable,
            query.name(),
        )
    })?;
    let installed_index = graph
        .layout
        .continuation_ordering_index_id(contract)
        .ok_or_else(|| {
            denial(
                WorthQueryApplicationQueryAdmissionDenialKind::RuntimeSupportUnavailable,
                query.name(),
            )
        })?;
    if continuation.scope_entity_id != scope_entity_id {
        return Err(denial(
            WorthQueryApplicationQueryAdmissionDenialKind::ContinuationScopeMismatch,
            query.name(),
        ));
    }
    if continuation.graph_authority_identity
        != application.primary_graph_authority.authority_identity()
        || continuation.provider_identity != application.primary_graph_authority.provider_identity()
        || continuation.index_id != installed_index
    {
        return Err(denial(
            WorthQueryApplicationQueryAdmissionDenialKind::ContinuationProviderMismatch,
            query.name(),
        ));
    }
    Ok(WorthQueryValidatedContinuationAffinity {
        parameter_basis: continuation.parameter_basis,
        index_generation: continuation.index_generation,
        boundary: continuation.boundary,
        page_ordinal: continuation.page_ordinal,
        product: continuation.product,
    })
}

fn validate_runtime_and_query<Schema, Query, Parameters, QueryResult, Scope>(
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    continuation_contract_digest: &CanonicalDigestId,
    continuation: &WorthQueryApplicationQueryContinuation<
        Schema,
        Query,
        Parameters,
        QueryResult,
        Scope,
    >,
) -> Result<(), WorthQueryApplicationQueryAdmissionDenial> {
    if continuation.runtime_authority != application.runtime.authority_identity().as_u64() {
        return Err(denial(
            WorthQueryApplicationQueryAdmissionDenialKind::ForeignContinuation,
            query.name(),
        ));
    }
    if continuation.schema_binding != *query.binding_identity()
        || continuation.query_identity != *query.identity()
        || continuation.query_authority_identity != query.authority_identity()
        || continuation.continuation_contract_digest != *continuation_contract_digest
    {
        return Err(denial(
            WorthQueryApplicationQueryAdmissionDenialKind::StaleContinuation,
            query.name(),
        ));
    }
    Ok(())
}

fn denial(
    kind: WorthQueryApplicationQueryAdmissionDenialKind,
    subject: impl Into<String>,
) -> WorthQueryApplicationQueryAdmissionDenial {
    WorthQueryApplicationQueryAdmissionDenial::new(kind, subject)
}
