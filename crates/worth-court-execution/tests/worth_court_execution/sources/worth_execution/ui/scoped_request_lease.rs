use worth_execution::ExecutionRequest;
fn inspect<'a>(request: ExecutionRequest<'a, 'a>) {
    let _ = request.in_scope(|lease| lease.is_some()).unwrap();
}
pub(crate) fn main() {
    let request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(None),
        worth_execution::CancellationToken::new(),
        None,
    );
    inspect(ExecutionRequest::serial(&request));
}
