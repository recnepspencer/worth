use super::*;

impl PoolInner {
    pub(in crate::physical_residency) fn begin_speculative_admission(
        self: &Arc<Self>,
        kind: crate::PhysicalSpeculativeWorkKind,
    ) -> crate::physical_residency::speculation::SpeculativeAdmissionAttempt {
        let mut state = self.lock();
        state.accounting.attempt_speculative(kind);
        crate::physical_residency::speculation::SpeculativeAdmissionAttempt::new(
            Arc::clone(self),
            kind,
        )
    }

    pub(in crate::physical_residency) fn record_speculative_admission_denial(
        &self,
        kind: crate::PhysicalSpeculativeWorkKind,
    ) {
        self.lock().accounting.record_speculative_denial(kind);
    }

    pub(super) fn reserve_operation(
        self: &Arc<Self>,
        scope: PhysicalOperationAllocationScope,
        bytes: std::num::NonZeroU64,
    ) -> Result<(), PhysicalResidencyDenial> {
        let mut state = self.lock();
        if !state.accepting {
            return Err(Self::deny(&mut state, PhysicalResidencyDenial::PoolClosed));
        }
        let bytes = bytes.get();
        self.admit_scope_bytes(&mut state, scope, bytes)?;
        self.admit_operation_bytes(&mut state, scope, bytes)?;
        self.admit_total_bytes(&mut state, scope, bytes)?;
        state.accounting.admit_operation(scope, bytes);
        Ok(())
    }

    fn admit_scope_bytes(
        &self,
        state: &mut PoolState,
        scope: PhysicalOperationAllocationScope,
        requested: u64,
    ) -> Result<u64, PhysicalResidencyDenial> {
        let scope_current = state.accounting.operation_scope_bytes(scope);
        let scope_limit = self.limits.usable_bytes(scope, self.limits.scope_bytes(scope));
        let scope_next = scope_current.saturating_add(requested);
        if scope_next > scope_limit {
            return Err(self.pressure(
                state,
                PhysicalResidencyPressureDemand {
                    dimension: PhysicalResidencyDimension::OperationScope(scope),
                    scope,
                    requested,
                    current: scope_current,
                    limit: scope_limit,
                },
            ));
        }
        Ok(scope_next)
    }

    fn admit_operation_bytes(
        &self,
        state: &mut PoolState,
        scope: PhysicalOperationAllocationScope,
        requested: u64,
    ) -> Result<u64, PhysicalResidencyDenial> {
        let operation_current = state.accounting.active_operation_bytes();
        let operation_limit = self
            .limits
            .usable_bytes(scope, self.limits.operation_bytes());
        let operation_next = operation_current.saturating_add(requested);
        if operation_next > operation_limit {
            return Err(self.pressure(
                state,
                PhysicalResidencyPressureDemand {
                    dimension: PhysicalResidencyDimension::OperationBytes,
                    scope,
                    requested,
                    current: operation_current,
                    limit: operation_limit,
                },
            ));
        }
        Ok(operation_next)
    }

    fn admit_total_bytes(
        &self,
        state: &mut PoolState,
        scope: PhysicalOperationAllocationScope,
        requested: u64,
    ) -> Result<(), PhysicalResidencyDenial> {
        let total_current = self.current_admitted_bytes(state);
        let total_limit = self.limits.usable_bytes(scope, self.limits.total_bytes());
        let total_next = total_current.saturating_add(requested);
        if total_next > total_limit {
            return Err(self.pressure(
                state,
                PhysicalResidencyPressureDemand {
                    dimension: PhysicalResidencyDimension::TotalBytes,
                    scope,
                    requested,
                    current: total_current,
                    limit: total_limit,
                },
            ));
        }
        Ok(())
    }

    pub(crate) fn release_operation(&self, scope: PhysicalOperationAllocationScope, bytes: u64) {
        let mut state = self.lock();
        state.accounting.release_operation(scope, bytes);
        self.changed.notify_all();
    }

