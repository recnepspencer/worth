//! The edits of the seeded sequence, and the facts they are made to: every
//! fact an entry or a set holds, the scope's ordinate, and the entries
//! themselves, made, deleted, moved between regions or trading numbers.

use std::collections::BTreeSet;

use super::super::super::entry_edit::EntryFact;
use super::super::super::facts::RegionFault;
use super::super::super::region_output::{OwnWrite, OwnWriteRead};
use super::*;

/// The entries a sequence seeds, spread over this many regions, and the
/// numbers it may make entries under.
const ENTRIES: usize = 32;
const NUMBERS: usize = ENTRIES + 8;
const REGIONS: u32 = 4;
/// The work of a heavy entry. One fits the declared work beside every other
/// entry; a second does not, so the run stops at the work ceiling in the
/// later of their regions.
const HEAVY: u64 = TOTALS_WORK as u64 / 2 + 1;
/// The sets an ordinate names, the even one first.
pub(super) const SETS: [&str; 2] = ["even", "odd"];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Kind {
    /// One entry's value, which only its partition gathers.
    Value,
    /// The weight of the input's set, which every even partition gathers.
    SharedWeight,
    /// One entry's region, which its item key reads: it moves between keys.
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
    /// One entry's fault set: the run refuses. The next edit is its
    /// `Repair`.
    Fault,
    /// A new entry, under a number no entry holds.
    Create,
    Delete,
    /// Every entry of one region moved to another: the region's key empties.
    EmptyKey,
    /// One entry moved to a region no entry lies in: a new key.
    NewKey,
    /// An entry deleted, then one made under its number.
    DeleteThenCreate,
    /// Two entries trade numbers: each number names the other entity.
    Swap,
    /// A second heavy entry, in a region the first does not lie in: the run
    /// stops at the work ceiling. The next edit is its `Relief`.
    Ceiling,
    /// The second heavy entry deleted.
    Relief,
    /// The faulted entry's fault cleared.
    Repair,
}

/// Every kind a round shuffles; `Relief` follows each `Ceiling`, and
/// `Repair` each `Fault`.
pub(super) const KINDS: [Kind; 15] = [
    Kind::Value,
    Kind::SharedWeight,
    Kind::ItemKey,
    Kind::OwnWrite,
    Kind::BlindWrite,
    Kind::Input,
    Kind::NoOp,
    Kind::Fault,
    Kind::Create,
    Kind::Delete,
    Kind::EmptyKey,
    Kind::NewKey,
    Kind::DeleteThenCreate,
    Kind::Swap,
    Kind::Ceiling,
];

pub(super) struct Lcg(pub(super) u64);
impl Lcg {
    pub(super) fn below(&mut self, bound: usize) -> usize {
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

    /// One of the seeded regions.
    fn region(&mut self) -> u32 {
        u32::try_from(self.below(usize::try_from(REGIONS).unwrap())).unwrap()
    }

    fn pick<T: Copy>(&mut self, from: &[T]) -> T {
        from[self.below(from.len())]
    }
}

#[derive(Clone, Copy, PartialEq)]
struct ModelEntry {
    region: u32,
    value: f64,
    work: u64,
    fault: bool,
}

/// The facts both runtimes hold: the entry each number names, if any.
#[derive(Clone, PartialEq)]
pub(super) struct Model {
    entries: Vec<Option<ModelEntry>>,
    weights: [f64; 2],
    pub(super) odd: bool,
    /// The heavy entry the last `Ceiling` made.
    ceiling: Option<usize>,
}

/// What a step changes: entry facts or entries, or the scope's ordinate.
pub(super) enum Change {
    Entry(EntryEdit),
    Ordinate(u64),
}

pub(super) struct Step {
    /// The edits the step commits, in order.
    pub(super) changes: Vec<Change>,
    /// The entry value the step's decision writes.
    pub(super) own_write: Option<OwnWrite>,
}

fn number(entry: usize) -> u64 {
    u64::try_from(entry).unwrap()
}

impl Model {
    /// The seeded entries, the first of them heavy.
    pub(super) fn new(rng: &mut Lcg) -> Self {
        let entries = (0..NUMBERS)
            .map(|place| {
                (place < ENTRIES).then(|| ModelEntry {
                    region: u32::try_from(place).unwrap() % REGIONS,
                    value: rng.value(),
                    work: if place == 0 { HEAVY } else { 1 },
                    fault: false,
                })
            })
            .collect();
        Self {
            entries,
            weights: [rng.value(), rng.value()],
            odd: false,
            ceiling: None,
        }
    }

