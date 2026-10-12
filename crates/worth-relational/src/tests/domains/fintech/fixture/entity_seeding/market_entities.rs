use crate::facade::identity::EntityId;
use crate::facade::runtime::RelationalRuntime;
use crate::tests::support::{
    aspect_field_patch_from_values, aspect_key, field_key, fixture_i64_number_aspect_value,
    string_aspect_value, usize_aspect_value,
};

use super::super::seed_catalog::FintechCaseSeed;
use super::super::MARKET_PARTITION;
use super::bulk_create::bulk_create_entities;

pub(super) fn seed_instruments(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    case_seeds: &[FintechCaseSeed],
) -> Vec<EntityId> {
    bulk_create_entities(
        execution,
        runtime,
        "seed-instruments",
        MARKET_PARTITION,
        case_seeds.iter().map(|seed| {
            (
                format!("instrument-{}", seed.slug),
                aspect_field_patch_from_values([
                    (
                        aspect_key("entity_type"),
                        field_key("entity_type"),
                        string_aspect_value("instrument"),
                    ),
                    (
                        aspect_key("case"),
                        field_key("case"),
                        string_aspect_value(seed.slug),
                    ),
                    (
                        aspect_key("symbol"),
                        field_key("symbol"),
                        string_aspect_value(seed.symbol),
                    ),
                    (
                        aspect_key("asset_class"),
                        field_key("asset_class"),
                        string_aspect_value(seed.asset_class),
                    ),
                ]),
            )
        }),
    )
}

pub(super) fn seed_market_points(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    case_seeds: &[FintechCaseSeed],
) -> Vec<EntityId> {
    bulk_create_entities(
        execution,
        runtime,
        "seed-market",
        MARKET_PARTITION,
        case_seeds.iter().enumerate().map(|(idx, seed)| {
            (
                format!("curve-{}", seed.slug),
                aspect_field_patch_from_values([
                    (
                        aspect_key("entity_type"),
                        field_key("entity_type"),
                        string_aspect_value("market_point"),
                    ),
                    (
                        aspect_key("case"),
                        field_key("case"),
                        string_aspect_value(seed.slug),
                    ),
                    (
                        aspect_key("curve_bucket"),
                        field_key("curve_bucket"),
                        usize_aspect_value(idx),
                    ),
                    (
                        aspect_key("mid"),
                        field_key("mid"),
                        fixture_i64_number_aspect_value(seed.market_mid),
                    ),
                ]),
            )
        }),
    )
}
