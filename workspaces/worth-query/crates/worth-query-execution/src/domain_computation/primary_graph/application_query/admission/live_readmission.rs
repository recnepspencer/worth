use worth_query_declaration::facade::{
    application_query::ApplicationQueryParameterSet, application_schema::ApplicationSchema,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationQuery;

use super::super::{
    disclosure::WorthQueryApplicationQueryGovernance, WorthQueryAdmittedApplicationQueryPlan,
    WorthQueryApplicationQueryAccessContext, WorthQueryApplicationQueryAdmissionDenial,
    WorthQueryApplicationQueryAdmissionDenialKind, WorthQueryApplicationQueryControls,
};
use super::denial::denial;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    pub(in crate::domain_computation::primary_graph::application_query) fn readmit_application_query_live<
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
        governance: WorthQueryApplicationQueryGovernance,
        parameters: ApplicationQueryParameterSet<Query>,
        controls: WorthQueryApplicationQueryControls<'a, Schema>,
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
        let (parameters, controls) =
            self.prepare_application_query_admission(query, access, parameters, controls)?;
        if !governance.readmission_matches(
            self.runtime.authority_identity(),
            query.identity(),
            parameters.identity(),
            access.principal().principal_entity_id(),
            access.scope().entity_id(),
        ) {
            return Err(denial(
                WorthQueryApplicationQueryAdmissionDenialKind::DisclosureAuthorizationMismatch,
                query.name(),
            ));
        }
        let pending = governance.into_pending();
        self.finish_application_query_admission(
            execution, query, access, parameters, controls, pending,
        )
    }
}
