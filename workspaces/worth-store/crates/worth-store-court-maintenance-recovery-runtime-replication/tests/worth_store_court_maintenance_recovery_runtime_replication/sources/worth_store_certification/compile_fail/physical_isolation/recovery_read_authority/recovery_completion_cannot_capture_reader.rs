use worth_store::physical_runtime::ServingPhysicalRuntime;
use worth_store_recovery_runtime::RecoveryCompletion;

fn capture(report: &RecoveryCompletion) {
    let _ = ServingPhysicalRuntime::records(report);
}

fn main() {}
