pub mod comparison;
pub mod inspection;
pub mod model;
pub mod policy;
pub(crate) mod runtime;

pub use comparison::compare;
pub use comparison::diff;
pub use inspection::access;
pub use inspection::display;
pub use inspection::history;
pub use model::{epochs, facts, failure, flow, lineage, replay, summary};
pub use policy::profile;
pub(crate) use runtime::execution_flow;
pub(crate) use runtime::recorder;
pub(crate) use runtime::state;

pub use access::RuntimeDiagnostics as Diagnostics;
pub use access::{
    diagnostics_for_graph, diagnostics_for_runtime, GraphComparisonDiagnostics, GraphDiagnostics,
    GraphHealthDiagnostics, GraphInspectDiagnostics,
};
pub use compare::{
    explanations_semantically_equivalent, graphs_semantically_equivalent,
    lineage_records_equivalent, plans_semantically_equivalent, repeat_run_summaries_equal,
    replay_slices_equivalent, reports_semantically_equivalent, serial_parallel_reports_equivalent,
};
pub use diff::{
    compare_execution_history, compare_execution_reports, compare_explanations, compare_failures,
    compare_flows, compare_graphs, compare_lineage_records, compare_plans, compare_replay_slices,
    DiagnosticMismatch, DiagnosticMismatchCategory, ExecutionReportDiff, ExplanationDiff,
    FailureDiff, FlowDiff, GraphDiff, HistoryDiff, LineageDiff, PlanDiff, ReplayDiff,
};
pub use display::{
    render_execution_history_summary, render_execution_report_summary, render_explanation_summary,
    render_failure_summary, render_flow_summary, render_graph_summary, render_plan_summary,
};
pub use epochs::{
    EventEpochOutcome, EventEpochSummary, EventSubscriberOutcome, EventSubscriberOutcomeKind,
};
pub use facts::{ExplanationFact, ProvenanceFact};
pub use failure::{
    ExecutionFailureContext, ExecutionFailurePhase, FailureSummary, RollbackDiagnostic,
};
pub use flow::{
    ApplySummary, ChangeInputSummary, FlowCauseSample, FlowSummary, InvalidationSummary,
    PlanningSummary, PrecomputeSummary, RetainedFlowSummaryView, RollbackSummary,
};
pub use history::{
    inspect_execution, inspect_flow, inspect_graph, inspect_plan, inspect_report,
    ExecutionInspector, FlowInspector, GraphInspector, PlanInspector, ReportInspector,
};
pub use lineage::LineageRecord as LineageEvent;
pub use lineage::{
    ArtifactTransitionKind, InvalidationCause, LineageArtifactId, LineageRecordKind,
    RetainedLineageView, SnapshotRestoreKind, SynthesizedLineageChain,
};
pub use policy::{
    ArtifactRetentionPolicy, DiagnosticsAvailability, FrontierCyclePolicy,
    FrontierPropagationPolicy, FrontierTracingPolicy, ReconstructionBudget, ReplayDetailPolicy,
    RetentionBudget, SemanticRetentionPolicy, SnapshotRestoreLineageMode,
};
pub use profile::DiagnosticsTier as DiagnosticsLevel;
pub use replay::ReplaySlice as ReplayView;
pub use replay::{
    ReplayCursor, ReplayEvent, ReplayEventKind, ReplayFrame, RetainedReplayView,
    SynthesizedReplaySlice,
};
pub use summary::{
    EvaluationPlanSummary, ExecutionHistoryNodeSummary, ExecutionHistorySummary,
    ExecutionReportSummary, ExplanationSummary, GraphSummary, RetainedExecutionHistoryView,
    TemporalCostContractSummary, TemporalDiagnosticsSummary, TemporalPerformanceFailureMode,
};
