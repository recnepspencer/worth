//! Request custody exists only while the runtime executes one advancement.

use super::super::execution_denial;
use super::{Form, QueryRequestExecution, Resource};
use crate::domain_computation::primary_graph::{
    WorthQueryManagedComputationCheckpointDenial, WorthQueryManagedComputationInterruption,
    WorthQueryPrimaryGraphApplicationRuntime,
};
use worth_execution::{ExecutionRequest, ExecutionWorkCeiling, WorkCeilingDenial};
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_installation::facade::ApplicationSchema;

/// Why an advancement could not return its ordinary result.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum WorthQueryAdvancementDenial {
    Resource(Resource),
    Interrupted(WorthQueryManagedComputationInterruption),
    /// A public call tried to open a second advancement on the same thread.
    NestedOpening,
    /// The phase belongs to a different installed runtime.
    ForeignPhase,
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
            Self::Resource(Resource::MemoryLimit { level, .. }) => match level {
                crate::domain_computation::primary_graph::WorthQueryMemoryLimitLevel::Process => {
                    true
                }
                crate::domain_computation::primary_graph::WorthQueryMemoryLimitLevel::Policy
                | crate::domain_computation::primary_graph::WorthQueryMemoryLimitLevel::Declared => {
                    false
                }
            },
            Self::Resource(
                Resource::WorkExhausted
                | Resource::WorkCounterOverflow
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
                | Resource::ForeignAdvancementPhase
                | Resource::NoActiveExecutionScope
                | Resource::EquivalenceContractUnavailable,
            )
            | Self::Interrupted(
                WorthQueryManagedComputationInterruption::Cancelled
                | WorthQueryManagedComputationInterruption::DeadlineExceeded,
            )
            | Self::NestedOpening
            | Self::ForeignPhase
            | Self::NestedStopped
            | Self::Panicked => false,
        }
    }
}

mod phase_custody;
use phase_custody::ActiveAdvancement;
pub use phase_custody::{WorthQueryAdvancementPhase, WorthQueryForeignAdvancementPhase};
mod bootstrap_custody;
pub use bootstrap_custody::WorthQueryBootstrapAdvancementPhase;
mod opening_custody;
use opening_custody::OpeningCustody;

impl QueryRequestExecution<'_> {
    fn execution_request(&self) -> Result<ExecutionRequest<'_, '_>, Resource> {
        match &self.form {
            Form::Leased { lease, .. } => lease
                .as_ref()
                .map(ExecutionRequest::leased)
                .map_err(|denial| execution_denial::lease_denial(*denial)),
            Form::Serial(request) => Ok(ExecutionRequest::serial(request)),
        }
    }

    fn run_advancement<R>(
        &self,
        owner: Option<worth_runtime_world::facade::RuntimeWorldOwnerIdentity>,
        source_instance: Option<u64>,
        body: impl for<'scope> FnOnce(WorthQueryAdvancementPhase<'scope>) -> R,
    ) -> Result<R, WorthQueryAdvancementDenial> {
        let result = self.run_admitted_advancement(owner, source_instance, body);
        #[cfg(any(test, feature = "test-query-execution-observer"))]
        observation::record_result(&result);
        result
    }

    fn run_admitted_advancement<R>(
        &self,
        owner: Option<worth_runtime_world::facade::RuntimeWorldOwnerIdentity>,
        source_instance: Option<u64>,
        body: impl for<'scope> FnOnce(WorthQueryAdvancementPhase<'scope>) -> R,
    ) -> Result<R, WorthQueryAdvancementDenial> {
        let request = self
            .execution_request()
            .map_err(WorthQueryAdvancementDenial::Resource)?;
        if self.declared_work == 0 {
            return Err(WorthQueryAdvancementDenial::Resource(
                Resource::WorkExhausted,
            ));
        }
        if matches!(&self.form, Form::Serial(_)) && request.memory_limit() == 0 {
            return Err(WorthQueryAdvancementDenial::Resource(
                Resource::PolicyMemoryLimit,
            ));
        }
        let mut body_unwind = None;
        let outcome = request.run(ExecutionWorkCeiling::new(self.declared_work), |_| {
            let active = ActiveAdvancement {
                execution: self,
                owner,
                source_instance,
            };
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                body(WorthQueryAdvancementPhase { active: &active })
            })) {
                Ok(result) => Some(result),
                Err(payload) => {
                    body_unwind = Some(payload);
                    None
                }
            }
        });
        // The execution owner catches unwinds to close its accounting scope.
        // Preserve the host body's original unwind and its existing posture.
        if let Some(payload) = body_unwind {
            std::panic::resume_unwind(payload);
        }
        outcome
            .map(|(result, report)| {
                #[cfg(any(test, feature = "test-query-execution-observer"))]
                observation::record_report(report);
                #[cfg(not(any(test, feature = "test-query-execution-observer")))]
                let _ = report;
                result.expect("a completed advancement has its body result")
            })
            .map_err(advancement_denial)
    }
}

