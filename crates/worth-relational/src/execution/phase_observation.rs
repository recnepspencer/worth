//! Passive, bounded observations of actual original-request phase work.
use super::RelationalExecutionDenialCause as Cause;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Inclusive scope work: enclosing and nested phase totals overlap. Do not sum
/// rows as disjoint work; use the original request report for the aggregate.
pub struct MaintenancePhaseWork {
    pub phase: &'static str,
    pub charged_work: u64,
    /// Native-only execution has controls but no charged-work meter.
    pub accounted: bool,
}

/// Passive events at the actual symbol-round and coupled-completion edges.
/// They confer no authority and never control execution or request accounting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolPreparationEvent {
    RoundStarted,
    AssemblerCallback,
    AssemblerBreak,
    CoupledCompleted(bool),
    ColdBucketRetry,
    ProbeReturnedOk,
}

#[derive(Debug, Default)]
pub struct MaintenanceWorkObservation {
    pub symbol_preparation: Vec<SymbolPreparationEvent>,
    pub omitted_symbol_preparation_events: usize,
    pub phases: Vec<MaintenancePhaseWork>,
    pub omitted_phases: usize,
    /// Actual Native ownership records written, including structural retirements.
    pub ownership_rows: u64,
    /// Canonical record rows of each completed publication in this observation.
    pub published_rows: Vec<usize>,
    pub omitted_publications: usize,
    pub content_index_hash_bytes: u64,
    pub partition_value_hash_bytes: u64,
    pub commit_payload_hash_bytes: u64,
    pub visibility_payload_hash_bytes: u64,
}

struct Recorder {
    maximum: usize,
    observation: MaintenanceWorkObservation,
}
thread_local! {
    static RECORDER: RefCell<Option<Rc<RefCell<Recorder>>>> = const { RefCell::new(None) };
}
struct Restore(Option<Rc<RefCell<Recorder>>>);
impl Drop for Restore {
    fn drop(&mut self) {
        RECORDER.with(|slot| *slot.borrow_mut() = self.0.take());
    }
}

/// Observe phases on this thread only while `operation` runs. Nested captures
/// are isolated and restore their predecessor on return or unwind. Observation
/// never changes a request's controls or bill. Cost: up to `maximum` records.
pub fn observe_maintenance_work<T>(
    maximum: usize,
    operation: impl FnOnce() -> T,
) -> (T, MaintenanceWorkObservation) {
    let recorder = Rc::new(RefCell::new(Recorder {
        maximum,
        observation: MaintenanceWorkObservation::default(),
    }));
    let restore = Restore(RECORDER.with(|slot| slot.replace(Some(Rc::clone(&recorder)))));
    let value = operation();
    drop(restore);
    let observation = std::mem::take(&mut recorder.borrow_mut().observation);
    (value, observation)
}

fn record(work: MaintenancePhaseWork) {
    RECORDER.with(|slot| {
        if let Some(recorder) = slot.borrow().as_ref() {
            let mut recorder = recorder.borrow_mut();
            if recorder.observation.phases.len() < recorder.maximum {
                recorder.observation.phases.push(work);
            } else {
                recorder.observation.omitted_phases =
                    recorder.observation.omitted_phases.saturating_add(1);
            }
        }
    });
}

/// Record only while the existing bounded, thread-owned observer is installed.
/// The product path never reads these events or changes a decision for them.
pub(crate) fn symbol_preparation_event(event: SymbolPreparationEvent) {
    RECORDER.with(|slot| {
        if let Some(recorder) = slot.borrow().as_ref() {
            let mut recorder = recorder.borrow_mut();
            if recorder.observation.symbol_preparation.len() < recorder.maximum {
                recorder.observation.symbol_preparation.push(event);
            } else {
                recorder.observation.omitted_symbol_preparation_events = recorder
                    .observation
                    .omitted_symbol_preparation_events
                    .saturating_add(1);
            }
        }
    });
}

pub(crate) enum HashPurpose {
    ContentIndex,
    PartitionValue,
    CommitPayload,
    BranchVisibility,
}

pub(crate) fn hash_bytes(purpose: HashPurpose, bytes: usize) {
    RECORDER.with(|slot| {
        if let Some(recorder) = slot.borrow().as_ref() {
            let mut recorder = recorder.borrow_mut();
            let observation = &mut recorder.observation;
            let total = match purpose {
                HashPurpose::ContentIndex => &mut observation.content_index_hash_bytes,
                HashPurpose::PartitionValue => &mut observation.partition_value_hash_bytes,
                HashPurpose::CommitPayload => &mut observation.commit_payload_hash_bytes,
                HashPurpose::BranchVisibility => &mut observation.visibility_payload_hash_bytes,
            };
            *total = total.saturating_add(bytes as u64);
        }
    });
}

pub(crate) fn ownership_rows(rows: usize) {
    RECORDER.with(|slot| {
        if let Some(recorder) = slot.borrow().as_ref() {
            let mut recorder = recorder.borrow_mut();
            recorder.observation.ownership_rows = recorder
                .observation
                .ownership_rows
                .saturating_add(rows as u64);
        }
    });
}

pub(crate) fn published_commit(rows: usize) {
    RECORDER.with(|slot| {
        if let Some(recorder) = slot.borrow().as_ref() {
            let mut recorder = recorder.borrow_mut();
            if recorder.observation.published_rows.len() < recorder.maximum {
                recorder.observation.published_rows.push(rows);
            } else {
                recorder.observation.omitted_publications =
                    recorder.observation.omitted_publications.saturating_add(1);
            }
        }
    });
}

/// Name a real Execution scope. All nested charges settle into its own meter.
pub(crate) fn run<T, E>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    phase: &'static str,
    operation: impl FnOnce() -> Result<T, E>,
    refusal: impl FnOnce(Cause) -> E,
) -> Result<T, E> {
    let mut result = None;
    let handoff = execution.run_native_with_checkpoint(|context| {
        let accounted = context.has_accounting();
        let before = if accounted { context.charged_work() } else { 0 };
        result = Some(operation());
        record(MaintenancePhaseWork {
            phase,
            charged_work: if accounted {
                context.charged_work().saturating_sub(before)
            } else {
                0
            },
            accounted,
        });
    });
    match (handoff, result) {
        (Ok(_), Some(value)) => value,
        (Err(stop), Some(Err(inner))) if Cause::from(stop) == Cause::NestedStopped => Err(inner),
        (Err(stop), _) => Err(refusal(stop.into())),
        // The scope engine invokes its callback exactly once before success.
        (Ok(_), None) => unreachable!("completed phase invokes its operation"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_observation_restores_the_outer_recorder_after_unwinding() {
        let (_, outer) = observe_maintenance_work(1, || {
            let interrupted = std::panic::catch_unwind(|| {
                observe_maintenance_work(1, || {
                    record(MaintenancePhaseWork {
                        phase: "inner",
                        charged_work: 0,
                        accounted: false,
                    });
                    panic!("interrupt observation");
                });
            });
            assert!(interrupted.is_err());
            record(MaintenancePhaseWork {
                phase: "outer",
                charged_work: 0,
                accounted: false,
            });
            record(MaintenancePhaseWork {
                phase: "overflow",
                charged_work: 0,
                accounted: false,
            });
        });
        assert_eq!(
            outer.phases.iter().map(|row| row.phase).collect::<Vec<_>>(),
            ["outer"]
        );
        assert!(outer.omitted_phases > 0);
        RECORDER.with(|slot| assert!(slot.borrow().is_none()));
    }
}
