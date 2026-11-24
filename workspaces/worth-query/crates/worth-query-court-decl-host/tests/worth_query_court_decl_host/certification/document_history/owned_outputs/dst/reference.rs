//! The reference oracle: `specs/tla/tree_based_undo/TreeBasedUndo.tla`
//! translated to an in-memory state machine.
//!
//! The model is nondeterministic where the harness cannot observe a choice
//! (where a stopped request stopped, which parked candidate maintenance
//! released). The oracle therefore keeps every model state consistent with
//! what the harness has seen, as TLC trace validation does, and disagrees when
//! no state remains. Among the outcomes the model allows for one call, the
//! oracle expects the one the as-built refusal order picks (plan 2.6).
use std::collections::{BTreeMap, BTreeSet};

mod custody;
mod invariants;
mod observation;
mod transitions;

pub use observation::{Fact, History, Label, Observed, Redo, RedoChoice, Row};

/// The authored member; `E2` is owned under it.
pub const E1: usize = 0;
/// The owned child of `E1`.
pub const E2: usize = 1;
pub const ENTITIES: [usize; 2] = [E1, E2];
pub const NAMES: [&str; 2] = ["e1", "e2"];

/// One envelope stage on one entity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Stage {
    Skip,
    Retire,
    Set(u64),
}

pub type Stages = [Stage; 2];

/// The order an envelope pushes its stages: `e1` first, unless `e1` retires,
/// when `e2`'s stage goes first (a retirement of the owner follows its
/// child's).
pub fn push_order(stages: &Stages) -> [usize; 2] {
    if stages[E1] == Stage::Retire {
        [E2, E1]
    } else {
        [E1, E2]
    }
}
pub const NO_STAGES: Stages = [Stage::Skip, Stage::Skip];

/// What a recorded step did to one entity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum StepOp {
    None,
    Born,
    Set,
    Retired,
    Restored,
}

/// Authored state of the document branch. Generations are the dense
/// first-seen numbering of the real full-generation IDs; 0 means not live.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Snapshot {
    pub live: [bool; 2],
    pub generation: [u32; 2],
    pub value: [u64; 2],
}

impl Snapshot {
    pub const EMPTY: Self = Self {
        live: [false; 2],
        generation: [0; 2],
        value: [0; 2],
    };

    /// The first output tree: one row per live entity, its value.
    pub fn outputs_a(&self) -> [u64; 2] {
        ENTITIES.map(|e| if self.live[e] { self.value[e] } else { 0 })
    }

    /// The second output tree: one row per live entity, its value plus 10.
    pub fn outputs_b(&self) -> [u64; 2] {
        ENTITIES.map(|e| if self.live[e] { self.value[e] + 10 } else { 0 })
    }
}

/// The complete intent of an envelope, which its key names.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Intent {
    Edit(Stages),
    /// Reverse entity `entity`'s stage in recorded step `source`, with the
    /// caller's coupled `stages` (which skip `entity`).
    Correction {
        source: u32,
        entity: usize,
        stages: Stages,
    },
}

/// Refusals whose cause lies outside the modelled state.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Environment {
    MetadataTooLarge,
    ForeignModel,
    UnownedOperationScope,
    OperationNotDeclared,
    Budget,
    UnsupportedCorrection,
}

impl Environment {
    /// The trace name; `UnsupportedCorrection` is reported as the stage
    /// refusal the model has for it (ruling 8a.2).
    pub fn trace_name(self) -> &'static str {
        match self {
            Self::MetadataTooLarge => "MetadataTooLarge",
            Self::ForeignModel => "ForeignModel",
            Self::UnownedOperationScope => "UnownedOperationScope",
            Self::OperationNotDeclared => "OperationNotDeclared",
            Self::Budget => "Budget",
            Self::UnsupportedCorrection => "UnsupportedOperation",
        }
    }
}

/// A writer that holds the move-only document between calls (model
/// `WriterClasses`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Writer {
    Edit,
    Gesture,
}

