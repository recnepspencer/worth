use crate::facade::history::BranchId;
use crate::facade::transactions::{
    CommitResult, EntityMutationIntent, MutationIntent, RollbackOutcome, UpdateEntityFieldsIntent,
    WorkerIntentBatch,
};

use super::super::fixture::FintechWorld;

pub(crate) fn rollback_case_trade_after_savepoint(
    world: &mut FintechWorld,
    branch_id: BranchId,
) -> RollbackOutcome {
    let case = world.late_trade_correction_case();
    let mut txn =
        crate::tests::support::test_owner_begin_transaction_for_branch(&world.runtime, branch_id);
    let savepoint = txn.create_savepoint().unwrap();
    txn.push_batch(
        WorkerIntentBatch::new("temporary-case-trade-correction").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: case.trade,
                fields: crate::tests::support::aspect_field_patch_from_values([
                    (
                        crate::tests::support::aspect_key("entity_type"),
                        crate::tests::support::field_key("entity_type"),
                        crate::tests::support::string_aspect_value("trade"),
                    ),
                    (
                        crate::tests::support::aspect_key("case"),
                        crate::tests::support::field_key("case"),
                        crate::tests::support::string_aspect_value("LateTradeCorrection"),
                    ),
                    (
                        crate::tests::support::aspect_key("desk"),
                        crate::tests::support::field_key("desk"),
                        crate::tests::support::string_aspect_value("analysis-risk"),
                    ),
                    (
                        crate::tests::support::aspect_key("book"),
                        crate::tests::support::field_key("book"),
                        crate::tests::support::string_aspect_value("temporary-correction"),
                    ),
                    (
                        crate::tests::support::aspect_key("notional"),
                        crate::tests::support::field_key("notional"),
                        crate::tests::support::u64_aspect_value(1_999_999),
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
                    (
                        crate::tests::support::aspect_key("transient"),
                        crate::tests::support::field_key("transient"),
                        crate::tests::support::bool_aspect_value(true),
                    ),
                ]),
            }),
        )),
    )
    .expect("test staging stays within configured resource budgets");
    txn.rollback_to_savepoint(savepoint).unwrap()
}

pub(crate) fn commit_case_trade_after_savepoint(
    world: &mut FintechWorld,
    branch_id: BranchId,
) -> CommitResult {
    let case = world.late_trade_correction_case();
    let mut txn =
        crate::tests::support::test_owner_begin_transaction_for_branch(&world.runtime, branch_id);
    let _savepoint = txn.create_savepoint().unwrap();
    txn.push_batch(WorkerIntentBatch::new("saved-case-trade-correction").push(
        MutationIntent::Entity(EntityMutationIntent::UpdateFields(
            UpdateEntityFieldsIntent {
                entity_id: case.trade,
                fields: crate::tests::support::aspect_field_patch_from_values([
                    (
                        crate::tests::support::aspect_key("entity_type"),
                        crate::tests::support::field_key("entity_type"),
                        crate::tests::support::string_aspect_value("trade"),
                    ),
                    (
                        crate::tests::support::aspect_key("case"),
                        crate::tests::support::field_key("case"),
                        crate::tests::support::string_aspect_value("LateTradeCorrection"),
                    ),
                    (
                        crate::tests::support::aspect_key("desk"),
                        crate::tests::support::field_key("desk"),
                        crate::tests::support::string_aspect_value("analysis-risk"),
                    ),
                    (
                        crate::tests::support::aspect_key("book"),
                        crate::tests::support::field_key("book"),
                        crate::tests::support::string_aspect_value("saved-correction"),
                    ),
                    (
                        crate::tests::support::aspect_key("notional"),
                        crate::tests::support::field_key("notional"),
                        crate::tests::support::u64_aspect_value(1_800_000),
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
                    (
                        crate::tests::support::aspect_key("savepoint_applied"),
                        crate::tests::support::field_key("savepoint_applied"),
                        crate::tests::support::bool_aspect_value(true),
                    ),
                ]),
            },
        )),
    ))
    .expect("test staging stays within configured resource budgets");
    txn.commit(&world.runtime).unwrap()
}
