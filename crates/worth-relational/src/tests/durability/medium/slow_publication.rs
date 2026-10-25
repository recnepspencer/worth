use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact;
use crate::durability::medium::{RelationalDurableMedium, RelationalDurablePublication};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

/// Failure budget for a gate or observation nobody completes; it is not the
/// ordering mechanism.
const STALL_BUDGET: Duration = Duration::from_secs(30);

/// A medium whose first publication after the test arms it holds until the
/// test opens its gate, so that publisher holds the reservation for as long as
/// the test needs.
#[derive(Debug)]
pub(super) struct SlowFirstPublication {
    pub(super) contents: Box<dyn RelationalDurableMedium>,
    pub(super) gate: Arc<PublicationGate>,
}

#[derive(Debug, Default)]
pub(super) struct PublicationGate {
    state: Mutex<GateState>,
    changed: Condvar,
}

#[derive(Debug, Default)]
struct GateState {
    armed: bool,
    publishing: bool,
    open: bool,
}

impl PublicationGate {
    pub(super) fn await_publication(&self) {
        self.wait_until(|state| state.publishing);
    }

    pub(super) fn pass(&self) {
        let first = {
            let mut state = self.state.lock().unwrap();
            let first = state.armed && !state.publishing;
            state.publishing |= state.armed;
            first
        };
        if first {
            self.changed.notify_all();
            self.wait_until(|state| state.open);
        }
    }

    pub(super) fn arm(&self) {
        self.state.lock().unwrap().armed = true;
    }

    pub(super) fn open(&self) {
        self.state.lock().unwrap().open = true;
        self.changed.notify_all();
    }

    fn wait_until(&self, reached: impl Fn(&GateState) -> bool) {
        let deadline = Instant::now() + STALL_BUDGET;
        let mut state = self.state.lock().unwrap();
        while !reached(&state) {
            let remaining = deadline
                .checked_duration_since(Instant::now())
                .expect("the publication gate stalled");
            state = self.changed.wait_timeout(state, remaining).unwrap().0;
        }
    }
}

impl RelationalDurableMedium for SlowFirstPublication {
    fn read_transition(
        &self,
        sequence: crate::branch::RelationalBranchTransitionSequence,
    ) -> Result<
        crate::durability::branch_transition_artifact::RelationalBranchTransitionArtifact,
        crate::durability::medium::RelationalDurableTransitionReadFailure,
    > {
        self.contents.read_transition(sequence)
    }

    fn publish(
        &self,
        artifact: &RelationalCanonicalCommitArtifact,
    ) -> RelationalDurablePublication {
        self.gate.pass();
        self.contents.publish(artifact)
    }

    fn publish_transition(
        &self,
        artifact: &crate::durability::branch_transition_artifact::RelationalBranchTransitionArtifact,
    ) -> RelationalDurablePublication {
        self.gate.pass();
        self.contents.publish_transition(artifact)
    }
}
