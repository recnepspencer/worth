use worth_query_admission::facade::application_query::{
    WorthQueryAdmittedApplicationQueryParameters, WorthQueryApplicationQueryLane,
};
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_installation::facade::WorthQueryInstalledApplicationQuery;

use super::super::super::{
    basis::{admit_application_query_permission_basis, ensure_selected_read_indexes_admitted},
    graph_read_plan_binding::WorthQueryQueryIndexPosture,
    WorthQueryAdmittedApplicationQueryPlan, WorthQueryApplicationQueryAccessContext,
    WorthQueryApplicationQueryAdmissionDenial, WorthQueryApplicationQueryAdmissionDenialKind,
    WorthQueryApplicationQueryControls,
};
use crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission;
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
        admission: &mut InvalidationEditAdmission,
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
            query,
            access,
            parameters,
            controls,
            None,
            |basis| {
                let selected = admit_application_query_permission_basis(self, basis)?;
                let prepared = ensure_selected_read_indexes_admitted(
                    graph,
                    query,
                    selected.selected_product().relational_basis(),
                    admission,
                )?;
                Ok((
                    selected,
                    WorthQueryQueryIndexPosture::SelectedInstalled(prepared),
                ))
            },
        )
    }
}
