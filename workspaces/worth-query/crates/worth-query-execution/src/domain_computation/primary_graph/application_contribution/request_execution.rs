//! How one request's managed computations run: opened once at request entry
//! from the World's execution placement, and dropped when the request ends.
//!
//! The form is chosen here and nowhere else. A leased request owns the lease
//! it drew from the World's authority, and each dispatch runs on a child of
//! it, so no run consumes the request's lease. Serial placement draws a
//! bounded one-worker lease from an inline ledger with no worker pool.

use worth_execution::{
    CanonicalBits, ExecutionMap, ExecutionMemoryReservation, ExecutionResourceLease,
    ExecutionWorkCeiling, LeaseDenial, LeaseRequest, MapKernelContext, MapKernelFailure,
    MapKernelStop, WorkCeilingDenial,
};
#[cfg(test)]
use worth_execution::{ChargedBytes, MapOutcome};
use worth_foundational::{ExecutionReport, ExecutionRequestPolicy};
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_runtime_world::facade::RuntimeWorldExecutionPlacement;

use super::execution_denial::{interruption_stop, lease_denial, memory_denial};
use super::{WorthQueryManagedComputationInterruption, WorthQueryManagedComputationResourceDenial};

mod test_placement;
#[cfg(test)]
pub(in crate::domain_computation::primary_graph) use test_placement::test_policy;
#[cfg(test)]
pub(in crate::domain_computation::primary_graph) use test_placement::FixturePlacement;
pub use test_placement::{
    bound_advancement_requests_on_this_thread_for_test,
    place_managed_computations_on_this_thread_for_test, test_execution_workers_for_test,
    WorthQueryExecutionPlacementForTest,
};
mod serial_placement;
mod test_authority;
pub use test_authority::query_test_execution_authority_for_test;

type Resource = WorthQueryManagedComputationResourceDenial;

mod accounting;
mod decision_stage;
mod descendants;
mod retained_dispatch;
mod tree_charge;
pub(in crate::domain_computation::primary_graph) use descendants::MemberRequestExecution;
pub(in crate::domain_computation::primary_graph) mod advancement;
pub(in crate::domain_computation::primary_graph) use advancement::advancement_denial;

pub use advancement::{
    WorthQueryAdvancementDenial, WorthQueryAdvancementPhase, WorthQueryForeignAdvancementPhase,
};

/// One request's execution, borrowed from the World owner the request entry
/// holds for the request's duration.
pub(in crate::domain_computation::primary_graph) struct QueryRequestExecution<'a> {
    cancellation: worth_execution::CancellationToken,
    deadline: Option<std::time::Instant>,
    declared_work: Option<u64>,
    certified_seed: Option<u64>,
    form: Form<'a>,
}

enum Form<'a> {
    /// The lease the request drew, or why the authority refused it. A
    /// refusal reaches only the runs that dispatch or reserve.
    Leased {
        lease: Result<ExecutionResourceLease<'a>, LeaseDenial>,
        policy: ExecutionRequestPolicy,
    },
    Serial {
        lease: Result<ExecutionResourceLease<'a>, LeaseDenial>,
        policy: ExecutionRequestPolicy,
    },
}

impl<'a> QueryRequestExecution<'a> {
    /// The actual current lease funds new persistent cells; the policy is
    /// borrowed for allocation only and is never stored in retained state.
    pub(in crate::domain_computation::primary_graph) fn allocation_policy(
        &self,
    ) -> Result<worth_execution::ExecutionAllocationPolicy<'_, 'a>, Resource> {
        match &self.form {
            Form::Leased { lease, .. } | Form::Serial { lease, .. } => lease
                .as_ref()
                .map(worth_execution::ExecutionAllocationPolicy::Execution)
                .map_err(|e| lease_denial(*e)),
        }
    }
    /// The request's execution under `placement`. The request's cancellation
    /// is the one every lease and serial run observes, and its deadline is
    /// theirs.
    pub(in crate::domain_computation::primary_graph) fn open(
        placement: RuntimeWorldExecutionPlacement<'a>,
        request: &'a WorthQueryRequestScope,
    ) -> Self {
        let cancellation = request.cancellation().execution_token();
        let deadline = request.deadline();
        Self::open_control(placement, cancellation, deadline)
    }

    fn open_control(
        placement: RuntimeWorldExecutionPlacement<'a>,
        cancellation: worth_execution::CancellationToken,
        deadline: Option<std::time::Instant>,
    ) -> Self {
        let placement = test_placement::placed(placement);
        let declared_work = match placement {
            RuntimeWorldExecutionPlacement::Serial(policy)
            | RuntimeWorldExecutionPlacement::Leased { policy, .. } => policy.work_ceiling(),
        };
        let form = match placement {
            RuntimeWorldExecutionPlacement::Serial(policy) => {
                serial_placement::form(policy, cancellation.clone(), deadline)
            }
            RuntimeWorldExecutionPlacement::Leased { authority, policy } => Form::Leased {
                lease: authority.request_lease(LeaseRequest {
                    policy,
                    deadline,
                    cancellation: cancellation.clone(),
                }),
                policy,
            },
        };
        Self {
            cancellation,
            deadline,
            declared_work,
            certified_seed: test_placement::certifying(),
            form,
        }
    }

    /// The request's cancellation or elapsed deadline, if either happened.
    pub(in crate::domain_computation::primary_graph) fn interruption(
        &self,
    ) -> Option<WorthQueryManagedComputationInterruption> {
        if self.cancellation.is_cancelled() {
            Some(WorthQueryManagedComputationInterruption::Cancelled)
        } else if self
            .deadline
            .is_some_and(|deadline| std::time::Instant::now() >= deadline)
        {
            Some(WorthQueryManagedComputationInterruption::DeadlineExceeded)
        } else {
            None
        }
    }

