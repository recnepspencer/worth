use worth_query_admission::facade::application_query::WorthQueryAdmittedApplicationQueryParameters;
use worth_query_admission::facade::application_query::WorthQueryApplicationQueryLane;
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_installation::facade::WorthQueryInstalledApplicationQuery;

use super::super::super::basis::admit_application_query_permission_basis;
use super::super::super::basis::ensure_selected_read_indexes_admitted;
use super::super::super::graph_read_plan_binding::WorthQueryQueryIndexPosture;
use super::super::super::WorthQueryAdmittedApplicationQueryPlan;
use super::super::super::WorthQueryApplicationQueryAccessContext;
use super::super::super::WorthQueryApplicationQueryAdmissionDenial;
use super::super::super::WorthQueryApplicationQueryAdmissionDenialKind;
use super::super::super::WorthQueryApplicationQueryControls;

use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Producer readmission prepares only this installed read's actual field
    /// indexes against its already selected Product and cumulative admission.
    pub(in crate::domain_computation::primary_graph) fn finish_application_query_readmission<
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
        let graph = self.runtime.primary_graph().ok_or_else(|| {
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
            None,
            |basis| {
                let selected = admit_application_query_permission_basis(execution, self, basis)?;
                let prepared = ensure_selected_read_indexes_admitted(
                    execution,
                    graph,
                    query,
                    selected.selected_product().relational_basis(),
                )?;
                Ok((
                    selected,
                    WorthQueryQueryIndexPosture::SelectedInstalled(prepared),
                ))
            },
        )
    }
}
