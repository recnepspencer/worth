//! A request opened under its caller's own lease or serial backing, narrowed
//! to any tighter installed World or test placement ceilings without dropping
//! the caller's live cancellation token or existing memory holds.

use super::{Form, QueryRequestExecution, WorthQueryManagedComputationInterruption};
use worth_execution::{CancellationToken, ExecutionResourceLease, LeaseRequest};
use worth_runtime_world::facade::RuntimeWorldExecutionPlacement;

impl<'a> QueryRequestExecution<'a> {
    /// The request's execution on a controlled child of the caller's `lease`,
    /// under the lease's policy and cancellation lineage, its deadline and
    /// the caller's own `cancellation` and `deadline`.
    pub(super) fn under_lease(
        lease: &ExecutionResourceLease<'a>,
        cancellation: CancellationToken,
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
                caller_serial: None,
                caller_serial_hold: None,
            },
        }
    }

    /// Narrows a caller-owned `lease` to any tighter installed placement
    /// ceiling (workers, memory, or work) while preserving the caller's exact
    /// lease lineage, live memory holds, deadline, and cancellation token.
    pub(super) fn under_lease_within_placement(
        lease: &ExecutionResourceLease<'a>,
        placement: RuntimeWorldExecutionPlacement<'a>,
        deadline: Option<worth_execution::MonotonicDeadline>,
    ) -> Self {
        let placement = super::test_placement::placed(placement);
        let placement_policy = match placement {
            RuntimeWorldExecutionPlacement::Serial(policy)
            | RuntimeWorldExecutionPlacement::Leased { policy, .. } => policy,
        };
        let caller_policy = *lease.policy();
        let effective_workers = caller_policy
            .max_workers()
            .min(placement_policy.max_workers());
        let effective_memory = min_option(
            caller_policy.charged_memory_bytes(),
            placement_policy.charged_memory_bytes(),
        );
        let effective_work = min_option(
            caller_policy.work_ceiling(),
            placement_policy.work_ceiling(),
        );
        let narrowed = narrowed_policy_with_workers(
            caller_policy,
            effective_workers,
            effective_memory,
            effective_work,
        );
        let controlled = lease.controlled_child(CancellationToken::new(), deadline.clone());
        let final_lease = if narrowed == caller_policy {
            Ok(controlled)
        } else {
            controlled.child(LeaseRequest {
                policy: narrowed,
                deadline: deadline.clone(),
                cancellation: CancellationToken::new(),
            })
        };
        Self {
            cancellation: CancellationToken::new(),
            deadline,
            declared_work: effective_work,
            certified_seed: super::test_placement::certifying(),
            form: Form::Leased {
                lease: final_lease,
                policy: narrowed,
                caller_serial: None,
                caller_serial_hold: None,
            },
        }
    }

    /// Preserves a lease-free `ExecutionRequest::serial` caller's backing,
    /// memory holds, deadline, and cancellation token while enforcing any
    /// tighter installed placement ceilings.
    pub fn open_serial_request(
        placement: RuntimeWorldExecutionPlacement<'a>,
        caller: worth_execution::ExecutionRequest<'a, 'a>,
    ) -> Self {
        let placement = super::test_placement::placed(placement);
        let placement_policy = match placement {
            RuntimeWorldExecutionPlacement::Serial(policy)
            | RuntimeWorldExecutionPlacement::Leased { policy, .. } => policy,
        };
        let effective_work = min_option(caller.work_ceiling(), placement_policy.work_ceiling());
        let effective_memory = min_option(
            caller.memory_limit(),
            placement_policy.charged_memory_bytes(),
        );
        let deadline = caller.deadline();
        match placement {
            RuntimeWorldExecutionPlacement::Serial(policy) => {
                let narrowed_pol = narrowed_policy_with_workers(
                    policy,
                    std::num::NonZeroUsize::MIN,
                    effective_memory,
                    effective_work,
                );
                Self {
                    cancellation: CancellationToken::new(),
                    deadline: deadline.clone(),
                    declared_work: effective_work,
                    certified_seed: super::test_placement::certifying(),
                    form: Form::Serial {
                        lease: worth_execution::ExecutionAuthority::serial().request_lease(
                            LeaseRequest {
                                policy: narrowed_pol,
                                deadline,
                                cancellation: CancellationToken::new(),
                            },
                        ),
                        policy: narrowed_pol,
                        caller_serial: Some(caller),
                        caller_serial_hold: Some(super::serial_hold::unbound_hold()),
                    },
                }
            }
            RuntimeWorldExecutionPlacement::Leased { authority, policy } => {
                let narrowed = narrowed_policy(policy, effective_memory, effective_work);
                Self {
                    cancellation: CancellationToken::new(),
                    deadline: deadline.clone(),
                    declared_work: effective_work,
                    certified_seed: super::test_placement::certifying(),
                    form: Form::Leased {
                        lease: authority.request_lease(LeaseRequest {
                            policy: narrowed,
                            deadline,
                            cancellation: CancellationToken::new(),
                        }),
                        policy: narrowed,
                        caller_serial: Some(caller),
                        caller_serial_hold: Some(super::serial_hold::unbound_hold()),
                    },
                }
            }
        }
    }

    /// Narrows this execution's inner lease to `available_bytes` when a serial
    /// caller's live remaining room (after pre-existing holds) is tighter than
    /// the static policy ceiling already installed on the lease.
    pub(super) fn narrowed_to_live_serial_room(&self, available_bytes: u64) -> Option<Self> {
        let (lease, policy, caller_serial, caller_serial_hold, is_serial) = match &self.form {
            Form::Leased {
                lease,
                policy,
                caller_serial,
                caller_serial_hold,
            } => (lease, *policy, *caller_serial, caller_serial_hold, false),
            Form::Serial {
                lease,
                policy,
                caller_serial,
                caller_serial_hold,
            } => (lease, *policy, *caller_serial, caller_serial_hold, true),
        };
        let current_limit = policy.charged_memory_bytes();
        if current_limit.is_some_and(|limit| available_bytes >= limit) {
            return None;
        }
        let narrowed = narrowed_policy_with_workers(
            policy,
            policy.max_workers(),
            Some(available_bytes),
            policy.work_ceiling(),
        );
        let child_lease = lease.as_ref().map_err(|denial| *denial).and_then(|parent| {
            parent.child(LeaseRequest {
                policy: narrowed,
                deadline: None,
                cancellation: CancellationToken::new(),
            })
        });
        let form = if is_serial {
            Form::Serial {
                lease: child_lease,
                policy: narrowed,
                caller_serial,
                caller_serial_hold: caller_serial_hold.clone(),
            }
        } else {
            Form::Leased {
                lease: child_lease,
                policy: narrowed,
                caller_serial,
                caller_serial_hold: caller_serial_hold.clone(),
            }
        };
        Some(Self {
            cancellation: self.cancellation.clone(),
            deadline: self.deadline.clone(),
            declared_work: self.declared_work,
            certified_seed: self.certified_seed,
            form,
        })
    }

    /// The request's cancellation or elapsed deadline, if either happened,
    /// including a cancellation anywhere in its lease or serial caller lineage.
    pub fn interruption(&self) -> Option<WorthQueryManagedComputationInterruption> {
        let (lease, serial_caller) = match &self.form {
            Form::Leased {
                lease,
                caller_serial,
                ..
            }
            | Form::Serial {
                lease,
                caller_serial,
                ..
            } => (lease.as_ref().ok(), *caller_serial),
        };
        if let Some(caller) = serial_caller {
            match caller.check_live() {
                Ok(()) => {}
                Err(worth_execution::MapKernelStop::Cancelled) => {
                    return Some(WorthQueryManagedComputationInterruption::Cancelled);
                }
                Err(worth_execution::MapKernelStop::DeadlineElapsed) => {
                    return Some(WorthQueryManagedComputationInterruption::DeadlineExceeded);
                }
                Err(_) => {}
            }
        }
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
    narrowed_policy_with_workers(policy, policy.max_workers(), memory, work)
}

fn narrowed_policy_with_workers(
    policy: worth_foundational::ExecutionRequestPolicy,
    workers: std::num::NonZeroUsize,
    memory: Option<u64>,
    work: Option<u64>,
) -> worth_foundational::ExecutionRequestPolicy {
    if policy.accounting().is_some() || memory.is_some() || work.is_some() {
        worth_foundational::ExecutionRequestPolicy::new(
            policy.posture(),
            policy.determinism(),
            worth_foundational::ExecutionBudget::new(workers, memory, work),
        )
    } else {
        worth_foundational::ExecutionRequestPolicy::for_workers(
            policy.posture(),
            policy.determinism(),
            workers,
        )
    }
}
