use crate::runtime::{WorthUiMeasurementCounterPacket, WorthUiRuntimeCounterFamily};

use super::denial::{WorthUiSteadyFrameCounterDenial, WorthUiSteadyFrameCounterDenialReason};

pub(crate) const ORDINARY_NODES_VISITED: &str = "lane.ordinary.execution.nodes_visited";
pub(crate) const ORDINARY_LAYOUT_RECOMPUTE_BREADTH: &str =
    "lane.ordinary.execution.layout_recompute_breadth";
pub(crate) const ORDINARY_COMMAND_SURFACES_TOUCHED: &str =
    "lane.ordinary.execution.command_surfaces_touched";
pub(crate) const ORDINARY_TOKEN_SUPPORT_TOUCHED: &str =
    "lane.ordinary.execution.token_support_touched";
pub(crate) const ORDINARY_TEXT_SHAPES: &str = "lane.ordinary.execution.text_shapes";
pub(crate) const ORDINARY_GLYPH_UPLOADS: &str = "lane.ordinary.execution.glyph_uploads";
pub(crate) const ORDINARY_SOURCE_PARSE_COUNT: &str = "lane.ordinary.execution.source_parse_count";
pub(crate) const ORDINARY_REGISTRY_LOOKUP_COUNT: &str =
    "lane.ordinary.execution.registry_lookup_count";
pub(crate) const ORDINARY_ARTIFACT_TREE_SCAN_COUNT: &str =
    "lane.ordinary.execution.artifact_tree_scan_count";
pub(crate) const ORDINARY_FULL_PLAN_SCAN_COUNT: &str =
    "lane.ordinary.execution.full_plan_scan_count";

pub(crate) const VIRTUALIZED_VISIBLE_ROWS_TOUCHED: &str =
    "lane.virtualized_data.execution.visible_rows_touched";
pub(crate) const VIRTUALIZED_VISIBLE_COLUMNS_TOUCHED: &str =
    "lane.virtualized_data.execution.visible_columns_touched";
pub(crate) const VIRTUALIZED_VISIBLE_CELLS_TOUCHED: &str =
    "lane.virtualized_data.execution.visible_cells_touched";
pub(crate) const VIRTUALIZED_DIRECT_ROW_LOOKUPS: &str =
    "lane.virtualized_data.execution.direct_row_lookups";
pub(crate) const VIRTUALIZED_EVIDENCE_REFERENCE_LOOKUPS: &str =
    "lane.virtualized_data.execution.evidence_reference_lookups";
pub(crate) const VIRTUALIZED_FULL_COLLECTION_SCAN_COUNT: &str =
    "lane.virtualized_data.execution.full_collection_scan_count";
pub(crate) const VIRTUALIZED_OFFSET_PAGINATION_SUBSTITUTE_COUNT: &str =
    "lane.virtualized_data.execution.offset_pagination_substitute_count";
pub(crate) const VIRTUALIZED_QUERY_COLLECTION_EXECUTION_COUNT: &str =
    "lane.virtualized_data.execution.query_collection_execution_count";
pub(crate) const VIRTUALIZED_DIAGNOSTIC_MATERIALIZATION_COUNT: &str =
    "lane.virtualized_data.execution.diagnostic_materialization_count";

pub(crate) const CANVAS_DRAW_HOOK_COUNT: &str = "lane.canvas_spatial.execution.draw_hook_count";
pub(crate) const CANVAS_SPATIAL_HIT_TESTS: &str = "lane.canvas_spatial.execution.spatial_hit_tests";
pub(crate) const CANVAS_OVERLAY_PLANS: &str = "lane.canvas_spatial.execution.overlay_plans";
pub(crate) const CANVAS_VIEWPORT_TRANSFORMS: &str =
    "lane.canvas_spatial.execution.viewport_transforms";
pub(crate) const CANVAS_DRAW_PASSES: &str = "lane.canvas_spatial.execution.draw_passes";
pub(crate) const CANVAS_RENDERER_REFERENCES: &str =
    "lane.canvas_spatial.execution.renderer_references";
pub(crate) const CANVAS_DOMAIN_GEOMETRY_TRUTH_READS: &str =
    "lane.canvas_spatial.execution.domain_geometry_truth_reads";
pub(crate) const CANVAS_RENDERER_INTERNAL_READS: &str =
    "lane.canvas_spatial.execution.renderer_internal_reads";

pub(crate) const REALTIME_OVERLAY_HOOKS: &str = "lane.realtime_overlay.execution.overlay_hooks";
pub(crate) const REALTIME_FRAME_SYNCHRONIZED_PASSES: &str =
    "lane.realtime_overlay.execution.frame_synchronized_passes";
pub(crate) const REALTIME_RENDERER_SURFACE_HANDOFFS: &str =
    "lane.realtime_overlay.execution.renderer_surface_handoffs";