    /// A safe point outside any pattern: the stop a kernel would make here.
    pub(super) fn checkpoint(&self) -> Result<(), MapKernelStop> {
        self.interruption()
            .map_or(Ok(()), |interruption| Err(interruption_stop(interruption)))
    }

    /// Holds `bytes` on the request's memory until the reservation drops.
    pub(in crate::domain_computation::primary_graph) fn reserve(
        &self,
        bytes: u64,
    ) -> Result<QueryMemoryReservation, Resource> {
        let lease = match &self.form {
            Form::Leased { lease, .. } | Form::Serial { lease, .. } => {
                lease.as_ref().map_err(|denial| lease_denial(*denial))?
            }
        };
        let held = worth_execution::ExecutionRequest::leased(lease)
            .in_scope(|active| {
                active
                    .expect("leased projection retains its backing")
                    .reserve_memory(bytes)
                    .map(Some)
            })
            .expect("leased projection reborrows without opening a scope");
        held.map(|held| QueryMemoryReservation { held, bytes })
            .map_err(memory_denial)
    }

    /// Every run, including serial placement, descends from the request's lease.
    pub(super) fn dispatch(&self) -> Result<QueryDispatch<'_>, Resource> {
        let form = match &self.form {
            Form::Leased { lease, policy } | Form::Serial { lease, policy } => {
                let lease = lease.as_ref().map_err(|denial| lease_denial(*denial))?;
                DispatchForm::Leased(
                    lease
                        .child(LeaseRequest {
                            policy: *policy,
                            deadline: None,
                            cancellation: worth_execution::CancellationToken::new(),
                        })
                        .map_err(lease_denial)?,
                )
            }
        };
        Ok(QueryDispatch {
            form,
            certified_seed: self.certified_seed,
        })
    }
}

/// Bytes held against the request's lease lineage until the reservation drops.
pub(in crate::domain_computation::primary_graph) struct QueryMemoryReservation {
    held: Option<ExecutionMemoryReservation>,
    bytes: u64,
}

impl QueryMemoryReservation {
    pub(in crate::domain_computation::primary_graph) const fn bytes(&self) -> u64 {
        self.bytes
    }

    /// Holds `bytes` in place of what was held; a refusal keeps what was.
    pub(in crate::domain_computation::primary_graph) fn resize(
        &mut self,
        bytes: u64,
    ) -> Result<(), Resource> {
        if let Some(held) = &mut self.held {
            held.resize(bytes).map_err(memory_denial)?;
        }
        self.bytes = bytes;
        Ok(())
    }

    /// Holds `bytes` more.
    pub(in crate::domain_computation::primary_graph) fn grow(
        &mut self,
        bytes: u64,
    ) -> Result<(), Resource> {
        let total = self
            .bytes
            .checked_add(bytes)
            .ok_or(Resource::CapacityOverflow)?;
        self.resize(total)
    }
}

enum DispatchForm<'d> {
    Leased(ExecutionResourceLease<'d>),
}

/// One run's placement. Each pattern runs inside a work ceiling on it.
pub(super) struct QueryDispatch<'d> {
    form: DispatchForm<'d>,
    certified_seed: Option<u64>,
}

impl QueryDispatch<'_> {
    #[cfg(test)]
    pub(in super::super) fn native_lease_for_test(&self) -> &ExecutionResourceLease<'_> {
        let DispatchForm::Leased(lease) = &self.form;
        lease
    }
    /// Maps every partition of `map` inside `ceiling`. The map's admission
    /// takes over `inputs`, the request memory held for its partitions, in
    /// one ledger step: no byte of them is unreserved between the two holds.
    #[cfg(test)]
    pub(super) fn map<T, K, R, E, Kernel>(
        &self,
        ceiling: u64,
        map: &ExecutionMap<T, K>,
        inputs: QueryMemoryReservation,
        kernel: Kernel,
    ) -> Result<MapOutcome<R, E>, WorkCeilingDenial>
    where
        T: Sync + ChargedBytes,
        R: Send + ChargedBytes,
        E: Send + ChargedBytes,
        Kernel: Fn(&T, &mut MapKernelContext<'_, '_>) -> Result<R, MapKernelFailure<E>> + Sync,
    {
        let ceiling = ExecutionWorkCeiling::new(Some(ceiling));
        let run = |lease: Option<&ExecutionResourceLease<'_>>| match inputs.held {
            Some(inputs) => map.run_taking(lease, inputs, kernel),
            None => map.run(lease, kernel),
        };
        match &self.form {
            DispatchForm::Leased(lease) => ceiling.run(lease, || run(Some(lease))),
        }
        .map(|(outcome, _)| outcome)
    }
}

pub use advancement::{
    advancement_observations_on_this_thread_for_test, advancement_requests_on_this_thread_for_test,
    caller_pass_reports_on_this_thread_for_test, WorthQueryAdvancementEventForTest,
    WorthQueryAdvancementEventKindForTest, WorthQueryAdvancementIdentityForTest,
    WorthQueryAdvancementObservationForTest,
};

#[cfg(test)]
pub(crate) use advancement::{with_test_advancement, TestHostRequest};

pub use advancement::{with_bootstrap_advancement, with_bootstrap_advancement_from_request};

pub use advancement::WorthQueryBootstrapAdvancementPhase;

pub(in crate::domain_computation::primary_graph) use advancement::record_caller_pass_for_test;

pub(in crate::domain_computation::primary_graph) use advancement::observation::record_event;

pub(in crate::domain_computation::primary_graph) fn certified_wave_seed_for_test(
    request: &QueryRequestExecution<'_>,
) -> Option<u64> {
    request.certified_seed
}
