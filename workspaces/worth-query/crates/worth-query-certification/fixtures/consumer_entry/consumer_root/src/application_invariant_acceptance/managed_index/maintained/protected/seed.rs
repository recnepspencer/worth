//! One literal active capability and independent READ grant on the actual root.
use super::{super::schema::*, access::grants::*};
use worth_query_host::facade::primary_graph::*;
pub(in super::super) fn populate(
    graph: &mut WorthQueryPrimaryGraphBootstrap<Catalog>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    graph.bind_entity(
        WorthQueryApplicationEntitySeed::new(
            BoxGrant::reference(),
            WorthQueryApplicationEntityKey::new("watch-access").unwrap(),
        )
        .field(
            GrantIdentity::at("GrantIdentity"),
            "watch-access".to_owned(),
        )
        .field(GrantAction::at("GrantAction"), 1u64)
        .field(GrantPurpose::at("GrantPurpose"), 1u64)
        .field(GrantStatus::at("GrantStatus"), 1u64)
        .field(GrantReadStatus::at("GrantReadStatus"), 1u64)
        .field(GrantScope::at("GrantScope"), "catalog".to_owned())
        .field(GrantNotBefore::at("GrantNotBefore"), 0u64)
        .field(GrantNotAfter::at("GrantNotAfter"), u64::MAX)
        .field(GrantDepth::at("GrantDepth"), 0u64)
        .field(GrantDisclosure::at("GrantDisclosure"), 1u64),
    )?;
    graph.bind_relation(WorthQueryApplicationRelationSeed::new(
        GrantResource::reference(),
        "watch-grant-resource",
        WorthQueryApplicationEntityKey::new("watch-access").unwrap(),
        WorthQueryApplicationEntityKey::new("catalog").unwrap(),
    ))?;
    graph.bind_relation(WorthQueryApplicationRelationSeed::new(
        GrantGrantee::reference(),
        "watch-grant-grantee",
        WorthQueryApplicationEntityKey::new("catalog-reader").unwrap(),
        WorthQueryApplicationEntityKey::new("watch-access").unwrap(),
    ))?;
    graph.bind_relation(WorthQueryApplicationRelationSeed::new(
        GrantGrantor::reference(),
        "watch-grant-grantor",
        WorthQueryApplicationEntityKey::new("catalog-reader").unwrap(),
        WorthQueryApplicationEntityKey::new("watch-access").unwrap(),
    ))?;
    Ok(())
}
