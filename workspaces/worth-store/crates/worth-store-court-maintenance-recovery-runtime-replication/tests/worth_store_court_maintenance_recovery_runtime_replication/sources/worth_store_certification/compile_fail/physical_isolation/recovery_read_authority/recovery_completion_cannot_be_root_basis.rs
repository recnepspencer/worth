use worth_store_physical_isolation::PhysicalIsolationRootEpochBasis;
use worth_store_recovery_runtime::RecoveryCompletion;

fn promote(report: RecoveryCompletion) -> PhysicalIsolationRootEpochBasis {
    report.into()
}

fn main() {}
