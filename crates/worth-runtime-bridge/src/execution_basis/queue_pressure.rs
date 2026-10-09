use worth_signal::facade::{
    ResourceManagedQueueMutationKind, ResourceManagedQueueMutationReport,
    ResourceQueuePressureClass,
};

use crate::source::with_async_request_signal_runtime;

use super::{BridgeBoundExecutionBasis, BridgeManagedQueueAdmission, BridgeManagedQueueOccupancy};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BridgeManagedQueueMutationKind {
    Enqueued,
    Dequeued,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BridgeExecutionQueuePressureState {
    Available,
    Saturated,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BridgeManagedQueueMutationCounters {
    exact_signal_request_lookup_count: usize,
    queue_state_mutation_count: usize,
}

impl BridgeManagedQueueMutationCounters {
    pub const fn exact_signal_request_lookup_count(self) -> usize {
        self.exact_signal_request_lookup_count
    }

    pub const fn queue_state_mutation_count(self) -> usize {
        self.queue_state_mutation_count
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeManagedQueueMutation {
    kind: BridgeManagedQueueMutationKind,
    pressure_state: BridgeExecutionQueuePressureState,
    queue_depth: u64,
    queue_capacity: u64,
    counters: BridgeManagedQueueMutationCounters,
}

impl BridgeManagedQueueMutation {
    pub const fn kind(&self) -> BridgeManagedQueueMutationKind {
        self.kind
    }

    pub const fn pressure_state(&self) -> BridgeExecutionQueuePressureState {
        self.pressure_state
    }

    pub const fn queue_depth(&self) -> u64 {
        self.queue_depth
    }

    pub const fn queue_capacity(&self) -> u64 {
        self.queue_capacity
    }

    pub const fn counters(&self) -> BridgeManagedQueueMutationCounters {
        self.counters
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BridgeManagedQueueFailureKind {
    AtomicExecutionUnsupported,
    SignalRuntimeThreadAffinityViolation,
    SignalQueueMutationDenied,
    SignalRequestMismatch,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeManagedQueueFailure {
    kind: BridgeManagedQueueFailureKind,
    detail: String,
}

impl BridgeManagedQueueFailure {
    pub(super) fn new(kind: BridgeManagedQueueFailureKind, detail: impl Into<String>) -> Self {
        Self {
            kind,
            detail: detail.into(),
        }
    }

    pub const fn kind(&self) -> BridgeManagedQueueFailureKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }
}

impl BridgeBoundExecutionBasis {
    pub fn enqueue_managed_queue(
        &mut self,
        width: u64,
    ) -> Result<BridgeManagedQueueAdmission, BridgeManagedQueueFailure> {
        let managed = self
            .posture
            .managed_mut()
            .ok_or_else(atomic_queue_failure)?;
        let report = with_async_request_signal_runtime(self.bridge_runtime_key, |signal_runtime| {
            signal_runtime.enqueue_resource_managed_queue(&managed.queue, width)
        })
        .map_err(thread_affinity_failure)?
        .map_err(|denial| {
            BridgeManagedQueueFailure::new(
                BridgeManagedQueueFailureKind::SignalQueueMutationDenied,
                denial.detail(),
            )
        })?;
        let mutation = project_queue_mutation(self, report)?;
        let managed = self
            .posture
            .managed_mut()
            .expect("queue mutation retains managed posture");
        managed.occupancy_width = managed
            .occupancy_width
            .checked_add(width)
            .expect("Signal queue admission cannot exceed its bounded u64 capacity");
        let occupancy = BridgeManagedQueueOccupancy::new(self, width);
        Ok(BridgeManagedQueueAdmission::new(mutation, occupancy))
    }

    pub(super) fn dequeue_managed_queue_width(
        &mut self,
        width: u64,
    ) -> Result<BridgeManagedQueueMutation, BridgeManagedQueueFailure> {
        let managed = self
            .posture
            .managed_mut()
            .ok_or_else(atomic_queue_failure)?;
        let report = with_async_request_signal_runtime(self.bridge_runtime_key, |signal_runtime| {
            signal_runtime.dequeue_resource_managed_queue(&managed.queue, width)
        })
        .map_err(thread_affinity_failure)?
        .map_err(|denial| {
            BridgeManagedQueueFailure::new(
                BridgeManagedQueueFailureKind::SignalQueueMutationDenied,
                denial.detail(),
            )
        })?;
        project_queue_mutation(self, report)
    }
}

fn project_queue_mutation(
    basis: &BridgeBoundExecutionBasis,
    report: ResourceManagedQueueMutationReport,
) -> Result<BridgeManagedQueueMutation, BridgeManagedQueueFailure> {
    if report.request() != basis.request.request_handle() {
        return Err(BridgeManagedQueueFailure::new(
            BridgeManagedQueueFailureKind::SignalRequestMismatch,
            "Signal managed-queue mutation belongs to another request attempt",
        ));
    }
    let counters = report.counters();
    Ok(BridgeManagedQueueMutation {
        kind: match report.kind() {
            ResourceManagedQueueMutationKind::Enqueued => BridgeManagedQueueMutationKind::Enqueued,
            ResourceManagedQueueMutationKind::Dequeued => BridgeManagedQueueMutationKind::Dequeued,
        },
        pressure_state: project_pressure_state(report.pressure().class()),
        queue_depth: report.pressure().queue_depth(),
        queue_capacity: report.pressure().queue_capacity(),
        counters: BridgeManagedQueueMutationCounters {
            exact_signal_request_lookup_count: counters.exact_request_lookup_count(),
            queue_state_mutation_count: counters.queue_state_mutation_count(),
        },
    })
}

pub(super) const fn project_pressure_state(
    class: ResourceQueuePressureClass,
) -> BridgeExecutionQueuePressureState {
    match class {
        ResourceQueuePressureClass::Available => BridgeExecutionQueuePressureState::Available,
        ResourceQueuePressureClass::Saturated => BridgeExecutionQueuePressureState::Saturated,
    }
}

fn thread_affinity_failure(
    error: crate::source::SignalRuntimeThreadAffinityError,
) -> BridgeManagedQueueFailure {
    BridgeManagedQueueFailure::new(
        BridgeManagedQueueFailureKind::SignalRuntimeThreadAffinityViolation,
        format!(
            "bridge Signal runtime {} belongs to thread {:?}, not {:?}",
            error.runtime_key(),
            error.owner(),
            error.current()
        ),
    )
}

fn atomic_queue_failure() -> BridgeManagedQueueFailure {
    BridgeManagedQueueFailure::new(
        BridgeManagedQueueFailureKind::AtomicExecutionUnsupported,
        "atomic execution has no managed queue authority",
    )
}
