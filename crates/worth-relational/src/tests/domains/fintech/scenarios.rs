use super::fixture::{FintechCaseRole, FintechWorld};
use super::scales::FintechScale;

#[derive(Clone, Copy)]
pub(super) enum FintechScenario {
    SmokeBook,
    PersistedSmokeBook,
    HistoricalVisibility,
    IntradayRisk,
    LateTradeCorrection,
    FailedSettlementRepair,
}

pub(super) struct FintechScenarioSelection {
    pub(super) scenario: FintechScenario,
    pub(super) scenario_key: &'static str,
    pub(super) canonical_case: FintechCaseRole,
    pub(super) expected_invariants: &'static [&'static str],
    pub(super) expected_artifacts: &'static [&'static str],
    pub(super) expected_read_alias: &'static str,
    pub(super) probe_prefix: &'static str,
    pub(super) persisted: bool,
}

pub(super) fn setup_world(execution: &worth_execution::ExecutionRequest<'_, '_>) -> FintechWorld {
    FintechWorld::setup_world(execution)
}

pub(super) fn setup_world_with(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    scale: FintechScale,
) -> FintechWorld {
    FintechWorld::setup_world_with(execution, scale)
}

pub(super) fn setup_world_for(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    scenario: FintechScenario,
) -> FintechWorld {
    match scenario {
        FintechScenario::SmokeBook => FintechWorld::setup_world(execution),
        FintechScenario::PersistedSmokeBook => FintechWorld::setup_persisted_world(execution),
        FintechScenario::HistoricalVisibility => FintechWorld::setup_world(execution),
        FintechScenario::IntradayRisk => FintechWorld::setup_world(execution),
        FintechScenario::LateTradeCorrection => FintechWorld::setup_world(execution),
        FintechScenario::FailedSettlementRepair => FintechWorld::setup_world(execution),
    }
}

pub(super) fn selection_for(scenario: FintechScenario) -> FintechScenarioSelection {
    match scenario {
        FintechScenario::SmokeBook => FintechScenarioSelection {
            scenario,
            scenario_key: "smoke-book",
            canonical_case: FintechCaseRole::BaselinePortfolio,
            expected_invariants: &["fixture_shape_smoke", "truth_world_named"],
            expected_artifacts: &["snapshot-visible-truth", "branch-head-state"],
            expected_read_alias: "portfolio.read.baseline",
            probe_prefix: "portfolio",
            persisted: false,
        },
        FintechScenario::PersistedSmokeBook => FintechScenarioSelection {
            scenario,
            scenario_key: "persisted-smoke-book",
            canonical_case: FintechCaseRole::BaselinePortfolio,
            expected_invariants: &[
                "fixture_shape_smoke",
                "truth_world_named",
                "recovery_queryable",
            ],
            expected_artifacts: &[
                "snapshot-visible-truth",
                "branch-head-state",
                "replay-recovery-truth",
            ],
            expected_read_alias: "portfolio.read.recovery",
            probe_prefix: "portfolio",
            persisted: true,
        },
        FintechScenario::HistoricalVisibility => FintechScenarioSelection {
            scenario,
            scenario_key: "historical-visibility",
            canonical_case: FintechCaseRole::LateTradeCorrection,
            expected_invariants: &[
                "snapshot_history_stable",
                "released_snapshot_unreadable",
                "version_read_stable",
            ],
            expected_artifacts: &["snapshot-visible-truth", "branch-head-state"],
            expected_read_alias: "trade-correction.read.historical",
            probe_prefix: "historical-visibility",
            persisted: false,
        },
        FintechScenario::IntradayRisk => FintechScenarioSelection {
            scenario,
            scenario_key: "intraday-risk",
            canonical_case: FintechCaseRole::IntradayRisk,
            expected_invariants: &["open_breach_visible", "analysis_branch_local"],
            expected_artifacts: &[
                "snapshot-visible-truth",
                "replay-recovery-truth",
                "diagnostics",
            ],
            expected_read_alias: "intraday-risk.read.post-mutation",
            probe_prefix: "intraday-risk",
            persisted: false,
        },
        FintechScenario::LateTradeCorrection => FintechScenarioSelection {
            scenario,
            scenario_key: "late-trade-correction",
            canonical_case: FintechCaseRole::LateTradeCorrection,
            expected_invariants: &[
                "correction_visible",
                "audit_truth_visible",
                "analysis_branch_local",
            ],
            expected_artifacts: &["snapshot-visible-truth", "branch-head-state", "diagnostics"],
            expected_read_alias: "trade-correction.read.post-mutation",
            probe_prefix: "trade-correction",
            persisted: false,
        },
        FintechScenario::FailedSettlementRepair => FintechScenarioSelection {
            scenario,
            scenario_key: "failed-settlement-repair",
            canonical_case: FintechCaseRole::FailedSettlementRepair,
            expected_invariants: &[
                "settlement_repaired",
                "repair_audit_visible",
                "analysis_branch_local",
            ],
            expected_artifacts: &[
                "snapshot-visible-truth",
                "replay-recovery-truth",
                "diagnostics",
            ],
            expected_read_alias: "settlement-repair.read.post-mutation",
            probe_prefix: "settlement-repair",
            persisted: false,
        },
    }
}

pub(super) fn setup_selected_world(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    scenario: FintechScenario,
) -> (FintechWorld, FintechScenarioSelection) {
    let selection = selection_for(scenario);
    (setup_world_for(execution, scenario), selection)
}

pub(super) fn setup_world_for_intraday_risk(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> FintechWorld {
    setup_world_for(execution, FintechScenario::IntradayRisk)
}

pub(super) fn setup_world_for_historical_visibility(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> FintechWorld {
    setup_world_for(execution, FintechScenario::HistoricalVisibility)
}

pub(super) fn setup_world_for_late_trade_correction(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> FintechWorld {
    setup_world_for(execution, FintechScenario::LateTradeCorrection)
}

pub(super) fn setup_world_for_failed_settlement_repair(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> FintechWorld {
    setup_world_for(execution, FintechScenario::FailedSettlementRepair)
}
