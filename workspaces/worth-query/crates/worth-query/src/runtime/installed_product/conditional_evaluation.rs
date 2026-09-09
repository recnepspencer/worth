mod budget;
mod cache;
mod entry;
mod preparation;
mod retention;

use std::sync::Arc;

use worth_query_execution::facade::primary_graph::WorthQueryProductBranchLease;
use worth_runtime_bridge::facade::{
    BridgeConditionalDenial, BridgeConditionalDenialKind, BridgeConditionalExecutionRequest,
};

use super::WorthQueryInstalledProduct;
use entry::WorthQueryConditionalEvaluationEntry;

type WorthQueryConditionalExecutionStop = (
    BridgeConditionalDenialKind,
    String,
    worth_signal::facade::SignalConditionalDecisionCounters,
    usize,
);

pub(crate) struct WorthQueryExecutedConditional {
    decision: worth_runtime_bridge::facade::BridgeConditionalDecisionEvidence,
    product: Arc<WorthQueryProductBranchLease>,
}

impl WorthQueryExecutedConditional {
    pub(crate) fn into_parts(
        self,
    ) -> (
        worth_runtime_bridge::facade::BridgeConditionalDecisionEvidence,
        Arc<WorthQueryProductBranchLease>,
    ) {
        (self.decision, self.product)
    }
}

pub(super) struct WorthQueryConditionalEvaluationRegistry {
    cache: cache::WorthQueryBoundedIdleCache<WorthQueryConditionalEvaluationEntry>,
}

impl WorthQueryInstalledProduct {
    pub(in crate::runtime) fn execute_conditional(
        &self,
        selected: &Arc<WorthQueryProductBranchLease>,
        request: BridgeConditionalExecutionRequest<'_>,
        context: &mut dyn std::any::Any,
    ) -> Result<WorthQueryExecutedConditional, WorthQueryConditionalExecutionStop> {
        self.validate_selected_source(selected, request.bridge_snapshot_identity)
            .map_err(|(kind, detail)| (kind, detail.to_string(), Default::default(), 0))?;
        self.conditional_evaluations
            .execute(self, selected, request, context)
    }
}

impl WorthQueryConditionalEvaluationRegistry {
    pub(super) fn new(
        budget: crate::runtime::WorthQueryConditionalEvaluationCacheBudget,
    ) -> Result<Self, crate::runtime::WorthQueryConditionalEvaluationCacheBudgetDenial> {
        Ok(Self {
            cache: cache::WorthQueryBoundedIdleCache::new(
                budget,
                budget::retained_entry_allocation_charge().map_err(|_| {
                    crate::runtime::WorthQueryConditionalEvaluationCacheBudgetDenial::InsufficientRetainedBytes {
                        required: u64::MAX,
                        available: budget.maximum_retained_bytes(),
                    }
                })?,
            )?,
        })
    }

    pub(super) fn observe(&self) -> cache::WorthQueryConditionalEvaluationCacheObservation {
        self.cache.observe()
    }

    fn execute(
        &self,
        installed: &WorthQueryInstalledProduct,
        product: &Arc<WorthQueryProductBranchLease>,
        request: BridgeConditionalExecutionRequest<'_>,
        context: &mut dyn std::any::Any,
    ) -> Result<WorthQueryExecutedConditional, WorthQueryConditionalExecutionStop> {
        let entry = self
            .cache
            .find_or_insert(
                |entry| entry.matches(product, request.lowering),
                || WorthQueryConditionalEvaluationEntry::new(product, request.lowering),
            )
            .map_err(|_| admission_capacity_stop())?;
        let session = loop {
            match entry.admit_session(installed) {
                Ok(session) => break session,
                Err(denial)
                    if denial.kind()
                        == BridgeConditionalDenialKind::ConditionalEvaluationAdmissionCapacity
                        && self.cache.evict_oldest_idle_except(&entry) => {}
                Err(denial) => return Err(denial_parts(denial)),
            }
        };
        entry
            .execute(&session, installed, request, context)
            .map_err(denial_parts)
    }
}

fn admission_capacity_stop() -> WorthQueryConditionalExecutionStop {
    (
        BridgeConditionalDenialKind::ConditionalEvaluationAdmissionCapacity,
        "all installed conditional evaluation slots are actively retained".to_string(),
        Default::default(),
        0,
    )
}

fn denial_parts(denial: BridgeConditionalDenial) -> WorthQueryConditionalExecutionStop {
    (
        denial.kind(),
        denial.detail().to_string(),
        denial.signal_counters(),
        denial.semantic_observation_reads(),
    )
}
