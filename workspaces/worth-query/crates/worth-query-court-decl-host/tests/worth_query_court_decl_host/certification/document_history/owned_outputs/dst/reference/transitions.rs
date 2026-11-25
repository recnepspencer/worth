//! The model's transitions, one function per TLA+ action.
use super::*;

impl State {
    /// The model transitions that `action` can take from this state, with the
    /// outcome each returns. Several refusals can be enabled at once; the
    /// as-built order picks one (plan 2.6 and r2 10.1): Empty, HeadDiverged,
    /// the correction's head pass (its source on the selected line), the
    /// journal (replay or drift), the candidate pass (no active fact says the
    /// source is already reversed, and it is still reversible), the stages,
    /// then recording.
    pub(super) fn transitions(&self, action: &Action) -> Vec<(Outcome, State)> {
        if self.closed {
            return Vec::new();
        }
        if self.writer.is_some() {
            // A writer holds the move-only document: only its own calls and
            // writes outside the document can run.
            return match action {
                Action::Commit { key, stages } => self.commit(*key, stages),
                Action::Discard => self.discard(),
                Action::OutOfBand(stages) => self.out_of_band(stages),
                _ => Vec::new(),
            };
        }
        match action {
            Action::Empty { .. } => self.unchanged(Outcome::Empty),
            _ if self.head_diverged && self.is_document_entry(action) => {
                // The correction's head pass reads no head (refinement (w)):
                // a source with no derivable meaning refuses there first.
                let underivable = matches!(action, Action::Envelope { intent: Intent::Correction { source, entity, .. }, .. }
                    if !self.meaning_derivable(*source, *entity));
                let mut successors =
                    self.unchanged(if underivable { Outcome::SourceRefused } else { Outcome::HeadDiverged });
                // A request that stops before its history read learns
                // nothing: the `Admission` stop needs only a held document
                // (TLA+ `WriterStopped`; traced, seed 0x7b020000 k7).
                if matches!(action, Action::Envelope { stop: Some(_), .. }) {
                    successors.extend(self.unchanged(Outcome::Stopped));
                }
                successors
            }
            Action::Envelope { key, intent, stop } => self.envelope(*key, intent, *stop),
            Action::Refused { cause, .. } => self.unchanged(Outcome::Environment(*cause)),
            Action::Undo if self.current == 0 => self.unchanged(Outcome::NavigationRefused),
            Action::Undo => self.select(self.parent[self.current as usize].unwrap()),
            Action::Redo(child) if self.is_child(*child) => self.select(*child),
            Action::Redo(_) => self.unchanged(Outcome::NavigationRefused),
            Action::Checkout(target) if *target >= self.nodes() => {
                self.unchanged(Outcome::NavigationRefused)
            }
            Action::Checkout(target) if *target == self.current => Vec::new(),
            Action::Checkout(target) => self.select(*target),
            Action::OutOfBand(stages) => self.out_of_band(stages),
            Action::Begin(writer) => self.begin(*writer),
            // No writer is open.
            Action::Commit { .. } | Action::Discard => Vec::new(),
            Action::Maintain => self.maintain(),
            Action::Close => vec![(Outcome::Closed, self.close())],
        }
    }

    /// Maintenance, close and writes outside the document are the only calls
    /// that run on a diverged head.
    fn is_document_entry(&self, action: &Action) -> bool {
        !matches!(
            action,
            Action::Maintain | Action::Close | Action::OutOfBand(_)
        )
    }

    fn unchanged(&self, outcome: Outcome) -> Vec<(Outcome, State)> {
        vec![(outcome, self.clone())]
    }

    fn is_child(&self, child: u32) -> bool {
        child < self.nodes() && self.parent[child as usize] == Some(self.current)
    }

    /// An envelope runs to its end, or, under a request that stops, may stop
    /// first at any point: `WriterStopped` needs only a writable document
    /// and an unseen key, never admissible stages or a fresh key of its own
    /// (TLA+ `WriterStopped`; traced, seed 0x7b020000 step 0). Keys here are
    /// unbounded, so an unseen one always exists.
    fn envelope(&self, key: u32, intent: &Intent, stop: Option<u32>) -> Vec<(Outcome, State)> {
        let mut successors = self.envelope_run(key, intent);
        if stop.is_some() {
            successors.extend(self.stopped());
        }
        successors
    }

