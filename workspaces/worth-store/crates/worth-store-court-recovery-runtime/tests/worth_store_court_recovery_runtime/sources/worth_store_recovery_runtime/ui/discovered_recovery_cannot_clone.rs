use worth_store_recovery_runtime::DiscoveredPhysicalRecovery;

fn duplicate(discovered: DiscoveredPhysicalRecovery) {
    let first = discovered;
    let second = discovered;
    drop((first, second));
}

fn main() {}
