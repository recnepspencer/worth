use crate::facade::history::BranchId;
use crate::facade::identity::EntityId;
use crate::facade::transactions::{
    CommitResult, EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent, WorkerIntentBatch,
};

use super::super::fixture::FintechWorld;

pub(crate) fn correct_trade_with_replacement(
    world: &mut FintechWorld,
    branch_id: BranchId,
    trade_id: EntityId,
) -> CommitResult {
    let mut txn =
        crate::tests::support::test_owner_begin_transaction_for_branch(&world.runtime, branch_id);
    txn.push_batch(
        WorkerIntentBatch::new("replace-trade").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: trade_id,
                fields: crate::tests::support::aspect_field_patch_from_values([
                    (
                        crate::tests::support::aspect_key("entity_type"),
                        crate::tests::support::field_key("entity_type"),
                        crate::tests::support::string_aspect_value("trade"),
                    ),
                    (
                        crate::tests::support::aspect_key("desk"),
                        crate::tests::support::field_key("desk"),
                        crate::tests::support::string_aspect_value("macro-flow"),
                    ),
                    (
                        crate::tests::support::aspect_key("book"),
                        crate::tests::support::field_key("book"),
                        crate::tests::support::string_aspect_value("analysis-risk"),
                    ),
                    (
                        crate::tests::support::aspect_key("notional"),
                        crate::tests::support::field_key("notional"),
                        crate::tests::support::u64_aspect_value(1_750_000),
                    ),
                    (
                        crate::tests::support::aspect_key("ccy"),
                        crate::tests::support::field_key("ccy"),
                        crate::tests::support::string_aspect_value("USD"),
                    ),
                    (
                        crate::tests::support::aspect_key("corrected"),
                        crate::tests::support::field_key("corrected"),
                        crate::tests::support::bool_aspect_value(true),
                    ),
                ]),
            }),
        )),
    )
    .expect("test staging stays within configured resource budgets");
    txn.commit(&world.runtime).unwrap()
}

pub(crate) fn correct_seeded_trade_candidate(
    world: &mut FintechWorld,
    branch_id: BranchId,
) -> CommitResult {
    correct_trade_with_replacement(world, branch_id, world.late_trade_correction_case().trade)
}
