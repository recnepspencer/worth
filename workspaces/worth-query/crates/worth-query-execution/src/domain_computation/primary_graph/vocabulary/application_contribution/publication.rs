//! Moved whole from `domain_computation::primary_graph::application_contribution::execution_denial::publication`: the items below and every impl they own.
use crate::domain_computation::vocabulary::provider_session::control_stopped::WorthQueryProviderSessionControlStopKind as Control;
use crate::domain_computation::vocabulary::provider_session::protocol_denial::WorthQueryProviderSessionDenialKind as Kind;
use crate::domain_computation::request_execution::execution_refusal::WorthQueryManagedComputationCheckpointDenial as Checkpoint;
use crate::domain_computation::request_execution::execution_refusal::WorthQueryManagedComputationInterruption as Interruption;
use crate::domain_computation::request_execution::execution_stop::checkpoint_denial;
use crate::domain_computation::request_execution::lease_denial;
use crate::domain_computation::request_execution::memory_denial;

pub(in crate::domain_computation) fn publication_execution_kind(
    cause: worth_execution::WorkCeilingDenial,
) -> Result<Kind, Control> {
    use worth_execution::{LeaseDenial, MemoryLimitLevel, WorkCeilingDenial as Denial};
    let resource = |denial, policy_ancestor| Kind::ExecutionResource {
        denial,
        partition_identity: None,
        policy_ancestor,
    };
    Ok(match cause {
        Denial::Admission(LeaseDenial::MemoryExhausted(denial)) => resource(
            memory_denial(denial),
            match denial {
                worth_execution::MemoryLimitDenial::Capacity { level, .. }
                | worth_execution::MemoryLimitDenial::CounterOverflow { level, .. } => {
                    match level {
                        MemoryLimitLevel::Policy { ancestor } => Some(ancestor),
                        MemoryLimitLevel::Declared | MemoryLimitLevel::Process => None,
                    }
                }
            },
        ),
        Denial::Admission(denial) => resource(lease_denial(denial), None),
        Denial::Stopped(stop) => match checkpoint_denial(stop) {
            Checkpoint::Resource(denial) => resource(denial, None),
            Checkpoint::Interrupted(Interruption::Cancelled) => return Err(Control::Cancelled),
            Checkpoint::Interrupted(Interruption::DeadlineExceeded) => {
                return Err(Control::TimedOut)
            }
            Checkpoint::NestedPatternStopped => Kind::ExecutionNestedPatternStopped {
                partition_identity: None,
            },
        },
        Denial::Panicked => Kind::ExecutionWorkerPanicked {
            partition_identity: None,
        },
    })
}
