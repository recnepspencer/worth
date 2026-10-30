pub use std::collections::BTreeMap;

pub use crate::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale;
pub use crate::relational_certification::ordinary_context::world::supply_chain::schema::RelationEdge;
pub use {
    crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor,
    crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey,
    crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind,
    crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey,
    crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind,
};

pub fn build(scale: SupplyChainScale) -> BTreeMap<RelationKey, RelationEdge> {
    let mut relations = std::collections::BTreeMap::new();
    for ordinal in 0..scale.terminals {
        add_edge(
            &mut relations,
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::TerminalAtPort,
            ordinal as u32,
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Terminal, ordinal as u32),
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(
                crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port,
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
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::BerthAtTerminal,
            ordinal as u32,
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Berth, ordinal as u32),
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Terminal, (ordinal / 2) as u32),
        );
    }
    add_edge(
        &mut relations,
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::VesselAssignedToBerth,
        0,
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::Aurora.entity(),
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::Atlas.entity(),
    );
    add_voyage_routes(scale, &mut relations);
    add_cargo_bookings(scale, &mut relations);
    add_inspection_links(scale, &mut relations);
    add_edge(
        &mut relations,
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::SharesPilotageZone,
        0,
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::Meridian.entity(),
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::Southpoint.entity(),
    );
    add_edge(
        &mut relations,
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::SharesPilotageZone,
        1,
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::Southpoint.entity(),
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::Meridian.entity(),
    );
    relations
}

pub fn add_voyage_routes(
    scale: SupplyChainScale,
    relations: &mut BTreeMap<RelationKey, RelationEdge>,
) {
    for ordinal in 0..scale.voyages {
        add_edge(
            relations,
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::VoyageUsesVessel,
            ordinal as u32,
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Voyage, ordinal as u32),
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Vessel, (ordinal % scale.vessels) as u32),
        );
        for sequence in 0..3 {
            let call_ordinal = (ordinal * 3 + sequence) as u32;
            add_edge(
                relations,
                crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::VoyageHasCall,
                call_ordinal,
                crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Voyage, ordinal as u32),
                crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::PortCall, call_ordinal),
            );
            add_edge(
                relations,
                crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CallAtPort,
                call_ordinal,
                crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::PortCall, call_ordinal),
                crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(
                    crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port,
                    ((ordinal + sequence) % scale.ports) as u32,
                ),
            );
            if sequence < 2 {
                add_edge(
                    relations,
                    crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CallPrecedes,
                    call_ordinal,
                    crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::PortCall, call_ordinal),
                    crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::PortCall, call_ordinal + 1),
                );
            }
        }
    }
}

pub fn add_cargo_bookings(
    scale: SupplyChainScale,
    relations: &mut BTreeMap<RelationKey, RelationEdge>,
) {
    for ordinal in 0..scale.cargo_lots {
        if ordinal % 2 == 0 {
            add_edge(
                relations,
                crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CargoBookedOnVoyage,
                ordinal as u32,
                crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::CargoLot, ordinal as u32),
                crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Voyage, (ordinal % scale.voyages) as u32),
            );
        }
    }
}

pub fn add_inspection_links(
    scale: SupplyChainScale,
    relations: &mut BTreeMap<RelationKey, RelationEdge>,
) {
    for ordinal in 0..scale.vessels {
        add_edge(
            relations,
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::InspectionCoversVessel,
            ordinal as u32,
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Inspection, ordinal as u32),
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Vessel, ordinal as u32),
        );
    }
}

pub fn add_edge(
    relations: &mut BTreeMap<RelationKey, RelationEdge>,
    kind: RelationKind,
    ordinal: u32,
    source: EntityKey,
    target: EntityKey,
) {
    let key = crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey::new(kind, ordinal);
    relations.insert(
        key,
        RelationEdge {
            key,
            source,
            target,
        },
    );
}
