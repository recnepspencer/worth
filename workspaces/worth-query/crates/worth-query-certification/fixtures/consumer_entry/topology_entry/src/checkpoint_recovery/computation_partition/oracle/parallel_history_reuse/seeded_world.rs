//! Declared physical items and edits; no observed values enter this world.
use super::*;
use crate::checkpoint_recovery::parallel_history::reuse_cases::{
    ReuseItem, ITEM_WORK, SHARED_WORK,
};
pub(super) const SEED: u64 = 0x9176_378a_0001;
pub(super) fn facts() -> ReuseFacts {
    ReuseFacts {
        items: (0..5)
            .map(|index| ReuseItem {
                number: if index == 4 { 8 } else { index as u64 },
                key: index as u32,
                value: 1 + (SEED.rotate_right(index as u32 * 7) % 64),
                work: ITEM_WORK[index],
                member: index < 4,
                exists: true,
                fault: false,
            })
            .collect(),
        weights: [3, 14],
        shared_work: SHARED_WORK,
        odd: false,
    }
}
pub(super) fn seed(model: &ReuseFacts, graph: &mut Graph) {
    for (index, set) in ["even", "odd"].iter().enumerate() {
        facts::seed_set(graph, set, model.weights[index] as f64);
    }
    facts::seed_set(graph, "outside", 0.0);
    for (index, item) in model.items.iter().enumerate() {
        if !item.exists {
            continue;
        }
        seed_entry(
            graph,
            if item.member {
                &["even", "odd"]
            } else {
                &["outside"]
            },
            index,
            RegionEntry {
                id: item.number,
                region: item.key,
                value: item.value as f64,
                work: item.work as usize,
                fault: item.fault.then_some(RegionFault::Refuse),
            },
        );
    }
}
#[derive(Clone, Copy)]
pub(super) enum Change {
    Gather,
    Shared,
    Key,
    Input,
    Fault,
    Join,
    Leave,
    DigestSwap,
    Replacement,
}
impl Change {
    pub(super) fn apply(self, model: &mut ReuseFacts) -> Option<EntryEdit> {
        let item = &mut model.items[0];
        match self {
            Self::Gather => {
                item.value += 7;
                Some(EntryEdit::new(
                    "even",
                    item.number,
                    EntryFact::Value,
                    (item.value as f64).to_bits(),
                ))
            }
            Self::Shared => {
                model.weights[0] += 11;
                Some(EntryEdit::new(
                    "even",
                    0,
                    EntryFact::Weight,
                    (model.weights[0] as f64).to_bits(),
                ))
            }
            Self::Key => {
                item.key = 3;
                Some(EntryEdit::new("even", item.number, EntryFact::Region, 3))
            }
            Self::Input => {
                model.odd = true;
                None
            }
            Self::Fault => {
                item.fault = true;
                Some(EntryEdit::new("even", item.number, EntryFact::Fault, 1))
            }
            Self::Join => {
                let item = ReuseItem {
                    number: 4,
                    key: 0,
                    value: 19,
                    work: ITEM_WORK[5],
                    member: true,
                    exists: true,
                    fault: false,
                };
                let edit = EntryEdit::create(
                    &["even", "odd"],
                    item.number,
                    item.key,
                    (item.value as f64).to_bits(),
                    item.work,
                );
                model.items.push(item);
                Some(edit)
            }
            Self::Leave => {
                item.member = false;
                item.exists = false;
                Some(EntryEdit::delete(item.number))
            }
            Self::DigestSwap | Self::Replacement => {
                let other = if matches!(self, Self::DigestSwap) {
                    1
                } else {
                    4
                };
                let edit = EntryEdit::swap(model.items[0].number, model.items[other].number);
                let number = model.items[0].number;
                model.items[0].number = model.items[other].number;
                model.items[other].number = number;
                Some(edit)
            }
        }
    }
}
