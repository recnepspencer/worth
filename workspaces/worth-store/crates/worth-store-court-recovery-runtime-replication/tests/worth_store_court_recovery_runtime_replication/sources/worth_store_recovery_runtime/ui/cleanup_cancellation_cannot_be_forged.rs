use worth_store_recovery_runtime::PhysicalRecoveryCleanupCancellation;

fn main() {
    let _forged = PhysicalRecoveryCleanupCancellation {
        plan: [0; 32],
        settled_actions: 0,
    };
}
