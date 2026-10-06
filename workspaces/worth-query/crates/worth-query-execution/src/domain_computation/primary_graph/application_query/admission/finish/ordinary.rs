use worth_query_admission::facade::application_query::WorthQueryAdmittedApplicationQueryParameters;
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_installation::facade::WorthQueryInstalledApplicationQuery;

use super::super::super::{
    basis::admit_application_query_basis, disclosure::WorthQueryPendingApplicationQueryGovernance,
    graph_read_plan_binding::WorthQueryQueryIndexPosture, WorthQueryAdmittedApplicationQueryPlan,
    WorthQueryApplicationQueryAccessContext, WorthQueryApplicationQueryAdmissionDenial,
    WorthQueryApplicationQueryControls,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// The established public Query path retains its historical index-currency owner.
    pub(in crate::domain_computation::primary_graph) fn finish_application_query_admission<
        'a,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >(
        &'a self,
        query: &'a WorthQueryInstalledApplicationQuery<
            Schema,
            Query,
            Parameters,
            QueryResult,
            Scope,
        >,
        access: &WorthQueryApplicationQueryAccessContext<
            'a,
            Schema,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
        parameters: WorthQueryAdmittedApplicationQueryParameters,
        controls: WorthQueryApplicationQueryControls<'a, Schema>,
        pending_governance: Option<WorthQueryPendingApplicationQueryGovernance>,
    ) -> Result<
        WorthQueryAdmittedApplicationQueryPlan<
            'a,
            Schema,
            Query,
            Parameters,
            QueryResult,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
        WorthQueryApplicationQueryAdmissionDenial,
    > {
        self.finish_application_query_admission_with_basis(
            query,
            access,
            parameters,
            controls,
            pending_governance,
            |basis| {
                admit_application_query_basis(self, basis)
                    .map(|basis| (basis, WorthQueryQueryIndexPosture::HistoricalAllPrimary))
            },
        )
    }
}
