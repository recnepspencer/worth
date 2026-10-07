//! Test-owned observations and bounded scheduling, never production result inputs.
use std::sync::{Arc, Condvar, Mutex, Weak};
use worth_execution::{CancellationSource, ExecutionAuthority};
static ACTIVE: Mutex<Option<Weak<Observations>>> = Mutex::new(None);
#[derive(Default)]
struct State {
    events: Vec<(String, usize)>,
    reorder: bool,
    timeout: Option<String>,
    entered: Vec<String>,
    cancellation: Option<(CancellationSource, &'static str, bool)>,
}
struct Observations {
    state: Mutex<State>,
    wake: Condvar,
}
/// Exclusive owner of every probe access for the entire test, across workers.
pub(crate) struct ComputeProbe {
    authority: crate::test_execution_authority::AuthorityGuard,
    observations: Arc<Observations>,
}
impl ComputeProbe {
    pub(crate) fn new() -> Self {
        let authority = crate::test_execution_authority::authority();
        let observations = Arc::new(Observations {
            state: Mutex::new(State::default()),
            wake: Condvar::new(),
        });
        *ACTIVE.lock().unwrap_or_else(|e| e.into_inner()) = Some(Arc::downgrade(&observations));
        Self {
            authority,
            observations,
        }
    }
    /// The owner's single root stays shared; this guard excludes all other tests
    /// for its lifetime, so constructed schedules have all four slots available.
    pub(crate) fn authority(&self) -> &ExecutionAuthority {
        &self.authority
    }
    pub(crate) fn assert_schedule_completed(&self) {
        let timeout = self
            .observations
            .state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .timeout
            .clone();
        if let Some(diagnostic) = timeout {
            panic!("{diagnostic}");
        }
    }
    pub(crate) fn reorder(&self) {
        self.observations
            .state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .reorder = true;
    }
    pub(crate) fn cancel_at_checkpoint(
        &self,
        source: CancellationSource,
        member: &'static str,
        hold_left: bool,
    ) {
        let mut state = self
            .observations
            .state
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        state.reorder = false;
        state.entered.clear();
        state.cancellation = Some((source, member, hold_left));
    }
    pub(crate) fn take_computes(&self) -> Vec<(String, usize)> {
        std::mem::take(
            &mut self
                .observations
                .state
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .events,
        )
    }
}
impl Drop for ComputeProbe {
    fn drop(&mut self) {
        *ACTIVE.lock().unwrap_or_else(|e| e.into_inner()) = None;
        if !std::thread::panicking() {
            self.assert_schedule_completed();
            assert_eq!(Arc::strong_count(&self.observations), 1);
        }
    }
}
fn observations() -> Option<Arc<Observations>> {
    ACTIVE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .as_ref()
        .and_then(Weak::upgrade)
}
pub(super) fn before(stage: &str) -> bool {
    if let Some(probe) = observations() {
        let mut state = probe.state.lock().unwrap_or_else(|e| e.into_inner());
        state.entered.push(stage.into());
        probe.wake.notify_all();
        loop {
            if state.timeout.is_some() {
                return false;
            }
            let missing = if state.reorder && stage == "left" {
                (!state.events.iter().any(|(member, _)| member == "middle")).then_some("middle")
            } else if let Some((source, interrupter, hold_left)) = &state.cancellation {
                if *hold_left && stage == "left" {
                    (!source.token().is_cancelled()).then_some(*interrupter)
                } else if *hold_left && stage == *interrupter {
                    (!state.entered.iter().any(|member| member == "left")).then_some("left")
                } else if !hold_left && stage == *interrupter {
                    super::super::MEMBERS
                        .iter()
                        .take_while(|member| **member != stage)
                        .find(|member| !state.events.iter().any(|(arrived, _)| arrived == **member))
                        .copied()
                } else {
                    None
                }
            } else {
                None
            };
            let Some(missing) = missing else { break };
            // Hang guard only: time never selects an expected production outcome.
            let (next, timeout) = probe
                .wake
                .wait_timeout(state, std::time::Duration::from_secs(20))
                .unwrap_or_else(|e| e.into_inner());
            state = next;
            if timeout.timed_out() {
                state.timeout = Some(format!(
                    "rendezvous for {stage}: member {missing} never arrived"
                ));
                probe.wake.notify_all();
                return false;
            }
        }
    }
    true
}
pub(super) fn completed(stage: &str, work: usize) -> usize {
    observations().map_or(0, |probe| {
        let mut state = probe.state.lock().unwrap_or_else(|e| e.into_inner());
        state.events.push((stage.into(), work));
        let ordinal = state.events.len();
        probe.wake.notify_all();
        ordinal
    })
}
pub(super) fn cancel() {
    if let Some(probe) = observations() {
        let state = probe.state.lock().unwrap_or_else(|e| e.into_inner());
        if let Some((source, _, _)) = &state.cancellation {
            source.cancel();
            probe.wake.notify_all();
        }
    }
}
