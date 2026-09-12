use std::sync::{Arc, Condvar, Mutex, Weak};

use crate::physical_runtime::{
    record_serving::ServingHealth, LifecycleGeneration, PhysicalExecutorCommand,
    PhysicalWorkAspectDelta, PhysicalWorkBatchDenial, PhysicalWorkExecutionBatchOutcome,
    PhysicalWorkExecutionOutcome, PhysicalWorkPreEffectDenial,
};

use super::{PhysicalWorkExecutor, PhysicalWorkSignalOwner};
use crate::physical_runtime::work::PhysicalWorkSubmissionOwner;

pub(in crate::physical_runtime) struct PhysicalStoreWorkRuntime {
    pub(in crate::physical_runtime) submission: PhysicalWorkSubmissionOwner,
    pub(in crate::physical_runtime) signal: PhysicalWorkSignalOwner,
    pub(in crate::physical_runtime) executor: PhysicalWorkExecutor,
    pub(in crate::physical_runtime) health: ServingHealth,
    pub(in crate::physical_runtime) recovery:
        crate::physical_runtime::work::PhysicalEffectRecoveryInventory,
    gate: Arc<PhysicalExecutionGate>,
}

#[derive(Clone)]
pub struct PhysicalWorkExecution {
    runtime: Weak<PhysicalStoreWorkRuntime>,
    gate: Arc<PhysicalExecutionGate>,
    generation: LifecycleGeneration,
}

#[derive(Debug)]
pub(in crate::physical_runtime) struct PhysicalProjectionFailureCapability {
    runtime: Weak<PhysicalStoreWorkRuntime>,
    delta: PhysicalWorkAspectDelta,
}

struct PhysicalExecutionGate {
    state: Mutex<PhysicalExecutionGateState>,
    changed: Condvar,
}

struct PhysicalExecutionGateState {
    accepting: bool,
    active: usize,
    #[cfg(feature = "certification-test-authority")]
    drain_waiting: bool,
}

pub(in crate::physical_runtime) struct PhysicalExecutionCall {
    gate: Arc<PhysicalExecutionGate>,
}

impl PhysicalStoreWorkRuntime {
    pub(super) fn new(
        submission: PhysicalWorkSubmissionOwner,
        signal: PhysicalWorkSignalOwner,
        executor: PhysicalWorkExecutor,
        health: ServingHealth,
        recovery: crate::physical_runtime::work::PhysicalEffectRecoveryInventory,
    ) -> Arc<Self> {
        Arc::new(Self {
            submission,
            signal,
            executor,
            health,
            recovery,
            gate: Arc::new(PhysicalExecutionGate {
                state: Mutex::new(PhysicalExecutionGateState {
                    accepting: true,
                    active: 0,
                    #[cfg(feature = "certification-test-authority")]
                    drain_waiting: false,
                }),
                changed: Condvar::new(),
            }),
        })
    }

    pub(in crate::physical_runtime) fn execution(
        runtime: &Arc<Self>,
        generation: LifecycleGeneration,
    ) -> PhysicalWorkExecution {
        PhysicalWorkExecution {
            runtime: Arc::downgrade(runtime),
            gate: Arc::clone(&runtime.gate),
            generation,
        }
    }

    pub(in crate::physical_runtime) fn stop_execution_admission(&self) {
        let mut state = self
            .gate
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.accepting = false;
        self.submission.stop_admission();
    }

