use worth_signal::branch::owner_services::SignalOwner;

fn invalid_private_cell_route() {
    let _ = std::mem::size_of::<SignalOwner<(), (), ()>>();
}

fn valid_public_route() {
    let _ = std::mem::size_of::<
        worth_signal::facade::branch::SignalOwnerServicePorts<(), (), (), (), ()>,
    >();
}

fn main() {}
