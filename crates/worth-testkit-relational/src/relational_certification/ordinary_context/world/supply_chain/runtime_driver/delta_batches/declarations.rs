pub use {crate::relational_certification::ordinary_context::world::supply_chain::runtime_driver::lowering::observed_entity, crate::relational_certification::ordinary_context::world::supply_chain::runtime_driver::lowering::observed_relation_source, crate::relational_certification::ordinary_context::world::supply_chain::runtime_driver::lowering::SupplyChainProductionDeltaLoweringError};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::runtime_driver::mutation_fields::number, crate::relational_certification::ordinary_context::world::supply_chain::runtime_driver::mutation_fields::text, crate::relational_certification::ordinary_context::world::supply_chain::runtime_driver::mutation_fields::update_fields, crate::relational_certification::ordinary_context::world::supply_chain::runtime_driver::mutation_fields::update_relation, crate::relational_certification::ordinary_context::world::supply_chain::runtime_driver::mutation_fields::SupplyChainField};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind, crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord, crate::relational_certification::ordinary_context::world::supply_chain::comparison_state::ObservedSupplyChainState, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind, crate::relational_certification::ordinary_context::world::supply_chain::handles::SupplyChainSemanticHandles};
pub use {worth_relational::facade::transactions::WorkerIntentBatch};

pub fn lower_storm(
    handles: &SupplyChainSemanticHandles,
    observed: &ObservedSupplyChainState,
) -> Result<WorkerIntentBatch, SupplyChainProductionDeltaLoweringError> {
    let voyage = crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::AuroraEastbound.entity();
    let crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Voyage(value) = observed_entity(observed, voyage)? else {
        return Err(crate::relational_certification::ordinary_context::world::supply_chain::runtime_driver::lowering::SupplyChainProductionDeltaLoweringError::UnexpectedEntity(
            voyage,
        ));
    };
    let arrival = checked_add_u32(value.arrival.0, 30, voyage)?;
    let revision = checked_add_u16(value.revision, 1, voyage)?;
    let relation = crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CallAtPort, 1);
    let source = observed_relation_source(observed, relation)?;
    Ok(worth_relational::facade::transactions::WorkerIntentBatch::new("supply-chain-storm-reroute-aurora")
        .push(update_fields(
            handles,
            voyage,
            [
                (crate::relational_certification::ordinary_context::world::supply_chain::runtime_driver::mutation_fields::SupplyChainField::Status, text("Rerouted")),
                (crate::relational_certification::ordinary_context::world::supply_chain::runtime_driver::mutation_fields::SupplyChainField::Arrival, number(arrival)),
                (crate::relational_certification::ordinary_context::world::supply_chain::runtime_driver::mutation_fields::SupplyChainField::Revision, number(revision)),
            ],
        ))
        .push(update_relation(
            handles,
            relation,
            source,
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port, 2),
        )))
}

pub fn lower_maintenance(
    handles: &SupplyChainSemanticHandles,
    observed: &ObservedSupplyChainState,
) -> Result<WorkerIntentBatch, SupplyChainProductionDeltaLoweringError> {
    let voyage = crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::AuroraEastbound.entity();
    let crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Voyage(value) = observed_entity(observed, voyage)? else {
        return Err(crate::relational_certification::ordinary_context::world::supply_chain::runtime_driver::lowering::SupplyChainProductionDeltaLoweringError::UnexpectedEntity(
            voyage,
        ));
    };
    let arrival = checked_add_u32(value.arrival.0, 60, voyage)?;
    let revision = checked_add_u16(value.revision, 1, voyage)?;
    let atlas = crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::Atlas.entity();
    let relation = crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::VesselAssignedToBerth, 0);
    let source = observed_relation_source(observed, relation)?;
    Ok(worth_relational::facade::transactions::WorkerIntentBatch::new("supply-chain-maintain-atlas-berth")
        .push(update_fields(
            handles,
            atlas,
            [(crate::relational_certification::ordinary_context::world::supply_chain::runtime_driver::mutation_fields::SupplyChainField::Posture, text("Maintenance"))],
        ))
        .push(update_fields(
            handles,
            voyage,
            [
                (crate::relational_certification::ordinary_context::world::supply_chain::runtime_driver::mutation_fields::SupplyChainField::Status, text("Delayed")),
                (crate::relational_certification::ordinary_context::world::supply_chain::runtime_driver::mutation_fields::SupplyChainField::Arrival, number(arrival)),
                (crate::relational_certification::ordinary_context::world::supply_chain::runtime_driver::mutation_fields::SupplyChainField::Revision, number(revision)),
            ],
        ))
        .push(update_relation(
            handles,
            relation,
            source,
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Berth, 1),
        )))
}

