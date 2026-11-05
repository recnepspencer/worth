use crate::domain_computation::primary_graph::application_query::admission::finish::FinishOwnOps;
use worth_query_admission::facade::application_query::WorthQueryAdmittedApplicationQueryParameters;
use worth_query_admission::facade::application_query::WorthQueryApplicationQueryLane;
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_installation::facade::WorthQueryInstalledApplicationQuery;

use super::super::super::basis::admit_application_query_permission_basis;
use super::super::super::basis::ensure_query_truth_and_security_indexes_admitted;
use super::super::super::graph_read_plan_binding::WorthQueryQueryIndexPosture;
use super::super::super::WorthQueryAdmittedApplicationQueryPlan;
use super::super::super::WorthQueryApplicationQueryAccessContext;
use super::super::super::WorthQueryApplicationQueryAdmissionDenial;
use super::super::super::WorthQueryApplicationQueryAdmissionDenialKind;
use super::super::super::WorthQueryApplicationQueryControls;


pub(in crate::domain_computation::primary_graph) trait ReadmittedScopedOps<
    Schema: ApplicationSchema,
>:
    crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<Schema>
{
    fn finish_application_query_readmission<
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
        pending_governance: Option<
            super::super::super::disclosure::WorthQueryPendingApplicationQueryGovernance,
        >,
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
        if controls.lane() != WorthQueryApplicationQueryLane::OneShot {
            return Err(WorthQueryApplicationQueryAdmissionDenial::new(
                WorthQueryApplicationQueryAdmissionDenialKind::RuntimeSupportUnavailable,
                query.name(),
            ));
        }
        let graph = self.primary_graph().ok_or_else(|| {
            WorthQueryApplicationQueryAdmissionDenial::new(
                WorthQueryApplicationQueryAdmissionDenialKind::StaleScope,
                query.name(),
            )
        })?;
        self.finish_application_query_admission_with_basis(
            execution,
            query,
            access,
            parameters,
            controls,
            pending_governance,
            |basis, security| {
                let selected = admit_application_query_permission_basis(
                    execution,
                    self.application_query_host(),
                    basis,
                )?;
                let prepared = ensure_query_truth_and_security_indexes_admitted(
                    execution,
                    graph,
                    query,
                    selected.selected_product().relational_basis(),
                    security.relational_basis(),
                )?;
                Ok((
                    selected,
                    WorthQueryQueryIndexPosture::SelectedInstalled(prepared),
                ))
            },
        )
    }
}

impl<
        Schema: ApplicationSchema,
        HostRuntime: crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<
                Schema,
            > + ?Sized,
    > ReadmittedScopedOps<Schema> for HostRuntime
{
}
