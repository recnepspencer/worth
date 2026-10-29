pub(crate) use std::collections::BTreeMap;

pub(crate) use super::scale::SupplyChainScale;
pub(crate) use super::schema::RelationEdge;
pub(crate) use super::semantic_key::{Anchor, EntityKey, EntityKind, RelationKey, RelationKind};

pub(crate) fn build(scale: SupplyChainScale) -> BTreeMap<RelationKey, RelationEdge> {
    let mut relations = BTreeMap::new();
    for ordinal in 0..scale.terminals {
        add_edge(
            &mut relations,
            RelationKind::TerminalAtPort,
            ordinal as u32,
            EntityKey::new(EntityKind::Terminal, ordinal as u32),
            EntityKey::new(
                EntityKind::Port,
                match ordinal {
                    0 => 0,
                    1 => 1,
                    _ => ((ordinal / 2) % scale.ports) as u32,
                },
            ),
        );
    }
    for ordinal in 0..scale.berths {
        add_edge(
            &mut relations,
            RelationKind::BerthAtTerminal,
            ordinal as u32,
            EntityKey::new(EntityKind::Berth, ordinal as u32),
            EntityKey::new(EntityKind::Terminal, (ordinal / 2) as u32),
        );
    }
    add_edge(
        &mut relations,
        RelationKind::VesselAssignedToBerth,
        0,
        Anchor::Aurora.entity(),
        Anchor::Atlas.entity(),
    );
    add_voyage_routes(scale, &mut relations);
    add_cargo_bookings(scale, &mut relations);
    add_inspection_links(scale, &mut relations);
    add_edge(
        &mut relations,
        RelationKind::SharesPilotageZone,
        0,
        Anchor::Meridian.entity(),
        Anchor::Southpoint.entity(),
    );
    add_edge(
        &mut relations,
        RelationKind::SharesPilotageZone,
        1,
        Anchor::Southpoint.entity(),
        Anchor::Meridian.entity(),
    );
    relations
}

pub(crate) fn add_voyage_routes(
    scale: SupplyChainScale,
    relations: &mut BTreeMap<RelationKey, RelationEdge>,
) {
    for ordinal in 0..scale.voyages {
        add_edge(
            relations,
            RelationKind::VoyageUsesVessel,
            ordinal as u32,
            EntityKey::new(EntityKind::Voyage, ordinal as u32),
            EntityKey::new(EntityKind::Vessel, (ordinal % scale.vessels) as u32),
        );
        for sequence in 0..3 {
            let call_ordinal = (ordinal * 3 + sequence) as u32;
            add_edge(
                relations,
                RelationKind::VoyageHasCall,
                call_ordinal,
                EntityKey::new(EntityKind::Voyage, ordinal as u32),
                EntityKey::new(EntityKind::PortCall, call_ordinal),
            );
            add_edge(
                relations,
                RelationKind::CallAtPort,
                call_ordinal,
                EntityKey::new(EntityKind::PortCall, call_ordinal),
                EntityKey::new(
                    EntityKind::Port,
                    ((ordinal + sequence) % scale.ports) as u32,
                ),
            );
            if sequence < 2 {
                add_edge(
                    relations,
                    RelationKind::CallPrecedes,
                    call_ordinal,
                    EntityKey::new(EntityKind::PortCall, call_ordinal),
                    EntityKey::new(EntityKind::PortCall, call_ordinal + 1),
                );
            }
        }
    }
}

pub(crate) fn add_cargo_bookings(
    scale: SupplyChainScale,
    relations: &mut BTreeMap<RelationKey, RelationEdge>,
) {
    for ordinal in 0..scale.cargo_lots {
        if ordinal % 2 == 0 {
            add_edge(
                relations,
                RelationKind::CargoBookedOnVoyage,
                ordinal as u32,
                EntityKey::new(EntityKind::CargoLot, ordinal as u32),
                EntityKey::new(EntityKind::Voyage, (ordinal % scale.voyages) as u32),
            );
        }
    }
}

pub(crate) fn add_inspection_links(
    scale: SupplyChainScale,
    relations: &mut BTreeMap<RelationKey, RelationEdge>,
) {
    for ordinal in 0..scale.vessels {
        add_edge(
            relations,
            RelationKind::InspectionCoversVessel,
            ordinal as u32,
            EntityKey::new(EntityKind::Inspection, ordinal as u32),
            EntityKey::new(EntityKind::Vessel, ordinal as u32),
        );
    }
}

pub(crate) fn add_edge(
    relations: &mut BTreeMap<RelationKey, RelationEdge>,
    kind: RelationKind,
    ordinal: u32,
    source: EntityKey,
    target: EntityKey,
) {
    let key = RelationKey::new(kind, ordinal);
    relations.insert(
        key,
        RelationEdge {
            key,
            source,
            target,
        },
    );
}
