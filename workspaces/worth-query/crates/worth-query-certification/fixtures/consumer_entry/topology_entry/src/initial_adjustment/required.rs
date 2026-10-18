//! Fixed output custody for the existing declared no-source initial adjustment.
use super::*;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationRequiredOutputConnection, WorthQueryApplicationRequiredOutputSource,
    WorthQueryRequiredOutputConnectionDenial,
};

impl<Schema: TopologySchemaBinding> WorthQueryApplicationRequiredOutputConnection<Schema>
    for PlanarInitialToOutputConnection
{
    type Source = PlanarInitialAdjustmentBinding<Schema>;
    type Demand = PlanarOutputDemand;
    const IDENTITY: &'static str = "worth.query.certification.planar-initial-to-output.v1";
    fn demand_from_source(
        source: &PlanarInitialAdjustment,
    ) -> Result<Self::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(PlanarOutputDemand::new(&source.scope_key))
    }
}
impl<Schema: TopologySchemaBinding>
    WorthQueryApplicationRequiredOutputSource<Schema, PlanarInitialToOutputConnection>
    for PlanarInitialAdjustmentBinding<Schema>
{
    fn demand_from_source(
        source: &PlanarInitialAdjustment,
    ) -> Result<PlanarOutputDemand, WorthQueryRequiredOutputConnectionDenial> {
        <PlanarInitialToOutputConnection as WorthQueryApplicationRequiredOutputConnection<Schema>>::demand_from_source(source)
    }
}
