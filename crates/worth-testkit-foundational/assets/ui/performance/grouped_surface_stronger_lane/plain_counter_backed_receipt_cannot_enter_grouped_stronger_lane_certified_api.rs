use worth_foundational::{
    FoundationalAuthoritativePerformanceClaim,
    performance_api::stronger_lane::certified,
    FoundationalCounterBackedPerformanceReceipt,
};

fn require_certified_bundle(
    _bundle: &worth_foundational::FoundationalCertifiedPerformanceBundle<
        FoundationalCounterBackedPerformanceReceipt<FoundationalAuthoritativePerformanceClaim>,
    >,
) {
}

fn main() {
    let receipt = impossible_receipt();
    require_certified_bundle(&receipt);
    let _ = certified::foundational_performance_certified_attachment_authority();
}

fn impossible_receipt(
) -> FoundationalCounterBackedPerformanceReceipt<FoundationalAuthoritativePerformanceClaim> {
    loop {
        std::hint::spin_loop();
    }
}
