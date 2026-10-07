//! How one request's managed computations run: opened once at request entry
//! from the World's execution placement, and dropped when the request ends.
//!
//! The form is chosen here and nowhere else. A leased request owns the lease
//! it drew from the World's authority, and each dispatch runs on a child of
//! it, so no run consumes the request's lease. A serial request carries its
//! cancellation, its deadline and, with a policy, the policy's memory.

use worth_execution::{
    CanonicalBits, ChargedBytes, ExecutionMap, ExecutionMemoryReservation, ExecutionResourceLease,
    ExecutionWorkCeiling, LeaseDenial, LeaseRequest, MapKernelContext, MapKernelFailure,
    MapKernelStop, MapOutcome, ReduceInputDenial, ReductionMetrics, ReductionTree,
    SerialMemoryBudget, SerialRequest, WorkCeilingDenial,
};
use worth_foundational::{ExecutionReport, ExecutionRequestPolicy};
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryRequestInterruption, WorthQueryRequestScope,
};
use worth_runtime_world::facade::RuntimeWorldExecutionPlacement;

use super::execution_denial::{interruption_stop, lease_denial, memory_denial};
use super::{WorthQueryManagedComputationInterruption, WorthQueryManagedComputationResourceDenial};

#[cfg(any(test, feature = "test-query-execution-observer"))]
mod test_placement;
#[cfg(feature = "test-query-execution-observer")]
pub use test_placement::{
    place_managed_computations_on_this_thread_for_test, test_execution_workers,
    WorthQueryExecutionPlacementForTest,
};
#[cfg(all(test, not(feature = "test-query-execution-observer")))]
pub(in crate::domain_computation::primary_graph) use test_placement::{
    place_managed_computations_on_this_thread_for_test, WorthQueryExecutionPlacementForTest,
};
#[cfg(test)]
pub(in crate::domain_computation::primary_graph) use test_placement::{
    test_authority, test_policy,
};

type Resource = WorthQueryManagedComputationResourceDenial;

/// One request's execution, borrowed from the World owner the request entry
/// holds for the request's duration.
pub(in crate::domain_computation::primary_graph) struct QueryRequestExecution<'a> {
    request: &'a WorthQueryRequestScope,
    form: Form<'a>,
}

enum Form<'a> {
    /// The lease the request drew, or why the authority refused it. A
    /// refusal reaches only the runs that dispatch or reserve.
    Leased {
        lease: Result<ExecutionResourceLease<'a>, LeaseDenial>,
        policy: ExecutionRequestPolicy,
    },
    Serial(SerialRequest),
}

impl<'a> QueryRequestExecution<'a> {
    /// The request's execution under `placement`. The request's cancellation
    /// is the one every lease and serial run observes, and its deadline is
    /// theirs.
    pub(in crate::domain_computation::primary_graph) fn open(
        placement: RuntimeWorldExecutionPlacement<'a>,
        request: &'a WorthQueryRequestScope,
    ) -> Self {
        #[cfg(any(test, feature = "test-query-execution-observer"))]
        let placement = test_placement::placed(placement);
        let cancellation = request.cancellation().execution_token();
        let deadline = Some(request.deadline());
        let form = match placement {
            RuntimeWorldExecutionPlacement::Serial(policy) => {
                Form::Serial(SerialRequest::from_memory(
                    SerialMemoryBudget::from_policy(&policy),
                    cancellation,
                    deadline,
                ))
            }
            RuntimeWorldExecutionPlacement::Leased { authority, policy } => Form::Leased {
                lease: authority.request_lease(LeaseRequest {
                    policy,
                    deadline,
                    cancellation,
                }),
                policy,
            },
        };
        Self { request, form }
    }

    pub(in crate::domain_computation::primary_graph) const fn request(
        &self,
    ) -> &'a WorthQueryRequestScope {
        self.request
    }

    /// The request's cancellation or elapsed deadline, if either happened.
    pub(super) fn interruption(&self) -> Option<WorthQueryManagedComputationInterruption> {
        self.request
            .interruption()
            .map(|interruption| match interruption {
                WorthQueryRequestInterruption::Cancelled => {
                    WorthQueryManagedComputationInterruption::Cancelled
                }
                WorthQueryRequestInterruption::DeadlineExceeded => {
                    WorthQueryManagedComputationInterruption::DeadlineExceeded
                }
            })
    }

    /// A safe point outside any pattern: the stop a kernel would make here.
    pub(super) fn checkpoint(&self) -> Result<(), MapKernelStop> {
        self.interruption()
            .map_or(Ok(()), |interruption| Err(interruption_stop(interruption)))
    }

    /// Holds `bytes` on the request's memory until the reservation drops.
    pub(super) fn reserve(&self, bytes: u64) -> Result<QueryMemoryReservation, Resource> {
        let held = match &self.form {
            Form::Leased { lease, .. } => lease
                .as_ref()
                .map_err(|denial| lease_denial(*denial))?
                .reserve_memory(bytes)
                .map(Some),
            Form::Serial(request) => request.memory().reserve(bytes).map(Some),
        };
        held.map(|held| QueryMemoryReservation { held, bytes })
            .map_err(memory_denial)
    }

    /// The placement for one run: a child of the request's lease, or the
    /// serial request itself.
    pub(super) fn dispatch(&self) -> Result<QueryDispatch<'_>, Resource> {
        let form = match &self.form {
            Form::Leased { lease, policy } => {
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
            Form::Serial(request) => DispatchForm::Serial(request),
        };
        Ok(QueryDispatch { form })
    }
}

