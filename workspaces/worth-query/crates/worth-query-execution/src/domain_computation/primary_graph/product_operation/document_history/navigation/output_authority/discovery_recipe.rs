//! Selected recipe data remains step-bound under the same current C control.
use super::*;
use crate::domain_computation::primary_graph::{
    application_query::request_recipe::CapturedSourceQueryRequest,
    WorthQueryHistoryOutputReadDenial as ReadDenial,
};
use worth_query_declaration::facade::application_query::{
    ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeResolution,
};
impl WorthQueryHistoryOutputAuthority<'_, '_> {
    /// Request data posture only; this grants no discovery or source permission.
    pub fn records_discovery_recipes(&self) -> bool {
        self.restored.records_recipes()
    }
    pub(in crate::domain_computation::primary_graph) fn discovery_request<S, B, C>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        parent: (
            crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
            [u8; 32],
        ),
        intent: Option<B::Input>,
    ) -> Result<Arc<CapturedSourceQueryRequest>, ReadDenial>
    where
        S: ApplicationSchema,
        B: ApplicationQueryBinding<S>,
        C: 'static,
        B::Input: ApplicationQueryIntent<S, Binding = B>,
        B::ScopeBinding: ApplicationQueryScopeResolution<S, B::PrincipalIdentity>,
    {
        self.validate(self.phase, runtime, self.request())
            .map_err(ReadDenial::Authority)?;
        let step = self.restored.recipe_step();
        let result = self.restored.discovery_request(execution, &self.profile.model,
            super::super::super::discovery_recipes::HistoryDiscoveryRecipeKey::new::<C>(&self.profile, parent),
            || CapturedSourceQueryRequest::capture::<S, B>(execution, runtime, step,
                intent.ok_or(ReadDenial::Authority(WorthQueryHistoryNavigationDenial::IntentDrift))?),
        ).and_then(|recipe| {
            let installed = runtime.installed_schema().installed_query_binding::<B>()
                .map_err(|cause| ReadDenial::Demand(crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial::new(
                    crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::SourceQueryInstallation(cause.kind()), B::IDENTITY)))?;
            if !recipe.matches_binding::<S, B>(&installed) { return Err(ReadDenial::Authority(WorthQueryHistoryNavigationDenial::DiscoveryRecipeCorrupt {step})); }
            Ok(recipe)
        });
        // Even a data refusal is delivered only while the original C control
        // remains current. It never discloses a foreign document or source.
        self.confirm_read(self.phase, runtime, self.request(), result)
            .map_err(ReadDenial::Authority)?
    }
}
