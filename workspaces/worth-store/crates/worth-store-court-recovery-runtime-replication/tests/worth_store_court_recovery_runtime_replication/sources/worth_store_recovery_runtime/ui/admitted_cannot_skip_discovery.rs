use worth_store_recovery_runtime::AdmittedPhysicalRecovery;

fn skip_discovery(admitted: AdmittedPhysicalRecovery) {
    let _ = admitted.select();
}

fn main() {}
