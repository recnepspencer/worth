pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::mutation_fields::text, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::mutation_fields::update_fields, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::mutation_fields::SupplyChainField};
pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::handles::SupplyChainSemanticHandles};
pub(crate) use {worth_relational::facade::transactions::WorkerIntentBatch};

pub(crate) fn lower_cargo_footprint_batch(
    handles: &SupplyChainSemanticHandles,
    scale: SupplyChainScale,
    footprint: usize,
) -> WorkerIntentBatch {
    assert!(footprint > 0 && footprint <= scale.cargo_lots);
    (0..footprint).fold(
        worth_relational::facade::transactions::WorkerIntentBatch::new(format!("supply-chain-cargo-footprint-{footprint}")),
        |batch, ordinal| {
            batch.push(update_fields(
                handles,
                worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::CargoLot, ordinal as u32),
                [(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::mutation_fields::SupplyChainField::Booking, text("Held"))],
            ))
        },
    )
}