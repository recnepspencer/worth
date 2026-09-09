use super::PersistentOrdSet;
use crate::data::persistent_ord_map::{
    RetainedMapMutationDenial as MutationDenial, RetainedMapMutationOutcome as Outcome,
};
use crate::data::retained_storage::{
    RetainedStorageMeasurement, RetainedStoragePreparation as Preparation,
};

fn work() -> Preparation {
    Preparation::new(100_000)
}
fn changed(outcome: Result<Outcome<bool>, MutationDenial>) -> bool {
    match outcome.unwrap() {
        Outcome::Accounted { output, .. } => output,
        Outcome::Unaccounted { denial, .. } => panic!("unexpected failure: {denial:?}"),
    }
}

#[test]
fn policy_identity_membership_and_pinned_charge_follow_the_existing_map_owner() {
    let mut identities = PersistentOrdSet::new();
    for byte in 0..16 {
        assert!(changed(
            identities.insert_with_retained_charge([byte; 32], &mut work())
        ));
    }
    let pinned = identities.fork_persistent();
    for byte in [0, 2, 1, 4, 3] {
        assert!(changed(
            identities.remove_with_retained_charge(&[byte; 32], &mut work())
        ));
        assert!(!identities.contains(&[byte; 32]));
        assert!(pinned.contains(&[byte; 32]));
        assert_eq!(
            identities.prepared_retained_charge().unwrap(),
            identities.retained_heap_charge(&mut work()).unwrap()
        );
    }
    assert!(changed(
        identities.insert_with_retained_charge([2; 32], &mut work())
    ));
    assert!(!changed(
        identities.insert_with_retained_charge([2; 32], &mut work())
    ));
    identities
        .prepare_retained_charge(&mut Preparation::new(0))
        .unwrap();
    assert_eq!(
        pinned.prepared_retained_charge().unwrap(),
        pinned.retained_heap_charge(&mut work()).unwrap()
    );
}

#[test]
fn membership_outcome_preserves_post_edit_failure_and_array_payload_preparation() {
    let mut set = PersistentOrdSet::<String>::new();
    match set
        .insert_with_retained_charge("inserted".into(), &mut Preparation::new(1))
        .unwrap()
    {
        Outcome::Unaccounted { output, .. } => assert!(output),
        other => panic!("unexpected outcome: {other:?}"),
    }
    assert!(set.contains("inserted"));
    assert_eq!(
        set.prepared_retained_charge(),
        Err(MutationDenial::PreparationRequired)
    );
    assert!(matches!(
        set.remove_with_retained_charge(&"inserted".into(), &mut work()),
        Err(MutationDenial::PreparationRequired)
    ));
    set.prepare_retained_charge(&mut work()).unwrap();
    assert!(changed(
        set.remove_with_retained_charge(&"inserted".into(), &mut work())
    ));
    let payload = [String::with_capacity(4096), String::with_capacity(8192)];
    assert!(payload
        .retained_heap_charge(&mut Preparation::new(2))
        .is_err());
    assert_eq!(
        payload
            .retained_heap_charge(&mut Preparation::new(3))
            .unwrap()
            .bytes(),
        12288
    );
}
