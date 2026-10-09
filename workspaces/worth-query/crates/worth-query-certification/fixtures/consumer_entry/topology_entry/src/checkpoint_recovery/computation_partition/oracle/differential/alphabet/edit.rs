//! Seeded transitions of the declared edit alphabet.
use super::*;
impl Model {
    pub(in super::super::super) fn observation_release(&mut self) -> Step {
        let mut changes = Vec::new();
        for number in self.held(false) {
            let entry = self.entries[number].unwrap();
            if entry.incoming {
                changes.push(self.deleted(number));
                changes.push(self.created(number, entry.region, entry.value, entry.work));
            }
        }
        Step {
            changes,
            own_write: None,
        }
    }

    pub(in super::super::super) fn observation_trim(&mut self) -> Step {
        let keep = self.held(true).into_iter().take(2).collect::<Vec<_>>();
        let mut changes = self
            .held(false)
            .into_iter()
            .filter(|number| !keep.contains(number))
            .map(|number| self.deleted(number))
            .collect::<Vec<_>>();
        if keep.len() == 1 {
            let region = self.entries[keep[0]].unwrap().region + 1;
            changes.push(self.created(self.free()[0], region, 0.0, 1));
        }
        Step {
            changes,
            own_write: None,
        }
    }

    pub(in super::super::super) fn wide_entries(&mut self) -> Vec<EntryEdit> {
        let mut sets = SETS.iter().map(|set| set.to_string()).collect::<Vec<_>>();
        sets.extend((0..OBSERVATION_SETS).map(|i| format!("incoming-{i}")));
        let names = sets.iter().map(String::as_str).collect::<Vec<_>>();
        self.held(false)
            .into_iter()
            .map(|number| {
                self.next_binding += 1;
                let entry = self.entries[number].as_mut().unwrap();
                entry.binding = self.next_binding;
                entry.incoming = true;
                EntryEdit::create(
                    &names,
                    number as u64,
                    entry.region,
                    entry.value.to_bits(),
                    entry.work,
                )
            })
            .collect()
    }

    pub(in super::super::super) fn edit_observation_item(
        &mut self,
        number: u64,
        rng: &mut Lcg,
    ) -> Step {
        let entry = self.entries[number as usize].as_mut().unwrap();
        let mut value = rng.value();
        while value.to_bits() == entry.value.to_bits() {
            value = rng.value();
        }
        entry.value = value;
        Step {
            changes: vec![Change::Entry(EntryEdit::new(
                self.set(),
                number,
                EntryFact::Value,
                value.to_bits(),
            ))],
            own_write: None,
        }
    }

    pub(in super::super::super) fn step(&mut self, kind: Kind, rng: &mut Lcg) -> Step {
        let set = self.set();
        let entry = rng.pick(&self.held(false));
        let edit = move |fact, value| EntryEdit::new(set, number(entry), fact, value);
        let mut own_write = None;
        let changes = match kind {
            Kind::SignedZero => {
                let value = if self.entries[entry].unwrap().value.to_bits() == (-0.0_f64).to_bits()
                {
                    0.0_f64
                } else {
                    -0.0_f64
                };
                self.entries[entry].as_mut().unwrap().value = value;
                vec![Change::Entry(edit(EntryFact::Value, value.to_bits()))]
            }
            Kind::Value
            | Kind::OwnWrite
            | Kind::BlindWrite
            | Kind::Evict
            | Kind::Unretained
            | Kind::ObservationOverBudget
            | Kind::Several => {
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
            Kind::Delete => {
                let deleted = rng.pick(&self.held(true));
                if self.held(false).len() == 1 {
                    let made = rng.pick(&self.free());
                    let created = self.created(made, rng.region(), rng.value(), 1);
                    vec![created, self.deleted(deleted)]
                } else {
                    vec![self.deleted(deleted)]
                }
            }
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
                let heavy = heavy.filter(|&e| self.entries[e].unwrap().work == self.heavy);
                let taken = heavy.map(|e| self.entries[e].unwrap().region);
                let taken = taken.collect::<BTreeSet<_>>();
                let free = (0..REGIONS)
                    .filter(|r| !taken.contains(r))
                    .collect::<Vec<_>>();
                let region = if free.is_empty() { 0 } else { rng.pick(&free) };
                let made = rng.pick(&self.free());
                self.ceiling = Some(made);
                vec![self.created(made, region, rng.value(), self.heavy)]
            }
            Kind::Relief => {
                let made = self.ceiling.take().expect("a relief follows its ceiling");
                vec![self.deleted(made)]
            }
        };
        Step { changes, own_write }
    }
}
