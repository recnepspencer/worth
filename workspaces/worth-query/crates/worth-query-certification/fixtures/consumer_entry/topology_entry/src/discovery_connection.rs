//! Dependent membership discovery through the same governed query as root discovery.
use super::*;
use worth_query_decl::facade::application_program::{
    ApplicationConnectionIdentity, ApplicationOccurrenceConnectionBinding,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationDependentOutputConnection, WorthQueryRequiredOutputConnectionDenial,
};
pub struct PlanarOutputToMembershipConnection;
impl ApplicationConnectionIdentity for PlanarOutputToMembershipConnection {
    const IDENTITY: &'static str = "worth.query.certification.planar-output-to-membership.v1";
}
impl<S: TopologySchemaBinding>
    ApplicationOccurrenceConnectionBinding<S, PlanarOutputFeature, PlanarFinalOutputFeature>
    for PlanarOutputToMembershipConnection
{
}
impl<S: TopologySchemaBinding> WorthQueryApplicationDependentOutputConnection<S>
    for PlanarOutputToMembershipConnection
{
    type RootDemand = PlanarOutputDemand;
    type Discovery = PlanarDiscoveryRead;
    type Demand = PlanarFinalOutputDemand;
    const IDENTITY: &'static str = "worth.query.certification.planar-output-to-membership.v1";
    fn discovery_from_root(
        root: &Self::RootDemand,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        Ok(PlanarDiscoveryRead {
            body_key: root.body_key().to_owned(),
        })
    }
    fn demands_from_discovery(
        discovery: &PlanarDiscoveryResult,
    ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        Ok(discovery
            .source_body_keys
            .iter()
            .map(PlanarFinalOutputDemand::new)
            .collect())
    }
}
