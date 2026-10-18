//! Authored live-group moves with fixed unrelated populations.
use super::*;
impl Model {
    pub(in super::super::super) fn live_group_move(unrelated: usize) -> Self {
        let entries = (0..unrelated + 3)
            .map(|place| {
                Some(ModelEntry {
                    binding: place as u64,
                    region: match place {
                        0 | 1 => 1,
                        2 => 3,
                        _ => 1000 + place as u32,
                    },
                    incoming: false,
                    value: match place {
                        0 => 2.0,
                        1 => 3.0,
                        2 => 5.0,
                        _ => 1.0,
                    },
                    work: 1,
                    fault: false,
                })
            })
            .collect();
        Self {
            entries,
            next_binding: (unrelated + 3) as u64,
            heavy: TOTALS_WORK as u64 + 1,
            weights: [-0.0, -0.0],
            odd: false,
            ceiling: None,
        }
    }
    pub(in super::super::super) fn move_first_to(&mut self, region: u32) -> Change {
        self.entries[0].as_mut().unwrap().region = region;
        Change::Entry(EntryEdit::new(
            "even",
            0,
            EntryFact::Region,
            u64::from(region),
        ))
    }
    pub(in super::super::super) fn set_first_value(&mut self, value: f64) -> Change {
        self.entries[0].as_mut().unwrap().value = value;
        Change::Entry(EntryEdit::new("even", 0, EntryFact::Value, value.to_bits()))
    }
}
