//! Explicit owner-only discovery; navigation supplies scope facts, not READ permission.
use crate::document_history::{PlanarNavigationGrantee, PlanarNavigationResource};
use crate::*;
use worth_query_decl::facade::{application_program::*, application_schema::*};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationDependentOutputConnection, WorthQueryRequiredOutputConnectionDenial,
};

mod query;
pub use query::{OwnerDiscoveryBinding, OwnerDiscoveryQuery, OwnerDiscoveryRead};

pub struct OwnerOnlyDiscovery;
impl OwnerOnlyDiscovery {
    pub fn reference<S: TopologySchemaBinding>() -> ApplicationAbilityRef<S, Self, Body> {
        ApplicationAbilityRef::from_schema_identifiers("OwnerOnlyDiscovery", "Body")
    }
}
pub struct OwnerDiscoveryPolicy;
impl OwnerDiscoveryPolicy {
    fn reference<S: TopologySchemaBinding>() -> ApplicationPolicyRef<S, Self> {
        ApplicationPolicyRef::from_schema_identifier("OwnerDiscoveryPolicy")
    }
}

pub(crate) fn declare<S: TopologySchemaBinding>(
    schema: ApplicationSchemaDeclarationBuilder<S>,
) -> ApplicationSchemaDeclarationBuilder<S> {
    schema
        .ability(OwnerOnlyDiscovery::reference())
        .policy(OwnerDiscoveryPolicy::reference())
        .ability_policy(
            OwnerOnlyDiscovery::reference(),
            OwnerDiscoveryPolicy::reference(),
            [
                ApplicationAuthorizationPathBuilder::from_principal(Principal::reference::<S>())
                    .where_equal(
                        PrincipalIdentity::reference::<S>(),
                        ApplicationEncodedScalarValue::<U64ApplicationValueBinding>::try_new(1)
                            .unwrap(),
                    )
                    .forward(PlanarNavigationGrantee::reference::<S>())
                    .forward(PlanarNavigationResource::reference::<S>())
                    .allow(Body::reference::<S>()),
            ],
        )
        .application_query(query::definition::<S>())
        .application_query_binding::<OwnerDiscoveryBinding<S>>()
}

pub struct PlanarOutputToOwnerFinalConnection;
impl ApplicationConnectionIdentity for PlanarOutputToOwnerFinalConnection {
    const IDENTITY: &'static str = "worth.query.certification.planar-output-to-owner-final.v1";
}
impl<S: TopologySchemaBinding>
    ApplicationOccurrenceConnectionBinding<S, PlanarOutputFeature, PlanarFinalOutputFeature>
    for PlanarOutputToOwnerFinalConnection
{
}
impl<S: TopologySchemaBinding> WorthQueryApplicationDependentOutputConnection<S>
    for PlanarOutputToOwnerFinalConnection
{
    type RootDemand = PlanarOutputDemand;
    type Discovery = OwnerDiscoveryRead;
    type Demand = PlanarFinalOutputDemand;
    const IDENTITY: &'static str = "worth.query.certification.planar-output-to-owner-final.v1";
    fn discovery_from_root(
        root: &Self::RootDemand,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        Ok(OwnerDiscoveryRead {
            body_key: root.body_key().to_owned(),
        })
    }
    fn demands_from_discovery(
        discovery: &PlanarReadResult,
    ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        <PlanarOutputToLateFinalConnection as WorthQueryApplicationDependentOutputConnection<S>>
            ::demands_from_discovery(discovery)
    }
}
