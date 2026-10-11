use crate::facade::identity::EntityId;
use crate::facade::runtime::RelationalRuntime;
use crate::tests::support::{
    aspect_field_patch_from_values, aspect_key, field_key, string_aspect_value, u64_aspect_value,
};

use super::super::seed_catalog::FintechCaseSeed;
use super::super::RISK_PARTITION;
use super::bulk_create::bulk_create_entities;

pub(super) fn seed_risk_views(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    case_seeds: &[FintechCaseSeed],
) -> Vec<EntityId> {
    bulk_create_entities(
        execution,
        runtime,
        "seed-risk",
        RISK_PARTITION,
        case_seeds.iter().map(|seed| {
            (
                format!("risk-{}", seed.slug),
                aspect_field_patch_from_values([
                    (
                        aspect_key("entity_type"),
                        field_key("entity_type"),
                        string_aspect_value("risk_view"),
                    ),
                    (
                        aspect_key("case"),
                        field_key("case"),
                        string_aspect_value(seed.slug),
                    ),
                    (
                        aspect_key("scenario"),
                        field_key("scenario"),
                        string_aspect_value(seed.risk_scenario),
                    ),
                ]),
            )
        }),
    )
}

pub(super) fn seed_limits(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    case_seeds: &[FintechCaseSeed],
) -> Vec<EntityId> {
    bulk_create_entities(
        execution,
        runtime,
        "seed-limits",
        RISK_PARTITION,
        case_seeds.iter().map(|seed| {
            (
                format!("limit-{}", seed.slug),
                aspect_field_patch_from_values([
                    (
                        aspect_key("entity_type"),
                        field_key("entity_type"),
                        string_aspect_value("limit"),
                    ),
                    (
                        aspect_key("case"),
                        field_key("case"),
                        string_aspect_value(seed.slug),
                    ),
                    (
                        aspect_key("threshold_bps"),
                        field_key("threshold_bps"),
                        u64_aspect_value(seed.limit_threshold_bps as u64),
                    ),
                ]),
            )
        }),
    )
}

pub(super) fn seed_breaches(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    case_seeds: &[FintechCaseSeed],
) -> Vec<EntityId> {
    bulk_create_entities(
        execution,
        runtime,
        "seed-breaches",
        RISK_PARTITION,
        case_seeds.iter().map(|seed| {
            (
                format!("breach-{}", seed.slug),
                aspect_field_patch_from_values([
                    (
                        aspect_key("entity_type"),
                        field_key("entity_type"),
                        string_aspect_value("limit_breach"),
                    ),
                    (
                        aspect_key("case"),
                        field_key("case"),
                        string_aspect_value(seed.slug),
                    ),
                    (
                        aspect_key("status"),
                        field_key("status"),
                        string_aspect_value(seed.breach_status),
                    ),
                ]),
            )
        }),
    )
}
