use super::{
    PlanarHandler, PlanarMutationBinding, PositivePlanarTurn, TopologyContribution,
    TopologySchemaBinding,
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
        WorthQueryApplicationContribution, WorthQueryApplicationContributionSetup,
    },
    primary_graph::WorthQueryPrimaryGraphInstallationDenial,
};

pub struct TopologyConfiguration {
    pub setup_calls: Arc<AtomicUsize>,
    pub invariant_calls: Arc<AtomicUsize>,
}

impl<Schema: TopologySchemaBinding> WorthQueryApplicationContribution<Schema>
    for TopologyContribution
{
    type Configuration = TopologyConfiguration;

    fn configure(
        configuration: Self::Configuration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        configuration.setup_calls.fetch_add(1, Ordering::SeqCst);
        setup.invariant(
            PositivePlanarTurn::reference(),
            ApplicationInvariantExecutionPoint::CommitBoundary,
            move |resolver| {
                super::planar_invariant::resolve_rule(resolver, configuration.invariant_calls)
            },
        )?;
        setup.handler::<PlanarMutationBinding<Schema>, _>(PlanarHandler)?;
        setup.handler::<super::VertexReplacementBinding<Schema>, _>(super::VertexReplacementHandler)
    }
}