pub fn lower_medical_hold(
    handles: &SupplyChainSemanticHandles,
) -> Result<WorkerIntentBatch, SupplyChainProductionDeltaLoweringError> {
    Ok(
        worth_relational::facade::transactions::WorkerIntentBatch::new("supply-chain-hold-medical-cargo").push(update_fields(
            handles,
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::MedicalSupplies.entity(),
            [(crate::relational_certification::ordinary_context::world::supply_chain::runtime_driver::mutation_fields::SupplyChainField::Booking, text("Held"))],
        )),
    )
}

pub fn lower_southpoint_expansion(
    handles: &SupplyChainSemanticHandles,
    observed: &ObservedSupplyChainState,
) -> Result<WorkerIntentBatch, SupplyChainProductionDeltaLoweringError> {
    let terminal = crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::SouthpointContainer.entity();
    let berth = crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::SouthpointBerth.entity();
    let crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Terminal(terminal_value) = observed_entity(observed, terminal)? else {
        return Err(crate::relational_certification::ordinary_context::world::supply_chain::runtime_driver::lowering::SupplyChainProductionDeltaLoweringError::UnexpectedEntity(
            terminal,
        ));
    };
    let crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Berth(berth_value) = observed_entity(observed, berth)? else {
        return Err(crate::relational_certification::ordinary_context::world::supply_chain::runtime_driver::lowering::SupplyChainProductionDeltaLoweringError::UnexpectedEntity(
            berth,
        ));
    };
    let terminal_capacity = checked_add_u32(terminal_value.capacity.0, 1_000, terminal)?;
    let berth_capacity = checked_add_u32(berth_value.capacity.0, 50, berth)?;
    Ok(
        worth_relational::facade::transactions::WorkerIntentBatch::new("supply-chain-expand-southpoint-capacity")
            .push(update_fields(
                handles,
                terminal,
                [(crate::relational_certification::ordinary_context::world::supply_chain::runtime_driver::mutation_fields::SupplyChainField::Capacity, number(terminal_capacity))],
            ))
            .push(update_fields(
                handles,
                berth,
                [(crate::relational_certification::ordinary_context::world::supply_chain::runtime_driver::mutation_fields::SupplyChainField::Capacity, number(berth_capacity))],
            )),
    )
}

pub fn lower_competing_arrival(
    handles: &SupplyChainSemanticHandles,
    observed: &ObservedSupplyChainState,
) -> Result<WorkerIntentBatch, SupplyChainProductionDeltaLoweringError> {
    let voyage = crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::AuroraEastbound.entity();
    let crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Voyage(value) = observed_entity(observed, voyage)? else {
        return Err(crate::relational_certification::ordinary_context::world::supply_chain::runtime_driver::lowering::SupplyChainProductionDeltaLoweringError::UnexpectedEntity(
            voyage,
        ));
    };
    let arrival = checked_add_u32(value.arrival.0, 50, voyage)?;
    let revision = checked_add_u16(value.revision, 1, voyage)?;
    Ok(
        worth_relational::facade::transactions::WorkerIntentBatch::new("supply-chain-competing-aurora-arrival").push(update_fields(
            handles,
            voyage,
            [
                (crate::relational_certification::ordinary_context::world::supply_chain::runtime_driver::mutation_fields::SupplyChainField::Arrival, number(arrival)),
                (crate::relational_certification::ordinary_context::world::supply_chain::runtime_driver::mutation_fields::SupplyChainField::Status, text("Delayed")),
                (crate::relational_certification::ordinary_context::world::supply_chain::runtime_driver::mutation_fields::SupplyChainField::Revision, number(revision)),
            ],
        )),
    )
}

