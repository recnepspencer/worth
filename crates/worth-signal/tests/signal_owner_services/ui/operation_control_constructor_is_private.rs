use worth_signal::facade::branch::SignalOwnerOperationControl;

fn invalid_public_constructor() -> SignalOwnerOperationControl {
    SignalOwnerOperationControl::new()
}

fn valid_control_carriage(control: SignalOwnerOperationControl) -> SignalOwnerOperationControl {
    control
}

fn main() {}
