//! Seeded workloads. A seed draws a sequence of choices; each choice is
//! resolved against the oracle's state when it runs, so a sequence with
//! choices deleted (shrinking) still runs.
use super::reference::{
    push_order, Action, Environment, Intent, Stage, Stages, State, StepOp, Writer, E1, E2, ENTITIES,
    NO_STAGES,
};
mod held;
mod keys;

pub use keys::Keys;

/// SplitMix64 (Steele, Lea and Flood): one `u64` of state, no dependency.
pub struct SplitMix64(pub u64);

impl SplitMix64 {
    pub fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    fn below(&mut self, bound: u64) -> u64 {
        self.next() % bound
    }

    fn pick(&mut self) -> u8 {
        self.below(256) as u8
    }
}

/// A stage choice; values stay in the trace module's `Values` ({1, 2}).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StagePick {
    Skip,
    Retire,
    Set(u64),
}

/// One drawn choice. `pick` fields select among what exists when it runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    Edit([StagePick; 2]),
    /// Repeat a recorded key with its intent or a changed one.
    Replay { pick: u8, change: Change },
    /// Reverse one entity's stage in a recorded step; `on_line` prefers a
    /// step on the selected line; `cascade` adds the caller's retirement of
    /// the owned child; `stop` stops the request at that consult.
    Correction { pick: u8, entity: usize, on_line: bool, cascade: bool, stop: Option<u16> },
    Refused(Environment),
    Empty,
    /// An edit under a request that stops at consult `at`.
    Stopped { stages: [StagePick; 2], at: u16 },
    /// The last stopped envelope again, same key and intent, not stopped.
    RetryStopped,
    Undo,
    Redo { pick: u8 },
    Checkout { pick: u8 },
    OutOfBand([StagePick; 2]),
    /// Start an `.edit` (or `.gesture`) that holds the document.
    Begin { gesture: bool },
    Maintain,
    Discard,
}

/// How a replay changes the recorded intent under the same key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Change {
    /// The recorded intent: `AlreadyRecorded`.
    Same,
    /// One coupled stage changed: `IntentDrift`.
    Stages,
    /// A correction's key on another step of the selected line that did the
    /// same to the entity, so the derived inverse input can be identical:
    /// `IntentDrift`, which only the source step in the meaning gives
    /// (plan r3, 11). An edit that stages the corrective binding directly is
    /// outside the harness's edit vocabulary; the S2 court and
    /// `hand-replay-other-source.ndjson` cover it.
    Source,
}

/// The milestone's mandated trace scenarios (`tree-based-undo.md:150-154`)
/// plus stopped-request recovery. `WORTH_DST_SCENARIO` puts the scenario's
/// skeleton first in every seed; the seeded choices follow it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scenario {
    BranchAfterDelete,
    Replay,
    OutOfBand,
    StopRecovery,
}

impl Scenario {
    pub fn from_name(name: &str) -> Self {
        match name {
            "branch-after-delete" => Self::BranchAfterDelete,
            "replay" => Self::Replay,
            "out-of-band" => Self::OutOfBand,
            "stop-recovery" => Self::StopRecovery,
            other => panic!("WORTH_DST_SCENARIO names no scenario: {other}"),
        }
    }

    fn skeleton(self) -> Vec<Op> {
        use StagePick::{Retire, Set, Skip};
        let delete = |stop| Op::Correction { pick: 0, entity: E1, on_line: true, cascade: false, stop };
        match self {
            // Ruling (u): create e1+e2 (step 1), delete e1 (step 2, co-retires
            // e2), undo, branch edit (step 3), delete again on the sibling
            // (step 4, must record), checkout step 2, where a fresh-key delete
            // of the same source must refuse SourceAlreadyReversed, then
            // checkout step 4 and step 3. A checkout pick indexes the steps
            // other than the current one.
            Self::BranchAfterDelete => vec![
                Op::Edit([Set(1), Set(1)]),
                delete(None),
                Op::Undo,
                Op::Edit([Set(2), Skip]),
                Op::Correction { pick: 1, entity: E1, on_line: true, cascade: false, stop: None },
                Op::Checkout { pick: 2 },
                Op::Correction { pick: 1, entity: E1, on_line: true, cascade: false, stop: None },
                Op::Checkout { pick: 3 },
                Op::Checkout { pick: 3 },
            ],
            // an edit and a correction replayed, again after navigation,
            // then with a changed payload; then e1 created again (step 3)
            // and the deletion's key replayed against that second creation:
            // the derived inverse input is the same, so only the source step
            // in the meaning answers IntentDrift (plan r3, 11).
            Self::Replay => vec![
                Op::Edit([Set(1), Set(2)]),
                Op::Replay { pick: 0, change: Change::Same },
                delete(None),
                Op::Replay { pick: 1, change: Change::Same },
                Op::Undo,
                Op::Redo { pick: 2 },
                Op::Replay { pick: 1, change: Change::Same },
                Op::Replay { pick: 1, change: Change::Stages },
                Op::Edit([Set(2), Skip]),
                Op::Replay { pick: 1, change: Change::Source },
            ],
            Self::OutOfBand => vec![
                Op::Edit([Set(1), Set(1)]),
                Op::Edit([Skip, Set(2)]),
                Op::OutOfBand([Skip, Retire]),
                Op::Edit([Set(2), Skip]),
                Op::Undo,
                Op::Maintain,
            ],
            Self::StopRecovery => vec![
                Op::Edit([Set(1), Set(1)]),
                Op::Stopped { stages: [Set(2), Skip], at: 600 },
                Op::Begin { gesture: false },
                Op::Discard,
                Op::Stopped { stages: [Skip, Set(2)], at: 900 },
                Op::Maintain,
                Op::Maintain,
                delete(Some(700)),
                Op::RetryStopped,
            ],
        }
    }
}

