use std::sync::{
    atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
    Arc, Condvar, Mutex,
};

use worth_store_physical_format::store_namespace::StableStoreIdentity;

use crate::physical_runtime::{
    instance::PhysicalSignalAdmissionStatus, lifecycle::LifecycleState, LifecycleGeneration,
    RuntimeIdentity,
};

use super::{
    command_storage::PhysicalCommandArena,
    observation::{PhysicalWorkAccounting, PhysicalWorkObservationOwner},
    PhysicalSignalAspectBindingSet, PhysicalSignalProfileIdentity, PhysicalWorkCapacity,
    PhysicalWorkDeclarationDenial, PhysicalWorkIdentity, PhysicalWorkIntent,
    PhysicalWorkObservation,
};

mod abandonment;
mod capacity_lease;
mod effect_activity;
mod in_flight_activity;
mod mutation_identity;
mod outcome;
mod request;
mod reservation;
mod shutdown;
#[cfg(feature = "certification-test-authority")]
mod yieldpoint;

pub(in crate::physical_runtime) use abandonment::{
    physical_work_abandonment_channel, PhysicalWorkAbandonmentInbox,
    PhysicalWorkAbandonmentPublisher, PhysicalWorkAbandonmentWake,
};
pub(super) use capacity_lease::PhysicalWorkCapacityLease;
pub(in crate::physical_runtime) use effect_activity::PhysicalEffectActivity;
pub(in crate::physical_runtime) use mutation_identity::PhysicalMutationIdentityReservationError;
pub use outcome::{
    PhysicalWorkCapacityDimension, PhysicalWorkSubmissionDeferred, PhysicalWorkSubmissionDenial,
    PhysicalWorkSubmissionFailure, PhysicalWorkSubmissionOutcome, PhysicalWorkSubmissionReceipt,
    PhysicalWorkSubmissionStale,
};
pub use request::{PhysicalMutationSubmission, PhysicalReadSubmission};
pub(in crate::physical_runtime) use shutdown::PhysicalWorkSafeCancellation;

pub(in crate::physical_runtime) struct PhysicalWorkSubmissionOwner {
    shared: Arc<PhysicalSubmissionState>,
    observation: PhysicalWorkObservationOwner,
}

pub(in crate::physical_runtime) struct PhysicalWorkSubmissionFoundation {
    pub(in crate::physical_runtime) store: StableStoreIdentity,
    pub(in crate::physical_runtime) runtime: RuntimeIdentity,
    pub(in crate::physical_runtime) generation: LifecycleGeneration,
    pub(in crate::physical_runtime) lifecycle: Arc<LifecycleState>,
    pub(in crate::physical_runtime) lifecycle_phase:
        crate::physical_runtime::lifecycle::ObservedLifecyclePhase,
    pub(in crate::physical_runtime) signal_profile: PhysicalSignalProfileIdentity,
    pub(in crate::physical_runtime) bindings: Arc<PhysicalSignalAspectBindingSet>,
    pub(in crate::physical_runtime) signal_admission: PhysicalSignalAdmissionStatus,
    pub(in crate::physical_runtime) abandonment: PhysicalWorkAbandonmentPublisher,
}

#[derive(Clone, Copy)]
pub(in crate::physical_runtime) enum PhysicalWorkStopKind {
    Close,
    Abort,
    Drop,
}

pub(super) struct PhysicalSubmissionState {
    accepting: AtomicBool,
    terminal_published: AtomicBool,
    store: StableStoreIdentity,
    runtime: RuntimeIdentity,
    generation: LifecycleGeneration,
    lifecycle: Arc<LifecycleState>,
    lifecycle_phase: crate::physical_runtime::lifecycle::ObservedLifecyclePhase,
    signal_admission: PhysicalSignalAdmissionStatus,
    signal_profile: PhysicalSignalProfileIdentity,
    bindings: Arc<PhysicalSignalAspectBindingSet>,
    capacity: PhysicalWorkCapacity,
    next_operation: AtomicU64,
    active_submissions: AtomicUsize,
    active_effects: AtomicUsize,
    active_wait: Mutex<()>,
    active_changed: Condvar,
    reserved_commands: AtomicUsize,
    reserved_scope_members: AtomicUsize,
    reserved_semantic_bytes: AtomicUsize,
    commands: PhysicalCommandArena,
    accounting: PhysicalWorkAccounting,
    terminal_ledger: super::PhysicalWorkTerminalLedger,
    abandonment: PhysicalWorkAbandonmentPublisher,
}

impl PhysicalWorkSubmissionOwner {
    pub(in crate::physical_runtime) fn runtime_identity(&self) -> RuntimeIdentity {
        self.shared.runtime
    }

    pub(in crate::physical_runtime) fn new(foundation: PhysicalWorkSubmissionFoundation) -> Self {
        let capacity = foundation.bindings.capacity();
        Self {
            shared: Arc::new(PhysicalSubmissionState {
                accepting: AtomicBool::new(true),
                terminal_published: AtomicBool::new(false),
                store: foundation.store,
                runtime: foundation.runtime,
                generation: foundation.generation,
                lifecycle: foundation.lifecycle,
                lifecycle_phase: foundation.lifecycle_phase,
                signal_admission: foundation.signal_admission,
                signal_profile: foundation.signal_profile,
                bindings: foundation.bindings,
                capacity,
                next_operation: AtomicU64::new(1),
                active_submissions: AtomicUsize::new(0),
                active_effects: AtomicUsize::new(0),
                active_wait: Mutex::new(()),
                active_changed: Condvar::new(),
                reserved_commands: AtomicUsize::new(0),
                reserved_scope_members: AtomicUsize::new(0),
                reserved_semantic_bytes: AtomicUsize::new(0),
                commands: PhysicalCommandArena::bounded(capacity.commands()),
                accounting: PhysicalWorkAccounting::new(),
                terminal_ledger: super::PhysicalWorkTerminalLedger::bounded(
                    capacity.terminal_evidence(),
                ),
                abandonment: foundation.abandonment,
            }),
            observation: PhysicalWorkObservationOwner::new(capacity.terminal_evidence()),
        }
    }

