use worth_runtime_bridge::facade::RuntimeWorldCorrespondenceInspectionCounters;

fn main() {
    let counters = RuntimeWorldCorrespondenceInspectionCounters::default();
    let _ = counters.authoritative_registration_inspections();
}
