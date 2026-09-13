use std::collections::{HashMap, HashSet};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Arc, Mutex, Weak};
use std::thread::JoinHandle;

use super::PhysicalMutationShutdown;
use crate::physical_runtime::{
    IndeterminatePhysicalMutation, PhysicalMutationHandle, PhysicalMutationIndeterminateStage,
    PhysicalMutationObservationCounters, PhysicalMutationTerminalClass,
    PhysicalMutationTerminalFact, PreparedPhysicalMutation,
};

pub(in crate::physical_runtime) struct PhysicalMutationRuntimeOwner {
    director: Weak<crate::physical_runtime::record_serving::RecordPublicationDirector>,
    state: Mutex<PhysicalMutationLifecycleState>,
    yieldpoint: super::yieldpoint::PhysicalMutationYieldpointOwner,
}

#[derive(Clone)]
pub(in crate::physical_runtime) struct PhysicalMutationStartPort {
    owner: Weak<PhysicalMutationRuntimeOwner>,
}

struct PhysicalMutationLifecycleState {
    accepting: bool,
    attempts: HashMap<
        crate::physical_runtime::PhysicalMutationIdentity,
        Arc<crate::physical_runtime::durability::mutation::PhysicalMutationAttempt>,
    >,
    workers: Vec<JoinHandle<()>>,
    counters: PhysicalMutationObservationCounters,
    completed_unobserved: Vec<crate::physical_runtime::CompletedUnobservedPhysicalMutation>,
    completed_groups: HashSet<crate::physical_runtime::PhysicalDurabilityGroupIdentity>,
    data_writes: u64,
    data_bytes: u64,
    records: u64,
    peak_group_members: u64,
}

impl PhysicalMutationRuntimeOwner {
    pub(in crate::physical_runtime) fn new(
        director: Weak<crate::physical_runtime::record_serving::RecordPublicationDirector>,
    ) -> Arc<Self> {
        Arc::new(Self {
            director,
            state: Mutex::new(PhysicalMutationLifecycleState {
                accepting: true,
                attempts: HashMap::new(),
                workers: Vec::new(),
                counters: PhysicalMutationObservationCounters::default(),
                completed_unobserved: Vec::new(),
                completed_groups: HashSet::new(),
                data_writes: 0,
                data_bytes: 0,
                records: 0,
                peak_group_members: 0,
            }),
            yieldpoint: super::yieldpoint::PhysicalMutationYieldpointOwner::new(),
        })
    }

    pub(in crate::physical_runtime) fn pause_at(
        &self,
        checkpoint: super::PhysicalMutationCheckpoint,
    ) -> super::PhysicalMutationPauseGate {
        self.yieldpoint.install(checkpoint)
    }

    pub(in crate::physical_runtime) fn reach_checkpoint(
        &self,
        checkpoint: super::PhysicalMutationCheckpoint,
    ) {
        self.yieldpoint.pause(checkpoint);
    }

    pub(in crate::physical_runtime) fn start_port(owner: &Arc<Self>) -> PhysicalMutationStartPort {
        PhysicalMutationStartPort {
            owner: Arc::downgrade(owner),
        }
    }

    fn start(self: &Arc<Self>, prepared: PreparedPhysicalMutation) -> PhysicalMutationHandle {
        let identity = prepared.mutation_identity();
        let mut state = self.state();
        if !state.accepting {
            let attempt =
                crate::physical_runtime::durability::mutation::PhysicalMutationAttempt::new(
                    &prepared,
                    Arc::downgrade(self),
                );
            attempt.mark_runtime_closing();
            let handle = PhysicalMutationHandle::new(&attempt);
            drop(state);
            self.settle_without_worker(
                attempt,
                prepared,
                crate::physical_runtime::PhysicalMutationProvenNoEffectCause::WorkerUnavailableBeforeGroupSeal,
            );
            return handle;
        }
        if let Some(existing) = state.attempts.get(&identity) {
            return PhysicalMutationHandle::new(existing);
        }
        let attempt = crate::physical_runtime::durability::mutation::PhysicalMutationAttempt::new(
            &prepared,
            Arc::downgrade(self),
        );
        state.attempts.insert(identity, Arc::clone(&attempt));
        state.counters.record_started();

        let handle = PhysicalMutationHandle::new(&attempt);
        let weak_owner = Arc::downgrade(self);
        let worker_attempt = Arc::clone(&attempt);
        let prepared_slot = Arc::new(Mutex::new(Some(prepared)));
        let worker_prepared = Arc::clone(&prepared_slot);
        let worker = std::thread::Builder::new()
            .name(format!(
                "worth-mutation-{}",
                identity.operation_identity().get()
            ))
            .spawn(move || {
                let prepared = worker_prepared
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .take()
                    .expect("one worker consumes one exact prepared mutation");
                run_worker(weak_owner, worker_attempt, prepared);
            });
        match worker {
            Ok(worker) => state.workers.push(worker),
            Err(_) => {
                let prepared = prepared_slot
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .take()
                    .expect("failed spawn retains the exact prepared mutation");
                drop(state);
                self.settle_without_worker(
                    attempt,
                    prepared,
                    crate::physical_runtime::PhysicalMutationProvenNoEffectCause::WorkerUnavailableBeforeGroupSeal,
                );
            }
        }
        handle
    }

