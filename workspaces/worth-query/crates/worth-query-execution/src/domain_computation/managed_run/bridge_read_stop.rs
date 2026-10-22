//! Bridge snapshot read stops translated once at managed admission.
use crate::domain_computation::primary_graph::{
    WorthQueryManagedComputationResourceDenial as Resource, WorthQueryMemoryLimitLevel as Level,
};
use crate::domain_computation::{
    WorthQueryProviderSessionControlStopKind as Control,
    WorthQueryProviderSessionDenialKind as Kind,
};
use worth_runtime_bridge::facade::BridgeExecutionDenial as Bridge;

pub(super) fn stopped(cause: Bridge) -> Result<Kind, Control> {
    let resource = |denial, policy_ancestor| Kind::ExecutionResource {
        denial,
        partition_identity: None,
        policy_ancestor,
    };
    Ok(match cause {
        Bridge::Cancelled => return Err(Control::Cancelled),
        Bridge::DeadlineElapsed => return Err(Control::TimedOut),
        Bridge::WorkerLimitExceedsParent => resource(Resource::WorkerLimit, None),
        Bridge::MemoryLimitExceedsParent => resource(Resource::PolicyMemoryLimit, None),
        Bridge::WorkLimitExceedsParent => resource(Resource::WorkLimit, None),
        Bridge::MemoryExhausted(memory) => {
            let (denial, level) = match memory {
                worth_execution::MemoryLimitDenial::CounterOverflow { level, .. } => {
                    (Resource::ChargedBytesOverflow, level)
                }
                worth_execution::MemoryLimitDenial::Capacity {
                    requested,
                    admitted,
                    level,
                } => (
                    Resource::MemoryLimit {
                        requested,
                        admitted,
                        level: match level {
                            worth_execution::MemoryLimitLevel::Policy { .. } => Level::Policy,
                            worth_execution::MemoryLimitLevel::Process => Level::Process,
                            worth_execution::MemoryLimitLevel::Declared => Level::Declared,
                        },
                    },
                    level,
                ),
            };
            resource(
                denial,
                match level {
                    worth_execution::MemoryLimitLevel::Policy { ancestor } => Some(ancestor),
                    _ => None,
                },
            )
        }
        Bridge::ChargedBytesOverflow => resource(Resource::ChargedBytesOverflow, None),
        Bridge::UnrelatedNestedLease => resource(Resource::NestedLeaseMisuse, None),
        Bridge::NoActiveExecutionScope => resource(Resource::NoActiveExecutionScope, None),
        Bridge::EquivalenceContractUnavailable => {
            resource(Resource::EquivalenceContractUnavailable, None)
        }
        Bridge::WorkCounterOverflow => resource(Resource::WorkCounterOverflow, None),
        Bridge::AccountingUnavailable => resource(Resource::AccountingUnavailable, None),
        Bridge::WorkCeiling => resource(Resource::WorkExhausted, None),
        Bridge::NestedStopped => Kind::ExecutionNestedPatternStopped {
            partition_identity: None,
        },
        Bridge::Panicked => Kind::ExecutionWorkerPanicked {
            partition_identity: None,
        },
    })
}
