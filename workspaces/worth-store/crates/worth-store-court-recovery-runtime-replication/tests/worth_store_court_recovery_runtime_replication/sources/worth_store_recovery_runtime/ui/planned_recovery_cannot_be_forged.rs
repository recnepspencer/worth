use worth_store_recovery_runtime::PlannedPhysicalRecovery;

fn forge() -> PlannedPhysicalRecovery {
    PlannedPhysicalRecovery {
        authority: (),
        coordination: (),
        selection: (),
        discovery_counters: Default::default(),
        freshness: (),
        fates: (),
        redo: (),
    }
}

fn main() {}
