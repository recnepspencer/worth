//! Only the idempotency writer marks the receipt position in its sealed batch.
use super::*;
use crate::domain_computation::primary_graph::schema_layout::WorthQueryProviderIdempotencyLayout;
use crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcomeIdentity;

use worth_relational::facade::{
    mvcc::RelationalPublicationAppendix,
    transactions::{CreateIntent, MutationIntent, WorkerIntentBatch},
};

#[derive(Clone, Copy)]
pub(in crate::domain_computation::primary_graph) struct ReceiptAppendix {
    position: usize,
}

pub(in crate::domain_computation::primary_graph::provider) fn bind_receipt(
    mut batch: WorkerIntentBatch,
    layout: &WorthQueryProviderIdempotencyLayout,
    binding: WorthQueryApplicationIdempotencyBinding,
    outcome: WorthQueryApplicationCommitOutcomeIdentity,
    emitted: u64,
    partition: worth_relational::facade::identity::PartitionId,
    product: &worth_runtime_world::facade::ProductBranchObservation,
    has_output_settlement: bool,
) -> (WorkerIntentBatch, ReceiptAppendix) {
    let position = batch.intents.len();
    batch = batch.push(idempotency_create_intent(
        layout,
        binding,
        outcome,
        emitted,
        partition,
        Some(product),
        has_output_settlement,
    ));
    (batch, ReceiptAppendix { position })
}

impl ReceiptAppendix {
    /// The batch remains sealed to provider registration. Producers cannot
    /// construct this marker, move it, or submit effects through the appendix.
    pub(in crate::domain_computation::primary_graph) fn detach(
        self,
        mut batch: WorkerIntentBatch,
    ) -> Result<(WorkerIntentBatch, RelationalPublicationAppendix), &'static str> {
        if self.position >= batch.intents.len() {
            return Err("sealed receipt appendix lost its writer position");
        }
        let MutationIntent::Create(CreateIntent::Entity(entity)) =
            batch.intents.remove(self.position)
        else {
            return Err("sealed receipt appendix does not contain its writer's fresh entity");
        };
        Ok((
            batch,
            RelationalPublicationAppendix::fresh_entity(entity, self.position),
        ))
    }
}
