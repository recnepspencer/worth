use worth_store::physical_runtime::{
    ClosedPhysicalRecoveryCleanup, CompletedPhysicalRecoveryFreshReopen,
};

fn forge(reopen: CompletedPhysicalRecoveryFreshReopen) -> ClosedPhysicalRecoveryCleanup {
    ClosedPhysicalRecoveryCleanup {
        reopen,
        descriptive_plan_identity: [0x11; 32],
        authority_plan_identity: None,
        live_media_handle_delta: 1,
    }
}

fn main() {
    let _ = forge;
}
