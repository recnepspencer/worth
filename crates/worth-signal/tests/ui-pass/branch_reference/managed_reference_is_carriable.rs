use worth_signal::facade::branch::ManagedSignalBranchReference;

fn carry(reference: &ManagedSignalBranchReference) -> ManagedSignalBranchReference {
    reference.clone()
}

fn assert_send_sync<T: Send + Sync>() {}

fn main() {
    assert_send_sync::<ManagedSignalBranchReference>();
    let _valid_carriage = carry;
}
