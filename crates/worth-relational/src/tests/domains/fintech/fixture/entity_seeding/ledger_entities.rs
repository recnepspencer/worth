use crate::facade::identity::EntityId;
use crate::facade::runtime::RelationalRuntime;
use crate::tests::support::{
    aspect_field_patch_from_values, aspect_key, bool_aspect_value, field_key,
    fixture_i64_number_aspect_value, string_aspect_value, usize_aspect_value,
};

use super::super::seed_catalog::FintechCaseSeed;
use super::super::{FintechCaseRole, LEDGER_PARTITION};
use super::bulk_create::bulk_create_entities;

pub(super) fn seed_desks(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    desk_names: &[String],
) -> Vec<EntityId> {
    bulk_create_entities(
        execution,
        runtime,
        "seed-desks",
        LEDGER_PARTITION,
        desk_names.iter().enumerate().map(|(idx, name)| {
            (
                format!("desk-{name}"),
                aspect_field_patch_from_values([
                    (
                        aspect_key("entity_type"),
                        field_key("entity_type"),
                        string_aspect_value("desk"),
                    ),
                    (
                        aspect_key("desk_name"),
                        field_key("desk_name"),
                        string_aspect_value(name),
                    ),
                    (
                        aspect_key("desk_index"),
                        field_key("desk_index"),
                        usize_aspect_value(idx),
                    ),
                ]),
            )
        }),
    )
}

pub(super) fn seed_books(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    case_seeds: &[FintechCaseSeed],
    book_names: &[String],
) -> Vec<EntityId> {
    bulk_create_entities(
        execution,
        runtime,
        "seed-books",
        LEDGER_PARTITION,
        book_names.iter().enumerate().map(|(idx, name)| {
            (
                format!("book-{name}"),
                aspect_field_patch_from_values([
                    (
                        aspect_key("entity_type"),
                        field_key("entity_type"),
                        string_aspect_value("book"),
                    ),
                    (
                        aspect_key("book_name"),
                        field_key("book_name"),
                        string_aspect_value(name),
                    ),
                    (
                        aspect_key("desk_name"),
                        field_key("desk_name"),
                        string_aspect_value(
                            case_seeds
                                .iter()
                                .find(|seed| seed.book_name == *name)
                                .map(|seed| seed.desk_name)
                                .unwrap_or("macro-flow"),
                        ),
                    ),
                    (
                        aspect_key("book_index"),
                        field_key("book_index"),
                        usize_aspect_value(idx),
                    ),
                ]),
            )
        }),
    )
}

pub(super) fn seed_accounts(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    case_seeds: &[FintechCaseSeed],
) -> Vec<EntityId> {
    bulk_create_entities(
        execution,
        runtime,
        "seed-accounts",
        LEDGER_PARTITION,
        case_seeds.iter().map(|seed| {
            (
                format!("account-{}", seed.slug),
                aspect_field_patch_from_values([
                    (
                        aspect_key("entity_type"),
                        field_key("entity_type"),
                        string_aspect_value("account"),
                    ),
                    (
                        aspect_key("case"),
                        field_key("case"),
                        string_aspect_value(seed.slug),
                    ),
                    (
                        aspect_key("book_name"),
                        field_key("book_name"),
                        string_aspect_value(seed.book_name),
                    ),
                    (
                        aspect_key("balance_cents"),
                        field_key("balance_cents"),
                        fixture_i64_number_aspect_value(seed.balance_cents),
                    ),
                ]),
            )
        }),
    )
}

pub(super) fn seed_counterparties(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    case_seeds: &[FintechCaseSeed],
) -> Vec<EntityId> {
    bulk_create_entities(
        execution,
        runtime,
        "seed-counterparties",
        LEDGER_PARTITION,
        case_seeds.iter().map(|seed| {
            (
                format!("counterparty-{}", seed.slug),
                aspect_field_patch_from_values([
                    (
                        aspect_key("entity_type"),
                        field_key("entity_type"),
                        string_aspect_value("counterparty"),
                    ),
                    (
                        aspect_key("name"),
                        field_key("name"),
                        string_aspect_value(seed.counterparty_name),
                    ),
                    (
                        aspect_key("rating"),
                        field_key("rating"),
                        string_aspect_value(seed.counterparty_rating),
                    ),
                    (
                        aspect_key("case"),
                        field_key("case"),
                        string_aspect_value(seed.slug),
                    ),
                ]),
            )
        }),
    )
}

