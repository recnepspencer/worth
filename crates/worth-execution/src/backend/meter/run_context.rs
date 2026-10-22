use std::sync::Arc;

use crate::authority::{ExecutionResourceLease, SerialRequest};

use super::{
    activity, NestedComputationStop, NestedStopOwner, PhysicalActivity, RunLimits, WorkerActivity,
    ACTIVE_METER,
};

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
            ceiling_owner,
            worker_activity,
            physical_activity,
            physical_cell_charged,
            inherited_serial,
            inherited_lease,
            serial_memory,
        ) = if let Some(parent) = parent {
            let parent = parent.borrow();
            tokens.extend(parent.limits.tokens.iter().cloned());
            (
                parent.limits.deadline,
                parent
                    .limits
                    .ceiling
                    .map(|ceiling| ceiling.saturating_sub(parent.work())),
                parent.limits.ceiling_owner,
                Arc::clone(&parent.limits.worker_activity),
                Arc::clone(&parent.limits.physical_activity),
                false,
                parent.limits.force_serial,
                parent.limits.bound_to_lease,
                parent.limits.serial_memory.clone(),
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
                None,
                NestedStopOwner::Request,
                Arc::new(WorkerActivity::default()),
                physical,
                charged,
                false,
                false,
                None,
            )
        };
        let mut limits = Self {
            tokens,
            deadline,
            ceiling,
            ceiling_owner,
            worker_activity,
            physical_activity,
            physical_cell_charged,
            force_serial: serial || inherited_serial,
            bound_to_lease: lease.is_some() || inherited_lease,
            serial_memory: if lease.is_some() { None } else { serial_memory },
            accounted: true,
        };
        if let Some(lease) = lease {
            limits.tokens.extend(lease.cancellation_lineage());
            limits.deadline = match (limits.deadline, lease.deadline()) {
                (Some(parent), Some(child)) => Some(parent.min(child)),
                (Some(parent), None) => Some(parent),
                (None, child) => child,
            };
            let requested = lease
                .policy()
                .accounting()
                .and_then(|budget| budget.work_ceiling());
            if requested
                .is_some_and(|requested| limits.ceiling.is_none_or(|ceiling| requested <= ceiling))
            {
                limits.ceiling = requested;
                limits.ceiling_owner = NestedStopOwner::Request;
            }
        }
        limits
    }

    /// Native work inherits control and real accounted ancestors, without
    /// constructing an absent meter from an unlimited budget.
    pub(crate) fn for_native(lease: Option<&ExecutionResourceLease<'_>>, serial: bool) -> Self {
        let mut limits = Self::for_run(lease, serial);
        limits.accounted = Self::native_accounting(lease);
        limits
    }

    pub(crate) fn is_accounted(&self) -> bool {
        self.accounted
    }

    pub(crate) fn native_accounting(lease: Option<&ExecutionResourceLease<'_>>) -> bool {
        Self::parent_accounted() || lease.is_some_and(ExecutionResourceLease::has_accounting)
    }

    /// Validate the existing allocation scope without inventing a memory ticket.
    pub(crate) fn check_native_scope(
        lease: Option<&ExecutionResourceLease<'_>>,
    ) -> Result<(), crate::LeaseDenial> {
        match lease {
            Some(lease) if !lease.accepts_current_worker_context() => {
                Err(crate::LeaseDenial::unrelated_nested_lease())
            }
            Some(_) => Ok(()),
            None if Self::has_leased_parent() => Err(crate::LeaseDenial::unrelated_nested_lease()),
            None if Self::has_parent() => Ok(()),
            None => Err(crate::LeaseDenial::refused(
                crate::LeaseDenial::NoActiveExecutionScope,
            )),
        }
    }

    pub(crate) fn parent_accounted() -> bool {
        ACTIVE_METER.with(|active| {
            active
                .borrow()
                .last()
                .is_some_and(|parent| parent.borrow().accounting.is_some())
        })
    }

    /// Installs a lease-free request's cancellation, deadline and memory.
    pub(crate) fn within_serial(mut self, request: Option<&SerialRequest>) -> Self {
        let Some(request) = request else {
            return self;
        };
        self.accounted |= request.memory().is_some();
        self.tokens.push(request.cancellation.clone());
        self.deadline = match (self.deadline, request.deadline) {
            (Some(parent), Some(own)) => Some(parent.min(own)),
            (parent, own) => parent.or(own),
        };
        if request
            .work_ceiling
            .is_some_and(|requested| self.ceiling.is_none_or(|ceiling| requested <= ceiling))
        {
            self.ceiling = request.work_ceiling;
            self.ceiling_owner = NestedStopOwner::Request;
        }
        if let Some(memory) = request.memory() {
            self.serial_memory = Some(match self.serial_memory {
                Some(parent) => memory.within_parent(&parent),
                None => memory.clone(),
            });
        }
        self
    }

    /// Narrows the ceiling to a computation's declared work, never widening it.
    pub(crate) fn within_work_ceiling(mut self, work_ceiling: Option<u64>) -> Self {
        if work_ceiling
            .is_some_and(|requested| self.ceiling.is_none_or(|ceiling| requested < ceiling))
        {
            self.accounted = true;
            self.ceiling = work_ceiling;
            self.ceiling_owner = NestedStopOwner::Computation(NestedComputationStop::WorkCeiling);
        }
        self
    }
}
