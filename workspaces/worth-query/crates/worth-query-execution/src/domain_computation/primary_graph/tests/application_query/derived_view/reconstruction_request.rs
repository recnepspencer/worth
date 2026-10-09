//! Finite standalone reconstruction policy; no independent process authority.

pub(super) fn serial_request(
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> worth_execution::SerialRequest {
    worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(16 * 1024 * 1024),
        request.cancellation().execution_token(),
        Some(request.deadline()),
    )
}
