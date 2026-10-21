use crate::identity::hash_parts;
use crate::runtime::{
    WorthQueryAdmittedGraphReadAccessPlan, WorthQueryGraphReadAccessPlanConsumption,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphReadAccessComplexityCounters {
    digest: String,
    executor_entry_count: usize,
    executor_strategy_rediscovery_count: usize,
    edge_scan_execution_count: usize,
    per_result_neighbor_lookup_count: usize,
    planned_access_step_count: usize,
    consumed_access_step_count: usize,
    persistent_artifact_bypass_count: usize,
}

impl WorthQueryGraphReadAccessComplexityCounters {
    pub fn digest(&self) -> &str {
        &self.digest
    }

    pub fn executor_entry_count(&self) -> usize {
        self.executor_entry_count
    }

    pub fn executor_strategy_rediscovery_count(&self) -> usize {
        self.executor_strategy_rediscovery_count
    }

    pub fn edge_scan_execution_count(&self) -> usize {
        self.edge_scan_execution_count
    }

    pub fn per_result_neighbor_lookup_count(&self) -> usize {
        self.per_result_neighbor_lookup_count
    }

    pub fn planned_access_step_count(&self) -> usize {
        self.planned_access_step_count
    }

    pub fn consumed_access_step_count(&self) -> usize {
        self.consumed_access_step_count
    }

    pub fn persistent_artifact_bypass_count(&self) -> usize {
        self.persistent_artifact_bypass_count
    }

    pub(in crate::runtime) fn from_execution_parts(
        plan: &WorthQueryAdmittedGraphReadAccessPlan,
        plan_consumption: &WorthQueryGraphReadAccessPlanConsumption,
    ) -> Self {
        let execution = plan_consumption.execution_counters();
        let planned_access_step_count = plan.admission().requirement_set().rows().len();
        let consumed_access_step_count =
            planned_access_step_count * execution.executor_entry_count();
        let digest = hash_parts(&[
            "worth_query_graph_read_access_complexity_counters_v1".to_string(),
            format!("plan:{}", plan.digest()),
            format!("consumption:{}", plan_consumption.digest()),
            format!("executor_entry:{}", execution.executor_entry_count()),
            format!(
                "strategy_rediscovery:{}",
                execution.strategy_recompute_count()
            ),
            format!("edge_scan:{}", execution.edge_scan_count()),
            format!(
                "per_result_neighbor_lookup:{}",
                execution.per_result_neighbor_lookup_count()
            ),
            format!("planned_access_steps:{planned_access_step_count}"),
            format!("consumed_access_steps:{consumed_access_step_count}"),
            format!(
                "persistent_artifact_bypass:{}",
                execution.persistent_artifact_bypass_count()
            ),
        ]);
        Self {
            digest,
            executor_entry_count: execution.executor_entry_count(),
            executor_strategy_rediscovery_count: execution.strategy_recompute_count(),
            edge_scan_execution_count: execution.edge_scan_count(),
            per_result_neighbor_lookup_count: execution.per_result_neighbor_lookup_count(),
            planned_access_step_count,
            consumed_access_step_count,
            persistent_artifact_bypass_count: execution.persistent_artifact_bypass_count(),
        }
    }
}
