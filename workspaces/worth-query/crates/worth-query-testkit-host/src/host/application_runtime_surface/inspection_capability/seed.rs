//! Native bootstrap facts for the real current grant.
use super::*;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationEntityKey, WorthQueryApplicationEntitySeed,
    WorthQueryApplicationRelationSeed, WorthQueryPrimaryGraphBootstrap,
};
pub(in super::super) fn bind(graph: &mut WorthQueryPrimaryGraphBootstrap<HostIdentitySchema>) {
    graph
        .bind_entity(
            WorthQueryApplicationEntitySeed::new(
                InspectionGrant::reference(),
                WorthQueryApplicationEntityKey::new("inspection-grant").unwrap(),
            )
            .field(
                GrantIdentity::at("GrantIdentity"),
                "inspection-grant".to_owned().into(),
            )
            .field(GrantAction::at("GrantAction"), "inspect".to_owned())
            .field(GrantPurpose::at("GrantPurpose"), "audit".to_owned())
            .field(GrantDisclosure::at("GrantDisclosure"), "number".to_owned())
            .field(GrantActive::at("GrantActive"), true)
            .field(GrantWorkflow::at("GrantWorkflow"), "account-001".to_owned())
            .field(GrantNotBefore::at("GrantNotBefore"), 0_u64)
            .field(GrantNotAfter::at("GrantNotAfter"), u64::MAX)
            .field(GrantDelegationLimit::at("GrantDelegationLimit"), 0_u64),
        )
        .unwrap();
    graph
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            Grantor::reference(),
            "inspection-grantor",
            WorthQueryApplicationEntityKey::new("host-principal").unwrap(),
            WorthQueryApplicationEntityKey::new("inspection-grant").unwrap(),
        ))
        .unwrap();
    graph
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            Grantee::reference(),
            "inspection-grantee",
            WorthQueryApplicationEntityKey::new("host-principal").unwrap(),
            WorthQueryApplicationEntityKey::new("inspection-grant").unwrap(),
        ))
        .unwrap();
    graph
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            GrantResource::reference(),
            "inspection-resource",
            WorthQueryApplicationEntityKey::new("inspection-grant").unwrap(),
            WorthQueryApplicationEntityKey::new("account-row").unwrap(),
        ))
        .unwrap();
}
