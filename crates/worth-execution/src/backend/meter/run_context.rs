use std::sync::Arc;

use crate::authority::ExecutionResourceLease;

use super::{activity, PhysicalActivity, RunLimits, WorkerActivity, ACTIVE_METER};

impl RunLimits {
    pub(crate) fn for_run(lease: Option<&ExecutionResourceLease<'_>>, serial: bool) -> Self {
        Self::for_run_with_activity(lease, serial, None, false)
    }

    /// A prepared ticket lends its retained physical ledger, while all
    /// cancellation, deadline, and remaining-work limits are read afresh.
    pub(super) fn for_run_with_activity(
        lease: Option<&ExecutionResourceLease<'_>>,
        serial: bool,
        prepared_activity: Option<Arc<PhysicalActivity>>,
        prepared_cell_charged: bool,
    ) -> Self {
        let parent = ACTIVE_METER.with(|active| active.borrow().last().cloned());
        let parent_tokens = parent
            .as_ref()
            .map_or(0, |meter| meter.borrow().limits.tokens.len());
        let lease_tokens = lease.map_or(0, ExecutionResourceLease::lineage_depth);
        let mut tokens = Vec::with_capacity(
            parent_tokens
                .checked_add(lease_tokens)
                .expect("checked framework context token count"),
        );
        let (
            deadline,
            ceiling,
            worker_activity,
            physical_activity,
            physical_cell_charged,
            inherited_serial,
            inherited_lease,
        ) = if let Some(parent) = parent {
            let parent = parent.borrow();
            tokens.extend(parent.limits.tokens.iter().cloned());
            (
                parent.limits.deadline,
                parent.limits.ceiling.saturating_sub(parent.work),
                Arc::clone(&parent.limits.worker_activity),
                Arc::clone(&parent.limits.physical_activity),
                false,
                parent.limits.force_serial,
                parent.limits.bound_to_lease,
            )
        } else {
            let certification = activity::active_certification_physical();
            let (physical, charged) = match (prepared_activity, certification) {
                (Some(physical), _) => (physical, prepared_cell_charged),
                (None, Some(physical)) => (physical, false),
                (None, None) => (Arc::new(PhysicalActivity::default()), true),
            };
            (
                None,
                u64::MAX,
                Arc::new(WorkerActivity::default()),
                physical,
                charged,
                false,
                false,
            )
        };
        let mut limits = Self {
            tokens,
            deadline,
            ceiling,
            worker_activity,
            physical_activity,
            physical_cell_charged,
            force_serial: serial || inherited_serial,
            bound_to_lease: lease.is_some() || inherited_lease,
        };
        if let Some(lease) = lease {
            limits.tokens.extend(lease.cancellation_lineage());
            limits.deadline = match (limits.deadline, lease.deadline()) {
                (Some(parent), Some(child)) => Some(parent.min(child)),
                (Some(parent), None) => Some(parent),
                (None, child) => child,
            };
            limits.ceiling = limits.ceiling.min(lease.policy().budget().work_ceiling());
        }
        limits
    }

    /// Narrows the ceiling to a computation's declared work, never widening it.
    pub(crate) fn within_work_ceiling(mut self, work_ceiling: u64) -> Self {
        self.ceiling = self.ceiling.min(work_ceiling);
        self
    }
}