pub fn lower_inspection_retirement(
    handles: &SupplyChainSemanticHandles,
) -> Result<WorkerIntentBatch, SupplyChainProductionDeltaLoweringError> {
    let inspection = crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Inspection, 0);
    Ok(
        worth_relational::facade::transactions::WorkerIntentBatch::new("supply-chain-retire-atlas-while-inspecting-aurora")
            .push(update_fields(
                handles,
                crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::Atlas.entity(),
                [(crate::relational_certification::ordinary_context::world::supply_chain::runtime_driver::mutation_fields::SupplyChainField::Posture, text("Retired"))],
            ))
            .push(update_fields(
                handles,
                inspection,
                [(crate::relational_certification::ordinary_context::world::supply_chain::runtime_driver::mutation_fields::SupplyChainField::Result, text("Flagged"))],
            )),
    )
}

pub fn lower_rewire(
    handles: &SupplyChainSemanticHandles,
    observed: &ObservedSupplyChainState,
) -> Result<WorkerIntentBatch, SupplyChainProductionDeltaLoweringError> {
    let call = crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::AuroraSouthpoint.entity();
    let crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::PortCall(value) = observed_entity(observed, call)? else {
        return Err(crate::relational_certification::ordinary_context::world::supply_chain::runtime_driver::lowering::SupplyChainProductionDeltaLoweringError::UnexpectedEntity(
            call,
        ));
    };
    let revision = checked_add_u16(value.revision, 1, call)?;
    let relation = crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CallAtPort, 1);
    let source = observed_relation_source(observed, relation)?;
    Ok(
        worth_relational::facade::transactions::WorkerIntentBatch::new("supply-chain-rewire-aurora-port-call")
            .push(update_fields(
                handles,
                call,
                [(crate::relational_certification::ordinary_context::world::supply_chain::runtime_driver::mutation_fields::SupplyChainField::Revision, number(revision))],
            ))
            .push(update_relation(
                handles,
                relation,
                source,
                crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port, 3),
            )),
    )
}

pub fn lower_hazard_v2(
    handles: &SupplyChainSemanticHandles,
) -> Result<WorkerIntentBatch, SupplyChainProductionDeltaLoweringError> {
    Ok(
        worth_relational::facade::transactions::WorkerIntentBatch::new("supply-chain-adopt-hazard-classification-v2").push(update_fields(
            handles,
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::MedicalSupplies.entity(),
            [(crate::relational_certification::ordinary_context::world::supply_chain::runtime_driver::mutation_fields::SupplyChainField::Hazard, text("HazardousV2"))],
        )),
    )
}

pub fn checked_add_u32(
    value: u32,
    increment: u32,
    entity: EntityKey,
) -> Result<u32, SupplyChainProductionDeltaLoweringError> {
    value
        .checked_add(increment)
        .ok_or(crate::relational_certification::ordinary_context::world::supply_chain::runtime_driver::lowering::SupplyChainProductionDeltaLoweringError::ArithmeticOverflow(
            entity,
        ))
}

pub fn checked_add_u16(
    value: u16,
    increment: u16,
    entity: EntityKey,
) -> Result<u16, SupplyChainProductionDeltaLoweringError> {
    value
        .checked_add(increment)
        .ok_or(crate::relational_certification::ordinary_context::world::supply_chain::runtime_driver::lowering::SupplyChainProductionDeltaLoweringError::ArithmeticOverflow(
            entity,
        ))
}