pub(in crate::domain_computation::primary_graph) fn advancement_denial(
    denial: WorkCeilingDenial,
) -> WorthQueryAdvancementDenial {
    use WorthQueryAdvancementDenial as Denial;
    match denial {
        WorkCeilingDenial::Admission(cause) => {
            Denial::Resource(execution_denial::lease_denial(cause))
        }
        WorkCeilingDenial::Stopped(stop) => match execution_denial::checkpoint_denial(stop) {
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

mod installed_opening;

/// Installation runs serially under the host's memory and aggregate work bounds.
/// Custody is per thread; another thread's public call owns its own request.
/// Installation's serial placement reports zero memory as `PolicyMemoryLimit`;
/// an installed leased opener instead reports a policy `MemoryLimit`.
pub fn with_bootstrap_advancement<R>(
    policy: worth_foundational::ExecutionRequestPolicy,
    body: impl for<'scope> FnOnce(WorthQueryBootstrapAdvancementPhase<'scope>) -> R,
) -> Result<R, WorthQueryAdvancementDenial> {
    let _custody = OpeningCustody::enter()?;
    let cancellation = worth_execution::CancellationToken::new();
    let execution = QueryRequestExecution {
        cancellation: cancellation.clone(),
        deadline: None,
        declared_work: policy.budget().work_ceiling(),
        form: Form::Serial(worth_execution::SerialRequest::from_memory(
            worth_execution::SerialMemoryBudget::from_policy(&policy),
            cancellation,
            None,
        )),
    };
    execution.run_advancement(None, None, |phase| {
        body(WorthQueryBootstrapAdvancementPhase {
            active: phase.active,
        })
    })
}

/// A held product remembers policy facts even when its weak owner has expired.
/// Publication still enters a bounded call and lets World report owner absence.
pub(in crate::domain_computation::primary_graph) fn with_serial_host_advancement<R>(
    owner: worth_runtime_world::facade::RuntimeWorldOwnerIdentity,
    policy: worth_foundational::ExecutionRequestPolicy,
    request: &WorthQueryRequestScope,
    body: impl for<'scope> FnOnce(WorthQueryAdvancementPhase<'scope>) -> R,
) -> Result<R, WorthQueryAdvancementDenial> {
    let _custody = OpeningCustody::enter()?;
    let cancellation = request.cancellation().execution_token();
    let deadline = Some(request.deadline());
    let execution = QueryRequestExecution {
        cancellation: cancellation.clone(),
        deadline,
        declared_work: policy.budget().work_ceiling(),
        form: Form::Serial(worth_execution::SerialRequest::from_memory(
            worth_execution::SerialMemoryBudget::from_policy(&policy),
            cancellation,
            deadline,
        )),
    };
    execution.run_advancement(Some(owner), None, body)
}

/// Opens the request for a host call whose staged product retained only owner facts.
pub(in crate::domain_computation::primary_graph) fn with_world_advancement<R>(
    world: &worth_runtime_world::facade::RuntimeWorldOwner<(), (), (), (), ()>,
    request: &WorthQueryRequestScope,
    body: impl for<'scope> FnOnce(WorthQueryAdvancementPhase<'scope>) -> R,
) -> Result<R, WorthQueryAdvancementDenial> {
    let _custody = OpeningCustody::enter()?;
    QueryRequestExecution::open(world.execution_placement(), request).run_advancement(
        Some(world.owner_identity()),
        None,
        body,
    )
}

impl WorthQueryAdvancementDenial {
    pub(in crate::domain_computation) fn provider_denial_cause(
        self,
    ) -> Result<
        crate::domain_computation::WorthQueryProviderSessionDenialKind,
        crate::domain_computation::WorthQueryProviderSessionControlStopKind,
    > {
        use crate::domain_computation::{
            WorthQueryProviderSessionControlStopKind as Control,
            WorthQueryProviderSessionDenialKind as Kind,
        };
        match self {
            Self::Resource(denial) => Ok(Kind::ExecutionResource {
                denial,
                partition_identity: None,
                policy_ancestor: None,
            }),
            Self::Interrupted(WorthQueryManagedComputationInterruption::Cancelled) => {
                Err(Control::Cancelled)
            }
            Self::Interrupted(WorthQueryManagedComputationInterruption::DeadlineExceeded) => {
                Err(Control::TimedOut)
            }
            Self::NestedOpening => Ok(Kind::ExecutionResource {
                denial: Resource::NestedAdvancementOpening,
                partition_identity: None,
                policy_ancestor: None,
            }),
            Self::ForeignPhase => Ok(Kind::ExecutionResource {
                denial: Resource::ForeignAdvancementPhase,
                partition_identity: None,
                policy_ancestor: None,
            }),
            Self::NestedStopped => Ok(Kind::ExecutionNestedPatternStopped {
                partition_identity: None,
            }),
            Self::Panicked => Ok(Kind::ExecutionWorkerPanicked {
                partition_identity: None,
            }),
        }
    }
}

#[cfg(any(test, feature = "test-query-execution-observer"))]
mod observation;
#[cfg(feature = "test-query-execution-observer")]
pub use observation::{
    advancement_requests_on_this_thread_for_test, caller_pass_reports_on_this_thread_for_test,
};

#[cfg(test)]
pub(crate) fn with_test_advancement<R>(
    body: impl for<'scope> FnOnce(WorthQueryAdvancementPhase<'scope>) -> R,
) -> R {
    let policy =
        crate::domain_computation::execution_runtime::product_world::test_product_world_resources()
            .execution_policy();
    with_bootstrap_advancement(policy, |bootstrap| {
        body(WorthQueryAdvancementPhase {
            active: bootstrap.active,
        })
    })
    .expect("the declared fixture policy admits its host call")
}

#[cfg(all(test, feature = "test-query-execution-observer"))]
mod standalone_test_custody;

#[cfg(test)]
mod tests;

#[cfg(feature = "test-query-execution-observer")]
pub(in crate::domain_computation::primary_graph) use observation::record_caller_pass;
