//! Translate harness profiles and observations at the presentation boundary.

use worth_harness::facade::{DiagnosticsLevel, ExecutionMode, ObservationStatus};

use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::node::{EvaluationCondition, NodeState};
use crate::diagnostics::profile::DiagnosticsTier;
use crate::logic::planner::EvaluationPlan;
use crate::runtime_policy::SignalRuntimePolicy;
use worth_foundational::ExecutionPosture;

use super::SignalHarnessBridge;

impl SignalHarnessBridge {
    #[cfg(test)]
    pub(super) fn requires_condition_aware_execution(
        graph: &SignalGraph,
        plan: &EvaluationPlan,
    ) -> Result<bool, SignalError> {
        for task in plan.stages.iter().flat_map(|stage| &stage.tasks) {
            let config = graph.node_eval_config(task.node)?;
            if !matches!(config.condition, EvaluationCondition::Always)
                || config.comparator.is_some()
            {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub(in crate::presentation::harness) fn diagnostics_profile(
        level: DiagnosticsLevel,
    ) -> DiagnosticsTier {
        match level {
            DiagnosticsLevel::Off | DiagnosticsLevel::Operational => DiagnosticsTier::Operational,
            DiagnosticsLevel::Development => DiagnosticsTier::Development,
            DiagnosticsLevel::Forensic => DiagnosticsTier::Forensic,
        }
    }

    pub(in crate::presentation::harness) fn runtime_policy(
        level: DiagnosticsLevel,
    ) -> SignalRuntimePolicy {
        SignalRuntimePolicy::for_tier(Self::diagnostics_profile(level))
    }

    pub(super) fn posture(mode: ExecutionMode) -> ExecutionPosture {
        match mode {
            ExecutionMode::RuntimeDefault | ExecutionMode::Serial => ExecutionPosture::Serial,
            ExecutionMode::StagedParallel | ExecutionMode::FullParallel => {
                ExecutionPosture::Automatic
            }
        }
    }

    pub(super) fn observation_status(state: NodeState) -> ObservationStatus {
        match state {
            NodeState::Clean => ObservationStatus::Clean,
            NodeState::MaybeStale => ObservationStatus::MaybeStale,
            NodeState::Dirty => ObservationStatus::Dirty,
        }
    }
}
