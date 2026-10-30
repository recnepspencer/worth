use worth_signal::facade::branch::{SignalBranchBasisPort, SignalOwnerServicePorts};

fn invalid_public_port_construction() -> SignalBranchBasisPort<(), (), ()> {
    SignalBranchBasisPort {
        owner: (),
        diagnostic_owner_runtime_instance_id: 0,
    }
}

fn valid_public_bundle_type() {
    let _ = std::mem::size_of::<SignalOwnerServicePorts<(), (), (), (), ()>>();
}

fn main() {}
