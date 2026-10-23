//! Bootstrap the real adoption grant and its principal/resource edges.
use super::*;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationEntityKey, WorthQueryApplicationEntitySeed,
    WorthQueryApplicationRelationSeed, WorthQueryPrimaryGraphBootstrap,
};
pub fn seed(graph: &mut WorthQueryPrimaryGraphBootstrap<NumericSchema>) {
    graph
        .bind_entity(
            WorthQueryApplicationEntitySeed::new(Grant::reference(), key("adoption-grant"))
                .field(GrantIdentity::reference(), "adoption-grant".to_owned())
                .field(GrantAction::reference(), "adopt-program".to_owned())
                .field(
                    GrantPurpose::reference(),
                    "numeric-administration".to_owned(),
                )
                .field(GrantStatus::reference(), "active".to_owned())
                .field(GrantScopeKey::reference(), "numeric".to_owned())
                .field(GrantNotBefore::reference(), 0_u64)
                .field(GrantNotAfter::reference(), u64::MAX)
                .field(GrantDepth::reference(), 0_u64),
        )
        .unwrap();
    graph
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            GrantResource::reference(),
            "adoption-resource",
            key::<Grant>("adoption-grant"),
            key::<Number>("numeric"),
        ))
        .unwrap();
    graph
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            GrantGrantor::reference(),
            "adoption-grantor",
            key::<Principal>("numeric-operator"),
            key::<Grant>("adoption-grant"),
        ))
        .unwrap();
    graph
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            GrantGrantee::reference(),
            "adoption-grantee",
            key::<Principal>("numeric-operator"),
            key::<Grant>("adoption-grant"),
        ))
        .unwrap();
}
fn key<E>(name: &str) -> WorthQueryApplicationEntityKey<NumericSchema, E> {
    WorthQueryApplicationEntityKey::new(name).unwrap()
}
