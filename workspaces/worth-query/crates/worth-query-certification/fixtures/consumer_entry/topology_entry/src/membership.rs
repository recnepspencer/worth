//! Additive membership-edit capability, installed only by its separate fixture schema.
use super::TopologySchemaBinding;
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
pub struct MembershipContribution;
impl<Schema: MembershipSchemaBinding>
    worth_query_decl::facade::application_schema::ApplicationSchemaContribution<Schema>
    for MembershipContribution
{
    const IDENTITY:
        worth_query_decl::facade::application_schema::ApplicationSchemaContributionIdentity =
        worth_query_decl::facade::application_schema::ApplicationSchemaContributionIdentity::new(
            "worth.query.certification.membership-edit.v1",
        );
    type Members = (MembershipContributionContracts,);
}
impl MembershipContribution {
    pub fn reference<Schema: MembershipSchemaBinding>(
    ) -> worth_query_decl::facade::application_schema::ApplicationSchemaContributionRef<Schema, Self>
    {
        worth_query_decl::facade::application_schema::ApplicationSchemaContributionRef::from_contribution()
    }
}
pub struct MembershipContributionContracts;
impl<Schema: MembershipSchemaBinding>
    worth_query_decl::facade::application_schema::fragment::Registration<Schema>
    for MembershipContributionContracts
{
    fn register(
        schema: worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<
            Schema,
        >,
    ) -> Result<
        worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<Schema>,
        worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        Ok(link::declare_planar_discovery_link(schema))
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