pub(super) fn seed_trades(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    case_seeds: &[FintechCaseSeed],
) -> Vec<EntityId> {
    bulk_create_entities(
        execution,
        runtime,
        "seed-trades",
        LEDGER_PARTITION,
        case_seeds.iter().map(|seed| {
            (
                format!("trade-{}", seed.slug),
                aspect_field_patch_from_values([
                    (
                        aspect_key("entity_type"),
                        field_key("entity_type"),
                        string_aspect_value("trade"),
                    ),
                    (
                        aspect_key("case"),
                        field_key("case"),
                        string_aspect_value(seed.slug),
                    ),
                    (
                        aspect_key("desk"),
                        field_key("desk"),
                        string_aspect_value(seed.desk_name),
                    ),
                    (
                        aspect_key("book"),
                        field_key("book"),
                        string_aspect_value(seed.book_name),
                    ),
                    (
                        aspect_key("notional"),
                        field_key("notional"),
                        fixture_i64_number_aspect_value(seed.notional),
                    ),
                    (
                        aspect_key("ccy"),
                        field_key("ccy"),
                        string_aspect_value(seed.ccy),
                    ),
                    (
                        aspect_key("correction_candidate"),
                        field_key("correction_candidate"),
                        bool_aspect_value(matches!(
                            seed.role,
                            FintechCaseRole::LateTradeCorrection
                        )),
                    ),
                ]),
            )
        }),
    )
}

pub(super) fn seed_settlements(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    case_seeds: &[FintechCaseSeed],
) -> Vec<EntityId> {
    bulk_create_entities(
        execution,
        runtime,
        "seed-settlements",
        LEDGER_PARTITION,
        case_seeds.iter().map(|seed| {
            (
                format!("settlement-{}", seed.slug),
                aspect_field_patch_from_values([
                    (
                        aspect_key("entity_type"),
                        field_key("entity_type"),
                        string_aspect_value("settlement"),
                    ),
                    (
                        aspect_key("case"),
                        field_key("case"),
                        string_aspect_value(seed.slug),
                    ),
                    (
                        aspect_key("status"),
                        field_key("status"),
                        string_aspect_value(seed.settlement_status),
                    ),
                ]),
            )
        }),
    )
}

pub(super) fn seed_cash_events(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    case_seeds: &[FintechCaseSeed],
) -> Vec<EntityId> {
    bulk_create_entities(
        execution,
        runtime,
        "seed-cash-events",
        LEDGER_PARTITION,
        case_seeds.iter().map(|seed| {
            (
                format!("cash-event-{}", seed.slug),
                aspect_field_patch_from_values([
                    (
                        aspect_key("entity_type"),
                        field_key("entity_type"),
                        string_aspect_value("cash_event"),
                    ),
                    (
                        aspect_key("case"),
                        field_key("case"),
                        string_aspect_value(seed.slug),
                    ),
                    (
                        aspect_key("kind"),
                        field_key("kind"),
                        string_aspect_value(seed.cash_event_kind),
                    ),
                ]),
            )
        }),
    )
}

pub(super) fn seed_audit_records(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    case_seeds: &[FintechCaseSeed],
) -> Vec<EntityId> {
    bulk_create_entities(
        execution,
        runtime,
        "seed-audit-records",
        LEDGER_PARTITION,
        case_seeds.iter().map(|seed| {
            (
                format!("audit-{}", seed.slug),
                aspect_field_patch_from_values([
                    (
                        aspect_key("entity_type"),
                        field_key("entity_type"),
                        string_aspect_value("audit_record"),
                    ),
                    (
                        aspect_key("case"),
                        field_key("case"),
                        string_aspect_value(seed.slug),
                    ),
                    (
                        aspect_key("event"),
                        field_key("event"),
                        string_aspect_value(seed.audit_event),
                    ),
                ]),
            )
        }),
    )
}
