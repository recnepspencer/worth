use super::WorthQueryAdmittedGraphReadAccessPlan;
use crate::identity::hash_parts;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphReadAccessExecutionCounters {
    executor_entry_count: usize,
    strategy_recompute_count: usize,
    ephemeral_index_allocation_count: usize,
    edge_scan_count: usize,
    per_result_neighbor_lookup_count: usize,
    persistent_artifact_bypass_count: usize,
    materialized_row_count: usize,
}

impl WorthQueryGraphReadAccessExecutionCounters {
    pub fn executor_entry_count(&self) -> usize {
        self.executor_entry_count
    }

    pub fn strategy_recompute_count(&self) -> usize {
        self.strategy_recompute_count
    }

    pub fn ephemeral_index_allocation_count(&self) -> usize {
        self.ephemeral_index_allocation_count
    }

    pub fn edge_scan_count(&self) -> usize {
        self.edge_scan_count
    }

    pub fn per_result_neighbor_lookup_count(&self) -> usize {
        self.per_result_neighbor_lookup_count
    }

    pub fn persistent_artifact_bypass_count(&self) -> usize {
        self.persistent_artifact_bypass_count
    }

    pub fn materialized_row_count(&self) -> usize {
        self.materialized_row_count
    }

    pub(crate) fn pre_execution_denial() -> Self {
        Self {
            executor_entry_count: 0,
            strategy_recompute_count: 0,
            ephemeral_index_allocation_count: 0,
            edge_scan_count: 0,
            per_result_neighbor_lookup_count: 0,
            persistent_artifact_bypass_count: 0,
            materialized_row_count: 0,
        }
    }

    pub(crate) fn observed_admitted_execution(materialized_row_count: usize) -> Self {
        WorthQueryGraphReadAccessExecutionRecorder::entered_executor()
            .with_materialized_rows(materialized_row_count)
            .finish()
    }

    pub(crate) fn digest_part(&self) -> String {
        format!(
            "access_execution_counters:executor_entry:{}:strategy_recompute:{}:ephemeral_index_allocation:{}:edge_scan:{}:per_result_neighbor_lookup:{}:persistent_artifact_bypass:{}:materialized_row:{}",
            self.executor_entry_count,
            self.strategy_recompute_count,
            self.ephemeral_index_allocation_count,
            self.edge_scan_count,
            self.per_result_neighbor_lookup_count,
            self.persistent_artifact_bypass_count,
            self.materialized_row_count
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorthQueryGraphReadAccessExecutionRecorder {
    executor_entry_count: usize,
    strategy_recompute_count: usize,
    ephemeral_index_allocation_count: usize,
    edge_scan_count: usize,
    per_result_neighbor_lookup_count: usize,
    persistent_artifact_bypass_count: usize,
    materialized_row_count: usize,
}

impl WorthQueryGraphReadAccessExecutionRecorder {
    pub(crate) fn entered_executor() -> Self {
        Self {
            executor_entry_count: 1,
            strategy_recompute_count: 0,
            ephemeral_index_allocation_count: 0,
            edge_scan_count: 0,
            per_result_neighbor_lookup_count: 0,
            persistent_artifact_bypass_count: 0,
            materialized_row_count: 0,
        }
    }

    pub(crate) fn record_materialized_rows(&mut self, row_count: usize) {
        self.materialized_row_count = row_count;
    }

    pub(crate) fn with_materialized_rows(mut self, row_count: usize) -> Self {
        self.record_materialized_rows(row_count);
        self
    }

    pub(crate) fn finish(self) -> WorthQueryGraphReadAccessExecutionCounters {
        WorthQueryGraphReadAccessExecutionCounters {
            executor_entry_count: self.executor_entry_count,
            strategy_recompute_count: self.strategy_recompute_count,
            ephemeral_index_allocation_count: self.ephemeral_index_allocation_count,
            edge_scan_count: self.edge_scan_count,
            per_result_neighbor_lookup_count: self.per_result_neighbor_lookup_count,
            persistent_artifact_bypass_count: self.persistent_artifact_bypass_count,
            materialized_row_count: self.materialized_row_count,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphReadPersistentArtifactAudit {
    durable_artifact_create_attempt_count: usize,
    durable_artifact_open_attempt_count: usize,
    durable_artifact_write_attempt_count: usize,
    declaration_only_stop_count: usize,
}

impl WorthQueryGraphReadPersistentArtifactAudit {
    pub fn durable_artifact_create_attempt_count(&self) -> usize {
        self.durable_artifact_create_attempt_count
    }

    pub fn durable_artifact_open_attempt_count(&self) -> usize {
        self.durable_artifact_open_attempt_count
    }

    pub fn durable_artifact_write_attempt_count(&self) -> usize {
        self.durable_artifact_write_attempt_count
    }

    pub fn declaration_only_stop_count(&self) -> usize {
        self.declaration_only_stop_count
    }

    pub(crate) fn declaration_only_stop() -> Self {
        Self {
            durable_artifact_create_attempt_count: 0,
            durable_artifact_open_attempt_count: 0,
            durable_artifact_write_attempt_count: 0,
            declaration_only_stop_count: 1,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphReadAccessPlanConsumption {
    digest: String,
    admitted_plan_digest: String,
    admission_digest: String,
    execution_binding_digest: String,
    execution_strategy: String,
    execution_counters: WorthQueryGraphReadAccessExecutionCounters,
}

impl WorthQueryGraphReadAccessPlanConsumption {
    pub fn digest(&self) -> &str {
        &self.digest
    }

    pub fn admitted_plan_digest(&self) -> &str {
        &self.admitted_plan_digest
    }

    pub fn plan_digest(&self) -> &str {
        &self.admitted_plan_digest
    }

    pub fn admission_digest(&self) -> &str {
        &self.admission_digest
    }

    pub fn execution_binding_digest(&self) -> &str {
        &self.execution_binding_digest
    }

    pub fn execution_strategy(&self) -> &str {
        &self.execution_strategy
    }

    pub fn execution_counters(&self) -> &WorthQueryGraphReadAccessExecutionCounters {
        &self.execution_counters
    }

    pub(crate) fn from_plan_binding_and_execution_counters(
        plan: &WorthQueryAdmittedGraphReadAccessPlan,
        execution_binding_digest: &str,
        execution_counters: WorthQueryGraphReadAccessExecutionCounters,
    ) -> Self {
        let admitted_plan_digest = plan.digest().to_string();
        let admission_digest = plan.admission().digest().to_string();
        let execution_strategy = plan.execution_strategy().to_string();
        let digest = hash_parts(&[
            "worth_query_graph_read_access_plan_consumption_v1".to_string(),
            format!("admitted_plan:{admitted_plan_digest}"),
            format!("admission:{admission_digest}"),
            format!("binding:{execution_binding_digest}"),
            format!("strategy:{execution_strategy}"),
            execution_counters.digest_part(),
        ]);
        Self {
            digest,
            admitted_plan_digest,
            admission_digest,
            execution_binding_digest: execution_binding_digest.to_string(),
            execution_strategy,
            execution_counters,
        }
    }
}
