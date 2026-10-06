//! Partition reuse proven exact: one seeded sequence of fact edits runs
//! through a runtime that keeps its output, and every step's run is checked
//! against a full run of the same facts in a runtime installed for that step,
//! which holds no record to reuse.

use worth_query_host::facade::application_contribution::{
    WorthQueryPartitionedComputationFullCause as FullCause,
    WorthQueryPartitionedComputationRun as Run,
};

use super::super::entry_edit::EntryFact;
use super::super::facts::RegionFault;
use super::super::region_output::{arm_own_write, OwnWrite, OwnWriteRead};
use super::*;

/// The entries a sequence edits, spread over this many regions.
const ENTRIES: usize = 32;
const REGIONS: u32 = 4;
/// Each round makes one edit of every kind, in a seeded order.
const ROUNDS: usize = 4;
const SEED: u64 = 0x9176_3c0b_5eed_0001;
/// The sets an ordinate names, the even one first.
const SETS: [&str; 2] = ["even", "odd"];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Kind {
    /// One entry's value, which only its partition gathers.
    Value,
    /// The weight of the input's set, which every even partition gathers.
    SharedWeight,
    /// One entry's region, which its item key reads.
    ItemKey,
    /// One entry's value, and a decision that reads and then writes another
    /// entry's value its own run gathered.
    OwnWrite,
    /// The same, but the decision writes the value without reading it: only
    /// the gather that carries or gathers it read it.
    BlindWrite,
    /// The scope's ordinate, which names the input's set.
    Input,
    /// One entry's value, written unchanged.
    NoOp,
    /// One entry's fault, set or cleared.
    Fault,
}

const KINDS: [Kind; 8] = [
    Kind::Value,
    Kind::SharedWeight,
    Kind::ItemKey,
    Kind::OwnWrite,
    Kind::BlindWrite,
    Kind::Input,
    Kind::NoOp,
    Kind::Fault,
];

struct Lcg(u64);
impl Lcg {
    fn below(&mut self, bound: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        usize::try_from((self.0 >> 33) % u64::try_from(bound).unwrap()).unwrap()
    }

    /// A float of either sign whose exponent lies in [-40, 40], so a sum of
    /// such floats depends on its order.
    fn value(&mut self) -> f64 {
        let mantissa = 1.0 + f64::from(u32::try_from(self.below(1 << 20)).unwrap()) / 1_048_576.0;
        let exponent = i32::try_from(self.below(81)).unwrap() - 40;
        let sign = if self.below(2) == 0 { 1.0 } else { -1.0 };
        sign * mantissa * 2_f64.powi(exponent)
    }
}

#[derive(Clone, Copy)]
struct ModelEntry {
    region: u32,
    value: f64,
    fault: bool,
}

/// The facts both runtimes hold.
struct Model {
    entries: Vec<ModelEntry>,
    weights: [f64; 2],
    odd: bool,
}

/// What a step changes: one entry fact, or the scope's ordinate.
enum Change {
    Entry(EntryEdit),
    Ordinate(u64),
}

struct Step {
    change: Change,
    /// The entry value the step's decision writes.
    own_write: Option<OwnWrite>,
}

fn number(entry: usize) -> u64 {
    u64::try_from(entry).unwrap()
}

impl Model {
    fn new(rng: &mut Lcg) -> Self {
        let entries = (0..ENTRIES)
            .map(|place| ModelEntry {
                region: u32::try_from(place).unwrap() % REGIONS,
                value: rng.value(),
                fault: false,
            })
            .collect();
        Self {
            entries,
            weights: [rng.value(), rng.value()],
            odd: false,
        }
    }

    fn set(&self) -> &'static str {
        SETS[usize::from(self.odd)]
    }

    fn seed(&self, graph: &mut Graph) {
        for (set, weight) in SETS.into_iter().zip(self.weights) {
            facts::seed_set(graph, set, weight);
        }
        for (place, entry) in self.entries.iter().enumerate() {
            let seeded = RegionEntry {
                id: number(place),
                region: entry.region,
                value: entry.value,
                work: 1,
                fault: entry.fault.then_some(RegionFault::Refuse),
            };
            seed_entry(graph, &SETS, place, seeded);
        }
    }

    /// Makes one edit of `kind` to the facts, and says how it is made.
    fn step(&mut self, kind: Kind, rng: &mut Lcg) -> Step {
        let set = self.set();
        let entry = rng.below(ENTRIES);
        let edit =
            move |fact, value| Change::Entry(EntryEdit::new(set, number(entry), fact, value));
        let mut own_write = None;
        let change = match kind {
            Kind::Value | Kind::OwnWrite | Kind::BlindWrite => {
                let value = rng.value();
                self.entries[entry].value = value;
                let read = match kind {
                    Kind::OwnWrite => Some(OwnWriteRead::Observed),
                    Kind::BlindWrite => Some(OwnWriteRead::Blind),
                    _ => None,
                };
                own_write = read.map(|read| OwnWrite {
                    number: number(rng.below(ENTRIES)),
                    bits: rng.value().to_bits(),
                    read,
                });
                edit(EntryFact::Value, value.to_bits())
            }
            Kind::NoOp => edit(EntryFact::Value, self.entries[entry].value.to_bits()),
            Kind::SharedWeight => {
                let weight = rng.value();
                self.weights[usize::from(self.odd)] = weight;
                Change::Entry(EntryEdit::new(set, 0, EntryFact::Weight, weight.to_bits()))
            }
            Kind::ItemKey => {
                let moved = u32::try_from(rng.below(3)).unwrap() + 1;
                let region = (self.entries[entry].region + moved) % REGIONS;
                self.entries[entry].region = region;
                edit(EntryFact::Region, u64::from(region))
            }
            Kind::Fault => {
                let faulted = (0..ENTRIES).find(|&e| self.entries[e].fault);
                let (entry, fault) = faulted.map_or((entry, true), |faulted| (faulted, false));
                self.entries[entry].fault = fault;
                let code = RegionFault::code(fault.then_some(RegionFault::Refuse));
                Change::Entry(EntryEdit::new(set, number(entry), EntryFact::Fault, code))
            }
            Kind::Input => {
                self.odd = !self.odd;
                Change::Ordinate(if self.odd { ODD_Y } else { EVEN_Y })
            }
        };
        Step { change, own_write }
    }
}

