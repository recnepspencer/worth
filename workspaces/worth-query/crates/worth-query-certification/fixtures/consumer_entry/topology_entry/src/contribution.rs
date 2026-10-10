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
        contracts.producer::<super::PlanarFinalOutputProducer<Schema>>()?;
        contracts.producer::<super::PlanarFinalPreserveProducer<Schema>>()?;
        contracts.producer::<super::AlternatePlanarOutputProducer<Schema>>()?;
        contracts.conditional::<InitialPlanarReadiness<Schema>>()?;
        contracts.conditional::<super::PlanarFinalOutputReadiness<Schema>>()?;
        contracts.conditional::<super::PlanarFinalPreserveReadiness<Schema>>()?;
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
        Self::configure_topology_with_provider(configuration, setup, provider)
    }

    pub(crate) fn configure_topology_with_provider<Schema: TopologySchemaBinding>(
        configuration: TopologyConfiguration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
        provider: super::InitialPlanarProvider,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        configuration.setup_calls.fetch_add(1, Ordering::SeqCst);
        #[cfg(test)]
        super::checkpoint_recovery::required_chain::configure(setup)?;
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
        setup.handler::<PlanarMutationBinding<Schema>, _>(PlanarHandler)?;
        setup.handler::<super::PlanarEditBinding<Schema>, _>(PlanarHandler)?;
        setup.handler::<super::FinalPlanarMutationBinding<Schema>, _>(
            super::FinalPlanarMutationHandler,
        )?;
        setup.handler::<super::FinalPlanarPreserveBinding<Schema>, _>(
            super::FinalPlanarPreserveHandler,
        )?;
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
        let final_provider = super::PlanarFinalOutputProvider::default();
        #[cfg(test)]
        let final_provider = if provider.has_uniform_decimal_key_width() {
            final_provider.with_uniform_decimal_key_width()
        } else {
            final_provider
        };
        setup.producer::<InitialPlanarProducer<Schema>>(provider)?;
        setup.producer::<super::PlanarFinalOutputProducer<Schema>>(final_provider)?;
        setup.producer::<super::PlanarFinalPreserveProducer<Schema>>(final_provider)?;
        setup.producer::<super::AlternatePlanarOutputProducer<Schema>>(
            super::AlternatePlanarOutputProvider,
        )?;
        setup.conditional::<InitialPlanarReadiness<Schema>>(())?;
        setup.conditional::<super::PlanarFinalOutputReadiness<Schema>>(())?;
        setup.conditional::<super::PlanarFinalPreserveReadiness<Schema>>(())?;
        setup.conditional::<super::AlternatePlanarReadiness<Schema>>(())?;
        setup.handler::<super::VertexReplacementBinding<Schema>, _>(
            super::VertexReplacementHandler,
        )?;
        #[cfg(test)]
        super::checkpoint_recovery::mixed_retirement::configure(setup)?;
        Ok(())
    }
}
