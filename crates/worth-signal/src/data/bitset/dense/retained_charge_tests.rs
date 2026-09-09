use std::collections::BTreeSet;

use super::DenseBitset;
use crate::data::persistent_vector::RetainedVectorMutationDenial;
use crate::data::retained_storage::{
    RetainedStorageMeasurement, RetainedStoragePreparation as Preparation,
};

fn assert_charge(bits: &DenseBitset) {
    assert_eq!(
        bits.prepared_retained_charge().unwrap(),
        bits.retained_heap_charge(&mut Preparation::new(100_000))
            .unwrap()
    );
}

#[test]
fn ordinary_bit_mutations_carry_charges_through_growth_fork_and_clear() {
    let mut bits = DenseBitset::new();
    let mut model = BTreeSet::new();
    bits.ensure_len(8192);
    assert_charge(&bits);
    for index in [0, 63, 64, 4095, 4096, 8191, 16384] {
        assert_eq!(bits.mark(index), model.insert(index));
        assert_charge(&bits);
    }
    let pinned = bits.fork_persistent();
    let pinned_model = model.clone();
    for index in [0, 64, 4096, 8191] {
        bits.clear(index);
        model.remove(&index);
        assert_charge(&bits);
        assert_charge(&pinned);
    }
    for index in [1, 65, 4100, 20000] {
        assert_eq!(bits.mark(index), model.insert(index));
        assert_charge(&bits);
    }
    assert_eq!(bits.marked_indices(), model.into_iter().collect::<Vec<_>>());
    assert_eq!(
        pinned.marked_indices(),
        pinned_model.into_iter().collect::<Vec<_>>()
    );
    bits.clear_all();
    assert!(bits.marked_indices().is_empty());
    assert!(!pinned.marked_indices().is_empty());
    assert_charge(&bits);
    bits.merge(&pinned);
    assert_eq!(bits.marked_indices(), pinned.marked_indices());
    assert_charge(&bits);
    assert_charge(&pinned);
}

#[test]
fn prepared_word_mutation_stays_ready_and_unprepared_mutation_never_scans_to_repair() {
    for word_count in [1, 8, 64] {
        let mut bits = DenseBitset::new();
        bits.ensure_len(word_count * 64);
        let pinned = bits.fork_persistent();
        bits.mark(0);
        bits.clear(0);
        bits.prepare_retained_charge(&mut Preparation::new(0))
            .unwrap();
        assert_charge(&bits);
        assert_charge(&pinned);
        let mut unprepared = bits.operational_clone();
        unprepared.mark(1);
        assert!(unprepared.contains(1));
        assert_eq!(
            unprepared.prepared_retained_charge(),
            Err(RetainedVectorMutationDenial::PreparationRequired)
        );
        assert!(unprepared
            .prepare_retained_charge(&mut Preparation::new(0))
            .is_err());
        unprepared
            .prepare_retained_charge(&mut Preparation::new(1000))
            .unwrap();
        assert_charge(&unprepared);
    }
}
