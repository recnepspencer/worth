use worth_server::WorthServerPreparedQueryHandoffIntent;

fn main() {
    let _ = WorthServerPreparedQueryHandoffIntent {
        kind: supplied(),
        operation_name: "users.profile".to_string(),
    };
}

// Type-check the forbidden action without a divergent argument lint.
fn supplied<T>() -> T {
    loop {}
}
