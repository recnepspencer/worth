//! Child admission from a verified historical parent preserves ordinary custody.
use super::dependent::{DiscoveryBinding, QueryRows};
use super::*;
use crate::domain_computation::primary_graph::{
    application_query::HistoryOutputControlPlan,
    product_operation::document_history::{
        navigation::HistoryDependentParent, HistoryDependentSource,
    },
    WorthQueryApplicationReadObservation, WorthQueryHistoryDependentOutputRead,
    WorthQueryHistoryOutputReadDenial as Denial, WorthQueryVerifiedHistoryParent,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationQueryBinding;

impl<S: ApplicationSchema + 'static, P: ApplicationProgramDefinition<S>>
    WorthQueryProgramApplicationRuntime<S, P>
{
    pub fn execute_verified_history_dependent_output_query<
        'control,
        'authority,
        'phase,
        'scope,
        D,
        C,
        B,
    >(
        &'control self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        parent: &WorthQueryVerifiedHistoryParent<S, P, D>,
        authority: &'authority WorthQueryHistoryOutputAuthority<'phase, 'scope>,
        basis: &'authority std::sync::Arc<WorthQueryApplicationReadObservation>,
        minimum: &'authority WorthQueryApplicationReadObservation,
        binding: &'control WorthQueryInstalledApplicationQueryBinding<S, B>,
        external: &WorthQueryAuthenticatedExternalPrincipal<S>,
        control: HistoryOutputControlPlan<'control, S, B>,
    ) -> Result<
        WorthQueryHistoryDependentOutputRead<'authority, 'phase, 'scope, B::Query, QueryRows<S, B>>,
        Denial,
    >
    where
        D: WorthQueryApplicationOutputDemand<S>,
        C: ApplicationConnectionShape<S>,
        ConnectionBinding<S, C>: WorthQueryApplicationDependentOutputConnection<S, RootDemand = D>,
        B: ApplicationQueryBinding<S>,
        QueryRows<S, B>: WorthQueryApplicationProjection<S, B::Query>,
    {
        let runtime = self.runtime();
        runtime.validate_history_output_control(phase, authority, binding, &control)?;
        self.validate_verified_history_parent::<D, C>(parent, authority)
            .map_err(Denial::Demand)?;
        let original = parent.custody().original();
        if basis.branch_incarnation() != original.branch_incarnation()
            || minimum.branch_incarnation() != original.branch_incarnation()
            || basis.branch_identity() != original.branch_identity()
            || minimum.branch_identity() != original.branch_identity()
            || basis.selected_commit().ordinal() < original.selected_commit().ordinal()
            || basis.selected_commit().ordinal() < minimum.selected_commit().ordinal()
        {
            return Err(foreign());
        }
        let output = runtime
            .installed_schema()
            .installed_query_binding::<Source<S, ConnectionDemand<S, C>>>()
            .map_err(|cause| Denial::Demand(installation(cause.kind())))?;
        let discovery = runtime
            .installed_schema()
            .installed_query_binding::<DiscoveryBinding<S, C>>()
            .map_err(|cause| Denial::Demand(installation(cause.kind())))?;
        let executed_binding = crate::equality_type_id::EqualityTypeId::of::<B>().into_external();
        let output_binding =
            crate::equality_type_id::EqualityTypeId::of::<Source<S, ConnectionDemand<S, C>>>()
                .into_external();
        let discovery_binding =
            crate::equality_type_id::EqualityTypeId::of::<DiscoveryBinding<S, C>>().into_external();
        if !((executed_binding == output_binding
            && binding.query().identity() == output.query().identity())
            || (executed_binding == discovery_binding
                && binding.query().identity() == discovery.query().identity()))
        {
            return Err(foreign());
        }
        let selected = runtime.select_application_read_observation(phase, basis)
            .map_err(|cause| Denial::Authority(crate::domain_computation::primary_graph::WorthQueryHistoryNavigationDenial::ProductAdmission(cause)))?;
        let result = runtime.execute_history_output_query_on_selected(
            phase, authority, binding, external, control, selected,
        )?;
        Ok(WorthQueryHistoryDependentOutputRead {
            source: HistoryDependentSource {
                authority,
                parent: HistoryDependentParent::Verified(std::sync::Arc::clone(parent.custody())),
                connection: crate::equality_type_id::EqualityTypeId::of::<C>().into_external(),
                source: basis,
                minimum,
                output_binding,
                discovery_binding,
                executed_binding,
                query: binding.query().identity().clone(),
            },
            result,
        })
    }

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
fn foreign() -> Denial {
    Denial::Demand(foreign_demand())
}
fn installation(
    kind: worth_query_installation::facade::WorthQueryApplicationQueryInstallationDenialKind,
) -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::SourceQueryInstallation(kind), "verified history dependent query")
}
