use worth_store::physical_runtime::PhysicalDurabilityRecoveryHandoff;

fn duplicate(handoff: PhysicalDurabilityRecoveryHandoff) {
    let _ = handoff.clone();
}

fn main() {}
