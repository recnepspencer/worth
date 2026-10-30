use worth_signal::facade::branch::SignalOwnerUnavailable;

fn invalid_forged_unavailable() -> SignalOwnerUnavailable {
    SignalOwnerUnavailable
}

fn valid_error_carriage(error: SignalOwnerUnavailable) -> SignalOwnerUnavailable {
    error
}

fn main() {}
