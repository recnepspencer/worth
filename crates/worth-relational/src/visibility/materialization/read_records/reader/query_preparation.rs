use worth_execution::{
    CancellationToken, ExecutionResourceLease, ExecutionScan, LeaseRequest, MapKernelContext,
    MapKernelFailure, ScanOutcome,
};
use worth_foundational::{
    ExecutionBudget, ExecutionPhysicalReport, ExecutionReport, PartitionIdentity,
};

use super::query_execution::QueryReadExecutionStop;
use super::query_packetization::PacketizedQueryWork;

/// Preparation retains at most this share of the request while it copies packets.
const PREPARATION_MEMORY_DIVISOR: u64 = 8;

pub(super) struct QueryPreparationBudget<'context, 'meter, 'lease> {
    context: &'context mut MapKernelContext<'meter, 'lease>,
    ceiling: u64,
    claimed: u64,
}

impl QueryPreparationBudget<'_, '_, '_> {
    pub(super) fn checkpoint(&mut self, units: u64) -> Result<(), MapKernelFailure<()>> {
        self.context
            .checkpoint(units)
            .map_err(MapKernelFailure::Stop)
    }

    pub(super) fn claim(&mut self, bytes: u64) -> Result<(), MapKernelFailure<()>> {
        self.claimed = self
            .claimed
            .checked_add(bytes)
            .filter(|claimed| *claimed <= self.ceiling)
            .ok_or(MapKernelFailure::ResultCapacityExceeded)?;
        Ok(())
    }

    pub(super) fn claim_items<T>(&mut self, count: usize) -> Result<(), MapKernelFailure<()>> {
        self.claim(
            u64::try_from(count)
                .unwrap_or(u64::MAX)
                .saturating_mul(std::mem::size_of::<T>() as u64),
        )
    }
}

pub(super) fn prepare_query_packets(
    lease: &ExecutionResourceLease<'_>,
    build: impl FnOnce(
        &mut QueryPreparationBudget<'_, '_, '_>,
    ) -> Result<Vec<PacketizedQueryWork>, MapKernelFailure<()>>,
) -> Result<(Vec<PacketizedQueryWork>, ExecutionReport), QueryReadExecutionStop> {
    let identity = PartitionIdentity::new(1);
    let scan = ExecutionScan::try_from_ordered(vec![identity], vec![(identity, ())])
        .map_err(QueryReadExecutionStop::PreparationAdmission)?;
    let ceiling = lease.policy().budget().charged_memory_bytes() / PREPARATION_MEMORY_DIVISOR;
    let mut build = Some(build);
    match scan.run(Some(lease), (), 0, ceiling, 0, ceiling, |_, _, context| {
        let mut budget = QueryPreparationBudget {
            context,
            ceiling,
            claimed: 0,
        };
        budget.checkpoint(0)?;
        let packets = build.take().expect("one preparation step")(&mut budget)?;
        Ok(((), packets))
    }) {
        ScanOutcome::Complete {
            mut prefixes,
            report,
            ..
        } => Ok((prefixes.pop().expect("one preparation output"), report)),
        ScanOutcome::Stopped { reason, report, .. } => {
            Err(QueryReadExecutionStop::PreparationStopped { reason, report })
        }
    }
}

pub(super) fn remaining_work_lease<'a>(
    lease: &ExecutionResourceLease<'a>,
    preparation: ExecutionReport,
) -> Result<ExecutionResourceLease<'a>, QueryReadExecutionStop> {
    let policy = *lease.policy();
    let budget = policy.budget();
    let remaining = budget
        .work_ceiling()
        .saturating_sub(preparation.charged_work());
    lease
        .child(LeaseRequest {
            policy: worth_foundational::ExecutionRequestPolicy::new(
                policy.posture(),
                policy.determinism(),
                ExecutionBudget::new(
                    budget.max_workers(),
                    budget.charged_memory_bytes(),
                    remaining,
                ),
            ),
            deadline: None,
            cancellation: CancellationToken::new(),
        })
        .map_err(QueryReadExecutionStop::PreparationLease)
}

pub(super) fn combine_query_reports(
    preparation: ExecutionReport,
    execution: ExecutionReport,
) -> ExecutionReport {
    let first = preparation.physical();
    let second = execution.physical();
    ExecutionReport::new(
        execution.resolved_posture(),
        preparation
            .charged_work()
            .saturating_add(execution.charged_work()),
        preparation
            .charged_span()
            .saturating_add(execution.charged_span()),
        ExecutionPhysicalReport::new(
            first
                .active_workers_high_watermark()
                .max(second.active_workers_high_watermark()),
            first
                .peak_charged_memory_bytes()
                .max(second.peak_charged_memory_bytes()),
            match (first.steals(), second.steals()) {
                (Some(a), Some(b)) => Some(a.saturating_add(b)),
                _ => None,
            },
            first.peak_queue_width().max(second.peak_queue_width()),
            first
                .discarded_in_flight_work()
                .saturating_add(second.discarded_in_flight_work()),
        ),
    )
}

pub(super) fn combine_query_stop(
    stop: QueryReadExecutionStop,
    preparation: ExecutionReport,
) -> QueryReadExecutionStop {
    match stop {
        QueryReadExecutionStop::PacketStopped {
            boundary,
            reason,
            report,
        } => QueryReadExecutionStop::PacketStopped {
            boundary,
            reason,
            report: combine_query_reports(preparation, report),
        },
        other => other,
    }
}
