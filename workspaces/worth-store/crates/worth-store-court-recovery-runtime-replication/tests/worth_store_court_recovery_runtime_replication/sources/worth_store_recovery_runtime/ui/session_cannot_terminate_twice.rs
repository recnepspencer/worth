use worth_store_recovery_runtime::AdmittedPhysicalRecovery;

fn terminate_twice(admitted: AdmittedPhysicalRecovery) {
    let _first = admitted.cancel_before_discovery();
    let _second = admitted.cancel_before_discovery();
}

fn main() {}
