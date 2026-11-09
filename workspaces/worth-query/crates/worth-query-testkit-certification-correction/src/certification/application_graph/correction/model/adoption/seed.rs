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
                .field(
                    GrantIdentity::at("GrantIdentity"),
                    "adoption-grant".to_owned(),
                )
                .field(GrantAction::at("GrantAction"), "adopt-program".to_owned())
                .field(
                    GrantPurpose::at("GrantPurpose"),
                    "numeric-administration".to_owned(),
                )
                .field(
                    super::super::causal_inspection::capability::Disclosure::at("Disclosure"),
                    "numeric-administration".to_owned(),
                )
                .field(GrantStatus::at("GrantStatus"), "active".to_owned())
                .field(GrantScopeKey::at("GrantScopeKey"), "numeric".to_owned())
                .field(GrantNotBefore::at("GrantNotBefore"), 0_u64)
                .field(GrantNotAfter::at("GrantNotAfter"), u64::MAX)
                .field(GrantDepth::at("GrantDepth"), 0_u64),
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
pub fn key<E>(name: &str) -> WorthQueryApplicationEntityKey<NumericSchema, E> {
    WorthQueryApplicationEntityKey::new(name).unwrap()
}
