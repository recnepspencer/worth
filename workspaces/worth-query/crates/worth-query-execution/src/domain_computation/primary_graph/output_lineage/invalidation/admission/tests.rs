use std::{cell::Cell, cmp::Ordering, rc::Rc};

use im::OrdMap;
use worth_foundational::facade::AspectKey;
use worth_relational::facade::{
    identity::{EntityId, PartitionId},
    mvcc::{CompanionPreflightBudget, CompanionPreflightStop},
};

use super::{
    super::{fact_key::FactPostingKey, InvalidationEditAdmission},
    index_capacity, IndexAdmission,
};

#[derive(Clone)]
struct CountedKey {
    value: &'static str,
    comparisons: Rc<Cell<usize>>,
}

impl PartialEq for CountedKey {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for CountedKey {}

impl PartialOrd for CountedKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CountedKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.comparisons.set(self.comparisons.get() + 1);
        self.value.cmp(other.value)
    }
}

#[derive(Default)]
struct WorkMeter(u64);

impl IndexAdmission for WorkMeter {
    fn work(&mut self, visits: u64) -> Result<(), CompanionPreflightStop> {
        self.0 = self.0.checked_add(visits).unwrap();
        Ok(())
    }

    fn bytes(&mut self, _bytes: u64) -> Result<(), CompanionPreflightStop> {
        Ok(())
    }

    // This court measures every visit, physical or declared.
    fn navigation(&mut self, units: u64) -> Result<(), CompanionPreflightStop> {
        self.work(units)
    }
}

#[test]
fn vacant_im_root_has_no_key_comparison_and_nonempty_key_cost_tracks_payload() {
    let comparisons = Rc::new(Cell::new(0));
    let key = CountedKey {
        value: "key",
        comparisons: Rc::clone(&comparisons),
    };
    let mut map = OrdMap::new();
    assert!(map.get(&key).is_none());
    map.insert(key.clone(), ());
    assert_eq!(comparisons.get(), 0);
    assert!(map.get(&key).is_some());
    assert!(comparisons.get() > 0);

    let entity = EntityId::new(PartitionId::main(), 1, 1);
    let short = FactPostingKey::AspectRevision {
        entity,
        aspect: AspectKey::new("a").unwrap(),
    };
    let long = FactPostingKey::AspectRevision {
        entity,
        aspect: AspectKey::new("a".repeat(40)).unwrap(),
    };
    let mut vacant_short = WorkMeter::default();
    let mut vacant_long = WorkMeter::default();
    vacant_short.key_read(&short, 0).unwrap();
    vacant_long.key_read(&long, 0).unwrap();
    assert_eq!(vacant_short.0, vacant_long.0);

    let mut occupied_short = WorkMeter::default();
    let mut occupied_long = WorkMeter::default();
    occupied_short.key_read(&short, 1).unwrap();
    occupied_long.key_read(&long, 1).unwrap();
    assert!(occupied_long.0 > occupied_short.0);
}

/// A fresh build is admitted its own growth, no more: the meter holding its
/// forecast completes it, one byte less refuses with the meter's cumulative
/// amounts, and every insertion navigates exactly as an ordered edit does.
#[test]
fn a_fresh_build_admits_its_own_growth_and_ordered_navigation() {
    let entries = 200;
    let whole = index_capacity::insertion_sequence_bytes::<usize, ()>(entries).unwrap();
    let build = |bytes| {
        let mut meter = InvalidationEditAdmission::new(CompanionPreflightBudget {
            maximum_work_visits: u64::MAX,
            maximum_preparation_bytes: bytes,
        });
        let built = (0..entries).try_for_each(|length| meter.fresh_insert::<usize, ()>(length));
        (built, meter.charged_bytes())
    };
    assert_eq!(build(whole), (Ok(()), whole));
    assert_eq!(
        build(whole - 1).0,
        Err(CompanionPreflightStop::PreparationMemoryExhausted {
            required: whole,
            maximum: whole - 1,
        })
    );
    let (mut fresh, mut edited) = (WorkMeter::default(), WorkMeter::default());
    for length in 0..entries {
        fresh.fresh_insert::<usize, ()>(length).unwrap();
        edited.ordered_edit::<usize, ()>(length).unwrap();
    }
    assert_eq!(fresh.0, edited.0);
}
