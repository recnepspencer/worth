use worth_server::WorthServerCompatibilityRequest;

fn main() {
    let _request = WorthServerCompatibilityRequest {
        admission: supplied(),
        request_contract: supplied(),
    };
}

// Type-check the forbidden action without a divergent argument lint.
fn supplied<T>() -> T {
    loop {}
}