    pub(in crate::physical_runtime) fn read_submission(&self) -> PhysicalReadSubmission {
        PhysicalReadSubmission {
            shared: Arc::downgrade(&self.shared),
            generation: self.shared.generation,
        }
    }

    pub(in crate::physical_runtime) fn mutation_submission(&self) -> PhysicalMutationSubmission {
        PhysicalMutationSubmission {
            shared: Arc::downgrade(&self.shared),
            generation: self.shared.generation,
        }
    }

    pub(in crate::physical_runtime) fn generation(&self) -> LifecycleGeneration {
        self.shared.generation
    }

    pub(in crate::physical_runtime) fn observation(&self) -> PhysicalWorkObservation {
        self.observation.handle()
    }

    pub(in crate::physical_runtime) fn counters(&self) -> super::PhysicalWorkCounterSnapshot {
        self.shared
            .commands
            .active_counters(self.shared.accounting.terminal_by_family_and_pressure())
    }

    pub(in crate::physical_runtime) fn cancel_before_dispatch(
        &self,
        identity: PhysicalWorkIdentity,
    ) -> bool {
        let Some(released) = self.shared.commands.cancel_before_dispatch(identity) else {
            return false;
        };
        self.shared
            .release_capacity(released.scope_members, released.semantic_bytes);
        self.shared
            .accounting
            .record_terminal(released.operation, released.pressure);
        self.shared.terminal_ledger.record(
            super::PhysicalWorkTerminalEvent::CancelledBeforeDispatch(identity),
        );
        true
    }

    pub(in crate::physical_runtime) fn mark_consumer_cancelled(
        &self,
        identity: PhysicalWorkIdentity,
    ) -> bool {
        self.shared.commands.mark_consumer_cancelled(identity)
    }

    pub(in crate::physical_runtime) fn record_derived_reconciliation_deferred(
        &self,
        identity: PhysicalWorkIdentity,
    ) {
        self.shared
            .terminal_ledger
            .record(super::PhysicalWorkTerminalEvent::DerivedReconciliationDeferred(identity));
    }

    pub(in crate::physical_runtime) fn record_settled_causality(
        &self,
        settled: &super::SettledPhysicalWork,
    ) {
        self.observation
            .causal()
            .record_settlement(settled, self.counters());
    }

    pub(in crate::physical_runtime) fn record_derived_completion_causality(
        &self,
        identity: PhysicalWorkIdentity,
        outcome: crate::physical_runtime::PhysicalSignalSettlementOutcome,
    ) {
        self.observation
            .causal()
            .record_derived_completion(identity, outcome);
    }

    pub(in crate::physical_runtime) fn record_reconciled_derived_completion(
        &self,
        identity: PhysicalWorkIdentity,
        outcome: crate::physical_runtime::PhysicalSignalSettlementOutcome,
    ) {
        self.record_derived_completion_causality(identity, outcome);
        self.shared
            .terminal_ledger
            .resolve_derived_reconciliation(identity);
    }

    pub(super) fn state(&self) -> &Arc<PhysicalSubmissionState> {
        &self.shared
    }

    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime) fn pause_after_command_shard_lock_for_certification(
        &self,
    ) -> yieldpoint::CertificationPhysicalSubmissionPauseGate {
        self.shared
            .commands
            .pause_after_shard_lock_for_certification()
    }
}

impl PhysicalSubmissionState {
    pub(super) fn accepts_work(&self) -> bool {
        self.accepting.load(Ordering::Acquire)
    }

    pub(super) const fn store(&self) -> StableStoreIdentity {
        self.store
    }

    pub(super) const fn runtime(&self) -> RuntimeIdentity {
        self.runtime
    }

    pub(super) const fn generation(&self) -> LifecycleGeneration {
        self.generation
    }

    pub(super) fn lifecycle_snapshot(
        &self,
    ) -> crate::physical_runtime::lifecycle::LifecycleStateSnapshot {
        self.lifecycle.snapshot()
    }

    pub(super) const fn lifecycle_phase(
        &self,
    ) -> crate::physical_runtime::lifecycle::ObservedLifecyclePhase {
        self.lifecycle_phase
    }

    pub(super) const fn signal_profile(&self) -> PhysicalSignalProfileIdentity {
        self.signal_profile
    }

    pub(super) fn signal_available(&self) -> bool {
        self.signal_admission.is_available()
    }

    pub(super) fn bindings(&self) -> &PhysicalSignalAspectBindingSet {
        &self.bindings
    }

    pub(super) fn admit_declared(
        self: &Arc<Self>,
        identity: PhysicalWorkIdentity,
    ) -> Option<(PhysicalWorkIntent, PhysicalWorkCapacityLease)> {
        let admitted = self.commands.admit_declared(identity)?;
        let lease = PhysicalWorkCapacityLease::new(self, identity, admitted.release);
        Some((admitted.intent, lease))
    }
}

impl Drop for PhysicalWorkSubmissionOwner {
    fn drop(&mut self) {
        if !self.shared.terminal_published.load(Ordering::Acquire) {
            let _ = self.stop(PhysicalWorkStopKind::Drop);
        }
    }
}

#[cfg(feature = "certification-test-authority")]
pub use yieldpoint::CertificationPhysicalSubmissionPauseGate;