    fn set(&self) -> &'static str {
        SETS[usize::from(self.odd)]
    }

    pub(super) fn seed(&self, graph: &mut Graph) {
        for (set, weight) in SETS.into_iter().zip(self.weights) {
            facts::seed_set(graph, set, weight);
        }
        for (place, entry) in self.entries.iter().enumerate() {
            let Some(entry) = entry else { continue };
            let seeded = RegionEntry {
                id: number(place),
                region: entry.region,
                value: entry.value,
                work: usize::try_from(entry.work).unwrap(),
                fault: entry.fault.then_some(RegionFault::Refuse),
            };
            seed_entry(graph, &SETS, place, seeded);
        }
    }

    /// The numbers that name an entry, the light ones only when `light`.
    fn held(&self, light: bool) -> Vec<usize> {
        let held = self.entries.iter().enumerate();
        held.filter(|(_, entry)| entry.is_some_and(|entry| !light || entry.work == 1))
            .map(|(place, _)| place)
            .collect()
    }

    fn free(&self) -> Vec<usize> {
        (0..NUMBERS)
            .filter(|&place| self.entries[place].is_none())
            .collect()
    }

    fn regions(&self) -> BTreeSet<u32> {
        self.entries
            .iter()
            .flatten()
            .map(|entry| entry.region)
            .collect()
    }

    /// Whether the entry `write` names already holds the value it writes.
    pub(super) fn holds(&self, write: OwnWrite) -> bool {
        let entry = self.entries[usize::try_from(write.number).unwrap()];
        entry.is_some_and(|entry| entry.value.to_bits() == write.bits)
    }

    pub(super) fn written(&mut self, write: OwnWrite) {
        let entry = usize::try_from(write.number).unwrap();
        self.entries[entry].as_mut().unwrap().value = f64::from_bits(write.bits);
    }

    fn moved(&mut self, entry: usize, region: u32) -> Change {
        self.entries[entry].as_mut().unwrap().region = region;
        let edit = EntryEdit::new("", number(entry), EntryFact::Region, u64::from(region));
        Change::Entry(edit)
    }

    fn created(&mut self, entry: usize, region: u32, value: f64, work: u64) -> Change {
        self.entries[entry] = Some(ModelEntry {
            region,
            value,
            work,
            fault: false,
        });
        let bits = value.to_bits();
        Change::Entry(EntryEdit::create(&SETS, number(entry), region, bits, work))
    }

    fn deleted(&mut self, entry: usize) -> Change {
        self.entries[entry] = None;
        Change::Entry(EntryEdit::delete(number(entry)))
    }

