use worth_signal::facade::{
    branch::ManagedSignalBranchReference,
    history::RuntimeBranch as SignalBranchHandle,
};

fn valid_managed_reference(reference: ManagedSignalBranchReference) -> ManagedSignalBranchReference {
    reference
}

fn raw_handle_cannot_construct_managed_reference(
    handle: SignalBranchHandle,
) -> ManagedSignalBranchReference {
    handle.into()
}

fn main() {}
