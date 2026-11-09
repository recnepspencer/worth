use worth_server::WorthServer;

fn main() {
    let _server = WorthServer::new(supplied());
}

// Type-check the forbidden action without a divergent argument lint.
fn supplied<T>() -> T {
    loop {}
}
