use crate::basis::ExecutionPreflightBundle;
use crate::identity::ResultDigest;
#[cfg(test)]
use crate::planning::SerialFallbackRoute;

use super::{ExecutionCounters, ExecutionError, ExecutionReport, ExecutionResultEnvelope};

pub(crate) fn execute_preflight_bundle(
    preflight: &ExecutionPreflightBundle,
) -> Result<ExecutionResultEnvelope, ExecutionError> {
    let collection = preflight.plan().collection();
    let is_cdc_collection = collection
        .map(|collection| {
            matches!(
                collection.post_read_shaping().result_family(),
                crate::collection::CollectionResultFamily::CdcCollection
            )
        })
        .unwrap_or(false);
    let is_count_rollup = collection
        .map(|collection| {
            matches!(
                collection
                    .post_read_shaping()
                    .aggregate_shape()
                    .function_family(),
                crate::collection::AggregateFunctionFamily::CountRows
            )
        })
        .unwrap_or(false);
    let collection_result_family = collection
        .map(|collection| {
            collection
                .post_read_shaping()
                .result_family()
                .digest_label()
        })
        .unwrap_or("detail");
    let is_display_label_derived =
        collection
            .map(|collection| {
                matches!(
                collection.post_read_shaping().derived_field_plan().computation_class(),
                crate::collection::DerivedFieldComputationClass::DisplayLabelFromIdentityAndProfile
            )
            })
            .unwrap_or(false);
    let rows: Vec<String> = (0..preflight.plan().result_shape().binding_count())
        .map(|index| {
            if is_cdc_collection {
                format!(
                    "cdc:{}:{}:{}",
                    preflight.plan().query().plan_digest().as_str(),
                    preflight.basis().proof().digest().as_str(),
                    index
                )
            } else if is_display_label_derived {
                format!(
                    "derived:display_label:{}:{}:{}",
                    preflight.plan().query().plan_digest().as_str(),
                    preflight.basis().proof().digest().as_str(),
                    index
                )
            } else {
                format!(
                    "result:{}:{}:{}",
                    preflight.plan().query().plan_digest().as_str(),
                    preflight.basis().proof().digest().as_str(),
                    index
                )
            }
        })
        .collect();

    let counters = ExecutionCounters::from_preflight(preflight);
    let result_digest = ResultDigest::from_parts(
        &rows
            .iter()
            .cloned()
            .chain(std::iter::once(format!(
                "plan:{}",
                preflight.plan().query().plan_digest().as_str()
            )))
            .chain(std::iter::once(format!(
                "basis:{}",
                preflight.basis().proof().digest().as_str()
            )))
            .chain(std::iter::once(format!(
                "collection_result_family:{}",
                collection_result_family
            )))
            .chain(std::iter::once(format!(
                "aggregate_family:{}",
                if is_count_rollup {
                    "count_rows"
                } else {
                    "none_admitted_yet"
                }
            )))
            .chain(std::iter::once(format!(
                "derived_field_family:{}",
                if is_display_label_derived {
                    "display_label"
                } else {
                    "none_admitted_yet"
                }
            )))
            .collect::<Vec<_>>(),
    );
    let report = ExecutionReport::from_preflight(preflight, result_digest);
    ExecutionResultEnvelope::new(rows, report, counters)
}

#[cfg(test)]
pub(crate) fn execute_serial_fallback_route(
    route: &SerialFallbackRoute,
) -> Result<ExecutionResultEnvelope, ExecutionError> {
    execute_preflight_bundle(route.preflight())
}
