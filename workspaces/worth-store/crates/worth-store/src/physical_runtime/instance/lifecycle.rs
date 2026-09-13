use std::marker::PhantomData;
use std::sync::Arc;

mod closeout_performance;

use closeout_performance::{closeout_performance_summary, PhysicalCloseoutPerformanceObservation};

use crate::physical_runtime::{
    lifecycle::LifecycleTerminationGuard,
    record_serving::{
        RecordPublicationResidueObservation, RecordServingOwner, RecordServingTerminalObservation,
        ServingHealth, ServingShutdownOutcome,
    },
    runtime::PhysicalRuntimeCore,
    work::{PhysicalWorkSafeCancellation, PhysicalWorkShutdownObservation, PhysicalWorkStopKind},
    AbortedRuntime, ClosedRuntime, MediaShutdownOutcome, PhysicalSignalShutdownOutcome,
};

use super::{
    PhysicalResidencyOwner, PhysicalStoreCloseProgressOwner, PhysicalStoreInstanceParts,
    PhysicalStoreWorkRuntime, PhysicalWorkExecutor, PhysicalWorkSignalOwner,
};

struct CheckpointDrained;
struct AdmissionStopped;
struct SafeCancellationComplete;
struct DispatchSettlementComplete;
struct SignalDisposed;
struct ResidencyClosed;

struct ShutdownProtocol<State, Terminate> {
    termination: LifecycleTerminationGuard,
    signal_owner: Option<PhysicalWorkSignalOwner>,
    executor: Option<PhysicalWorkExecutor>,
    core: PhysicalRuntimeCore,
    record_owner: RecordServingOwner,
    publication_residue: RecordPublicationResidueObservation,
    mutation: crate::physical_runtime::PhysicalMutationShutdown,
    mutation_cost: crate::physical_runtime::PhysicalMutationCostSnapshot,
    checkpoint: crate::physical_runtime::PhysicalCheckpointShutdown,
    completed_unobserved:
        Option<Box<[crate::physical_runtime::CompletedUnobservedPhysicalMutation]>>,
    latest_checkpoint: Option<crate::physical_runtime::CompletedPhysicalCheckpoint>,
    recovery_roots: Option<crate::physical_runtime::PhysicalRecoveryRootBasis>,
    recovery_wal_tail: Option<crate::physical_runtime::PhysicalRecoveryWalTail>,
    recovery_allocation: Option<crate::physical_runtime::PhysicalRecoveryAllocationAdmission>,
    wal_observation: crate::physical_runtime::PhysicalWalObservation,
    performance_witness: worth_store_aspect_native::StorePhysicalBoundaryWitness,
    health: Option<ServingHealth>,
    residency_owner: Option<PhysicalResidencyOwner>,
    durability_owner: crate::physical_runtime::durability::ReopenedPhysicalDurabilityRuntimeOwner,
    work_runtime: Option<Arc<PhysicalStoreWorkRuntime>>,
    work_cancellation: Option<PhysicalWorkSafeCancellation>,
    work: Option<PhysicalWorkShutdownObservation>,
    stop: PhysicalWorkStopKind,
    signal_cancellation_failures: u64,
    signal_summary: Option<worth_signal::facade::ResourceRuntimeSummary>,
    signal: Option<PhysicalSignalShutdownOutcome>,
    residency: Option<worth_store_buffer_pool::PhysicalResidencyShutdown>,
    terminate_core: Option<Terminate>,
    progress: PhysicalStoreCloseProgressOwner,
    state: PhantomData<State>,
}

impl PhysicalStoreInstanceParts {
    pub(in crate::physical_runtime) fn close(
        self,
        progress: PhysicalStoreCloseProgressOwner,
    ) -> ServingShutdownOutcome<ClosedRuntime> {
        self.shutdown(PhysicalWorkStopKind::Close, |core| core.close(), progress)
    }

    pub(in crate::physical_runtime) fn abort(
        self,
        progress: PhysicalStoreCloseProgressOwner,
    ) -> ServingShutdownOutcome<AbortedRuntime> {
        self.shutdown(PhysicalWorkStopKind::Abort, |core| core.abort(), progress)
    }

    fn shutdown<Terminal, Terminate>(
        self,
        stop: PhysicalWorkStopKind,
        terminate_core: Terminate,
        progress: PhysicalStoreCloseProgressOwner,
    ) -> ServingShutdownOutcome<Terminal>
    where
        Terminate: FnOnce(PhysicalRuntimeCore) -> Terminal,
    {
        ShutdownProtocol::drain_checkpoints(self, stop, terminate_core, progress)
            .stop_admission()
            .cancel_safe_work()
            .classify_dispatch_settlement()
            .dispose_signal()
            .close_residency()
            .release_media()
    }
}

