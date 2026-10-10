use std::mem::size_of;

use crate::authority::{CancellationToken, ExecutionResourceLease};

use super::{activity, KernelMeter, PhysicalActivity, RunLimits, WorkerActivity, ACTIVE_METER};

impl RunLimits {
    /// The serial map inherits only the active parent's checkpoint lineage.
    /// Prospective admission must not allocate a replacement run context.
    pub(crate) fn framework_context_bytes_for_serial_scope(partitions: usize) -> Option<u64> {
        let (tokens, has_parent) = ACTIVE_METER.with(|active| {
            let active = active.borrow();
            (
                active
                    .last()
                    .map_or(0, |parent| parent.borrow().limits.tokens.len()),
                !active.is_empty(),
            )
        });
        Self::context_bytes_for_tokens(
            tokens,
            partitions,
            1,
            !has_parent && activity::active_certification_physical().is_none(),
        )
    }

    pub(crate) fn framework_context_bytes(
        &self,
        partitions: usize,
        max_workers: usize,
    ) -> Option<u64> {
        Self::context_bytes_for_tokens(
            self.tokens.len(),
            partitions,
            max_workers,
            self.physical_cell_charged,
        )
    }

    /// Read the active lineage length without cloning its token Vec. A later
    /// checked run builds that same parent-plus-lease lineage before reserve.
    pub(crate) fn framework_context_bytes_for_lease(
        lease: &ExecutionResourceLease<'_>,
        partitions: usize,
    ) -> Option<u64> {
        let (parent_tokens, has_parent) = ACTIVE_METER.with(|active| {
            let active = active.borrow();
            (
                active
                    .last()
                    .map_or(0, |parent| parent.borrow().limits.tokens.len()),
                !active.is_empty(),
            )
        });
        let tokens = parent_tokens.checked_add(lease.lineage_depth())?;
        Self::context_bytes_for_tokens(
            tokens,
            partitions,
            lease.policy().budget().max_workers().get(),
            !has_parent && activity::active_certification_physical().is_none(),
        )
    }

    fn context_bytes_for_tokens(
        token_count: usize,
        partitions: usize,
        max_workers: usize,
        physical_cell_charged: bool,
    ) -> Option<u64> {
        let token_bytes = token_count.checked_mul(size_of::<CancellationToken>())?;
        let per_partition = size_of::<KernelMeter>()
            .checked_add(2 * size_of::<usize>())? // Rc allocation header
            .checked_add(token_bytes)? // cloned checkpoint lineage
            .checked_add(size_of::<(usize, u64)>())?; // worker context stack
        let partition_bytes = partitions.checked_mul(per_partition)?;
        let worker_guards =
            max_workers.checked_mul(size_of::<crate::authority::ResourceReservation>())?;
        let fixed = size_of::<WorkerActivity>()
            .checked_add(2 * size_of::<usize>())? // Arc allocation header
            .checked_add(token_bytes)?
            .checked_add(worker_guards)?
            .checked_add(if physical_cell_charged {
                size_of::<PhysicalActivity>().checked_add(2 * size_of::<usize>())?
            } else {
                0
            })?;
        u64::try_from(partition_bytes.checked_add(fixed)?).ok()
    }
}
