//! Proposed truth uses the exact D identity; all authority remains at C.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryEntityResolutionDenialKind as Resolution,
    WorthQueryEntityResolutionRefusalKind as Refusal,
};
use worth_relational::facade::runtime::{
    RelationalAppliedCandidateReadIdentity, RelationalPinnedRead,
};
impl<S: ApplicationSchema> SourceQueryReaderAccess<'_, S> {
    pub(in crate::domain_computation::primary_graph) fn with_applied_plan<B, R, E>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        intent: B::Input,
        before: &RelationalPinnedRead,
        candidate: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
        native: &mut ApplicationQueryNativeAccess<'_>,
        consume: impl for<'plan> FnOnce(
            SourceReadPlan<'plan, S, B>,
            &mut ApplicationQueryNativeAccess<'_>,
        ) -> Result<R, E>,
    ) -> Result<R, E>
    where
        B: ApplicationQueryBinding<S>,
        B::Input: ApplicationQueryIntent<S, Binding = B>,
        B::ScopeBinding: ApplicationQueryScopeResolution<S, B::PrincipalIdentity>,
        SourceValue<S, B>: WorthQueryApplicationProjection<S, B::Query>,
        E: From<SourceQueryObservationStop>,
    {
        let result = self.with_applied_plan_in_control::<B, R, E>(
            execution, intent, before, candidate, native, consume,
        );
        self.confirm_control(execution, native, result)
            .map_err(|cause| E::from(self.record_control_refusal(cause)))?
    }
    fn with_applied_plan_in_control<B, R, E>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        intent: B::Input,
        before: &RelationalPinnedRead,
        candidate: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
        native: &mut ApplicationQueryNativeAccess<'_>,
        consume: impl for<'plan> FnOnce(
            SourceReadPlan<'plan, S, B>,
            &mut ApplicationQueryNativeAccess<'_>,
        ) -> Result<R, E>,
    ) -> Result<R, E>
    where
        B: ApplicationQueryBinding<S>,
        B::Input: ApplicationQueryIntent<S, Binding = B>,
        B::ScopeBinding: ApplicationQueryScopeResolution<S, B::PrincipalIdentity>,
        SourceValue<S, B>: WorthQueryApplicationProjection<S, B::Query>,
        E: From<SourceQueryObservationStop>,
    {
        use SourceQueryObservationStop as Stop;
        self.confirm_control(execution, native, ())
            .map_err(|cause| self.record_control_refusal(cause))?;
        let selected = self.shared.selected();
        if self.history.is_some()
            || !std::ptr::eq(self.application, selected.application())
            || candidate
                .candidate_identity()
                .proposal()
                .branch_observation()
                != selected.product().relational_basis_descriptor().reference()
        {
            return Err(Stop::ForeignClosure.into());
        }
        let installed = self
            .application
            .installed_schema()
            .installed_query_binding::<B>()
            .map_err(Stop::Installation)?;
        let principal = selected
            .resolve_authenticated_principal_issued_with_native_access(
                execution,
                installed.principal_binding(),
                self.principal,
                self.request,
                WorthQueryPrincipalResolutionMode::Ordinary,
                None,
                native,
            )
            .map_err(Stop::Principal)?;
        let parameters = intent.parameters();
        let (field, value) = intent
            .into_scope()
            .into_field_parts(principal.principal().principal_identity());
        let absent = match selected.resolve_query_scope_selector_issued_with_native_access(
            execution,
            field,
            &value,
            self.request,
            native,
        ) {
            Ok(scope) => {
                return self.with_resolved_plan::<B, R, E>(
                    execution,
                    &installed,
                    permission::SourceQueryParameters::Live(parameters),
                    &principal,
                    &scope,
                    SourceScopePosture::Current,
                    native,
                    consume,
                )
            }
            Err(cause) if cause.kind() == Resolution::Refused(Refusal::UnknownEntity) => cause,
            Err(cause) => return Err(Stop::Scope(cause).into()),
        };
        let graph = self
            .application
            .runtime
            .primary_graph()
            .ok_or(Stop::ForeignClosure)?;
        let scope = graph
            .retain_entity_resolution_context()
            .resolve_applied_field::<S, _>(execution, before, candidate, field, value);
        if let Some(lapse) = principal.request_lapse() {
            return Err(
                Stop::Admission(WorthQueryApplicationQueryAdmissionDenial::new(
                    WorthQueryApplicationQueryAdmissionDenialKind::Authorization(
                        lapse.denial_kind(),
                    ),
                    B::IDENTITY,
                ))
                .into(),
            );
        }
        let scope = scope.map_err(|cause| {
            Stop::Scope(if matches!(cause.kind(), Resolution::Refused(_)) {
                absent.clone()
            } else {
                cause
            })
        })?;
        let (product, basis) = self
            .application
            .retain_selected_query_basis_admitted(self.shared)
            .map_err(Stop::Basis)?;
        let limits = self
            .application
            .resolve_application_query_limits(installed.limits());
        let controls = WorthQueryApplicationQueryControls::selected_read_one_shot(
            product,
            basis,
            limits.maximum_results(),
            self.request,
        )
        .map_err(|cause| Stop::Read(WorthQueryAdmittedOneShotStop::Admission(cause)))?;
        let access = WorthQueryApplicationQueryAccessContext::new(principal.principal(), &scope);
        let plan = self
            .application
            .admit_fresh_applied_source_query(
                execution,
                installed.query(),
                access,
                parameters,
                controls,
                selected,
                &principal,
                before,
                candidate,
                self.request,
                native,
            )
            .map_err(|cause| {
                use crate::domain_computation::authorization::{
                    WorthQueryOperationAuthorizationDenialKind as Authority,
                    WorthQueryOperationAuthorizationRefusalKind as Refused,
                };
                if matches!(
                    cause.kind(),
                    WorthQueryApplicationQueryAdmissionDenialKind::Authorization(
                        Authority::Refused(
                            Refused::PermissionDenied
                                | Refused::ScopeMismatch
                                | Refused::RelationalObservationRejected
                        )
                    )
                ) {
                    Stop::Scope(absent)
                } else {
                    Stop::Admission(cause)
                }
            })?;
        consume(plan, native)
    }
}
