//! Stopped requests, writers that hold the document, maintenance and close:
//! who holds the parked candidates and the recording trees (model
//! `WriterStopped`, `WriterBegin`, `WriterCommit`, `WriterDiscard`,
//! `MaintainHistory`, `MaintainIdle`, `Close`; `WedgeBug = FALSE`).
use super::*;

impl State {
    /// `WriterStopped(point)` for every point the model allows here. A stop
    /// at `Admission` lands before the candidate forks and changes nothing.
    pub(super) fn stopped(&self) -> Vec<(Outcome, State)> {
        let mut parked = self.clone();
        parked.parked.insert(self.next_candidate);
        parked.next_candidate += 1;
        let stages = parked.clone();
        let mut recording = parked.clone();
        recording.forest_origin = self.next_candidate;
        recording.forest_disposing = false;
        let mut points = vec![
            (Outcome::Stopped, self.clone()),
            (Outcome::Stopped, stages),
            (Outcome::Stopped, recording),
        ];
        if self.forest_origin != 0 {
            let mut release = parked;
            release.forest_disposing = true;
            points.push((Outcome::Stopped, release));
        }
        points
    }

    /// The `WriterStopped` point that leads from this state to `after`.
    /// `Stages` and `Release` coincide when the trees are already being
    /// released; `Stages` is then the one the model allows either way.
    pub(super) fn stop_point(&self, after: &State) -> &'static str {
        if after == self {
            "Admission"
        } else if after.forest_origin != self.forest_origin {
            "Recording"
        } else if after.forest_disposing && !self.forest_disposing {
            "Release"
        } else {
            "Stages"
        }
    }

    /// `WriterBegin(class)`: the writer takes the document and releases trees
    /// a stopped request left bound to another candidate (Phase 1's fix).
    pub(super) fn begin(&self, writer: Writer) -> Vec<(Outcome, State)> {
        let mut next = self.clone();
        next.writer = Some(writer);
        next.forest_origin = 0;
        next.forest_disposing = false;
        vec![(Outcome::Begun, next)]
    }

    /// `WriterCommit(stages, key)`. The model has no step for a commit on a
    /// diverged head or with refused stages; the workload makes neither
    /// (README, questions the DST settles).
    pub(super) fn commit(&self, key: u32, stages: &Stages) -> Vec<(Outcome, State)> {
        let free = self.forest_origin == 0 && !self.forest_disposing;
        if self.head_diverged
            || self.journal.contains_key(&(Namespace::Writer, key))
            || !self.stages_admissible(stages)
            || !free
        {
            return Vec::new();
        }
        let recorded = self.record_step(
            stages,
            [false; 2],
            (Namespace::Writer, key),
            Intent::Edit(*stages),
            Label::of_edit(stages),
        );
        vec![(Outcome::Recorded { node: self.nodes() }, recorded)]
    }

    /// `WriterDiscard`: the writer returns the document, releasing any trees.
    pub(super) fn discard(&self) -> Vec<(Outcome, State)> {
        let mut next = self.clone();
        next.writer = None;
        next.forest_origin = 0;
        next.forest_disposing = false;
        vec![(Outcome::Discarded, next)]
    }

    /// `MaintainHistory(c)` for every parked `c` and `MaintainHead` for a
    /// queued past head, or `MaintainIdle`.
    pub(super) fn maintain(&self) -> Vec<(Outcome, State)> {
        if self.parked.is_empty() && self.heads == 0 {
            return vec![(Outcome::NoPendingRelease, self.clone())];
        }
        let mut released: Vec<(Outcome, State)> = self
            .parked
            .iter()
            .map(|&candidate| {
                let mut next = self.clone();
                next.parked.remove(&candidate);
                if self.forest_origin == candidate {
                    next.forest_origin = 0;
                    next.forest_disposing = false;
                }
                (Outcome::Released, next)
            })
            .collect();
        if self.heads > 0 {
            let mut next = self.clone();
            next.heads -= 1;
            released.push((Outcome::Released, next));
        }
        released
    }

    pub(super) fn close(&self) -> State {
        let mut next = self.clone();
        next.closed = true;
        next.parked.clear();
        next.heads = 0;
        next.forest_origin = 0;
        next.forest_disposing = false;
        next
    }
}