/// Every environment refusal the model names; a system meets a subset.
pub const ENVIRONMENT: [Environment; 6] = [
    Environment::MetadataTooLarge,
    Environment::ForeignModel,
    Environment::UnownedOperationScope,
    Environment::OperationNotDeclared,
    Environment::Budget,
    Environment::UnsupportedCorrection,
];

fn stage(rng: &mut SplitMix64) -> StagePick {
    match rng.below(6) {
        0 | 1 => StagePick::Skip,
        2 => StagePick::Retire,
        3 | 4 => StagePick::Set(1),
        _ => StagePick::Set(2),
    }
}

/// `count` choices for one seed, after `scenario`'s skeleton. Weights favour
/// recording and navigation; every Phase 1 and Phase 2 workload of the
/// formal contract appears.
pub fn generate(
    seed: u64,
    count: usize,
    corrections: bool,
    scenario: Option<Scenario>,
    environment: &[Environment],
) -> Vec<Op> {
    let mut rng = SplitMix64(seed);
    let mut ops = scenario.map_or_else(Vec::new, Scenario::skeleton);
    let tail = count.saturating_sub(ops.len());
    ops.extend((0..tail).map(|_| held::narrow(match rng.below(if corrections { 28 } else { 23 }) {
        0..=4 => Op::Edit([stage(&mut rng), stage(&mut rng)]),
        5 => Op::Replay {
            pick: rng.pick(),
            change: [Change::Same, Change::Stages, Change::Source][rng.below(3) as usize],
        },
        6 => Op::Refused(environment[rng.below(environment.len() as u64) as usize]),
        7 => Op::Empty,
        8 => Op::Stopped { stages: [stage(&mut rng), stage(&mut rng)], at: rng.below(1024) as u16 },
        9..=11 => Op::Undo,
        12 | 13 => Op::Redo { pick: rng.pick() },
        14 | 15 => Op::Checkout { pick: rng.pick() },
        16 => Op::OutOfBand([stage(&mut rng), stage(&mut rng)]),
        17 => Op::Maintain,
        18 => Op::Begin { gesture: rng.below(2) == 0 },
        19 => Op::Discard,
        20 => Op::RetryStopped,
        21 | 22 => Op::Edit([StagePick::Set(1 + rng.below(2)), StagePick::Set(1 + rng.below(2))]),
        _ => Op::Correction {
            pick: rng.pick(),
            entity: rng.below(2) as usize,
            on_line: rng.below(4) != 0,
            cascade: rng.below(2) == 0,
            stop: (rng.below(4) == 0).then(|| rng.below(1024) as u16),
        },
    })));
    ops
}

fn stages(picks: [StagePick; 2]) -> Stages {
    picks.map(|pick| match pick {
        StagePick::Skip => Stage::Skip,
        StagePick::Retire => Stage::Retire,
        StagePick::Set(value) => Stage::Set(value),
    })
}

fn nth<T: Copy>(items: &[T], pick: u8) -> Option<T> {
    (!items.is_empty()).then(|| items[pick as usize % items.len()])
}

