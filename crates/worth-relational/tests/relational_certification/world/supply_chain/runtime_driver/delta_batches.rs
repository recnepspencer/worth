use super::lowering::{
    observed_entity, observed_relation_source, SupplyChainProductionDeltaLoweringError,
};
use super::mutation_fields::{number, text, update_fields, update_relation, SupplyChainField};
use crate::relational_certification::world::supply_chain::{
    Anchor, EntityKey, EntityKind, EntityRecord, ObservedSupplyChainState, RelationKey,
    RelationKind, SupplyChainSemanticHandles,
};
use worth_relational::facade::transactions::WorkerIntentBatch;

pub(super) fn lower_storm(
    handles: &SupplyChainSemanticHandles,
    observed: &ObservedSupplyChainState,
) -> Result<WorkerIntentBatch, SupplyChainProductionDeltaLoweringError> {
    let voyage = Anchor::AuroraEastbound.entity();
    let EntityRecord::Voyage(value) = observed_entity(observed, voyage)? else {
        return Err(SupplyChainProductionDeltaLoweringError::UnexpectedEntity(
            voyage,
        ));
    };
    let arrival = checked_add_u32(value.arrival.0, 30, voyage)?;
    let revision = checked_add_u16(value.revision, 1, voyage)?;
    let relation = RelationKey::new(RelationKind::CallAtPort, 1);
    let source = observed_relation_source(observed, relation)?;
    Ok(WorkerIntentBatch::new("supply-chain-storm-reroute-aurora")
        .push(update_fields(
            handles,
            voyage,
            [
                (SupplyChainField::Status, text("Rerouted")),
                (SupplyChainField::Arrival, number(arrival)),
                (SupplyChainField::Revision, number(revision)),
            ],
        ))
        .push(update_relation(
            handles,
            relation,
            source,
            EntityKey::new(EntityKind::Port, 2),
        )))
}

pub(super) fn lower_maintenance(
    handles: &SupplyChainSemanticHandles,
    observed: &ObservedSupplyChainState,
) -> Result<WorkerIntentBatch, SupplyChainProductionDeltaLoweringError> {
    let voyage = Anchor::AuroraEastbound.entity();
    let EntityRecord::Voyage(value) = observed_entity(observed, voyage)? else {
        return Err(SupplyChainProductionDeltaLoweringError::UnexpectedEntity(
            voyage,
        ));
    };
    let arrival = checked_add_u32(value.arrival.0, 60, voyage)?;
    let revision = checked_add_u16(value.revision, 1, voyage)?;
    let atlas = Anchor::Atlas.entity();
    let relation = RelationKey::new(RelationKind::VesselAssignedToBerth, 0);
    let source = observed_relation_source(observed, relation)?;
    Ok(WorkerIntentBatch::new("supply-chain-maintain-atlas-berth")
        .push(update_fields(
            handles,
            atlas,
            [(SupplyChainField::Posture, text("Maintenance"))],
        ))
        .push(update_fields(
            handles,
            voyage,
            [
                (SupplyChainField::Status, text("Delayed")),
                (SupplyChainField::Arrival, number(arrival)),
                (SupplyChainField::Revision, number(revision)),
            ],
        ))
        .push(update_relation(
            handles,
            relation,
            source,
            EntityKey::new(EntityKind::Berth, 1),
        )))
}

pub(super) fn lower_medical_hold(
    handles: &SupplyChainSemanticHandles,
) -> Result<WorkerIntentBatch, SupplyChainProductionDeltaLoweringError> {
    Ok(
        WorkerIntentBatch::new("supply-chain-hold-medical-cargo").push(update_fields(
            handles,
            Anchor::MedicalSupplies.entity(),
            [(SupplyChainField::Booking, text("Held"))],
        )),
    )
}

pub(super) fn lower_southpoint_expansion(
    handles: &SupplyChainSemanticHandles,
    observed: &ObservedSupplyChainState,
) -> Result<WorkerIntentBatch, SupplyChainProductionDeltaLoweringError> {
    let terminal = Anchor::SouthpointContainer.entity();
    let berth = Anchor::SouthpointBerth.entity();
    let EntityRecord::Terminal(terminal_value) = observed_entity(observed, terminal)? else {
        return Err(SupplyChainProductionDeltaLoweringError::UnexpectedEntity(
            terminal,
        ));
    };
    let EntityRecord::Berth(berth_value) = observed_entity(observed, berth)? else {
        return Err(SupplyChainProductionDeltaLoweringError::UnexpectedEntity(
            berth,
        ));
    };
    let terminal_capacity = checked_add_u32(terminal_value.capacity.0, 1_000, terminal)?;
    let berth_capacity = checked_add_u32(berth_value.capacity.0, 50, berth)?;
    Ok(
        WorkerIntentBatch::new("supply-chain-expand-southpoint-capacity")
            .push(update_fields(
                handles,
                terminal,
                [(SupplyChainField::Capacity, number(terminal_capacity))],
            ))
            .push(update_fields(
                handles,
                berth,
                [(SupplyChainField::Capacity, number(berth_capacity))],
            )),
    )
}

