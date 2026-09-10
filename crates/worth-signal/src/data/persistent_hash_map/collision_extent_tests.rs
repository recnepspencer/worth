use std::hash::{Hash, Hasher};

use super::{PersistentHashMap, PersistentHashMapStorage};

#[derive(Clone, Debug, PartialEq, Eq)]
struct Colliding(u32);

impl Hash for Colliding {
    fn hash<H: Hasher>(&self, state: &mut H) {
        0u8.hash(state);
    }
}

fn counts(map: &PersistentHashMap<Colliding, String>) -> (usize, usize, usize) {
    match &map.storage {
        PersistentHashMapStorage::ForkShared {
            collision_extents, ..
        } => collision_extents
            .as_ref()
            .expect("mutation preserved extent history")
            .counts(),
        _ => panic!("test requires an actual persistent overlay"),
    }
}

#[test]
fn collision_extent_survives_shrink_and_fork_then_releases_on_collapse() {
    let mut original = PersistentHashMap::new();
    let mut map = original.fork_persistent();
    for key in 0..64 {
        map.insert(Colliding(key), format!("value-{key}"));
    }
    assert_eq!(counts(&map), (1, 1, 128));
    let all = map.clone();
    for key in 2..64 {
        assert!(map.remove(&Colliding(key)).is_some());
    }
    assert_eq!(map.len(), 2);
    assert_eq!(counts(&map), (1, 1, 128));
    let two = map.fork_persistent();
    map.get_mut(&Colliding(0)).unwrap().push_str(" changed");
    assert_eq!(counts(&map), (1, 1, 128));
    map.remove(&Colliding(1));
    assert_eq!(counts(&map), (1, 0, 0));
    assert_eq!(counts(&two), (1, 1, 128));
    assert_eq!(counts(&all), (1, 1, 128));
    assert_eq!(two.get(&Colliding(0)).unwrap(), "value-0");
    assert_eq!(all.len(), 64);
    for key in 64..128 {
        map.insert(Colliding(key), "temporary".into());
        assert_eq!(counts(&map), (1, 1, 2));
        map.remove(&Colliding(key));
        assert_eq!(counts(&map), (1, 0, 0));
    }
    map.remove(&Colliding(0));
    assert_eq!(counts(&map), (0, 0, 0));
}

#[test]
fn inherited_tombstones_keep_occupying_collision_storage_and_charge_its_metadata() {
    let mut base: PersistentHashMap<_, _> = (0..8)
        .map(|key| (Colliding(key), "base".to_owned()))
        .collect();
    let mut map = base.fork_persistent();
    for key in 0..8 {
        map.get_mut(&Colliding(key)).unwrap().push_str(" changed");
    }
    assert_eq!(counts(&map), (1, 1, 16));
    for key in 0..8 {
        map.remove(&Colliding(key));
    }
    assert!(map.is_empty());
    assert_eq!(counts(&map), (1, 1, 16));
    let before = match &map.storage {
        PersistentHashMapStorage::ForkShared {
            collision_extents, ..
        } => collision_extents
            .as_ref()
            .unwrap()
            .retained_structure_charge::<Colliding, String>()
            .unwrap(),
        _ => unreachable!(),
    };
    // Readmission replaces an existing tombstone, rather than adding an entry.
    map.entry(Colliding(3)).or_default().push_str("readmitted");
    assert_eq!(counts(&map), (1, 1, 16));
    let after = match &map.storage {
        PersistentHashMapStorage::ForkShared {
            collision_extents, ..
        } => collision_extents
            .as_ref()
            .unwrap()
            .retained_structure_charge::<Colliding, String>()
            .unwrap(),
        _ => unreachable!(),
    };
    assert_eq!(before, after);
    assert!(before.bytes() > 8 * std::mem::size_of::<usize>() as u64);
    assert_eq!(base.get(&Colliding(3)).unwrap(), "base");
    map.clear();
    let empty = map.fork_persistent();
    assert_eq!(counts(&empty), (0, 0, 0));
}
