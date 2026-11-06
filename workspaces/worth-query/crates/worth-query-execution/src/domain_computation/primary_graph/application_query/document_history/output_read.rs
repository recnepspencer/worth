//! Consume actual C admission before selecting the independent output truth.
use super::*;
use crate::domain_computation::primary_graph::application_query::{
    execution_validation::{
        validate_execution_lifetimes, validate_execution_plan,
        WorthQueryApplicationQueryExecutionValidationDenial as ControlStop,
    },
    WorthQueryApplicationQueryAccessContext, WorthQueryApplicationQueryControls,
};
use crate::domain_computation::primary_graph::{
    WorthQueryHistoryOutputAuthority, WorthQueryHistoryOutputReadDenial,
    WorthQuerySelectedProductOperation,
};
use worth_query_declaration::facade::application_query::ApplicationQueryScopeBinding;

type Scope<S, B> =
    <<B as ApplicationQueryBinding<S>>::ScopeBinding as ApplicationQueryScopeBinding<S>>::Scope;
type BoundParameters<S, B> = <<B as ApplicationQueryBinding<S>>::ParameterBinding as ApplicationStructuredValueBinding>::Value;
type BoundRows<S, B> =
    <<B as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value;
pub(in crate::domain_computation::primary_graph) type HistoryOutputControlPlan<'a, S, B> =
    WorthQueryAdmittedApplicationQueryPlan<
        'a,
        S,
        <B as ApplicationQueryBinding<S>>::Query,
        BoundParameters<S, B>,
        BoundRows<S, B>,
        <B as ApplicationQueryBinding<S>>::Principal,
        <B as ApplicationQueryBinding<S>>::PrincipalIdentity,
        Scope<S, B>,
    >;

impl<S: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<S> {
    pub(in crate::domain_computation::primary_graph) fn validate_history_output_control<B>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        authority: &WorthQueryHistoryOutputAuthority<'_, '_>,
        binding: &WorthQueryInstalledApplicationQueryBinding<S, B>,
        control: &HistoryOutputControlPlan<'_, S, B>,
    ) -> Result<(), WorthQueryHistoryOutputReadDenial>
    where
        B: ApplicationQueryBinding<S>,
    {
        use WorthQueryHistoryOutputReadDenial as ReadDenial;
        let request = control.controls.request_scope();
        authority
            .validate(phase, self, request)
            .map_err(ReadDenial::Authority)?;
        validate_execution_plan(self, control).map_err(control_denial)?;
        validate_execution_lifetimes(self, &control.controls, control.principal)
            .map_err(control_denial)?;
        let access = WorthQueryApplicationQueryAccessContext::new(control.principal, control.scope);
        if control.query.identity() != binding.query().identity()
            || control.security_product.observation() != authority.current().observation()
            || control.basis.selected_product().observation() != authority.current().observation()
            || !control.controls.is_current_product_one_shot()
            || !authority.matches_read_principal(&access)
        {
            return Err(ReadDenial::Authority(crate::domain_computation::primary_graph::WorthQueryHistoryNavigationDenial::ForeignDocument));
        }
        authority
            .confirm_read(phase, self, request, ())
            .map_err(ReadDenial::Authority)
    }

    pub(in crate::domain_computation::primary_graph) fn execute_history_output_query_on_selected<
        'control,
        B,
    >(
        &'control self,
        phase: &WorthQueryAdvancementPhase<'_>,
        authority: &WorthQueryHistoryOutputAuthority<'_, '_>,
        binding: &'control WorthQueryInstalledApplicationQueryBinding<S, B>,
        external: &WorthQueryAuthenticatedExternalPrincipal<S>,
        control: HistoryOutputControlPlan<'control, S, B>,
        retained: WorthQuerySelectedProductOperation<'control, S>,
    ) -> Result<
        WorthQueryApplicationOneShotResult<B::Query, BoundRows<S, B>>,
        WorthQueryHistoryOutputReadDenial,
    >
    where
        B: ApplicationQueryBinding<S>,
        BoundRows<S, B>: WorthQueryApplicationProjection<S, B::Query>,
    {
        use WorthQueryHistoryOutputReadDenial as ReadDenial;
        let request = control.controls.request_scope();
        self.validate_history_output_control(phase, authority, binding, &control)?;
        let access = WorthQueryApplicationQueryAccessContext::new(control.principal, control.scope);
        let execution = phase.checked_execution_request_for(&self.runtime.product_runtime)
            .map_err(|cause| ReadDenial::Authority(crate::domain_computation::primary_graph::WorthQueryHistoryNavigationDenial::Stopped(cause)))?;
        let producer = retained.original_settled_producer().cloned();
        let (_, product, basis) = retained.into_parts();
        let controls = WorthQueryApplicationQueryControls::retained_product_one_shot(
            control.security_product.retained_clone(),
            product,
            basis,
            control.controls.maximum_result_count(),
            request,
        )
        .with_original_settled_producer(producer);
        // The original C session remains alive during this transfer. D receives
        // a newly admitted session and freshly read C authorization; no session
        // ID or C-bound decision fact is transplanted into D.
        let plan = self
            .finish_application_query_admission(
                &execution,
                binding.query(),
                &access,
                control.parameters,
                controls,
                control.governance.into_readmission(),
            )
            .map_err(ReadDenial::Admission)?;
        let result = self
            .execute_application_query_binding_one_shot(phase, binding, external, plan)
            .map_err(ReadDenial::Read)?;
        validate_execution_lifetimes(self, &control.controls, control.principal)
            .map_err(control_denial)?;
        let result = authority
            .confirm_read(phase, self, request, result)
            .map_err(ReadDenial::Authority)?;
        validate_execution_lifetimes(self, &control.controls, control.principal)
            .map_err(control_denial)?;
        Ok(result)
    }
}

pub(super) fn control_denial(stop: ControlStop) -> WorthQueryHistoryOutputReadDenial {
    use crate::domain_computation::primary_graph::WorthQueryHistoryOutputControlDenial as Kind;
    WorthQueryHistoryOutputReadDenial::Control(match stop {
        ControlStop::ForeignPlan => Kind::ForeignPlan,
        ControlStop::StaleInstalledQuery => Kind::StaleInstalledQuery,
        ControlStop::StalePrincipal => Kind::StalePrincipal,
        ControlStop::Cancelled => Kind::Cancelled,
        ControlStop::DeadlineExceeded => Kind::DeadlineExceeded,
        ControlStop::ExpiredBasis => Kind::ExpiredBasis,
        ControlStop::BasisUnavailable => Kind::BasisUnavailable,
    })
}
