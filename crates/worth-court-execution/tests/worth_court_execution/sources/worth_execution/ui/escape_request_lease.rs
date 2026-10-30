use worth_execution::ExecutionRequest;
fn inspect<'a>(request: ExecutionRequest<'a, 'a>) {
    let _ = request.in_scope(|lease| lease).unwrap();
}
fn main() {}
