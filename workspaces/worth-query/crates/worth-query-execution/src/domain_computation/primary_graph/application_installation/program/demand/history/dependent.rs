//! Read and admit a child only through its original history parent connection.
use super::*;
use crate::domain_computation::primary_graph::{
    application_query::HistoryOutputControlPlan,
    product_operation::document_history::HistoryDependentSource,
    WorthQueryApplicationReadObservation, WorthQueryHistoryDependentOutputRead,
    WorthQueryHistoryOutputReadDenial,
};
use worth_query_declaration::facade::application_query::ApplicationQueryIntent;
use worth_query_installation::facade::WorthQueryInstalledApplicationQueryBinding;

type Discovery<S, C> =
    <ConnectionBinding<S, C> as WorthQueryApplicationDependentOutputConnection<S>>::Discovery;
pub(super) type DiscoveryBinding<S, C> = <Discovery<S, C> as ApplicationQueryIntent<S>>::Binding;
pub(super) type QueryRows<S, B> =
    <<B as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value;

impl<S: ApplicationSchema + 'static, P: ApplicationProgramDefinition<S>>
    WorthQueryProgramApplicationRuntime<S, P>
{
    pub fn execute_history_dependent_output_query<'control, 'authority, 'phase, 'scope, D, C, B>(
        &'control self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        parent: &'authority WorthQuerySettledHistoryProgramOutput<S, P, D>,
        authority: &'authority WorthQueryHistoryOutputAuthority<'phase, 'scope>,
        basis: &'authority std::sync::Arc<WorthQueryApplicationReadObservation>,
        minimum: &'authority WorthQueryApplicationReadObservation,
        binding: &'control WorthQueryInstalledApplicationQueryBinding<S, B>,
        external: &WorthQueryAuthenticatedExternalPrincipal<S>,
        control: HistoryOutputControlPlan<'control, S, B>,
    ) -> Result<
        WorthQueryHistoryDependentOutputRead<'authority, 'phase, 'scope, B::Query, QueryRows<S, B>>,
        WorthQueryHistoryOutputReadDenial,
    >
    where
        D: WorthQueryApplicationOutputDemand<S>,
        C: ApplicationConnectionShape<S>,
        ConnectionBinding<S, C>: WorthQueryApplicationDependentOutputConnection<S, RootDemand = D>,
        B: ApplicationQueryBinding<S>,
        QueryRows<S, B>: WorthQueryApplicationProjection<S, B::Query>,
    {
        use WorthQueryHistoryOutputReadDenial as Denial;
        let runtime = self.runtime();
        // This consumes real C READ admission before parent/basis lookup. Its
        // captured parameters, principal and full-GEN scope are never rebuilt.
        runtime.validate_history_output_control(phase, authority, binding, &control)?;
        if !parent.profile.same_custody(authority.profile())
            || !parent.profile.matches_program(self)
            || !std::sync::Arc::ptr_eq(&parent.original, authority.original())
            || basis.branch_incarnation() != parent.original.branch_incarnation()
        {
            return Err(Denial::Authority(crate::domain_computation::primary_graph::WorthQueryHistoryNavigationDenial::ForeignDocument));
        }
        self.validate_dependent_output_parent::<D, C>(&parent.settled)
            .map_err(Denial::Demand)?;
        let schema = runtime.installed_schema();
        let output = schema
            .installed_query_binding::<Source<S, ConnectionDemand<S, C>>>()
            .map_err(|stop| Denial::Demand(installation_denial(stop.kind())))?;
        let discovery = schema
            .installed_query_binding::<DiscoveryBinding<S, C>>()
            .map_err(|stop| Denial::Demand(installation_denial(stop.kind())))?;
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
            return Err(Denial::Demand(foreign_source()));
        }
        let execution = phase.checked_execution_request_for(&runtime.runtime.product_runtime)
            .map_err(|cause| Denial::Authority(crate::domain_computation::primary_graph::WorthQueryHistoryNavigationDenial::Stopped(cause)))?;
        let retained = self
            .select_dependent_output_basis(&execution, &parent.settled, basis, minimum)
            .map_err(Denial::Demand)?;
        let query = binding.query().identity().clone();
        let result = runtime.execute_history_output_query_on_selected(
            phase, authority, binding, external, control, retained,
        )?;
        Ok(WorthQueryHistoryDependentOutputRead {
            source: HistoryDependentSource {
                authority,
                parent: &parent.settled.retained,
                connection: crate::equality_type_id::EqualityTypeId::of::<C>().into_external(),
                source: basis,
                minimum,
                output_binding,
                discovery_binding,
                executed_binding,
                query,
            },
            result,
        })
    }

    pub fn admit_history_program_dependent_output<D, C>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        parent: &WorthQuerySettledHistoryProgramOutput<S, P, D>,
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
        use WorthQueryHistoryOutputAdvanceDenial as Denial;
        let source = read.source;
        source
            .authority
            .validate(phase, self.runtime(), source.authority.request())
            .map_err(Denial::Authority)?;
        let installed = self
            .runtime()
            .installed_schema()
            .installed_query_binding::<Source<S, ConnectionDemand<S, C>>>()
            .map_err(|stop| Denial::Demand(installation_denial(stop.kind())))?;
        if !parent.profile.matches_program(self)
            || !parent.profile.same_custody(source.authority.profile())
            || !std::sync::Arc::ptr_eq(&parent.original, source.authority.original())
            || !std::sync::Arc::ptr_eq(source.parent, &parent.settled.retained)
            || source.connection
                != crate::equality_type_id::EqualityTypeId::of::<C>().into_external()
            || source.executed_binding != source.output_binding
            || source.output_binding
                != crate::equality_type_id::EqualityTypeId::of::<Source<S, ConnectionDemand<S, C>>>(
                )
                .into_external()
            || &source.query != installed.query().identity()
        {
            return Err(Denial::Demand(foreign_source()));
        }
        let disclosed = read
            .result
            .into_admitted_disclosed()
            .into_output_demand_source();
        let disclosed = source
            .authority
            .confirm_read(phase, self.runtime(), source.authority.request(), disclosed)
            .map_err(Denial::Authority)?;
        let admitted = self
            .admit_program_dependent_output::<D, C>(
                phase,
                access,
                &parent.settled,
                source.source,
                source.minimum,
                disclosed,
                limits,
            )
            .map_err(Denial::Demand)?;
        Ok(WorthQueryAdmittedHistoryProgramOutput {
            admitted,
            profile: parent.profile.clone(),
            original: std::sync::Arc::clone(&parent.original),
        })
    }
}

fn installation_denial(
    kind: worth_query_installation::facade::WorthQueryApplicationQueryInstallationDenialKind,
) -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::SourceQueryInstallation(kind), "history dependent query")
}

fn foreign_source() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::ForeignSource,
        "history output read does not belong to this installed parent connection",
    )
}
