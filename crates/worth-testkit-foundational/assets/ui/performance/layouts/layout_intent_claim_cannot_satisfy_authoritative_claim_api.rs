use worth_foundational::performance;
use worth_foundational::{
    FoundationalAuthoritativePerformanceClaim, FoundationalPerformanceAccessPatternPosture,
    FoundationalPerformanceAllocationPosture, FoundationalPerformanceLayoutIntent,
};

fn require_authoritative_claim(_claim: &FoundationalAuthoritativePerformanceClaim) {}

fn main() {
    let layout_claim = performance().define_layout_intent(
        FoundationalPerformanceLayoutIntent::AoS,
        FoundationalPerformanceAccessPatternPosture::PointLookup,
        FoundationalPerformanceAllocationPosture::ActionLocal,
    );

    require_authoritative_claim(&layout_claim);
}
