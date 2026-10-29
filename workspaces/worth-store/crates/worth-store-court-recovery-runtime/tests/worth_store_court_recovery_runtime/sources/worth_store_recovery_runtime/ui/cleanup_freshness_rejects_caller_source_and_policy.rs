use worth_store::physical_runtime::StoreRecoveryCleanupPlan;

fn fake<T>() -> T {
    panic!("compile-only specimen")
}

fn main() {
    let _forged = StoreRecoveryCleanupPlan {
        identity: [1; 32],
        store: fake(),
        media_generation: fake(),
        session: [1; 16],
        policy_identity: [2; 32],
        candidates: fake(),
    };
}
