use worth_store_buffer_pool::PhysicalFrameLease;
use worth_store_physical_integrity::{
    PhysicalArtifactScope, PhysicalIntegrityRejection, PhysicalIntegrityValidationRecord,
};

use super::{
    denial::ResidentIntegrityAdmissionDenial, record_binding::ResidentIntegrityRecordBinding,
    source_scope::require_exact_resident_source,
};
use crate::physical_runtime::{
    lifecycle::{LifecycleState, LifecycleStateSnapshot, ObservedLifecyclePhase},
    ResidentAdmissionCounterCells,
};

#[derive(Clone)]
pub(in crate::physical_runtime) struct ResidentAdmissionContext<'counter> {
    lifecycle: std::sync::Arc<LifecycleState>,
    snapshot: LifecycleStateSnapshot,
    counters: ResidentAdmissionCounters<'counter>,
}

#[derive(Clone)]
enum ResidentAdmissionCounters<'counter> {
    Borrowed(&'counter ResidentAdmissionCounterCells),
    Shared(std::sync::Arc<ResidentAdmissionCounterCells>),
}

impl ResidentAdmissionCounters<'_> {
    fn cells(&self) -> &ResidentAdmissionCounterCells {
        match self {
            Self::Borrowed(counters) => counters,
            Self::Shared(counters) => counters,
        }
    }
}

impl<'counter> ResidentAdmissionContext<'counter> {
    pub(in crate::physical_runtime) fn new(
        lifecycle: std::sync::Arc<LifecycleState>,
        counters: &'counter ResidentAdmissionCounterCells,
    ) -> Self {
        let snapshot = lifecycle.snapshot();
        Self {
            lifecycle,
            snapshot,
            counters: ResidentAdmissionCounters::Borrowed(counters),
        }
    }

    pub(in crate::physical_runtime) fn from_shared(
        lifecycle: std::sync::Arc<LifecycleState>,
        counters: std::sync::Arc<ResidentAdmissionCounterCells>,
    ) -> ResidentAdmissionContext<'static> {
        let snapshot = lifecycle.snapshot();
        ResidentAdmissionContext {
            lifecycle,
            snapshot,
            counters: ResidentAdmissionCounters::Shared(counters),
        }
    }

    pub(super) fn exact_input<'lease>(
        &self,
        lease: &'lease PhysicalFrameLease,
        scope: PhysicalArtifactScope,
    ) -> Result<
        worth_store_physical_integrity::UntrustedPhysicalArtifact<'lease>,
        ResidentIntegrityAdmissionDenial,
    > {
        self.require_live()?;
        require_exact_resident_source(lease, scope)
            .map_err(|denial| self.refuse_before_owner_entry(denial))
    }

    pub(super) fn reuse<'lease>(
        &self,
        lease: &'lease PhysicalFrameLease,
        scope: PhysicalArtifactScope,
    ) -> Result<Option<ResidentIntegrityRecordBinding<'lease>>, ResidentIntegrityAdmissionDenial>
    {
        self.require_live()?;
        match ResidentIntegrityRecordBinding::reuse_exact(
            lease,
            std::sync::Arc::clone(&self.lifecycle),
            self.snapshot,
            scope,
        ) {
            Ok(Some(binding)) => {
                self.counters.cells().observe_exact_record_reuse();
                Ok(Some(binding))
            }
            Ok(None) => Ok(None),
            Err(denial) => Err(self.refuse_before_owner_entry(denial)),
        }
    }

    pub(super) fn bind_validated<'lease>(
        &self,
        lease: &'lease PhysicalFrameLease,
        scope: PhysicalArtifactScope,
        record: PhysicalIntegrityValidationRecord,
    ) -> Result<ResidentIntegrityRecordBinding<'lease>, ResidentIntegrityAdmissionDenial> {
        self.require_live()?;
        ResidentIntegrityRecordBinding::bind_fresh(
            lease,
            std::sync::Arc::clone(&self.lifecycle),
            self.snapshot,
            scope,
            record,
        )
        .map_err(|denial| self.refuse_before_owner_entry(denial))
    }

    pub(super) fn observe_fresh_validation(&self) {
        self.counters.cells().observe_fresh_validation();
    }

    pub(super) fn validation_rejected<T>(
        &self,
        rejection: PhysicalIntegrityRejection,
    ) -> Result<T, ResidentIntegrityAdmissionDenial> {
        Err(self.refuse_before_owner_entry(ResidentIntegrityAdmissionDenial::Validation(rejection)))
    }

    pub(super) fn deny<T>(
        &self,
        denial: ResidentIntegrityAdmissionDenial,
    ) -> Result<T, ResidentIntegrityAdmissionDenial> {
        Err(self.refuse_before_owner_entry(denial))
    }

    pub(super) fn refuse_source(
        &self,
        denial: ResidentIntegrityAdmissionDenial,
    ) -> ResidentIntegrityAdmissionDenial {
        self.refuse_before_owner_entry(denial)
    }

    pub(super) fn with_owner_decoder<'lease, T, F>(
        &self,
        binding: ResidentIntegrityRecordBinding<'lease>,
        decoder: F,
    ) -> Result<T, ResidentIntegrityAdmissionDenial>
    where
        F: for<'view> FnOnce(&'view PhysicalFrameLease, PhysicalArtifactScope) -> T,
    {
        self.require_live()?;
        let lease = match binding.enter_owner_decoder() {
            Ok(lease) => lease,
            Err(denial) => return Err(self.refuse_before_owner_entry(denial)),
        };
        self.counters.cells().observe_owner_decoder_entry();
        let result = decoder(lease, binding.scope());
        if let Err(denial) = binding.enter_owner_decoder() {
            self.counters
                .cells()
                .observe_failed_recheck_after_owner_entry();
            return Err(denial);
        }
        Ok(result)
    }

    pub(super) fn with_owner_projection<'lease, T, F>(
        &self,
        binding: ResidentIntegrityRecordBinding<'lease>,
        projection: F,
    ) -> Result<T, ResidentIntegrityAdmissionDenial>
    where
        F: FnOnce() -> T,
    {
        self.require_live()?;
        binding
            .require_current_binding()
            .map_err(|denial| self.refuse_before_owner_entry(denial))?;
        self.counters.cells().observe_owner_projection_entry();
        let result = projection();
        if let Err(denial) = binding.require_current_binding() {
            self.counters
                .cells()
                .observe_failed_recheck_after_owner_entry();
            return Err(denial);
        }
        Ok(result)
    }

    fn require_live(&self) -> Result<(), ResidentIntegrityAdmissionDenial> {
        if self.lifecycle.snapshot() == self.snapshot
            && matches!(
                self.snapshot.phase,
                ObservedLifecyclePhase::MediaOwned | ObservedLifecyclePhase::RecordServing
            )
        {
            Ok(())
        } else {
            Err(self.refuse_before_owner_entry(
                ResidentIntegrityAdmissionDenial::LifecycleGenerationChanged,
            ))
        }
    }

    fn refuse_before_owner_entry(
        &self,
        denial: ResidentIntegrityAdmissionDenial,
    ) -> ResidentIntegrityAdmissionDenial {
        self.counters.cells().observe_refusal_before_owner_entry();
        denial
    }
}
