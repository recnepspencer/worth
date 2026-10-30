use worth_store_aspect_native::StorePerformanceReceiptEvidence;

struct LocalPerformanceClaim;

fn require_store_performance_evidence(
    _evidence: Option<StorePerformanceReceiptEvidence<LocalPerformanceClaim>>,
) {
}

fn main() {
    require_store_performance_evidence(None);
}
