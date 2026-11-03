//! One classifier for caller-local native stops at source read and producer commit.
use super::WorthQueryOutputDemandDenialKind as Demand;
use crate::domain_computation::primary_graph::{
    WorthQueryAdvancementDenial as Request, WorthQueryApplicationOneShotDenialKind as Query,
};
use crate::domain_computation::{
    WorthQueryProviderSessionControlStopKind as Control,
    WorthQueryProviderSessionDenialKind as Native,
};

pub(super) fn kind(cause: Native) -> Demand {
    stop(Ok(cause)).unwrap_or(Demand::SourceQueryExecution(Query::ExecutionRequest(cause)))
}

pub(super) fn stop(cause: Result<Native, Control>) -> Option<Demand> {
    // An unpartitioned executor stop is not proof that the installed producer
    // is unavailable to every caller. It belongs to this request, as do its
    // root budget and interruption. Partition/ancestor attribution remains
    // native evidence; those failures are not collapsed into a root request.
    match cause {
        Err(Control::Cancelled) => Some(Demand::Cancelled),
        Err(Control::TimedOut) => Some(Demand::TimedOut),
        Ok(native) => match native {
            Native::ExecutionResource {
                denial,
                partition_identity: None,
                policy_ancestor: None,
            } => Some(Demand::ExecutionRequest(Request::Resource(denial))),
            Native::ExecutionNestedPatternStopped {
                partition_identity: None,
            } => Some(Demand::ExecutionRequest(Request::NestedStopped)),
            Native::ExecutionWorkerPanicked {
                partition_identity: None,
            } => Some(Demand::ExecutionRequest(Request::Panicked)),
            _ => None,
        },
    }
}
