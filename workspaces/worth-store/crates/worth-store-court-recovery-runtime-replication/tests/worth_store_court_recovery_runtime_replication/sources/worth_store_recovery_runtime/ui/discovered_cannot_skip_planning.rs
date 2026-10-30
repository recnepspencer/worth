use worth_store_recovery_runtime::DiscoveredPhysicalRecovery;

fn skip_selection(discovered: DiscoveredPhysicalRecovery) {
    let _ = discovered.plan();
}

fn main() {}
