//! Checked kernel custody for direct storage-owner tests.
pub(crate) fn run<R>(
    request: worth_execution::ExecutionRequest<'_, '_>,
    body: impl FnOnce(&mut worth_execution::MapKernelContext<'_, '_>) -> R,
) -> R {
    let mut body = Some(body);
    let result = crate::logic::planner::run_signal_preparation_request(request, |work, _, _| {
        Ok(std::panic::catch_unwind(std::panic::AssertUnwindSafe(
            || body.take().expect("one checked test operation")(work),
        )))
    })
    .expect("the caller's declared memory policy admits the storage test")
    .0;
    match result {
        Ok(value) => value,
        Err(payload) => std::panic::resume_unwind(payload),
    }
}