    fn envelope_run(&self, key: u32, intent: &Intent) -> Vec<(Outcome, State)> {
        // The head pass: a correction's meaning comes from its source, which
        // must be a step on the selected line (`MeaningDerivable`).
        if let Intent::Correction { source, entity, .. } = intent {
            if !self.meaning_derivable(*source, *entity) {
                return self.unchanged(Outcome::SourceRefused);
            }
        }
        if let Some(entry) = self.journal.get(&(Namespace::Envelope, key)) {
            return self.unchanged(if entry.intent == *intent {
                Outcome::AlreadyRecorded { node: entry.node }
            } else {
                Outcome::IntentDrift
            });
        }
        let (stages, restored, label) = match intent {
            Intent::Edit(stages) => (*stages, [false; 2], Label::of_edit(stages)),
            Intent::Correction { source, entity, .. }
                if self.reversed_by_fact(*source, *entity)
                    || !self.source_reversible(*source, *entity) =>
            {
                return self.unchanged(Outcome::SourceRefused);
            }
            Intent::Correction {
                source,
                entity,
                stages,
            } => (
                self.with_corrective(*source, *entity, *stages),
                self.restored_by(*source, *entity),
                Some(self.corrective_label(*source, *entity)),
            ),
        };
        if !self.stages_admissible(&stages) {
            return self.unchanged(Outcome::StageRefused);
        }
        let recorded = self.record_step(&stages, restored, (Namespace::Envelope, key), intent.clone(), label);
        vec![(Outcome::Recorded { node: self.nodes() }, recorded)]
    }

    // ---- stage semantics shared by every write ----

    fn new_live(&self, stages: &Stages) -> [bool; 2] {
        ENTITIES.map(|e| match stages[e] {
            Stage::Retire => false,
            Stage::Set(_) => true,
            Stage::Skip => self.branch.live[e],
        })
    }

    /// Every stage is legal and the owned graph stays closed: retiring `E1`
    /// while `E2` stays live is a refused partial retirement.
    pub(super) fn stages_admissible(&self, stages: &Stages) -> bool {
        let live = self.new_live(stages);
        stages.iter().any(|stage| *stage != Stage::Skip)
            && ENTITIES.iter().all(|&e| stages[e] != Stage::Retire || self.branch.live[e])
            && (!live[E2] || live[E1])
    }

    /// The branch after `stages`, what they did to each entity, and the next
    /// fresh generation. Births take fresh generations `e1` first.
    fn apply(&self, stages: &Stages, restored: [bool; 2]) -> (Snapshot, [StepOp; 2], u32) {
        let mut generation = self.next_generation;
        let mut snapshot = Snapshot {
            live: self.new_live(stages),
            ..self.branch.clone()
        };
        let mut ops = [StepOp::None; 2];
        for e in ENTITIES {
            match stages[e] {
                Stage::Set(value) => {
                    snapshot.value[e] = value;
                    ops[e] = StepOp::Set;
                    if !self.branch.live[e] {
                        snapshot.generation[e] = generation;
                        generation += 1;
                        ops[e] = if restored[e] { StepOp::Restored } else { StepOp::Born };
                    }
                }
                Stage::Retire => {
                    snapshot.value[e] = 0;
                    snapshot.generation[e] = 0;
                    ops[e] = StepOp::Retired;
                }
                Stage::Skip => {}
            }
        }
        (snapshot, ops, generation)
    }

    /// The one recording owner: one node, one adoption, one journal record;
    /// the trees a stopped request left bound are released, and an open
    /// writer is done.
    pub(super) fn record_step(
        &self,
        stages: &Stages,
        restored: [bool; 2],
        key: (Namespace, u32),
        intent: Intent,
        label: Option<Label>,
    ) -> State {
        let (snapshot, ops, generation) = self.apply(stages, restored);
        let node = self.nodes();
        let mut next = self.clone();
        next.parent.push(Some(self.current));
        next.snapshots.push(snapshot.clone());
        next.ops.push(ops);
        next.labels.push(label);
        next.current = node;
        next.branch = snapshot;
        next.next_generation = generation;
        if let Intent::Correction { source, entity, .. } = intent {
            next.facts.insert(Fact { step: node, source, entity });
        }
        next.journal.insert(key, Journal { intent, node });
        next.forest_origin = 0;
        next.forest_disposing = false;
        next.writer = None;
        // `LeaveHead`: a fork the document leaves is queued for release.
        next.heads += u32::from(self.forked);
        next.forked = false;
        next
    }

    // ---- Phase 2: the correction source ----

    /// The selected step and its ancestors.
    pub(super) fn line(&self, mut node: u32) -> Vec<u32> {
        let mut line = vec![node];
        while let Some(parent) = self.parent[node as usize] {
            line.push(parent);
            node = parent;
        }
        line
    }

    /// Steps of the selected line from `source` to the current step.
    fn since(&self, source: u32) -> Vec<u32> {
        let line = self.line(self.current);
        let end = line.iter().position(|n| *n == source).map_or(0, |i| i + 1);
        line[..end].to_vec()
    }

    /// `MeaningDerivable`: the source is a recorded step on the selected
    /// line, it is not itself a correction (`CorrectionSteps`: a corrective
    /// binding declares no correction, so selecting it refuses
    /// `SourceNotCorrectable`; reapply is out of Phase 2), and its stage on
    /// `entity` has a corrective binding.
    pub(super) fn meaning_derivable(&self, source: u32, entity: usize) -> bool {
        source != 0
            && source < self.nodes()
            && self.line(self.current).contains(&source)
            && !self.is_correction_step(source)
            && matches!(self.ops[source as usize][entity], StepOp::Born | StepOp::Retired)
    }

