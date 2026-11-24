//! Compare the ordinary kind update with reconstruction from storage metadata.
use super::*;
pub mod cost;
pub mod fixture;

fn cold(partition: &PartitionState) -> RegionKindIndexes {
    crate::storage::residency::resident_history::with_reconstruction_fixture(|fixture_lane| {
        let mut rebuilt = partition.clone();
        crate::storage::residency::rebuild_region_history_kind_buckets(fixture_lane, &mut rebuilt)
            .unwrap();
        RegionKindIndexes::capture(fixture_lane, &rebuilt).unwrap()
    })
}

#[test]
fn mixed_retirement_restoration_and_history_preserve_cold_commitment_bytes() {
    for seed in 0..8 {
        let mut partition = fixture::partition(7);
        let mut prior = cold(&partition);
        for step in 0..40 {
            let previous = partition.clone();
            let journal = fixture::edit(&mut partition, (step + seed) % 7, step + seed);
            let updated = update_value(
                &mut partition,
                &previous,
                &prior,
                &journal,
                &mut |_, _| Ok(()),
                &mut |_, _| Ok(None),
            )
            .unwrap();
            assert_eq!(updated.commitment(), cold(&partition).commitment());
            prior = updated;
        }
    }
}

#[test]
fn independent_reconstruction_detects_a_dropped_kind_update() {
    let mut partition = fixture::partition(7);
    let prior = cold(&partition);
    fixture::edit(&mut partition, 0, 2);
    assert_ne!(
        prior.commitment(),
        cold(&partition).commitment(),
        "retaining the predecessor must not authenticate a changed kind set"
    );
}

#[test]
fn replacing_and_suspending_history_remove_only_unsupported_incidence() {
    let mut partition = fixture::partition(7);
    let retained = partition.clone();
    for step in 0..20 {
        fixture::edit(&mut partition, 0, step);
    }
    let before_suspension = partition.clone();
    let mut prior = cold(&partition);
    for action in 0..4 {
        let previous = partition.clone();
        let journal = match action {
            0 => fixture::restore(&mut partition, &retained, 0),
            1 => fixture::suspend(&mut partition, 0),
            2 => fixture::restore(&mut partition, &before_suspension, 0),
            _ => fixture::edit(&mut partition, 0, 25),
        };
        let updated = update_value(
            &mut partition,
            &previous,
            &prior,
            &journal,
            &mut |_, _| Ok(()),
            &mut |_, _| Ok(None),
        )
        .unwrap();
        assert_eq!(updated.commitment(), cold(&partition).commitment());
        prior = updated;
    }
}
