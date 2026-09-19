use super::*;
use worth_query_host::facade::primary_graph::WorthQueryApplicationRequiredOutputSource;

impl<Schema: TopologySchemaBinding> WorthQueryApplicationRequiredOutputConnection<Schema>
    for PlanarSourceToOutputConnection
{
    type Source = PlanarSourceAdjustmentBinding<Schema>;
    type Demand = PlanarOutputDemand;

    const IDENTITY: &'static str = "worth.query.certification.planar-source-to-output.v1";

    fn demand_from_source(
        source: &PlanarSourceAdjustment,
    ) -> Result<Self::Demand, WorthQueryRequiredOutputConnectionDenial> {
        if source.scope_key.is_empty() {
            return Err(WorthQueryRequiredOutputConnectionDenial::new(
                "source occurrence key is empty",
            ));
        }
        Ok(PlanarOutputDemand::new(&source.scope_key))
    }
}

impl<Schema: TopologySchemaBinding>
    WorthQueryApplicationRequiredOutputSource<Schema, PlanarSourceToOutputConnection>
    for PlanarSourceAdjustmentBinding<Schema>
{
    fn demand_from_source(
        source: &PlanarSourceAdjustment,
    ) -> Result<PlanarOutputDemand, WorthQueryRequiredOutputConnectionDenial> {
        <PlanarSourceToOutputConnection as WorthQueryApplicationRequiredOutputConnection<
            Schema,
        >>::demand_from_source(source)
    }
}

impl<Schema: TopologySchemaBinding> WorthQueryApplicationRequiredOutputConnection<Schema>
    for PlanarSourceToRemoteOutputConnection
{
    type Source = PlanarSourceAdjustmentBinding<Schema>;
    type Demand = PlanarOutputDemand;

    const IDENTITY: &'static str = "worth.query.certification.planar-source-to-remote-output.v1";

    fn demand_from_source(
        source: &PlanarSourceAdjustment,
    ) -> Result<Self::Demand, WorthQueryRequiredOutputConnectionDenial> {
        if source.scope_key.is_empty() {
            return Err(WorthQueryRequiredOutputConnectionDenial::new(
                "source occurrence key is empty",
            ));
        }
        Ok(PlanarOutputDemand::new("remote-b"))
    }
}

impl<Schema: TopologySchemaBinding>
    WorthQueryApplicationRequiredOutputSource<Schema, PlanarSourceToRemoteOutputConnection>
    for PlanarSourceAdjustmentBinding<Schema>
{
    fn demand_from_source(
        source: &PlanarSourceAdjustment,
    ) -> Result<PlanarOutputDemand, WorthQueryRequiredOutputConnectionDenial> {
        <PlanarSourceToRemoteOutputConnection as WorthQueryApplicationRequiredOutputConnection<
            Schema,
        >>::demand_from_source(source)
    }
}

impl<Schema: TopologySchemaBinding> WorthQueryApplicationDiscoveredOutputConnection<Schema>
    for PlanarSourceToOutputConnection
{
    type Source = PlanarSourceAdjustmentBinding<Schema>;
    type Discovery = crate::PlanarDiscoveryRead;
    type Demand = PlanarOutputDemand;

    const IDENTITY: &'static str = "worth.query.certification.planar-source-to-output.v1";

    fn discovery_from_source(
        source: &PlanarSourceAdjustment,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        Ok(crate::PlanarDiscoveryRead {
            body_key: source.scope_key.clone(),
        })
    }

    fn demands_from_discovery(
        discovery: &crate::PlanarDiscoveryResult,
    ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        Ok(discovery
            .source_body_keys
            .iter()
            .map(PlanarOutputDemand::new)
            .collect())
    }
}

