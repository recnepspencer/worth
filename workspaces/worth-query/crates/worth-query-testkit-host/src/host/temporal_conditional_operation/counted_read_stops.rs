pub use super::{primary_graph as graph, world::CourtroomWorld};
pub use graph::{
    WorthQueryConditionalExecutionCause as Cause,
    WorthQueryConditionalExecutionTerminal as Terminal,
};
#[derive(Debug, Eq, PartialEq)]
pub enum Stop {
    Operation(Terminal, Cause),
    Clock(graph::WorthQueryConditionalClockObservationFailureKind),
}

pub fn provider_work_stop() -> Stop {
    use worth_query_host::facade::application_contribution::WorthQueryManagedComputationResourceDenial as Resource;
    Stop::Operation(
        Terminal::Retryable,
        Cause::ApplicationCommitDenied(
            graph::WorthQueryApplicationCommitDenialKind::ExecutionResource {
                denial: Resource::WorkExhausted,
                partition_identity: None,
                policy_ancestor: None,
            },
        ),
    )
}