pub(crate) const REALTIME_TARGETED_OVERLAY_ROWS: &str =
    "lane.realtime_overlay.execution.targeted_overlay_rows";
pub(crate) const REALTIME_ORDINARY_LAYOUT_PASSES: &str =
    "lane.realtime_overlay.execution.ordinary_layout_passes";
pub(crate) const REALTIME_SOURCE_PARSE_COUNT: &str =
    "lane.realtime_overlay.execution.source_parse_count";
pub(crate) const REALTIME_REGISTRY_LOOKUP_COUNT: &str =
    "lane.realtime_overlay.execution.registry_lookup_count";
pub(crate) const REALTIME_ALLOCATION_COUNT: &str =
    "lane.realtime_overlay.execution.executor_allocation_count";
pub(crate) const REALTIME_DIAGNOSTIC_MATERIALIZATION_COUNT: &str =
    "lane.realtime_overlay.execution.diagnostic_materialization_count";

pub(crate) const STEADY_NODES_VISITED: &str = "frame.steady_rendering.nodes_visited";
pub(crate) const STEADY_LAYOUT_RECOMPUTE_BREADTH: &str =
    "frame.steady_rendering.layout_recompute_breadth";
pub(crate) const STEADY_HIT_TEST_BREADTH: &str = "frame.steady_rendering.hit_test_breadth";
pub(crate) const STEADY_VIRTUALIZED_ROWS_TOUCHED: &str =
    "frame.steady_rendering.virtualized_rows_touched";
pub(crate) const STEADY_VIRTUALIZED_COLUMNS_TOUCHED: &str =
    "frame.steady_rendering.virtualized_columns_touched";
pub(crate) const STEADY_DRAW_BATCHES: &str = "frame.steady_rendering.draw_batches";
pub(crate) const STEADY_RENDER_PASSES: &str = "frame.steady_rendering.render_passes";
pub(crate) const STEADY_TEXT_SHAPES: &str = "frame.steady_rendering.text_shapes";
pub(crate) const STEADY_GLYPH_UPLOADS: &str = "frame.steady_rendering.glyph_uploads";
pub(crate) const STEADY_EXECUTOR_ALLOCATIONS: &str = "frame.steady_rendering.executor_allocations";
pub(crate) const STEADY_DIAGNOSTIC_MATERIALIZATIONS: &str =
    "frame.steady_rendering.diagnostic_materializations";
pub(crate) const STEADY_SOURCE_OR_REGISTRY_WORK: &str =
    "frame.steady_rendering.source_or_registry_work";

const ORDINARY_NAMES: &[&str] = &[
    ORDINARY_NODES_VISITED,
    ORDINARY_LAYOUT_RECOMPUTE_BREADTH,
    ORDINARY_COMMAND_SURFACES_TOUCHED,
    ORDINARY_TOKEN_SUPPORT_TOUCHED,
    ORDINARY_TEXT_SHAPES,
    ORDINARY_GLYPH_UPLOADS,
    ORDINARY_SOURCE_PARSE_COUNT,
    ORDINARY_REGISTRY_LOOKUP_COUNT,
    ORDINARY_ARTIFACT_TREE_SCAN_COUNT,
    ORDINARY_FULL_PLAN_SCAN_COUNT,
];

const VIRTUALIZED_NAMES: &[&str] = &[
    VIRTUALIZED_VISIBLE_ROWS_TOUCHED,
    VIRTUALIZED_VISIBLE_COLUMNS_TOUCHED,
    VIRTUALIZED_VISIBLE_CELLS_TOUCHED,
    VIRTUALIZED_DIRECT_ROW_LOOKUPS,
    VIRTUALIZED_EVIDENCE_REFERENCE_LOOKUPS,
    VIRTUALIZED_FULL_COLLECTION_SCAN_COUNT,
    VIRTUALIZED_OFFSET_PAGINATION_SUBSTITUTE_COUNT,
    VIRTUALIZED_QUERY_COLLECTION_EXECUTION_COUNT,
    VIRTUALIZED_DIAGNOSTIC_MATERIALIZATION_COUNT,
];

const CANVAS_NAMES: &[&str] = &[
    CANVAS_DRAW_HOOK_COUNT,
    CANVAS_SPATIAL_HIT_TESTS,
    CANVAS_OVERLAY_PLANS,
    CANVAS_VIEWPORT_TRANSFORMS,
    CANVAS_DRAW_PASSES,
    CANVAS_RENDERER_REFERENCES,
    CANVAS_DOMAIN_GEOMETRY_TRUTH_READS,
    CANVAS_RENDERER_INTERNAL_READS,
];

