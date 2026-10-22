use super::{
    InitialPlanarProducer, InitialPlanarReadiness, PlanarHandler, PlanarMutationBinding,
    PositivePlanarTurn, TopologyContribution, TopologySchemaBinding,
};
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc,
};
use worth_query_decl::facade::application_schema::{
    ApplicationInvariantExecutionPoint, ApplicationInvariantMarkerIdentity,
};
use worth_query_host::facade::{
    application_contribution::{
        WorthQueryApplicationContribution, WorthQueryApplicationContributionContracts,
        WorthQueryApplicationContributionSetup,
    },
    primary_graph::WorthQueryPrimaryGraphInstallationDenial,
};

pub struct TopologyConfiguration {
    pub final_feature: super::PlanarFinalFeatureRuntime,
    pub setup_calls: Arc<AtomicUsize>,
    pub invariant_calls: Arc<AtomicUsize>,
    pub invariant_probe: Arc<AtomicUsize>,
    pub producer_authorization_denials: Arc<AtomicUsize>,
    pub producer_domain_denial: Arc<AtomicBool>,
}

impl<Schema: TopologySchemaBinding> WorthQueryApplicationContribution<Schema>
    for TopologyContribution
{
    type Configuration = TopologyConfiguration;

    fn contracts(
        contracts: &mut WorthQueryApplicationContributionContracts<Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        contracts.producer::<InitialPlanarProducer<Schema>>()?;
        #[cfg(test)]
        super::checkpoint_recovery::required_chain::contracts(contracts)?;
        #[cfg(test)]
        super::checkpoint_recovery::mixed_retirement::contracts(contracts)?;
        contracts.feature::<super::PlanarFinalFeatureRoles<Schema>>()?;
        contracts.producer::<super::AlternatePlanarOutputProducer<Schema>>()?;
        contracts.conditional::<InitialPlanarReadiness<Schema>>()?;
        contracts.conditional::<super::AlternatePlanarReadiness<Schema>>()?;
        Ok(())
    }

    fn configure(
        configuration: Self::Configuration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        #[cfg(test)]
        super::checkpoint_recovery::computation_partition::configure(setup)?;
        Self::configure_topology(configuration, setup)
    }
}

impl TopologyContribution {
    /// Configures the topology apart from the region totals demand, so a
    /// program can decide that one operation its own way.
    pub(crate) fn configure_topology<Schema: TopologySchemaBinding>(
        configuration: TopologyConfiguration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        let provider = super::InitialPlanarProvider::new(
            Arc::clone(&configuration.producer_authorization_denials),
            Arc::clone(&configuration.producer_domain_denial),
        );
        Self::configure_topology_with_provider(configuration, setup, provider, PlanarHandler)
    }

    pub(crate) fn configure_topology_with_provider<Schema: TopologySchemaBinding, Handler>(
        configuration: TopologyConfiguration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
        provider: super::InitialPlanarProvider,
        handler: Handler,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>
    where
        Handler: worth_query_host::facade::primary_graph::OperationHandler<
            Schema,
            PlanarMutationBinding<Schema>,
        >,
    {
        configuration.setup_calls.fetch_add(1, Ordering::SeqCst);
        #[cfg(test)]
        super::checkpoint_recovery::required_chain::configure(
            setup,
            provider.has_uniform_decimal_key_width(),
        )?;
        setup.invariant(
            PositivePlanarTurn::reference(),
            ApplicationInvariantExecutionPoint::CommitBoundary,
            move |resolver| {
                super::planar_invariant::resolve_rule(
                    resolver,
                    configuration.invariant_calls,
                    configuration.invariant_probe,
                )
            },
        )?;
        setup.handler::<PlanarMutationBinding<Schema>, _>(handler)?;
        setup.handler::<super::PlanarEditBinding<Schema>, _>(PlanarHandler)?;
        setup.handler::<super::AlternatePlanarOutputBinding<Schema>, _>(
            super::AlternatePlanarOutputHandler,
        )?;
        setup.handler::<super::PriorCycleAdjustmentBinding<Schema>, _>(
            super::PriorCycleAdjustmentHandler,
        )?;
        setup.handler::<super::PlanarSourceAdjustmentBinding<Schema>, _>(
            super::PlanarSourceAdjustmentHandler,
        )?;
        setup.handler::<super::PlanarInitialAdjustmentBinding<Schema>, _>(
            super::PlanarInitialAdjustmentHandler,
        )?;
        let final_feature = configuration.final_feature;
        #[cfg(test)]
        let final_feature = {
            let mut configured = final_feature;
            if provider.has_uniform_decimal_key_width() {
                if let Some(role) = &mut configured.initial {
                    role.provider = role
                        .provider
                        .map(|value| value.with_uniform_decimal_key_width());
                }
                if let Some(role) = &mut configured.preserve {
                    role.provider = role
                        .provider
                        .map(|value| value.with_uniform_decimal_key_width());
                }
            }
            configured
        };
        setup.producer::<InitialPlanarProducer<Schema>>(provider)?;
        setup.feature::<super::PlanarFinalFeatureRoles<Schema>>(final_feature)?;
        setup.producer::<super::AlternatePlanarOutputProducer<Schema>>(
            super::AlternatePlanarOutputProvider,
        )?;
        setup.conditional::<InitialPlanarReadiness<Schema>>(())?;
        setup.conditional::<super::AlternatePlanarReadiness<Schema>>(())?;
        setup.handler::<super::VertexReplacementBinding<Schema>, _>(
            super::VertexReplacementHandler,
        )?;
        #[cfg(test)]
        super::checkpoint_recovery::mixed_retirement::configure(setup)?;
        Ok(())
    }
}
