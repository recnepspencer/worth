//! Graph continuation of the already prepared native permission.
use super::*;
use crate::domain_computation::primary_graph::application_query::execution_shape::PreparedOneShotShape;
use crate::domain_computation::primary_graph::application_query::WorthQueryAdmittedApplicationQueryControls;
impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn prepare_application_query_graph_work_admitted<
        Query,
        Parameters,
        QueryResult,
        Scope,
    >(
        &self,
        query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
        parameters: &WorthQueryAdmittedApplicationQueryParameters,
        controls: &WorthQueryAdmittedApplicationQueryControls<'_>,
        shape: PreparedOneShotShape<'_>,
        graph: &WorthQueryPrimaryGraph,
        disclosure: WorthQueryAdmittedApplicationDisclosureContract,
    ) -> Result<PreparedApplicationQueryGraphWork, WorthQueryApplicationQueryAdmissionDenial> {
        shape.consume(query, controls)?;
        validate_admission_request(controls.request_scope(), query.name())?;
        let requirements = derive_graph_read_access_requirements_for_contract(
            query.read_family_binding().planning_contract(),
            controls.lane(),
            controls.maximum_result_count().get(),
            parameters.identity(),
            query.canonical_work_policy().admission_planning(),
        )
        .map_err(|_| {
            denial(
                WorthQueryApplicationQueryAdmissionDenialKind::CanonicalWorkDenied,
                query.name(),
            )
        })?;
        let inventory = primary_graph_support_inventory(&graph.layout, query.continuation(), query.live(), &requirements)
            .map_err(|_| denial(WorthQueryApplicationQueryAdmissionDenialKind::ReadmissionPreparationMemoryExhausted, query.name()))?;
        let obligations = query.retain_graph_obligations_for_admission();
        let obligation_identity = obligations.identity().clone();
        let selected = select_installed_graph_obligations(
            obligations,
            WorthQueryGraphWorkIntent::application_query_read(),
        )
        .map_err(|cause| graph_work_denial(format!("{}: {cause:?}", query.name())))?;
        let reviewed = review_application_query_graph_work(selected, requirements, inventory)
            .map_err(|cause| graph_work_denial(format!("{}: {cause:?}", query.name())))?;
        if let Some(rejected) = reviewed.review().denial() {
            return Err(denial(
                WorthQueryApplicationQueryAdmissionDenialKind::GraphReadPlan(rejected.kind()),
                query.name(),
            ));
        }
        let canonical_work = WorthQueryCanonicalWorkPhases::new(
            query.installation_canonical_work(),
            parameters
                .canonical_basis()
                .work()
                .combine(reviewed.review().requirements().canonical_work()),
            worth_query_installation::facade::WorthQueryCanonicalWorkEvidence::zero(),
            worth_query_installation::facade::WorthQueryCanonicalWorkEvidence::zero(),
            worth_query_installation::facade::WorthQueryCanonicalWorkEvidence::zero(),
        );
        let plan =
            admit_application_query_graph_work(reviewed, self.graph_work_resource_support_ref())
                .map_err(|cause| graph_work_denial(format!("{}: {cause:?}", query.name())))?;
        Ok(PreparedApplicationQueryGraphWork {
            plan,
            obligation_identity,
            disclosure,
            canonical_work,
            continuation_index_id: None,
        })
    }
}
