use crate::facade::identity::{EntityId, KindId, PartitionId, RelationId};
use crate::facade::runtime::RelationalRuntime;
use crate::facade::transactions::{
    AspectFieldPatch, BulkRelationCreateIntent, CommitResult, CreateIntent, MutationIntent,
    RecordRef, WorkerIntentBatch,
};
use crate::tests::support::single_string_aspect_field_patch;

use super::entity_seeding::SeededEntityState;
use super::seed_catalog::FintechCaseSeed;
use super::{FintechWorkflowCase, LEDGER_PARTITION, MARKET_PARTITION, RISK_PARTITION};

pub(super) fn seed_relations(
    runtime: &RelationalRuntime,
    case_seeds: &[FintechCaseSeed],
    seeded: &SeededEntityState,
    workflow_cases: &[FintechWorkflowCase],
) -> Vec<RelationId> {
    let mut relations = Vec::new();
    relations.extend(bulk_create_relations(
        runtime,
        "desk-book",
        LEDGER_PARTITION,
        seeded.book_names.iter().map(|name| {
            (
                format!("desk-book-{name}"),
                (
                    *seeded.book_map.get(name).expect("book exists"),
                    *seeded
                        .desk_map
                        .get(
                            case_seeds
                                .iter()
                                .find(|seed| seed.book_name == *name)
                                .map(|seed| seed.desk_name)
                                .expect("book should belong to desk"),
                        )
                        .expect("desk exists"),
                ),
                relation_role_patch("owned_by_desk"),
            )
        }),
    ));
    relations.extend(bulk_create_relations(
        runtime,
        "trade-account",
        LEDGER_PARTITION,
        workflow_cases.iter().map(|case| {
            (
                format!("trade-account-{:?}", case.role),
                (case.trade, case.account),
                relation_role_patch("book_owner"),
            )
        }),
    ));
    relations.extend(bulk_create_relations(
        runtime,
        "trade-book",
        LEDGER_PARTITION,
        workflow_cases.iter().map(|case| {
            (
                format!("trade-book-{:?}", case.role),
                (case.trade, case.book),
                relation_role_patch("booked_in"),
            )
        }),
    ));
    relations.extend(bulk_create_relations(
        runtime,
        "trade-counterparty",
        LEDGER_PARTITION,
        workflow_cases.iter().map(|case| {
            (
                format!("trade-counterparty-{:?}", case.role),
                (case.trade, case.counterparty),
                relation_role_patch("facing"),
            )
        }),
    ));
    relations.extend(bulk_create_relations(
        runtime,
        "trade-settlement",
        LEDGER_PARTITION,
        workflow_cases.iter().map(|case| {
            (
                format!("trade-settlement-{:?}", case.role),
                (case.trade, case.settlement),
                relation_role_patch("settles_via"),
            )
        }),
    ));
    relations.extend(bulk_create_relations(
        runtime,
        "settlement-cash-event",
        LEDGER_PARTITION,
        workflow_cases.iter().map(|case| {
            (
                format!("settlement-cash-event-{:?}", case.role),
                (case.settlement, case.cash_event),
                relation_role_patch("funded_by"),
            )
        }),
    ));
    relations.extend(bulk_create_relations(
        runtime,
        "trade-audit-record",
        LEDGER_PARTITION,
        workflow_cases.iter().map(|case| {
            (
                format!("trade-audit-{:?}", case.role),
                (case.trade, case.audit_record),
                relation_role_patch("audited_by"),
            )
        }),
    ));
    relations.extend(bulk_create_relations(
        runtime,
        "trade-instrument",
        MARKET_PARTITION,
        workflow_cases.iter().map(|case| {
            (
                format!("trade-instrument-{:?}", case.role),
                (case.trade, case.instrument),
                relation_role_patch("references_instrument"),
            )
        }),
    ));
    relations.extend(bulk_create_relations(
        runtime,
        "trade-market",
        MARKET_PARTITION,
        workflow_cases.iter().map(|case| {
            (
                format!("trade-market-{:?}", case.role),
                (case.trade, case.market_point),
                relation_role_patch("marks"),
            )
        }),
    ));
    relations.extend(bulk_create_relations(
        runtime,
        "trade-risk",
        RISK_PARTITION,
        workflow_cases.iter().map(|case| {
            (
                format!("trade-risk-{:?}", case.role),
                (case.trade, case.risk_view),
                relation_role_patch("derived_risk"),
            )
        }),
    ));
    relations.extend(bulk_create_relations(
        runtime,
        "risk-limit",
        RISK_PARTITION,
        workflow_cases.iter().map(|case| {
            (
                format!("risk-limit-{:?}", case.role),
                (case.risk_view, case.limit),
                relation_role_patch("checked_against"),
            )
        }),
    ));
    relations.extend(bulk_create_relations(
        runtime,
        "limit-breach",
        RISK_PARTITION,
        workflow_cases.iter().map(|case| {
            (
                format!("limit-breach-{:?}", case.role),
                (case.limit, case.breach),
                relation_role_patch("breach_state"),
            )
        }),
    ));
    relations
}

pub(super) fn bulk_create_relations<I>(
    runtime: &RelationalRuntime,
    batch_name: &str,
    partition_id: PartitionId,
    specs: I,
) -> Vec<RelationId>
where
    I: IntoIterator<Item = (String, (EntityId, EntityId), AspectFieldPatch)>,
{
    let mut client_keys = Vec::new();
    let mut endpoints = Vec::new();
    let mut field_patches = Vec::new();
    for (key, (source, target), fields) in specs {
        client_keys.push(crate::facade::symbols::ClientKey::raw(key));
        endpoints.push((
            crate::transactions::data::EntityReference::Existing(source),
            crate::transactions::data::EntityReference::Existing(target),
        ));
        field_patches.push(fields);
    }
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(runtime);
    txn.push_batch(
        WorkerIntentBatch::new(batch_name).push(MutationIntent::Create(
            CreateIntent::BulkRelations(BulkRelationCreateIntent {
                partition_id,
                kind_id: KindId(2),
                client_keys,
                endpoints,
                field_patches,
            }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    changed_relations(
        &txn.commit(
            runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap(),
    )
}

fn relation_role_patch(role: &str) -> AspectFieldPatch {
    single_string_aspect_field_patch(
        crate::tests::support::aspect_key("role"),
        crate::tests::support::field_key("role"),
        role,
    )
}

fn changed_relations(outcome: &CommitResult) -> Vec<RelationId> {
    outcome
        .changed_records
        .iter()
        .filter_map(|record| match record {
            RecordRef::Entity(_) => None,
            RecordRef::Relation(id) => Some(*id),
        })
        .collect()
}
