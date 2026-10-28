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
            .field(GrantIdentity::reference(), "inspection-grant".to_owned())
            .field(GrantAction::reference(), "inspect".to_owned())
            .field(GrantPurpose::reference(), "audit".to_owned())
            .field(GrantDisclosure::reference(), "number".to_owned())
            .field(GrantActive::reference(), true)
            .field(GrantWorkflow::reference(), "account-001".to_owned())
            .field(GrantNotBefore::reference(), 0_u64)
            .field(GrantNotAfter::reference(), u64::MAX)
            .field(GrantDelegationLimit::reference(), 0_u64),
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