/// The run of a runtime installed over `model`, which holds no record.
fn full_run(model: &Model, own_write: Option<OwnWrite>) -> OracleRun {
    let application = install(|graph| model.seed(graph));
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    if model.odd {
        adjust(&request, &application, ODD_Y, 1);
    }
    arm_own_write(own_write);
    let (contacts, mut runs) = demand(&request, &application);
    arm_own_write(None);
    assert_eq!(
        (contacts, runs.len()),
        (1, 1),
        "a fresh runtime runs the producer once"
    );
    let run = runs.remove(0);
    assert!(
        run.runs
            .iter()
            .all(|run| *run == Run::Full(FullCause::NoPriorRecord)),
        "a fresh runtime has no record to reuse: {run:?}"
    );
    run
}

/// How a step's run must run when it and the run before it completed.
fn expected(kind: Kind) -> Option<Run> {
    match kind {
        Kind::Value | Kind::SharedWeight | Kind::OwnWrite | Kind::BlindWrite | Kind::NoOp => {
            Some(Run::Incremental)
        }
        Kind::ItemKey => Some(Run::Full(FullCause::PartitionerRebuilt)),
        Kind::Input => Some(Run::Full(FullCause::InputChanged)),
        Kind::Fault => None,
    }
}

/// One demand of the seeded sequence: the facts it was made over, the edit
/// before it, none for the first, the value its decision writes, and what
/// it ran.
pub(super) struct Demanded<'model> {
    model: &'model Model,
    kind: Option<Kind>,
    own_write: Option<OwnWrite>,
    pub(super) contacts: usize,
    pub(super) runs: Vec<OracleRun>,
}

/// Runs the seeded sequence of edits through one runtime that keeps its
/// output, and hands `each` every demand in order. The caller holds the
/// checkpoint recovery guard.
pub(super) fn sequence(mut each: impl FnMut(&str, Demanded<'_>)) {
    let mut rng = Lcg(SEED);
    let mut model = Model::new(&mut rng);
    let application = install(|graph| model.seed(graph));
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let (contacts, runs) = demand(&request, &application);
    let first = Demanded {
        model: &model,
        kind: None,
        own_write: None,
        contacts,
        runs,
    };
    each("the first demand", first);
    let mut command = 0_u64;
    for round in 0..ROUNDS {
        let mut kinds = KINDS;
        for place in (1..kinds.len()).rev() {
            kinds.swap(place, rng.below(place + 1));
        }
        for kind in kinds {
            let step = model.step(kind, &mut rng);
            command += 1;
            match step.change {
                Change::Entry(change) => edit(&request, &application, change, command),
                Change::Ordinate(y) => adjust(&request, &application, y, command),
            }
            arm_own_write(step.own_write);
            let (contacts, runs) = demand(&request, &application);
            arm_own_write(None);
            let demanded = Demanded {
                model: &model,
                kind: Some(kind),
                own_write: step.own_write,
                contacts,
                runs,
            };
            each(&format!("round {round}, {kind:?}"), demanded);
            if let Some(write) = step.own_write {
                let entry = usize::try_from(write.number).unwrap();
                model.entries[entry].value = f64::from_bits(write.bits);
            }
        }
    }
}

#[test]
fn every_reused_run_equals_a_full_run_of_the_same_facts() {
    let _guard = checkpoint_recovery_test_guard();
    let mut last: Option<OracleRun> = None;
    let (mut incremental, mut full) = (0_usize, 0_usize);
    sequence(|at, mut demanded| {
        let reference = full_run(demanded.model, demanded.own_write);
        let Some(last_run) = last.as_ref() else {
            assert_eq!((demanded.contacts, demanded.runs.len()), (1, 1));
            let first = demanded.runs.remove(0);
            assert_eq!(first.outcome, reference.outcome);
            last = Some(first);
            return;
        };
        if demanded.runs.is_empty() {
            assert_eq!(
                demanded.contacts, 0,
                "{at}: a demand that ran nothing kept its output"
            );
            assert_eq!(
                reference.outcome, last_run.outcome,
                "{at}: the kept output is current"
            );
            return;
        }
        assert_eq!(
            demanded.runs.len(),
            1,
            "{at}: one decision runs the totals once"
        );
        let run = demanded.runs.remove(0);
        assert_eq!(
            run.outcome, reference.outcome,
            "{at}: reuse equals a full run"
        );
        match run.runs.as_slice() {
            [Run::Incremental] => incremental += 1,
            [Run::Full(_)] => full += 1,
            _ => {}
        }
        if let (Some(expected), true) = (
            demanded.kind.and_then(expected),
            last_run.outcome.is_ok() && run.outcome.is_ok(),
        ) {
            assert_eq!(run.runs, [expected], "{at}: the run's kind");
        }
        last = Some(run);
    });
    assert!(
        incremental > full,
        "most steps reuse partitions: {incremental} incremental, {full} full"
    );
}