/// One call the harness makes, resolved against the oracle's state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    /// `edit_envelope` / `correction_envelope`, then `execute_recorded`;
    /// with `stop`, under a request that stops at that consult.
    Envelope {
        key: u32,
        intent: Intent,
        stop: Option<u32>,
    },
    /// An envelope built to meet one environment refusal, under a fresh key.
    Refused { key: u32, cause: Environment },
    /// An envelope with no stage.
    Empty { key: u32 },
    Undo,
    Redo(u32),
    Checkout(u32),
    /// Write these stages on the document branch outside the document.
    OutOfBand(Stages),
    /// Start an `.edit` or `.gesture` and advance it to `AwaitingSelection`.
    Begin(Writer),
    /// Commit the open writer's step with these stages under `key`.
    Commit { key: u32, stages: Stages },
    /// `discard` the open writer.
    Discard,
    Maintain,
    Close,
}

/// What the call returned, in the model's terms.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Outcome {
    Recorded { node: u32 },
    AlreadyRecorded { node: u32 },
    IntentDrift,
    StageRefused,
    SourceRefused,
    HeadDiverged,
    Empty,
    Stopped,
    Environment(Environment),
    Moved,
    NavigationRefused,
    OutOfBand,
    Begun,
    NoPendingRelease,
    Released,
    Discarded,
    Closed,
}

/// Who recorded a step under a key (TLA+ `Namespaces`, refinement (v)): an
/// envelope's key is tagged as one, a writer's is its gesture group or its
/// operation's mutation key, and the two never meet.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Namespace {
    Envelope,
    Writer,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Journal {
    pub intent: Intent,
    pub node: u32,
}

/// One model state. Field names follow the TLA+ variables.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct State {
    pub(super) parent: Vec<Option<u32>>,
    pub(super) snapshots: Vec<Snapshot>,
    pub(super) ops: Vec<[StepOp; 2]>,
    /// Each step's label (its stage-0 operation); the root has none.
    pub(super) labels: Vec<Option<Label>>,
    pub(super) current: u32,
    pub(super) branch: Snapshot,
    pub(super) next_generation: u32,
    pub(super) journal: BTreeMap<(Namespace, u32), Journal>,
    pub(super) head_diverged: bool,
    pub(super) closed: bool,
    pub(super) next_candidate: u32,
    pub(super) parked: BTreeSet<u32>,
    pub(super) forest_origin: u32,
    pub(super) forest_disposing: bool,
    pub(super) writer: Option<Writer>,
    /// The document stands on a fork a navigation made (`forked`).
    pub(super) forked: bool,
    /// Past heads queued for `maintain_history` (`heads`).
    pub(super) heads: u32,
    /// The active reversal facts (ruling (u)).
    pub(super) facts: BTreeSet<Fact>,
}

impl State {
    fn initial() -> Self {
        Self {
            parent: vec![None],
            snapshots: vec![Snapshot::EMPTY],
            ops: vec![[StepOp::None; 2]],
            labels: vec![None],
            current: 0,
            branch: Snapshot::EMPTY,
            next_generation: 1,
            journal: BTreeMap::new(),
            head_diverged: false,
            closed: false,
            next_candidate: 1,
            parked: BTreeSet::new(),
            forest_origin: 0,
            forest_disposing: false,
            writer: None,
            forked: false,
            heads: 0,
            facts: BTreeSet::new(),
        }
    }

    pub fn nodes(&self) -> u32 {
        self.parent.len() as u32
    }

    pub fn current(&self) -> u32 {
        self.current
    }

    pub fn head_diverged(&self) -> bool {
        self.head_diverged
    }

    pub fn writer(&self) -> Option<Writer> {
        self.writer
    }

    /// The stages leave the owned graph closed and touch something.
    pub fn admits(&self, stages: &Stages) -> bool {
        self.stages_admissible(stages)
    }

    pub fn parent_of(&self, node: u32) -> Option<u32> {
        self.parent[node as usize]
    }

