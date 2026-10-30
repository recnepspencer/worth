use worth_store::physical_runtime::{
    AbortedRuntime, AdmittedPhysicalRuntime, ClosedRuntime, ObservationHandle,
};

fn invoke_physical_work(runtime: &AdmittedPhysicalRuntime) {
    runtime.append_physical_record(b"not installed");
    runtime.open_physical_media();
    runtime.media_observation();
    runtime.lookup_owner("media");
}

fn promote_observation(observation: &ObservationHandle) {
    observation.append_physical_record(b"not installed");
    observation.publish_physical_changes();
    observation.recover_physical_store();
    observation.close();
    observation.raw_physical_owner();
}

fn reuse_after_close(runtime: AdmittedPhysicalRuntime) {
    let _closed = runtime.close();
    let _identity = runtime.runtime_identity();
}

fn observe_closed(closed: ClosedRuntime) {
    let _observer = closed.observe();
}

fn observe_aborted(aborted: AbortedRuntime) {
    let _observer = aborted.observe();
}

fn main() {}