impl<Terminate> ShutdownProtocol<CheckpointDrained, Terminate> {
    fn drain_checkpoints(
        parts: PhysicalStoreInstanceParts,
        stop: PhysicalWorkStopKind,
        terminate_core: Terminate,
        progress: PhysicalStoreCloseProgressOwner,
    ) -> Self {
        let PhysicalStoreInstanceParts {
            termination,
            work_admission: _work_admission,
            work_runtime,
            scheduler_admission: _scheduler_admission,
            record_work: _record_work,
            core,
            record_owner,
            format: _format,
            access: _access,
            publication,
            checkpoint,
            root_protocol_counters: _root_protocol_counters,
            residency,
            durability,
        } = parts;
        let (checkpoint, latest_checkpoint) = checkpoint.stop_and_drain().into_parts();
        let publication =
            crate::physical_runtime::record_serving::RecordPublicationDirector::stop_and_extract(
                publication,
            );
        let publication_residue = publication.residue;
        let (mutation, completed_unobserved, mutation_cost) = publication.mutations.into_parts();
        let recovery_allocation = residency.recovery_allocation_admission();
        let protocol = Self {
            termination,
            signal_owner: None,
            executor: None,
            core,
            record_owner,
            publication_residue,
            mutation,
            mutation_cost,
            checkpoint,
            completed_unobserved: Some(completed_unobserved),
            latest_checkpoint,
            recovery_roots: Some(publication.roots),
            recovery_wal_tail: Some(publication.wal_tail),
            recovery_allocation: Some(recovery_allocation),
            wal_observation: publication.wal_observation,
            performance_witness: publication.performance_witness,
            health: None,
            residency_owner: Some(residency),
            durability_owner: durability,
            work_runtime: Some(work_runtime),
            work_cancellation: None,
            work: None,
            stop,
            signal_cancellation_failures: 0,
            signal_summary: None,
            signal: None,
            residency: None,
            terminate_core: Some(terminate_core),
            progress,
            state: PhantomData,
        };
        protocol
            .progress
            .record(super::PhysicalStoreClosePhase::CheckpointDrained);
        protocol
    }

    fn stop_admission(self) -> ShutdownProtocol<AdmissionStopped, Terminate> {
        self.work_runtime
            .as_ref()
            .expect("checkpoint-drained phase owns work runtime")
            .stop_execution_admission();
        self.progress
            .record(super::PhysicalStoreClosePhase::AdmissionStopped);
        self.transition()
    }
}

impl<Terminate> ShutdownProtocol<AdmissionStopped, Terminate> {
    fn cancel_safe_work(mut self) -> ShutdownProtocol<SafeCancellationComplete, Terminate> {
        let work_runtime = self.work_runtime.as_ref().expect("phase owns work runtime");
        work_runtime.await_execution_calls();
        let cancellation = work_runtime.submission.cancel_safe_work(self.stop);
        let signal_owner = &work_runtime.signal;
        for consumer in cancellation.cancellation_candidates().iter().copied() {
            if signal_owner.cancel(consumer).is_err() {
                self.signal_cancellation_failures =
                    self.signal_cancellation_failures.saturating_add(1);
            }
        }
        self.work_cancellation = Some(cancellation);
        self.progress
            .record(super::PhysicalStoreClosePhase::SafeCancellationComplete);
        self.transition()
    }
}

impl<Terminate> ShutdownProtocol<SafeCancellationComplete, Terminate> {
    fn classify_dispatch_settlement(
        mut self,
    ) -> ShutdownProtocol<DispatchSettlementComplete, Terminate> {
        let work_runtime = self.work_runtime.take().expect("phase owns work runtime");
        work_runtime.reconcile_signal_derivation();
        self.work = Some(
            work_runtime.submission.settle_dispatches(
                self.work_cancellation
                    .take()
                    .expect("safe cancellation phase completed"),
            ),
        );
        let runtime = Arc::try_unwrap(work_runtime)
            .unwrap_or_else(|_| unreachable!("execution gate excludes remaining strong owners"));
        let PhysicalStoreWorkRuntime {
            submission,
            signal,
            executor,
            health,
            recovery: _recovery,
            ..
        } = runtime;
        self.signal_summary = signal.runtime_summary().ok();
        drop(submission);
        self.signal_owner = Some(signal);
        self.executor = Some(executor);
        self.health = Some(health);
        self.progress
            .record(super::PhysicalStoreClosePhase::DispatchSettlementComplete);
        self.transition()
    }
}

impl<Terminate> ShutdownProtocol<DispatchSettlementComplete, Terminate> {
    fn dispose_signal(mut self) -> ShutdownProtocol<SignalDisposed, Terminate> {
        self.signal = Some(
            self.signal_owner
                .take()
                .expect("phase owns Signal")
                .dispose(),
        );
        self.progress
            .record(super::PhysicalStoreClosePhase::SignalDisposed);
        self.transition()
    }
}

