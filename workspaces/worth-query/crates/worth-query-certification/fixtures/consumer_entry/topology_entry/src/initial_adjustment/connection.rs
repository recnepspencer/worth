use super::*;
use worth_query_decl::facade::application_program::{
    ApplicationConnectionIdentity, ApplicationOccurrenceConnectionBinding,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationDiscoveredOutputConnection, WorthQueryRequiredOutputConnectionDenial,
};

pub struct PlanarInitialToOutputConnection;

impl ApplicationConnectionIdentity for PlanarInitialToOutputConnection {
    const IDENTITY: &'static str = "worth.query.certification.planar-initial-to-output.v1";
}

impl<Schema: TopologySchemaBinding>
    ApplicationOccurrenceConnectionBinding<Schema, PlanarSourceFeature, PlanarOutputFeature>
    for PlanarInitialToOutputConnection
{
}

impl<Schema: TopologySchemaBinding> WorthQueryApplicationDiscoveredOutputConnection<Schema>
    for PlanarInitialToOutputConnection
{
    type Source = PlanarInitialAdjustmentBinding<Schema>;
    type Discovery = PlanarDiscoveryRead;
    type Demand = PlanarOutputDemand;

    const IDENTITY: &'static str = "worth.query.certification.planar-initial-to-output.v1";

    fn discovery_from_source(
        source: &PlanarInitialAdjustment,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        Ok(PlanarDiscoveryRead {
            body_key: source.scope_key.clone(),
        })
    }

    fn demands_from_discovery(
        discovery: &PlanarDiscoveryResult,
    ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        Ok(discovery
            .source_body_keys
            .iter()
            .map(PlanarOutputDemand::new)
            .collect())
    }
}
