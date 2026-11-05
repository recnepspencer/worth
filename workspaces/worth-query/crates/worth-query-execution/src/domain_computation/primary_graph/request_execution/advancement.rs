//! Request custody exists only while the runtime executes one advancement.

use super::{Form, QueryRequestExecution, Resource};
use worth_execution::{ExecutionRequest, ExecutionWorkCeiling, WorkCeilingDenial};
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use {
    crate::domain_computation::primary_graph::request_execution::execution_refusal::WorthQueryManagedComputationCheckpointDenial,
    crate::domain_computation::primary_graph::request_execution::execution_refusal::WorthQueryManagedComputationInterruption,
};

/// Why an advancement could not return its ordinary result.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum WorthQueryAdvancementDenial {
    Resource(Resource),
    Interrupted(WorthQueryManagedComputationInterruption),
    /// A public call tried to open an advancement from an active host call or
    /// a member compute.
    NestedOpening,
    NestedStopped,
    Panicked,
}

impl std::fmt::Display for WorthQueryAdvancementDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "advancement request refused: {self:?}")
    }
}

impl std::error::Error for WorthQueryAdvancementDenial {}

impl WorthQueryAdvancementDenial {
    pub const fn is_transient(self) -> bool {
        match self {
            Self::Resource(Resource::NativeAllocationRefused { kind, .. }) => matches!(
                kind,
                worth_execution::ExecutionPhysicalAllocationDenialKind::Allocator
            ),
            Self::Resource(Resource::MemoryLimit { level, .. }) => match level {
                crate::domain_computation::primary_graph::request_execution::execution_refusal::WorthQueryMemoryLimitLevel::Process => {
                    true
                }
                crate::domain_computation::primary_graph::request_execution::execution_refusal::WorthQueryMemoryLimitLevel::Policy
                | crate::domain_computation::primary_graph::request_execution::execution_refusal::WorthQueryMemoryLimitLevel::Declared => {
                    false
                }
            },
            Self::Resource(
                Resource::WorkExhausted
                | Resource::WorkCounterOverflow
                | Resource::AccountingUnavailable
                | Resource::RetainedBytesExhausted
                | Resource::ScratchCapacityExceeded
                | Resource::ResultCapacityExceeded
                | Resource::CapacityOverflow
                | Resource::ChargedBytesOverflow
                | Resource::WorkerLimit
                | Resource::PolicyMemoryLimit
                | Resource::WorkLimit
                | Resource::NestedLeaseMisuse
                | Resource::NestedAdvancementOpening
                | Resource::ForeignAdmissionClock
                | Resource::ForeignAdvancementPhase
                | Resource::NoActiveExecutionScope
                | Resource::EquivalenceContractUnavailable,
            )
            | Self::Interrupted(
                WorthQueryManagedComputationInterruption::Cancelled
                | WorthQueryManagedComputationInterruption::DeadlineExceeded,
            )
            | Self::NestedOpening
            | Self::NestedStopped
            | Self::Panicked => false,
        }
    }
}

mod phase_custody;
pub(crate) mod runner;
pub use phase_custody::{WorthQueryAdvancementPhase, WorthQueryForeignAdvancementPhase};
mod bootstrap_opening;
pub use bootstrap_opening::with_bootstrap_advancement_from_request;
mod bootstrap_custody;
pub use bootstrap_custody::WorthQueryBootstrapAdvancementPhase;
pub(in crate::domain_computation::primary_graph) use bootstrap_custody::WorthQueryCapturePolicyRefusal;
pub(crate) mod opening_custody;
use opening_custody::OpeningCustody;

impl QueryRequestExecution<'_> {
    pub fn execution_request(&self) -> Result<ExecutionRequest<'_, '_>, Resource> {
        match &self.form {
            Form::Leased { lease, .. } | Form::Serial { lease, .. } => lease
                .as_ref()
                .map(ExecutionRequest::leased)
                .map_err(|denial| crate::domain_computation::primary_graph::request_execution::execution_stop::lease_denial(*denial)),
        }
    }

    fn run_advancement<R>(
        &self,
        owner: Option<worth_runtime_world::facade::RuntimeWorldOwnerIdentity>,
        source_instance: Option<u64>,
        body: impl for<'scope> FnOnce(WorthQueryAdvancementPhase<'scope>) -> R,
    ) -> Result<R, WorthQueryAdvancementDenial> {
        let pending = std::cell::RefCell::new(Some(body));
        let result = std::cell::RefCell::new(None);
        runner::run_advancement(
            self,
            owner,
            source_instance,
            &mut |phase| {
                let body = pending
                    .borrow_mut()
                    .take()
                    .expect("an advancement invokes its body once");
                let value = body(phase);
                *result.borrow_mut() = Some(value);
            },
            &mut || {
                // End both borrows before arbitrary callback/result destruction.
                let pending = pending.borrow_mut().take();
                let value = result.borrow_mut().take();
                drop(pending);
                drop(value);
            },
        )?;
        Ok(result
            .into_inner()
            .expect("a completed advancement has its body result"))
    }
}

