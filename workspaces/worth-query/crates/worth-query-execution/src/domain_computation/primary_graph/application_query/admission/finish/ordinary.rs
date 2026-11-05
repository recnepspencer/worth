use crate::domain_computation::primary_graph::application_query::admission::finish::FinishOwnOps;
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

pub(in crate::domain_computation::primary_graph) trait OrdinaryScopedOps<Schema: ApplicationSchema>:
    crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<Schema>
{
    fn finish_application_query_admission<
        'a,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >(
        &'a self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
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
            execution,
            query,
            access,
            parameters,
            controls,
            pending_governance,
            |basis, _security| {
                admit_application_query_basis(execution, self.application_query_host(), basis)
                    .map(|basis| (basis, WorthQueryQueryIndexPosture::HistoricalAllPrimary))
            },
        )
    }
}

impl<
        Schema: ApplicationSchema,
        HostRuntime: crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<
                Schema,
            > + ?Sized,
    > OrdinaryScopedOps<Schema> for HostRuntime
{
}