impl<Schema: TopologySchemaBinding> WorthQueryApplicationDependentOutputConnection<Schema>
    for PlanarOutputToFinalConnection
{
    type RootDemand = PlanarOutputDemand;
    type Discovery = crate::PlanarOutputRead;
    type Demand = crate::PlanarFinalOutputDemand;

    const IDENTITY: &'static str = "worth.query.certification.planar-output-to-final.v1";

    fn discovery_from_root(
        root: &Self::RootDemand,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        Ok(crate::PlanarOutputRead {
            body_key: root.body_key().to_owned(),
        })
    }

    fn demands_from_discovery(
        discovery: &crate::PlanarOutputReadResult,
    ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        Ok(vec![crate::PlanarFinalOutputDemand::new(
            &discovery.body_key,
        )])
    }
}

impl<Schema: TopologySchemaBinding> WorthQueryApplicationDependentOutputConnection<Schema>
    for PlanarOutputToLateFinalConnection
{
    type RootDemand = PlanarOutputDemand;
    type Discovery = PlanarRead;
    type Demand = crate::PlanarFinalOutputDemand;

    const IDENTITY: &'static str = "worth.query.certification.planar-output-to-late-final.v1";

    fn discovery_from_root(
        _: &Self::RootDemand,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        Ok(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
    }

    fn demands_from_discovery(
        discovery: &crate::PlanarReadResult,
    ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        let target = match worth_query_consumer_values::PositiveLength::get(&discovery.y) {
            0..=2 => "remote-b",
            3 => "sibling-b",
            _ => "sibling-c",
        };
        Ok(vec![crate::PlanarFinalOutputDemand::new(target)])
    }
}

impl<Schema: TopologySchemaBinding> WorthQueryApplicationDependentOutputConnection<Schema>
    for PlanarFinalToSummaryConnection
{
    type RootDemand = crate::PlanarFinalOutputDemand;
    type Discovery = PlanarRead;
    type Demand = crate::PlanarFinalOutputDemand;

    const IDENTITY: &'static str = "worth.query.certification.planar-final-to-summary.v1";

    fn discovery_from_root(
        root: &Self::RootDemand,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        Ok(PlanarRead {
            body_key: format!("final:{}", root.body_key()),
        })
    }

    fn demands_from_discovery(
        discovery: &crate::PlanarReadResult,
    ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        Ok(vec![crate::PlanarFinalOutputDemand::new(
            &discovery.body_key,
        )])
    }
}

impl<Schema: TopologySchemaBinding> WorthQueryApplicationDependentOutputConnection<Schema>
    for PlanarOutputToAlternateFinalConnection
{
    type RootDemand = PlanarOutputDemand;
    type Discovery = crate::PlanarOutputRead;
    type Demand = crate::PlanarFinalOutputDemand;

    const IDENTITY: &'static str = "worth.query.certification.planar-output-to-alternate-final.v1";

    fn discovery_from_root(
        root: &Self::RootDemand,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        Ok(crate::PlanarOutputRead {
            body_key: root.body_key().to_owned(),
        })
    }

    fn demands_from_discovery(
        discovery: &crate::PlanarOutputReadResult,
    ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        Ok(vec![crate::PlanarFinalOutputDemand::new(
            &discovery.successor_body_key,
        )])
    }
}

impl<Schema: TopologySchemaBinding> WorthQueryApplicationDependentOutputConnection<Schema>
    for PlanarAlternateFinalToSummaryConnection
{
    type RootDemand = crate::PlanarFinalOutputDemand;
    type Discovery = PlanarRead;
    type Demand = crate::PlanarFinalOutputDemand;

    const IDENTITY: &'static str = "worth.query.certification.planar-alternate-final-to-summary.v1";

    fn discovery_from_root(
        root: &Self::RootDemand,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        Ok(PlanarRead {
            body_key: format!("final:{}", root.body_key()),
        })
    }

    fn demands_from_discovery(
        discovery: &crate::PlanarReadResult,
    ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        Ok(vec![crate::PlanarFinalOutputDemand::new(
            &discovery.body_key,
        )])
    }
}
