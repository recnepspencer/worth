//! A foreign issued schedule cannot publish an entry; the original schedule can.
use super::super::SceneLabel;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationQueryBatchAdmission as Batch,
    WorthQueryManagedDerivedCollectionBatchRefreshDenial as Denial,
    WorthQueryManagedDerivedViewDenial as ViewDenial, WorthQueryManagedDerivedViewKey,
    WorthQueryManagedDerivedViewSnapshot,
};
use std::{cell::Cell, sync::Arc};
pub(super) fn refuse_then_refresh(
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
