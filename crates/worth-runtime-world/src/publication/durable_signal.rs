//! The same Signal task survives a stopped World request; no effect is replayed.
use std::sync::{Arc, Mutex};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
use worth_foundational::facade::DurablePublicationFate;
use worth_signal::facade::durability::{
    SignalDurableAdvanceCompletion, SignalDurableAdvanceDenial, SignalDurableAdvanceOutput,
    SignalDurableAdvanceProgress, SignalDurableAdvanceTask,
};

pub(crate) struct WorldDurableSignalResult {
    pub(crate) output: SignalDurableAdvanceOutput,
    _funding: Option<ExecutionMemoryReservation>,
}
impl std::fmt::Debug for WorldDurableSignalResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorldDurableSignalResult")
            .field("source", self.output.branch().source_point())
            .finish()
    }
}
enum State {
    Pending {
        task: SignalDurableAdvanceTask,
        denial: Option<SignalDurableAdvanceDenial>,
    },
    OwnerRetained,
    Committed(Arc<WorldDurableSignalResult>),
    Terminal(SignalDurableAdvanceCompletion),
}
pub(crate) struct WorldDurableSignalAdvance {
    state: Mutex<Option<State>>,
    original: worth_foundational::facade::SignalBranchAdvanceIdentity,
    result_funding: Mutex<Option<ExecutionMemoryReservation>>,
    _funding: Option<ExecutionMemoryReservation>,
}
impl std::fmt::Debug for WorldDurableSignalAdvance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug = f.debug_struct("WorldDurableSignalAdvance");
        debug.field("original", &self.original);
        if let Ok(state) = self.state.try_lock() {
            if let Some(State::Pending { denial, .. }) = state.as_ref() {
                debug.field("last_denial", denial);
            }
        }
        debug.finish_non_exhaustive()
    }
}
pub(crate) enum WorldDurableSignalProgress {
    Pending,
    Committed(Arc<WorldDurableSignalResult>),
    ProvenNoEffect,
    Indeterminate,
}
pub(crate) struct WorldDurableSignalBacking {
    task: Option<ExecutionMemoryReservation>,
    result: Option<ExecutionMemoryReservation>,
}
impl WorldDurableSignalBacking {
    pub(crate) fn reserve(
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, worth_execution::WorkCeilingDenial> {
        fn arc_size<T>() -> u64 {
            std::alloc::Layout::new::<[std::sync::atomic::AtomicUsize; 2]>()
                .extend(std::alloc::Layout::new::<T>())
                .expect("fixed task layout")
                .0
                .pad_to_align()
                .size() as u64
        }
        Ok(Self {
            task: request.reserve_retained_memory(arc_size::<WorldDurableSignalAdvance>())?,
            result: request.reserve_retained_memory(arc_size::<WorldDurableSignalResult>())?,
        })
    }
    pub(crate) fn retain(self, task: SignalDurableAdvanceTask) -> Arc<WorldDurableSignalAdvance> {
        Arc::new(WorldDurableSignalAdvance {
            original: task.original(),
            state: Mutex::new(Some(State::Pending { task, denial: None })),
            result_funding: Mutex::new(self.result),
            _funding: self.task,
        })
    }
}
impl WorldDurableSignalAdvance {
    pub(crate) fn original(&self) -> worth_foundational::facade::SignalBranchAdvanceIdentity {
        self.original
    }
    pub(crate) fn drive(&self, request: &ExecutionRequest<'_, '_>) -> WorldDurableSignalProgress {
        let Ok(mut state) = self.state.try_lock() else {
            return WorldDurableSignalProgress::Pending;
        };
        loop {
            match state
                .take()
                .expect("Signal original retained between drives")
            {
                State::Committed(output) => {
                    *state = Some(State::Committed(Arc::clone(&output)));
                    return WorldDurableSignalProgress::Committed(output);
                }
                State::Terminal(completion) => {
                    let result = match completion.fate() {
                        DurablePublicationFate::Committed => {
                            unreachable!("committed output is retained separately")
                        }
                        DurablePublicationFate::ProvenNoEffect { .. } => {
                            WorldDurableSignalProgress::ProvenNoEffect
                        }
                        DurablePublicationFate::Indeterminate { .. } => {
                            WorldDurableSignalProgress::Indeterminate
                        }
                    };
                    *state = Some(State::Terminal(completion));
                    return result;
                }
                State::OwnerRetained => {
                    *state = Some(State::OwnerRetained);
                    return WorldDurableSignalProgress::Indeterminate;
                }
                State::Pending { task, denial: _ } => {
                    // The Signal owner retains the task if adapter code unwinds.
                    // Keep its original identity in World before handing out the move-only observer.
                    *state = Some(State::OwnerRetained);
                    let answer = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        task.drive(request)
                    }));
                    let answer = match answer {
                        Ok(answer) => answer,
                        Err(payload) => {
                            drop(state);
                            std::panic::resume_unwind(payload)
                        }
                    };
                    match answer {
                        Ok(SignalDurableAdvanceProgress::Pending(task)) => {
                            *state = Some(State::Pending { task, denial: None });
                            if request.consult().is_err() {
                                return WorldDurableSignalProgress::Pending;
                            }
                        }
                        Ok(SignalDurableAdvanceProgress::Finished(completion)) => {
                            match completion.into_committed() {
                                Ok(output) => {
                                    let output = Arc::new(WorldDurableSignalResult {
                                        output,
                                        _funding: self
                                            .result_funding
                                            .lock()
                                            .expect("result backing")
                                            .take(),
                                    });
                                    *state = Some(State::Committed(output));
                                }
                                Err(completion) => {
                                    *state = Some(State::Terminal(completion));
                                }
                            }
                        }
                        Err(refusal) => {
                            let (task, denial) = refusal.into_parts();
                            *state = Some(State::Pending {
                                task,
                                denial: Some(denial),
                            });
                            return WorldDurableSignalProgress::Pending;
                        }
                    }
                }
            }
        }
    }
}
