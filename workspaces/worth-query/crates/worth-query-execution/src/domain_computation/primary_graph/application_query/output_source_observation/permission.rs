//! Source and Parent admissions share one installed permission/session owner.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryIssuedSelectedPrincipal, WorthQueryIssuedSelectedScope,
};
use worth_query_declaration::facade::application_query::{
    ApplicationQueryParameterSet, ApplicationQueryScopeBinding,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationQueryBinding;

impl<Schema: ApplicationSchema> SourceQueryReaderAccess<'_, Schema> {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn with_resolved_plan<
        B: ApplicationQueryBinding<Schema>,
        R,
        E: From<SourceQueryObservationStop>,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        installed: &WorthQueryInstalledApplicationQueryBinding<Schema, B>,
        parameters: ApplicationQueryParameterSet<B::Query>,
        principal: &WorthQueryIssuedSelectedPrincipal<
            '_,
            '_,
            Schema,
            B::Principal,
            B::PrincipalIdentity,
        >,
        scope: &WorthQueryIssuedSelectedScope<
            '_,
            '_,
            Schema,
            <B::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Scope,
        >,
        native: &mut ApplicationQueryNativeAccess<'_>,
        consume: impl for<'plan> FnOnce(
            SourceReadPlan<'plan, Schema, B>,
            &mut ApplicationQueryNativeAccess<'_>,
        ) -> Result<R, E>,
    ) -> Result<R, E> {
        use SourceQueryObservationStop as Stop;
        let application = self.application;
        let selected = self
            .history
            .as_ref()
            .map_or(self.shared, |history| history.current)
            .selected();
        let access =
            WorthQueryApplicationQueryAccessContext::new(principal.principal(), scope.scope());
        let query = installed.query();
        let limits = application.resolve_application_query_limits(installed.limits());
        let query_maximum = NonZeroUsize::new(query.read_graph().maximum_result_count())
            .unwrap_or(NonZeroUsize::MAX);
        let (product, basis) = application
            .retain_selected_query_basis_admitted(self.shared)
            .map_err(|stop| self.record_history_refusal(Stop::Basis(stop)))?;
        let controls = if let Some(history) = &self.history {
            WorthQueryApplicationQueryControls::selected_read_one_shot_with_control(
                history.invocation.current().retained_clone(),
                product,
                basis,
                limits.maximum_results().min(query_maximum),
                self.request,
            )
        } else {
            WorthQueryApplicationQueryControls::selected_read_one_shot(
                product,
                basis,
                limits.maximum_results().min(query_maximum),
                self.request,
            )
            .map_err(|stop| {
                self.record_history_refusal(Stop::Read(WorthQueryAdmittedOneShotStop::Admission(
                    stop,
                )))
            })?
        }
        .with_original_settled_producer(
            self.shared.selected().original_settled_producer().cloned(),
        );
        let (parameters, controls) = application
            .prepare_source_query_admission(query, &access, parameters, controls)
            .map_err(|cause| {
                self.record_history_refusal(Stop::Preparation {
                    phase: SourceQueryPreparationPhase::Parameters,
                    cause,
                })
            })?;
        let permission = application
            .prepare_application_query_permission_with_parameters(
                execution,
                query,
                access,
                query.identity(),
                parameters,
                controls,
                selected,
                self.history.as_ref().map(|_| self.shared.selected()),
                self.request,
                principal,
                scope,
                native,
            )
            .map_err(|cause| {
                self.record_history_refusal(Stop::Preparation {
                    phase: SourceQueryPreparationPhase::Permission,
                    cause,
                })
            })?;
        if let Some(history) = &self.history {
            permission
                .require_history_principal(history.invocation)
                .map_err(|cause| {
                    self.record_history_refusal(Stop::Preparation {
                        phase: SourceQueryPreparationPhase::Permission,
                        cause,
                    })
                })?;
        }
        let plan = application
            .finish_prepared_application_query_permission_with_native_access(
                execution, permission, native,
            )
            .map_err(|cause| {
                self.record_history_refusal(Stop::Preparation {
                    phase: SourceQueryPreparationPhase::Session,
                    cause,
                })
            })?;
        consume(plan, native)
    }
}
