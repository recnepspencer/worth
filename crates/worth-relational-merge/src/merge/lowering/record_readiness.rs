use std::sync::Arc;

use crate::merge::data::{
    LoweredAspectOutcome, LoweredMergePlanRecord, LoweredMergePlanSummary, MergeExecutionReadiness,
};

pub(super) fn summarize_lowered_records(
    records: Arc<[LoweredMergePlanRecord]>,
) -> LoweredMergePlanSummary {
    let mut admitted_count = 0;
    let mut blocked_count = 0;
    let mut rejected_count = 0;

    for record in records.iter() {
        match record.readiness {
            MergeExecutionReadiness::Admitted => admitted_count += 1,
            MergeExecutionReadiness::Blocked => blocked_count += 1,
            MergeExecutionReadiness::Rejected => rejected_count += 1,
        }
    }

    LoweredMergePlanSummary {
        record_count: records.len(),
        admitted_count,
        blocked_count,
        rejected_count,
        fully_execution_ready: blocked_count == 0 && rejected_count == 0,
        records,
    }
}

pub(super) fn aggregate_record_readiness(
    aspects: &[LoweredAspectOutcome],
) -> MergeExecutionReadiness {
    if aspects
        .iter()
        .any(|aspect| aspect.readiness == MergeExecutionReadiness::Rejected)
    {
        MergeExecutionReadiness::Rejected
    } else if aspects
        .iter()
        .any(|aspect| aspect.readiness == MergeExecutionReadiness::Blocked)
    {
        MergeExecutionReadiness::Blocked
    } else {
        MergeExecutionReadiness::Admitted
    }
}