pub(super) fn lower_competing_arrival(
    handles: &SupplyChainSemanticHandles,
    observed: &ObservedSupplyChainState,
) -> Result<WorkerIntentBatch, SupplyChainProductionDeltaLoweringError> {
    let voyage = Anchor::AuroraEastbound.entity();
    let EntityRecord::Voyage(value) = observed_entity(observed, voyage)? else {
        return Err(SupplyChainProductionDeltaLoweringError::UnexpectedEntity(
            voyage,
        ));
    };
    let arrival = checked_add_u32(value.arrival.0, 50, voyage)?;
    let revision = checked_add_u16(value.revision, 1, voyage)?;
    Ok(
        WorkerIntentBatch::new("supply-chain-competing-aurora-arrival").push(update_fields(
            handles,
            voyage,
            [
                (SupplyChainField::Arrival, number(arrival)),
                (SupplyChainField::Status, text("Delayed")),
                (SupplyChainField::Revision, number(revision)),
            ],
        )),
    )
}

pub(super) fn lower_inspection_retirement(
    handles: &SupplyChainSemanticHandles,
) -> Result<WorkerIntentBatch, SupplyChainProductionDeltaLoweringError> {
    let inspection = EntityKey::new(EntityKind::Inspection, 0);
    Ok(
        WorkerIntentBatch::new("supply-chain-retire-atlas-while-inspecting-aurora")
            .push(update_fields(
                handles,
                Anchor::Atlas.entity(),
                [(SupplyChainField::Posture, text("Retired"))],
            ))
            .push(update_fields(
                handles,
                inspection,
                [(SupplyChainField::Result, text("Flagged"))],
            )),
    )
}

pub(super) fn lower_rewire(
    handles: &SupplyChainSemanticHandles,
    observed: &ObservedSupplyChainState,
) -> Result<WorkerIntentBatch, SupplyChainProductionDeltaLoweringError> {
    let call = Anchor::AuroraSouthpoint.entity();
    let EntityRecord::PortCall(value) = observed_entity(observed, call)? else {
        return Err(SupplyChainProductionDeltaLoweringError::UnexpectedEntity(
            call,
        ));
    };
    let revision = checked_add_u16(value.revision, 1, call)?;
    let relation = RelationKey::new(RelationKind::CallAtPort, 1);
    let source = observed_relation_source(observed, relation)?;
    Ok(
        WorkerIntentBatch::new("supply-chain-rewire-aurora-port-call")
            .push(update_fields(
                handles,
                call,
                [(SupplyChainField::Revision, number(revision))],
            ))
            .push(update_relation(
                handles,
                relation,
                source,
                EntityKey::new(EntityKind::Port, 3),
            )),
    )
}

pub(crate) fn lower_hazard_v2(
    handles: &SupplyChainSemanticHandles,
) -> Result<WorkerIntentBatch, SupplyChainProductionDeltaLoweringError> {
    Ok(
        WorkerIntentBatch::new("supply-chain-adopt-hazard-classification-v2").push(update_fields(
            handles,
            Anchor::MedicalSupplies.entity(),
            [(SupplyChainField::Hazard, text("HazardousV2"))],
        )),
    )
}

fn checked_add_u32(
    value: u32,
    increment: u32,
    entity: EntityKey,
) -> Result<u32, SupplyChainProductionDeltaLoweringError> {
    value
        .checked_add(increment)
        .ok_or(SupplyChainProductionDeltaLoweringError::ArithmeticOverflow(
            entity,
        ))
}

fn checked_add_u16(
    value: u16,
    increment: u16,
    entity: EntityKey,
) -> Result<u16, SupplyChainProductionDeltaLoweringError> {
    value
        .checked_add(increment)
        .ok_or(SupplyChainProductionDeltaLoweringError::ArithmeticOverflow(
            entity,
        ))
}
