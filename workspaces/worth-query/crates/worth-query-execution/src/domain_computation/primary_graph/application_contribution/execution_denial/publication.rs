//! Publication request failures use the same SDK-to-Query resource vocabulary.
use super::{checkpoint_denial, lease_denial, memory_denial};
use crate::domain_computation::primary_graph::{
    WorthQueryManagedComputationCheckpointDenial as Checkpoint,
    WorthQueryManagedComputationInterruption as Interruption,
};
use crate::domain_computation::{
    WorthQueryProviderSessionControlStopKind as Control,
    WorthQueryProviderSessionDenialKind as Kind,
};
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
