use crate::facade::history::BranchId;
use crate::facade::transactions::{
    CommitResult, EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent, WorkerIntentBatch,
};

use super::super::fixture::FintechWorld;

pub(crate) fn shock_market_on_branch(
    world: &mut FintechWorld,
    branch_id: BranchId,
) -> CommitResult {
    let mut txn =
        crate::tests::support::test_owner_begin_transaction_for_branch(&world.runtime, branch_id);
    for (idx, market_point) in world.market.market_points.iter().enumerate() {
        txn.push_batch(WorkerIntentBatch::new(format!("shock-market-{idx}")).push(
            MutationIntent::Entity(EntityMutationIntent::UpdateFields(
                UpdateEntityFieldsIntent {
                    entity_id: *market_point,
                    fields: crate::tests::support::aspect_field_patch_from_values([
                        (
                            crate::tests::support::aspect_key("entity_type"),
                            crate::tests::support::field_key("entity_type"),
                            crate::tests::support::string_aspect_value("market_point"),
                        ),
                        (
                            crate::tests::support::aspect_key("curve_bucket"),
                            crate::tests::support::field_key("curve_bucket"),
                            crate::tests::support::usize_aspect_value(idx),
                        ),
                        (
                            crate::tests::support::aspect_key("mid"),
                            crate::tests::support::field_key("mid"),
                            crate::tests::support::fixture_i64_number_aspect_value(
                                10_200 + (idx as i64 * 40),
                            ),
                        ),
                        (
                            crate::tests::support::aspect_key("stress_regime"),
                            crate::tests::support::field_key("stress_regime"),
                            crate::tests::support::string_aspect_value("intraday-shock"),
                        ),
                    ]),
                },
            )),
        ))
        .expect("test staging stays within configured resource budgets");
    }
    txn.commit(&world.runtime).unwrap()
}

pub(crate) fn refresh_risk_views(world: &mut FintechWorld, branch_id: BranchId) -> CommitResult {
    let mut txn =
        crate::tests::support::test_owner_begin_transaction_for_branch(&world.runtime, branch_id);
    for (idx, risk_view) in world.risk.risk_views.iter().enumerate() {
        txn.push_batch(WorkerIntentBatch::new(format!("refresh-risk-{idx}")).push(
            MutationIntent::Entity(EntityMutationIntent::UpdateFields(
                UpdateEntityFieldsIntent {
                    entity_id: *risk_view,
                    fields: crate::tests::support::aspect_field_patch_from_values([
                        (
                            crate::tests::support::aspect_key("entity_type"),
                            crate::tests::support::field_key("entity_type"),
                            crate::tests::support::string_aspect_value("risk_view"),
                        ),
                        (
                            crate::tests::support::aspect_key("scenario"),
                            crate::tests::support::field_key("scenario"),
                            crate::tests::support::string_aspect_value("intraday-shock"),
                        ),
                        (
                            crate::tests::support::aspect_key("trade_index"),
                            crate::tests::support::field_key("trade_index"),
                            crate::tests::support::usize_aspect_value(idx),
                        ),
                        (
                            crate::tests::support::aspect_key("refreshed"),
                            crate::tests::support::field_key("refreshed"),
                            crate::tests::support::bool_aspect_value(true),
                        ),
                    ]),
                },
            )),
        ))
        .expect("test staging stays within configured resource budgets");
    }
    txn.commit(&world.runtime).unwrap()
}