const REALTIME_NAMES: &[&str] = &[
    REALTIME_OVERLAY_HOOKS,
    REALTIME_FRAME_SYNCHRONIZED_PASSES,
    REALTIME_RENDERER_SURFACE_HANDOFFS,
    REALTIME_TARGETED_OVERLAY_ROWS,
    REALTIME_ORDINARY_LAYOUT_PASSES,
    REALTIME_SOURCE_PARSE_COUNT,
    REALTIME_REGISTRY_LOOKUP_COUNT,
    REALTIME_ALLOCATION_COUNT,
    REALTIME_DIAGNOSTIC_MATERIALIZATION_COUNT,
];

const STEADY_NAMES: &[&str] = &[
    STEADY_NODES_VISITED,
    STEADY_LAYOUT_RECOMPUTE_BREADTH,
    STEADY_HIT_TEST_BREADTH,
    STEADY_VIRTUALIZED_ROWS_TOUCHED,
    STEADY_VIRTUALIZED_COLUMNS_TOUCHED,
    STEADY_DRAW_BATCHES,
    STEADY_RENDER_PASSES,
    STEADY_TEXT_SHAPES,
    STEADY_GLYPH_UPLOADS,
    STEADY_EXECUTOR_ALLOCATIONS,
    STEADY_DIAGNOSTIC_MATERIALIZATIONS,
    STEADY_SOURCE_OR_REGISTRY_WORK,
];

pub(crate) fn validate_packet_schema(
    packet: &WorthUiMeasurementCounterPacket,
) -> Result<(), WorthUiSteadyFrameCounterDenial> {
    let expected = expected_names_for_family(packet.family());
    if expected.is_empty() {
        return Err(WorthUiSteadyFrameCounterDenial::new(
            WorthUiSteadyFrameCounterDenialReason::UnexpectedCounterRow,
        ));
    }
    validate_no_duplicate_rows(packet)?;
    validate_required_rows(packet, expected)?;
    validate_no_unexpected_rows(packet, expected)
}

fn expected_names_for_family(family: WorthUiRuntimeCounterFamily) -> &'static [&'static str] {
    match family {
        WorthUiRuntimeCounterFamily::OrdinaryLaneExecution => ORDINARY_NAMES,
        WorthUiRuntimeCounterFamily::VirtualizedDataExecution => VIRTUALIZED_NAMES,
        WorthUiRuntimeCounterFamily::CanvasSpatialExecution => CANVAS_NAMES,
        WorthUiRuntimeCounterFamily::RealtimeOverlayExecution => REALTIME_NAMES,
        WorthUiRuntimeCounterFamily::SteadyFrameRendering => STEADY_NAMES,
        WorthUiRuntimeCounterFamily::ReloadCandidateAdmission
        | WorthUiRuntimeCounterFamily::SourceIngress
        | WorthUiRuntimeCounterFamily::ArtifactComparison
        | WorthUiRuntimeCounterFamily::ImpactNarrowing
        | WorthUiRuntimeCounterFamily::IdentityReplacement
        | WorthUiRuntimeCounterFamily::DurableStateReconciliation
        | WorthUiRuntimeCounterFamily::QueryRebindPlanning
        | WorthUiRuntimeCounterFamily::PlanAssembly
        | WorthUiRuntimeCounterFamily::PlanLowering
        | WorthUiRuntimeCounterFamily::LaneAdmission
        | WorthUiRuntimeCounterFamily::Activation
        | WorthUiRuntimeCounterFamily::CommittedAllocationActivation
        | WorthUiRuntimeCounterFamily::DiagnosticsProjection => &[],
    }
}

fn validate_no_duplicate_rows(
    packet: &WorthUiMeasurementCounterPacket,
) -> Result<(), WorthUiSteadyFrameCounterDenial> {
    if packet
        .counters()
        .windows(2)
        .any(|window| window[0].name() == window[1].name())
    {
        return Err(WorthUiSteadyFrameCounterDenial::new(
            WorthUiSteadyFrameCounterDenialReason::DuplicateCounterRow,
        ));
    }
    Ok(())
}

fn validate_required_rows(
    packet: &WorthUiMeasurementCounterPacket,
    expected: &[&str],
) -> Result<(), WorthUiSteadyFrameCounterDenial> {
    if expected.iter().any(|name| {
        packet
            .counters()
            .iter()
            .all(|counter| counter.name() != *name)
    }) {
        return Err(WorthUiSteadyFrameCounterDenial::new(
            WorthUiSteadyFrameCounterDenialReason::MissingRequiredCounterRow,
        ));
    }
    Ok(())
}

fn validate_no_unexpected_rows(
    packet: &WorthUiMeasurementCounterPacket,
    expected: &[&str],
) -> Result<(), WorthUiSteadyFrameCounterDenial> {
    if packet
        .counters()
        .iter()
        .any(|counter| expected.iter().all(|name| *name != counter.name()))
    {
        return Err(WorthUiSteadyFrameCounterDenial::new(
            WorthUiSteadyFrameCounterDenialReason::UnexpectedCounterRow,
        ));
    }
    Ok(())
}
