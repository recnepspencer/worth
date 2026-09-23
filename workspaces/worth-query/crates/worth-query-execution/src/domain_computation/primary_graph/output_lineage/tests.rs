use std::collections::BTreeMap;
use std::sync::Arc;

use super::{
    latest_output_in_partition, latest_output_in_partition_budgeted, RecordedOutput,
    WorthQueryApplicationOutputCorrespondence,
};

#[test]
fn prior_output_selection_stays_with_its_parameter_partition() {
    let first_partition = [0x11; 32];
    let sibling_partition = [0x22; 32];
    let first = Arc::new(WorthQueryApplicationOutputCorrespondence::default());
    let sibling = Arc::new(WorthQueryApplicationOutputCorrespondence::default());
    let revised_first = Arc::new(WorthQueryApplicationOutputCorrespondence::default());
    let mut history = BTreeMap::new();
    history.insert(1, record(Arc::clone(&first), first_partition));
    history.insert(2, record(Arc::clone(&sibling), sibling_partition));
    history.insert(3, record(Arc::clone(&revised_first), first_partition));

    let (_, selected_first_before_revision) =
        latest_output_in_partition(&history, 2, first_partition).unwrap();
    let (_, selected_sibling) = latest_output_in_partition(&history, 3, sibling_partition).unwrap();
    let (_, selected_first_after_revision) =
        latest_output_in_partition(&history, 3, first_partition).unwrap();

    assert!(Arc::ptr_eq(
        &selected_first_before_revision.correspondence,
        &first
    ));
    assert!(Arc::ptr_eq(&selected_sibling.correspondence, &sibling));
    assert!(Arc::ptr_eq(
        &selected_first_after_revision.correspondence,
        &revised_first
    ));
}

#[test]
fn partition_lookup_charges_every_examined_sibling_record() {
    let target = [0x11; 32];
    let sibling = [0x22; 32];
    let correspondence = Arc::new(WorthQueryApplicationOutputCorrespondence::default());
    let mut history = BTreeMap::new();
    history.insert(1, record(Arc::clone(&correspondence), target));
    for generation in 2..=4 {
        history.insert(generation, record(Arc::clone(&correspondence), sibling));
    }

    assert!(latest_output_in_partition_budgeted(&history, 4, target, 3).is_err());
    let (selected, work) = latest_output_in_partition_budgeted(&history, 4, target, 4)
        .expect("the exact record count fits the budget");
    assert_eq!(selected.map(|(generation, _)| *generation), Some(1));
    assert_eq!(work, 4);
    assert!(latest_output_in_partition_budgeted(&history, 0, target, 0).is_err());
    assert_eq!(
        latest_output_in_partition_budgeted(&history, 0, target, 1)
            .expect("an empty coordinate costs one lookup")
            .1,
        1,
    );
}

fn record(
    correspondence: Arc<WorthQueryApplicationOutputCorrespondence>,
    source_partition_identity: [u8; 32],
) -> RecordedOutput {
    RecordedOutput {
        correspondence,
        source_identity: Some([0x33; 32]),
        source_partition_identity: Some(source_partition_identity),
        producer_dependency_identity: None,
        idempotency_key_identity: [0x44; 32],
        observed_source_facts: Arc::from([]),
    }
}
