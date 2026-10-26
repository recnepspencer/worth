use super::{
    EpochRetryReceipt, PhysicalByteGuardAdmission, PhysicalReadExecutionDenial,
    PhysicalReadIoAttempt, PhysicalReadIoPosture, StablePhysicalReadEpochFreshnessOutcome,
    StablePhysicalReadExecutionCounters, StablePhysicalReadExecutionOutcome,
    StablePhysicalReadReceipt,
};
use crate::physical_runtime::stability::{
    LogicalDecodeSecurityScopeEntry, PhysicalByteGuard, PhysicalByteGuardScope,
};
use worth_proof::TransitionOutcome;
use worth_store_physical_isolation::{
    CurrentPhysicalRoot, PhysicalReadPlanRetryPosture, StablePhysicalReadHandle,
};

#[derive(Debug)]
pub struct StablePhysicalReadExecution {
    handle: StablePhysicalReadHandle,
    counters: StablePhysicalReadExecutionCounters,
    io_posture: PhysicalReadIoPosture,
}

#[derive(Debug, Clone, Copy)]
pub struct ByteGuardedPhysicalRead<'a> {
    scope: PhysicalByteGuardScope,
    bytes: &'a [u8],
    _execution: &'a StablePhysicalReadExecution,
}

impl StablePhysicalReadExecution {
    pub fn from_execution_ready_handle(handle: StablePhysicalReadHandle) -> Self {
        let counters =
            StablePhysicalReadExecutionCounters::from_plan_counters(handle.plan().counters());
        Self {
            handle,
            counters,
            io_posture: PhysicalReadIoPosture::ordinary(),
        }
    }

    pub fn admit_byte_guard(
        &mut self,
        scope: PhysicalByteGuardScope,
    ) -> Result<PhysicalByteGuardAdmission, PhysicalReadExecutionDenial> {
        match PhysicalByteGuardAdmission::from_execution_handle(&self.handle, scope) {
            Ok(admission) => {
                self.counters = self
                    .counters
                    .with_compact_footprint_check()
                    .with_guard_admission();
                Ok(admission)
            }
            Err(denial) => {
                self.counters = self
                    .counters
                    .with_compact_footprint_check()
                    .with_execution_time_reference_discovery();
                Err(PhysicalReadExecutionDenial::ReadPlanDenied(denial))
            }
        }
    }

