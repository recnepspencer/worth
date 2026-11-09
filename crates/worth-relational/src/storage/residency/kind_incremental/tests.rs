//! Compare the ordinary kind update with reconstruction from storage metadata.
use super::*;
use std::sync::Arc;
use worth_testkit_relational::maintenance::MaintainedCourt;
mod cost;
mod fixture;

fn cold(partition: &PartitionState) -> RegionKindIndexes {
    let mut rebuilt = partition.clone();
    crate::storage::residency::rebuild_region_history_kind_buckets(
        &crate::storage::substrate::maintenance::ReconstructionLane::oracle(),
        &mut rebuilt,
    )
    .unwrap();
    RegionKindIndexes::capture(
        &crate::storage::substrate::maintenance::ReconstructionLane::oracle(),
        &rebuilt,
    )
    .unwrap()
}

#[test]
fn mixed_retirement_restoration_and_history_preserve_cold_commitment_bytes() {
    let host = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    for seed in 0..8 {
        let mut partition = fixture::partition(7, &execution);
        let mut court = MaintainedCourt::new("kind commitment", Arc::new(cold(&partition)));
        for step in 0..40 {
            let previous = partition.clone();
            let journal = fixture::edit(&mut partition, (step + seed) % 7, step + seed, &execution);
            let updated = update_value(
                &mut partition,
                &previous,
                court.previous(),
                &journal,
                &mut |_, _| Ok(()),
                &mut |_, _| Ok(None),
            )
            .unwrap();
            court
                .check_against_cold::<Cause>(
                    Arc::new(updated),
                    Arc::new(cold(&partition)),
                    |a, b| a.commitment() == b.commitment(),
                )
                .unwrap();
        }
    }
}

#[test]
fn independent_reconstruction_detects_a_dropped_kind_update() {
    let host = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let mut partition = fixture::partition(7, &execution);
    let prior = Arc::new(cold(&partition));
    fixture::edit(&mut partition, 0, 2, &execution);
    let mut court = MaintainedCourt::new("kind commitment", prior.clone());
    let result = court.check_against_cold::<Cause>(prior, Arc::new(cold(&partition)), |a, b| {
        a.commitment() == b.commitment()
    });
    assert!(
        result.is_err(),
        "retaining the predecessor must not authenticate a changed kind set"
    );
}

#[test]
fn replacing_and_suspending_history_remove_only_unsupported_incidence() {
    let host = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let mut partition = fixture::partition(7, &execution);
    let retained = partition.clone();
    for step in 0..20 {
        fixture::edit(&mut partition, 0, step, &execution);
    }
    let before_suspension = partition.clone();
    let mut court = MaintainedCourt::new("kind history replacement", Arc::new(cold(&partition)));
    for action in 0..4 {
        let previous = partition.clone();
        let journal = match action {
            0 => fixture::restore(&mut partition, &retained, 0),
            1 => fixture::suspend(&mut partition, 0),
            2 => fixture::restore(&mut partition, &before_suspension, 0),
            _ => fixture::edit(&mut partition, 0, 25, &execution),
        };
        let updated = update_value(
            &mut partition,
            &previous,
            court.previous(),
            &journal,
            &mut |_, _| Ok(()),
            &mut |_, _| Ok(None),
        )
        .unwrap();
        court
            .check_against_cold::<Cause>(Arc::new(updated), Arc::new(cold(&partition)), |a, b| {
                a.commitment() == b.commitment()
            })
            .unwrap();
    }
}