    /// `CorrectionSteps`: a step a recorded correction envelope made.
    fn is_correction_step(&self, node: u32) -> bool {
        self.journal
            .values()
            .any(|entry| entry.node == node && matches!(entry.intent, Intent::Correction { .. }))
    }

    /// `SourceReversible`: the source's occurrence is still the live one, or
    /// its retirement has held on the selected line.
    pub(super) fn source_reversible(&self, source: u32, entity: usize) -> bool {
        if !self.meaning_derivable(source, entity) {
            return false;
        }
        let recorded = &self.snapshots[source as usize];
        match self.ops[source as usize][entity] {
            StepOp::Born => {
                self.branch.live[entity]
                    && self.branch.generation[entity] == recorded.generation[entity]
            }
            _ => self
                .since(source)
                .iter()
                .all(|n| !self.snapshots[*n as usize].live[entity]),
        }
    }

    /// `ReversedByFact`: an active fact says the source is already reversed
    /// on `entity` (`reverse_lineage`'s `SourceAlreadyReversed`).
    fn reversed_by_fact(&self, source: u32, entity: usize) -> bool {
        self.facts.iter().any(|f| f.source == source && f.entity == entity)
    }

    /// Every fact a recorded correction posted, active or not.
    pub(super) fn posted_facts(&self) -> impl Iterator<Item = Fact> + '_ {
        self.journal.values().filter_map(|entry| match entry.intent {
            Intent::Correction { source, entity, .. } => Some(Fact { step: entry.node, source, entity }),
            Intent::Edit(_) => None,
        })
    }

    /// The owned child the source created with `E1`, while that occurrence is
    /// still live: `co_retire_created` retires it with its owner (refinement
    /// (p)).
    /// `CoRetired`: what the same source stage created. Each stage creates
    /// one entity, so nothing is co-retired across stages (refinement (p),
    /// narrowed to `co_retire_created`'s declared contract, ruling
    /// 2026-11-25); a live sibling makes the reversal a refused partial
    /// retirement.
    fn co_retired(&self, _source: u32, _entity: usize) -> bool {
        false
    }

    fn with_corrective(&self, source: u32, entity: usize, stages: Stages) -> Stages {
        let s = source as usize;
        let mut corrected = stages;
        corrected[entity] = match self.ops[s][entity] {
            StepOp::Born => Stage::Retire,
            _ => {
                let before = self.parent[s].expect("a recorded step has a parent");
                Stage::Set(self.snapshots[before as usize].value[entity])
            }
        };
        if self.co_retired(source, entity) && corrected[E2] == Stage::Skip {
            corrected[E2] = Stage::Retire;
        }
        corrected
    }

    fn corrective_label(&self, source: u32, entity: usize) -> Label {
        match self.ops[source as usize][entity] {
            StepOp::Born => Label::UndoCreate(entity),
            _ => Label::UndoRetire(entity),
        }
    }

    fn restored_by(&self, source: u32, entity: usize) -> [bool; 2] {
        let retired = self.ops[source as usize][entity] == StepOp::Retired;
        ENTITIES.map(|e| retired && e == entity)
    }

    // ---- navigation and writes outside the document ----

    fn select(&self, target: u32) -> Vec<(Outcome, State)> {
        let mut next = self.clone();
        next.current = target;
        next.branch = self.snapshots[target as usize].clone();
        next.facts = self.facts_after_move(target);
        // A move forks a new branch and queues the fork it leaves (TLC
        // bounds the queue with `MaxHeads`; the DST's runs stay inside it).
        next.heads += u32::from(self.forked);
        next.forked = true;
        vec![(Outcome::Moved, next)]
    }

    /// The move retires the facts of the steps it leaves and restores, at
    /// their original IDs, those of the steps it enters (ruling (u)), one
    /// step at a time as production does, not recomputed from the line.
    fn facts_after_move(&self, target: u32) -> BTreeSet<Fact> {
        let from = self.line(self.current);
        let to = self.line(target);
        let left = |f: &Fact| from.contains(&f.step) && !to.contains(&f.step);
        let entered = |f: &Fact| to.contains(&f.step) && !from.contains(&f.step);
        let mut facts: BTreeSet<Fact> = self.facts.iter().filter(|f| !left(f)).copied().collect();
        facts.extend(self.posted_facts().filter(entered));
        facts
    }

    /// `OutOfBandMutation(stages)`: any admissible change, births fresh.
    fn out_of_band(&self, stages: &Stages) -> Vec<(Outcome, State)> {
        if self.head_diverged || !self.stages_admissible(stages) {
            return Vec::new();
        }
        let (snapshot, _, generation) = self.apply(stages, [false; 2]);
        let mut next = self.clone();
        next.branch = snapshot;
        next.next_generation = generation;
        next.head_diverged = true;
        vec![(Outcome::OutOfBand, next)]
    }
}
