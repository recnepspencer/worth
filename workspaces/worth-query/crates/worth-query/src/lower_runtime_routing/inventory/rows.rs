use super::{
    WorthQueryLowerRuntimeDirectImportAudit, WorthQueryLowerRuntimeDirectImportAuditRow,
    WorthQueryLowerRuntimeDirectImportPosture, WorthQueryLowerRuntimeGapRegistry,
    WorthQueryLowerRuntimeGapRegistryRow, WorthQueryLowerRuntimeSeamKey,
};

const GAP_ROWS: &[WorthQueryLowerRuntimeGapRegistryRow] = &[];

const DIRECT_IMPORT_AUDIT_ROWS: &[WorthQueryLowerRuntimeDirectImportAuditRow] = &[
    WorthQueryLowerRuntimeDirectImportAuditRow::new(
        WorthQueryLowerRuntimeSeamKey::RuntimeBackendBoundaryModules,
        "crates/worth-query/src/runtime/backend/*",
        WorthQueryLowerRuntimeDirectImportPosture::RuntimeBackendBoundary,
        "backend-boundary modules may import lower-runtime facades because they are the canonical ordinary crossing hub",
    ),
    WorthQueryLowerRuntimeDirectImportAuditRow::new(
        WorthQueryLowerRuntimeSeamKey::HistoricalBridgeLoweringModule,
        "crates/worth-query/src/historical/bridge_lowering.rs",
        WorthQueryLowerRuntimeDirectImportPosture::AllowedAdapter,
        "historical policy lowering remains an allowed Query boundary adapter",
    ),
    WorthQueryLowerRuntimeDirectImportAuditRow::new(
        WorthQueryLowerRuntimeSeamKey::PublicLiveViewDeclaration,
        "crates/worth-query/src/runtime/bridge_async_live_view_declaration.rs",
        WorthQueryLowerRuntimeDirectImportPosture::AllowedAdapter,
        "bridge-backed async live declaration remains an allowed Query boundary adapter over an admitted Runtime Bridge request",
    ),
    WorthQueryLowerRuntimeDirectImportAuditRow::new(
        WorthQueryLowerRuntimeSeamKey::EffectBackedRelationalMutation,
        "crates/worth-query/src/effect_lifecycle/execution.rs",
        WorthQueryLowerRuntimeDirectImportPosture::AllowedAdapter,
        "effect-backed relational mutation execution remains an allowed Query boundary adapter over admitted relational authority",
    ),
    WorthQueryLowerRuntimeDirectImportAuditRow::new(
        WorthQueryLowerRuntimeSeamKey::EffectBackedRelationalMerge,
        "crates/worth-query/src/effect_lifecycle/execution.rs",
        WorthQueryLowerRuntimeDirectImportPosture::AllowedAdapter,
        "effect-backed relational merge execution remains an allowed Query boundary adapter over admitted relational authority",
    ),
    WorthQueryLowerRuntimeDirectImportAuditRow::new(
        WorthQueryLowerRuntimeSeamKey::ProjectionConsumptionSourceModule,
        "crates/worth-query/src/projection_consumption/source/mod.rs",
        WorthQueryLowerRuntimeDirectImportPosture::AllowedAdapter,
        "projection-consumption source intake remains an allowed Query boundary adapter",
    ),
    WorthQueryLowerRuntimeDirectImportAuditRow::new(
        WorthQueryLowerRuntimeSeamKey::CausalBuilderBridgeModule,
        "crates/worth-query-execution/src/domain_computation/primary_graph/causal_inspection/builder_bridge.rs",
        WorthQueryLowerRuntimeDirectImportPosture::AllowedAdapter,
        "causal bridge materialization remains an allowed Query boundary adapter",
    ),
    WorthQueryLowerRuntimeDirectImportAuditRow::new(
        WorthQueryLowerRuntimeSeamKey::FrontierSignalAdapterModule,
        "crates/worth-query/src/frontier_signal_adapter.rs",
        WorthQueryLowerRuntimeDirectImportPosture::AllowedAdapter,
        "frontier evidence intake remains an allowed Query boundary adapter over signal facade receipts",
    ),
    WorthQueryLowerRuntimeDirectImportAuditRow::new(
        WorthQueryLowerRuntimeSeamKey::EffectExecutionBridgeModule,
        "crates/worth-query/src/effect_lifecycle/execution_bridge.rs",
        WorthQueryLowerRuntimeDirectImportPosture::AllowedAdapter,
        "bridge writeback execution remains an allowed Query boundary adapter over the bridge admitted execution contract",
    ),
    WorthQueryLowerRuntimeDirectImportAuditRow::new(
        WorthQueryLowerRuntimeSeamKey::IntentRuntimeExecution,
        "crates/worth-query/src/runtime/backend/intent_authority.rs",
        WorthQueryLowerRuntimeDirectImportPosture::AllowedAdapter,
        "installed intent authority remains an allowed backend-boundary adapter over runtime bridge and relational authority contracts",
    ),
];

pub fn worth_query_lower_runtime_gap_registry() -> WorthQueryLowerRuntimeGapRegistry {
    WorthQueryLowerRuntimeGapRegistry::new(GAP_ROWS)
}

pub fn worth_query_lower_runtime_direct_import_audit() -> WorthQueryLowerRuntimeDirectImportAudit {
    WorthQueryLowerRuntimeDirectImportAudit::new(DIRECT_IMPORT_AUDIT_ROWS)
}
