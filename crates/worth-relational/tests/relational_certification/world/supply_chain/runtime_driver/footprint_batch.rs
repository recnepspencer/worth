use super::mutation_fields::{text, update_fields, SupplyChainField};
use crate::relational_certification::world::supply_chain::{
    EntityKey, EntityKind, SupplyChainScale, SupplyChainSemanticHandles,
};
use worth_relational::facade::transactions::WorkerIntentBatch;

pub(crate) fn lower_cargo_footprint_batch(
    handles: &SupplyChainSemanticHandles,
    scale: SupplyChainScale,
    footprint: usize,
) -> WorkerIntentBatch {
    assert!(footprint > 0 && footprint <= scale.cargo_lots);
    (0..footprint).fold(
        WorkerIntentBatch::new(format!("supply-chain-cargo-footprint-{footprint}")),
        |batch, ordinal| {
            batch.push(update_fields(
                handles,
                EntityKey::new(EntityKind::CargoLot, ordinal as u32),
                [(SupplyChainField::Booking, text("Held"))],
            ))
        },
    )
}