/// The call `op` makes from `state`, or `None` when it has nothing to act on.
pub fn resolve(op: &Op, state: &State, keys: &mut Keys) -> Option<Action> {
    if state.writer().is_some() {
        return Some(writer_call(op, state, keys));
    }
    let nodes: Vec<u32> = (0..state.nodes()).collect();
    Some(match *op {
        // Stages that all skip push nothing: the real call is the empty
        // envelope (`RefuseEmpty`), not an edit (traced, S1 probe M1).
        Op::Edit(picks) | Op::Stopped { stages: picks, .. } if stages(picks) == NO_STAGES => {
            Action::Empty { key: keys.fresh() }
        }
        Op::Edit(picks) => keys.envelope(Intent::Edit(stages(picks)), None),
        Op::Replay { pick, change } => {
            let recorded: Vec<(u32, Intent)> = state
                .recorded()
                .filter(|(_, intent)| change != Change::Source || matches!(intent, Intent::Correction { .. }))
                .map(|(k, i)| (k, i.clone()))
                .collect();
            let (key, intent) = recorded.get(pick as usize % recorded.len().max(1))?.clone();
            let intent = match change {
                Change::Same => intent,
                Change::Stages => changed(intent),
                Change::Source => other_source(state, intent)?,
            };
            Action::Envelope { key, intent, stop: None }
        }
        Op::Correction { pick, entity, on_line, cascade, stop } => {
            let line = line(state);
            let source = nth(if on_line { &line } else { &nodes[1..] }, pick)?;
            let entity = [entity, 1 - entity]
                .into_iter()
                .find(|&e| matches!(state.op(source, e), StepOp::Born | StepOp::Retired))
                .unwrap_or(entity);
            let mut coupled = [Stage::Skip; 2];
            if cascade && entity == E1 {
                coupled[E2] = Stage::Retire;
            }
            keys.envelope(Intent::Correction { source, entity, stages: coupled }, stop.map(u32::from))
        }
        Op::Refused(cause) => Action::Refused { key: keys.fresh(), cause },
        Op::Empty => Action::Empty { key: keys.fresh() },
        Op::Stopped { stages: picks, at } => keys.envelope(Intent::Edit(stages(picks)), Some(u32::from(at))),
        Op::RetryStopped => {
            let (key, intent) = keys.stopped.clone()?;
            Action::Envelope { key, intent, stop: None }
        }
        Op::Undo => Action::Undo,
        Op::Redo { pick } => Action::Redo(nth(&nodes, pick)?),
        Op::Checkout { pick } => {
            let others: Vec<u32> = nodes.into_iter().filter(|n| *n != state.current()).collect();
            Action::Checkout(nth(&others, pick)?)
        }
        Op::OutOfBand(picks) => out_of_band(state, picks)?,
        // A writer starts only on a current head (`WriterBegin` is
        // `Writable`); the trace module has no refused begin.
        Op::Begin { .. } if state.head_diverged() => return None,
        Op::Begin { gesture } => Action::Begin(if gesture { Writer::Gesture } else { Writer::Edit }),
        Op::Maintain => Action::Maintain,
        // No writer is open: nothing to discard.
        Op::Discard => return None,
    })
}

/// While a writer holds the document only its calls and writes outside the
/// document run: an edit's stages become the writer's commit (admissible,
/// on a current head), a write outside stays one, anything else discards.
fn writer_call(op: &Op, state: &State, keys: &mut Keys) -> Action {
    match *op {
        Op::OutOfBand(picks) => out_of_band(state, picks).unwrap_or(Action::Discard),
        Op::Edit(picks) | Op::Stopped { stages: picks, .. } if !state.head_diverged() => {
            let mut stages = stages(picks);
            // An `.edit` runs one binding: it keeps the first stage it pushes.
            if state.writer() == Some(Writer::Edit) {
                let first = push_order(&stages).into_iter().find(|e| stages[*e] != Stage::Skip);
                for entity in ENTITIES {
                    if Some(entity) != first {
                        stages[entity] = Stage::Skip;
                    }
                }
            }
            // A writer's step needs a stage.
            if stages == NO_STAGES || !state.admits(&stages) {
                stages = [Stage::Set(1), Stage::Skip];
            }
            Action::Commit { key: keys.fresh(), stages }
        }
        _ => Action::Discard,
    }
}

/// An admissible write outside the document, on a current head.
fn out_of_band(state: &State, picks: [StagePick; 2]) -> Option<Action> {
    let stages = stages(picks);
    (!state.head_diverged() && state.admits(&stages)).then_some(Action::OutOfBand(stages))
}

/// The selected line without the root.
fn line(state: &State) -> Vec<u32> {
    let mut line = Vec::new();
    let mut node = state.current();
    while node != 0 {
        line.push(node);
        node = state.parent_of(node).unwrap_or(0);
    }
    line
}

/// The correction on another step of the selected line that did the same to
/// its entity (another creation, or another retirement).
fn other_source(state: &State, intent: Intent) -> Option<Intent> {
    let Intent::Correction { source, entity, stages } = intent else {
        return None;
    };
    let kind = state.op(source, entity);
    let other = line(state).into_iter().find(|&n| n != source && state.op(n, entity) == kind)?;
    Some(Intent::Correction { source: other, entity, stages })
}

/// The same key with one stage changed: a conflicting payload.
fn changed(intent: Intent) -> Intent {
    let bump = |stage: Stage| match stage {
        Stage::Set(value) => Stage::Set(3 - value),
        Stage::Skip => Stage::Set(1),
        // Never `Skip`: a changed intent that became empty would refuse
        // `Empty` before its key is looked at.
        Stage::Retire => Stage::Set(1),
    };
    match intent {
        Intent::Edit(mut stages) => {
            stages[E1] = bump(stages[E1]);
            Intent::Edit(stages)
        }
        Intent::Correction { source, entity, mut stages } => {
            let other = 1 - entity;
            stages[other] = bump(stages[other]);
            Intent::Correction { source, entity, stages }
        }
    }
}
