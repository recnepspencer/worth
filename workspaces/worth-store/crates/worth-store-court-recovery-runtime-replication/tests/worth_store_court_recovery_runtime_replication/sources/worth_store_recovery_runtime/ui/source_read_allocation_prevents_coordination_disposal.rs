use worth_store::physical_runtime::PhysicalRecoveryCoordination;

fn dispose_during_source_read(mut coordination: PhysicalRecoveryCoordination) {
    let mut reads = coordination.begin_source_read_allocation().unwrap();
    coordination.shutdown_is_quiescent();
    reads.reserve_total(1).unwrap();
}

fn main() {}
