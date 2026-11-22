use std::sync::Arc;

use crate::domain_computation::provider_session::graph_provider::bounded_step::{
    provider_anchor::{WorthQueryGraphProviderAnchor, WorthQueryGraphProviderStartInvocation},
    WorthQueryGraphProviderExecutionStart, WorthQueryGraphProviderMemoryArena,
    WorthQueryOwnedGraphProviderExecution,
};
use crate::domain_computation::{
    WorthQueryGraphProviderCall, WorthQueryGraphProviderExecution,
    WorthQueryProviderExecutionReleaseEvidence,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum WorthQueryManagedProviderStartFailureKind {
    Rejected,
    Panicked,
    ContractDenied,
    MemoryLeaked,
    ProviderExecutionReleaseRecoveryRequired,
}

pub(super) struct WorthQueryManagedProviderStarted {
    pub(super) execution: Box<dyn WorthQueryGraphProviderExecution>,
    pub(super) memory: WorthQueryGraphProviderMemoryArena,
}

pub(super) struct WorthQueryManagedProviderStartFailure {
    pub(super) kind: WorthQueryManagedProviderStartFailureKind,
    pub(super) detail: Arc<str>,
    pub(super) memory: WorthQueryGraphProviderMemoryArena,
    pub(super) provider_execution_release: Option<WorthQueryProviderExecutionReleaseEvidence>,
}

pub(super) fn start_managed_provider(
    execution: worth_execution::ExecutionRequest<'_, '_>,

    anchor: &WorthQueryGraphProviderAnchor,
    call: &WorthQueryGraphProviderCall,
    retained_bytes_ceiling: u64,
) -> Result<WorthQueryManagedProviderStarted, WorthQueryManagedProviderStartFailure> {
    let memory = WorthQueryGraphProviderMemoryArena::new(retained_bytes_ceiling);
    let mut start = WorthQueryGraphProviderExecutionStart::new(execution, memory.clone());
    let invocation = anchor.begin(call, &mut start);
    let unreturned_execution_release = start.release_unreturned_execution();
    let contract = start.finish();
    match invocation {
        WorthQueryGraphProviderStartInvocation::Returned(Ok(execution)) => {
            if let Err(denial) = contract {
                return Err(release_denied_execution(memory, execution, denial.detail()));
            }
            Ok(WorthQueryManagedProviderStarted { execution, memory })
        }
        WorthQueryGraphProviderStartInvocation::Returned(Err(failure)) => {
            let (kind, detail) = match contract {
                Ok(()) => (
                    WorthQueryManagedProviderStartFailureKind::Rejected,
                    Arc::from(failure.detail()),
                ),
                Err(denial) => (
                    WorthQueryManagedProviderStartFailureKind::ContractDenied,
                    Arc::from(denial.detail()),
                ),
            };
            Err(failed_without_returned_execution(
                kind,
                detail,
                memory,
                unreturned_execution_release,
            ))
        }
        WorthQueryGraphProviderStartInvocation::Panicked => {
            let detail = contract.err().map_or_else(
                || Arc::from("provider execution construction panicked"),
                |denial| {
                    Arc::from(format!(
                        "provider execution construction panicked after start contract denial: {}",
                        denial.detail()
                    ))
                },
            );
            Err(failed_without_returned_execution(
                WorthQueryManagedProviderStartFailureKind::Panicked,
                detail,
                memory,
                unreturned_execution_release,
            ))
        }
    }
}

fn release_denied_execution(
    memory: WorthQueryGraphProviderMemoryArena,
    execution: Box<dyn WorthQueryGraphProviderExecution>,
    detail: &str,
) -> WorthQueryManagedProviderStartFailure {
    let release = WorthQueryOwnedGraphProviderExecution::new(execution).release();
    let snapshot = memory.snapshot();
    let kind = if release.recovery_required() {
        WorthQueryManagedProviderStartFailureKind::ProviderExecutionReleaseRecoveryRequired
    } else if snapshot.retained_bytes() != 0 {
        WorthQueryManagedProviderStartFailureKind::MemoryLeaked
    } else {
        WorthQueryManagedProviderStartFailureKind::ContractDenied
    };
    WorthQueryManagedProviderStartFailure {
        kind,
        detail: Arc::from(detail),
        memory,
        provider_execution_release: Some(release),
    }
}

fn failed_without_returned_execution(
    primary: WorthQueryManagedProviderStartFailureKind,
    detail: Arc<str>,
    memory: WorthQueryGraphProviderMemoryArena,
    provider_execution_release: Option<WorthQueryProviderExecutionReleaseEvidence>,
) -> WorthQueryManagedProviderStartFailure {
    let snapshot = memory.snapshot();
    let kind = if provider_execution_release
        .as_ref()
        .is_some_and(WorthQueryProviderExecutionReleaseEvidence::recovery_required)
    {
        WorthQueryManagedProviderStartFailureKind::ProviderExecutionReleaseRecoveryRequired
    } else if snapshot.retained_bytes() != 0 {
        WorthQueryManagedProviderStartFailureKind::MemoryLeaked
    } else {
        primary
    };
    WorthQueryManagedProviderStartFailure {
        kind,
        detail,
        memory,
        provider_execution_release,
    }
}
