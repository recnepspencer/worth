//! Additive membership-edit capability, installed only by its separate fixture schema.
use super::TopologySchemaBinding;
use worth_query_decl::facade::worth_query_application_contribution;
use worth_query_host::facade::{
    application_contribution::{
        WorthQueryApplicationContribution, WorthQueryApplicationContributionSetup,
    },
    primary_graph::WorthQueryPrimaryGraphInstallationDenial,
};
mod link;
pub use link::{PlanarDiscoveryLink, PlanarDiscoveryLinkBinding, PlanarDiscoveryLinkHandler};
/// Schemas explicitly opting into the membership-edit fixture capability.
pub trait MembershipSchemaBinding: TopologySchemaBinding {}
worth_query_application_contribution! {
    pub contribution MembershipContribution for Schema: MembershipSchemaBinding {
        identity: "worth.query.certification.membership-edit.v1",
        members: |schema| { link::declare_planar_discovery_link(schema) }
    }
}
impl<Schema: MembershipSchemaBinding> WorthQueryApplicationContribution<Schema>
    for MembershipContribution
{
    type Configuration = ();
    fn configure(
        _: (),
        setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        setup.handler::<PlanarDiscoveryLinkBinding<Schema>, _>(PlanarDiscoveryLinkHandler)
    }
}
