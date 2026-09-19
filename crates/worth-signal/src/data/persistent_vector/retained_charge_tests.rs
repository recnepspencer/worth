use super::PersistentVector;
use crate::data::retained_storage::{
    RetainedStorageMeasurement, RetainedStoragePreparation, RetainedStoragePreparationDenial,
};

#[test]
fn charge_overflow_cannot_become_a_small_admissible_value() {
    use crate::data::retained_storage::RetainedStorageCharge;
    assert_eq!(
        RetainedStorageCharge::capacity::<u64>(usize::MAX),
        Err(RetainedStoragePreparationDenial::ChargeOverflow)
    );
    let mut large = RetainedStorageCharge::capacity::<u8>(1).unwrap();
    for _ in 0..63 {
        large = large.checked_mul(2).unwrap();
    }
    assert_eq!(
        large.checked_mul(2),
        Err(RetainedStoragePreparationDenial::ChargeOverflow)
    );
    assert_eq!(
        large.checked_add(large),
        Err(RetainedStoragePreparationDenial::ChargeOverflow)
    );
}

fn charge<T: RetainedStorageMeasurement>(value: &T) -> u64 {
    value
        .retained_heap_charge(&mut RetainedStoragePreparation::new(1000))
        .unwrap()
        .bytes()
}

#[test]
fn retained_charge_sees_hidden_base_payloads_after_override_and_removal() {
    let mut original = String::with_capacity(32_768);
    original.push_str("large retained allocation");
    let original_capacity = original.capacity() as u64;
    let mut parent: PersistentVector<String> = [original].into_iter().collect();
    let mut child = parent.fork_persistent();
    child[0] = String::from("small");
    drop(parent);
    assert!(
        charge(&child) >= original_capacity,
        "overridden base payload remains retained even with no parent holder"
    );
    assert_eq!(child.pop_back().as_deref(), Some("small"));
    assert!(child.is_empty());
    assert!(
        charge(&child) >= original_capacity,
        "logical emptiness must not erase the retained base allocation"
    );
    child.clear();
    assert_eq!(charge(&child), 0);
}

#[test]
fn preparation_bounds_nested_work_and_never_returns_a_partial_charge() {
    let values: PersistentVector<Vec<String>> =
        [vec![String::from("payload")]].into_iter().collect();
    let mut exact = RetainedStoragePreparation::new(4);
    assert!(values.retained_heap_charge(&mut exact).is_ok());
    assert_eq!(exact.visits(), 4);
    let mut insufficient = RetainedStoragePreparation::new(3);
    assert_eq!(
        values.retained_heap_charge(&mut insufficient),
        Err(RetainedStoragePreparationDenial::WorkExhausted { maximum_visits: 3 })
    );
    assert_eq!(insufficient.visits(), 3);
    assert!(values.retained_heap_charge(&mut insufficient).is_err());
    assert_eq!(insufficient.visits(), 3);
}

#[test]
fn retained_charge_counts_nested_capacity_and_append_allocations() {
    let mut empty = PersistentVector::<Vec<String>>::new();
    let mut shared = empty.fork_persistent();
    let baseline = charge(&shared);
    let mut nested = Vec::with_capacity(17);
    nested.push(String::with_capacity(4_096));
    let nested_minimum =
        (nested.capacity() * std::mem::size_of::<String>() + nested[0].capacity()) as u64;
    shared.push_back(nested);
    assert!(charge(&shared) >= baseline + nested_minimum);
    assert_eq!(charge(&empty), baseline);
}
