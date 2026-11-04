//! A foreign issued schedule cannot publish an entry; the original schedule can.
use crate::domain_computation::primary_graph::tests::application_query::derived_view::SceneLabel;
use std::cell::Cell;
use std::sync::Arc;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryBatchAdmission as Batch;
use worth_query_execution::facade::domain_computation::WorthQueryManagedDerivedCollectionBatchRefreshDenial as Denial;
use worth_query_execution::facade::domain_computation::WorthQueryManagedDerivedViewDenial as ViewDenial;
use worth_query_execution::facade::domain_computation::WorthQueryManagedDerivedViewKey;
use worth_query_execution::facade::domain_computation::WorthQueryManagedDerivedViewSnapshot;
pub fn refuse_then_refresh(
    mut issue: impl FnMut() -> Batch,
    projections: &Cell<usize>,
    snapshot: &WorthQueryManagedDerivedViewSnapshot<SceneLabel>,
    key: &WorthQueryManagedDerivedViewKey,
    mut refresh: impl FnMut(&Batch, &Batch, &Batch) -> Result<Arc<SceneLabel>, Denial>,
) {
    let original = issue();
    let foreign = issue();
    assert!(matches!(
        refresh(&foreign, &original, &original),
        Err(Denial::ForeignBatch)
    ));
    assert_eq!(projections.get(), 0);
    assert_eq!(
        snapshot.get(key).err(),
        Some(ViewDenial::EntryRefreshRequired)
    );
    // The first foreign attempt consumed its own read, not any original slot.
    let original = issue();
    let foreign = issue();
    assert!(matches!(
        refresh(&original, &original, &foreign),
        Err(Denial::ForeignBatch)
    ));
    assert_eq!(projections.get(), 0);
    assert_eq!(
        snapshot.get(key).err(),
        Some(ViewDenial::EntryRefreshRequired)
    );
    let complete = issue();
    let refreshed = refresh(&complete, &complete, &complete).unwrap();
    assert_eq!(refreshed.0, "primary-changed");
    assert_eq!(projections.get(), 1);
    assert!(Arc::ptr_eq(
        &refreshed,
        &snapshot.get(key).unwrap().unwrap()
    ));
}
