use worth_store::physical_runtime::PhysicalRecoveryConstructionPort;

fn forge() -> PhysicalRecoveryConstructionPort {
    PhysicalRecoveryConstructionPort {}
}

fn main() {
    let _ = forge();
}
