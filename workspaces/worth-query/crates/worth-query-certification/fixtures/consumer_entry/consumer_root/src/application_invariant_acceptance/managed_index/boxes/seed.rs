//! Literal authored boxes and an actual scoped active grant.
use super::super::schema::*;
use super::{access::grants::*, coordinates::Coordinates, field::ItemBounds};
use worth_query_host::facade::primary_graph::*;
pub const BOXES: [(&str, Coordinates, bool); 6] = [
    ("box-a", Coordinates::new(0, 0, 2, 2), true),
    ("box-b", Coordinates::new(2, 2, 4, 4), true),
    ("box-c", Coordinates::new(0, 0, 2, 2), true),
    ("box-d", Coordinates::new(-4, -3, -2, -1), true),
    ("box-e", Coordinates::new(1, 1, 3, 3), false),
    ("box-f", Coordinates::new(8, 8, 8, 8), true),
];
pub fn populate(
    graph: &mut WorthQueryPrimaryGraphBootstrap<Catalog>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    for (identity, bounds, enabled) in BOXES {
        seed_box(graph, "boxes", identity, bounds, enabled)?;
    }
    seed_box(
        graph,
        "invalid-boxes",
        "invalid-box",
        Coordinates::new(2, 0, 1, 1),
        true,
    )?;
    graph.bind_entity(
        WorthQueryApplicationEntitySeed::new(
            BoxGrant::reference(),
            WorthQueryApplicationEntityKey::new("box-access").unwrap(),
        )
        .field(GrantIdentity::reference(), "box-access".to_owned())
        .field(GrantAction::reference(), 1u64)
        .field(GrantPurpose::reference(), 1u64)
        .field(GrantStatus::reference(), 1u64)
        .field(GrantReadStatus::reference(), 1u64)
        .field(GrantScope::reference(), "boxes".to_owned())
        .field(GrantNotBefore::reference(), 0u64)
        .field(GrantNotAfter::reference(), u64::MAX)
        .field(GrantDepth::reference(), 0u64)
        .field(GrantDisclosure::reference(), 1u64),
    )?;
    graph.bind_relation(WorthQueryApplicationRelationSeed::new(
        GrantResource::reference(),
        "box-grant-resource",
        WorthQueryApplicationEntityKey::new("box-access").unwrap(),
        WorthQueryApplicationEntityKey::new("boxes").unwrap(),
    ))?;
    graph.bind_relation(WorthQueryApplicationRelationSeed::new(
        GrantGrantee::reference(),
        "box-grant-grantee",
        WorthQueryApplicationEntityKey::new("catalog-reader").unwrap(),
        WorthQueryApplicationEntityKey::new("box-access").unwrap(),
    ))?;
    graph.bind_relation(WorthQueryApplicationRelationSeed::new(
        GrantGrantor::reference(),
        "box-grant-grantor",
        WorthQueryApplicationEntityKey::new("catalog-reader").unwrap(),
        WorthQueryApplicationEntityKey::new("box-access").unwrap(),
    ))?;
    Ok(())
}
fn seed_box(
    graph: &mut WorthQueryPrimaryGraphBootstrap<Catalog>,
    directory: &str,
    identity: &str,
    bounds: Coordinates,
    enabled: bool,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    let key = WorthQueryApplicationEntityKey::new(identity).unwrap();
    graph.bind_entity(
        WorthQueryApplicationEntitySeed::new(
            Item::reference(),
            WorthQueryApplicationEntityKey::new(identity).unwrap(),
        )
        .field(ItemIdentity::reference(), identity.to_owned())
        .field(ItemName::reference(), identity.to_owned())
        .field(ItemEnabled::reference(), enabled)
        .field(ItemBounds::reference(), bounds),
    )?;
    graph.bind_relation(WorthQueryApplicationRelationSeed::new(
        Contains::reference(),
        format!("contains-{identity}"),
        WorthQueryApplicationEntityKey::new(directory).unwrap(),
        key,
    ))
}