    pub(in crate::physical_runtime) fn await_execution_calls(&self) {
        let mut state = self
            .gate
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        while state.active != 0 {
            #[cfg(feature = "certification-test-authority")]
            {
                state.drain_waiting = true;
            }
            state = self
                .gate
                .changed
                .wait(state)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
        #[cfg(feature = "certification-test-authority")]
        {
            state.drain_waiting = false;
        }
    }

    pub(in crate::physical_runtime) fn reconcile_signal_derivation(&self) {
        for (identity, outcome) in self.signal.reconcile_settlements() {
            self.submission
                .record_reconciled_derived_completion(identity, outcome);
        }
    }
}

impl PhysicalWorkExecution {
    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime) fn read_call_drain_is_waiting(&self) -> bool {
        self.gate
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .drain_waiting
    }
    pub(in crate::physical_runtime) fn bind_projection_failure(
        &self,
        delta: PhysicalWorkAspectDelta,
    ) -> PhysicalProjectionFailureCapability {
        PhysicalProjectionFailureCapability {
            runtime: self.runtime.clone(),
            delta,
        }
    }

    pub fn execute_physical_work(
        &self,
        command: PhysicalExecutorCommand,
    ) -> Result<PhysicalWorkExecutionOutcome, PhysicalWorkPreEffectDenial> {
        let call = self.admit_call()?;
        let runtime = self
            .runtime
            .upgrade()
            .ok_or(PhysicalWorkPreEffectDenial::AdmissionStopped)?;
        if command.intent().identity().generation().lifecycle() != self.generation {
            drop(runtime);
            drop(call);
            return Err(PhysicalWorkPreEffectDenial::StaleGeneration);
        }
        let outcome = runtime.execute_physical_work(command);
        drop(runtime);
        drop(call);
        outcome
    }

    pub fn execute_physical_work_batch(
        &self,
        commands: Box<[PhysicalExecutorCommand]>,
    ) -> PhysicalWorkExecutionBatchOutcome {
        let call = match self.admit_call() {
            Ok(call) => call,
            Err(denial) => return deny_batch_before_effect(commands, denial),
        };
        let Some(runtime) = self.runtime.upgrade() else {
            drop(call);
            return deny_batch_before_effect(
                commands,
                PhysicalWorkPreEffectDenial::AdmissionStopped,
            );
        };
        let outcome = runtime.execute_physical_work_batch(commands, self.generation);
        drop(runtime);
        drop(call);
        outcome
    }

    pub(in crate::physical_runtime) fn admit_call(
        &self,
    ) -> Result<PhysicalExecutionCall, PhysicalWorkPreEffectDenial> {
        let mut state = self
            .gate
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if !state.accepting {
            return Err(PhysicalWorkPreEffectDenial::AdmissionStopped);
        }
        // Stop/drain uses this same mutex. Never create an uncounted strong
        // runtime reference while shutdown may consume the final owner.
        let runtime = self
            .runtime
            .upgrade()
            .ok_or(PhysicalWorkPreEffectDenial::AdmissionStopped)?;
        if runtime.submission.generation() != self.generation {
            return Err(PhysicalWorkPreEffectDenial::StaleGeneration);
        }
        state.active = state.active.saturating_add(1);
        drop(runtime);
        Ok(PhysicalExecutionCall {
            gate: Arc::clone(&self.gate),
        })
    }
}

impl PhysicalProjectionFailureCapability {
    pub(in crate::physical_runtime) fn consume(self) {
        let Some(runtime) = self.runtime.upgrade() else {
            return;
        };
        if runtime.signal.apply_delta(self.delta).is_err() {
            runtime.signal.revoke_derived_admission();
        }
        runtime.health.revoke();
    }
}

impl Drop for PhysicalExecutionCall {
    fn drop(&mut self) {
        let mut state = self
            .gate
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.active = state.active.saturating_sub(1);
        self.gate.changed.notify_all();
    }
}

fn deny_batch_before_effect(
    commands: Box<[PhysicalExecutorCommand]>,
    denial: PhysicalWorkPreEffectDenial,
) -> PhysicalWorkExecutionBatchOutcome {
    PhysicalWorkExecutionBatchOutcome::new(
        Vec::new(),
        commands
            .into_vec()
            .into_iter()
            .map(|command| PhysicalWorkBatchDenial::new(command.intent().identity(), denial))
            .collect(),
    )
}
