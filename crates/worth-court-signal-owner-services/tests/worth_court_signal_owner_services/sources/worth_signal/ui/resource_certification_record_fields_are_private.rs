use worth_signal::facade::{
    ResourceBoundaryPerformanceEnvelope, ResourceCertificationFamily, ResourceCertificationRecord,
};

fn forged_performance() -> ResourceBoundaryPerformanceEnvelope {
    loop {}
}

fn main() {
    let _forged = ResourceCertificationRecord {
        family: ResourceCertificationFamily::AsyncResourceLifecycleParity,
        evidence_digest: String::new(),
        performance: forged_performance(),
        passed: true,
    };
}
