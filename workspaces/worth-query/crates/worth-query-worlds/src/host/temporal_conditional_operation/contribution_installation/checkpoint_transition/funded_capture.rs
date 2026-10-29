//! A caller lease funds an Execution-mode capture as the call's one budget.
pub use super::*;
pub use application_installation::WorthQueryCheckpointCaptureDenial as CaptureDenial;
pub use original_request::{request, transition_original};
pub use worth_execution::ExecutionRequest;
pub use worth_query_host::facade::runtime::{
    CancellationToken, ExecutionAllocationDenialKind as AllocationKind, LeaseDenial,
};

pub fn funded_target() -> original_request::Target {
    let (source, predecessor) = source();
    let setup = primary_graph::query_test_execution_authority_for_test()
        .request_lease(request(CancellationToken::new()))
        .unwrap();
    transition_original(
        ExecutionRequest::leased(&setup),
        source,
        predecessor,
        |writer, _| admission::author_record(writer, "funded-capture", 151),
    )
    .unwrap()
}