/// Bytes held on the request's memory. A request with no memory bound holds
/// nothing and refuses nothing, but still counts what it would hold.
pub(super) struct QueryMemoryReservation {
    held: Option<ExecutionMemoryReservation>,
    bytes: u64,
}

impl QueryMemoryReservation {
    pub(super) const fn bytes(&self) -> u64 {
        self.bytes
    }

    /// Holds `bytes` in place of what was held; a refusal keeps what was.
    pub(super) fn resize(&mut self, bytes: u64) -> Result<(), Resource> {
        if let Some(held) = &mut self.held {
            held.resize(bytes).map_err(memory_denial)?;
        }
        self.bytes = bytes;
        Ok(())
    }

    /// Holds `bytes` more.
    pub(super) fn grow(&mut self, bytes: u64) -> Result<(), Resource> {
        let total = self
            .bytes
            .checked_add(bytes)
            .ok_or(Resource::CapacityOverflow)?;
        self.resize(total)
    }
}

/// What a completed tree keeps once its run ends, as execution counts it
/// for a reducer with no declared storage: its nodes and the tree itself.
pub(super) fn kept_tree_bytes<R: ChargedBytes, Combine>(
    tree: &ReductionTree<R, Combine>,
) -> Option<u64> {
    u64::try_from(size_of::<ReductionTree<R, Combine>>())
        .ok()?
        .checked_add(tree.additional_charged_bytes())
}

enum DispatchForm<'d> {
    Leased(ExecutionResourceLease<'d>),
    Serial(&'d SerialRequest),
}

/// One run's placement. Each pattern runs inside a work ceiling on it.
pub(super) struct QueryDispatch<'d> {
    form: DispatchForm<'d>,
}

/// What a reduction produced, as `run_reduce` returns it.
pub(super) type Reduced<R, Combine, E> =
    Result<(ReductionTree<R, Combine>, ExecutionReport, ReductionMetrics), ReduceInputDenial<E>>;

impl QueryDispatch<'_> {
    /// Maps every partition of `map` inside `ceiling`. The map's admission
    /// takes over `inputs`, the request memory held for its partitions, in
    /// one ledger step: no byte of them is unreserved between the two holds.
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
        let ceiling = ExecutionWorkCeiling::new(ceiling);
        let run = |lease: Option<&ExecutionResourceLease<'_>>| match inputs.held {
            Some(inputs) => map.run_taking(lease, inputs, kernel),
            None => map.run(lease, kernel),
        };
        match &self.form {
            DispatchForm::Leased(lease) => ceiling.run(lease, || run(Some(lease))),
            DispatchForm::Serial(request) => ceiling.run_serial(request, || run(None)),
        }
        .map(|(outcome, _)| outcome)
    }

    /// Maps every partition of `map` and reduces the results over the
    /// canonical tree, inside `ceiling`, taking over `inputs` as
    /// [`Self::map`] does. The reduction hands what its completed tree keeps
    /// to `tree` in one ledger step, so the tree is held from its build until
    /// `tree` drops. Where a test chose certification, a completed reduction
    /// is then certified outside the ceiling.
    pub(super) fn reduce<T, K, R, E, Kernel, Combine>(
        &self,
        ceiling: u64,
        map: &ExecutionMap<T, K>,
        inputs: QueryMemoryReservation,
        tree: &mut QueryMemoryReservation,
        kernel: Kernel,
        identity: R,
        combine: Combine,
        max_value_bytes: u64,
    ) -> Result<Reduced<R, Combine, E>, WorkCeilingDenial>
    where
        T: Sync + ChargedBytes,
        R: Send + Sync + ChargedBytes + Clone + CanonicalBits,
        E: Send + ChargedBytes,
        Kernel:
            Fn(&T, &mut MapKernelContext<'_, '_>) -> Result<R, MapKernelFailure<E>> + Clone + Sync,
        Combine: Fn(&R, &R) -> R + Clone + Sync,
    {
        #[cfg(any(test, feature = "test-query-execution-observer"))]
        let certification = test_placement::certifying()
            .map(|seed| (seed, kernel.clone(), identity.clone(), combine.clone()));
        let ceiling = ExecutionWorkCeiling::new(ceiling);
        let held = tree.held.as_mut();
        let run = |lease: Option<&ExecutionResourceLease<'_>>| {
            map.run_reduce_holding(
                lease,
                inputs.held,
                held,
                kernel,
                identity,
                combine,
                max_value_bytes,
                0,
            )
        };
        let reduced = match &self.form {
            DispatchForm::Leased(lease) => ceiling.run(lease, || run(Some(lease))),
            DispatchForm::Serial(request) => ceiling.run_serial(request, || run(None)),
        }
        .map(|(reduced, _)| reduced);
        // The request counts the memory its retained tree keeps.
        tree.bytes = match (&tree.held, &reduced) {
            (Some(held), _) => held.bytes(),
            (None, Ok(Ok((reduced, ..)))) => kept_tree_bytes(reduced).unwrap_or(u64::MAX),
            (None, _) => 0,
        };
        #[cfg(any(test, feature = "test-query-execution-observer"))]
        if let (
            Some((seed, kernel, identity, combine)),
            DispatchForm::Leased(lease),
            Ok(Ok((tree, report, _))),
        ) = (certification, &self.form, &reduced)
        {
            test_placement::certify(
                map,
                lease,
                seed,
                kernel,
                identity,
                combine,
                max_value_bytes,
                (tree, report),
            );
        }
        reduced
    }
}
