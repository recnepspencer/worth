use super::direct_graph_execution::{
    WorthQueryActiveDirectGraphExecution, WorthQueryDirectGraphStepOutcome,
};
use super::interruption_classification::consumer_terminal_kind;
use super::WorthQueryManagedRunTerminalKind;
use crate::domain_computation::provider_session::graph_provider::bounded_step::WorthQueryGraphProviderStepCompletion;
use crate::domain_computation::{WorthQueryGraphProviderStepReport, WorthQueryGraphReadMaterial};

pub(super) struct WorthQueryPendingDirectGraphQueueState {
    occupancy: worth_runtime_bridge::facade::BridgeManagedQueueOccupancy,
    depth: u64,
    capacity: u64,
}

impl WorthQueryPendingDirectGraphQueueState {
    pub(super) const fn new(
        occupancy: worth_runtime_bridge::facade::BridgeManagedQueueOccupancy,
        depth: u64,
        capacity: u64,
    ) -> Self {
        Self {
            occupancy,
            depth,
            capacity,
        }
    }
}

#[must_use = "pending graph chunk must be acknowledged or explicitly abandoned"]
pub struct WorthQueryPendingDirectGraphChunk {
    active: WorthQueryActiveDirectGraphExecution,
    report: WorthQueryGraphProviderStepReport,
    material: WorthQueryGraphReadMaterial,
    queue: WorthQueryPendingDirectGraphQueueState,
    retained_bytes: usize,
}

impl WorthQueryPendingDirectGraphChunk {
    pub(super) fn new(
        active: WorthQueryActiveDirectGraphExecution,
        report: WorthQueryGraphProviderStepReport,
        material: WorthQueryGraphReadMaterial,
        queue: WorthQueryPendingDirectGraphQueueState,
        retained_bytes: usize,
    ) -> Self {
        Self {
            active,
            report,
            material,
            queue,
            retained_bytes,
        }
    }

    pub fn chunk(&self) -> &WorthQueryGraphReadMaterial {
        &self.material
    }

    pub const fn queue_depth(&self) -> u64 {
        self.queue.depth
    }

    pub const fn queue_capacity(&self) -> u64 {
        self.queue.capacity
    }

    pub fn request_cancellation(
        &self,
        reason: worth_runtime_bridge::facade::BridgeManagedExecutionCancellationReason,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeManagedExecutionCancellation,
        worth_runtime_bridge::facade::BridgeManagedExecutionInterruptionFailure,
    > {
        self.active.request_cancellation(reason)
    }

    pub fn acknowledge(self) -> WorthQueryDirectGraphStepOutcome {
        let Self {
            mut active,
            report,
            material,
            queue,
            retained_bytes,
        } = self;
        let before = match active.observe_safe_point() {
            Ok(observation) => observation,
            Err(_) => {
                release_or_retain_queue(&mut active, queue.occupancy);
                drop(material);
                let _ = active.release_pending_chunk(retained_bytes);
                return active.abandoned_terminal(WorthQueryManagedRunTerminalKind::Failed);
            }
        };
        let terminal = consumer_terminal_kind(&before);
        let released = release_or_retain_queue(&mut active, queue.occupancy);
        if !released {
            drop(material);
            let _ = active.release_pending_chunk(retained_bytes);
            return active.abandoned_terminal(WorthQueryManagedRunTerminalKind::Failed);
        }
        if let Some(kind) = terminal {
            drop(material);
            let _ = active.release_pending_chunk(retained_bytes);
            return active.interrupted_terminal(kind);
        }
        if !active.retain_pending_chunk(material) {
            return active.abandoned_terminal(WorthQueryManagedRunTerminalKind::Failed);
        }
        match report.completion() {
            WorthQueryGraphProviderStepCompletion::Continue => active.continue_after_safe_point(),
            WorthQueryGraphProviderStepCompletion::Complete => active.finish_completion(&report),
            WorthQueryGraphProviderStepCompletion::Failed => {
                active.abandoned_terminal(WorthQueryManagedRunTerminalKind::Failed)
            }
        }
    }

    pub fn abandon(self) -> WorthQueryDirectGraphStepOutcome {
        let Self {
            mut active,
            report: _,
            material,
            queue,
            retained_bytes,
        } = self;
        release_or_retain_queue(&mut active, queue.occupancy);
        drop(material);
        let _ = active.release_pending_chunk(retained_bytes);
        active.abandoned_terminal(WorthQueryManagedRunTerminalKind::Failed)
    }
}

fn release_or_retain_queue(
    active: &mut WorthQueryActiveDirectGraphExecution,
    occupancy: worth_runtime_bridge::facade::BridgeManagedQueueOccupancy,
) -> bool {
    let running = &mut active.running;
    running
        .affinity
        .provider_work_mut()
        .release_or_retain_queue_occupancy(&mut running.bridge_basis, occupancy)
}