    pub(crate) fn deny_operation_grant_use(
        &self,
        scope: PhysicalOperationAllocationScope,
        requested: u64,
        current: u64,
        limit: u64,
    ) -> PhysicalResidencyDenial {
        let mut state = self.lock();
        self.pressure(
            &mut state,
            PhysicalResidencyPressureDemand {
                dimension: PhysicalResidencyDimension::OperationBytes,
                scope,
                requested,
                current,
                limit,
            },
        )
    }

    pub(crate) fn record_copy(&self, bytes: u64) {
        let mut state = self.lock();
        state.accounting.record_copy(bytes);
    }

    pub(in crate::physical_residency) fn require_bounded_speculative_validation(
        &self,
        scope: PhysicalOperationAllocationScope,
        kind: crate::PhysicalSpeculativeWorkKind,
        frames: u32,
    ) -> Result<(), PhysicalResidencyDenial> {
        let limit = self.limits.speculative_frames(kind);
        if frames <= limit {
            return Ok(());
        }
        let mut state = self.lock();
        if !state.accepting {
            return Err(Self::deny(&mut state, PhysicalResidencyDenial::PoolClosed));
        }
        let current = state.accounting.active_speculative_frames(kind);
        Err(self.pressure(
            &mut state,
            PhysicalResidencyPressureDemand {
                dimension: PhysicalResidencyDimension::SpeculativeFrames(kind),
                scope,
                requested: u64::from(frames),
                current: u64::from(current),
                limit: u64::from(limit),
            },
        ))
    }

    pub(in crate::physical_residency) fn reserve_speculative(
        self: &Arc<Self>,
        scope: PhysicalOperationAllocationScope,
        kind: crate::PhysicalSpeculativeWorkKind,
        frames: u32,
    ) -> Result<(), PhysicalResidencyDenial> {
        let mut state = self.lock();
        if !state.accepting {
            return Err(Self::deny(&mut state, PhysicalResidencyDenial::PoolClosed));
        }
        if frames == 0 {
            let current = u64::from(state.accounting.active_speculative_frames(kind));
            return Err(self.pressure(
                &mut state,
                PhysicalResidencyPressureDemand {
                    dimension: PhysicalResidencyDimension::SpeculativeFrames(kind),
                    scope,
                    requested: 0,
                    current,
                    limit: u64::from(self.limits.speculative_frames(kind)),
                },
            ));
        }
        let limit = self.limits.speculative_frames(kind);
        let current = state.accounting.active_speculative_frames(kind);
        let Some(next) = current.checked_add(frames) else {
            return Err(self.pressure(
                &mut state,
                PhysicalResidencyPressureDemand {
                    dimension: PhysicalResidencyDimension::SpeculativeFrames(kind),
                    scope,
                    requested: u64::from(frames),
                    current: u64::from(current),
                    limit: u64::from(limit),
                },
            ));
        };
        if next > limit {
            return Err(self.pressure(
                &mut state,
                PhysicalResidencyPressureDemand {
                    dimension: PhysicalResidencyDimension::SpeculativeFrames(kind),
                    scope,
                    requested: u64::from(frames),
                    current: u64::from(current),
                    limit: u64::from(limit),
                },
            ));
        }
        if kind == crate::PhysicalSpeculativeWorkKind::WriteBehind
            && next > state.accounting.dirty_frames()
        {
            return Err(Self::deny(
                &mut state,
                PhysicalResidencyDenial::WriteBackExceedsDirtyPosture,
            ));
        }
        state.accounting.admit_speculative(kind, frames);
        Ok(())
    }

    pub(crate) fn release_speculative(
        &self,
        kind: crate::PhysicalSpeculativeWorkKind,
        frames: u32,
    ) {
        let mut state = self.lock();
        state.accounting.release_speculative(kind, frames);
        self.changed.notify_all();
    }
}
