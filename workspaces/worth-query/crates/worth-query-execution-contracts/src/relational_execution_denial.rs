//! Pure conversion from Relational execution denial causes to Query denial kinds.

use crate::computation_denial::{
    WorthQueryManagedComputationResourceDenial as Resource, WorthQueryMemoryLimitLevel as Level,
};
use crate::provider_session_denial_kind::{
    WorthQueryProviderSessionControlStopKind as Control, WorthQueryProviderSessionDenialKind as Kind,
};
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;

/// The only conversion from Relational execution causes to Query denial kinds.
pub fn relational_execution_kind(
    cause: Cause,
    partition_identity: Option<u64>,
) -> Result<Kind, Control> {
    let resource = |denial, policy_ancestor| Kind::ExecutionResource {
        denial,
        partition_identity,
        policy_ancestor,
    };
    let kind = match cause {
        Cause::ProjectionContract(denial) => Kind::ProjectionContract {
            denial,
            partition_identity,
        },
        Cause::HistoricalVisibility(denial) => Kind::HistoricalVisibility {
            denial,
            partition_identity,
        },
        Cause::StorageUnavailable(failure) => Kind::StorageUnavailable(failure),
        Cause::ContentInconsistency => Kind::ContentInconsistency,
        Cause::Content(cause) => Kind::Content(cause),
        Cause::WorkerLimitExceedsParent => resource(Resource::WorkerLimit, None),
        Cause::MemoryLimitExceedsParent => resource(Resource::PolicyMemoryLimit, None),
        Cause::WorkLimitExceedsParent => resource(Resource::WorkLimit, None),
        Cause::PolicyMemoryExhausted {
            requested,
            admitted,
            ancestor,
        } => resource(
            Resource::MemoryLimit {
                requested,
                admitted,
                level: Level::Policy,
            },
            Some(ancestor),
        ),
        Cause::ProcessMemoryExhausted {
            requested,
            admitted,
        } => resource(
            Resource::MemoryLimit {
                requested,
                admitted,
                level: Level::Process,
            },
            None,
        ),
        Cause::DeclaredMemoryExhausted {
            requested,
            admitted,
        } => resource(
            Resource::MemoryLimit {
                requested,
                admitted,
                level: Level::Declared,
            },
            None,
        ),
        Cause::ChargedBytesOverflow => resource(Resource::ChargedBytesOverflow, None),
        Cause::UnrelatedNestedLease => resource(Resource::NestedLeaseMisuse, None),
        Cause::NoActiveExecutionScope => resource(Resource::NoActiveExecutionScope, None),
        Cause::EquivalenceContractUnavailable => {
            resource(Resource::EquivalenceContractUnavailable, None)
        }
        Cause::Cancelled => return Err(Control::Cancelled),
        Cause::DeadlineElapsed => return Err(Control::TimedOut),
        Cause::WorkCounterOverflow => resource(Resource::WorkCounterOverflow, None),
        Cause::AccountingUnavailable => resource(Resource::AccountingUnavailable, None),
        Cause::WorkExhausted => resource(Resource::WorkExhausted, None),
        Cause::NestedStopped => Kind::ExecutionNestedPatternStopped { partition_identity },
        Cause::ResultCapacityExceeded => resource(Resource::ResultCapacityExceeded, None),
        Cause::ScratchCapacityExceeded => resource(Resource::ScratchCapacityExceeded, None),
        Cause::AllocationFailed => Kind::AllocationDenied,
        Cause::WorkerFailed => Kind::ExecutionWorkerPanicked { partition_identity },
        Cause::UncheckedCustomKernel => Kind::ExecutionUncheckedCustomKernel { partition_identity },
        Cause::ExpectedIdentitiesNotCanonical => {
            Kind::ExecutionIdentitiesNotCanonical { partition_identity }
        }
        Cause::MemoryOverflow => resource(Resource::CapacityOverflow, None),
    };
    Ok(kind)
}
