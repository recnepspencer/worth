use crate::facade::history::BranchId;
use crate::facade::identity::EntityId;
use crate::facade::transactions::{
    CommitResult, EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent, WorkerIntentBatch,
};

use super::super::fixture::FintechWorld;

pub(crate) fn repair_seeded_failed_settlement(
    world: &mut FintechWorld,
    branch_id: BranchId,
) -> CommitResult {
    let case = world.failed_settlement_repair_case();
    repair_settlement_with_aspect_field_patches(
        world,
        branch_id,
        case.settlement,
        case.cash_event,
        case.audit_record,
    )
}

pub(crate) fn repair_settlement_with_aspect_field_patches(
    world: &mut FintechWorld,
    branch_id: BranchId,
    settlement_id: EntityId,
    cash_event_id: EntityId,
    audit_record_id: EntityId,
) -> CommitResult {
    let mut txn =
        crate::tests::support::test_owner_begin_transaction_for_branch(&world.runtime, branch_id);
    txn.push_batch(
        WorkerIntentBatch::new("repair-settlement").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: settlement_id,
                fields: crate::tests::support::aspect_field_patch_from_values([
                    (
                        crate::tests::support::aspect_key("entity_type"),
                        crate::tests::support::field_key("entity_type"),
                        crate::tests::support::string_aspect_value("settlement"),
                    ),
                    (
                        crate::tests::support::aspect_key("case"),
                        crate::tests::support::field_key("case"),
                        crate::tests::support::string_aspect_value("failed-settlement-repair"),
                    ),
                    (
                        crate::tests::support::aspect_key("status"),
                        crate::tests::support::field_key("status"),
                        crate::tests::support::string_aspect_value("repaired"),
                    ),
                    (
                        crate::tests::support::aspect_key("repair_completed"),
                        crate::tests::support::field_key("repair_completed"),
                        crate::tests::support::bool_aspect_value(true),
                    ),
                ]),
            }),
        )),
    )
    .expect("test staging stays within configured resource budgets");
    txn.push_batch(
        WorkerIntentBatch::new("repair-cash-event").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: cash_event_id,
                fields: crate::tests::support::string_aspect_field_patch([
                    (
                        crate::tests::support::aspect_key("entity_type"),
                        crate::tests::support::field_key("entity_type"),
                        "cash_event",
                    ),
                    (
                        crate::tests::support::aspect_key("case"),
                        crate::tests::support::field_key("case"),
                        "failed-settlement-repair",
                    ),
                    (
                        crate::tests::support::aspect_key("kind"),
                        crate::tests::support::field_key("kind"),
                        "repair-funding",
                    ),
                    (
                        crate::tests::support::aspect_key("status"),
                        crate::tests::support::field_key("status"),
                        "applied",
                    ),
                ]),
            }),
        )),
    )
    .expect("test staging stays within configured resource budgets");
    txn.push_batch(
        WorkerIntentBatch::new("repair-audit-record").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: audit_record_id,
                fields: crate::tests::support::string_aspect_field_patch([
                    (
                        crate::tests::support::aspect_key("entity_type"),
                        crate::tests::support::field_key("entity_type"),
                        "audit_record",
                    ),
                    (
                        crate::tests::support::aspect_key("case"),
                        crate::tests::support::field_key("case"),
                        "failed-settlement-repair",
                    ),
                    (
                        crate::tests::support::aspect_key("event"),
                        crate::tests::support::field_key("event"),
                        "settlement-repaired",
                    ),
                ]),
            }),
        )),
    )
    .expect("test staging stays within configured resource budgets");
    txn.commit(&world.runtime).unwrap()
}
