use worth_execution::{
    ChargedBytes, ExecutionResourceLease, ExecutionScan, MapKernelFailure, ScanOutcome,
};
use worth_foundational::{ExecutionReport, PartitionIdentity};

use crate::query::data::{
    reduce_query_fragments_checked, CanonicalQueryResult, QueryExecutionShape,
    QueryOrderingContract, QueryWorkerFragment,
};

use super::QueryReadExecutionStop;

pub(super) fn complete_leased_query(
    shape: QueryExecutionShape,
    ordering: QueryOrderingContract,
    fragments: Vec<QueryWorkerFragment>,
    lease: &ExecutionResourceLease<'_>,
) -> Result<(CanonicalQueryResult, ExecutionReport), QueryReadExecutionStop> {
    let identity = PartitionIdentity::new(1);
    let scan = ExecutionScan::try_from_ordered(vec![identity], vec![(identity, ())])
        .map_err(QueryReadExecutionStop::CompletionAdmission)?;
    let memory = lease.policy().budget().charged_memory_bytes();
    let captured_bytes = fragments.additional_charged_bytes();
    let scratch_ceiling = captured_bytes
        .saturating_mul(8)
        .saturating_add(4096)
        .min(memory / 3);
    let result_ceiling = captured_bytes
        .saturating_mul(2)
        .saturating_add(1024)
        .min(memory / 3);
    let mut fragments = Some(fragments);
    match scan.run(
        Some(lease),
        (),
        0,
        result_ceiling,
        0,
        scratch_ceiling,
        |_, _, context| {
            context.checkpoint(0).map_err(MapKernelFailure::Stop)?;
            context
                .checkpoint(captured_bytes.saturating_add(4095) / 4096)
                .map_err(MapKernelFailure::Stop)?;
            if captured_bytes > scratch_ceiling {
                return Err(MapKernelFailure::ResultCapacityExceeded);
            }
            let result = reduce_query_fragments_checked(
                shape,
                ordering,
                fragments.take().expect("one completion step"),
                |work, intermediate| {
                    context.checkpoint(work).map_err(MapKernelFailure::Stop)?;
                    if captured_bytes.saturating_add(intermediate) > scratch_ceiling {
                        return Err(MapKernelFailure::ResultCapacityExceeded);
                    }
                    Ok(())
                },
            )?;
            Ok(((), result))
        },
    ) {
        ScanOutcome::Complete {
            mut prefixes,
            report,
            ..
        } => Ok((prefixes.pop().expect("one completion output"), report)),
        ScanOutcome::Stopped { reason, report, .. } => {
            Err(QueryReadExecutionStop::CompletionStopped { reason, report })
        }
    }
}
