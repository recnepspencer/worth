//! Runs one seed's choices against a system and the oracle, step by step.
use super::reference::{Action, Disagreement, Observed, Oracle, Outcome, State};
use super::trace::Trace;
use super::workload::{resolve, Keys, Op, SplitMix64};

/// What the harness drives: the real document, or the model itself.
pub trait System {
    /// Make the call and report what it returned, in the model's terms.
    fn act(&mut self, action: &Action) -> Outcome;
    /// Read the document and its branch with a fresh request, paging
    /// `history_page` and `redo_choices` at `limit` entries.
    fn observe(&mut self, limit: u32) -> Observed;
    /// Steps the document holds, the root included.
    fn nodes(&self) -> u32;
    /// Give back what a run stopped by a disagreement still holds: shrink
    /// replays share one host, and an open document keeps its capacity.
    fn release(&mut self);
}

/// The first call the oracle disagreed with.
#[derive(Debug)]
pub struct Failure {
    /// Index of the choice in the seed's sequence (`ops.len()` for the
    /// closing call).
    pub step: usize,
    pub disagreement: Box<Disagreement>,
}

/// Run `ops`, then close the document. Every call is checked against the
/// oracle before the next one; the trace, if any, records every call.
pub fn run<S: System>(system: &mut S, ops: &[Op], trace: Option<&mut Trace>) -> Result<(), Failure> {
    let result = steps(system, ops, trace);
    if result.is_err() {
        system.release();
    }
    result
}

fn steps<S: System>(system: &mut S, ops: &[Op], mut trace: Option<&mut Trace>) -> Result<(), Failure> {
    let mut oracle = Oracle::default();
    let mut keys = Keys::default();
    let mut call = |step: usize, action: Action, oracle: &mut Oracle| {
        let outcome = system.act(&action);
        // Page sizes cycle 1..=4 so every limit meets every tree shape.
        let observed = system.observe(1 + (step % 4) as u32);
        if let Some(trace) = trace.as_deref_mut() {
            trace.push(&action, &outcome, &observed, system.nodes());
        }
        oracle
            .step(&action, &outcome, &observed)
            .map_err(|disagreement| Failure { step, disagreement })
    };
    for (step, op) in ops.iter().enumerate() {
        if let Some(action) = resolve(op, oracle.state(), &mut keys) {
            call(step, action, &mut oracle)?;
        }
    }
    // Close needs the document back: a writer still open is discarded.
    if oracle.state().writer().is_some() {
        call(ops.len(), Action::Discard, &mut oracle)?;
    }
    call(ops.len(), Action::Close, &mut oracle)?;
    if let Some(trace) = trace {
        trace.set_points(oracle.points());
    }
    Ok(())
}

/// The model run as the system: each call takes one of the successors the
/// model allows, chosen by its own seeded stream. Agreement checks the
/// oracle's bookkeeping, and its traces check the translation against the
/// TLA+ model under TLC.
pub struct ModelSystem {
    state: State,
    choices: SplitMix64,
}

impl ModelSystem {
    pub fn new(seed: u64) -> Self {
        Self {
            state: Oracle::default().state().clone(),
            choices: SplitMix64(!seed),
        }
    }
}

impl System for ModelSystem {
    fn act(&mut self, action: &Action) -> Outcome {
        let (outcome, next) = self
            .state
            .simulate(action, self.choices.next())
            .unwrap_or_else(|| panic!("the workload resolved a call the model has no step for: {action:?}"));
        self.state = next;
        outcome
    }

    fn observe(&mut self, limit: u32) -> Observed {
        self.state.observed(limit)
    }

    fn nodes(&self) -> u32 {
        self.state.nodes()
    }

    fn release(&mut self) {}
}

/// Greedy one-at-a-time deletion: the shortest sequence found that still
/// fails. Every choice resolves against the state it meets, so any
/// subsequence runs.
pub fn shrink(mut ops: Vec<Op>, mut fails: impl FnMut(&[Op]) -> bool) -> Vec<Op> {
    loop {
        let before = ops.len();
        let mut index = ops.len();
        while index > 0 {
            index -= 1;
            let mut candidate = ops.clone();
            candidate.remove(index);
            if fails(&candidate) {
                ops = candidate;
            }
        }
        if ops.len() == before {
            return ops;
        }
    }
}
