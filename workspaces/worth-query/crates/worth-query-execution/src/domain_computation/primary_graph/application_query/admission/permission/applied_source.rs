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
            &mut native,
        )?;
        self.finish_prepared_application_query_permission_with_native_access(
            execution,
            permission,
            &mut native,
        )
    }
}
