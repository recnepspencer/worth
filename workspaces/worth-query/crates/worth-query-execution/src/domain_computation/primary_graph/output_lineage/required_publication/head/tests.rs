//! Factory-established ordinary heads differ from unresolved aliases and partial native claims.
use super::*;

#[test]
fn restored_unattested_head_is_selectable_but_cannot_supply_a_cold_predecessor() {
    let selected = restored_cell();
    let head = SelectedRequiredPublicationHead::retain(&selected).unwrap();
    assert!(head.warm_publication().is_none());
    let mut admission =
        InvalidationEditAdmission::new(worth_relational::facade::mvcc::CompanionPreflightBudget {
            maximum_work_visits: Some(1_000),
            maximum_preparation_bytes: Some(1_000),
        });
    assert!(matches!(
        head.cold_predecessor(&mut admission),
        Err(Stop::SelectedSourceMismatch)
    ));
}

#[test]
fn uninitialized_performed_alias_refuses_instead_of_becoming_an_unattested_head() {
    let mut row = restored_row();
    row.performed_origin = Some(Arc::new(OnceLock::new()));
    let selected = Arc::new(OnceLock::from(row));
    assert!(matches!(
        SelectedRequiredPublicationHead::retain(&selected),
        Err(Stop::SelectedSourceMismatch)
    ));
}

#[test]
fn incomplete_native_predecessor_description_is_not_an_ordinary_factory_head() {
    let mut row = restored_row();
    // Deliberately incomplete metadata is the invalid input, not an issued authority.
    row.native_prior_checkpoint = Some(
        super::super::super::native_prior_checkpoint::NativePriorCheckpointLocator::Performed {
            producer: "incomplete-native-claim".to_owned(),
            source: [0; 32],
        },
    );
    assert!(row.producer_dependency_identity.is_none());
    let selected = Arc::new(OnceLock::from(row));
    assert!(matches!(
        SelectedRequiredPublicationHead::retain(&selected),
        Err(Stop::SelectedSourceMismatch)
    ));
}

fn restored_cell() -> Arc<OnceLock<RecordedOutput>> {
    let (lineage, _) = super::super::super::registry_fixture::recorded_settlement();
    lineage
        .by_source
        .values()
        .next()
        .unwrap()
        .values()
        .next()
        .unwrap()
        .values()
        .next()
        .unwrap()
        .last()
        .cloned()
        .unwrap()
}

fn restored_row() -> RecordedOutput {
    match Arc::try_unwrap(restored_cell()) {
        Ok(cell) => cell.into_inner().unwrap(),
        Err(_) => panic!("the restored fixture has one record owner"),
    }
}