    pub fn op(&self, node: u32, entity: usize) -> StepOp {
        self.ops[node as usize][entity]
    }

    /// Recorded keys of either namespace and their intents, in key order (the
    /// order a run draws keys), so a replay's pick names the same key
    /// whichever surface recorded it.
    pub fn recorded(&self) -> impl Iterator<Item = (u32, &Intent)> {
        let mut recorded: Vec<_> = self.journal.iter().map(|(&(namespace, key), entry)| (key, namespace, &entry.intent)).collect();
        recorded.sort_by_key(|&(key, namespace, _)| (key, namespace));
        recorded.into_iter().map(|(key, _, intent)| (key, intent))
    }

    /// Successor `choice` (modulo their number) of `action`: the translation
    /// itself run as the system, for the no-world conformance court.
    pub fn simulate(&self, action: &Action, choice: u64) -> Option<(Outcome, State)> {
        let successors = self.transitions(action);
        let index = (choice % successors.len().max(1) as u64) as usize;
        successors.get(index).cloned()
    }
}

/// The call, what it returned and what was read afterwards, when no model
/// state allows them.
#[derive(Debug)]
pub struct Disagreement {
    pub action: Action,
    pub outcome: Outcome,
    pub observed: Observed,
    /// Every outcome the model allows from the states held before the call.
    pub allowed: BTreeSet<Outcome>,
    /// A broken model invariant, when the disagreement is the oracle's own.
    pub invariant: Option<&'static str>,
}

/// Every model state consistent with the calls so far, each with the stop
/// point of every call on one path that reaches it (`None` for a call that
/// did not stop).
pub struct Oracle {
    beliefs: BTreeMap<State, Vec<Option<&'static str>>>,
}

impl Default for Oracle {
    fn default() -> Self {
        Self {
            beliefs: BTreeMap::from([(State::initial(), Vec::new())]),
        }
    }
}

impl Oracle {
    /// One held state. Every held state has the same tree, journal and branch
    /// (they differ only in stopped-request custody), so the workload can
    /// resolve its choices against any of them.
    pub fn state(&self) -> &State {
        self.beliefs.keys().next().expect("the oracle always holds a state")
    }

    /// The stop points of one path through every call so far: the real
    /// system's stop is rarely observable when it returns, so the trace
    /// records the point a later call settled (`TreeBasedUndoTrace.tla`
    /// then follows one path, as the oracle did).
    pub fn points(&self) -> &[Option<&'static str>] {
        self.beliefs.values().next().expect("the oracle always holds a state")
    }

    /// The outcomes the model allows for `action` from the held states.
    pub fn allowed(&self, action: &Action) -> BTreeSet<Outcome> {
        self.beliefs
            .keys()
            .flat_map(|state| state.transitions(action))
            .map(|(outcome, _)| outcome)
            .collect()
    }

    /// Keep the successor states that return `outcome` and read as
    /// `observed`; disagree when none does.
    pub fn step(
        &mut self,
        action: &Action,
        outcome: &Outcome,
        observed: &Observed,
    ) -> Result<(), Box<Disagreement>> {
        let disagreement = |invariant| {
            Box::new(Disagreement {
                action: action.clone(),
                outcome: outcome.clone(),
                observed: observed.clone(),
                allowed: self.allowed(action),
                invariant,
            })
        };
        let mut next = BTreeMap::new();
        for (before, path) in &self.beliefs {
            for (allowed, state) in before.transitions(action) {
                if allowed == *outcome && state.matches(observed) {
                    let point = (allowed == Outcome::Stopped).then(|| before.stop_point(&state));
                    next.entry(state).or_insert_with(|| [path.as_slice(), &[point]].concat());
                }
            }
        }
        if next.is_empty() {
            return Err(disagreement(None));
        }
        if let Some(broken) = next.keys().find_map(State::broken_invariant) {
            return Err(disagreement(Some(broken)));
        }
        self.beliefs = next;
        Ok(())
    }
}
