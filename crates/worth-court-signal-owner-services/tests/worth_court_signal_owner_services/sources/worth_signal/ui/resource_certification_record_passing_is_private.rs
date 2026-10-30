use worth_signal::facade::{
    ResourceBoundaryPerformanceEnvelope, ResourceCertificationFamily, ResourceCertificationRecord,
};

fn forged_performance() -> ResourceBoundaryPerformanceEnvelope {
    loop {}
}

fn main() {
    let _forged = ResourceCertificationRecord::passing(
        ResourceCertificationFamily::AsyncResourceLifecycleParity,
        "fake-digest",
        forged_performance(),
    );
}
