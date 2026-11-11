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

pub fn original_capture_work_stop() -> Stop {
    Stop::Operation(
        Terminal::ProductUnpublished,
        Cause::ProductUnpublished(graph::ProductUnpublishedCause::OriginalObservationStopped(
            worth_relational::facade::transactions::RelationalExecutionDenialCause::WorkExhausted,
        )),
    )
}

pub fn assert_work_stop_preserves_its_owning_phase(stop: &Stop) {
    assert!(
        *stop == provider_work_stop() || *stop == original_capture_work_stop(),
        "unexpected work-stop phase: {stop:?}"
    );
}
