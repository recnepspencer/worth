use worth_signal::facade::branch::SignalOwnerOperationControlForTest;

fn invalid_public_constructor() -> SignalOwnerOperationControlForTest {
    SignalOwnerOperationControlForTest::new()
}

fn valid_control_carriage(control: SignalOwnerOperationControlForTest) -> SignalOwnerOperationControlForTest {
    control
}

fn main() {}
