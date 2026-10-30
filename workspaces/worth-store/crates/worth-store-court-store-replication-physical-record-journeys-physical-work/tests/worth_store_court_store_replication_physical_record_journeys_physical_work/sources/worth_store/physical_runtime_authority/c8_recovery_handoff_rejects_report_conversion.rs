use worth_store::physical_runtime::{
    PhysicalDurabilityPerformanceSummary, PhysicalDurabilityRecoveryHandoff,
};

fn mint(report: PhysicalDurabilityPerformanceSummary) -> PhysicalDurabilityRecoveryHandoff {
    report.into()
}

fn main() {}
