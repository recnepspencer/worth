use super::{
    InitialPlanarProducer, InitialPlanarReadiness, PlanarHandler, PlanarMutationBinding,
    PositivePlanarTurn, TopologyContribution, TopologySchemaBinding,
};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
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
}

impl<Schema: TopologySchemaBinding> WorthQueryApplicationContribution<Schema>
    for TopologyContribution
{
    type Configuration = TopologyConfiguration;

    fn contracts(
        contracts: &mut WorthQueryApplicationContributionContracts<Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        contracts.producer::<InitialPlanarProducer<Schema>>()?;
        contracts.producer::<super::PlanarFinalOutputProducer<Schema>>()?;
        contracts.producer::<super::AlternatePlanarOutputProducer<Schema>>()?;
        contracts.conditional::<InitialPlanarReadiness<Schema>>()?;
        contracts.conditional::<super::PlanarFinalOutputReadiness<Schema>>()?;
        contracts.conditional::<super::AlternatePlanarReadiness<Schema>>()?;
        Ok(())
    }

    fn configure(
        configuration: Self::Configuration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        configuration.setup_calls.fetch_add(1, Ordering::SeqCst);
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
        setup.handler::<super::AlternatePlanarOutputBinding<Schema>, _>(
            super::AlternatePlanarOutputHandler,
        )?;
        setup.handler::<super::PriorCycleAdjustmentBinding<Schema>, _>(
            super::PriorCycleAdjustmentHandler,
        )?;
        setup.handler::<super::PlanarSourceAdjustmentBinding<Schema>, _>(
            super::PlanarSourceAdjustmentHandler,
        )?;
        setup.producer::<InitialPlanarProducer<Schema>>(super::InitialPlanarProvider::new(
            configuration.producer_authorization_denials,
        ))?;
        setup.producer::<super::PlanarFinalOutputProducer<Schema>>(
            super::PlanarFinalOutputProvider,
        )?;
        setup.producer::<super::AlternatePlanarOutputProducer<Schema>>(
            super::AlternatePlanarOutputProvider,
        )?;
        setup.conditional::<InitialPlanarReadiness<Schema>>(())?;
        setup.conditional::<super::PlanarFinalOutputReadiness<Schema>>(())?;
        setup.conditional::<super::AlternatePlanarReadiness<Schema>>(())?;
        setup.handler::<super::VertexReplacementBinding<Schema>, _>(super::VertexReplacementHandler)
    }
}