    /// Makes one edit of `kind` to the facts, and says how it is made.
    pub(super) fn step(&mut self, kind: Kind, rng: &mut Lcg) -> Step {
        let set = self.set();
        let entry = rng.pick(&self.held(false));
        let edit = move |fact, value| EntryEdit::new(set, number(entry), fact, value);
        let mut own_write = None;
        let changes = match kind {
            Kind::Value | Kind::OwnWrite | Kind::BlindWrite => {
                let value = rng.value();
                self.entries[entry].as_mut().unwrap().value = value;
                let read = match kind {
                    Kind::OwnWrite => Some(OwnWriteRead::Observed),
                    Kind::BlindWrite => Some(OwnWriteRead::Blind),
                    _ => None,
                };
                let written = rng.pick(&self.held(false));
                own_write = read.map(|read| OwnWrite {
                    number: number(written),
                    bits: rng.value().to_bits(),
                    read,
                });
                vec![Change::Entry(edit(EntryFact::Value, value.to_bits()))]
            }
            Kind::NoOp => {
                let value = self.entries[entry].unwrap().value;
                vec![Change::Entry(edit(EntryFact::Value, value.to_bits()))]
            }
            Kind::SharedWeight => {
                let weight = rng.value();
                self.weights[usize::from(self.odd)] = weight;
                let edit = EntryEdit::new(set, 0, EntryFact::Weight, weight.to_bits());
                vec![Change::Entry(edit)]
            }
            Kind::ItemKey => {
                let moved = u32::try_from(rng.below(3)).unwrap() + 1;
                let region = (self.entries[entry].unwrap().region + moved) % REGIONS;
                vec![self.moved(entry, region)]
            }
            Kind::Fault | Kind::Repair => {
                let fault = kind == Kind::Fault;
                let faulted = self.held(false).into_iter();
                let mut faulted = faulted.filter(|&e| self.entries[e].unwrap().fault);
                let entry = if fault {
                    entry
                } else {
                    faulted.next().expect("a repair follows its fault")
                };
                self.entries[entry].as_mut().unwrap().fault = fault;
                let code = RegionFault::code(fault.then_some(RegionFault::Refuse));
                let edit = EntryEdit::new(set, number(entry), EntryFact::Fault, code);
                vec![Change::Entry(edit)]
            }
            Kind::Input => {
                self.odd = !self.odd;
                vec![Change::Ordinate(if self.odd { ODD_Y } else { EVEN_Y })]
            }
            Kind::Create => {
                let created = rng.pick(&self.free());
                let region = rng.region();
                vec![self.created(created, region, rng.value(), 1)]
            }
            Kind::Delete => vec![self.deleted(rng.pick(&self.held(true)))],
            Kind::EmptyKey => {
                let regions = self.regions().into_iter().collect::<Vec<_>>();
                let emptied = rng.pick(&regions);
                let others = regions.iter().copied().filter(|r| *r != emptied);
                let into = others.collect::<Vec<_>>();
                let into = if into.is_empty() {
                    emptied + 1
                } else {
                    rng.pick(&into)
                };
                let held = self.held(false).into_iter();
                let emptying = held.filter(|&e| self.entries[e].unwrap().region == emptied);
                let emptying = emptying.collect::<Vec<_>>();
                emptying.into_iter().map(|e| self.moved(e, into)).collect()
            }
            Kind::NewKey => {
                let regions = self.regions();
                let unused = (0..2 * REGIONS).filter(|r| !regions.contains(r));
                let unused = unused.collect::<Vec<_>>();
                let region = if unused.is_empty() {
                    regions.last().unwrap() + 1
                } else {
                    rng.pick(&unused)
                };
                vec![self.moved(entry, region)]
            }
            Kind::DeleteThenCreate => {
                let remade = rng.pick(&self.held(true));
                let region = rng.region();
                let deleted = self.deleted(remade);
                vec![deleted, self.created(remade, region, rng.value(), 1)]
            }
            Kind::Swap => {
                let held = self.held(false);
                let other = held[(held.binary_search(&entry).unwrap() + 1) % held.len()];
                self.entries.swap(entry, other);
                vec![Change::Entry(EntryEdit::swap(number(entry), number(other)))]
            }
            Kind::Ceiling => {
                let heavy = self.held(false).into_iter();
                let heavy = heavy.filter(|&e| self.entries[e].unwrap().work == HEAVY);
                let taken = heavy.map(|e| self.entries[e].unwrap().region);
                let taken = taken.collect::<BTreeSet<_>>();
                let free = (0..REGIONS)
                    .filter(|r| !taken.contains(r))
                    .collect::<Vec<_>>();
                let region = if free.is_empty() { 0 } else { rng.pick(&free) };
                let made = rng.pick(&self.free());
                self.ceiling = Some(made);
                vec![self.created(made, region, rng.value(), HEAVY)]
            }
            Kind::Relief => {
                let made = self.ceiling.take().expect("a relief follows its ceiling");
                vec![self.deleted(made)]
            }
        };
        Step { changes, own_write }
    }
}