    pub(in crate::physical_runtime) fn stop_and_drain(
        &self,
    ) -> super::PhysicalMutationTerminalState {
        let (attempts, workers) = {
            let mut state = self.state();
            state.accepting = false;
            let attempts = state.attempts.values().cloned().collect::<Vec<_>>();
            let workers = std::mem::take(&mut state.workers);
            (attempts, workers)
        };
        for attempt in attempts {
            attempt.mark_runtime_closing();
        }
        self.yieldpoint
            .pause(super::PhysicalMutationCheckpoint::RuntimeClosingMarked);
        for worker in workers {
            let _ = worker.join();
        }
        let mut state = self.state();
        let shutdown = PhysicalMutationShutdown::from_observation(state.counters.snapshot());
        let completed_unobserved = std::mem::take(&mut state.completed_unobserved);
        let cost = super::PhysicalMutationCostSnapshot {
            groups_formed: state.completed_groups.len() as u64,
            data_writes: state.data_writes,
            data_bytes: state.data_bytes,
            records: state.records,
            acknowledgments: shutdown.completed(),
            peak_group_members: state.peak_group_members,
        };
        super::PhysicalMutationTerminalState::new(shutdown, completed_unobserved, cost)
    }

    pub(in crate::physical_runtime) fn observation(
        &self,
    ) -> crate::physical_runtime::PhysicalMutationObservation {
        self.state().counters.snapshot()
    }

    pub(in crate::physical_runtime) fn record_completed_unobserved(
        &self,
        event: crate::physical_runtime::CompletedUnobservedPhysicalMutation,
    ) {
        let mut state = self.state();
        state.counters.record_completed_unobserved(event);
        state.completed_unobserved.push(event);
    }

    pub(in crate::physical_runtime) fn record_cancellation(
        &self,
        class: crate::physical_runtime::PhysicalMutationCancellationClass,
    ) {
        self.state().counters.record_cancellation(class);
    }

    fn record_terminal(
        &self,
        attempt: &crate::physical_runtime::durability::mutation::PhysicalMutationAttempt,
        terminal: PhysicalMutationTerminalFact,
        panicked: bool,
    ) {
        self.director
            .upgrade()
            .expect("managed mutation retains its publication director through settlement")
            .persist_mutation_terminal(&terminal)
            .expect("carried mutation terminal fact matches its exact idempotency binding");
        let terminal_class = match &terminal {
            PhysicalMutationTerminalFact::Completed(_) => PhysicalMutationTerminalClass::Completed,
            PhysicalMutationTerminalFact::ProvenNoEffect(_) => {
                PhysicalMutationTerminalClass::ProvenNoEffect
            }
            PhysicalMutationTerminalFact::Indeterminate(_) => {
                PhysicalMutationTerminalClass::Indeterminate
            }
        };
        let completed_cost = match &terminal {
            PhysicalMutationTerminalFact::Completed(fact) => Some((
                fact.group_binding().group_identity(),
                u64::from(fact.breadth().data_effect_count()),
                fact.observation().bytes_completed(),
                fact.persisted_records().len() as u64,
                u64::from(fact.group_binding().member_count().get()),
            )),
            PhysicalMutationTerminalFact::ProvenNoEffect(_)
            | PhysicalMutationTerminalFact::Indeterminate(_) => None,
        };
        attempt.install_terminal(terminal);
        let completed_unobserved =
            matches!(terminal_class, PhysicalMutationTerminalClass::Completed)
                .then(|| attempt.record_completed_unobserved_on_completion())
                .flatten();
        let mut state = self.state();
        state.counters.record_terminal(terminal_class, panicked);
        if let Some((group, data_writes, data_bytes, records, group_members)) = completed_cost {
            state.completed_groups.insert(group);
            state.data_writes = state.data_writes.saturating_add(data_writes);
            state.data_bytes = state.data_bytes.saturating_add(data_bytes);
            state.records = state.records.saturating_add(records);
            state.peak_group_members = state.peak_group_members.max(group_members);
        }
        if let Some(event) = completed_unobserved {
            state.counters.record_completed_unobserved(event);
            state.completed_unobserved.push(event);
        }
        drop(state);
        attempt.publish_terminal();
    }

