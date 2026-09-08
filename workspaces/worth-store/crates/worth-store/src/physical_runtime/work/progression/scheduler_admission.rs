use worth_store_io_scheduler::QueueExecutionReadyPlan;

use super::{DispatchedPhysicalWork, ReadyPhysicalWork};
use crate::physical_runtime::work::{
    PhysicalWorkConcurrencyScope, PhysicalWorkConsumerHandle, PhysicalWorkIntent,
    PhysicalWorkPreEffectDenial, PhysicalWorkTerminalStage,
};

pub struct ResourceAdmittedPhysicalWork {
    ready: ReadyPhysicalWork,
    queue_plan: QueueExecutionReadyPlan,
    scheduler_capacity: Option<
        worth_store_io_scheduler::foreground_reservation::PhysicalInstanceForegroundCapacityLease,
    >,
}

impl ResourceAdmittedPhysicalWork {
    pub(in crate::physical_runtime) fn new(
        ready: ReadyPhysicalWork,
        queue_plan: QueueExecutionReadyPlan,
        scheduler_capacity:
            Option<worth_store_io_scheduler::foreground_reservation::PhysicalInstanceForegroundCapacityLease>,
    ) -> Self {
        ready.admitted.mark_stage(PhysicalWorkTerminalStage::Queued);
        Self {
            ready,
            queue_plan,
            scheduler_capacity,
        }
    }

    pub const fn intent(&self) -> &PhysicalWorkIntent {
        self.ready.intent()
    }

    pub const fn queue_plan(&self) -> &QueueExecutionReadyPlan {
        &self.queue_plan
    }

    pub fn consumer_handle(&self) -> PhysicalWorkConsumerHandle {
        PhysicalWorkConsumerHandle::new(
            self.intent().identity(),
            self.ready.signal.signal_request,
            self.ready.authority().binding(),
        )
    }

    pub fn concurrency_scope(&self) -> PhysicalWorkConcurrencyScope {
        PhysicalWorkConcurrencyScope::derive(self.intent())
    }

    pub(in crate::physical_runtime) fn is_cancelled(&self) -> bool {
        self.ready.admitted.is_cancelled()
    }

    pub(in crate::physical_runtime) fn into_execution_parts(
        self,
        payload_digest: Option<[u8; 32]>,
    ) -> Result<(DispatchedPhysicalWork, QueueExecutionReadyPlan), PhysicalWorkPreEffectDenial>
    {
        let Self {
            ready,
            queue_plan,
            scheduler_capacity,
        } = self;
        let ReadyPhysicalWork { admitted, signal } = ready;
        let effect_activity = admitted
            .begin_dispatch()
            .ok_or(PhysicalWorkPreEffectDenial::ConsumerCancelled)?;
        Ok((
            DispatchedPhysicalWork {
                inspection_source: None,
                admitted,
                signal,
                effect_activity: Some(effect_activity),
                scheduler_capacity,
                scheduler_binding: queue_plan
                    .backend_completion_binding()
                    .backend_execution_binding(),
                payload_digest,
            },
            queue_plan,
        ))
    }
}
