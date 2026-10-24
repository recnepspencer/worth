//! The edits of the seeded sequence, and the facts they are made to: every
//! fact an entry or a set holds, the scope's ordinate, and the entries
//! themselves, made, deleted, moved between regions or trading numbers.

use std::collections::BTreeSet;

use super::super::super::entry_edit::EntryFact;
use super::super::super::facts::RegionFault;
use super::super::super::own_write::{OwnWrite, OwnWriteRead};
use super::*;

/// The entries a sequence seeds, spread over this many regions, and the
/// numbers it may make entries under.
const ENTRIES: usize = 32;
const NUMBERS: usize = ENTRIES + 8;
const REGIONS: u32 = 4;
/// The sets an ordinate names, the even one first.
pub(in super::super) const SETS: [&str; 2] = ["even", "odd"];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in super::super) enum Kind {
    /// One entry's value, which only its partition gathers.
    Value,
    SignedZero,
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
    /// A value edit at each real runtime absence boundary.
    Evict,
    Unretained,
    Several,
    /// The faulted entry's fault cleared.
    Repair,
}

/// Every kind a round shuffles; `Repair` follows each `Fault`.
pub(in super::super) const KINDS: [Kind; 18] = [
    Kind::Value,
    Kind::SignedZero,
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
    Kind::Evict,
    Kind::Unretained,
    Kind::Several,
];

pub(in super::super) struct Lcg(pub(in super::super) u64);
mod group_move;
mod values;

#[derive(Clone, Copy)]
struct ModelEntry {
    binding: u64,
    region: u32,
    incoming: bool,
    value: f64,
    work: u64,
    fault: bool,
}

/// The facts both runtimes hold: the entry each number names, if any.
#[derive(Clone)]
pub(in super::super) struct Model {
    entries: Vec<Option<ModelEntry>>,
    next_binding: u64,
    weights: [f64; 2],
    pub(in super::super) odd: bool,
}

/// What a step changes: entry facts or entries, or the scope's ordinate.
#[derive(Clone)]
pub(in super::super) enum Change {
    Entry(EntryEdit),
    Ordinate(u64),
}

pub(in super::super) struct Step {
    /// The edits the step commits, in order.
    pub(in super::super) changes: Vec<Change>,
    /// The entry value the step's decision writes.
    pub(in super::super) own_write: Option<OwnWrite>,
}

fn number(entry: usize) -> u64 {
    u64::try_from(entry).unwrap()
}

impl Model {
    /// The seeded entries, the first of them heavy.
    pub(in super::super) fn new(rng: &mut Lcg) -> Self {
        Self::bounded(rng, ENTRIES, NUMBERS, TOTALS_WORK)
    }

    pub(in super::super) fn bounded(
        rng: &mut Lcg,
        count: usize,
        numbers: usize,
        work: usize,
    ) -> Self {
        let heavy = work as u64 / 2 + 1;
        let entries = (0..numbers)
            .map(|place| {
                (place < count).then(|| ModelEntry {
                    binding: place as u64,
                    region: u32::try_from(place).unwrap() % REGIONS,
                    incoming: place == 0,
                    value: rng.value(),
                    work: if place == 0 { heavy } else { 1 },
                    fault: false,
                })
            })
            .collect();
        Self {
            entries,
            next_binding: numbers as u64,
            weights: [rng.value(), rng.value()],
            odd: false,
        }
    }

    fn set(&self) -> &'static str {
        SETS[usize::from(self.odd)]
    }

    pub(in super::super) fn seed(&self, graph: &mut Graph) {
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
        (0..self.entries.len())
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
    pub(in super::super) fn holds(&self, write: OwnWrite) -> bool {
        let entry = self.entries[usize::try_from(write.number).unwrap()];
        entry.is_some_and(|entry| entry.value.to_bits() == write.bits)
    }

    pub(in super::super) fn written(&mut self, write: OwnWrite) {
        let entry = usize::try_from(write.number).unwrap();
        self.entries[entry].as_mut().unwrap().value = f64::from_bits(write.bits);
    }

    fn moved(&mut self, entry: usize, region: u32) -> Change {
        self.entries[entry].as_mut().unwrap().region = region;
        let edit = EntryEdit::new("", number(entry), EntryFact::Region, u64::from(region));
        Change::Entry(edit)
    }

    fn created(&mut self, entry: usize, region: u32, value: f64, work: u64) -> Change {
        self.next_binding += 1;
        self.entries[entry] = Some(ModelEntry {
            binding: self.next_binding,
            region,
            incoming: false,
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

    pub(in super::super) fn first_number(&self) -> u64 {
        number(self.held(false)[0])
    }
}

mod basis;
mod counts;
mod edit;

mod laws;