    fn settle_without_worker(
        &self,
        attempt: Arc<crate::physical_runtime::durability::mutation::PhysicalMutationAttempt>,
        prepared: PreparedPhysicalMutation,
        cause: crate::physical_runtime::PhysicalMutationProvenNoEffectCause,
    ) {
        let terminal = self.director.upgrade().map_or_else(
            || {
                PhysicalMutationTerminalFact::Indeterminate(
                    IndeterminatePhysicalMutation::possible_effect(
                        attempt.identity(),
                        attempt.idempotency_identity(),
                        attempt.fingerprint(),
                        PhysicalMutationIndeterminateStage::RuntimeUnavailable,
                        0,
                    ),
                )
            },
            |director| match director.settle_prepared_before_group_seal(prepared, cause) {
                crate::physical_runtime::PhysicalPreSealCancellationOutcome::ProvenNoEffect(
                    terminal,
                ) => PhysicalMutationTerminalFact::ProvenNoEffect(terminal),
                crate::physical_runtime::PhysicalPreSealCancellationOutcome::NotCancelled {
                    ..
                } => PhysicalMutationTerminalFact::Indeterminate(
                    IndeterminatePhysicalMutation::possible_effect(
                        attempt.identity(),
                        attempt.idempotency_identity(),
                        attempt.fingerprint(),
                        PhysicalMutationIndeterminateStage::WalAppend,
                        0,
                    ),
                ),
            },
        );
        self.record_terminal(&attempt, terminal, false);
    }

    fn state(&self) -> std::sync::MutexGuard<'_, PhysicalMutationLifecycleState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl PhysicalMutationStartPort {
    pub(in crate::physical_runtime) fn start(
        &self,
        prepared: PreparedPhysicalMutation,
    ) -> PhysicalMutationHandle {
        let Some(owner) = self.owner.upgrade() else {
            let identity = prepared.mutation_identity();
            let attempt =
                crate::physical_runtime::durability::mutation::PhysicalMutationAttempt::new(
                    &prepared,
                    Weak::new(),
                );
            attempt.mark_stale();
            attempt.install_terminal(PhysicalMutationTerminalFact::Indeterminate(
                IndeterminatePhysicalMutation::possible_effect(
                    identity,
                    prepared.idempotency_identity(),
                    prepared.request_fingerprint(),
                    PhysicalMutationIndeterminateStage::RuntimeUnavailable,
                    0,
                ),
            ));
            attempt.publish_terminal();
            return PhysicalMutationHandle::new(&attempt);
        };
        owner.start(prepared)
    }
}

fn run_worker(
    owner: Weak<PhysicalMutationRuntimeOwner>,
    attempt: Arc<crate::physical_runtime::durability::mutation::PhysicalMutationAttempt>,
    prepared: PreparedPhysicalMutation,
) {
    let execution = catch_unwind(AssertUnwindSafe(|| {
        let Some(owner) = owner.upgrade() else {
            return PhysicalMutationTerminalFact::Indeterminate(
                IndeterminatePhysicalMutation::possible_effect(
                    attempt.identity(),
                    attempt.idempotency_identity(),
                    attempt.fingerprint(),
                    PhysicalMutationIndeterminateStage::RuntimeUnavailable,
                    0,
                ),
            );
        };
        let Some(director) = owner.director.upgrade() else {
            return PhysicalMutationTerminalFact::Indeterminate(
                IndeterminatePhysicalMutation::possible_effect(
                    attempt.identity(),
                    attempt.idempotency_identity(),
                    attempt.fingerprint(),
                    PhysicalMutationIndeterminateStage::RuntimeUnavailable,
                    0,
                ),
            );
        };
        owner
            .yieldpoint
            .pause(super::PhysicalMutationCheckpoint::BeforeEffectCutover);
        director.execute_managed_mutation(prepared, &attempt)
    }));
    let Some(owner) = owner.upgrade() else {
        attempt.mark_stale();
        return;
    };
    let (terminal, panicked) = match execution {
        Ok(terminal) => (terminal, false),
        Err(_) => (
            PhysicalMutationTerminalFact::Indeterminate(
                IndeterminatePhysicalMutation::possible_effect(
                    attempt.identity(),
                    attempt.idempotency_identity(),
                    attempt.fingerprint(),
                    PhysicalMutationIndeterminateStage::WorkerPanicked,
                    0,
                ),
            ),
            true,
        ),
    };
    owner
        .yieldpoint
        .pause(super::PhysicalMutationCheckpoint::BeforeTerminalFinalization);
    owner.record_terminal(&attempt, terminal, panicked);
}
