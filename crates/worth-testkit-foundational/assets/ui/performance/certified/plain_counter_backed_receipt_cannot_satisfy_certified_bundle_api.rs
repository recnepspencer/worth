use worth_foundational::{
    FoundationalAuthoritativePerformanceClaim, FoundationalCertifiedPerformanceBundle,
    FoundationalCounterBackedPerformanceReceipt,
};

fn require_certified_bundle(
    _bundle: &FoundationalCertifiedPerformanceBundle<
        FoundationalCounterBackedPerformanceReceipt<FoundationalAuthoritativePerformanceClaim>,
    >,
) {
}

fn main() {
    let receipt = impossible_receipt();
    require_certified_bundle(&receipt);
}

fn impossible_receipt(
) -> FoundationalCounterBackedPerformanceReceipt<FoundationalAuthoritativePerformanceClaim> {
    loop {
        std::hint::spin_loop();
    }
}
