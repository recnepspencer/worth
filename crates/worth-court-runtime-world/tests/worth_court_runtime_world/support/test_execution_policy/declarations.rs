// Host policy for Runtime World's serial fixture entries.
pub(crate) use {
    worth_execution::CancellationSource, worth_execution::SerialMemoryBudget,
    worth_execution::SerialRequest,
};

pub(crate) fn serial_request() -> SerialRequest {
    worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(None),
        worth_execution::CancellationSource::new().token(),
        None,
    )
}
