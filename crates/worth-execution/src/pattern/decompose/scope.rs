use std::{cell::Cell, mem::size_of};

use worth_foundational::{ExecutionPhysicalReport, ExecutionPosture};

use super::*;
use crate::backend::{run_scope_with_charge, BackendKind};

impl<I, C, S, B, O, F> ExecutionDecompose<I, C, S, B, O, F>
where
    I: Clone + Send + Sync + ChargedBytes + CanonicalBits,
    C: Clone + Send + Sync + ChargedBytes + CanonicalBits,
    S: Clone + Send + ChargedBytes,
    B: Clone + Send + Sync + ChargedBytes + CanonicalBits,
    O: Clone + Send + ChargedBytes,
    F: Fn(&C, &C) -> C + Clone + Sync,
{
    pub fn run<T, K, E, Interior, Interface, Back>(
        &mut self,
        lease: Option<&ExecutionResourceLease<'_>>,
        changed: &ExecutionMap<T, K>,
        editions: DecomposeKernelEditions,
        interior: Interior,
        solve_interface: Interface,
        back_substitute: Back,
    ) -> Result<DecomposeComplete<O>, DecomposeRunFailure<E>>
    where
        T: Sync + ChargedBytes,
        E: Send + ChargedBytes,
        Interior: Fn(
                &T,
                &mut MapKernelContext<'_, '_>,
            ) -> Result<InteriorResult<I, C>, MapKernelFailure<E>>
            + Sync,
        Interface: Fn(
                &C,
                &mut MapKernelContext<'_, '_>,
            ) -> Result<InterfaceSolution<S, B>, MapKernelFailure<E>>
            + Sync,
        Back: Fn(&BackInput<I, B>, &mut MapKernelContext<'_, '_>) -> Result<O, MapKernelFailure<E>>
            + Sync,
    {
        let backend =
            if lease.is_some_and(|value| value.resolved_posture() == ExecutionPosture::Automatic) {
                BackendKind::Native
            } else {
                BackendKind::Serial
            };
        self.run_with_backend(
            lease,
            backend,
            changed,
            editions,
            interior,
            solve_interface,
            back_substitute,
            true,
            None,
        )
    }

    pub(super) fn run_with_backend<T, K, E, Interior, Interface, Back>(
        &mut self,
        lease: Option<&ExecutionResourceLease<'_>>,
        backend: BackendKind,
        changed: &ExecutionMap<T, K>,
        editions: DecomposeKernelEditions,
        interior: Interior,
        solve_interface: Interface,
        back_substitute: Back,
        charge_parent: bool,
        mut retained_handoff: Option<&mut dyn FnMut(u64) -> Result<(), LeaseDenial>>,
    ) -> Result<DecomposeComplete<O>, DecomposeRunFailure<E>>
    where
        T: Sync + ChargedBytes,
        E: Send + ChargedBytes,
        Interior: Fn(
                &T,
                &mut MapKernelContext<'_, '_>,
            ) -> Result<InteriorResult<I, C>, MapKernelFailure<E>>
            + Sync,
        Interface: Fn(
                &C,
                &mut MapKernelContext<'_, '_>,
            ) -> Result<InterfaceSolution<S, B>, MapKernelFailure<E>>
            + Sync,
        Back: Fn(&BackInput<I, B>, &mut MapKernelContext<'_, '_>) -> Result<O, MapKernelFailure<E>>
            + Sync,
    {
        let tree_bound =
            match &self.snapshot {
                None => ReductionTree::<C, F>::checked_build_memory_bound(
                    self.identities.len(),
                    self.max_reduction_value_bytes,
                ),
                Some(snapshot) => changed.identities().iter().try_fold(
                    self.max_reduction_value_bytes,
                    |sum, identity| {
                        sum.checked_add(snapshot.tree.checked_update_memory_bound(
                            *identity,
                            self.max_reduction_value_bytes,
                        )?)
                    },
                ),
            };
        let bound = tree_bound.and_then(|tree_scratch| {
            let tree = tree_scratch.checked_add(
                self.snapshot
                    .as_ref()
                    .map_or(0, |snapshot| snapshot.tree.additional_charged_bytes()),
            )?;
            // Eight capped buffers can coexist: changed map output, merged
            // interiors, merged contributions, tree input clone, interface
            // result, back output, final outputs, and one output clone or
            // canonical comparison buffer. Nested maps reserve their own
            // admitted input and output storage separately.
            let identities = u64::try_from(
                self.identities
                    .len()
                    .checked_mul(size_of::<worth_foundational::PartitionIdentity>())?
                    .checked_mul(2)?,
            )
            .ok()?;
            let retained = self
                .max_staged_bytes
                .checked_mul(8)?
                .checked_add(tree)?
                .checked_add(identities)?
                .checked_add(u64::try_from(size_of::<F>()).ok()?)?
                .checked_add(self.reducer_storage_bytes)?;
            let result = self.max_staged_bytes.checked_mul(2)?.checked_add(tree)?;
            Some((retained, result))
        });
        let Some((retained_bytes, max_result_bytes)) = bound else {
            return Err(DecomposeRunFailure {
                cause: DecomposeFailure::Input(DecomposeInputDenial::MemoryOverflow),
                total_report: empty_report(),
            });
        };
        let stage = Cell::new(DecomposeStage::Interior);
        let scope = run_scope_with_charge(
            lease,
            retained_bytes,
            max_result_bytes,
            charge_parent,
            |_context| {
                let staged = self
                    .run_stages(
                        lease,
                        backend,
                        changed,
                        editions,
                        &interior,
                        &solve_interface,
                        &back_substitute,
                        _context,
                        &stage,
                    )
                    .map_err(MapKernelFailure::Domain)?;
                if let Some(handoff) = retained_handoff.as_mut() {
                    let bytes = staged
                        .additional_charged_bytes()
                        .checked_add(self.reducer_storage_bytes)
                        .and_then(|bytes| {
                            bytes.checked_add(
                                u64::try_from(size_of::<Staged<I, C, S, B, O, F>>()).ok()?,
                            )
                        })
                        .ok_or({
                            MapKernelFailure::Domain(DecomposeFailure::ScopeAdmission(
                                LeaseDenial::ResourceExhausted,
                            ))
                        })?;
                    handoff(bytes).map_err(|denial| {
                        MapKernelFailure::Domain(DecomposeFailure::ScopeAdmission(denial))
                    })?;
                }
                Ok(staged)
            },
        );
        match scope.result {
            Ok(mut staged) => {
                staged.complete.total_report = scope.report;
                self.snapshot = Some(staged.snapshot);
                Ok(staged.complete)
            }
            Err(stop) => {
                let cause = match stop {
                    ScopeStop::Admission(denial) => DecomposeFailure::ScopeAdmission(denial),
                    ScopeStop::Failure(MapKernelFailure::Domain(cause)) => cause,
                    ScopeStop::Failure(MapKernelFailure::Stop(reason)) => {
                        DecomposeFailure::ScopeStopped {
                            stage: stage.get(),
                            reason,
                        }
                    }
                    ScopeStop::Failure(MapKernelFailure::Panic) => {
                        DecomposeFailure::ScopePanic { stage: stage.get() }
                    }
                    ScopeStop::Failure(MapKernelFailure::ResultCapacityExceeded) => {
                        DecomposeFailure::Input(DecomposeInputDenial::StagedCapacityExceeded)
                    }
                };
                Err(DecomposeRunFailure {
                    cause,
                    total_report: scope.report,
                })
            }
        }
    }
}

pub(super) fn empty_report() -> ExecutionReport {
    ExecutionReport::new(
        ExecutionPosture::Serial,
        0,
        0,
        ExecutionPhysicalReport::default(),
    )
}
