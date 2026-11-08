//! Child admission from a verified historical parent preserves ordinary custody.
use super::*;
use crate::domain_computation::primary_graph::{
    product_operation::document_history::navigation::HistoryDependentParent,
    WorthQueryHistoryDependentOutputRead, WorthQueryVerifiedHistoryParent,
};

impl<S: ApplicationSchema + 'static, P: ApplicationProgramDefinition<S>>
    WorthQueryProgramApplicationRuntime<S, P>
{
    pub fn admit_verified_history_program_dependent_output<D, C>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        parent: &WorthQueryVerifiedHistoryParent<S, P, D>,
        read: WorthQueryHistoryDependentOutputRead<
            '_,
            '_,
            '_,
            SourceQuery<S, ConnectionDemand<S, C>>,
            SourceValue<S, ConnectionDemand<S, C>>,
        >,
        limits: crate::domain_computation::execution_runtime::WorthQueryOutputDemandLimits,
    ) -> Result<
        WorthQueryAdmittedHistoryProgramOutput<S, P, ConnectionDemand<S, C>>,
        WorthQueryHistoryOutputAdvanceDenial,
    >
    where
        D: WorthQueryApplicationOutputDemand<S>,
        C: ApplicationConnectionShape<S>,
        ConnectionBinding<S, C>: WorthQueryApplicationDependentOutputConnection<S, RootDemand = D>,
    {
        use WorthQueryHistoryOutputAdvanceDenial as Stop;
        let source = read.source;
        let authority = source.authority;
        authority
            .validate(phase, self.runtime(), authority.request())
            .map_err(Stop::Authority)?;
        let artifact = self
            .validate_verified_history_parent::<D, C>(parent, authority)
            .map_err(Stop::Demand)?;
        if !source
            .parent
            .same_custody(&HistoryDependentParent::Verified(std::sync::Arc::clone(
                parent.custody(),
            )))
            || source.connection
                != crate::equality_type_id::EqualityTypeId::of::<C>().into_external()
            || source.executed_binding != source.output_binding
            || source.output_binding
                != crate::equality_type_id::EqualityTypeId::of::<Source<S, ConnectionDemand<S, C>>>(
                )
                .into_external()
        {
            return Err(Stop::Demand(foreign_demand()));
        }
        let disclosed = read
            .result
            .into_admitted_disclosed()
            .into_output_demand_source();
        let disclosed = authority
            .confirm_read(phase, self.runtime(), authority.request(), disclosed)
            .map_err(Stop::Authority)?;
        let execution = phase.checked_execution_request_for(self.runtime().product_runtime())
            .map_err(|cause| Stop::Demand(WorthQueryOutputDemandDenial::new(crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::ExecutionRequest(cause), "verified history child")))?;
        let selected = self
            .runtime()
            .select_application_read_observation_with_request(&execution, source.source)
            .map_err(|cause| {
                Stop::Demand(WorthQueryOutputDemandDenial::product_selection(
                    cause,
                    "verified history child",
                ))
            })?;
        if disclosed.rows().len() != 1
            || disclosed.observed_sources().len() != 1
            || disclosed.observed_sources()[0].selected_product_commit()
                != Some(source.source.selected_commit())
            || disclosed.observed_sources()[0].selected_product_occurrence()
                != Some(source.source.branch_incarnation())
        {
            return Err(Stop::Demand(foreign_demand()));
        }
        let admitted = self
            .runtime()
            .admit_dependent_output_demand::<Family<S, ConnectionDemand<S, C>>>(
                &execution,
                disclosed,
                self.runtime()
                    .output_demand_resource_profile()
                    .constrain(limits),
                selected.retain_application_read(),
            )
            .map_err(Stop::Demand)?;
        Ok(WorthQueryAdmittedHistoryProgramOutput {
            admitted: WorthQueryAdmittedProgramOutput {
                admitted,
                original_root_completion: None,
                target_feature: crate::equality_type_id::EqualityTypeId::of::<C::TargetFeature>(),
                artifact,
                marker: std::marker::PhantomData,
            },
            profile: parent.custody().profile().clone(),
            original: std::sync::Arc::clone(parent.custody().original()),
        })
    }
}
fn foreign_demand() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::ForeignSource,
        "verified history parent connection",
    )
}
