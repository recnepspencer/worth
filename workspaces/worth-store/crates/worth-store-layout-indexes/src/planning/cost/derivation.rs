use super::{AccessPlanCostClass, AccessPlanCostDenial, AccessPlanCostEstimate};
use crate::access::budget::PlannedCounterEnvelope;
use crate::access::AdmittedAccessIntent;
use crate::planning::candidates::EligibleStrategyOperation;

const PAGE_BYTES: u64 = 4_096;
const CHUNK_NODE_BYTES: u64 = 8_192;
const DEGRADED_ROW_WORKING_BYTES: u64 = 64;

pub(in crate::planning) fn derive_access_plan_cost(
    operation: Option<EligibleStrategyOperation>,
    intent: AdmittedAccessIntent,
    envelope: PlannedCounterEnvelope,
    materialization: Option<crate::AdmittedLayoutMaterialization>,
) -> Result<AccessPlanCostEstimate, AccessPlanCostDenial> {
    let (class, counters, degraded_rows) = match operation {
        Some(EligibleStrategyOperation::LsmLookup) => {
            (AccessPlanCostClass::LsmLookup, envelope.lookup(), None)
        }
        Some(EligibleStrategyOperation::LsmRunPublication) => (
            AccessPlanCostClass::LsmRunPublication,
            envelope.publication(),
            None,
        ),
        Some(EligibleStrategyOperation::LsmReplayRecovery) => (
            AccessPlanCostClass::LsmReplayRecovery,
            envelope.recovery(),
            None,
        ),
        Some(EligibleStrategyOperation::LsmCompaction) => (
            AccessPlanCostClass::LsmCompaction,
            envelope.publication(),
            None,
        ),
        None => {
            let requested_rows = intent.budget_rows().unwrap_or(0);
            if requested_rows > u16::MAX as u64 {
                return Err(AccessPlanCostDenial::DegradedRowDemandNotRepresentable {
                    requested_rows,
                    maximum: u16::MAX as u64,
                });
            }
            (
                AccessPlanCostClass::DegradedExactScan,
                envelope.lookup(),
                Some(requested_rows),
            )
        }
    };

    let page_reads = counters.page_touches();
    let chunk_reads = counters.chunk_tree_node_reads();
    let range_touches = degraded_rows.map_or_else(
        || {
            counters
                .range_steps()
                .saturating_add(counters.prefix_steps())
        },
        |rows| rows as u16,
    );
    let memory_bytes = (page_reads as u64)
        .saturating_mul(PAGE_BYTES)
        .saturating_add((chunk_reads as u64).saturating_mul(CHUNK_NODE_BYTES))
        .saturating_add(
            degraded_rows
                .unwrap_or(0)
                .saturating_mul(DEGRADED_ROW_WORKING_BYTES),
        );
    let byte_reads = if let Some(rows) = degraded_rows {
        counters
            .bytes_read()
            .saturating_add(rows.saturating_mul(DEGRADED_ROW_WORKING_BYTES))
    } else {
        counters.bytes_read()
    };

    Ok(AccessPlanCostEstimate::issue(
        class,
        counters,
        memory_bytes,
        page_reads,
        chunk_reads,
        range_touches,
        byte_reads,
        materialization,
    ))
}