pub(in crate::domain_computation::primary_graph) fn advancement_denial(
    denial: WorkCeilingDenial,
) -> WorthQueryAdvancementDenial {
    use WorthQueryAdvancementDenial as Denial;
    match denial {
        WorkCeilingDenial::Admission(cause) => {
            Denial::Resource(crate::domain_computation::primary_graph::request_execution::execution_stop::lease_denial(cause))
        }
        WorkCeilingDenial::Stopped(stop) => match crate::domain_computation::primary_graph::request_execution::execution_stop::checkpoint_denial(stop) {
            WorthQueryManagedComputationCheckpointDenial::Resource(cause) => {
                Denial::Resource(cause)
            }
            WorthQueryManagedComputationCheckpointDenial::Interrupted(cause) => {
                Denial::Interrupted(cause)
            }
            WorthQueryManagedComputationCheckpointDenial::NestedPatternStopped => {
                Denial::NestedStopped
            }
        },
        WorkCeilingDenial::Panicked => Denial::Panicked,
    }
}

pub(crate) mod installed_opening;
pub(crate) mod source;
pub use source::WorthQueryAdvancementSource;

/// Installation runs serially under the host's memory and aggregate work bounds.
/// Custody is per thread; another thread's public call owns its own request.
/// Installation's serial placement reports zero memory as `PolicyMemoryLimit`;
/// an installed leased opener instead reports a policy `MemoryLimit`.
pub fn with_bootstrap_advancement<R>(
    policy: worth_foundational::ExecutionRequestPolicy,
    body: impl for<'scope> FnOnce(WorthQueryBootstrapAdvancementPhase<'scope>) -> R,
) -> Result<R, WorthQueryAdvancementDenial> {
    with_owned_bootstrap_advancement(policy, |phase| {
        body(WorthQueryBootstrapAdvancementPhase::new(
            phase
                .active
                .execution
                .execution_request()
                .expect("owned bootstrap was admitted"),
        ))
    })
}

fn with_owned_bootstrap_advancement<R>(
    policy: worth_foundational::ExecutionRequestPolicy,
    body: impl for<'scope> FnOnce(WorthQueryAdvancementPhase<'scope>) -> R,
) -> Result<R, WorthQueryAdvancementDenial> {
    let _custody = OpeningCustody::enter()?;
    let cancellation = worth_execution::CancellationToken::new();
    let execution = QueryRequestExecution {
        cancellation: cancellation.clone(),
        deadline: None,
        declared_work: policy.work_ceiling(),
        certified_seed: super::test_placement::certifying(),
        form: super::serial_placement::form(policy, cancellation, None),
    };
    execution.run_advancement(None, None, body)
}

pub(in crate::domain_computation::primary_graph) mod observation;

pub use observation::{
    advancement_observations_on_this_thread_for_test, advancement_requests_on_this_thread_for_test,
    WorthQueryAdvancementEventForTest, WorthQueryAdvancementEventKindForTest,
    WorthQueryAdvancementIdentityForTest, WorthQueryAdvancementObservationForTest,
};

pub fn with_test_advancement<R>(
    body: impl for<'scope> FnOnce(test_host_policy::TestHostRequest<'scope>) -> R,
) -> R {
    with_owned_bootstrap_advancement(fixture_request_policy(), |phase| {
        body(test_host_policy::TestHostRequest::new(phase.active))
    })
    .expect("the declared fixture policy admits its host call")
}

/// The declared fixture request policy: one serial worker with canonical bitwise results and
/// a finite work allowance. The fixture Product World installs it too.
pub(crate) fn fixture_request_policy() -> worth_foundational::ExecutionRequestPolicy {
    worth_foundational::ExecutionRequestPolicy::new(
        worth_foundational::ExecutionPosture::Serial,
        worth_foundational::DeterminismContract::CanonicalBitwise,
        worth_foundational::ExecutionBudget::new(
            std::num::NonZeroUsize::MIN,
            None,
            Some(8_000_000),
        ),
    )
}

/// Positive fixture authoring has its own finite allowance, outside measured host calls.
pub fn with_test_bootstrap_advancement<R>(
    work: u64,
    body: impl for<'scope> FnOnce(test_host_policy::TestHostRequest<'scope>) -> R,
) -> R {
    let policy = worth_foundational::ExecutionRequestPolicy::new(
        worth_foundational::ExecutionPosture::Serial,
        worth_foundational::DeterminismContract::CanonicalBitwise,
        worth_foundational::ExecutionBudget::new(std::num::NonZeroUsize::MIN, None, Some(work)),
    );
    with_owned_bootstrap_advancement(policy, |phase| {
        body(test_host_policy::TestHostRequest::new(phase.active))
    })
    .expect("the declared fixture bootstrap allowance admits authoring")
}

pub(crate) mod test_host_policy;
pub(crate) use test_host_policy::TestHostRequest;

mod court_doors_for_test;
pub(crate) mod worker_custody;
