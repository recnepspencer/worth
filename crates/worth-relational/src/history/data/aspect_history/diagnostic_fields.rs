use crate::diagnostics::data::{RelationalDiagnosticFields, RelationalDiagnosticValue};
use crate::visibility::materialization::read_records::projection::contracts::projection_aspect_filter::ProjectionAspectFilter;
use crate::visibility::materialization::read_records::projection::contracts::projection_aspect_filter::ProjectionAspectFilterMode;

use super::{
    AspectHistoryCommitSpan, AspectHistoryLineageEventSpan, AspectHistoryResolutionTrace,
    HistoryAspectQueryTarget,
};

pub(super) fn aspect_history_resolution_trace_fields(
    trace: &AspectHistoryResolutionTrace,
) -> RelationalDiagnosticFields {
    RelationalDiagnosticValue::object([
        (
            "requested_target",
            history_query_target_value(&trace.requested_target),
        ),
        (
            "branch_id",
            RelationalDiagnosticValue::string(trace.branch_id.0.clone()),
        ),
        ("filter", filter_value(trace.filter.as_ref())),
        (
            "resolved_aspects",
            canonical_aspect_set_value(&trace.resolved_aspects),
        ),
        (
            "searched_commit_span",
            RelationalDiagnosticValue::optional(trace.searched_commit_span.map(commit_span_value)),
        ),
        (
            "searched_lineage_event_span",
            RelationalDiagnosticValue::optional(
                trace
                    .searched_lineage_event_span
                    .map(lineage_event_span_value),
            ),
        ),
        (
            "returned_entries",
            RelationalDiagnosticValue::Unsigned(trace.returned_entries),
        ),
        (
            "traversed_commits",
            RelationalDiagnosticValue::Unsigned(trace.traversed_commits),
        ),
        (
            "traversed_lineage_events",
            RelationalDiagnosticValue::Unsigned(trace.traversed_lineage_events),
        ),
    ])
    .into()
}

fn history_query_target_value(target: &HistoryAspectQueryTarget) -> RelationalDiagnosticValue {
    match target {
        HistoryAspectQueryTarget::Entity(entity_id) => RelationalDiagnosticValue::object([
            ("target_kind", RelationalDiagnosticValue::string("entity")),
            ("entity", RelationalDiagnosticValue::EntityId(*entity_id)),
        ]),
        HistoryAspectQueryTarget::Relation(relation_id) => RelationalDiagnosticValue::object([
            ("target_kind", RelationalDiagnosticValue::string("relation")),
            (
                "relation",
                RelationalDiagnosticValue::RelationId(*relation_id),
            ),
        ]),
        HistoryAspectQueryTarget::Lineage(lineage_id) => RelationalDiagnosticValue::object([
            ("target_kind", RelationalDiagnosticValue::string("lineage")),
            (
                "lineage_id",
                RelationalDiagnosticValue::LineageId(*lineage_id),
            ),
        ]),
    }
}

fn filter_value(filter: Option<&ProjectionAspectFilter>) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::optional(filter.map(|filter| {
        RelationalDiagnosticValue::object([
            ("filter_mode", filter_mode_value(filter.mode())),
            (
                "requested_projection_scope",
                RelationalDiagnosticValue::array(
                    filter
                        .projection_scope()
                        .requirements()
                        .iter()
                        .map(|requirement| {
                            RelationalDiagnosticValue::object([
                                (
                                    "aspect_key",
                                    RelationalDiagnosticValue::AspectKey(
                                        requirement.aspect_key().clone(),
                                    ),
                                ),
                                (
                                    "mask_locus",
                                    if requirement.mask().is_whole_aspect() {
                                        RelationalDiagnosticValue::string("whole_aspect")
                                    } else {
                                        RelationalDiagnosticValue::string("field_paths")
                                    },
                                ),
                            ])
                        }),
                ),
            ),
        ])
    }))
}

fn filter_mode_value(mode: ProjectionAspectFilterMode) -> RelationalDiagnosticValue {
    match mode {
        ProjectionAspectFilterMode::Any => RelationalDiagnosticValue::string("any"),
        ProjectionAspectFilterMode::All => RelationalDiagnosticValue::string("all"),
        _ => unreachable!("unhandled Relational contract variant"),
    }
}

fn canonical_aspect_set_value(
    aspects: &[worth_foundational::facade::AspectKey],
) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::array(
        aspects
            .iter()
            .cloned()
            .map(RelationalDiagnosticValue::AspectKey),
    )
}

fn commit_span_value(span: AspectHistoryCommitSpan) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::object([
        (
            "first_commit_id",
            RelationalDiagnosticValue::Unsigned(span.first_commit_id.0),
        ),
        (
            "last_commit_id",
            RelationalDiagnosticValue::Unsigned(span.last_commit_id.0),
        ),
    ])
}

fn lineage_event_span_value(span: AspectHistoryLineageEventSpan) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::object([
        (
            "first_event_id",
            RelationalDiagnosticValue::Unsigned(span.first_event_id),
        ),
        (
            "last_event_id",
            RelationalDiagnosticValue::Unsigned(span.last_event_id),
        ),
    ])
}
