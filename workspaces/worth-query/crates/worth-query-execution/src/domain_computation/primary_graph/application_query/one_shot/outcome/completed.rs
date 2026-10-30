//! Native query completion before selecting its public result or source closure.
use super::*;

pub(in crate::domain_computation::primary_graph::application_query) struct CompletedOneShotRead<
    Query,
    QueryResult,
    Capture = (),
> {
    pub(in crate::domain_computation::primary_graph::application_query) read:
        WorthQueryApplicationOneShotResult<Query, QueryResult>,
    pub(in crate::domain_computation::primary_graph::application_query) capture: Capture,
    pub(in crate::domain_computation::primary_graph::application_query) source_context:
        Option<crate::domain_computation::provider_session::WorthQueryQuerySourceContext>,
    pub(in crate::domain_computation::primary_graph::application_query) material_source: Option<
        std::sync::Arc<
            crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation,
        >,
    >,
    pub(in crate::domain_computation::primary_graph::application_query) authorization:
        crate::domain_computation::authorization::WorthQueryRetainedAuthorizationDecisionFacts,
}

pub(in crate::domain_computation::primary_graph::application_query) struct CompletedSourceParts<
    Query,
> {
    pub sources: Vec<super::super::super::WorthQueryObservedSource<Query>>,
    pub result_set: super::super::super::WorthQueryObservedResultSet<Query>,
    pub affinity: super::super::super::admitted_result::WorthQueryApplicationQueryRequestAffinity,
    pub terminal: crate::domain_computation::provider_session::WorthQueryCompletedSourceRead,
    pub allocation: std::sync::Arc<
        crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation,
    >,
}

impl<Query> CompletedOneShotRead<Query, ()> {
    /// Consumes the completed lane, never a descriptive receipt supplied by a caller.
    pub(in crate::domain_computation::primary_graph::application_query) fn into_source_parts(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<CompletedSourceParts<Query>, WorthQueryApplicationOneShotDenial> {
        let allocation = self.material_source.ok_or_else(|| {
            denial(
                WorthQueryApplicationOneShotDenialKind::ForeignPlan,
                "managed-source",
                "material custody",
            )
        })?;
        let context = self.source_context.ok_or_else(|| {
            denial(
                WorthQueryApplicationOneShotDenialKind::ForeignPlan,
                "managed-source",
                "source context",
            )
        })?;
        let storage = material::MaterialCompletion::new(execution, &allocation);
        let terminal = self.read.receipt.into_managed_source_terminal(
            context, execution, &allocation, |units| storage.work(units),
        ).map_err(|cause| super::super::map_authorized_read_denial(
            super::super::super::authorized_read::WorthQueryAuthorizedApplicationReadDenial::Read(
                super::super::super::read_execution::relational_read_denial(cause, "managed-source"),
            ), "managed-source",
        ))?.ok_or_else(|| denial(
            WorthQueryApplicationOneShotDenialKind::ForeignPlan,
            "managed-source",
            "source context session",
        ))?;
        Ok(CompletedSourceParts {
            sources: self.read.observed_sources,
            result_set: self.read.result_set_observation,
            affinity: self.read.request_affinity,
            terminal,
            allocation,
        })
    }
}

pub(super) fn complete_query(
    session: crate::domain_computation::provider_session::WorthQueryManagedGraphWorkSession,
    proof: crate::domain_computation::provider_session::WorthQuerySessionGraphReadProof,
    observed: crate::domain_computation::provider_session::WorthQueryObservedGraphReadWork,
    release: crate::domain_computation::primary_graph::WorthQueryApplicationBasisReleaseReceipt,
    material: bool,
    query: &str,
) -> Result<
    (
        crate::domain_computation::provider_session::WorthQueryGraphReadCompletion,
        Option<crate::domain_computation::provider_session::WorthQueryQuerySourceContext>,
    ),
    WorthQueryApplicationOneShotDenial,
> {
    let context = material
        .then(|| session.retain_query_source_context())
        .transpose()
        .map_err(|_| {
            denial(
                WorthQueryApplicationOneShotDenialKind::ForeignPlan,
                query,
                query,
            )
        })?;
    let terminal = session
        .complete_query_read(proof, observed, release)
        .map_err(|_| {
            denial(
                WorthQueryApplicationOneShotDenialKind::ForeignPlan,
                query,
                query,
            )
        })?;
    Ok((terminal, context))
}
