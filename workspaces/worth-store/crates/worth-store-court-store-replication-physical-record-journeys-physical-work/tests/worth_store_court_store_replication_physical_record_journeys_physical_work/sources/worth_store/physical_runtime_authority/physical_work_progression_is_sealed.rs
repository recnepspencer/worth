use worth_store::physical_runtime::{
    AdmittedPhysicalWork, PhysicalWorkIntent, ReadyPhysicalWork, ResourceAdmittedPhysicalWork,
};
use worth_store_io_scheduler::QueueExecutionReadyPlan;

fn forge_admission(intent: PhysicalWorkIntent) {
    let _admitted = AdmittedPhysicalWork { intent };
}

fn splice_scheduler_plan(ready: ReadyPhysicalWork, queue_plan: QueueExecutionReadyPlan) {
    let _resource_admitted = ResourceAdmittedPhysicalWork { ready, queue_plan };
}

fn main() {}
