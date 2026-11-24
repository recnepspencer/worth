//! A request opened under its caller's own lease.
//!
//! A run inside an active worker charges only that worker's lineage, so a
//! caller who funds work with `Execution(lease)` inside an advancement opens
//! the advancement under that lease: its worker is then a descendant of the
//! caller's lease, and the caller's charge reaches the lease it names.

use super::{Form, QueryRequestExecution, WorthQueryManagedComputationInterruption};
use worth_execution::ExecutionResourceLease;

impl<'a> QueryRequestExecution<'a> {
    /// The request's execution on a controlled child of the caller's `lease`,
    /// under the lease's policy and cancellation lineage, its deadline and
    /// the caller's own `cancellation` and `deadline`.
    pub(super) fn under_lease(
        lease: &ExecutionResourceLease<'a>,
        cancellation: worth_execution::CancellationToken,
        deadline: Option<worth_execution::MonotonicDeadline>,
    ) -> Self {
        let policy = *lease.policy();
        Self {
            cancellation: cancellation.clone(),
            deadline: deadline.clone(),
            declared_work: policy.work_ceiling(),
            certified_seed: super::test_placement::certifying(),
            form: Form::Leased {
                lease: Ok(lease.controlled_child(cancellation, deadline)),
                policy,
            },
        }
    }

    pub(super) fn open_serial_request(
        placement: worth_runtime_world::facade::RuntimeWorldExecutionPlacement<'a>,
        caller: worth_execution::ExecutionRequest<'a, 'a>,
    ) -> Self {
        let placement = super::test_placement::placed(placement);
        let placement_policy = match placement {
            worth_runtime_world::facade::RuntimeWorldExecutionPlacement::Serial(policy)
            | worth_runtime_world::facade::RuntimeWorldExecutionPlacement::Leased {
                policy, ..
            } => policy,
        };
        let effective_work = min_option(caller.work_ceiling(), placement_policy.work_ceiling());
        let effective_memory = min_option(
            caller.memory_limit(),
            placement_policy.charged_memory_bytes(),
        );
        let placement_tighter =
            effective_work != caller.work_ceiling() || effective_memory != caller.memory_limit();
        match placement {
            worth_runtime_world::facade::RuntimeWorldExecutionPlacement::Serial(_)
                if !placement_tighter =>
            {
                Self::borrowed_serial(caller)
            }
            worth_runtime_world::facade::RuntimeWorldExecutionPlacement::Serial(policy) => {
                let narrowed = narrowed_policy(policy, effective_memory, effective_work);
                Self::open_placed(
                    worth_runtime_world::facade::RuntimeWorldExecutionPlacement::Serial(narrowed),
                    worth_execution::CancellationToken::new(),
                    caller.deadline(),
                )
            }
            worth_runtime_world::facade::RuntimeWorldExecutionPlacement::Leased {
                authority,
                policy,
            } => {
                let narrowed = narrowed_policy(policy, effective_memory, effective_work);
                Self::open_placed(
                    worth_runtime_world::facade::RuntimeWorldExecutionPlacement::Leased {
                        authority,
                        policy: narrowed,
                    },
                    worth_execution::CancellationToken::new(),
                    caller.deadline(),
                )
            }
        }
    }

    pub(super) fn borrowed_serial(request: worth_execution::ExecutionRequest<'a, 'a>) -> Self {
        let budget = worth_foundational::ExecutionBudget::new(
            std::num::NonZeroUsize::MIN,
            request.memory_limit(),
            request.work_ceiling(),
        );
        let policy = if request.has_accounting() {
            worth_foundational::ExecutionRequestPolicy::new(
                worth_foundational::ExecutionPosture::Serial,
                worth_foundational::DeterminismContract::CanonicalBitwise,
                budget,
            )
        } else {
            worth_foundational::ExecutionRequestPolicy::for_workers(
                worth_foundational::ExecutionPosture::Serial,
                worth_foundational::DeterminismContract::CanonicalBitwise,
                std::num::NonZeroUsize::MIN,
            )
        };
        Self {
            cancellation: worth_execution::CancellationToken::new(),
            deadline: request.deadline(),
            declared_work: request.work_ceiling(),
            certified_seed: super::test_placement::certifying(),
            form: Form::BorrowedSerial { request, policy },
        }
    }

    /// The request's cancellation or elapsed deadline, if either happened,
    /// including a cancellation anywhere in its lease's lineage.
    pub fn interruption(&self) -> Option<WorthQueryManagedComputationInterruption> {
        if let Form::BorrowedSerial { request, .. } = &self.form {
            return match request.check_live() {
                Ok(()) => None,
                Err(worth_execution::MapKernelStop::Cancelled) => {
                    Some(WorthQueryManagedComputationInterruption::Cancelled)
                }
                Err(worth_execution::MapKernelStop::DeadlineElapsed) => {
                    Some(WorthQueryManagedComputationInterruption::DeadlineExceeded)
                }
                Err(_) => None,
            };
        }
        let lease = match &self.form {
            Form::Leased { lease, .. } | Form::Serial { lease, .. } => lease.as_ref().ok(),
            Form::BorrowedSerial { .. } => unreachable!(),
        };
        if self.cancellation.is_cancelled()
            || lease.is_some_and(ExecutionResourceLease::is_cancelled)
        {
            Some(WorthQueryManagedComputationInterruption::Cancelled)
        } else if self
            .deadline
            .as_ref()
            .is_some_and(|deadline| deadline.is_elapsed())
            || lease.is_some_and(ExecutionResourceLease::deadline_elapsed)
        {
            Some(WorthQueryManagedComputationInterruption::DeadlineExceeded)
        } else {
            None
        }
    }
}

fn min_option(a: Option<u64>, b: Option<u64>) -> Option<u64> {
    match (a, b) {
        (Some(x), Some(y)) => Some(x.min(y)),
        (Some(x), None) | (None, Some(x)) => Some(x),
        (None, None) => None,
    }
}

fn narrowed_policy(
    policy: worth_foundational::ExecutionRequestPolicy,
    memory: Option<u64>,
    work: Option<u64>,
) -> worth_foundational::ExecutionRequestPolicy {
    if policy.accounting().is_some() || memory.is_some() || work.is_some() {
        worth_foundational::ExecutionRequestPolicy::new(
            policy.posture(),
            policy.determinism(),
            worth_foundational::ExecutionBudget::new(policy.max_workers(), memory, work),
        )
    } else {
        policy
    }
}
