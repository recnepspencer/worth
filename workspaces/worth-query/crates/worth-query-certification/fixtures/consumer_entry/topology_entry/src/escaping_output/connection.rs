//! Real fixed graph connections to the distinct external producer family.
use super::*;
use worth_query_decl::facade::application_program::*;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationDependentOutputConnection, WorthQueryApplicationRequiredOutputConnection,
    WorthQueryApplicationRequiredOutputSource, WorthQueryRequiredOutputConnectionDenial,
};

pub struct PlanarInitialToEscapingOutput;
pub struct PlanarOutputToEscapingOutput;
impl ApplicationConnectionIdentity for PlanarInitialToEscapingOutput {
    const IDENTITY: &'static str = "planar.initial-to-escaping.v1";
}
impl ApplicationConnectionIdentity for PlanarOutputToEscapingOutput {
    const IDENTITY: &'static str = "planar.output-to-escaping.v1";
}
impl<S: TopologySchemaBinding>
    ApplicationOccurrenceConnectionBinding<S, PlanarSourceFeature, PlanarOutputFeature>
    for PlanarInitialToEscapingOutput
{
}
impl<S: TopologySchemaBinding>
    ApplicationOccurrenceConnectionBinding<S, PlanarOutputFeature, PlanarFinalOutputFeature>
    for PlanarOutputToEscapingOutput
{
}
impl<S: TopologySchemaBinding> WorthQueryApplicationRequiredOutputConnection<S>
    for PlanarInitialToEscapingOutput
{
    type Source = PlanarInitialAdjustmentBinding<S>;
    type Demand = EscapingPlanarDemand;
    const IDENTITY: &'static str = "planar.initial-to-escaping.v1";
    fn demand_from_source(
        source: &PlanarInitialAdjustment,
    ) -> Result<Self::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(EscapingPlanarDemand(PlanarRead {
            body_key: source.scope_key.clone(),
        }))
    }
}
impl<S: TopologySchemaBinding>
    WorthQueryApplicationRequiredOutputSource<S, PlanarInitialToEscapingOutput>
    for PlanarInitialAdjustmentBinding<S>
{
    fn demand_from_source(
        source: &PlanarInitialAdjustment,
    ) -> Result<EscapingPlanarDemand, WorthQueryRequiredOutputConnectionDenial> {
        <PlanarInitialToEscapingOutput as WorthQueryApplicationRequiredOutputConnection<S>>::demand_from_source(source)
    }
}
impl<S: TopologySchemaBinding> WorthQueryApplicationDependentOutputConnection<S>
    for PlanarOutputToEscapingOutput
{
    type RootDemand = PlanarOutputDemand;
    type Discovery = PlanarDiscoveryRead;
    type Demand = EscapingPlanarDemand;
    const IDENTITY: &'static str = "planar.output-to-escaping.v1";
    fn discovery_from_root(
        root: &PlanarOutputDemand,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        Ok(PlanarDiscoveryRead {
            body_key: root.body_key().into(),
        })
    }
    fn demands_from_discovery(
        discovery: &PlanarDiscoveryResult,
    ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        Ok(discovery
            .source_body_keys
            .iter()
            .map(|key| {
                EscapingPlanarDemand(PlanarRead {
                    body_key: key.clone(),
                })
            })
            .collect())
    }
}
