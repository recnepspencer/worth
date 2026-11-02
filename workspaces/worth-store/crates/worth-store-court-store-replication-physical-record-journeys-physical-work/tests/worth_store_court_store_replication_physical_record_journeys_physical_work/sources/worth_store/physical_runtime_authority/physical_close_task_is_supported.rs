use worth_store::physical_runtime::{
    PhysicalStoreCloseFailure, PhysicalStoreCloseStartRefused, PhysicalStoreCloseTask,
    PhysicalStoreCloseTaskPoll, ServingPhysicalRuntime,
};

fn observe_and_poll(task: PhysicalStoreCloseTask) {
    let _ = task.runtime_identity();
    let _ = task.readiness();
    let _ = task.observation();
    match task.poll() {
        PhysicalStoreCloseTaskPoll::Pending(task) => drop(task),
        PhysicalStoreCloseTaskPoll::Completed(actual) => {
            let _ = actual.into_shutdown();
        }
        PhysicalStoreCloseTaskPoll::Failed(failure) => inspect_failure(failure),
    }
}

fn inspect_failure(failure: PhysicalStoreCloseFailure) {
    let _ = failure.runtime_identity();
    let _ = failure.cause();
    let _ = failure.actual_outcome();
    let _ = failure.observation();
    let _ = failure.into_parts();
}

fn return_unstarted(refused: PhysicalStoreCloseStartRefused) {
    let _ = refused.error();
    let (_serving, _error) = refused.into_parts();
}

fn start(serving: ServingPhysicalRuntime) {
    match PhysicalStoreCloseTask::start(serving) {
        Ok(task) => observe_and_poll(task),
        Err(refused) => return_unstarted(refused),
    }
}

pub fn main() {
    // Compile-only API bodies; no Store construction or runtime-close claim.
    std::hint::black_box(start as fn(ServingPhysicalRuntime));
}
