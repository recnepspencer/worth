//! The prepared permission owner admits proposed sources with C-only authority.
use super::*;
use worth_relational::facade::runtime::{
    RelationalAppliedCandidateReadIdentity, RelationalPinnedRead,
};
impl<S: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<S> {
    pub(in crate::domain_computation::primary_graph) fn admit_applied_source_query<
        'a,
        Q,
        P,
        R,
        A,
        I,
        E,
    >(
        &'a self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        query: &'a WorthQueryInstalledApplicationQuery<S, Q, P, R, E>,
        access: WorthQueryApplicationQueryAccessContext<'a, S, A, I, E>,
        parameters: &crate::domain_computation::primary_graph::application_query::CapturedApplicationQueryParameters,
        controls: super::super::WorthQueryApplicationQueryControls<'a, S>,
        selected: &'a WorthQuerySelectedProductOperation<'_, S>,
        principal: &'a WorthQueryIssuedSelectedPrincipal<'_, '_, S, A, I>,
        before: &'a RelationalPinnedRead,
        candidate: &'a RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    ) -> Result<
        super::super::WorthQueryAdmittedApplicationQueryPlan<'a, S, Q, P, R, A, I, E>,
        WorthQueryApplicationQueryAdmissionDenial,
    > {
        let source_query = parameters.query();
        let (parameters, controls) = self.prepare_application_query_source_readmission(
            query,
            &access,
            parameters.parameters(),
            controls,
        )?;
        let mut native = crate::domain_computation::primary_graph::application_query::ApplicationQueryNativeAccess::acquire();
        self.admit_prepared_applied_source_query(
            execution,
            query,
            access,
            source_query,
            parameters,
            controls,
            selected,
            principal,
            before,
            candidate,
            request,
            &mut native,
        )
    }
    pub(in crate::domain_computation::primary_graph::application_query) fn admit_fresh_applied_source_query<
        'a,
        Q,
        P,
        R,
        A,
        I,
        E,
    >(
        &'a self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        query: &'a WorthQueryInstalledApplicationQuery<S, Q, P, R, E>,
        access: WorthQueryApplicationQueryAccessContext<'a, S, A, I, E>,

        parameters:worth_query_declaration::facade::application_query::ApplicationQueryParameterSet<Q>,
        controls: super::super::WorthQueryApplicationQueryControls<'a, S>,
        selected: &'a WorthQuerySelectedProductOperation<'_, S>,
        principal: &'a WorthQueryIssuedSelectedPrincipal<'_, '_, S, A, I>,
        before: &'a RelationalPinnedRead,
        candidate: &'a RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
        native:&mut crate::domain_computation::primary_graph::application_query::ApplicationQueryNativeAccess<'_>,
    ) -> Result<
        super::super::WorthQueryAdmittedApplicationQueryPlan<'a, S, Q, P, R, A, I, E>,
        WorthQueryApplicationQueryAdmissionDenial,
    > {
        let (parameters, controls) =
            self.prepare_source_query_admission(query, &access, parameters, controls)?;
        self.admit_prepared_applied_source_query(
            execution,
            query,
            access,
            query.identity(),
            parameters,
            controls,
            selected,
            principal,
            before,
            candidate,
            request,
            native,
        )
    }
    fn admit_prepared_applied_source_query<'a, Q, P, R, A, I, E>(
        &'a self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        query: &'a WorthQueryInstalledApplicationQuery<S, Q, P, R, E>,
        access: WorthQueryApplicationQueryAccessContext<'a, S, A, I, E>,
        source_query:&worth_query_installation::facade::WorthQueryInstalledApplicationQueryIdentity,
        parameters:worth_query_admission::facade::application_query::WorthQueryAdmittedApplicationQueryParameters,
        controls: super::super::WorthQueryApplicationQueryControls<'a, S>,
        selected: &'a WorthQuerySelectedProductOperation<'_, S>,
        principal: &'a WorthQueryIssuedSelectedPrincipal<'_, '_, S, A, I>,
        before: &'a RelationalPinnedRead,
        candidate: &'a RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
        native:&mut crate::domain_computation::primary_graph::application_query::ApplicationQueryNativeAccess<'_>,
    ) -> Result<
        super::super::WorthQueryAdmittedApplicationQueryPlan<'a, S, Q, P, R, A, I, E>,
        WorthQueryApplicationQueryAdmissionDenial,
    > {
        let permission = self.prepare_application_query_permission_on_root(
            execution,
            query,
            access,
            source_query,
            parameters,
            controls,
            selected,
            None,
            request,
            |access| {
                SelectedIssuedAccessRoot::from_applied_source(
                    selected,
                    access,
                    principal,
                    before,
                    candidate,
                    request,
                    query.name(),
                )
            },
            native,
        )?;
        self.finish_prepared_application_query_permission_with_native_access(
            execution, permission, native,
        )
    }
}
