use super::{
    PersistentVector, RetainedVectorCapacityDenial, RetainedVectorCapacityOutcome,
    RetainedVectorMutationDenial, RetainedVectorStagingDenial,
};
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Work, RetainedStoragePreparationDenial,
};

fn prepared_fork(count: usize) -> PersistentVector<String> {
    let mut values: PersistentVector<_> = (0..count).map(|_| "held".to_string()).collect();
    values
        .prepare_retained_charge(&mut Work::new(count + 10))
        .unwrap();
    values.fork_persistent()
}

#[test]
fn retained_edit_preconditions_include_logical_extent_and_charge_readiness() {
    let original = prepared_fork(64);
    let mut changed_extent = original.clone();
    // Representation-level adversary: hide a retained base tail while keeping
    // both immutable roots and their still-correct physical charge unchanged.
    let super::PersistentVectorStorage::ForkShared { len, .. } = &mut changed_extent.storage else {
        unreachable!()
    };
    *len -= 1;
    assert!(original.shares_storage_with(&changed_extent));
    assert_eq!(
        original.prepared_retained_charge(),
        changed_extent.prepared_retained_charge()
    );
    assert!(!original.matches_retained_edit_preconditions(&changed_extent));
    let mut unaccounted = original.clone();
    assert!(unaccounted.get_mut(usize::MAX).is_none());
    assert!(original.shares_storage_with(&unaccounted));
    assert_eq!(original.len(), unaccounted.len());
    assert!(!original.matches_retained_edit_preconditions(&unaccounted));
    assert!(original.matches_retained_edit_preconditions(&original.clone()));
}

#[test]
fn staged_capacity_edit_has_exact_bound_and_preserves_rejected_roots() {
    let mut values = prepared_fork(64);
    let sibling = values.clone();
    let before = values.prepared_retained_charge().unwrap();
    let mut independently_edited = values.clone();
    independently_edited[0] = "replacement".repeat(1_024);
    let expected = independently_edited
        .retained_heap_charge(&mut Work::new(10_000))
        .unwrap();
    let short = expected
        .checked_sub(Charge::capacity::<u8>(1).unwrap())
        .unwrap();
    let rejected = values
        .edit_with_retained_capacity(0, short, &mut Work::new(10_000), |value| {
            *value = "replacement".repeat(1_024);
            7
        })
        .unwrap();
    assert!(
        matches!(rejected, RetainedVectorCapacityOutcome::Rejected { output: 7,
        denial: RetainedVectorCapacityDenial::CapacityExhausted { maximum, required } }
        if maximum == short && required == expected)
    );
    assert_eq!(values[0], "held");
    assert_eq!(values.prepared_retained_charge().unwrap(), before);
    assert!(values.shares_storage_with(&sibling));
    let installed = values
        .edit_with_retained_capacity(0, expected, &mut Work::new(10_000), |value| {
            *value = "replacement".repeat(1_024);
            8
        })
        .unwrap();
    assert!(
        matches!(installed, RetainedVectorCapacityOutcome::Installed { output: 8, charge } if charge == expected)
    );
    assert_eq!(values[0], independently_edited[0]);
    assert_eq!(sibling[0], "held");
    assert_eq!(values.prepared_retained_charge().unwrap(), expected);
}

#[test]
fn staging_denies_before_edit_without_prepared_forked_storage() {
    let maximum = Charge::capacity::<u8>(1_000_000).unwrap();
    let mut values: PersistentVector<_> = ["held".to_string()].into_iter().collect();
    values.prepare_retained_charge(&mut Work::new(100)).unwrap();
    assert!(matches!(
        values.edit_with_retained_capacity(0, maximum, &mut Work::new(100), |_| panic!(
            "exclusive edit ran"
        )),
        Err(RetainedVectorStagingDenial::ForkPreparationRequired)
    ));
    let mut fork = values.fork_persistent();
    fork[0].push_str(" changed");
    assert!(matches!(
        fork.edit_with_retained_capacity(0, maximum, &mut Work::new(100), |_| panic!(
            "unprepared edit ran"
        )),
        Err(RetainedVectorStagingDenial::Mutation(
            RetainedVectorMutationDenial::PreparationRequired
        ))
    ));
}

#[test]
fn staged_edit_work_scales_with_page_and_failure_preserves_original_storage() {
    let maximum = Charge::capacity::<u8>(10_000_000).unwrap();
    let mut visits = Vec::new();
    for count in [64, 4_096] {
        let mut original = prepared_fork(count);
        let before = original.prepared_retained_charge().unwrap();
        let sibling = original.clone();
        let mut sample = original.clone();
        let mut work = Work::new(1_000);
        let installed = sample
            .edit_with_retained_capacity(0, maximum, &mut work, |value| value.push_str(" updated"))
            .unwrap();
        assert!(matches!(
            installed,
            RetainedVectorCapacityOutcome::Installed { .. }
        ));
        visits.push(work.visits());
        let mut ran = false;
        let result = original
            .edit_with_retained_capacity(0, maximum, &mut Work::new(work.visits() - 1), |value| {
                value.push_str(" updated");
                ran = true;
                9
            })
            .unwrap();
        assert!(ran);
        assert!(matches!(
            result,
            RetainedVectorCapacityOutcome::Rejected {
                output: 9,
                denial: RetainedVectorCapacityDenial::Accounting(
                    RetainedStoragePreparationDenial::WorkExhausted { .. }
                )
            }
        ));
        assert_eq!(original.prepared_retained_charge().unwrap(), before);
        assert!(original.shares_storage_with(&sibling));
        let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ =
                original.edit_with_retained_capacity(0, maximum, &mut Work::new(1_000), |value| {
                    value.push_str(" panic edit");
                    panic!("provider unwind");
                });
        }));
        assert!(panic.is_err());
        assert_eq!(original[0], "held");
        assert_eq!(original.prepared_retained_charge().unwrap(), before);
        assert!(original.shares_storage_with(&sibling));
    }
    assert_eq!(visits[0], visits[1]);
}
