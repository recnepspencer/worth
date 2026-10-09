use crate::facade::history::BranchId;
use crate::facade::transactions::{
    CommitResult, EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent, WorkerIntentBatch,
};

use super::super::fixture::{FintechCaseRole, FintechWorld};

pub(crate) fn emit_case_audit_record(
    world: &mut FintechWorld,
    branch_id: BranchId,
    case_role: FintechCaseRole,
    event: &str,
) -> CommitResult {
    let case = world.workflow_case(case_role);
    let mut txn =
        crate::tests::support::test_owner_begin_transaction_for_branch(&world.runtime, branch_id);
    txn.push_batch(
        WorkerIntentBatch::new(format!("audit-{}", event.replace(' ', "-"))).push(
            MutationIntent::Entity(EntityMutationIntent::UpdateFields(
                UpdateEntityFieldsIntent {
                    entity_id: case.audit_record,
                    fields: crate::tests::support::aspect_field_patch_from_values([
                        (
                            crate::tests::support::aspect_key("entity_type"),
                            crate::tests::support::field_key("entity_type"),
                            crate::tests::support::string_aspect_value("audit_record"),
                        ),
                        (
                            crate::tests::support::aspect_key("case"),
                            crate::tests::support::field_key("case"),
                            crate::tests::support::string_aspect_value(&format!("{:?}", case.role)),
                        ),
                        (
                            crate::tests::support::aspect_key("event"),
                            crate::tests::support::field_key("event"),
                            crate::tests::support::string_aspect_value(event),
                        ),
                        (
                            crate::tests::support::aspect_key("recorded_by"),
                            crate::tests::support::field_key("recorded_by"),
                            crate::tests::support::string_aspect_value("fintech-domain-workflow"),
                        ),
                    ]),
                },
            )),
        ),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    txn.commit(
        &world.runtime,
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .unwrap()
}

pub(crate) fn emit_trade_correction_audit_record(
    world: &mut FintechWorld,
    branch_id: BranchId,
) -> CommitResult {
    emit_case_audit_record(
        world,
        branch_id,
        FintechCaseRole::LateTradeCorrection,
        "trade-correction-confirmed",
    )
}
