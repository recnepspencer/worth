use worth_foundational::{
    FoundationalPerformanceEvidenceStrength, FoundationalPerformanceExecutionTemperature,
};

fn requires_strength(_: FoundationalPerformanceEvidenceStrength) {}

fn main() {
    requires_strength(FoundationalPerformanceExecutionTemperature::HotPath);
}
