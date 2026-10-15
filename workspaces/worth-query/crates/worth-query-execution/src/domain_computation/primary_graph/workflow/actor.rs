//! Native admission is the only author of canonical workflow actors.
mod dependencies;
pub(in crate::domain_computation) mod record;
mod revision_transport;
pub(in crate::domain_computation) use dependencies::WorkflowActorDependencies;

fn storage_denial(
    cause: worth_execution::ExecutionAllocationDenial,
    subject: &str,
) -> crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial {
    crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StoreDenial::Allocation(cause)
        .into_attempt_denial(subject)
}

/// Native JSON transport admission, with no actor, permission or policy choice.
#[derive(Debug)]
pub(in crate::domain_computation) struct ActorCodecFunding {
    _payload: std::sync::Arc<crate::domain_computation::primary_graph::application_checkpoint::facts::CodecPayloadReservation>,
}
pub(in crate::domain_computation) fn fund_actor_codec(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    bytes: usize,
) -> Result<
    std::sync::Arc<ActorCodecFunding>,
    crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial,
> {
    use crate::domain_computation::primary_graph::{
        application_attempt::retained_decision_facts::StorageControl, application_checkpoint::facts,
    };
    let payload = facts::CodecPayloadReservation::admit(
        StorageControl::new(execution.allocation_policy(), None),
        bytes,
    )
    .map_err(dependencies::decode_denial)?;
    Ok(std::sync::Arc::new(ActorCodecFunding { _payload: payload }))
}