pub(crate) fn stress_seeded_intraday_risk(
    world: &mut FintechWorld,
    branch_id: BranchId,
) -> CommitResult {
    let case = world.intraday_risk_case();
    let mut txn =
        crate::tests::support::test_owner_begin_transaction_for_branch(&world.runtime, branch_id);
    txn.push_batch(
        WorkerIntentBatch::new("stress-intraday-market").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: case.market_point,
                fields: crate::tests::support::aspect_field_patch_from_values([
                    (
                        crate::tests::support::aspect_key("entity_type"),
                        crate::tests::support::field_key("entity_type"),
                        crate::tests::support::string_aspect_value("market_point"),
                    ),
                    (
                        crate::tests::support::aspect_key("case"),
                        crate::tests::support::field_key("case"),
                        crate::tests::support::string_aspect_value("intraday-risk"),
                    ),
                    (
                        crate::tests::support::aspect_key("curve_bucket"),
                        crate::tests::support::field_key("curve_bucket"),
                        crate::tests::support::u64_aspect_value(2),
                    ),
                    (
                        crate::tests::support::aspect_key("mid"),
                        crate::tests::support::field_key("mid"),
                        crate::tests::support::u64_aspect_value(10_375),
                    ),
                    (
                        crate::tests::support::aspect_key("stress_regime"),
                        crate::tests::support::field_key("stress_regime"),
                        crate::tests::support::string_aspect_value("intraday-shock"),
                    ),
                ]),
            }),
        )),
    )
    .expect("test staging stays within configured resource budgets");
    txn.push_batch(
        WorkerIntentBatch::new("stress-intraday-risk").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: case.risk_view,
                fields: crate::tests::support::aspect_field_patch_from_values([
                    (
                        crate::tests::support::aspect_key("entity_type"),
                        crate::tests::support::field_key("entity_type"),
                        crate::tests::support::string_aspect_value("risk_view"),
                    ),
                    (
                        crate::tests::support::aspect_key("case"),
                        crate::tests::support::field_key("case"),
                        crate::tests::support::string_aspect_value("intraday-risk"),
                    ),
                    (
                        crate::tests::support::aspect_key("scenario"),
                        crate::tests::support::field_key("scenario"),
                        crate::tests::support::string_aspect_value("intraday-shock"),
                    ),
                    (
                        crate::tests::support::aspect_key("limit_status"),
                        crate::tests::support::field_key("limit_status"),
                        crate::tests::support::string_aspect_value("breached"),
                    ),
                    (
                        crate::tests::support::aspect_key("refreshed"),
                        crate::tests::support::field_key("refreshed"),
                        crate::tests::support::bool_aspect_value(true),
                    ),
                ]),
            }),
        )),
    )
    .expect("test staging stays within configured resource budgets");
    txn.push_batch(
        WorkerIntentBatch::new("stress-intraday-limit").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: case.limit,
                fields: crate::tests::support::aspect_field_patch_from_values([
                    (
                        crate::tests::support::aspect_key("entity_type"),
                        crate::tests::support::field_key("entity_type"),
                        crate::tests::support::string_aspect_value("limit"),
                    ),
                    (
                        crate::tests::support::aspect_key("case"),
                        crate::tests::support::field_key("case"),
                        crate::tests::support::string_aspect_value("intraday-risk"),
                    ),
                    (
                        crate::tests::support::aspect_key("threshold_bps"),
                        crate::tests::support::field_key("threshold_bps"),
                        crate::tests::support::u64_aspect_value(140),
                    ),
                    (
                        crate::tests::support::aspect_key("breach_state"),
                        crate::tests::support::field_key("breach_state"),
                        crate::tests::support::string_aspect_value("open"),
                    ),
                ]),
            }),
        )),
    )
    .expect("test staging stays within configured resource budgets");
    txn.push_batch(
        WorkerIntentBatch::new("stress-intraday-breach").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: case.breach,
                fields: crate::tests::support::string_aspect_field_patch([
                    (
                        crate::tests::support::aspect_key("entity_type"),
                        crate::tests::support::field_key("entity_type"),
                        "limit_breach",
                    ),
                    (
                        crate::tests::support::aspect_key("case"),
                        crate::tests::support::field_key("case"),
                        "intraday-risk",
                    ),
                    (
                        crate::tests::support::aspect_key("status"),
                        crate::tests::support::field_key("status"),
                        "open",
                    ),
                    (
                        crate::tests::support::aspect_key("severity"),
                        crate::tests::support::field_key("severity"),
                        "critical",
                    ),
                ]),
            }),
        )),
    )
    .expect("test staging stays within configured resource budgets");
    txn.commit(&world.runtime).unwrap()
}
