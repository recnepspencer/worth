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
        .field(GrantIdentity::reference(), "watch-access".to_owned())
        .field(GrantAction::reference(), 1u64)
        .field(GrantPurpose::reference(), 1u64)
        .field(GrantStatus::reference(), 1u64)
        .field(GrantReadStatus::reference(), 1u64)
        .field(GrantScope::reference(), "catalog".to_owned())
        .field(GrantNotBefore::reference(), 0u64)
        .field(GrantNotAfter::reference(), u64::MAX)
        .field(GrantDepth::reference(), 0u64)
        .field(GrantDisclosure::reference(), 1u64),
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