impl<Terminate> ShutdownProtocol<SignalDisposed, Terminate> {
    fn close_residency(mut self) -> ShutdownProtocol<ResidencyClosed, Terminate> {
        self.residency = Some(
            self.residency_owner
                .take()
                .expect("phase owns residency")
                .close(),
        );
        self.progress
            .record(super::PhysicalStoreClosePhase::ResidencyClosed);
        self.transition()
    }
}

impl<Terminate> ShutdownProtocol<ResidencyClosed, Terminate> {
    fn release_media<Terminal>(mut self) -> ServingShutdownOutcome<Terminal>
    where
        Terminate: FnOnce(PhysicalRuntimeCore) -> Terminal,
    {
        let durability_closeout = if matches!(self.stop, PhysicalWorkStopKind::Close) {
            match self.durability_owner.into_recovery_basis(
                self.completed_unobserved
                    .take()
                    .expect("mutation drain retained completed-unobserved facts"),
            ) {
                Ok((policy, operations)) => {
                    crate::physical_runtime::PhysicalDurabilityCloseoutOutcome::RecoveryHandoff(
                        crate::physical_runtime::PhysicalDurabilityRecoveryHandoff::finalize(
                            policy,
                            self.recovery_roots
                                .take()
                                .expect("publication drain retained root lineage"),
                            crate::physical_runtime::PhysicalRecoveryCheckpointBasis::from_latest(
                                self.latest_checkpoint.take(),
                            ),
                            self.recovery_wal_tail
                                .take()
                                .expect("publication drain retained WAL inventory"),
                            operations,
                            self.recovery_allocation
                                .take()
                                .expect("residency owner supplied recovery admission"),
                            crate::physical_runtime::PhysicalArtifactResidueClassification::new(
                                self.publication_residue,
                            ),
                        ),
                    )
                }
                Err(denial) => {
                    crate::physical_runtime::PhysicalDurabilityCloseoutOutcome::InspectionRequired(
                        denial,
                    )
                }
            }
        } else {
            drop(self.durability_owner);
            crate::physical_runtime::PhysicalDurabilityCloseoutOutcome::NotProducedForAbort
        };
        drop(self.termination);
        let residency = self.residency.expect("residency phase completed");
        let record_counters = self.record_owner.into_terminal_snapshot();
        let records = RecordServingTerminalObservation::new(
            self.health
                .as_ref()
                .expect("phase owns serving health")
                .requires_inspection()
                || !self.publication_residue.is_empty()
                || self.mutation.requires_inspection()
                || residency.requires_inspection(),
            self.publication_residue,
            record_counters,
        );
        let performance = closeout_performance_summary(PhysicalCloseoutPerformanceObservation {
            witness: self.performance_witness,
            mutation: self.mutation,
            mutation_cost: self.mutation_cost,
            checkpoint: self.checkpoint,
            wal: self.wal_observation,
            records: record_counters,
            residency,
            work: self
                .work
                .as_ref()
                .expect("dispatch settlement phase completed"),
            residue: self.publication_residue,
            closeout: &durability_closeout,
        });
        let media_release = self
            .executor
            .expect("phase owns physical executor")
            .into_media()
            .close();
        let terminal = self.terminate_core.expect("terminal action retained")(self.core);
        let media = MediaShutdownOutcome::new(terminal, media_release);
        self.progress
            .record(super::PhysicalStoreClosePhase::MediaReleased);
        ServingShutdownOutcome {
            media,
            records,
            mutation: self.mutation,
            checkpoint: self.checkpoint,
            residency,
            work: self.work.expect("dispatch settlement phase completed"),
            signal: self.signal.expect("Signal phase completed"),
            signal_summary: self.signal_summary,
            signal_cancellation_failures: self.signal_cancellation_failures,
            durability_closeout,
            performance,
        }
    }
}

impl<State, Terminate> ShutdownProtocol<State, Terminate> {
    fn transition<Next>(self) -> ShutdownProtocol<Next, Terminate> {
        ShutdownProtocol {
            termination: self.termination,
            signal_owner: self.signal_owner,
            executor: self.executor,
            core: self.core,
            record_owner: self.record_owner,
            publication_residue: self.publication_residue,
            mutation: self.mutation,
            mutation_cost: self.mutation_cost,
            checkpoint: self.checkpoint,
            completed_unobserved: self.completed_unobserved,
            latest_checkpoint: self.latest_checkpoint,
            recovery_roots: self.recovery_roots,
            recovery_wal_tail: self.recovery_wal_tail,
            recovery_allocation: self.recovery_allocation,
            wal_observation: self.wal_observation,
            performance_witness: self.performance_witness,
            health: self.health,
            residency_owner: self.residency_owner,
            durability_owner: self.durability_owner,
            work_runtime: self.work_runtime,
            work_cancellation: self.work_cancellation,
            work: self.work,
            stop: self.stop,
            signal_cancellation_failures: self.signal_cancellation_failures,
            signal_summary: self.signal_summary,
            signal: self.signal,
            residency: self.residency,
            terminate_core: self.terminate_core,
            progress: self.progress,
            state: PhantomData,
        }
    }
}
