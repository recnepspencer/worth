use worth_signal::facade::branch::SignalOwnerServicePorts;

fn bypass(ports: SignalOwnerServicePorts<(), (), (), (), ()>) {
    let _ = ports.conditional_execution();
}

fn main() {}
