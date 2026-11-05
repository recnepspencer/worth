//! Fresh C-only source permission before a retained tree inspects cached custody.
use super::*;
use crate::domain_computation::primary_graph::application_query::RetainedPermissionScopedOps;
use crate::domain_computation::primary_graph::runtime_host::WorthQueryRuntimeIdentityHost;
use crate::domain_computation::primary_graph::WorthQueryHistoryOutputReadDenial;

impl<S: ApplicationSchema + 'static, P: ApplicationProgramDefinition<S>>
    WorthQueryProgramApplicationRuntime<S, P>
{
    /// Linearize a prepared SDK delivery without destroying it on refusal.
    /// The receiver moves values only; it runs no callback or cleanup under guards.
    pub fn confirm_history_output_delivery<T>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        authority: &WorthQueryHistoryOutputAuthority<'_, '_>,
        request: &WorthQueryRequestScope,
        value: T,
    ) -> Result<
        T,
        (
            T,
            crate::domain_computation::primary_graph::WorthQueryHistoryNavigationDenial,
        ),
    > {
        use crate::domain_computation::primary_graph::WorthQueryHistoryNavigationDenial as Denial;
        if let Err(cause) = authority.validate(phase, self.runtime().application_runtime(), request)
        {
            return Err((value, cause));
        }
        if !authority.profile().matches_program(self) {
            return Err((value, Denial::ForeignDocument));
        }
        let execution = match phase.checked_execution_request_for(self.runtime().product_runtime())
        {
            Ok(execution) => execution,
            Err(stop) => return Err((value, Denial::Stopped(stop))),
        };
        authority
            .producer_invocation()
            .confirm_in_execution_preserving(
                &execution,
                self.runtime().application_runtime(),
                value,
            )
    }

    pub(super) fn validate_history_demand<D: WorthQueryApplicationOutputDemand<S>>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        demand: &WorthQueryAdmittedHistoryProgramOutput<S, P, D>,
        authority: &WorthQueryHistoryOutputAuthority<'_, '_>,
        request: &WorthQueryRequestScope,
        branch: crate::basis::WorthQueryProductBranch,
    ) -> Result<(), WorthQueryHistoryOutputAdvanceDenial> {
        use WorthQueryHistoryOutputAdvanceDenial as Denial;
        authority
            .validate(phase, self.runtime().application_runtime(), request)
            .map_err(Denial::Authority)?;
        if !demand.profile.same_custody(authority.profile())
            || !demand.profile.matches_program(self)
            || !std::sync::Arc::ptr_eq(&demand.original, authority.original())
            || branch
                != crate::basis::WorthQueryProductBranch::from_occurrence(
                    demand
                        .original
                        .product_observation()
                        .lifecycle_incarnation(),
                )
        {
            return Err(Denial::Demand(WorthQueryOutputDemandDenial::new(
                crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenialKind::ForeignDemand,
                "history output demand belongs to another original navigation",
            )));
        }
        Ok(())
    }

    pub fn authorize_history_program_output<D: WorthQueryApplicationOutputDemand<S>>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        demand: &WorthQueryAdmittedHistoryProgramOutput<S, P, D>,
        authority: &WorthQueryHistoryOutputAuthority<'_, '_>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<S>,
        request: &WorthQueryRequestScope,
        branch: crate::basis::WorthQueryProductBranch,
    ) -> Result<(), WorthQueryHistoryOutputReadDenial> {
        use WorthQueryHistoryOutputReadDenial as Denial;
        self.validate_history_demand(phase, demand, authority, request, branch)
            .map_err(|stop| match stop {
                WorthQueryHistoryOutputAdvanceDenial::Authority(stop) => Denial::Authority(stop),
                WorthQueryHistoryOutputAdvanceDenial::Demand(stop) => Denial::Demand(stop),
            })?;
        let runtime = self.runtime();
        let execution = phase.checked_execution_request_for(runtime.product_runtime())
            .map_err(|cause| Denial::Authority(crate::domain_computation::primary_graph::WorthQueryHistoryNavigationDenial::Stopped(cause)))?;
        // No retained E lookup or currentness requirement. Recovery may still
        // own partial Native work whose original World settlement is unfinished.
        runtime
            .application_runtime()
            .authorize_retained_history_query::<Source<S, D>>(
                &execution,
                authority.producer_invocation(),
                demand.observed_source(),
                principal,
                request,
            )
            .map_err(Denial::from)
    }
}