    pub(crate) fn read_guarded_bytes<'a>(
        &'a mut self,
        guard: &'a PhysicalByteGuard<'a>,
    ) -> Result<ByteGuardedPhysicalRead<'a>, PhysicalReadExecutionDenial> {
        self.reject_guard_from_different_footprint(guard)?;
        let bytes = guard.bytes_for_execution();
        self.counters = self.counters.with_guarded_byte_read(bytes.len() as u64);
        Ok(ByteGuardedPhysicalRead {
            scope: guard.scope(),
            bytes,
            _execution: self,
        })
    }

    pub fn read_guarded_bytes_with_security_scope<'a>(
        &'a mut self,
        guard: &'a PhysicalByteGuard<'a>,
        logical_decode_entry: LogicalDecodeSecurityScopeEntry,
    ) -> Result<ByteGuardedPhysicalRead<'a>, PhysicalReadExecutionDenial> {
        self.reject_logical_decode_scope_mismatch(logical_decode_entry, guard)?;
        self.read_guarded_bytes(guard)
    }

    pub fn read_guarded_bytes_after_io_attempt<'a>(
        &'a mut self,
        attempt: PhysicalReadIoAttempt,
        guard: &'a PhysicalByteGuard<'a>,
        logical_decode_entry: LogicalDecodeSecurityScopeEntry,
    ) -> Result<ByteGuardedPhysicalRead<'a>, PhysicalReadExecutionDenial> {
        if attempt.requires_declared_structural_latch_io_cost() {
            self.reject_blocking_io_while_holding_structural_latch()?;
        }
        self.read_guarded_bytes_with_security_scope(guard, logical_decode_entry)
    }

    pub fn observe_epoch_freshness(
        &mut self,
        observed_root: CurrentPhysicalRoot,
    ) -> StablePhysicalReadEpochFreshnessOutcome {
        let admitted = self.handle.plan().root();
        if observed_root.epoch().get() == admitted.epoch().get()
            && observed_root.manifest_epoch().get() == admitted.manifest_epoch().get()
        {
            return TransitionOutcome::success(());
        }

        self.counters = self.counters.with_retry_decision();
        let receipt = EpochRetryReceipt::new(
            admitted.epoch(),
            observed_root.epoch(),
            admitted.manifest_epoch(),
            observed_root.manifest_epoch(),
            classify_retry_posture(admitted, observed_root),
            self.counters,
        );
        if observed_root.manifest_epoch().get() == admitted.manifest_epoch().get() {
            TransitionOutcome::stale(receipt)
        } else {
            TransitionOutcome::rebind_required(receipt)
        }
    }

    pub fn reject_blocking_io_while_holding_structural_latch(
        &mut self,
    ) -> Result<(), PhysicalReadExecutionDenial> {
        if self
            .io_posture
            .permits_blocking_io_while_holding_structural_latch()
        {
            self.counters = self.counters.with_blocking_io_event();
            Ok(())
        } else {
            self.counters = self
                .counters
                .with_blocking_io_event()
                .with_hidden_latch_io_denial();
            Err(
                PhysicalReadExecutionDenial::HiddenStructuralLatchIoWithoutDeclaredCost {
                    counters: self.counters,
                },
            )
        }
    }

    pub const fn counters(&self) -> StablePhysicalReadExecutionCounters {
        self.counters
    }

    pub const fn io_posture(&self) -> PhysicalReadIoPosture {
        self.io_posture
    }

    pub fn complete(self) -> StablePhysicalReadReceipt {
        StablePhysicalReadReceipt::new(self.handle.complete_plan(), self.counters, self.io_posture)
    }

    pub fn complete_with_proof(self) -> StablePhysicalReadExecutionOutcome {
        TransitionOutcome::success(self.complete())
    }

    fn reject_guard_from_different_footprint(
        &self,
        guard: &PhysicalByteGuard<'_>,
    ) -> Result<(), PhysicalReadExecutionDenial> {
        let expected = self.handle.plan().footprint().declared_footprint_basis();
        if guard.footprint_basis() == expected {
            Ok(())
        } else {
            Err(PhysicalReadExecutionDenial::GuardScopeNotInPlan {
                scope: guard.scope(),
                counters: self.counters,
            })
        }
    }

    fn reject_logical_decode_scope_mismatch(
        &self,
        logical_decode_entry: LogicalDecodeSecurityScopeEntry,
        guard: &PhysicalByteGuard<'_>,
    ) -> Result<(), PhysicalReadExecutionDenial> {
        let expected_root = self.handle.plan().root();
        if logical_decode_entry.observed_root() != expected_root {
            return Err(
                PhysicalReadExecutionDenial::LogicalDecodeScopeRootMismatch {
                    admitted: expected_root,
                    observed: logical_decode_entry.observed_root(),
                },
            );
        }

        let expected_footprint = self.handle.plan().footprint().declared_footprint_basis();
        if logical_decode_entry.footprint_basis() != expected_footprint {
            return Err(
                PhysicalReadExecutionDenial::LogicalDecodeScopeFootprintMismatch {
                    admitted: expected_footprint,
                    observed: logical_decode_entry.footprint_basis(),
                },
            );
        }

        if logical_decode_entry.footprint_basis() != guard.footprint_basis() {
            return Err(
                PhysicalReadExecutionDenial::LogicalDecodeScopeFootprintMismatch {
                    admitted: logical_decode_entry.footprint_basis(),
                    observed: guard.footprint_basis(),
                },
            );
        }

        if logical_decode_entry.guard_scope() != guard.scope() {
            return Err(PhysicalReadExecutionDenial::ByteGuardScopeMismatch {
                admitted: logical_decode_entry.guard_scope(),
                observed: guard.scope(),
            });
        }

        if !logical_decode_entry
            .carrier_basis()
            .matches_guard_scope(guard.scope())
        {
            return Err(
                PhysicalReadExecutionDenial::LogicalDecodeScopeCarrierMismatch {
                    admitted: logical_decode_entry.carrier_basis(),
                    observed: guard.scope(),
                },
            );
        }

        Ok(())
    }
}

fn classify_retry_posture(
    admitted: CurrentPhysicalRoot,
    observed: CurrentPhysicalRoot,
) -> PhysicalReadPlanRetryPosture {
    if observed.manifest_epoch().get() == admitted.manifest_epoch().get() {
        PhysicalReadPlanRetryPosture::Retry
    } else {
        PhysicalReadPlanRetryPosture::RebindRequired
    }
}

impl<'a> ByteGuardedPhysicalRead<'a> {
    pub const fn scope(self) -> PhysicalByteGuardScope {
        self.scope
    }

    pub const fn physical_bytes(self) -> &'a [u8] {
        let _ = self._execution.counters();
        self.bytes
    }
}
