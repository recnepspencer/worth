use crate::access::execution::AccessPathCounterSnapshot;
use crate::access::shape::{AccessLaneClassification, AccessShape};
use crate::access::AdmittedAccessIntent;
use worth_store_budgets::{PreExecutionBudgetRequest, PreExecutionBudgetScope};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AccessPlanCostClass {
    LsmLookup,
    LsmRunPublication,
    LsmReplayRecovery,
    LsmCompaction,
    DegradedExactScan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessPlanCostDenial {
    DegradedRowDemandNotRepresentable { requested_rows: u64, maximum: u64 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessPlanCostEstimate {
    class: AccessPlanCostClass,
    operation_counters: AccessPathCounterSnapshot,
    estimated_memory_bytes: u64,
    estimated_page_reads: u16,
    estimated_chunk_reads: u16,
    estimated_range_touches: u16,
    estimated_byte_reads: u64,
    exact_coverage: Option<crate::materialization::LayoutCoverageWitness>,
}

impl AccessPlanCostEstimate {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn issue(
        class: AccessPlanCostClass,
        operation_counters: AccessPathCounterSnapshot,
        estimated_memory_bytes: u64,
        estimated_page_reads: u16,
        estimated_chunk_reads: u16,
        estimated_range_touches: u16,
        estimated_byte_reads: u64,
        materialization: Option<crate::AdmittedLayoutMaterialization>,
    ) -> Self {
        Self {
            class,
            operation_counters,
            estimated_memory_bytes,
            estimated_page_reads,
            estimated_chunk_reads,
            estimated_range_touches,
            estimated_byte_reads,
            exact_coverage: materialization.map(|value| value.coverage().clone()),
        }
    }

    pub(in crate::planning) const fn to_budget_request(
        &self,
        scope: PreExecutionBudgetScope,
    ) -> PreExecutionBudgetRequest {
        PreExecutionBudgetRequest::new(
            scope,
            self.estimated_memory_bytes,
            self.estimated_page_reads,
            self.estimated_chunk_reads,
            self.estimated_range_touches,
            self.estimated_byte_reads,
        )
    }

    pub(in crate::planning) const fn budget_scope_for(
        intent: AdmittedAccessIntent,
    ) -> PreExecutionBudgetScope {
        match intent.shape() {
            AccessShape::ChunkTreeWalk
            | AccessShape::StreamingRead
            | AccessShape::StreamingContinuationRead => PreExecutionBudgetScope::Streaming,
            _ => match intent.lane() {
                AccessLaneClassification::Foreground => PreExecutionBudgetScope::Foreground,
                AccessLaneClassification::Maintenance => PreExecutionBudgetScope::Maintenance,
                AccessLaneClassification::Verifier => PreExecutionBudgetScope::Verifier,
                AccessLaneClassification::Terminal => PreExecutionBudgetScope::Terminal,
            },
        }
    }

    pub const fn class(&self) -> AccessPlanCostClass {
        self.class
    }

    pub const fn operation_counters(&self) -> AccessPathCounterSnapshot {
        self.operation_counters
    }

    pub const fn estimated_memory_bytes(&self) -> u64 {
        self.estimated_memory_bytes
    }

    pub const fn estimated_page_reads(&self) -> u16 {
        self.estimated_page_reads
    }

    pub const fn estimated_chunk_reads(&self) -> u16 {
        self.estimated_chunk_reads
    }

    pub const fn estimated_range_touches(&self) -> u16 {
        self.estimated_range_touches
    }

    pub const fn estimated_byte_reads(&self) -> u64 {
        self.estimated_byte_reads
    }

    pub fn materialization_source(&self) -> Option<&crate::LayoutMaterializationSourceIdentity> {
        self.exact_coverage
            .as_ref()
            .map(|coverage| coverage.source())
    }

    pub const fn exact_coverage(&self) -> Option<&crate::materialization::LayoutCoverageWitness> {
        self.exact_coverage.as_ref()
    }
}
